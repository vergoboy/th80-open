# Phase 11.1 — Repository Reliability Closure

## 1. Executive summary

Phase 11.1 closes the code-level gaps around Windows replacement, cross-process coordination and post-replacement recovery. The repository now uses a narrowly scoped Windows `ReplaceFileW` implementation, records lock-owner PIDs, tests real child-process races, and has deterministic failure injection on both sides of the replacement commit point. A child-process test exits immediately after replacement visibility and verifies that the new complete profile remains authoritative while the abandoned lock fails closed.

Linux tests were executed natively in this environment. Windows core/CLI test targets compile for `x86_64-pc-windows-msvc`, but Windows filesystem tests were not executed natively here. macOS was not available. A native GitHub Actions matrix for Ubuntu, Windows and macOS was added; until those jobs actually run, Windows is classified **COMPILES ONLY** and macOS **UNVERIFIED**. Therefore the strict acceptance criteria requiring native Windows and macOS execution are not yet fully satisfied.

No dependency was added. No historical Phase 5–11 report was edited. No commit was made. No hardware communication occurred.

## 2. Initial repository reliability findings

Inspection confirmed the Phase 11 report's key limitations were still present:

- Existing-profile updates used `std::fs::rename`; on Windows this does not replace an existing destination and failed safely.
- The lock used `create_new`, so exclusion was cross-process in principle, but no child-process tests exercised it and lock files did not record an owner PID.
- Normal `Drop` removed locks; abrupt termination left a lock behind. There was no automatic stale-lock breaking, which is retained as the safe policy.
- Failure injection stopped before replacement. There was no test of the state after the replacement became visible.
- Unix parent-directory sync was best-effort and errors were swallowed.
- No CI workflow existed for native Windows or macOS testing.

The timestamp-based revision check remains in place: after a writer commits, `modified_at_unix_ms` is advanced beyond the prior revision; a second writer holding an older loaded profile receives a conflict after obtaining the lock.

## 3. Windows replacement implementation and verification

`crates/th80-core/src/repository.rs` now dispatches replacement by platform:

- Unix: same-directory `std::fs::rename` over the existing destination.
- Windows: native `ReplaceFileW`, with the prepared source and destination in the same directory/volume.
- Other platforms: explicit `Unsupported` error rather than an unsafe fallback.

The temporary file is created exclusively, fully serialized, written, assigned the destination’s portable permission bits, `sync_all`-ed, and closed before replacement. `ReplaceFileW` is called once. There is no delete-destination-then-rename fallback. Failure is returned as a structured `CoreError::Io`; the original destination is not intentionally removed first. A reader handle that does not grant delete sharing can block replacement; this is a safe failure mode.

A Windows-only native test, `windows_replace_fails_safely_when_destination_denies_delete_sharing`, holds the destination without `FILE_SHARE_DELETE`, verifies the replacement fails and original bytes remain, then retries successfully after releasing the handle. The general replacement and open-reader tests also run on Windows CI.

**Verification in this session:** `cargo check --locked --target x86_64-pc-windows-msvc --tests -p th80-core -p th80-cli` passed. This is compile-only evidence, not native Windows filesystem verification. Native Windows CI has been configured but has not executed within this session.

The Windows FFI is isolated in one module with a narrowly scoped `#[allow(unsafe_code)]`; the crate retains `#![deny(unsafe_code)]` everywhere else. The FFI call uses NUL-terminated UTF-16 buffers valid for the duration of the call and propagates `GetLastError` through `last_os_error`.

## 4. Cross-process concurrency policy

The per-profile lock remains a same-directory hidden lock file created with `create_new`. It now records and synchronizes `pid=<process id>` before the mutation proceeds. Different profile IDs use different locks; there is no global repository lock.

Same-profile behavior:

1. The first writer to create the lock proceeds.
2. A concurrent writer receives `CoreError::Busy` while the lock exists.
3. A writer that loaded an earlier revision but obtains the lock after another writer commits receives `CoreError::Conflict`.
4. No last-writer-wins update is allowed.

Creation and import use the same per-ID lock and no-clobber installation. Readers read the current destination file; replacement exposes a complete old or new file on supported local filesystems. Open-reader tests retain the old file handle while a replacement becomes visible and validate both the old handle's bytes and the new repository contents.

Case-only profile IDs are tested. Case-sensitive filesystems may store both; case-insensitive filesystems must reject the collision without overwriting the first profile.

## 5. Lock lifecycle and stale-lock behavior

