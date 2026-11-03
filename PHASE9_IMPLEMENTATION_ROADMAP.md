# Phase 9 — Implementation Roadmap

Date: 2026-11-01  
Status: prioritized implementation plan; no source implementation or hardware communication.

## 1. Roadmap rules

- Preserve all Phase 5–8 reports as historical evidence. New findings belong in new phase documents, not retroactive edits.
- Reuse `th80-probe` and its offline tests before creating new crates or replacing dependencies.
- Keep the first product useful without device writes: passive identification, descriptor diagnostics, offline profiles and validation.
- Do not implement an active command based on a frontend method name, opcode constant, synthetic packet, parser alone or a strongly supported native call edge.
- Treat keyboard and receiver as separate targets even when VID/PID family and descriptors look similar.
- No hardware communication, transport activation, updater execution or firmware modification is authorized by this roadmap.

## 2. Existing components to reuse

| Existing component | Path | Planned use | Required follow-up |
|---|---|---|---|
| Passive Linux sysfs discovery and CLI | `crates/th80-probe/src/main.rs` | Initial safe discovery/inspection implementation | Extract parsing/discovery from presentation; add synthetic sysfs fixtures; preserve current commands. |
| HID report descriptor parser | `crates/th80-probe/src/hid.rs` | Shared descriptor parser | Fuzz/property test later; preserve parser diagnostics and report-field provenance. |
| Typed identity and interface metadata | `crates/th80-probe/src/device.rs` | Core device/domain types | Add observed-instance locator separate from VID/PID identity; do not treat topology as permanent ID. |
| Capability conversion | `crates/th80-probe/src/capabilities.rs` | Descriptor-derived capability model | Keep reported capability distinct from a usable/verified product feature. |
| Error model | `crates/th80-probe/src/error.rs` | Shared errors | Refine contextual errors and unsupported-capability representation as workflows grow. |
| Offline packet research | `crates/th80-probe/src/protocol.rs` | Regression reference for recovered buffer transformations | Keep research-only; do not expose C01–C12 command APIs or connect to transport. |
| Disabled transport contract | `crates/th80-probe/src/transport.rs` | Fail-closed safety placeholder/tests | Split passive discovery from active I/O before any backend; keep all active methods disabled. |
| Offline regression suite | `crates/th80-probe/tests/offline_protocol.rs` | Permanent no-hardware test baseline | Preserve all 12 integration tests and clearly label simulated behavior. |
| Captured descriptor fixture | `crates/th80-probe/tests/fixtures/captured_interface2_descriptor.hex` | Descriptor parser regression fixture | Preserve its SHA-256 and its identity as a descriptor, not command traffic. |
| Root legacy HIDAPI executable | `src/main.rs` | Research/diagnostic reference only | Do not use as default app entry point; isolate `record-input` due to sensitive keystroke capture and Linux-specific native dependency. |

Current baseline: 47 tests (26 library, 6 probe CLI, 12 offline protocol integration, 3 workspace CLI), as previously verified. Re-run all required checks at each implementation milestone.

## 3. Prioritized implementation tasks

### P0 — Establish a safe, unambiguous product baseline

**Tasks**

1. Keep `th80-probe` as the default safe diagnostic executable during the first product iteration.
2. Document the distinction between the sysfs-only `th80-probe` and root `th80ctl` HIDAPI executable. Resolve README command/safety ambiguity before marketing a release.
3. Keep raw keyboard-input capture outside the product's default command surface; it can contain typed passwords/messages and is unnecessary for profile management.
4. Add a short architecture decision record in code only when implementation begins; this Phase 9 report is the current design authority.
5. Capture current dependency tree, binary size and baseline test counts for comparison in a future implementation phase.

**Acceptance criteria**

- A user can identify which binary performs only sysfs reads and which code can open a HID path.
- The default product workflow has no HID Feature, Output, Interrupt or USB control-transfer operation.
- No service, daemon, auto-probe loop or network dependency is introduced.
- Existing commands/tests remain compatible until a deliberate migration is documented.

