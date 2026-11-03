# Phase 9 — Product Architecture

Date: 2026-11-01  
Status: architecture decision record and product boundary; no application implementation or device communication.

## 1. Product vision

Build a small, Rust-first, cross-platform companion for the EPOMAKER TH80 Pro keyboard and its ROYUAN 2.4 GHz receiver. The product should make ordinary device management understandable and reliable without reproducing the vendor application's runtime, web stack, updater, or opaque global state.

The first useful product is not a configuration writer. It is a dependable offline configuration workspace backed by accurate passive device identification and descriptor inspection. Users should be able to create, validate, organize, import and export local profiles before any device command is eligible for implementation.

The application must distinguish four different claims:

1. **Local draft** — data currently edited in the application.
2. **Imported/exported profile** — a local file, not evidence about a connected device.
3. **Device-reported state** — bytes returned by a verified operation and parsed under a compatible device/protocol profile.
4. **Device-applied state** — a setting whose application is confirmed by a separately verified acknowledgement/readback contract.

No UI, CLI output, or documentation may describe a local edit or successful file export as having changed the physical keyboard.

## 2. Evidence and product boundaries

Protocol evidence levels are carried with every future command and capability:

- `CONFIRMED` — direct, unambiguous evidence for the specific device/model/protocol version.
- `STRONGLY SUPPORTED` — multiple independent observations, with a material edge still unproven.
- `HYPOTHESIS` — plausible interpretation without direct evidence.
- `UNKNOWN` — insufficient evidence.

These grades describe evidence, not permission. A `STRONGLY SUPPORTED`, `HYPOTHESIS`, or `UNKNOWN` operation is not an active product command. A `CONFIRMED` command still requires a command-specific side-effect/safety review and explicit authorization in a later phase before transport integration.

Current constraints inherited from Phases 5–8:

- Keyboard VID:PID `3151:4010`; receiver VID:PID `3151:4011`.
- Captured normal-mode devices expose three HID interfaces. Interface 2 has an unnumbered 64-byte Feature report (implicit Report ID 0). Matching descriptors do not prove that keyboard and receiver have identical firmware behavior.
- The vendor macOS implementation uses Feature class control transfers with `wValue=0x0300`, 64-byte transfer length and hardcoded `wIndex=0`, despite the captured Feature descriptor being on interface 2. The reason/correctness of that mismatch is unresolved.
- The frontend's ordinary command helper submits a command-bearing Feature SET and then a Feature GET. A method name or GET response does not establish read-only semantics.
- The native RPC URI-to-wrapper layer has partial evidence; final indirect dispatch and ordinary handler mapping are unresolved.
- Opcode `0x89` is used by both per-key RGB/user-picture and keymap frontend paths with different request layouts.
- Battery and Fn-keymap methods previously associated with TH80 were found in unrelated model classes. `watchSystemInfo` is host system telemetry, not proof of battery status.
- The adjacent Windows updater's `X` sidecar has no established format, target, firmware compatibility, or command semantics. It must not be treated as verified firmware.
- None of the 12 Phase 7B/8 candidates is established as read-only or approved for hardware testing.

The authoritative detail remains in [Phase 5C Feature correlation](PHASE5C_FEATURE_REPORT_CORRELATION.md), [Phase 5C RPC resolution](PHASE5C_RPC_DISPATCH_RESOLUTION.md), [Phase 5C updater correlation](PHASE5C_UPDATER_CORRELATION.md), [Phase 5D RPC audit](PHASE5D_RPC_DISPATCH_AUDIT.md), [Phase 5D Feature buffer layout](PHASE5D_FEATURE_BUFFER_LAYOUT.md), [Phase 6 foundation](PHASE6_DRIVER_FOUNDATION.md), [Phase 7A offline tests](PHASE7A_OFFLINE_PROTOCOL_TESTS.md), [Phase 7B command matrix](PHASE7B_COMMAND_EVIDENCE_MATRIX.md), [Phase 8 evidence closure](PHASE8_STATIC_EVIDENCE_CLOSURE.md), and [Phase 8 command matrix](PHASE8_COMMAND_SEMANTICS_MATRIX.md).

