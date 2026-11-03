# Phase 12.1 — Final GUI Closure and RGB Investigation

**Outcome: Phase 12 GUI closure retained; RGB investigation completed at static/offline level. Hardware diagnosis remains unapproved.**

## 1. Completed GUI work

- Inspected the existing README, Phase 12 Entropy audit, architecture report, implementation report, GUI package, core lighting model and core validation. No GUI redesign or framework change was made.
- Confirmed the GUI remains a separate native `eframe`/`egui` package whose only project-crate dependency is `th80-core`. It does not depend on `th80-probe` or the legacy HID research package.
- Confirmed the existing profile lifecycle is routed through `ProfileService` and `th80_core::ProfileRepository`: create/list/load/save/delete/import/export/validate. The core remains responsible for profile JSON, validation, atomic persistence and stale-save conflict detection.
- Confirmed the dashboard, keymap, layers, macros, lighting, options and profile screens exist. Existing save/dirty handling, import/create deferral while dirty, delete confirmation, and core conflict errors were retained.
- Added a clear Lighting-screen notice: **“These settings are stored locally in the profile. They have not been applied to the physical keyboard.”** The screen now distinguishes `enabled=true`, `enabled=false`, and an unspecified `enabled` field rather than visually implying that an absent value means the device is off.
- No GUI control was added for device access, hardware status, RGB live state, or applying settings.
- README links to the Phase 12.1 RGB diagnostic and closure documents.

The core `LightingConfig` represents optional `enabled`, generic mode (`Static`, `Breathing`, `Wave`, `Reactive`, `Custom(String)`), optional normalized brightness/speed, one RGB color, logical-position per-key colors, and opaque JSON effect parameters. The validator applies only local policy: brightness/speed 0–100, non-empty/unique logical positions and non-empty effect parameter keys. These are not firmware limits. `LightingConfig` uses `deny_unknown_fields`; arbitrary unknown lighting object members are not silently retained. Opaque effect parameter values and custom mode strings are retained as modeled. This is not a live view of the physical keyboard.

## 2. Verified test results

The required commands are run for this Phase 12.1 closure and their exact results are recorded below after execution:

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS (exit 0) |
| `cargo test --workspace` | PASS — 93 tests passed, 0 failed; doc-tests passed |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS (exit 0) |
| `cargo build --workspace --release` | PASS (exit 0) |
| `cargo doc --workspace --no-deps` | PASS (exit 0) |

Native Linux X11 smoke test: PASS. Launched `target/release/th80-gui` with a fresh temporary `TH80_PROFILE_DIR`; the Dashboard Create action produced `profile-1793549405510-0.json`. Inspected the generated identity/schema/lighting defaults with `jq`; `target/release/th80ctl profile validate` returned `Valid local profile ...; device compatibility: NOT VERIFIED`. `target/release/th80ctl --help` also exited 0. The temporary profile directory and GUI process were removed. This verifies launch/create/CLI validation only; it does not establish that every screen was manually exercised.

## 3. RGB investigation findings

- Vendor quick-start/manual material documents `Fn + Backspace` as backlight on/off; `Fn + \\` / `Fn + |` as effect toggle; `Fn + P` as color toggle; arrow shortcuts for brightness and speed. This is vendor-manual evidence, not a recovered firmware key handler. Exact hardware/firmware revision match for the user's keyboard is not established.
- Static TH80 frontend evidence identifies `0x87` as the strongest lighting-parameter query candidate. The frontend parses response bytes `[1]`–`[7]` as mode, speed encoding, value, option and RGB bytes. Firmware semantics and side effects remain unknown.
- Static TH80 frontend evidence identifies `0x89` in two distinct methods: per-key RGB/user-picture retrieval (`[1]=0`, additional reads) and keymap retrieval (`[1]=profile`, `[2]=page`, eight pages). Therefore `0x89` is not safely classifiable as an RGB-only opcode.
- The vendor frontend's common helper submits a Feature SET request before reading a Feature GET response. A `GET_*` name does not prove a read-only operation.
- No firmware default lighting state, persistent storage location, RGB controller, physical Fn+Backspace handler, or proven firmware dispatch for `0x87`/`0x89` was established.
- `setLightType(2)` appears adjacent to the vendor device-insertion lifecycle as a lighting-stream stop operation; it is not proven to be the RGB toggle and is not approved for testing.

See `PHASE12_1_RGB_DIAGNOSTICS.md` for evidence grades, opcode/request details, shortcut sources, safe offline workflow and the hardware approval boundary.

## 4. Unresolved hardware questions

- Exact PCB/MCU/RGB-controller revision and firmware build of the user's unit.
- Whether the user's firmware matches the vendor manual and inspected TH80 frontend model class.
- Whether current lighting state is volatile or stored in keyboard/receiver/controller/host storage.
- Whether the symptom is a profile/layer/keymap issue, a lighting state/mode/brightness issue, a firmware issue, or a physical RGB hardware fault.
- Device-side semantics, side effects, response framing and route mapping for `0x87`, both `0x89` request forms, and `setLightType`.

## 5. Next action required from the user

Before considering any hardware test, provide (if available) an exported vendor configuration/profile from before and after the symptom, confirm the exact TH80 Pro revision and current wired/2.4G/BT mode, and describe whether other Fn shortcuts work. Inspect those files offline only. Do not reset, pair, update firmware, send HID commands or run RGB protocol experiments as part of this step.

Any future hardware test requires a separate written test specification, evidence review, reversibility/risk assessment, and the user's explicit authorization for the specific operation. **No hardware test is authorized by this report.**

## 6. Safety and repository discipline

No physical HID device was enumerated or opened for Phase 12.1. No HID Feature GET/SET, Output report, USB control transfer, RGB command, reset, pairing, calibration, firmware update, bootloader operation or device-state change occurred. No real profile directory was accessed for testing. No Phase 13 work was started. No commits, remotes, pushes, releases or publication actions were performed. Historical Phase 5–11.3 reports were not edited.