### P1 — Consolidate shared domain and discovery code

**Tasks**

1. Extract the private identity, interface, endpoint and descriptor snapshot models from `th80-probe/src/main.rs` to reusable library types where practical.
2. Reuse `DeviceIdentity`, `InterfaceRole`, `EndpointInfo`, `InterfaceInfo`, `DescriptorFingerprint`, and `CapabilityProfile`; avoid a second parallel type hierarchy.
3. Introduce a passive `DiscoveryProvider`/`DescriptorSource` abstraction separate from active `Transport`.
4. Implement only a Linux sysfs discovery provider initially. Its API returns observations and source/availability details; it never opens device nodes.
5. Keep OS paths and USB topology as ephemeral instance locators. Keep model identity as VID/PID plus role.
6. Preserve `th80-probe` CLI output or version the JSON schema if a change is unavoidable.

**Acceptance criteria**

- Core identity/capability types compile without HIDAPI/libusb/OS transport dependencies.
- Discovery tests use temporary synthetic sysfs trees and do not require attached devices.
- Missing metadata, permission errors, partial enumeration and disconnect races are explicit results, not panics or fabricated values.
- No new transport implementation exists.

**Implementation follow-through — 2026-11-02:** The Linux-first P1 path is implemented in `crates/th80-probe/src/discovery.rs` and `sysfs.rs`: passive `DiscoveryProvider` and `DescriptorSource` are separate from active `Transport`; `SysfsDiscovery`/`SysfsDescriptorSource` reuse the shared device and capability models; `th80-probe` uses the provider while retaining its existing JSON schema. Synthetic sysfs tests cover both verified VID/PID roles, unrelated-device filtering, descriptor parsing, malformed descriptors, and instance identity reuse. Diagnostic JSON wrappers remain in the CLI only to preserve optional-field and raw-descriptor output compatibility.

### P2 — Define offline configuration domain

**Tasks**

1. Define a versioned `ProfileDocument` envelope with schema version, target family, profile metadata and payload.
2. Separate generic user intent from device-specific encoding. Example: a key assignment is a domain concept; its physical matrix index and command-page bytes remain an adapter concern until established.
3. Model layers and key assignments only with known constraints. Mark unknown Fn behavior, matrix geometry, macro limits and RGB effect enums as unsupported/unspecified rather than inventing defaults.
4. Add provenance/status to every displayed value: local draft, imported, descriptor-derived, device-reported or device-applied.
5. Define validation errors for unsupported keys, duplicate identifiers, invalid references, oversized files and schema versions.
6. Choose JSON for the first serialization only if a concrete interchange contract is required; use `serde`/`serde_json` already present. Do not add a database.

**Acceptance criteria**

- Profile models do not import protocol, transport, HIDAPI or UI modules.
- Round-trip serialization preserves supported data and rejects incompatible schema versions clearly.
- Import never triggers device I/O.
- Export is labelled as a local profile file and never as a device backup unless it came from a verified device read.

### P3 — Offline profile workflows and CLI

**Tasks**

1. Add `profile validate`, `profile import`, `profile export`, and local profile listing/editing workflows to a product CLI when the model exists.
2. Use explicit file paths and safe write behavior (write temporary file then atomic rename where supported); never overwrite silently.
3. Keep human-readable output and versioned JSON diagnostics separate.
4. Preserve `list`, `inspect`, and `capabilities` as passive discovery commands.
5. Do not implement `profile apply`, `device set`, arbitrary `feature`, or generic raw command commands.

**Acceptance criteria**

- All workflows work with no connected keyboard/receiver.
- Import/export round trips are covered by tests.
- File errors, invalid profiles and unsupported fields are actionable.
- The CLI makes local-vs-device state explicit in help and output.

### P4 — Harden parser and offline test coverage

**Tasks**