## 3. Existing project audit

### 3.1 Workspace and executables

The current workspace contains two packages:

| Package | Current role | Product assessment |
|---|---|---|
| Root package `th80ctl` | Legacy Linux HIDAPI diagnostic executable in `src/main.rs` | Research/diagnostic utility; not the safe cross-platform product base. |
| `th80-probe` | Linux sysfs diagnostic binary plus reusable library in `crates/th80-probe` | Best existing foundation for passive discovery, descriptor parsing and offline domain/protocol work. |

The root `th80ctl` source uses `HidApi::new()` and HIDAPI enumeration; its opt-in `record-input` command calls `open_path` and reads keyboard reports. That command captures potentially sensitive typed input. The root manifest enables `hidapi` with `linux-native`, bringing native `libudev`/C build dependencies and making the current root executable Linux-specific. Do not silently merge this path into the future application's default discovery or diagnostics.

The `th80-probe` binary reads USB/HID sysfs attributes and kernel-exposed `report_descriptor` files. It does not open `/dev/hidraw*` or `/dev/bus/usb/*`, and does not issue HID/USB requests. Its implementation is in `crates/th80-probe/src/main.rs`; the descriptor parser and library models are in sibling modules. This is the preferred starting point for the first safe product CLI.

There is documentation drift: README descriptions refer to both the sysfs-only probe and the older HIDAPI-based root executable. A later documentation cleanup should give the binaries unambiguous names and safety descriptions without removing the historical research utility prematurely.

### 3.2 Reusable code

| Component | Current location | Reuse decision / caveat |
|---|---|---|
| Exact VID/PID identity | `crates/th80-probe/src/device.rs` | Reuse; extend with instance identity separate from model identity. |
| HID descriptor parser | `crates/th80-probe/src/hid.rs` | Reuse as the primary parser; retain bounds checks, warnings, field metadata and descriptor hashes. Add fuzz/property coverage before treating hostile inputs as fully hardened. |
| Capability conversion | `crates/th80-probe/src/capabilities.rs` | Reuse; keep descriptor facts separate from inferred product features. |
| Sysfs passive discovery | `crates/th80-probe/src/main.rs` | Reuse and gradually extract into a platform-neutral discovery interface plus Linux adapter. |
| Typed metadata/error models | `device.rs`, `error.rs` | Reuse and refine; avoid duplicate private CLI models. |
| Offline tests and descriptor fixture | `tests/offline_protocol.rs`, `tests/fixtures/` | Preserve. Fixture is a captured descriptor, not captured command traffic. |
| Feature envelope and platform packet reconstructions | `protocol.rs` | Research-only. Keep pure and isolated; do not expose as production commands or send them. |
| `DisabledTransport` and `Transport` trait | `transport.rs` | Preserve the disabled behavior; redesign boundaries before any future backend. It currently mixes passive discovery with active I/O. |
| Root HIDAPI diagnostic/capture executable | `src/main.rs` | Keep separate and clearly label research-only. Do not make raw input capture a normal product feature. |

### 3.3 Research-only / not production protocol code

- `build_macos_feature`, `build_windows_feature`, `simulate_macos_get_response`, and `command_semantics` are reconstructions of observed buffer behavior, not validated device protocol operations. The macOS and Windows formats differ and must remain separate.
- The `0x01` Windows prefix is not established as a target Report ID.
- `wIndex=0` is a recovered vendor implementation detail, not an endorsed interface selection.
- The command candidate inventory C01–C12 is evidence tracking, not a callable command API.
- The `Transport` trait is only a contract; `DisabledTransport` is not a functional device backend.
- The custom HID parser is useful and well tested, but should receive fuzzing and cross-platform fixture coverage before a stability guarantee.

