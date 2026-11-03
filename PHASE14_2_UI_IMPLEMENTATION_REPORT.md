# Phase 14.2 — UI Implementation Report

**Verification date:** 2026-11-02  
**Status:** Implemented; visual/platform acceptance limitations remain open.

## Scope and preserved architecture

The redesign remains a native Rust `eframe`/`egui` application. `th80-core` remains authoritative for profile data, validation, serialization and repository persistence. The GUI continues to use the existing `ProfileService`; no second persistence implementation, browser frontend, WebView, GUI framework, device transport or firmware operation was added. The `th80ctl` CLI and research crates were not modified for this UI work.

No commits, remotes, staging, publishing, firmware writes or physical hardware operations were performed. Tests and screenshots use the synthetic profile store under `/tmp/th80-phase14-2/`.

## Implemented application structure

### Layout

- Layout is the startup workspace (`Page::Keymap`). A single unambiguous local profile opens automatically; multiple profiles still require explicit selection.
- The compact shell exposes Layout / Advanced / Config, local profile selection, local save status, file import/export, Save, scale controls and contextual footer status.
- The board is explicitly labelled as logical positions. Each profile-defined position is rendered as a selectable key-like control with its opaque ID and current assignment legend. No physical switch geometry, matrix coordinates, stagger or encoder is fabricated.
- The active layer name is clickable to rename. Previous/next arrows, mouse-wheel navigation over the board and Left/Right navigation are implemented. Keyboard layer navigation is suppressed while text input has focus and while the picker is open.
- The right-side inspector identifies the selected logical position and current action, provides action details, Edit action, Clear assignment and Remove logical position.
- The picker edits a temporary `Action` draft and applies only to the captured layer/position. Cancel/Escape leaves the original assignment unchanged.

### Action picker

The dedicated modal picker derives its taxonomy from the eight `th80-core::config::Action` variants:

| Category | Variants |
|---|---|
| Keys | Key |
| Modifiers | Modifier, Combination |
| Layers | LayerSwitch, LayerMomentary |
| Macros | Macro |
| Other | Transparent, Unassigned |

It provides search, category tabs, action-kind selection, current-action preview, parameter details/editor, profile-derived key/modifier/layer/macro suggestions, Apply and Cancel. Key suggestions are restricted to values already present in the local profile; no generic QMK/Vial/RMK catalog is claimed. Layer suggestion selection preserves the LayerSwitch versus LayerMomentary action variant.

### Advanced

The existing macro library/editor is exposed under Advanced. Existing macro naming, ordered steps, key down/up, delay and text steps, step reordering, validation and delete confirmation remain in place.

### Config

Config exposes local Lighting, Keyboard options, Profile library, Profile details and Application settings. Lighting remains local profile data and retains optional enabled/mode/brightness/speed/color semantics and per-position/effect-parameter data. Keyboard options remain application-owned JSON; unknown top-level extensions remain read-only in the UI and preserved by the core. Application settings expose light/dark theme, accent color, UI scale and file chooser start directory.

## Preferences and save behavior

Application preferences are stored separately from profiles at the platform settings path (or `TH80_SETTINGS_PATH` for isolated runs). Theme, accent and scale are loaded at startup and saved when changed. UI scale is applied through egui zoom. Profile edits remain an explicit local draft until Save; conflict handling remains in the existing repository/state path. No UI text implies a physical device has been updated.

## Source areas inspected/changed in the Phase 14.2 implementation

- `crates/th80-gui/src/app.rs` — three-section navigation, local profile/save shell, theme and scaling application, modal focus tracking and page dispatch.
- `crates/th80-gui/src/state.rs` — picker draft/target/category/search state, appearance preferences and Layout startup state.
- `crates/th80-gui/src/screens/keymap.rs` — logical-position workspace, layer controls, inspector and picker entry point.
- `crates/th80-gui/src/screens/picker.rs` — categorized action picker, model-driven choices, parameter editor and safe apply/cancel behavior.
- `crates/th80-gui/src/screens/settings.rs` — persistent theme/accent/scale and application preferences.
- Existing `screens/macros.rs`, `lighting.rs`, `options.rs`, `profile.rs`, `layers.rs`, and `dashboard.rs` supply the Advanced/Config capabilities; their domain behavior was retained.
- `PHASE14_2_ENTROPY_REFERENCE_AND_DESIGN_SPEC.md`, `README.md`, and the Phase 14.2 reports document design and verification.

The repository has no initial Git commit, so Git reports the project files as untracked. This report does not claim a complete Git diff or baseline-relative file attribution.

## Visual evidence

Native screenshots are retained under `.phase14-2-evidence/th80/`. The primary views include:

- `layout-wayland.png`, `layout-light.png`, `layout-dark.png`
- `layer-selection.png`
- `selected-position-inspector.png`, `wayland-selected-inspector.png`
- `key-picker-light.png`, `key-picker-dark.png`, `wayland-picker-search.png`
- `advanced-macros.png`, `config-lighting.png`, `config-keyboard-options.png`, `application-settings.png`
- `dialog-macro-delete-confirmation.png`
- Additional keyboard/focus/wheel/cancel captures prefixed `wayland-`.

The screenshots demonstrate native rendering and representative interaction states; not every interaction or platform state has a complete capture matrix. See the visual comparison report.

## Remaining limitations

- The logical-position collection intentionally does not reproduce Entropy's verified device-specific physical key geometry.
- The current picker is an editor for opaque profile values, not a physical keyboard catalog. Its action catalog is limited to the core model.
- The header/navigation and wrapped logical-position board remain visually different from Entropy. The result is structurally inspired, not pixel-equivalent.
- The retained Entropy references are 2798 × 1606 px; TH80 captures are 926 × 492 or 926 × 1012 px. A matched-size capture set was not produced.
- Full native keyboard-only traversal and dialog focus behavior remain partially verified. AT-SPI registration and actual screen-reader interaction remain blocked/unverified. Native Windows/macOS execution was not performed.
- Phase 14.1 remains OPEN with 9 PASS, 5 PARTIAL and 4 BLOCKED; Phase 14.2 does not close those findings.