1. Add property/fuzz testing for the HID descriptor parser after selecting and reviewing the smallest suitable development-only tool.
2. Add tests for integer/buffer overflow boundaries, malformed nested collections, invalid report IDs and large descriptor limits.
3. Add fixture-driven sysfs discovery tests for both known VID/PID roles and unknown devices.
4. Add profile validation and serialization property tests.
5. Keep synthetic protocol buffers explicitly marked synthetic. No fixture should be called a capture unless it is an actual recorded artifact with provenance.
6. Preserve tests proving `DisabledTransport` rejects Feature GET/SET, Input, Output and discovery operations until the API is intentionally split.

**Acceptance criteria**

- Test suite runs fully offline and does not enumerate/open host devices.
- No test depends on the user's keyboard, receiver or USB topology.
- Parser rejects malformed/oversized input deterministically and does not panic.
- Existing Phase 7A assertions remain intact.

### P5 — Cross-platform passive discovery

**Tasks**

1. Keep Linux sysfs as the first supported provider.
2. Design Windows metadata/descriptor discovery around SetupAPI/HID metadata, without Feature or Output report calls.
3. Design macOS metadata/descriptor discovery around IOKit/IOHID metadata, without invoking Feature requests.
4. Define common behavior for inaccessible descriptors, missing strings, duplicate interfaces, disconnect races and unsupported products.
5. Add OS-specific dependencies only inside target-specific crates/features; core remains platform-neutral.

**Acceptance criteria**

- Linux remains buildable without Windows/macOS dependencies.
- Windows and macOS implementations have fixture-backed parsing tests and explicit permission/error behavior.
- No platform provider issues report transfers or claims interfaces.
- A platform is not advertised as supported until its provider is implemented and tested on that OS.

### P6 — Desktop UI evaluation and implementation

**Tasks**

1. Build equivalent minimal prototypes in Slint and egui/eframe; evaluate Tauri only if a web UI is justified.
2. Measure cold startup, idle RSS, release package size, idle CPU, accessibility, native look/feel and cross-platform packaging.
3. Keep profile/domain/application APIs independent of UI toolkit.
4. Implement device overview, profile list, offline keymap editor, macro editor and lighting model editor only for fields whose semantics are defined locally.
5. Clearly label unsupported device capabilities and local-only changes.

**Acceptance criteria**

- Framework selected by recorded measurements, not assumption.
- UI does not link to transport or protocol internals directly.
- No Electron and no background service.
- The UI works offline and cannot claim settings have been applied to the keyboard.

### P7 — Protocol evidence closure (separate research gate)

This work is not a prerequisite for offline profiles and does not authorize hardware testing.

**Tasks**

1. Obtain a provenance-verified firmware image for the exact keyboard/receiver hardware and firmware revision, if one exists.
2. Resolve frontend command dialect/model binding for each candidate separately.
3. Resolve ordinary RPC route-to-handler mapping and complete native request/response conversion.
4. Recover device-side dispatch, argument validation, response framing and side effects from compatible firmware or other authoritative evidence.
5. Resolve the `0x89` RGB/keymap collision using full request context and firmware dispatch.
6. Establish the correct battery/online producer; do not reuse the unrelated model battery parser or `watchSystemInfo` host telemetry.
7. Recover a TH80-specific Fn-keymap path before modeling it as supported.
8. Record evidence and confidence for every request/response field; no fabricated response samples.

**Acceptance criteria**

- Each operation has a complete, target-specific path from model/call site through transport and response parser to device-side semantics.
- Request/response lengths, endianness, report-ID convention, status/error behavior and sequence preconditions are established.
- Persistent/volatile side effects and failure/recovery behavior are assessed.
- `UNKNOWN`, `HYPOTHESIS`, and `STRONGLY SUPPORTED` operations remain unavailable for active use.
- A separate explicit safety review and user authorization exists before any hardware experiment or transport implementation.

### P8 — Narrow transport integration (blocked; separate authorization required)