No configuration/profile domain model or import/export schema currently exists. No product UI exists.

## 4. Proposed minimal architecture

Do not create every proposed crate immediately. Establish boundaries first and split crates only when there is a real independent consumer or dependency boundary.

### 4.1 Logical layers

```text
┌──────────────────────────────────────────────────────┐
│ Optional UI (native Rust UI; framework not selected) │
└──────────────────────────┬───────────────────────────┘
                           │ application use cases
┌──────────────────────────▼───────────────────────────┐
│ Application / CLI                                     │
│ workflows, profile lifecycle, export/import, display │
└──────────────────────────┬───────────────────────────┘
                           │ domain APIs
┌──────────────────────────▼───────────────────────────┐
│ Core                                                  │
│ identity, models, validation, state provenance, error│
└──────────────┬───────────────────────────┬───────────┘
               │ descriptor facts          │ verified ops only
┌──────────────▼────────────┐  ┌───────────▼───────────┐
│ Device discovery          │  │ Protocol              │
│ OS metadata, HID descs    │  │ evidence-gated codecs │
└──────────────┬────────────┘  └───────────┬───────────┘
               │                           │ typed reports
┌──────────────▼───────────────────────────▼───────────┐
│ Transport boundary — disabled / absent in initial MVP│
│ future OS adapters; explicit operations and errors  │
└──────────────────────────────────────────────────────┘
```

Dependency direction is inward: UI/CLI depend on application and core; core must not depend on UI, OS APIs, protocol transport or a runtime. Protocol codecs operate on byte slices and typed models only. Discovery returns observations; it does not own command execution. Transport must never be selected implicitly as a fallback.

### 4.2 Crate evolution

Target shape when the code justifies it:

```text
crates/
  th80-core/       # identity, domain models, validation, state provenance, errors
  th80-device/     # discovery abstractions, descriptor parsing, capability facts
  th80-protocol/   # evidence registry, verified command codecs and parsers
  th80-transport/  # future OS adapters; no backend in the initial product
  th80-cli/        # safe CLI workflows
  th80-app/        # optional desktop UI, added only after UI evaluation
```

`th80-probe` remains as a compatibility binary during migration. Its current library can be the temporary home for parser and model code; do not perform a mechanical crate split as Phase 9 work. The root `th80ctl` package should be treated as a legacy diagnostic package until its HIDAPI dependency and input-capture scope are deliberately resolved.

A sensible first implementation may use only two packages: one reusable library (initially the existing `th80-probe` library) and one passive CLI. Introduce `th80-core`, `th80-device`, `th80-protocol`, and `th80-transport` as separate crates only when compile-time dependency isolation, platform-specific dependencies, or multiple frontends make the split valuable.

### 4.3 Core domain

Core should own:

- `DeviceModel`/`DeviceIdentity` for known VID/PID and product role.
- `DeviceInstanceId` for an observed instance; topology paths and OS paths are ephemeral locators, not permanent identities.
- `DeviceCapabilities` containing only descriptor-derived or otherwise sourced facts, each with provenance.
- Configuration models for key assignments, layers, macros, lighting preferences and profiles. Initially these are local abstract models; fields whose physical representation is unknown must not be guessed.
- Validation errors and explicit unsupported-capability results.
- State provenance such as `LocalDraft`, `Imported`, `DeviceReported`, and `DeviceApplied`, with timestamp/source metadata where applicable.

Do not bake assumed TH80 defaults, matrix geometry, macro limits, RGB enum values, Fn-layer behavior or receiver parity into the core until supported by evidence. Keep model/version-specific constraints in capability profiles rather than global constants.

### 4.4 Protocol

Protocol owns pure transformations only:

- Typed request/response definitions for operations whose evidence gate is satisfied.
- Exact request builders and response parsers with checked lengths, enum validation, endianness and explicit partial/error behavior.
- Protocol dialect/version selection tied to an identified device model/firmware compatibility range.
- An evidence record per operation: source report, evidence grade, known request sequence, response schema, side effects, unresolved risks and approval state.

