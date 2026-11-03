# Phase 11.2 — Native Platform Verification

**Status: PARTIALLY COMPLETE — native Linux verification passed; native Windows and macOS execution evidence is unavailable.**

## 1. Executive summary

Phase 11.2 re-ran the complete workspace verification on the available native Linux host, audited the platform CI workflow, and added the missing formatting, release-build and documentation checks to the existing Ubuntu/Windows/macOS matrix. The Linux workspace suite passed all 85 tests. The three offline crates also passed their CI-scoped build and documentation commands.

Native Windows and macOS tests were not executed. This workspace snapshot has no `.git` metadata or remote, and the authenticated GitHub account has no matching `th80-open` repository. Consequently, no Actions run could be found or triggered, and no workflow run ID, commit SHA, or hosted-runner image can be truthfully reported. A Windows target check compiled `th80-core` and `th80-cli` test code, including the `ReplaceFileW` implementation, but that is compile-only evidence and is not counted as Windows filesystem verification. macOS remains unverified.

No repository implementation or dependency changes were made. The only changes are this report and the focused CI workflow additions. Phase 11.2 acceptance criteria requiring native Windows and macOS execution therefore remain open.

## 2. Initial verification gaps

At the start of this phase:

- Phase 11.1 reported native Linux success (85 workspace tests), Windows test-target compilation only, and no macOS execution.
- The workflow already declared native `ubuntu-latest`, `windows-latest`, and `macos-latest` matrix entries and used `dtolnay/rust-toolchain@stable`.
- The workflow ran the offline-crate tests and Clippy, but omitted formatting, release build and documentation checks.
- The local workspace contained no `.git` directory, no resolvable Git remote and no commit identity.
- The available host was Linux; no native Windows or macOS runner was attached to this execution environment.

## 3. CI workflow audit

File: `.github/workflows/repository-platform.yml`

The workflow declares `push`, `pull_request`, and `workflow_dispatch` triggers. Its single matrix job has these native runner labels:

| Matrix entry | Intended native runner |
|---|---|
| `ubuntu-latest` | GitHub-hosted Ubuntu |
| `windows-latest` | GitHub-hosted Windows |
| `macos-latest` | GitHub-hosted macOS |

All jobs check out the repository and install the stable Rust toolchain with `dtolnay/rust-toolchain@stable`. Cargo commands use `--locked`. The matrix uses `fail-fast: false`, so one platform failure does not cancel the other matrix jobs. No step-level `if` condition silently skips a platform. Ordinary step failures fail that job.

The workflow was extended to run formatting, release builds and crate documentation in addition to the existing tests and Clippy. The commands are deliberately scoped to `th80-core`, `th80-cli`, and `th80-probe` for build/test/doc/lint operations. These are the hardware-independent/offline crates under test; the root `th80ctl` package is a separate legacy Linux HIDAPI research utility, not part of the offline repository manager.

The full workspace commands were independently run on Linux. A Linux-hosted cross-target check of the root package for `x86_64-pc-windows-msvc` failed inside `hidapi` because the Linux host's GNU toolchain lacks the Windows SDK header `guiddef.h` and `hidapi` reported that GNU is unsupported for that target. This is a cross-compilation/toolchain limitation, not a native Windows failure. It supports keeping the cross-platform filesystem matrix focused on the three offline crates rather than expanding it to the legacy hardware research binary. No dependency or product architecture change was made to work around this unrelated cross-target build.

The updated YAML was parsed locally. The three native runner labels and all five CI command categories were confirmed, and there are no step-level skip conditions. No system package installation is required for the selected offline crates; their dependency trees contain only the serialization crates and local `th80-core` dependency. The legacy root HIDAPI dependency is outside those selected CI packages.

**Important:** this audit verifies workflow configuration only. The workflow has not been observed running on GitHub in this phase. The presence of native runner labels is not native execution evidence.

## 4. Linux native results

Runner: native local Linux host, not a GitHub-hosted runner.

Observed environment:

- OS/kernel: `Linux Archlinux 7.2.11-arch1-1 #1 SMP PREEMPT_DYNAMIC Fri, 16 Oct 2026 00:28:58 +0000 x86_64 GNU/Linux`
- Rust: `rustc 1.99.1 (58cf8f9c3 2026-10-13)`
- Cargo: `cargo 1.99.1 (5f94df478 2026-08-27)`
- Target: `x86_64-unknown-linux-gnu`

The complete workspace test command passed with 85 tests and zero failures:

| Test target | Passed |
|---|---:|
| CLI unit tests | 3 |
| CLI offline workflow integration tests | 5 |
| Core unit tests | 24 |
| Phase 10 compatibility fixture | 1 |
| Cross-process repository integration tests | 5 |
| Probe library tests | 26 |
| Probe binary tests | 6 |
| Offline protocol tests | 12 |
| Legacy research parser tests | 3 |
| **Total** | **85** |

Repository tests executed on Linux cover new profile creation, existing-profile replacement, injected failures before and after replacement visibility, temp-file cleanup and deliberate retention on cleanup failure, Unix mode-bit preservation, open-reader behavior, same-profile writer conflict, independent profile mutations, create/import races, stale-revision rejection, child-process lock termination, stale-lock detection and explicit stale-lock recovery. The subprocess crash checkpoint verifies that the complete replacement remains authoritative after the child exits immediately after replacement visibility.

The 85-test result includes CLI structured JSON diagnostics and stable error-code assertions, Phase 10 schema compatibility, and passive probe/parser regressions. All tests use synthetic profiles and isolated temporary directories.

## 5. Windows native results

**Status: NOT NATIVE-VERIFIED.** No Windows runner executed during this phase.

The available Windows evidence is compile-only:

```text
cargo check --locked --target x86_64-pc-windows-msvc --tests -p th80-core -p th80-cli
Finished `dev` profile
```

This confirms that the Windows-gated `ReplaceFileW` code and Windows-only test source compile for the MSVC target. It does not execute `ReplaceFileW`, Windows sharing semantics, filesystem locks, child processes, permission behavior or crash recovery.

The Windows-specific test `windows_replace_fails_safely_when_destination_denies_delete_sharing` is present and was compiled, not run. It opens the destination without `FILE_SHARE_DELETE`, expects replacement to fail while original bytes remain intact, then retries after closing the handle. The native Windows job must execute this test before replacement failure preservation can be marked verified.

No native Windows test failures or fixes can be reported because no Windows test process ran. There is no Windows runner image/version, Actions run ID, commit SHA, test count or log available from this phase.

## 6. macOS native results

**Status: UNVERIFIED.** No native macOS runner executed during this phase, and no macOS Rust target/runtime was available locally.

The repository uses the Unix same-directory rename and parent-directory `sync_all` path on macOS. That implementation was executed on Linux only. No macOS claim is inferred from Linux behavior or source inspection.

No native macOS tests, filesystem metadata checks, file/directory synchronization calls, child-process races or crash-recovery checkpoints were executed. There is no macOS runner image/version, Actions run ID, commit SHA, test count or log available from this phase.

## 7. Platform verification matrix

Statuses below refer only to tests actually executed on the named operating system. `COMPILES ONLY` is used for Windows test code compiled for the Windows target without Windows runtime execution. `UNSUPPORTED` means the implementation explicitly does not provide that guarantee.

