# Phase 12 — Entropy Reference Audit

**Audit status: COMPLETE.** This document records a source-and-documentation review performed before the TH80 GUI implementation. No Entropy source, artwork, branding, or device-communication code was copied.

## 1. Repository and license

- Upstream: [ergohaven/entropy](https://github.com/ergohaven/entropy)
- Product documentation: [entropy.tools](https://entropy.tools) and the [Ergohaven Entropy documentation](https://docs.eh.industries/software/entropy/)
- Audited source checkout: the local `entropy-audit` source snapshot in the sibling TH80 project directory, cross-checked against the public upstream repository and documentation.
- Audited manifest version: `0.5.0-test.4`.
- Package license declaration: `GPL-3.0-or-later`; the repository `LICENSE` contains GNU GPL version 3.
- Repository source currently exposes one application package/binary (`ergohaven-entropy` / `entropy`); the root `Cargo.toml` is a package manifest, not a Cargo workspace manifest with multiple first-party crates.

Entropy is described by its maintainers as a native desktop application for Vial-QMK and Vial-RMK programmable input devices. Its public compatibility and quick-start documentation explicitly assumes a compatible Vial interface and a connected device. That is materially different from this project's offline-only profile manager.

## 2. Relevant crates and dependencies

The application manifest identifies these principal dependencies:

| Dependency | Role in Entropy | TH80 relevance |
|---|---|---|
| `eframe` 0.34.3 (with a vendored patch) | Native egui application/window integration | Proven Rust-native immediate-mode GUI framework; suitable architectural reference |
| `egui` 0.34.3 | Immediate-mode UI widgets and painting | Suitable UI framework dependency; no Entropy source reuse required |
| `serde`, `serde_json` | Settings/layout serialization | Same ecosystem already used by `th80-core` |
| `rfd` | Native file dialogs on non-WASM targets | Optional UX pattern; TH80 may use its own direct dependency |
| `hidapi` | HID discovery and communication | **Must not be imported into the TH80 GUI** |
| `rmk-types` (pinned Git revision) | RMK-native action types | Firmware-specific; **must not be imported** |
| `postcard`, compression/image/emoji/font crates | Firmware data, layout serialization, rendering assets | Not required for the TH80 profile model |
| platform-specific tray, D-Bus, Win32, macOS and input-method crates | OS integration, device access and background features | Not required for the first offline GUI |

The GUI crate should depend on `th80-core` plus the selected UI and native-dialog crates only. It must not depend on `th80-probe`, the root HIDAPI research package, `hidapi`, `rmk-types`, or any Entropy transport crate/module.

## 3. GUI framework and rendering approach

Entropy uses `eframe` and `egui`, with `eframe::run_native` creating a native desktop window. The UI is immediate-mode: the application state is retained in Rust structs and each frame draws the current navigation page and controls. The manifest selects the Glow renderer and native Wayland/X11 integrations. The source is organized into one application binary with a substantial set of Rust modules rather than a separate browser frontend or WebView.

The source entry point (`src/main.rs`) constructs a native viewport and launches `EntropyApp` through `eframe::run_native`. The application implementation (`src/app.rs`) composes state, storage, update/repaint, layout, key-picker and page modules. `src/ui/settings_shell.rs` dispatches selected settings pages to individual screen renderers. `src/ui/layout_view.rs`, `layout_keyboard.rs`, `layout_layer_switcher.rs`, and `key_assignment.rs` implement the layout editing surface.

This architecture is a good fit for the TH80 GUI: a single native Rust process, no web frontend, direct Rust model binding, and a small application state machine. The TH80 implementation will use the current cached stable `eframe` release compatible with this environment rather than copy Entropy's vendored framework or pin its older version.

## 4. Navigation and screen structure

Entropy groups the primary keyboard layout workflow separately from configuration/settings pages. Its UI dispatches settings tabs to dedicated modules, while the main layout page owns the keyboard canvas, layer switcher, top tabs, key assignment interactions, and advanced layout operations. Device selection and firmware-gated page visibility are central to Entropy's connected-device workflow.

Reusable UX concepts:

- Persistent left/top-level navigation with a clear current-page state.
- One central editing surface with context-sensitive inspector/picker panels.
- Layer selection adjacent to the layout rather than buried in general settings.
- Compact settings rows with labels, current values, and inline controls.
- Distinct app preferences versus keyboard/firmware settings.
- Explicit empty, loading, error, and unsupported-capability states.

TH80 adaptation: navigation will represent Dashboard, Keymap, Layers, Macros, Lighting, Options, and Profile. There will be no device picker, connection indicator, firmware page, or transport action.

## 5. Keymap editor architecture

Entropy's `keyboard.rs` defines visual physical-key/encoder records with geometry (`x`, `y`, `w`, `h`), matrix coordinates, labels, rotation, and optional layout conditions. The keyboard editor consumes a firmware/device-specific `KeyboardLayout`; it is not a generic layout inferred from a key count. Key assignments use Entropy's own `KeyBinding` type and Vial/RMK-specific keycode models. The keycode picker is split into multiple modules for catalog, search, tabs, rows, keyboard input, special actions, macros and tap dance.

The key editor interaction pattern is useful: select a layer, select a key, inspect its current assignment, then choose a replacement from a searchable categorized picker. However, the physical geometry, matrix coordinates, keycode catalog, and assignment representation are not transferable to TH80 without independent evidence.

TH80 core currently models **opaque logical position identifiers** in `BTreeMap<String, KeyAssignment>`. It does not define TH80 physical geometry, matrix coordinates, key count, firmware keycode IDs, or a verified supported-key catalog. Therefore the first TH80 editor must render a model-driven logical-position list/grid, not a fabricated TH80 keyboard silhouette or matrix.

## 6. Layers

Entropy models layers as distinct assignments over a common physical layout and supports layer switching, names, previews, copy/paste/mirror operations, and firmware-specific layer actions. The documentation describes layer 0 as the normal base layer and includes Vial/RMK actions such as MO, TG, OSL, TT, TO, DF and LT.

TH80 core's `keymap::Layer` has `id`, `name`, and a map of logical position to typed `Action`. The supported generic actions are `Key`, `Modifier`, `Combination`, `LayerSwitch`, `LayerMomentary`, `Macro`, `Transparent`, and `Unassigned`. These are the only assignment kinds the GUI may expose. Vial-specific layer opcodes and actions must not be added.

## 7. Macro editor

Entropy's macro editor is associated with Vial/RMK keycode semantics and device macro slots. Its public documentation describes Text, Tap, Down, Up and Delay operations. Its internal keycode picker has macro-specific support and special handling for firmware keycodes.

TH80 core independently defines `MacroDefinition { id, name, steps }` and `MacroStep::{KeyDown, KeyUp, Delay, Text}`. The GUI can expose exactly those four typed operations, use the existing core validator for balanced key-down/key-up sequences and local resource limits, and display structured diagnostics. No Entropy macro serialization or firmware slot limits are applicable.

## 8. Lighting and keyboard settings

Entropy's lighting/RGB controls are part of its device-settings UI. The controls are driven by firmware-reported capabilities and setting schemas; unsupported controls are hidden or disabled based on device support. It supports multiple firmware-specific lighting effects, parameter ranges and per-key operations.

TH80 core provides a generic local `LightingConfig`: optional enabled/mode/brightness/speed/color, per-logical-position RGB values, and arbitrary JSON `effect_parameters`. Its `LightingMode::Custom(String)` allows unrecognized mode names. The normalized brightness/speed bound is application policy, not a firmware limit. The GUI must preserve and display custom modes and opaque parameter values, and must label these as local profile values rather than firmware-compatible controls.

Entropy's device settings include Vial/RMK-specific firmware options, Bluetooth controls, displays, touchpads, modules, layer LEDs, live features and firmware update workflows. None are suitable for import into this project absent independent TH80 evidence. TH80 `KeyboardOptions.values` are explicitly application-owned JSON options; their presence does not establish firmware support.

## 9. Persistence and serialization

Entropy has separate device layout backup/import/export formats (`.entlayout`) with explicit format versions and distinct device/application scopes. The source contains version checks and compatibility handling; application preferences are persisted separately under the platform configuration directory as JSON/settings data. Device state is acquired through its device/firmware layer, and layout export serializes a snapshot for later restoration.

TH80 must not adopt `.entlayout`, its versions, or its Vial/RMK serialization structures. The existing `th80-core::ProfileRepository` is authoritative and already provides JSON profile create/load/list/save/delete/import/export, schema validation, cross-process mutation locks, stale-save conflict detection and atomic replacement. GUI actions must call this repository directly. A GUI-local selected-profile/draft state is acceptable; a second on-disk repository is not.

## 10. Separation of UI, model and hardware transport

Entropy's code has identifiable domain/keyboard/device, UI, storage and HID modules, but the application is a single large package and its app state and screen code are coupled to firmware/device types. Its device layer performs enumeration and its HID layer contains Vial protocol implementation. RMK-native handling is also firmware-specific.

TH80 already has a cleaner boundary for this product: `th80-core` is hardware-independent and denies unsafe code; it contains profile models, validation and repository logic. `th80-gui` will be a separate consumer of this library. It will not import `th80-probe` or the legacy HIDAPI executable. No HID discovery, device enumeration, transport abstraction, firmware operation or hardware-related background task will be included in the GUI package.

## 11. License implications and reuse decision

Entropy's application source is GPL-3.0-or-later. Copying or modifying its source files, UI components, icons, assets, or other copyrightable implementation material and distributing the result would require careful compliance with the GPL's corresponding-source, notices, license and downstream-rights requirements. Combining/linking copied GPL-covered code into a distributed application may cause the combined work to be subject to GPL terms; the exact result depends on the actual combination and distribution facts. Obtain qualified legal advice for a distribution decision if code reuse is later proposed.

The TH80 repository is MIT-licensed. To avoid imposing a copyleft licensing change on the project and to avoid compatibility risks, this phase does **not** copy, adapt, link, vendor, or translate Entropy application source code. It does not use Entropy branding, logos, fonts, screenshots, icons, key catalogs, firmware models, `.entlayout` format, Vial/RMK dependencies, or hardware communication. It reuses only general UX/architecture ideas (native egui application, clear navigation, layer-first editing, contextual assignment inspector, explicit unsupported states), which are implemented independently. The `eframe`/`egui` crates are separately licensed framework dependencies; their own license notices remain applicable.

## 12. Features specifically excluded as Vial/RMK-specific

- Vial HID discovery, protocol messages, read/write transport, device connection lifecycle and device selection.
- QMK/Vial keycode numeric catalog, custom keycodes and Vial keycode-picker behavior.
- RMK `KeyAction` and `rmk-types` integration.
- Firmware slot indices, matrix coordinates, physical layouts and device-defined encoders.
- Combos, tap dance, key overrides, Auto Shift, Tap-Hold, One Shot, Magic, Grave Escape and firmware-specific settings.
- Device RGB capability discovery, Layer LEDs, display/touchpad/module configuration, Bluetooth and live host features.
- Firmware updater, bootloader, device reset, pairing, calibration and firmware-write behavior.
- Entropy `.entlayout` and `.entsettings` formats.

## 13. Recommended TH80 architecture

1. `th80-gui` is a native `eframe`/`egui` binary package in `crates/th80-gui/`.
2. `th80-core` remains the only authority for profile structures, validation, JSON representation and persistence.
3. A thin GUI application state owns the active `Profile` draft, selected page/layer/logical position, dirty flag, and structured UI feedback.
4. A profile-service adapter inside the GUI translates user actions into calls to `ProfileRepository`; it contains no duplicate serialization or validation implementation.
5. The keymap surface uses logical position IDs from the active layer and makes no claim about physical keyboard geometry.
6. Macro, lighting and options editors mutate only the corresponding public core model fields and call core validation before save.
7. Import/export uses the existing repository methods. Save conflicts require reload/review; the GUI never retries by overwriting a newer file.
8. Tests use synthetic profiles and temporary directories. No test uses a physical device or the user's actual profile directory.

## 14. Audit conclusion

Entropy's native egui/eframe architecture is suitable, so there is no need for a Slint fallback or framework bake-off. Its UX patterns are useful, but its keyboard geometry, keycodes, macro serialization, lighting capabilities, firmware options and transport are not compatible evidence for TH80 Pro. The implementation will independently use egui/eframe and the existing TH80 core model/repository. No Entropy code is copied or adapted.

### Source review pointers

- `Cargo.toml`, `LICENSE`, `README.md`
- `src/main.rs`, `src/app.rs`, `src/app_state.rs`, `src/app_storage.rs`
- `src/keyboard.rs`, `src/keycode_picker.rs`, `src/keycode.rs`
- `src/entlayout.rs`, `src/entsettings.rs`
- `src/device.rs`, `src/hid.rs`, `src/qmk_hid_host.rs`, `src/rmk_native.rs`
- `src/ui/layout_view.rs`, `layout_keyboard.rs`, `layout_layer_switcher.rs`, `key_assignment.rs`, `settings_shell.rs`, `rgb_settings.rs`, `app_settings.rs`
- Official docs: [Layouts and layers](https://docs.eh.industries/software/entropy/layouts/) and [Settings and features](https://docs.eh.industries/software/entropy/features/)
