# Phase 11 — Offline Engine Hardening

**Status: implemented with explicit platform and verification limitations.** This phase hardens the offline Rust profile engine and CLI. It does not implement a desktop UI, activate transport, or establish firmware compatibility.

## 1. Executive summary

Phase 11 leaves the project with three separate layers:

1. `th80-probe`: passive Linux sysfs and report-descriptor inspection.
2. `th80-core`: synchronous, hardware-independent profile models, application validation, diagnostics, evidence metadata and local JSON persistence.
3. `th80-cli` (`th80ctl`): offline profile creation, inspection, editing, validation, import/export and deletion.

The core no longer depends on the probe crate. The CLI depends on the core and does not initialize HIDAPI or access USB/HID devices. The CLI now supports practical macro, lighting and extensible-option editing. Validation diagnostics are structured and the validation command has a JSON envelope with an explicit `not_verified` device-compatibility state.

Repository writes use same-directory temporary files, exclusive creation for new files, per-profile exclusive lock files, stale-copy rejection and fail-closed replacement. On Unix, replacement uses `rename`; on Windows, replacement of an existing target currently fails safely rather than deleting the original. Cross-platform runtime guarantees are therefore not complete.

## 2. Initial architecture audit

The Phase 10 architecture already contained profile/keymap/macro/lighting/option models, schema version 1, validation, a repository and a profile CLI. The audit identified an undesirable dependency direction: `th80-core` imported device identity types from `th80-probe`. This made the reusable configuration domain depend on the discovery layer.

`DeviceKind` and `DeviceIdentity` now live in `th80-core`; `th80-probe` re-exports those types. `th80-core` depends only on Serde/Serde JSON at runtime and no longer depends on `th80-probe`. The root `th80-hid-research` executable remains a separate legacy research utility.

## 3. Public API boundary and compatibility

The public API is grouped into these modules:

| Module | Public responsibility |
|---|---|
| `config` | `Profile`, `ProfileId`, `Configuration`, metadata/provenance, keymap/actions, macros, lighting and options |
| `capability` | `CapabilityDescriptor`, `EvidenceClass`, `SupportStatus`, `FirmwareConstraintEvidence`, `offline_capabilities` |
| `device` | `DeviceKind`, `DeviceIdentity`, `DeviceDescriptor` |
| `validation` | `validate_profile`, `assess_device_compatibility`, compatibility report/status and application policy |
| `error` | `CoreError`, `Diagnostic`, severity/domain and the `ValidationIssue` compatibility alias |
| `repository` | `ProfileRepository` filesystem operations |

The crate root re-exports the common profile, device, diagnostic, repository and policy types. Filesystem paths are confined to repository APIs and repository errors; domain profiles do not contain storage handles. The CLI is not exposed as a library API. Public model fields remain directly accessible to support independent frontends, so callers must validate before persistence and must not infer compatibility from constructibility.

The crate is `0.1.0` (pre-1.0). Rust public APIs may evolve between `0.1.x` releases; intentional breaking changes should be accompanied by release/migration notes. No full stable API guarantee is claimed. Persisted `schema_version` is a separate contract and is not derived from the crate version.

## 4. Library example

`crates/th80-core/examples/offline_profile.rs` demonstrates creating a profile with multiple logical layers, typed key and layer actions, a balanced macro, generic lighting, a per-key color and an application-owned option; validating it; JSON round-tripping it; and saving/loading through `ProfileRepository`. It was executed successfully with Cargo. The data is synthetic and local.

## 5. Application policy, capability evidence and firmware constraints

These are separate concepts in the model and documentation:

- **Application policy** is a local resource/representation rule. `APPLICATION_POLICY` currently sets 10,000 macro steps, 600,000 ms maximum individual delay and normalized lighting values up to 100. These are not firmware specifications.
- **Capability evidence** is represented by `CapabilityDescriptor`, `EvidenceClass` (`Confirmed`, `StronglySupported`, `Hypothesis`, `Unknown`), rationale and source references. Generic offline-editable features are initialized with `Unknown` evidence and no sources. Offline editability does not upgrade evidence.
- **Firmware constraints** have their own `FirmwareConstraintEvidence` records, including feature/constraint IDs, optional value, evidence class, sources and rationale. No TH80 Pro firmware limits are populated by the default descriptor; unknown values remain absent.

