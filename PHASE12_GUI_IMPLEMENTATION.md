# Phase 12 — Native GUI Implementation Report

**Status: implemented; required workspace verification passed.** This report distinguishes automated verification from the limited manual GUI smoke test. The application is an offline profile editor, not a firmware configurator or a device communication tool.

## 1. Delivered architecture

- Added the `th80-gui` workspace package at `crates/th80-gui/`.
- Implemented a native desktop application using `eframe`/`egui` 0.35.0 with the Glow renderer and native Wayland/X11 support. Native file dialogs use `rfd` 0.17.
- The GUI's only project-crate dependency is `th80-core`. `ProfileService` is a thin adapter that forwards list/load/create/save/delete/import/export/validate operations to `th80_core::ProfileRepository`.
- `th80-core` remains the authority for profile structures, JSON serialization, validation, repository locking, atomic persistence and stale-save conflict detection. No duplicate repository or validation implementation was introduced.
- The existing CLI remains a separate workspace package and was exercised after the GUI build.

`eframe`/`egui` was selected because the Entropy source audit found that its application is itself a native Rust `eframe`/`egui` application. This reuses a proven framework choice and broad UX concepts without copying Entropy implementation. `rfd` provides native file selection without a browser frontend or WebView. The GUI dependencies are isolated from `th80-core` and `th80-cli`.

## 2. GUI functionality

- **Dashboard:** create a new profile, import a JSON profile through a native file picker, refresh and browse profiles ordered by most recent modification, and open a selected profile. It does not maintain a separate recently-opened history.
- **Keymap:** select a layer, add/remove logical position IDs, inspect assignments, and edit the core `Action` variants (`Key`, `Modifier`, `Combination`, `LayerSwitch`, `LayerMomentary`, `Macro`, `Transparent`, `Unassigned`). Opaque key/action names are not presented as a firmware keycode catalog.
- **Layers:** list, select, add, rename and remove layers. Removal is blocked while another assignment references the layer.
- **Macros:** create, select, rename, edit and delete macros; add/remove/edit the core `KeyDown`, `KeyUp`, `Delay` and `Text` steps; show core validation diagnostics.
- **Lighting:** edit generic optional enable/mode/brightness/speed/color values, per-logical-position colors and opaque JSON effect parameters. Unknown custom mode names are retained. The screen explicitly states that settings are stored locally and have not been applied to the physical keyboard, and distinguishes enabled/disabled/unspecified values. The primary-color editor also distinguishes an absent color from an explicitly stored white color. Values are not verified firmware capabilities.
- **Options:** edit application-owned JSON option values with persistent text drafts, parse-error feedback, and a read-only view of top-level profile extension fields.
- **Profile management:** save, save as, export, validate, and delete with confirmation. Import/open/create paths guard dirty drafts with save/discard/cancel handling. Stale-save errors are surfaced from the core repository; the GUI does not overwrite/retry conflicts automatically.
- **Status and feedback:** persistent offline-only indication, selected profile/storage path, dirty/saved state, structured core error codes and validation messages.

The keymap editor intentionally uses a model-driven logical-position list rather than a fabricated TH80 keyboard drawing. No verified TH80 matrix geometry, physical key coordinates, firmware keycode catalog or firmware key limits were available in the audited core/evidence.

## 3. Entropy audit and reuse

See `PHASE12_ENTROPY_REFERENCE_AUDIT.md` and `PHASE12_GUI_ARCHITECTURE.md`.

- Audited upstream repository/source architecture, package manifest, license, app/state/storage, keymap/layout/key-picker, macro, lighting/settings, serialization and device/HID separation before selecting the framework.
- The audited Entropy source declares `GPL-3.0-or-later`; this repository is MIT-licensed. No Entropy application code, UI component, artwork, branding, icon, font, firmware model, Vial/RMK dependency, transport code or serialization format was copied, adapted, vendored or linked.
- Independently implemented UX concepts include native immediate-mode Rust UI, clear navigation, a central editing surface, layer-first key assignment, compact settings rows and explicit unsupported/offline states.
- No Slint fallback or unnecessary framework bake-off was performed.

## 4. Hardware and offline safety boundary

The GUI does not enumerate/open HID devices, enumerate USB devices, send or receive reports, perform USB transfers, issue Feature GET/SET, access a keyboard/receiver, flash firmware, enter bootloader mode, pair, calibrate, reset or change device state. No hardware-operation buttons or disabled hardware placeholders are present.

The GUI manifest depends on `th80-core`, `eframe`, `rfd` and `serde_json`; `tempfile` is a test-only dependency. A normal dependency-tree review found no `hidapi`, `th80-probe`, `rmk-types`, `rusb`, USB, Vial or QMK packages. Source review found no transport or device-control code in `crates/th80-gui/`. The GUI was smoke-tested only against a temporary profile directory. No real profile directory was accessed, and **no hardware communication occurred**.

## 5. Tests

Eight `th80-gui` unit tests cover:

1. Core-model edits (logical key assignments, layers, macros and custom lighting mode) round-trip through the core repository.
2. Core macro validation diagnostics are returned without GUI-side validation duplication.
3. Core normalized lighting validation rejects an out-of-range brightness value.
4. Unknown top-level extension JSON and application option JSON survive save/load.
5. Repository import/export and stale-writer conflict behavior.
6. Structured not-found and malformed-JSON repository errors.
7. In-memory dirty-state detection and returning to the saved baseline.
8. Dashboard ordering by most recently modified profile.

All test fixtures use `tempfile` directories and synthetic profiles. The test suite does not touch the user's profile store or physical hardware.

## 6. Verification results

The following required commands were run from the workspace root; each exited successfully (exit code 0):

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo test --workspace` | PASS; all workspace test binaries and doc-test suites completed successfully |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo build --workspace --release` | PASS |
| `cargo doc --workspace --no-deps` | PASS; workspace documentation generated |

Additional checks:

- `cargo test -p th80-gui -- --list` reports 8 GUI tests.
- `target/release/th80ctl --help` exited successfully.
- Native GUI window launch was observed on Linux under Wayland and X11.
- In the final X11 smoke test, the GUI's **Create profile** action generated a profile JSON file in a fresh temporary directory. `jq` inspected its ID/name/schema version, and `target/release/th80ctl profile validate <temporary-profile.json>` returned: `Valid local profile ...; device compatibility: NOT VERIFIED`.
- The final smoke test did not manually verify every screen or every editing action. No Windows or macOS GUI launch/interaction verification was performed.

## 7. Limitations and remaining work

- No verified physical TH80 key geometry, matrix, firmware keycode catalog, supported lighting modes/ranges or firmware option schema is available. The GUI therefore edits generic core data only and cannot assert firmware compatibility.
- The dashboard orders profiles by modification time; it does not maintain a separate recently-opened history.
- The X11 smoke test verified launch, profile creation and CLI validation only. Import/export, save-as, deletion, layer/macro/lighting/options editing and unsaved-change dialogs are covered by code review and relevant core/service tests where applicable, but were not all manually exercised through the GUI.
- Automated GUI interaction tests and Windows/macOS GUI verification remain future work.
- Local application validation does not establish that a profile can be applied to the physical keyboard.

## 8. Repository and publication status

The README now documents the native GUI build/run commands and offline limitations. The three Phase 12 reports are:

- `PHASE12_ENTROPY_REFERENCE_AUDIT.md`
- `PHASE12_GUI_ARCHITECTURE.md`
- `PHASE12_GUI_IMPLEMENTATION.md`

No historical Phase 5–11.3 report was intentionally modified. No Git commit, staging, remote creation, push, release or publication action was performed.
