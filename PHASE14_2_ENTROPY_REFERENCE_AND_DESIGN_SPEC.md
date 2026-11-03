# Phase 14.2 — Entropy Reference and Design Specification

**Status:** Core redesign implemented and automated checks pass; visual acceptance is partial and cross-platform/accessibility acceptance remains open.

## 1. Reference set inspected

Primary reference: [Ergohaven Entropy repository](https://github.com/ergohaven/entropy), [official product site](https://entropy.tools/), [quick start and navigation](https://docs.eh.industries/software/entropy/quick-start/), [layouts and layers](https://docs.eh.industries/software/entropy/layouts/), and [settings and features](https://docs.eh.industries/software/entropy/features/).

The current upstream repository description identifies Entropy as a native desktop workspace for Vial-QMK/Vial-RMK devices. Its official quick-start documentation confirms three top-level sections (Layout, Advanced, Config), a layer-centric center workspace, arrows adjacent to the layer name, wheel and Left/Right layer navigation, click-to-rename, click-to-open key picker, and a bottom contextual-hint line. Its documented key picker is sectioned by action intent, and its scale controls occupy the upper-right while theme selection sits at the lower-right. Entropy writes device changes immediately; TH80 Studio must intentionally retain explicit local save semantics instead.

The live `entropy.tools` product page was opened and its current overview/features were checked. It confirms the current product emphasis on one coherent workspace, key picker, layer editing, advanced actions, import/export, light/dark appearance and UI scale. The live page is a product/marketing surface rather than an interactive application session; detailed behavior is grounded in the official docs and retained application captures, not inferred from the marketing page.

## 2. Retained visual reference inspection

All four existing captures were opened visually and preserved unchanged. Each is **2798 × 1606 px**.

### Layout capture

- Minimal, light neutral full-window surface with a very compact top strip.
- Top-level navigation is centered; Undo is at upper-left; scale decrement/current percentage/increment are aligned at upper-right.
- Layer name is centered prominently below navigation, with previous/next arrows immediately adjacent.
- The central keyboard surface dominates the viewport and uses compact, low-contrast white keycaps with thin neutral outlines and modest corner radii.
- The shown device is split with a large center gap and device-specific stagger/encoders. This is a reference device only; none of its geometry is evidence for TH80 Pro.
- Context/footer information is at the bottom edge; theme control is lower-right.
- No dashboard cards or oversized page title compete with the editing surface.

### Key picker light/dark captures

- Picker is a large centered modal occupying most of the content area, with a subdued/dimmed parent surface.
- A compact title and close affordance sit at the top; a short keyboard-or-mouse instruction follows.
- Category tabs are centered in a single compact row, with a clearly filled active tab.
- Content is organized as a wide key/action surface with consistent keycap sizes, thin borders, low visual noise and generous usable area.
- The dark capture preserves the same geometry and hierarchy, changing surface, outline and text contrast consistently.
- The reference's Basic/Symbols/Mods/Special/Macros/Tap Dance/Custom catalog is firmware- and device-dependent. TH80 Studio must not copy its unsupported catalog entries or imply QMK/Vial compatibility.

### Matrix Tester capture

Inspected as reference material only. It shows a firmware-driven physical matrix test with a tested-count indicator and device-specific geometry. **No Matrix Tester feature will be implemented for TH80 Studio.**

## 3. Verified TH80 model constraints

The current core model is in `crates/th80-core/src/config.rs`:

- `Keymap.layers: Vec<Layer>`; each `Layer` has `id`, `name`, and `BTreeMap<String, KeyAssignment>`.
- Position identifiers are opaque strings. There are no verified physical coordinates, row/column indices, switch dimensions, stagger, encoder locations or TH80 geometry definitions.
- `Action` currently has exactly eight variants: `Key { code }`, `Modifier { modifier }`, `Combination { keys }`, `LayerSwitch { layer_id }`, `LayerMomentary { layer_id }`, `Macro { macro_id }`, `Transparent`, and `Unassigned`.
- `MacroStep` is exactly `KeyDown { key }`, `KeyUp { key }`, `Delay { milliseconds }`, and `Text { value }`.
- `LightingConfig` stores optional enabled/mode/brightness/speed/color values, per-logical-position colors and opaque JSON effect parameters. Optional fields preserve unspecified versus explicit false/zero/black values.
- `KeyboardOptions.values` is application-owned JSON. It does not imply firmware settings.
- `ProfileRepository` and the existing GUI profile service remain the only persistence path.

Therefore the board is a **logical-position workspace**, not a physical TH80 keyboard illustration. Logical positions are displayed as individually selectable key-like controls in a responsive wrapped arrangement. The UI must label this distinction. A future verified geometry definition can be added separately without changing the profile model.

## 4. Target composition

### Shared shell

- One compact top bar with the three Entropy-style primary sections: **Layout / Advanced / Config**.
- Local profile identity and profile selection remain visible in the shell. No connected-device selector, battery, firmware, connection, or hardware capability state is shown.
- Explicit local save action and a visible local draft state: Saved locally / Unsaved changes / Saving / Save failed / Conflict detected. No action is described as applying to a keyboard.
- Import/export and profile operations remain available as local-file actions.
- Scale controls remain in the upper-right region; theme toggle is in the lower-right region. Accent is configurable in application settings.
- A restrained footer provides contextual hints and local storage/save feedback.

### Layout

- Startup page and dominant workspace.
- Layer title and adjacent previous/next arrows; click-to-rename; wheel and Left/Right navigation when focus is not inside an editable control; layer list/operations remain available.
- Central logical-position board with key-like controls, assignment legends, selected/hover/focus states, scroll support and compact spacing.
- A right-side inspector shows selected logical ID, current action, a concise action summary, Edit action and Clear assignment.
- Selecting a position opens the dedicated action picker. The picker edits a temporary action draft; Apply mutates the active profile draft, while Escape/Cancel leaves the original assignment unchanged.
- A bottom hint line explains click, layer navigation and picker cancellation/confirmation.

### Advanced

- Contains the existing Macro library/editor only.
- Preserve naming, step creation/editing, reorder, delay, text, delete confirmation and validation semantics.
- Do not add Combos, Tap Dance, Auto Shift, Text Expander, Typing Trainer or firmware-specific controls.

### Config

- Contains existing Lighting, Keyboard options, Profile library/details and Application settings.
- Lighting is explicitly labeled local profile configuration; preserve all optional-field and color-presence semantics.
- Keep unknown top-level extensions read-only and preserve them through core serialization.
- App settings own persistent theme, accent, scale and file chooser preferences; these are distinct from profile configuration.

## 5. Dedicated action picker taxonomy

Picker categories are presentation groupings of the eight existing model variants, not a claim to provide a hardware keycode catalog:

| Picker category | Model variants | Editing surface |
|---|---|---|
| Keys | `Key` | Opaque key-code string; suggestions only from values already in the active local profile |
| Modifiers | `Modifier`, `Combination` | Opaque modifier string or comma-separated combination members |
| Layers | `LayerSwitch`, `LayerMomentary` | Select an existing layer by its ID/name |
| Macros | `Macro` | Select an existing macro by its ID/name |
| Other | `Transparent`, `Unassigned` | No extra parameters; explicit clear/unassigned semantics |

Search filters category labels and current profile values. Picker content must be scrollable, keyboard-operable and accessible. The selected action is previewed and its parameters are shown in a detail/editor area. No generic QMK/Vial/RMK action catalog is introduced.

## 6. Visual system and proportions

Use the retained Entropy captures as composition references rather than copying assets or source. Target the same restrained hierarchy: compact header, centered primary navigation, prominent layer heading, central editing canvas, compact key-like controls, quiet borders, one accent for selected/active/focus states, low-radius surfaces and small contextual footer. Avoid dashboard cards, large decorative graphics, gradient backgrounds and large section headers.

The local Entropy reference capture is 2798 × 1606; actual TH80 comparison captures should use a matched logical desktop viewport where possible (also record physical pixel dimensions and egui scale). Since the reference has a device-specific physical map and TH80 does not, visual similarity will be assessed by composition, density, keycap treatment, navigation and interaction structure—not by matching its key geometry.

## 7. Accessibility and interaction requirements

- Preserve explicit AccessKit labels/descriptions for logical positions, layer controls, macros, JSON values and dialog roles.
- All picker categories, search, candidate selection, Apply and Cancel must be reachable with keyboard navigation; Enter applies a selected candidate only when appropriate; Escape cancels.
- Preserve modal focus restoration and add picker lifecycle coverage.
- Layer Left/Right and wheel navigation must not steal input while a text editor, combo popup, modal, or other interactive child has focus.
- Maintain visible focus and selected states in both themes.
- Phase 14.1 remains OPEN. AT-SPI/Orca, native Windows/macOS, and outstanding traversal/focus limitations remain explicit acceptance items; do not claim screen-reader certification.

## 8. Explicitly excluded

No device transport, connection state, firmware writes, battery reporting, matrix tester, physical geometry, encoder, device-defined key catalog, Vial/RMK serialization, `.entlayout`, `.entsettings`, firmware update, Combos, Tap Dance, Auto Shift, Key Overrides, Bluetooth, modules, touchpads, displays or live desktop integrations.

## 9. Implementation boundaries

- Keep `eframe 0.35`, `accesskit_unix 0.21.1`, `th80-core`, `th80ctl` and the current profile format.
- Add UI-only state and modules under `crates/th80-gui`; mutate only the active profile draft and persist only through the existing service/repository.
- Do not add HID/USB/serial/udev dependencies to the GUI normal dependency tree.
- Preserve all existing Phase 14.1 reports, reference captures, evidence and running application sessions.
- Do not stage, commit, add remotes, push, publish or access real profile data/hardware.

## 10. Acceptance evidence still required

The implementation now includes the three-section shell, logical-position workspace, selected-position inspector, dedicated picker, macro editor, local lighting/options/profile screens, persistent appearance preferences and local-save status. Actual native Wayland screenshots exist for Layout, picker, layer navigation, inspector focus, picker search, and picker cancellation. X11 evidence includes rendered screens but does not establish the full interaction matrix.

The retained Entropy captures are 2798 × 1606 px. The TH80 evidence was captured at 926 × 492 or 926 × 1012 px depending on the view; these are not pixel-matched captures. The visual comparison report therefore uses a structured composition/interaction comparison and explicitly records the remaining visual differences. The implementation is recognizably Entropy-inspired but is not pixel-equivalent: the TH80 board remains a wrapped logical-position collection, the header retains a local-profile selector, and the picker is a model editor rather than a physical keyboard catalog.

Fresh validation on 2026-11-02: `cargo fmt --all --check`, `cargo test --workspace` (135 passed, 0 failed), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --workspace --release`, and `cargo doc --workspace --no-deps` all passed. The GUI normal dependency tree and GUI source/manifest scan contain no HID/USB/serial/udev/Vial/RMK transport dependencies or APIs.

See `PHASE14_2_UI_IMPLEMENTATION_REPORT.md`, `PHASE14_2_VISUAL_COMPARISON.md`, and `PHASE14_2_TEST_AND_REGRESSION_REPORT.md` for implementation inventory, screenshot evidence, actual comparisons and remaining limitations. Phase 14.1 remains OPEN (9 PASS, 5 PARTIAL, 4 BLOCKED); AT-SPI registration, actual screen-reader operation, native Windows/macOS execution and outstanding keyboard/focus limitations are not claimed as complete.

Phase 14.2 is **implemented with acceptance limitations**, not certified complete: all primary functional sections and automated checks are present, but matched-size visual comparisons and some accessibility/platform verification remain outstanding.