| Behavior | Linux | Windows | macOS |
|---|---|---|---|
| New profile creation | VERIFIED | COMPILES ONLY | UNVERIFIED |
| Existing profile replacement | VERIFIED | COMPILES ONLY | UNVERIFIED |
| Replacement failure preservation | VERIFIED (injected pre-commit failures) | COMPILES ONLY (sharing-denial test not executed) | UNVERIFIED |
| Concurrent same-profile writers | VERIFIED | COMPILES ONLY | UNVERIFIED |
| Concurrent different-profile writers | VERIFIED | COMPILES ONLY | UNVERIFIED |
| Reader during replacement | VERIFIED | COMPILES ONLY | UNVERIFIED |
| Process termination with lock | VERIFIED | COMPILES ONLY | UNVERIFIED |
| Stale-lock detection | VERIFIED | COMPILES ONLY | UNVERIFIED |
| Explicit stale-lock recovery | VERIFIED | COMPILES ONLY | UNVERIFIED |
| Post-replacement crash recovery | VERIFIED | COMPILES ONLY | UNVERIFIED |
| Temporary-file cleanup | VERIFIED | COMPILES ONLY | UNVERIFIED |
| File synchronization | VERIFIED (successful `sync_all` path exercised) | COMPILES ONLY | UNVERIFIED |
| Directory synchronization | VERIFIED (Unix parent-directory sync path exercised) | UNSUPPORTED (not claimed by implementation) | UNVERIFIED |

Windows rows are compile-only because the Windows-specific test target compiled, but no test ran on Windows. The `UNSUPPORTED` Windows directory-sync status is intentional: the implementation flushes the replacement file but does not claim directory-entry durability. macOS rows remain unverified because native macOS execution is absent. The Linux synchronization results show the implemented calls succeeded during tests; they do not prove survival of arbitrary power loss or storage failure.

## 8. Runner environments

| OS | Actual execution environment | Rust/toolchain | Result |
|---|---|---|---|
| Linux | Arch Linux, kernel `7.2.11-arch1-1`, x86_64 | rustc/cargo 1.99.1 | Native tests executed |
| Windows | No native runner available to this session | Windows MSVC target installed on Linux host | Compile-only check; no runtime tests |
| macOS | No native runner available to this session | No macOS target/runtime installed | No compilation or runtime test |

The GitHub workflow declares `ubuntu-latest`, `windows-latest`, and `macos-latest`, but those are configured labels, not observed runner environments for this phase.

## 9. Workflow identifiers and commit SHAs

- Workflow: `.github/workflows/repository-platform.yml`
- Workflow run identifier: **not available** — no run could be resolved or triggered.
- Commit SHA: **not available** — the workspace has no `.git` metadata.
- Git remote: **not available** — no remote is configured and Git cannot resolve a repository root.
- GitHub lookup: authenticated `gh` could not resolve `gh repo view` or `gh run list` for this workspace; direct lookup of `vergoboy/th80-open` returned HTTP 404, and the account repository listing contained no matching project.
- Failed/skipped Actions jobs: **unknown** — there is no Actions run record to inspect. No job is claimed as passed or skipped.

### Required action to obtain remaining evidence

The project must be present in an accessible GitHub repository with this workflow committed or otherwise checked out on a valid ref. Then dispatch the workflow (or push/open a pull request) and retain the resulting Actions run URL/ID and commit SHA. Confirm that all three matrix jobs actually start and finish successfully. For each OS, preserve logs for the five CI steps and the platform-specific repository tests. If the project is intentionally hosted outside GitHub, execute the same offline-crate commands on native Windows and macOS runners in that CI provider and record its run identifiers and runner images instead.

## 10. Mandatory command results

The exact workspace-wide commands were executed on native Linux after the workflow edit:

| Command | Linux result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo test --locked` | PASS — 85 passed, 0 failed |
| `cargo clippy --locked --all-targets -- -D warnings` | PASS |
| `cargo build --locked --release` | PASS |
| `cargo doc --locked --workspace --no-deps` | PASS |

The updated CI workflow also runs these five categories for the three offline crates, using package-scoped commands for tests, Clippy, release build and documentation. The corresponding package-scoped release build and documentation commands passed locally on Linux. The exact workspace-wide commands were not run natively on Windows or macOS because no such runners were accessible. They must not be inferred from the Linux result or the Windows cross-target check.

Additional checks:

- Windows MSVC target: `cargo check --locked --target x86_64-pc-windows-msvc --tests -p th80-core -p th80-cli` — PASS, compile-only.
- YAML parsing and structural assertions — PASS; native matrix labels, command list and absence of conditional skips confirmed.
- Markdown local-link validation — PASS for README, core API, CLI and Phase 10/11/11.1 reports.
- `cargo tree --locked -p th80-core` and `cargo tree --locked -p th80-cli` — PASS; only serde/serde_json and the local core dependency are present.
- Static core/CLI boundary scan — PASS; no HIDAPI, USB, network or async-runtime references found in those crates.
- CLI JSON diagnostic parsing and stable error-code behavior — covered by passing CLI integration tests.

The Linux full-workspace command counts and test breakdown above are the only native runtime counts available. No Windows or macOS pass/fail/skipped counts exist.

## 11. Failures and fixes

1. **CI workflow omitted required checks.** Added `cargo fmt`, release build and documentation steps to the existing native matrix. The existing offline-crate test and Clippy steps remain intact. No new matrix entries or dependencies were introduced.
2. **Full root-package Windows cross-check failed on the Linux host.** `cargo check --locked --target x86_64-pc-windows-msvc --package th80ctl` failed in `hidapi 2.6.7`: the Linux GNU cross-toolchain lacks `guiddef.h` and is unsupported for that target. This does not establish whether the package builds on a native MSVC runner. The root package is the legacy Linux HIDAPI research utility; the native cross-platform repository workflow remains scoped to the offline crates. No attempt was made to redesign or port the legacy hardware research utility in this verification phase.
3. **Native Windows/macOS infrastructure could not be reached.** No source-level filesystem defect was observed on Linux, and no speculative platform changes were made. Windows/macOS remain open pending actual native CI execution.

No failing native Windows or macOS tests occurred; neither platform's tests were run. No fix is claimed for an unobserved platform behavior.

## 12. Remaining limitations

- Windows `ReplaceFileW`, locked-destination failure, native reader sharing, concurrency, process termination and crash recovery still require a passing native Windows run.
- macOS rename replacement, reader behavior, file/parent-directory sync and metadata behavior still require a passing native macOS run.
- Workflow changes exist only in this local workspace snapshot until published to an accessible repository/ref; no Actions run can consume them yet.
- No runner image/version, workflow run ID, commit SHA or hosted logs can be supplied.
- Linux tests use deterministic fault injection and controlled child-process exits, not physical power-loss testing.
- Unix mode-bit preservation is tested on Linux; ownership, ACLs and extended attributes are not claimed as portable guarantees.
- Windows directory-entry durability is intentionally unsupported/not claimed.
- Filesystem guarantees on network, unusual, damaged or volatile-storage filesystems remain outside the evidence collected here.

## 13. Final repository reliability assessment

The offline repository implementation has a passing native Linux regression suite, including cross-process writer exclusion, atomic reader visibility, stale-lock handling, compatibility and crash checkpoints. The Windows implementation and its platform-specific failure test compile for the MSVC target, but native replacement semantics remain unverified. macOS behavior remains unverified. The repository is therefore reliable to the extent demonstrated on Linux, but **Phase 11.2 does not meet full cross-platform acceptance** until the native Windows and macOS jobs execute and pass.

## 14. Hardware safety statement

No physical keyboard or receiver was used. No HID handle was opened. No HID Feature GET/SET, HID input/output report transfer, USB control transfer, USB endpoint communication, device configuration access, firmware communication, flashing/updating, bootloader operation, pairing, calibration, reset, experimental device command or transport activation occurred. The tests use synthetic profile data and isolated temporary directories. The disabled transport contract was not changed. No real user profile directory was intentionally accessed or modified. No desktop UI, database, async runtime or network client was introduced.

## 15. Scope and historical reports

- Preserved all existing Phase 5–11.1 reports.
- Created exactly one Phase 11.2 report: `PHASE11_2_NATIVE_PLATFORM_VERIFICATION.md`.
- No third-party dependency was added; `Cargo.lock` was not changed.
- No implementation/API/schema/CLI behavior was changed.
- No commit was made.
- Phase 12 and desktop UI development were not started.