`DeviceDescriptor::th80_pro()` leaves hardware revision and firmware version unset, marks them unverified, and leaves firmware constraints empty. The evidence records retain provenance fields, but the library does not independently authenticate source references or prove the truth of an externally supplied `Confirmed` record. Consumers must assess the supplied provenance.

## 6. Validation and compatibility architecture

`validate_profile` performs deterministic application/schema validation: identifier/name requirements, schema/provenance checks, unique layer/macro IDs, action references, macro key sequencing, local resource policy, lighting representation and duplicate logical per-key assignments. These diagnostics use the `application_validation` domain.

`assess_device_compatibility` is separate. A profile explicitly associated with a different model yields `model_mismatch`; otherwise it yields `not_verified`. There is no firmware acceptance result because no firmware communication or validated device-specific constraint model exists. A locally valid profile is not necessarily accepted by the keyboard.

## 7. Machine-readable diagnostics and stable code registry

`Diagnostic` contains a stable code, severity, domain, message, optional profile ID/path, structured context and optional remediation. `ValidationIssue` remains an alias for compatibility with earlier consumers. `CoreError::code()` returns stable repository/serialization identifiers rather than Rust debug output.

The validation code registry is documented in `docs/core-api.md`. Current validation codes are:

- `unsupported_schema`, `invalid_profile_id`, `required`, `invalid_timestamp`, `unsupported_provenance`
- `duplicate_layer_id`, `invalid_layer_id`, `invalid_position`, `invalid_action`, `invalid_combination`
- `invalid_layer_reference`, `invalid_macro_reference`, `duplicate_macro_id`, `invalid_macro_id`
- `macro_too_large`, `delay_too_long`, `invalid_key_down`, `invalid_key_up`, `unreleased_keys`, `empty_macro_text`
- `brightness_out_of_range`, `speed_out_of_range`, `invalid_lighting_position`, `duplicate_per_key_color`, `invalid_effect_parameter`, `unsafe_capability`

Core error codes are `REPOSITORY_IO_ERROR`, `PROFILE_JSON_INVALID`, `PROFILE_VALIDATION_FAILED`, `PROFILE_ALREADY_EXISTS`, `PROFILE_NOT_FOUND`, `PROFILE_ID_INVALID`, `PROFILE_SCHEMA_UNSUPPORTED`, `PROFILE_LOCK_BUSY` and `PROFILE_CONCURRENT_MODIFICATION`.

## 8. JSON validation and process exit codes

`th80ctl profile validate FILE --output json` writes a JSON envelope to stdout with envelope/schema version, `valid`, `device_compatibility` and structured `diagnostics`. Malformed JSON emits a JSON error envelope when JSON output was requested. Human validation errors and command errors are sent to stderr.

Exit codes are:

| Exit | Meaning |
|---:|---|
| `0` | Command succeeded; validation passed where applicable |
| `2` | Application validation failed |
| `3` | Input, malformed JSON or argument-value failure |
| `4` | Repository/filesystem, duplicate, lock or stale-save failure |
| `64` | CLI usage/unknown command |

The JSON envelope has a version field. Consumers should branch on stable codes and structured fields, not message wording. The JSON validation integration tests parse stdout with `serde_json`.

## 9. Offline editing commands

The CLI provides:

- Macros: `add`, `list`, `show`, `rename`, `remove`, `step-add`, `step-remove`, `step-replace`. `step-add` accepts repeated `--step` flags so a balanced ordered sequence can be applied and validated in one save.
- Lighting: `show`, `set`, `key-set`, `key-remove`, `param-set`, `param-remove`.
- Options: `list`, `set`, `remove`, using JSON values while preserving unrelated application-owned option keys and top-level extension fields.

Macro edits validate the complete profile before saving. Generic lighting mode names and normalized values are application abstractions, not confirmed TH80 firmware settings. Unknown nested schema fields are rejected rather than silently discarded.

The end-to-end CLI workflow covers profile creation, multiple layers, key assignment, macro create/edit/rename/show/remove, lighting and per-key color, option editing, human/JSON validation, show, export/import and deletion. It also tests duplicate import preservation and malformed JSON/validation/usage exit codes.

## 10. Repository atomicity and durability

New profile files are serialized to a same-directory temporary file opened with exclusive creation, fully written and `sync_all`'d, then installed with a hard link so an existing target is not clobbered. Updates serialize to an exclusive same-directory temporary file, sync it, and replace the target with `rename`. Failed pre-replacement writes clean up the temporary file and leave the original target bytes unchanged.

