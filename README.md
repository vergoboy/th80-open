# TH80 Open

Native Rust tooling for the EPOMAKER TH80 family, including an **offline desktop configurator**, an offline profile CLI, and separate legacy hardware-research utilities.

The current user-facing product is **TH80 Studio**, a native `eframe`/`egui` desktop application for creating, editing, validating, importing, exporting and organizing local JSON profiles. It does not require a connected keyboard and does not communicate with a device. Keymap positions are logical identifiers, not a claimed TH80 physical matrix. Lighting, macros and options are local profile data; their physical firmware support is not established.

The `th80-core` crate owns the profile model, validation and atomic repository. `th80-cli` (`th80ctl`) and `th80-gui` use this same core. The separate `th80-probe` and root `th80-hid-research` tools are legacy research utilities and are not dependencies of the GUI or offline profile workflow.

## Current product status

- **Available today:** offline profile creation, duplication, rename, deletion, import/export, logical layers and assignments, ordered macros, generic lighting preferences, application-owned JSON options, validation, unknown top-level extension preservation and conflict-aware persistence.
- **Not available:** physical keyboard read/write, live device state, RGB application, firmware updates, bootloader operations, pairing, reset, calibration or replacement firmware.
- **Compatibility:** local schema validation is not firmware validation. The exact TH80 board MCU/variant, bootloader acceptance model and configuration protocol remain unresolved; see [Phase 13.12 findings](PHASE13_12_FINDINGS_AND_NEXT_STEPS.md).

See the [feature support matrix](docs/feature-support.md), [profile format](docs/profile-format.md), [CLI usage](docs/cli.md), [hardware integration readiness](PHASE14_HARDWARE_INTEGRATION_READINESS.md), [hardware inventory](HARDWARE.md), [research status](RESEARCH.md) and [safety policy](SAFETY.md).

## Build and run TH80 Studio

Requirements: Rust stable toolchain, Cargo, a native Linux graphical session (Wayland or X11) and the native graphics/windowing development libraries required by `eframe` on the target distribution. No Node.js, Electron or WebView runtime is used.

```sh
cargo run --locked -p th80-gui
cargo build --locked -p th80-gui --release
```

The release executable is `target/release/th80-gui`. Profiles are stored in `$TH80_PROFILE_DIR` when set; otherwise the GUI uses the platform application-data location (normally `~/.local/share/th80-profiles` on Linux). To run with an isolated store, point `TH80_PROFILE_DIR` at a directory you choose:

```sh
TH80_PROFILE_DIR="$(mktemp -d)" cargo run --locked -p th80-gui
```

Do not use a temporary directory for profiles you want to keep. See [installation and development notes](docs/installation.md).

## Offline profile CLI

`th80ctl` provides the same core profile model and repository operations without a GUI:

```sh
cargo run --locked -p th80-cli -- profile create --name "Daily"
cargo run --locked -p th80-cli -- profile list
cargo run --locked -p th80-cli -- profile duplicate PROFILE_ID
cargo run --locked -p th80-cli -- profile rename PROFILE_ID --name "Work"
cargo run --locked -p th80-cli -- profile validate PROFILE.json --output json
```

See [CLI usage](docs/cli.md) for keymap, macro, lighting, options, import/export and validation commands. Set `TH80_PROFILE_DIR` to isolate CLI operations and tests.

## Phase 14 productization reports

- [Application audit and feature matrix](PHASE14_APPLICATION_AUDIT.md)
- [Implementation report](PHASE14_IMPLEMENTATION_REPORT.md)
- [Test and platform report](PHASE14_TEST_AND_PLATFORM_REPORT.md)
- [Hardware integration readiness](PHASE14_HARDWARE_INTEGRATION_READINESS.md)

## Phase 14.2 — Entropy-inspired native UI redesign

The current desktop workspace uses three primary sections (**Layout / Advanced / Config**) and a native, searchable action picker. Layout is the startup workspace; a single unambiguous local profile opens automatically, while multiple profiles still require an explicit selection. The board displays profile-defined **logical positions**, not verified physical TH80 switch geometry. The picker exposes only the eight action variants supported by `th80-core`, grouped into Keys, Modifiers, Layers, Macros and Other. Advanced contains the existing macro editor; Config contains local lighting, keyboard options, profile management and application preferences. Light/dark theme, accent and interface scale preferences are stored locally.