No work in this section is authorized by Phase 9. It is a future gate, not an automatic next phase.

**Preconditions**

- Command-specific evidence and side-effect review satisfy P7.
- Exact device model, firmware revision and interface/report contract are supported.
- Platform-specific API behavior is understood; macOS `wIndex=0` discrepancy is resolved before adopting that path.
- The user explicitly authorizes a separate hardware communication phase and its exact scope.

**Design requirements if later approved**

- No generic raw report API exposed to ordinary application/UI code.
- Typed allow-listed operations only, with per-target capabilities.
- Explicit instance/interface selection; no silent fallback or automatic probing.
- Clear cancellation, timeout, disconnect and partial-transfer errors.
- Separate Feature GET/SET, Output and Input operations; no API semantic conflation.
- Fake-transport tests plus offline protocol fixtures; no implicit real-device tests in `cargo test`.
- Destructive/update/bootloader/pairing/reset operations remain out of scope unless independently reviewed and authorized.

## 4. Protocol and feature status

### 4.1 Existing 12 candidates

The Phase 7B candidate identities are preserved exactly in the Phase 8 matrix:

| ID | Operation | Current product status | Reason |
|---|---|---|---|
| C01 | Firmware revision | BLOCKED | Frontend builder/parser found; SET-before-GET semantics and device-side dispatch unknown. |
| C02 | Device information | BLOCKED | `0x8f` constant only; no TH80 builder/call site/parser. |
| C03 | Battery query | BLOCKED | Parser/method belongs to another model; TH80 implementation not established. |
| C04 | Battery/online stream | BLOCKED | `Status24` producer/stream mapping unresolved; `watchSystemInfo` is unrelated host telemetry. |
| C05 | Active profile | BLOCKED | Frontend parser/cache behavior found; command side effects unknown. |
| C06 | Report rate | BLOCKED | Frontend request/parser found; firmware semantics and effects unknown. |
| C07 | Keyboard options | BLOCKED | Frontend flags found; device-side meanings/effects unknown. |
| C08 | RGB/lighting mode | BLOCKED | Frontend fields found; device-side enum/mode effects unknown. |
| C09 | Per-key RGB/user picture | BLOCKED | `0x89` request path found; device dispatch and effects unknown. |
| C10 | Keymap | BLOCKED | `0x89` paged request path found; collides with C09 and firmware dispatch unknown. |
| C11 | Fn keymap | BLOCKED | No TH80-specific builder/parser established. |
| C12 | Macro data | BLOCKED | Frontend request/parser found; device-side framing, bounds and effects unknown. |

No candidate is statically established read-only. Do not implement them as production commands in the offline phases.

### 4.2 Product features that are not blocked

These can be built without device command semantics:

- Passive device/receiver identity and descriptor diagnostics on supported host platforms.
- Local profile creation and validation.
- Local keymap/layer/macro/lighting data editing where the abstract data model is explicitly marked as not device-encoded.
- Versioned profile import/export.
- Offline comparison/diff of local profile documents.
- Capability display based only on descriptor evidence, with explicit `unknown` entries.

A local editor may need an abstract model before physical geometry or encoding is known. Such a model must not imply that the keyboard accepts the result.

## 5. Dependency decisions

| Area | Recommendation | Add dependency now? |
|---|---|---|
| Domain serialization | Existing `serde` + `serde_json`; version the schema | No new dependency. |
| CLI parsing | Keep current small parser until commands become unwieldy; consider `clap` then | No. |
| Async | Synchronous filesystem/profile workflows | No runtime. |
| HID discovery | Reuse sysfs provider; future per-OS adapters | No transport library. |
| HID transport | Not in initial product; review HIDAPI/platform APIs only after P8 authorization | No. |
| UI | Prototype Slint and egui; evaluate Tauri only with a clear web frontend need | No selection/dependency now. |
| Persistence | Files first; no DB | No. |
| Fuzz/property tests | Evaluate development-only tooling after parser boundary is stable | Not in Phase 9. |
| Logging/telemetry | Standard errors and structured diagnostic output | No telemetry SDK. |

