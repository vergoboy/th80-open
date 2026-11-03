# Phase 12 — Native GUI Architecture

**Decision: use `eframe` + `egui` for the first native Rust GUI.** Entropy's audited native immediate-mode architecture is suitable. Slint is not needed; no framework bake-off is warranted. No Entropy source or assets are reused.

## 1. Existing workspace facts

The root Cargo workspace contains the legacy `th80ctl` research package and the `th80-probe`, `th80-core`, and `th80-cli` crates. `th80-core` is hardware-independent and exposes:

- `config::{Profile, ProfileId, Configuration, keymap::*, macros::*, lighting::*, KeyboardOptions}`
- `ProfileRepository::{new, create, load, list, save, delete, import, export, validate}`
- `validation::validate_profile` and `validation::assess_device_compatibility`
- structured `CoreError` and `Diagnostic` values

The GUI will be a new workspace member `crates/th80-gui`, with a binary named `th80-gui`. Its only project crate dependency is `th80-core`. It must not depend on `th80-probe`, the root HIDAPI package, or hardware/protocol crates.

## 2. Dependency decision

- `eframe`/`egui`: native Rust desktop window, immediate-mode UI and native rendering. Selected because it is the actual audited Entropy framework, is cached in the current Rust environment, and supports a compact single-process app without WebView.
- `rfd`: native open/save dialogs. It does not introduce a browser frontend or keyboard transport.
- `th80-core`: domain model, validation, repository, serialization and persistence.
- No async runtime, database, web server, frontend toolchain, HID library, USB library, firmware crate, or Entropy source dependency.

The GUI crate will enable native X11/Wayland and Glow features through `eframe`. GUI-specific dependencies stay out of `th80-core` and the CLI.

## 3. Module boundaries

```text
th80-gui
├── main.rs              native eframe startup and storage-root selection
├── app.rs               frame loop, navigation, dialogs and user feedback
├── state.rs             active profile draft, selected page/layer/position, dirty state
├── profile_service.rs   thin calls to ProfileRepository; no duplicate persistence
└── screens/
    ├── dashboard.rs     recent/listed profiles, create/import/open
    ├── keymap.rs         logical-position editor and typed Action controls
    ├── layers.rs         layer list, add/remove/rename/select
    ├── macros.rs         MacroDefinition/MacroStep editor
    ├── lighting.rs       generic LightingConfig editor
    ├── options.rs        KeyboardOptions JSON values and opaque extensions
    └── profile.rs        save/save-as/export/delete/validation/error display
```

The application state holds one in-memory `Profile` draft. Editing mutates the draft's public core model fields and marks it dirty. A page change does not create a new persistence layer. Saving calls `ProfileRepository::save`; creating/saving-as calls `create`; loading calls `load`; import/export/delete call the matching repository methods.

## 4. Persistence and profile lifecycle

- Default root: `$TH80_PROFILE_DIR` when explicitly set; otherwise `$XDG_DATA_HOME/th80-profiles`, falling back to `$HOME/.local/share/th80-profiles` (and platform-appropriate data directories on Windows/macOS).
- The GUI displays the resolved profile root and the offline-only status.
- `list` populates the dashboard. `load` opens a selected profile.
- New profile uses `Profile::new`, then `create`.
- Import uses `ProfileRepository::import`, retaining core schema/validation and create-new collision behavior.
- Export uses `ProfileRepository::export`, retaining its no-clobber behavior.
- Save uses `ProfileRepository::save`, retaining stale revision detection and atomic replacement.
- Save As creates a new `ProfileId` through `Profile::new` and `create`; it never overwrites an existing profile.
- Delete requires an explicit confirmation and calls `delete`.
- A stale-save `CoreError::Conflict` is surfaced with reload/review guidance. The GUI must not automatically retry or overwrite.
- Unsaved-change detection compares the current draft to the last loaded/saved in-memory snapshot; it is not a substitute for repository conflict detection.

No test or startup path reads the user's real profile directory implicitly during automated tests. Tests inject a `ProfileRepository` rooted in `tempfile::TempDir`.

## 5. Model-driven editing constraints

### Keymap and layers