No generic `send_command(opcode, bytes)` API should be exposed to the application. No raw command console should be shipped. Unknown operations must fail closed before any transport call. Strongly supported is not enough for active execution. The current C01–C12 remain unavailable.

### 4.5 Discovery and transport

Separate passive discovery from active transport:

- `DiscoveryProvider`: enumerate metadata and read OS-exposed descriptors only. A discovery result must state which fields are unavailable and where each fact came from.
- `DeviceTransport`: future explicit I/O session for a selected instance/interface and an allow-listed operation. It should not provide arbitrary raw writes to ordinary application code.
- Keep `FeatureGet`, `FeatureSet`, `InputRead`, and `OutputWrite` as distinct transport capabilities; do not conflate HIDAPI Output reports with libusb Feature control transfers.
- Interface selection must be explicit and descriptor-validated. No fallback from interface 2 to interface 0, from Feature to Output, or between operating-system APIs.
- No transport implementation is part of the initial MVP. `DisabledTransport` remains a hard failure if retained as a placeholder.

Platform strategy:

| Platform | Initial passive path | Later transport consideration | Current status |
|---|---|---|---|
| Linux | sysfs USB/HID metadata and kernel-exposed report descriptors; no device-node open | hidraw/HIDAPI or a carefully selected USB API only after review | Passive CLI exists; no product backend. |
| Windows | SetupAPI/HID metadata and descriptor discovery without issuing reports | Windows HID APIs behind a Windows-only adapter | Not implemented. |
| macOS | IOKit/IOHID metadata and descriptor discovery without Feature requests | IOHID/libusb choice requires resolving interface/`wIndex` semantics and API differences | Not implemented. |

Cross-platform support should be achieved by small platform adapters implementing the same passive discovery contract, not by putting OS-specific code in core. Avoid a custom kernel driver; standard HID class interfaces are already present.

## 5. CLI and application scope

### Initial user-facing scope

1. Identify supported keyboard/receiver VID:PID and distinguish their roles.
2. Show passive interface, endpoint and descriptor facts with source and confidence.
3. Validate and display local profiles.
4. Create/edit local keymap, macro and lighting models only to the extent their abstract schemas are well-defined.
5. Import/export local profiles with a versioned, documented schema.
6. Explain clearly that local data is not applied to a connected device.

The CLI should provide predictable machine-readable output, stable exit codes, useful human-readable diagnostics and no background daemon. The first CLI should reuse the `th80-probe` sysfs implementation. Keep any raw input capture command separate, explicitly opt-in and out of the default product; it can record typed text and is not needed for configuration management.

### Desktop UI

A desktop UI is optional and follows a working core/CLI. Candidate evaluations:

- **Slint:** Rust-oriented declarative native UI; worth a small prototype for memory, startup, accessibility, widget maturity, packaging and cross-platform behavior.
- **egui/eframe:** productive Rust-native immediate-mode option; prototype with default features minimized and measure rendering/runtime cost.
- **Tauri:** smaller than Electron in many deployments because it uses the system WebView, but still introduces a web frontend/runtime boundary and frontend toolchain. Not the default choice for a Rust-first, minimal-runtime product; consider only if web UI reuse is a strong requirement.
- **Electron:** explicitly excluded.

Do not select a framework by reputation or theoretical binary size. Build the same small device/profile screen in candidate frameworks and measure cold startup, idle RSS, release bundle size, CPU while idle, accessibility and packaging on Linux/Windows/macOS. Keep the UI behind an application API so it can be replaced without touching protocol/core code.

## 6. Dependency policy

Current direct dependencies:

| Dependency | Current use | Recommendation |
|---|---|---|
| `serde` | Derives for typed data and snapshots | Keep; mature and directly useful. |
| `serde_json` | Diagnostic JSON output | Keep; use for diagnostics and the first profile schema if a versioned JSON format is chosen. |
| `hidapi` | Root `th80ctl` Linux diagnostic/capture executable only | Isolate from product/core. Do not add it to `th80-probe` or core. Reassess only in a separately authorized transport phase; it has native/system dependency and platform/API semantics to account for. |