On Unix, `rename` replacement is atomic with respect to readers on supported local filesystems. Parent-directory sync is attempted best-effort after create/replace/delete. On non-Unix platforms parent-directory sync is not implemented. File sync and directory sync improve durability but cannot guarantee survival across every filesystem, mount configuration, controller cache or power-loss scenario.

The fault-injection test simulates failures before temp creation, after temp creation, during partial write and immediately before replacement. It asserts the original bytes remain and temporary files are cleaned. There is no injected post-replacement crash point: after a successful rename the new file is visible, and durability then depends on platform/filesystem behavior.

## 11. Concurrent writer policy

Mutations acquire a per-profile lock using `create_new`, which is an OS-level exclusive file creation primitive and therefore coordinates processes sharing the same repository directory. A second writer fails closed with `PROFILE_LOCK_BUSY`. Different profile IDs use different lock files. Save additionally compares the loaded profile's modification timestamp to the currently stored revision marker and rejects stale copies with `PROFILE_CONCURRENT_MODIFICATION`; a successful save advances the timestamp beyond the prior stored value even when the wall clock has not advanced.

Locks are removed on normal drop. A crash can leave a stale lock. The implementation deliberately does not break stale locks automatically; an operator must establish that no writer is active before manually removing `.PROFILE_ID.lock`. The test suite verifies contention by creating a competing lock file and exercises concurrent different-profile creates and readers during replacement. It does **not** include a child-process race harness for same-profile writes or simultaneous same-ID imports; the cross-process property follows from exclusive file creation but those full process-race scenarios remain unverified.

## 12. Crash tests and corrupt-profile behavior

The deterministic write-stage hook covers four pre-replacement failure points and checks original-byte preservation and temporary cleanup. Abandoned hidden temp files and lock files are ignored by profile listing. Corrupt/truncated JSON returns a structured parse error and source bytes remain unchanged. Unsupported schema and invalid nested fields are rejected. No automatic repair, overwrite, deletion or quarantine occurs. Recovery is intentionally manual: preserve a byte-for-byte copy, inspect/repair a separate copy, and import it under an explicit profile ID after validation.

## 13. Cross-platform verification

- **Linux:** mandatory workspace tests/build/docs/clippy ran in the current Linux environment. Unix rename replacement and best-effort directory sync are the exercised implementation path.
- **Windows:** `cargo check --locked --target x86_64-pc-windows-msvc -p th80-core -p th80-cli` succeeded as a cross-target compile check. This is not Windows runtime testing. Existing-target replacement currently fails safely because `std::fs::rename` does not provide the required replace-existing behavior on this path. Windows replacement support and runtime crash/locking semantics remain incomplete.
- **macOS:** no macOS target/runtime was available; no macOS compile or runtime verification is claimed.

There is no repository CI configuration in `.github` to supply additional platform coverage. Cross-platform replacement semantics are a known remaining limitation, not a completed acceptance claim.

## 14. Schema evolution and Phase 10 compatibility

The persisted schema remains version 1, independent of crate version. Unsupported future versions are rejected. Top-level extension fields are preserved through the profile extension map; nested structs intentionally reject unknown fields to avoid silent loss. Serialization uses ordered maps where applicable for deterministic output.

A synthetic schema-v1 fixture at `crates/th80-core/tests/fixtures/phase10_profile_v1.json` represents the Phase 10 JSON shape. Its integration test deserializes, application-validates and round-trips the fixture. It is a synthetic compatibility fixture, not a device capture. No migration to a newer schema was introduced in this phase.

## 15. Dependencies

No new third-party dependency was added for Phase 11. `th80-core` uses the existing Serde and Serde JSON dependencies. The CLI uses those and the local core crate. The root legacy research package retains its existing HIDAPI dependency, isolated from the profile CLI/core dependency path. No Tokio, database, networking, UI or filesystem-abstraction dependency was added.

## 16. Known Phase 11 changed-file inventory

The Phase 11 implementation/documentation files recorded as changed are:

- `Cargo.lock`
- `README.md`
- `docs/cli.md`
- `docs/core-api.md`
- `crates/th80-core/Cargo.toml`
- `crates/th80-core/src/lib.rs`
- `crates/th80-core/src/device.rs`
- `crates/th80-core/src/capability.rs`
- `crates/th80-core/src/error.rs`
- `crates/th80-core/src/validation.rs`
- `crates/th80-core/src/repository.rs`
- `crates/th80-core/examples/offline_profile.rs`
- `crates/th80-core/tests/fixtures/phase10_profile_v1.json` (added in final audit)
- `crates/th80-core/tests/phase10_compatibility.rs` (added in final audit)
- `crates/th80-cli/src/main.rs`
- `crates/th80-cli/tests/offline_workflow.rs`
- `crates/th80-probe/Cargo.toml`
- `crates/th80-probe/src/device.rs`
- `PHASE11_OFFLINE_ENGINE_HARDENING.md` (this report)

The workspace has no Git metadata (`git rev-parse --show-toplevel` reports that this directory is not a Git repository), so a Git diff/status and a byte-for-byte changed-file comparison cannot be produced. The inventory is based on the inspected source timestamps and recorded Phase 11 edits; files that predate the Phase 11 work are not included. Phase 5–10 report files were not edited in this phase; their presence and timestamps were checked. No commit was made.

## 17. Test counts and verification results

Latest `cargo test --locked` result: **75 passed, 0 failed**.

| Target | Passed |
|---|---:|
| `th80-cli` unit tests | 3 |
| `th80-cli/tests/offline_workflow.rs` | 5 |
| `th80-core` unit tests | 19 |
| `th80-core/tests/phase10_compatibility.rs` | 1 |
| `th80-probe` library tests | 26 |
| `th80-probe` binary tests | 6 |
| `th80-probe/tests/offline_protocol.rs` | 12 |
| `th80-hid-research` tests | 3 |
| Doc tests | 0 |

Mandatory commands, run after the Phase 10 fixture was added:

- `cargo fmt --all -- --check` — passed.
- `cargo test --locked` — passed, 75 tests.
- `cargo clippy --locked --all-targets -- -D warnings` — passed.
- `cargo build --locked --release` — passed.
- `cargo doc --locked --workspace --no-deps` — passed.

Additional checks:

- `cargo run --locked -p th80-core --example offline_profile` — passed.
- `cargo check --locked --target x86_64-pc-windows-msvc -p th80-core -p th80-cli` — passed (cross-target compile only).
- `cargo tree --locked -p th80-core` — core dependency path contains Serde/Serde JSON and no probe/HID/network/async runtime.
- `cargo tree --locked -p th80-cli` — CLI depends on the local core and Serde/Serde JSON, not HIDAPI.
- Static scan of `crates/th80-core` and `crates/th80-cli` for HIDAPI, HID device paths, feature/output report APIs, Tokio and Reqwest — no matches.

The CLI `--help` and `profile --help` commands were run; both show the offline command surface without creating the profile repository. The end-to-end integration test runs with isolated temporary profile directories. The example also uses a temporary directory.

## 18. Limitations and remaining work

1. Windows update/replace of an existing profile is safe-failure only; implement and runtime-test a platform-specific replace operation before claiming full Windows CRUD parity.
2. macOS has not been compiled or runtime-tested.
3. Same-profile cross-process race and simultaneous same-ID import races have not been exercised with child processes; the lock primitive is cross-process by design, but those scenarios need explicit process tests.
4. Stale lock recovery is manual; no automatic PID/liveness-based lock breaking is attempted.
5. Post-replacement crash/power-loss behavior is not fault-injected and depends on the platform/filesystem durability contract.
6. Capability and firmware evidence records carry source/rationale fields but the library does not authenticate external source references. No firmware compatibility claim or device-specific firmware limit is established.
7. Validation JSON covers the validation workflow; the CLI's general input/filesystem error classification is still coarser than a full independently versioned diagnostic registry for every command path.
8. Markdown links were inspected in the touched docs, but there is no automated repository-wide link checker in this phase.
9. No Git metadata exists in the workspace, preventing a canonical diff/status review.

These limitations are recorded rather than represented as completed acceptance criteria.

## 19. Hardware safety statement

No physical keyboard or receiver was used. No HID handle was opened by Phase 11 work. No HID Feature GET/SET, Output report, USB control transfer, endpoint transfer, firmware communication/update, bootloader, pairing, calibration, reset or transport activation occurred. The existing disabled transport contract and passive probe behavior were not activated or repurposed. All Phase 11 tests use synthetic data and isolated temporary directories.

Phase 11 ends here. No GUI or Phase 12 work is included.
