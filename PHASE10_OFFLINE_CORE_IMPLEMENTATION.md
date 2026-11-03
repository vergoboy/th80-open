# Phase 10 — Offline Core, Configuration Models & Profile Management

Date: 2026-11-01  
Status: implemented and verified; offline-only. No physical device communication.

## 1. Summary

Phase 10 adds a reusable Rust library (`th80-core`) and an independent profile CLI (`th80ctl`). Profiles can be created, validated, edited locally, serialized, listed, imported, exported and deleted without enumerating or opening a keyboard or receiver. The old root-package HIDAPI diagnostic binary remains separate and is now named `th80-hid-research`.

This phase does **not** implement the TH80 vendor configuration protocol. A locally valid profile is not proof of firmware support, a profile file is not a device readback, and a successful save/export does not apply settings to a keyboard.

## 2. Existing code reused

- `th80_probe::device::{DeviceIdentity, DeviceKind}` remains the source of truth for the known `3151:4010` keyboard and `3151:4011` receiver identities. The existing types gained `Deserialize` so they can be used in typed configuration metadata.
- The Phase 5B HID report-descriptor parser in `crates/th80-probe/src/hid.rs` was not copied, rewritten or modified. Its existing tests and captured descriptor fixture remain intact.
- Existing `DriverError`, passive probe models, protocol research types and `DisabledTransport` were preserved. The new core has its own configuration error and validation-issue types because the probe's driver/transport error model does not describe profile storage or domain validation.
- The Phase 7A offline protocol regression harness and all earlier diagnostic/protocol tests were retained without weakening or deleting tests.

The `th80-core` crate depends on the reusable `th80-probe` library for the existing identity types. The core does not depend on the probe executable, HIDAPI, USB libraries, sysfs, an OS device API, or a transport implementation.

## 3. Implemented workspace architecture

```text
crates/
  th80-probe/       Existing passive Linux sysfs probe + descriptor parser
  th80-core/        Reusable, hardware-independent Rust domain library
    src/
      lib.rs
      config.rs     Profiles, keymaps/actions, macros, lighting, options
      device.rs     Identity metadata and unverified properties
      capability.rs Evidence classes and offline capability descriptors
      validation.rs Application validation and separate device assessment
      repository.rs Versioned JSON and local profile repository
      error.rs      Structured core errors and validation issues
  th80-cli/         Offline `th80ctl` profile CLI
```

The core is a normal synchronous Rust library. It has no global mutable state, unsafe code, async runtime, UI framework, database, network client, HID handle or physical transport. Serialization and file storage live in the repository layer; a separate serialization crate/module was not justified at this stage.

## 4. Domain model decisions

### Device identity

The core reuses the established `DeviceIdentity` and `DeviceKind`. `DeviceDescriptor` adds optional hardware revision and firmware version fields, a capability list and a map of unverified properties. The TH80 Pro constructor leaves revision and firmware unset and explicitly records that the protocol remains unverified. No hardware revision or firmware version is fabricated.

### Profiles and provenance

`Profile` contains a stable string identifier, name, description, schema version, optional device-model association, configuration, metadata and provenance. Metadata includes creation/modification Unix-millisecond timestamps and tags. The generated ID uses a timestamp plus a process-local sequence; repository path validation independently restricts IDs to 1–128 ASCII letters, digits, hyphens or underscores.

Provenance is typed and distinct:

- `LocalDraft`
- `ValidatedLocally`
- `SerializedToDisk`
- `ImportedFile`
- `DeviceReported`
- `DeviceApplied`

Repository persistence changes a local draft/locally validated copy to `SerializedToDisk`. Import marks it `ImportedFile`. Phase 10 validation rejects device-reported/device-applied provenance because there is no verified read/apply implementation. A saved file can therefore never be mistaken for device-applied state.

### Keymap and actions

Keymaps contain an arbitrary number of layers. Each layer has a stable ID, display name and a map from opaque logical-position strings to one typed action. Logical positions are intentionally not coordinates: no physical matrix dimensions, switch count, geometry or mapping are asserted.

The action enum supports key, modifier, combination, layer switch, momentary layer, macro reference, transparent and unassigned. One action is stored per position; multi-action/chord semantics beyond the explicit combination variant are not invented. Transparent and unassigned are separate tagged variants, so contradictory simultaneous states cannot be represented.

### Macros

Macro definitions have IDs, names and ordered typed steps: key-down, key-up, delay and text. Text is an offline abstraction only; no vendor macro encoding or text injection behavior is claimed. Local validation checks balanced key-down/key-up sequences, duplicate key-down, unmatched key-up, empty text, maximum step count and delay bounds.