Transitive dependencies from `hidapi` include `libudev-sys`, `udev`, `libc`, `nix`, `cc` and `pkg-config`; the `th80-probe` package itself currently depends only on `serde` and `serde_json`.

Avoid adding dependencies until a concrete feature needs them:

- Keep the current small argument parser until CLI complexity justifies `clap`.
- Do not add Tokio or another async runtime for sysfs inspection, profile editing or ordinary CLI workflows. Add async only if a demonstrated platform API or concurrent workload requires it.
- Do not add a database, ORM, global state manager, plugin framework, telemetry SDK or background service.
- Use `serde_json` for a first portable profile format; define schema versioning and validation before committing to it. Add TOML only if human-edited configuration is a real requirement.
- Preserve the current in-tree SHA-256 implementation unless a broader cryptographic requirement justifies a maintained crypto crate; do not imply a descriptor fingerprint is an authenticity signature.
- UI dependencies remain unselected until measured prototypes exist.

Keep `Cargo.lock` committed for reproducible application builds. Review every new dependency for maintenance, license, transitive tree, platform system packages, binary size and attack surface.

## 7. Safety model

The initial product is offline-first and has no active transport implementation. Explicitly prohibited in this phase and the initial MVP:

- HID Feature GET/SET, Output writes, Interrupt writes/reads and USB control transfers.
- Opening `/dev/hidraw*` or `/dev/bus/usb/*` for product configuration workflows.
- Interface claiming/detaching, resets, pairing, calibration, profile selection, persistent configuration writes, bootloader entry, firmware update/erase/recovery or factory reset.
- Arbitrary opcodes, undocumented commands, automatic command probing or transport fallbacks.
- Running vendor updater/firmware binaries or extracting/executing firmware code.

Passive sysfs descriptor inspection is distinct from report communication. The legacy raw input capture path is also distinct from configuration and must not be invoked by the product workflow. No background process should watch devices or capture input without an explicit user action.

A future transport proposal requires a separate explicit authorization and safety review. A protocol command must not be wired merely because a builder/parser exists. The review must establish device/firmware compatibility, complete request/response sequence, all meaningful side effects, failure behavior, cancellation/disconnect behavior, and a recovery story appropriate to the operation. Unknown or unsupported operations fail closed.

## 8. Testing strategy

All core and protocol tests must run without hardware. Preserve the current 47-test baseline and the captured interface-2 descriptor fixture exactly as a descriptor fixture.

Add tests incrementally for:

- VID/PID identity, unknown IDs and keyboard-versus-receiver distinction.
- Descriptor parser malformed/truncated items, report IDs, report lengths, overflow boundaries, nested collections and warnings.
- Discovery parsing using synthetic sysfs directory fixtures, missing/permission-denied metadata and device-disconnect races.
- Capability provenance: descriptor fact versus inferred feature versus unknown.
- Profile validation, schema version compatibility, import/export round trips and unknown-field policy.
- Keymap/layer/macro/RGB domain invariants without asserting unsupported physical layouts.
- Protocol builders/parsers only after command schemas become sufficiently evidenced; malformed lengths, invalid enum values, truncation and partial responses.
- Evidence gate: `UNKNOWN`, `HYPOTHESIS`, and `STRONGLY SUPPORTED` commands cannot be selected for active execution.
- Transport tests using fakes only after a future transport phase; tests must prove that unsupported operations fail closed.

Use synthetic fixtures with explicit `synthetic` labels. Never invent captured traffic. Consider `cargo-fuzz` or property-based tests only when the parser is exposed to untrusted descriptor/profile input at product scale; do not add them as Phase 9 dependencies.

## 9. Performance and reliability goals

Initial engineering targets (to be measured on reference Linux hardware and recorded rather than claimed in advance):