This remains an offline profile editor: Save writes the local profile only. Lighting and keyboard options are local profile data and are not sent to a physical keyboard. No Entropy source, assets, branding, Vial models or device transport are included. Phase 14.2's core UI implementation and automated Rust checks are complete; matched-size visual acceptance and some native accessibility/platform verification remain open. The implementation is Entropy-inspired, not pixel-equivalent. See the visual and test reports for the evidence and limitations.

- [Entropy reference and design specification](PHASE14_2_ENTROPY_REFERENCE_AND_DESIGN_SPEC.md)
- [UI implementation report](PHASE14_2_UI_IMPLEMENTATION_REPORT.md)
- [Visual comparison report](PHASE14_2_VISUAL_COMPARISON.md)
- [Test and regression report](PHASE14_2_TEST_AND_REGRESSION_REPORT.md)

## Phase 5A diagnostic CLI

A standalone passive USB/HID inventory tool is available as `th80-probe` (see [Phase 5A documentation](PHASE5A_DIAGNOSTIC_CLI.md)). It reads Linux USB sysfs metadata and readable HID report-descriptor files; it does not open USB/HID devices or send reports.

```sh
cargo run --locked -p th80-probe -- list
cargo run --locked -p th80-probe -- inspect --device keyboard
cargo run --locked -p th80-probe -- inspect --device receiver
cargo run --locked -p th80-probe -- inspect --json
cargo run --locked -p th80-probe -- capabilities --all
cargo run --locked -p th80-probe -- capabilities --json
```

Phase 5B adds passive HID report-descriptor parsing, report-length calculations, SHA-256 fingerprints and JSON capability profiles. See [Phase 5B HID capability discovery](PHASE5B_HID_CAPABILITY_DISCOVERY.md) for parser behavior, captured keyboard/receiver report definitions and limitations. The capability path only parses descriptors already exposed through sysfs; it does not send HID or USB requests.

Phase 5C correlates those descriptors with the macOS Feature transport, frontend RPC and Windows updater. See [Feature report correlation](PHASE5C_FEATURE_REPORT_CORRELATION.md), [wIndex analysis](PHASE5C_WINDEX_ANALYSIS.md), [RPC dispatch resolution](PHASE5C_RPC_DISPATCH_RESOLUTION.md), and [updater correlation](PHASE5C_UPDATER_CORRELATION.md). The `wIndex=0` implementation is confirmed but its reason/validity for interface 2 remains unresolved; raw URI-to-native-poll mapping remains strongly supported rather than conclusive; updater packet equivalence is not established. Phase 5C is static-only and does not authorize active protocol tests.

Phase 5D extends the static transport audit. See [HIDAPI/IOKit Feature audit](PHASE5D_HIDAPI_FEATURE_AUDIT.md), [wIndex resolution](PHASE5D_WINDEX_RESOLUTION.md), [Feature buffer layout](PHASE5D_FEATURE_BUFFER_LAYOUT.md), [Windows updater buffer](PHASE5D_WINDOWS_UPDATER_BUFFER.md), and [RPC dispatch audit](PHASE5D_RPC_DISPATCH_AUDIT.md). No HIDAPI/IOKit Feature path was identified; `wIndex=0` is hardcoded and copied directly into the USB setup packet, but its reason remains unknown. The macOS logical SET buffer starts as an empty `BytesMut`, appends the caller slice, resizes to at least 66 bytes, applies selector-specific checksums over offsets 1–7 or 1–8, and sends the first 64 bytes. GET returns a fixed 64-byte buffer even after a positive short transfer. Windows prepends `0x01`, appends a dynamic payload, resizes to `max(65, payload_length+1)`, and checksums the complete source payload; checksum destinations are one byte later, but checksum domains differ. This is structural correlation, not proven packet equivalence or proof of firmware-updater use. Native routing now proves raw URI → Tonic wrapper polls (`0x1000a0f50` SET, `0x100092ec0` GET); the final indirect wrapper → handler edge remains unresolved. Phase 5D remains static-only; no hardware I/O or updater execution occurred.