Normal completion removes the lock file through `MutationLock::drop`. Abrupt process termination bypasses `Drop` and leaves the lock file. The owner PID is written as plain text to support operator inspection.

The repository never breaks a lock based on age, never assumes a PID is dead from file age, and does not automatically remove a lock. Recovery remains explicit:

1. Inspect `.PROFILE_ID.lock` and read `pid=...`.
2. Confirm that process is no longer running and that no other writer is active.
3. Remove only that lock file.
4. Reload the profile before editing or retrying a save.

If ownership cannot be established, leave the lock in place and fail closed. There is no automatic lock-recovery command. A lock left after a crash immediately following replacement does not mean the profile is incomplete; reload the profile after confirming the process is gone.

## 6. Crash-recovery design

No journal or backup file was added. The replacement operation itself is the commit/visibility boundary:

- Before replacement: the original profile is authoritative. A process killed during preparation may leave a hidden temporary file. It is ignored by listing and is never auto-promoted.
- After replacement becomes visible: the new complete profile is authoritative. The temporary path has been consumed by replacement. A process killed at this point may leave the lock file.
- After Unix directory synchronization: the parent directory entry has been synchronized to the extent supported by the filesystem. This is not a universal power-loss guarantee.
- Windows: the temporary file is flushed before `ReplaceFileW`; directory-entry durability is not claimed because a portable directory flush is not exposed by this implementation.

On Unix, parent-directory sync errors are now propagated. Such an error can occur after the new file is already visible; callers must reload before retrying rather than assuming the write was rolled back. This distinction is documented in the core API and CLI docs.

No automatic rollback is attempted. Atomic replacement leaves an old-or-new complete primary file; a journal-driven guess could discard user data. Corrupt profiles are not overwritten or auto-repaired.

## 7. Fault-injection and process-crash coverage

`atomic_replace_with_hook` now exposes deterministic test checkpoints:

- Before temporary-file creation.
- After temporary-file creation.
- During writing.
- After file synchronization.
- Immediately before replacement.
- Immediately after replacement becomes visible.
- Before parent-directory synchronization.
- After parent-directory synchronization.
- During failed-operation temporary cleanup.

Pre-commit injected failures preserve and reload the original profile. Post-commit injected failures return an error but reload the complete replacement. A simulated cleanup failure retains the prepared temp file and the original primary file; it does not silently delete potentially useful evidence.

A child process is terminated with `process::exit` at the post-replacement visibility checkpoint. The parent verifies that the new profile loads and validates, the old temporary path is gone, and the lock remains. A subsequent writer gets `Busy`; after the child is known to have exited, the test explicitly removes the stale lock and verifies the profile remains intact.

These are deterministic software fault-injection and controlled child-process termination tests. They are not actual kernel crash, sudden power-loss, storage-controller failure or filesystem-corruption tests.

## 8. macOS verification results

macOS was not available in the current Linux environment, and no macOS Rust target/runtime was installed. The implementation reuses the Unix same-directory rename and parent-directory `sync_all` path; no separate macOS implementation was added.

A native `macos-latest` job is included in `.github/workflows/repository-platform.yml`. It runs core, CLI and probe tests and Clippy, including repository replacement, open-reader, child-process concurrency, lock and crash-state tests. Until that job has actually run successfully, macOS remains **UNVERIFIED**. Source inspection and Linux execution are not represented as macOS verification.

## 9. Linux verification results

The core repository and process tests were executed natively on Linux. They cover existing-profile replacement, open readers, case-only ID behavior, same-profile and different-profile writers, duplicate create/import races, stale revisions, child-process termination with a stale lock, post-replacement visibility, and pre/post-commit fault injection.

The complete workspace verification results are recorded in Section 14 below after final execution.

## 10. Platform guarantee matrix

| Platform | Status in this session | Replacement behavior | Durability statement |
|---|---|---|---|
| Linux | **VERIFIED** | Same-directory rename-over-existing; native tests executed | File `sync_all`, then parent-directory `sync_all`; directory-sync errors are surfaced after visibility. No universal power-loss guarantee. |
| Windows | **COMPILES ONLY** | `ReplaceFileW`; Windows-specific locked-destination test added | File `sync_all` before replacement. Parent-directory durability is not claimed. Native filesystem tests are pending CI execution. |
| macOS | **UNVERIFIED** | Reuses Unix rename path; native tests added to CI matrix | Parent-directory sync attempted and errors surfaced. Native macOS execution is pending CI. |

Network filesystems, unusual mount options, removable media semantics and all forms of sudden power loss remain outside the verified guarantee. A successful cross-target compile is not counted as runtime verification.