- CLI cold start: under 100 ms for local profile operations on a typical desktop, excluding OS device enumeration.
- Idle memory: under 20 MiB for the passive CLI; desktop UI target set only after framework prototypes.
- No resident daemon or background polling in the initial release.
- No network access required for ordinary discovery, profile editing or import/export.
- No large runtime or async runtime for basic CLI operations.
- Bounded memory for descriptor/profile parsing; reject oversized inputs before allocation where possible.
- Deterministic validation and explicit errors; no silent truncation, implicit default command values or hidden transport fallback.
- Cross-platform builds must not pull Linux-only dependencies into Windows/macOS targets.

These are targets, not measured current performance results. Release size and memory must be measured on each supported target.

## 10. Technical risks and product-defect attribution

| Risk | Why it matters | Mitigation / classification |
|---|---|---|
| Protocol/firmware semantics unknown | UI commands may cause writes or mode transitions | Keep C01–C12 unavailable; continue static evidence only until separately authorized. |
| Keyboard/receiver similarity over-assumed | Matching descriptors do not prove identical behavior | Model firmware/device role separately; require per-target evidence. |
| Vendor app bug vs firmware defect | A failed vendor workflow may originate at several layers | Record observations by layer: frontend builder/sequence, native transport, device response, firmware behavior. Do not label a firmware defect without device-side evidence. |
| Interface 2 vs `wIndex=0` | Host request may target a different interface | Do not copy the vendor setup packet into a backend; resolve in a separate evidence phase. |
| Opcode `0x89` collision | Command byte alone is not a unique operation | Typed command dialect and full request context required; no opcode-only dispatch. |
| No verified firmware image | Device-side parser/side effects cannot be confirmed | Do not treat updater sidecar `X` as firmware. Seek only provenance-verified artifacts in a later research phase. |
| Legacy CLI safety/documentation drift | Two binaries have different access behavior | Keep sysfs-only `th80-probe` as safe default; label or retire HIDAPI raw input capture separately. |
| Cross-platform API differences | HIDAPI/libusb/IOKit/Windows HID have different report conventions | Platform adapters and transport capability declarations; no hidden fallback. |
| Profile schema guesses | Incorrect physical mapping could corrupt a future write | Keep abstract/local models and explicit unsupported fields until layout evidence exists. |
| UI framework overhead | Heavy runtime undermines product goals | Prototype and benchmark before selection. |

## 11. Future product phases

1. **Phase 10 — Safe core consolidation:** extract shared identity, parser, capability and error types from `th80-probe`; remove duplicate CLI models; separate passive discovery from transport contracts. No active I/O.
2. **Phase 11 — Offline profile domain:** define versioned local profile schema, validation, import/export, provenance and round-trip tests.
3. **Phase 12 — Safe CLI product:** stable `list`, `inspect`, `capabilities`, `profile validate`, `profile import/export` workflows; keep device writes absent.
4. **Phase 13 — Cross-platform passive discovery:** Linux sysfs first, then Windows and macOS metadata/descriptor adapters, with fixture-driven tests.
5. **Phase 14 — UI evaluation and implementation:** benchmark Slint/egui prototypes, select only after evidence, then implement device overview and offline profile editor.
6. **Protocol research track (separate gate):** resolve device-side command semantics and firmware compatibility. No command becomes active by roadmap progression alone.
7. **Transport phase (separate explicit authorization):** only after a command-specific safety review; implement narrow allow-listed operations, no generic raw console, and maintain offline tests.

The product roadmap does not imply that firmware replacement or bootloader support is required. Firmware work remains a separate, hardware-specific project with independent identity and recovery requirements.

## 12. Phase 9 outcome

Phase 9 defines architecture and sequencing only. No source code, packet builder, manifest, lockfile, firmware artifact or historical report is changed by this phase. The intended outcome is a maintainable offline-first Rust product path with an explicit, evidence-gated boundary around future device communication.