Core positions are opaque logical strings. No TH80 matrix geometry, row/column coordinates, key count or firmware keycode catalog is established. The editor therefore renders the logical positions present in the selected layer as compact selectable rows/cards. It can add a new opaque logical position only as a user-entered ID; it must not label it as a physical TH80 key. Supported actions are exactly `Key`, `Modifier`, `Combination`, `LayerSwitch`, `LayerMomentary`, `Macro`, `Transparent`, and `Unassigned`.

Layer add, rename and selection mutate `keymap.layers`. Removal is allowed only after checking references; core validation remains authoritative and any broken references are shown as errors. No Vial/QMK layer opcodes are represented.

### Macros

Expose the four core step variants: key-down, key-up, delay and text. Add/edit/delete operations modify `MacroSet.items`. The UI calls core validation and displays the structured diagnostics; it does not reimplement balanced key or delay rules.

### Lighting

Expose optional enabled, generic mode/custom mode, brightness, speed, RGB color, per-logical-position colors and JSON effect parameters. Values are labeled as local profile abstractions only. Custom/unknown mode names and JSON parameters remain visible and are preserved unless the user explicitly edits them. No device capability or firmware compatibility is implied.

### Keyboard options and unknown fields

`KeyboardOptions.values` and flattened `Profile.extensions` are shown as JSON values and preserved through edits. The core model preserves unknown **top-level** profile extension fields through `Profile.extensions`. Several nested core structs use `deny_unknown_fields`, so arbitrary unknown nested schema fields are rejected by the existing core deserializer rather than round-tripped. The GUI will not claim otherwise or invent a parallel raw JSON persistence layer.

## 6. Validation and error presentation

Call `ProfileRepository::validate` (or `validate_profile`) for local validation. Display each diagnostic's severity, code, domain, path and message. Display `CoreError` by stable `code()` and its contextual `Display` text. Compatibility status comes only from `assess_device_compatibility`; `NotVerified` must remain visible and must not be upgraded by local validation.

## 7. Unsaved changes and concurrency

Keep an original snapshot of the last loaded/saved profile. The dirty flag is derived from `draft != baseline`. Before discarding or opening another profile, request confirmation if dirty. Save errors—including `Busy`, `Conflict`, unsupported schema, validation and I/O—remain explicit. A successful save reloads the persisted profile so the updated modification timestamp becomes the new baseline.

## 8. Hardware safety

The GUI source must not contain device enumeration, HID handles, HIDAPI imports, USB access, Vial/VIA/QMK/RMK protocol code, firmware update logic, or device-state actions. No buttons or disabled placeholders for hardware actions are included. Tests use synthetic profiles and temporary directories only.

## 9. Verification strategy

Unit/integration tests cover model editing, validation, dirty tracking, repository load/save/conflict, import/export, unknown top-level extension preservation and structured errors. All filesystem tests use isolated temporary directories. The requested full workspace commands are run after implementation. Native Linux GUI launch and a basic interaction smoke test are recorded separately from compilation and tests. No Windows/macOS GUI verification is inferred from Linux results.

## 10. Phase 14.2 UI evolution

Phase 14.2 retains the architecture above and reorganizes the native shell into three primary sections: Layout, Advanced and Config. Layout is the startup workspace. When exactly one local profile exists it opens automatically; with multiple profiles, selection remains explicit. This is a startup convenience only and does not add a persistent active-profile pointer or change repository semantics.

The Layout workspace presents the profile's opaque logical-position identifiers as key-like controls. It deliberately does not infer physical geometry. A selected-position inspector and a separate action picker edit the in-memory profile draft. The picker groups the eight existing `Action` variants into five presentation categories (Keys, Modifiers, Layers, Macros, Other); Apply changes the targeted draft assignment and Cancel preserves it. It does not introduce QMK/Vial/RMK catalogs or firmware-specific actions.

Advanced contains the existing macro editor. Config groups existing lighting, keyboard options, profile management and application preferences. Theme, accent and interface scale are desktop preferences; lighting and keyboard options remain local profile data. The shell retains explicit local Save, repository conflict protection, import/export and unsaved-change confirmation. No device connection or live-write semantics are added.

See `PHASE14_2_ENTROPY_REFERENCE_AND_DESIGN_SPEC.md`, `PHASE14_2_UI_IMPLEMENTATION_REPORT.md`, `PHASE14_2_VISUAL_COMPARISON.md` and `PHASE14_2_TEST_AND_REGRESSION_REPORT.md` for Phase 14.2 acceptance evidence and remaining limitations.
