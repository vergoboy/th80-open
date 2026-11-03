# Phase 13 — Application RGB Roadmap

**Status:** roadmap only. No hardware transport, RGB implementation or GUI architecture change was made in Phase 13.

## 1. Current application boundary

The existing application remains:

- Rust native GUI using `eframe`/`egui`;
- `th80-core` as the sole authority for offline profile structures, validation, JSON representation and persistence;
- `th80ctl` as the offline CLI profile manager;
- separate passive diagnostic tools;
- no hardware transport in the GUI.

Phase 12.2 confirms the GUI cannot apply lighting to the physical keyboard. Its normalized brightness/speed scale, generic effect names, 8-bit RGB model and logical per-key positions are application abstractions, not verified TH80 firmware values or geometry.

Phase 13 found no firmware-side RGB implementation or proven command contract. The roadmap therefore separates offline profile work from any future device capability.

## 2. Stage A — offline RGB profile editing

**Can proceed without hardware protocol assumptions.**

1. Keep all edits local and label them clearly as profile drafts; never present them as device state or applied settings.
2. Preserve optional-field semantics (`None` versus explicit false/zero/color) and the Phase 12.2 color-presence indicator.
3. Keep generic `LightingMode`, normalized brightness/speed and RGB8 values explicitly documented as application-level representations.
4. Preserve per-key colors as logical positions. Do not imply that logical position IDs correspond to physical matrix coordinates or the TH80's actual LED ordering.
5. Improve forward compatibility only through an explicit schema decision: either preserve unknown nested lighting extensions safely or reject them with an actionable error. Do not silently drop unknown vendor fields.
6. Add import/export fixtures only from user-supplied or clearly documented sample profiles. Distinguish known vendor fields from inferred fields and do not fabricate firmware-native values.
7. Continue round-trip, absent/false/zero, unknown-field and profile-open non-mutation tests using temporary directories only.

**Exit criteria:** offline profile editing is predictable, loss-aware and visibly distinct from physical device configuration. No transport dependency is required.

## 3. Stage B — passive/read-only diagnostics

This stage must begin with static proof, not trial-and-error.

Required evidence before a command can be called read-only:

1. exact board revision, MCU/LED controller and installed firmware identity;
2. firmware-side request parser/dispatch and complete response path for the exact operation;
3. proof that the request does not alter persistent configuration, volatile lighting state, profile selection, mode, pairing, calibration or boot state;
4. exact interface, report ID, Feature length, request/response sizes, checksum and selector rules;
5. proven frontend/native RPC route-to-handler mapping and correct interface index (the existing `wIndex=0` versus interface-2 mismatch must be resolved);
6. error/status handling, retry behavior, timeout and device-path selection;
7. independent review and explicit authorization for a narrowly scoped test.

Current `0x87` and both `0x89` paths fail this gate. The frontend sends Feature SET before Feature GET. None is `READ_ONLY_PROVEN`.

A future diagnostic layer should expose only operations with a command-specific proof record, explicit device/model/firmware applicability, strict length/bounds checks and a no-write guarantee. Until then, diagnostics remain passive: local files, known USB descriptors already exported, board photographs, package metadata and offline profile validation.

## 4. Stage C — live RGB control

**Blocked pending firmware and hardware evidence.**

Minimum prerequisites:

- exact PCB and MCU/LED-driver identification;
- exact installed firmware version/build and compatibility with the inspected image;
- recovered firmware handler for the frontend's lighting setter (`0x07`) and its state structure;
- verified semantics and field ranges for mode, speed, value, option and RGB;
- verified persistence owner and behavior across restart, USB, 2.4 GHz, Bluetooth and sleep/wake;
- proven native transport path and correct interface selection;
- explicit acknowledgement/readback semantics and error handling;
- backup/restore strategy and a reversible, user-reviewed recovery path;
- command-specific risk review and explicit user authorization.

Only after those prerequisites should a separate optional device backend be considered. Keep it out of `th80-core`'s offline model and out of GUI rendering. Use a narrow typed command interface, explicit device selection, capability gating and visible confirmation before writes. Do not expose arbitrary raw Feature reports as an RGB API.

## 5. Stage D — firmware-specific features

Firmware-specific capability data must be versioned and evidence-backed. Potential future items include:

- actual TH80 firmware mode enumeration and supported mode names;
- valid speed/brightness/value ranges and option-bit meanings;
- color presets and RGB channel ordering;
- custom per-key RGB picture dimensions, page/block count and matrix-to-HID mapping;
- lighting enable/off state and persistence behavior;
- Fn-layer keymap and shortcut semantics;
- sleep/wake, battery/power and transport-transition interactions.

Each feature should have a compatibility tuple such as `{vendor, product, PCB revision, MCU, firmware family/version, transport}` and an evidence grade/source. Never copy values from a related YZW model or another TH80 revision without proving applicability.

## 6. Suggested capability evidence model

For future metadata, retain the distinction between:

- `STATIC_ONLY`: local binary/document evidence only;
- `FRONTEND_OBSERVED`: vendor frontend request or parser observed;
- `FIRMWARE_CORRELATED`: request is tied to a concrete firmware handler and response/state behavior;
- `READ_ONLY_PROVEN`: no state mutation established across the complete operation;
- `WRITE_OPERATION`: operation is established as a write;
- `UNKNOWN`: material semantics or side effects remain unresolved.

Evidence grade is not permission. Even `FIRMWARE_CORRELATED` does not authorize hardware access. A later test still requires explicit approval.

## 7. Explicit non-goals for Phase 13

- no hardware transport dependency;
- no `hidapi`, `hidraw`, libusb or Feature GET/SET integration;
- no live device state in the GUI;
- no command execution for `0x07`, `0x87`, `0x89` or `setLightType`;
- no reset, pairing, calibration, updater or firmware workflow;
- no GUI redesign or architecture change;
- no changes to the user's real profile directory;
- no firmware modification;
- no commits, pushes, remotes, releases or Git history changes.

## 8. Recommended next project input

Before designing a hardware backend, obtain a clear photograph of the exact PCB's main MCU and any LED-driver/controller markings, together with the board revision, if already safely accessible. This is the most valuable missing identity evidence because it tests whether the YC3121-L/BK3632 family lead applies to this unit and whether a separate LED controller exists. Do not disassemble the keyboard solely to obtain it if that would be uncomfortable or risky.

After hardware identity is established, the next static target should be a firmware image/package explicitly tied to that exact PCB and installed version, plus trusted firmware source/header material or a linker map if available.

## 9. References

- `README.md`, `PHASE9_PRODUCT_ARCHITECTURE.md`, `PHASE9_IMPLEMENTATION_ROADMAP.md`
- `PHASE10_OFFLINE_CORE_IMPLEMENTATION.md`
- `PHASE11_OFFLINE_ENGINE_HARDENING.md`, `PHASE11_1_REPOSITORY_RELIABILITY_CLOSURE.md`
- `PHASE12_GUI_ARCHITECTURE.md`, `PHASE12_GUI_IMPLEMENTATION.md`, `PHASE12_2_LIGHTING_APP_AUDIT.md`
- `PHASE12_1_RGB_DIAGNOSTICS.md`, `PHASE13_FIRMWARE_RGB_ANALYSIS.md`, `PHASE13_VENDOR_PROTOCOL_RGB_CORRELATION.md`, `PHASE13_RGB_ROOT_CAUSE.md`