The root `hidapi` dependency currently exists for the separate root `th80ctl` executable and brings native Linux dependencies. Keep it out of core and the `th80-probe` library. Any later removal or relocation must preserve legacy diagnostic behavior deliberately and should be a separate source-change task.

## 6. Explicitly blocked work

The following are not implementation tasks for the initial product and remain blocked pending evidence plus separate authorization:

- Any C01–C12 command transmission or response polling.
- Profile apply, active profile selection or report-rate/keyboard-option writes.
- RGB effect changes, per-key RGB upload, lighting mode changes or streaming control.
- Keymap/Fn-keymap writes, macro upload/deletion and persistent configuration changes.
- Pairing, calibration, reset, mode switch, bootloader, firmware update/erase/recovery.
- Generic raw Feature/Output report UI or command console.
- Any hardware integration test, including one that claims to be read-only.
- Treating `X` as firmware or deriving commands from it without format/target/compatibility proof.

## 7. Risks, dependencies and ordering

Critical dependency chain:

```text
Existing sysfs probe + parser
          │
          ▼
Shared core/discovery models
          │
          ▼
Offline profile schema + validation
          │
          ▼
Import/export + safe CLI workflows
          │
          ├──────────────► Cross-platform passive discovery
          │
          └──────────────► UI prototype and measured framework choice

Separate evidence track (no device I/O)
          │
          ▼
Target-specific command semantics + side-effect proof
          │
          ▼
Separate safety review + explicit authorization
          │
          ▼
Potential narrow transport phase (not assumed)
```

The offline product work can progress independently of protocol reverse engineering. This prevents the project from becoming an endless investigation while keeping unsafe assumptions out of the application.

## 8. Release gates

### Gate A — Offline CLI preview

- Linux sysfs discovery works with no device attached and with unknown devices present.
- Descriptor parser and capability output have stable tests and explicit evidence provenance.
- Profile validate/import/export work entirely offline.
- No active transport dependency is linked into the safe CLI.
- `cargo fmt`, tests, Clippy and release build pass.

### Gate B — Offline desktop preview

- Framework decision has measured startup/RSS/package-size results.
- UI uses application/domain APIs only.
- Local state and device state are visually distinct.
- No background daemon, network dependency or active device communication.

### Gate C — Cross-platform passive release

- Linux, Windows and macOS discovery adapters are implemented and tested on their respective systems.
- No OS adapter sends report transfers or claims interfaces.
- Errors and unavailable metadata are represented consistently.

### Gate D — Device command capability (not scheduled/approved)

- Per-operation evidence and side-effect criteria from P7 satisfied.
- Separate explicit authorization obtained.
- No blanket activation of a generic transport; only narrowly scoped operations reviewed.

## 9. Phase 9 acceptance checklist

- [x] Existing workspace, README, Phase 5–8 evidence, Rust packages, CLI, transport, device/protocol models, tests and fixtures audited.
- [x] Reusable versus research-only components identified.
- [x] Minimal modular architecture proposed without requiring all conceptual crates immediately.
- [x] Existing and candidate dependencies evaluated; no new dependency proposed for Phase 9.
- [x] Linux-first passive discovery and later Windows/macOS strategy defined.
- [x] Offline profile-first product scope defined.
- [x] Evidence confidence and command safety boundaries preserved.
- [x] Prioritized tasks, dependencies, acceptance criteria and blocked tasks recorded.
- [x] No source implementation, transport activation, firmware change or hardware communication performed.

## 10. Phase 9 stop condition

This roadmap ends with offline-first architecture, local profile workflows and passive cross-platform discovery as the next implementation priorities. It does not authorize any C01–C12 operation, active transport, firmware action, or hardware testing. Stop after Phase 9; a later implementation phase must be explicitly requested.