The local resource limits are 10,000 steps per macro and 600,000 ms per delay. These are application safety limits, **not** claimed firmware limits.

### Lighting and options

Lighting supports optional enabled/mode/brightness/speed/color values, logical per-key colors and generic JSON effect parameters. RGB channels are `u8`, so each channel is inherently bounded to 0–255. Brightness and speed use an application-normalized 0–100 range only; this is not asserted to match a TH80 firmware range or encoding. Effect parameters remain generic and do not imply any vendor mode IDs.

Keyboard options are held in a typed extension map (`BTreeMap<String, serde_json::Value>`). No report-rate, debounce, sleep or other undocumented device setting is exposed as a confirmed TH80 option. Users can retain local application-specific values without claiming firmware compatibility.

## 5. Evidence and capability behavior

Capability descriptors contain a feature ID, support status, evidence class (`CONFIRMED`, `STRONGLY_SUPPORTED`, `HYPOTHESIS`, `UNKNOWN`), rationale, offline-editability and a device-communication approval flag. The Phase 10 offline profile features are locally editable while device protocol evidence remains `UNKNOWN`; all communication approval flags are false.

Application validation (`validate_profile`) is separate from device compatibility assessment (`assess_device_compatibility`). The latter can identify a profile/device-model mismatch, but otherwise reports `NotVerified`; it does not claim device support or communicate with the device. No capability is promoted to firmware-confirmed based solely on a local model.

## 6. Validation behavior

Validation returns structured issues with stable code, path and actionable message. It checks:

- supported schema version, non-empty profile name, safe/unique identifier form and timestamp ordering;
- duplicate layer IDs and macro IDs;
- non-empty logical positions, key/modifier names and combination members;
- layer-switch/momentary-layer and macro references;
- macro step ordering, held-key balance, duplicate key-down, unmatched key-up, empty text and local resource limits;
- normalized lighting brightness/speed bounds, non-empty logical per-key positions, duplicate per-key assignments and non-empty effect-parameter names;
- forbidden claims of device-reported/device-applied provenance.

Typed RGB fields prevent out-of-range channel values from being represented. Application validation does not infer device-specific numeric ranges. Missing required JSON fields and malformed types fail during deserialization with parse errors. Unknown nested fields are rejected rather than silently discarded. Unknown top-level profile fields are preserved in the `extensions` map and survive serialization round trips.

## 7. Serialization and profile repository

The on-disk format is JSON with `schema_version: 1`. Serde/serde_json were already present in the workspace; no JSON dependency was added. Pretty JSON uses deterministic struct field order and `BTreeMap` ordering for maps. The repository validates after deserialization and before create/save/export/import.

`ProfileRepository` supports create, load, save, list, delete, import, export and validate. Filenames are derived from validated profile IDs; an arbitrary filename is never trusted as an identifier. Create and export use exclusive file creation and refuse to overwrite. Import rejects duplicate IDs. Save writes a uniquely named temporary file in the repository directory, flushes/syncs it, then atomically renames it over the existing profile on platforms whose filesystem rename supports replacement (verified on the Linux development host). Temporary files are removed after a failed replacement. Cross-platform replacement semantics, especially Windows replacement of an existing target, still require platform-specific verification before claiming full cross-platform atomicity.

There is no database. The default repository path is `~/.local/share/th80-profiles`; `TH80_PROFILE_DIR` overrides it. Tests use unique temporary directories and do not depend on the user's actual profile directory.

## 8. CLI functionality

The new `th80ctl` binary is provided by package `th80-cli` and supports:

```text
th80ctl profile create --name NAME [--description TEXT]
th80ctl profile list
th80ctl profile show PROFILE_ID
th80ctl profile add-layer PROFILE_ID --layer-id ID --name NAME
th80ctl profile set-key PROFILE_ID --layer ID --position LOGICAL_POSITION --key KEY_CODE
th80ctl profile validate FILE.json
th80ctl profile export PROFILE_ID --output FILE.json
th80ctl profile import FILE.json
th80ctl profile delete PROFILE_ID
```

`add-layer` and `set-key` demonstrate offline editing. Positions remain logical opaque IDs. `set-key` explicitly replaces an existing assignment at that logical position. Macro and lighting structures can be edited through the library API or JSON import/export; dedicated CLI editors are future work. Errors are printed to stderr and return exit code 2; malformed JSON and validation failures are not silently accepted.

The legacy root HIDAPI program is renamed `th80-hid-research` in the manifest. It remains separate and is not initialized by `th80ctl`. The passive sysfs utility remains `th80-probe`.

## 9. Tests and verification