Phase 6 adds a reusable Rust foundation in `th80-probe` with typed device identities, capability profiles, separate pure-memory macOS/Windows Feature packet builders, a shared error model and an explicitly disabled transport contract. The existing CLI is preserved. No active USB/HID backend is connected; see [Phase 6 driver foundation](PHASE6_DRIVER_FOUNDATION.md).

Phase 7A adds an offline protocol regression harness covering the recovered macOS/Windows buffer layouts, checksum branches, length boundaries, simulated short GET responses, malformed inputs, unknown command semantics and disabled-transport guarantees. Captured descriptor fixtures are explicitly distinguished from synthetic packet payloads; no captured Feature traffic is claimed. See [Phase 7A offline protocol tests](PHASE7A_OFFLINE_PROTOCOL_TESTS.md).

## Legacy HIDAPI research utility

Requirements: Rust stable, Cargo, and the Linux HIDAPI development/runtime dependencies required by `hidapi`.

```sh
cd th80-open
cargo run --locked --bin th80-hid-research --release
```

This legacy root-package utility uses HIDAPI for Linux enumeration and retains an opt-in raw input-capture path. It is separate from both `th80ctl` profile workflows and the passive `th80-probe` sysfs utility. It has no Feature-report read/write, Output-report write, arbitrary control-transfer, or firmware-flashing operation. Input capture can record typed text; treat output as sensitive.

Available commands:

- list — enumerate supported keyboard and receiver HID interfaces.
- descriptors — print parsed reports and raw report-descriptor bytes.
- diagnostics FILE — write a JSON diagnostic snapshot.
- record-input --pid 4010|4011 --seconds N --out FILE — opt-in, bounded raw input capture from the standard boot-keyboard interface.

Input recordings can contain typed text. Treat them as sensitive.

## Offline profile manager (Phase 11)

`th80ctl` is the offline profile-management CLI backed by the reusable `th80-core` Rust library. It does not enumerate, open or communicate with a keyboard or receiver. The legacy HIDAPI research executable remains separate as `th80-hid-research`; it is not used by profile workflows.

```sh
cargo run --locked -p th80-cli -- profile create --name "Daily"
cargo run --locked -p th80-cli -- profile list
cargo run --locked -p th80-cli -- profile show PROFILE_ID --format text
cargo run --locked -p th80-cli -- profile add-layer PROFILE_ID --layer-id base --name Base
cargo run --locked -p th80-cli -- profile set-key PROFILE_ID --layer base --position logical-01 --key KEY_A
cargo run --locked -p th80-cli -- profile macro add PROFILE_ID --macro-id tap --name "Tap"
cargo run --locked -p th80-cli -- profile macro step-add PROFILE_ID --macro-id tap --step key-down:KEY_A --step key-up:KEY_A
cargo run --locked -p th80-cli -- profile lighting set PROFILE_ID --enabled true --mode static --brightness 50
cargo run --locked -p th80-cli -- profile lighting key-set PROFILE_ID --position logical-01 --color 255,80,20
cargo run --locked -p th80-cli -- profile options set PROFILE_ID --key local.preference --value '{"enabled":true}'
cargo run --locked -p th80-cli -- profile validate profile.json --output json
```

Profiles are versioned JSON in `$TH80_PROFILE_DIR`, or by default in `~/.local/share/th80-profiles`. See [CLI usage](docs/cli.md), [core library API](docs/core-api.md), and [Phase 11 hardening report](PHASE11_OFFLINE_ENGINE_HARDENING.md). The profile model supports logical key positions/layers, typed actions, macros, generic lighting and application-owned extension options. These are offline editing abstractions, not confirmed firmware encodings or proof that any value is accepted by the keyboard.