## 11. Dependencies added or avoided

No dependency was added and `Cargo.lock` was not changed. Windows replacement uses a small, isolated `ReplaceFileW` FFI declaration rather than a broad filesystem abstraction or runtime dependency. Unsafe code is denied crate-wide except for that isolated Windows module. No Tokio, database, networking, GUI or hardware dependency was introduced.

## 12. Exact changed-file inventory

Files changed in Phase 11.1:

- `.github/workflows/repository-platform.yml` — native Ubuntu/Windows/macOS CI matrix.
- `README.md` — corrected repository guarantees and report/CI links.
- `docs/core-api.md` — API-level locking, replacement, crash and recovery contract.
- `docs/cli.md` — platform status matrix and explicit manual recovery instructions.
- `crates/th80-core/src/lib.rs` — crate-wide unsafe lint changed from `forbid` to `deny` so the one isolated Windows FFI module can explicitly allow unsafe code.
- `crates/th80-core/src/repository.rs` — Windows replacement, sync error propagation, permission preservation, PID lock metadata, extended fault injection and repository unit tests.
- `crates/th80-core/tests/repository_process.rs` — child-process concurrency, duplicate create/import, reader, stale lock and manual recovery tests.
- `crates/th80-cli/tests/offline_workflow.rs` — explicit stdout/stderr and exit-code assertions for JSON, validation, malformed input and CLI usage.
- `PHASE11_1_REPOSITORY_RELIABILITY_CLOSURE.md` — this report.

No Cargo manifest or lockfile changed. No Phase 5–11 report was modified.

## 13. Test counts

Final native Linux workspace results:

| Test target | Passed |
|---|---:|
| `th80-cli` unit tests | 3 |
| CLI offline workflow integration tests | 5 |
| `th80-core` unit tests | 24 |
| Phase 10 schema-v1 compatibility fixture | 1 |
| Cross-process repository integration tests | 5 |
| `th80-probe` library tests | 26 |
| `th80-probe` binary tests | 6 |
| Offline protocol integration tests | 12 |
| Legacy research binary tests | 3 |
| **Total** | **85 passed, 0 failed** |

The core unit suite also verifies Unix mode-bit preservation. The five process integration tests launch real child processes. The core unit suite includes the controlled post-replacement child exit and deterministic pre/post-commit failure injection. A Windows-only locked-destination test is compiled for the Windows target but was not executed natively in this session.

## 14. Mandatory command results

All mandatory commands were run from the workspace root after the final source changes:

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | **Passed** |
| `cargo test --locked` | **Passed — 85 tests, 0 failures** |
| `cargo clippy --locked --all-targets -- -D warnings` | **Passed** |
| `cargo build --locked --release` | **Passed** |
| `cargo doc --locked --workspace --no-deps` | **Passed** |
| `cargo run --locked -p th80-core --example offline_profile` | **Passed** |
| `cargo check --locked --target x86_64-pc-windows-msvc --tests -p th80-core -p th80-cli` | **Passed (compile-only)** |

## 15. Remaining limitations

- Native Windows filesystem tests have not executed in this session. The Windows implementation and tests require the newly added Windows CI job to pass before Windows can be called verified.
- Native macOS tests have not executed. The newly added macOS CI job must pass before macOS can be called verified.
- Real power-loss and kernel-crash durability are not tested. Software fault injection and a child-process exit at a controlled post-replacement checkpoint are tested.
- Abandoned lock files and temporary files are not automatically recovered. Manual inspection is intentional; uncertain ownership means fail closed.
- Windows does not claim parent-directory durability. Unix directory synchronization is attempted, but filesystem and power-loss guarantees remain conditional.
- Case-only profile IDs cannot coexist on case-insensitive filesystems; collision is rejected without overwriting.

If either native CI job is unavailable or fails, keep the platform status at COMPILES ONLY/UNVERIFIED and report the exact failure. Do not downgrade the tests or describe compilation as native verification.

## 16. Hardware safety statement

All tests use synthetic profiles and isolated temporary directories. No real profile directory was intentionally accessed or modified. No physical keyboard or receiver was used. No HID handle was opened. No HID Feature GET/SET, input/output report transfer, USB control/endpoint transfer, firmware communication, flashing, bootloader, pairing, calibration, reset or transport activation occurred. `th80-probe` remains passive; `th80-core` and `th80-cli` remain independent of HIDAPI, USB, networking and asynchronous runtimes.

Phase 5–11 historical reports are preserved. No commit was made. Work stops at Phase 11.1; Phase 12 and desktop UI development are out of scope.