The final workspace suite contains **62 passing tests**: 47 pre-existing tests remain, and 15 Phase 10 tests were added (12 in `th80-core`, 3 in `th80-cli`). Coverage includes identity metadata, provenance, device compatibility not being inferred, schema/missing-field errors, extension preservation, nested unknown-field rejection, duplicate layer/macro IDs, invalid layer/macro references, macro key sequencing and delay limits, lighting bounds and duplicate per-key colors, repository create/load/list/save/delete/import/export, duplicate imports, malformed JSON, and non-clobbering/atomic save behavior.

Final commands:

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo test --locked` | PASS — 62 passed, 0 failed |
| `cargo clippy --locked --all-targets -- -D warnings` | PASS |
| `cargo build --locked --release` | PASS |

The release CLI was exercised end-to-end in temporary directories: help, create, list, show, add-layer, set-key, validate, export, import, duplicate-import rejection, delete and malformed-JSON nonzero exit. A synthetic two-layer profile containing key assignments, a macro, lighting values and a top-level extension was also imported and round-tripped; it was explicitly labelled synthetic, not captured from hardware. `th80-probe list` was run to confirm the existing passive sysfs probe still works.

The `th80-cli` dependency graph contains only `th80-core`, serde and serde_json (plus their existing transitive dependencies); it contains no HIDAPI, USB library, async runtime or database. `th80-core` has the same lightweight serialization dependencies and the local `th80-probe` library for identity reuse. No third-party dependency was added; Cargo.lock gained the two new local workspace package entries.

## 10. Performance and memory

The core is synchronous and uses ordinary owned Rust data structures. Profiles and ordered macro steps are held in memory; maps use `BTreeMap` for deterministic output. There is no runtime, background worker, database, network service or global cache. JSON serialization is whole-profile in-memory serialization, appropriate for the current small local profile use case. Macro resource limits constrain pathological local inputs.

## 11. Known limitations

- No device read, write, apply, synchronization or firmware compatibility operation exists.
- No physical matrix layout, switch coordinate mapping, device-side layer count, macro limit, lighting enum or option range is known.
- Symbolic key names and macro text are local abstractions, not a confirmed vendor keycode namespace or firmware encoding.
- Brightness/speed bounds and macro resource limits are application policy, not hardware specifications.
- Only the profile's top-level unknown fields are preserved as extensions; unknown nested fields are rejected to prevent silent data loss.
- Dedicated CLI editors for macros/lighting/options are not yet provided; the public core model and JSON import/export cover these data types.
- The Linux atomic replacement path is tested. Windows/macOS filesystem behavior and platform-specific replacement guarantees still need verification.
- The current profile repository is a simple local JSON directory, not a synchronized or multi-process database. Concurrent writers to the same profile are not coordinated.
- The existing passive discovery implementation remains Linux-specific; this phase does not add Windows/macOS discovery adapters.

## 12. Features blocked by missing protocol evidence

All physical communication remains disabled. No Feature GET/SET, Output report, interrupt transfer, USB control transfer, interface claim/detach, persistent setting write, pairing/calibration, reset, bootloader entry or firmware update was implemented or attempted. Phase 7B/8 command candidates remain unapproved; a local profile cannot be mapped to the recovered vendor command candidates without verified firmware semantics and a separate safety authorization.

## 13. Changed files

- `Cargo.toml`, `Cargo.lock`
- `README.md`, `docs/cli.md`
- `crates/th80-probe/src/device.rs` (serde deserialization on reused identity types)
- `crates/th80-core/Cargo.toml`
- `crates/th80-core/src/lib.rs`
- `crates/th80-core/src/config.rs`
- `crates/th80-core/src/device.rs`
- `crates/th80-core/src/capability.rs`
- `crates/th80-core/src/error.rs`
- `crates/th80-core/src/validation.rs`
- `crates/th80-core/src/repository.rs`
- `crates/th80-cli/Cargo.toml`
- `crates/th80-cli/src/main.rs`
- `PHASE10_OFFLINE_CORE_IMPLEMENTATION.md`

No Phase 5–9 report, captured fixture, descriptor parser, protocol reconstruction, `DisabledTransport`, firmware artifact or existing test was rewritten. The legacy `src/main.rs` source is unchanged; only its Cargo binary name changed.

## 14. Safety confirmation and phase boundary

The profile CLI and core contain no hardware APIs or device-node paths. Their dependency graph contains no HIDAPI or USB library. No keyboard/receiver handle was opened; no HID or USB report/transfer was sent or received; no interface was claimed or detached; no transport was activated; no firmware/updater was executed or modified. The existing passive `th80-probe` was used only to read Linux sysfs metadata and kernel-exposed descriptors.

Phase 10 is complete. Work stops here; Phase 11 is not started.