The core keeps application policy, device capability evidence and firmware constraints separate. The current macro and normalized-lighting bounds are local application policy only. Capability evidence retains an evidence grade and source references; firmware constraints have a separate evidence record and remain absent/unknown when no reliable value is established. Local validation does not establish device compatibility; the current TH80 Pro assessment remains `NotVerified`.

The core crate is currently `0.1.x`; see its [public API and compatibility policy](docs/core-api.md). The schema version is independent of the crate version. Repository mutation uses per-profile exclusive lock files, stale-save detection and same-directory temporary files. Existing-profile replacement uses same-directory rename on Unix and `ReplaceFileW` on Windows; a failed Windows replacement does not delete the destination first. File contents are synchronized before replacement; Unix parent-directory synchronization is attempted and errors are surfaced after the new file is visible. Windows does not claim directory-entry durability after power loss. A process crash before replacement leaves the old profile authoritative; after replacement the complete new profile is authoritative, although durability may be uncertain. Abandoned lock files fail closed and require explicit operator recovery. See the [Phase 11.1 reliability report](PHASE11_1_REPOSITORY_RELIABILITY_CLOSURE.md) and native [Linux/Windows/macOS repository CI](.github/workflows/repository-platform.yml) for platform-specific verification status.

## Native desktop profile editor (Phase 12)

`th80-gui` is a native Rust desktop application built with `eframe`/`egui`. It edits offline profiles through the same `th80-core::ProfileRepository`, profile model, validation, JSON serialization, atomic persistence, and stale-save protection used by `th80ctl`. The GUI does not implement a second repository or validation policy. It depends on `th80-core`, not `th80-probe` or the legacy HIDAPI research executable.

The editor is strictly offline: it does not enumerate, open, or communicate with a keyboard or receiver, and it has no firmware, bootloader, pairing, reset, calibration, or device-operation controls. Logical key positions are shown without asserting physical TH80 geometry or firmware key limits. Locally valid profile values do not imply firmware compatibility.

The Phase 14.2 interface is organized as Layout / Advanced / Config. Layout prioritizes a logical-position workspace, layer navigation and a selected-position inspector; the action picker groups only the eight core action variants. Advanced contains macros, while Config contains local lighting, generic options, profile management and application preferences. The application uses an explicit local Save action; it does not write to a connected keyboard. See the four Phase 14.2 reports linked above for reference analysis, implementation details, screenshots and verification status.

Run the desktop editor from the workspace root:

```sh
cargo run --locked -p th80-gui
cargo build --locked -p th80-gui --release
```

Profiles use `$TH80_PROFILE_DIR` when set; otherwise the application uses the platform data directory (on Linux, normally `~/.local/share/th80-profiles`). For an isolated profile store, set `TH80_PROFILE_DIR` to a directory you own before launching. See [Entropy reference audit](PHASE12_ENTROPY_REFERENCE_AUDIT.md), [GUI architecture](PHASE12_GUI_ARCHITECTURE.md), and [GUI implementation report](PHASE12_GUI_IMPLEMENTATION.md). Phase 12.1's offline [RGB diagnostics](PHASE12_1_RGB_DIAGNOSTICS.md) document vendor shortcut references and static protocol evidence; [Phase 12.1 closure](PHASE12_1_CLOSURE.md) records verification and safety boundaries. [Phase 12.2 lighting application audit](PHASE12_2_LIGHTING_APP_AUDIT.md) documents optional-field semantics, persistence checks and the primary-color presence indicator. The GUI's Lighting page only edits local profile data; it cannot read or apply physical keyboard lighting state.

## Future hardware integration boundary

No hardware transport is part of TH80 Studio. A future device layer is gated on exact board identity, firmware compatibility, protocol semantics, read/write acknowledgement and recovery evidence. The GUI must remain independent of HID/USB implementations; any later transport would be a separately reviewed component behind explicit capability and operation-authorization checks. See [Phase 14 hardware integration readiness](PHASE14_HARDWARE_INTEGRATION_READINESS.md).

## Safety

Do not use an updater or image intended for another TH80 revision or layout. Do not write to the bootloader until the exact board identity, image bounds, checksum rules, and recovery path have been verified. Keep the original vendor updater and firmware package available as recovery references.
