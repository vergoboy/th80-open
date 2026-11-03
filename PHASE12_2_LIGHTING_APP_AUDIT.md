# Phase 12.2 — Lighting Application Audit

**Status: audit complete; one confirmed UI ambiguity corrected; offline regression coverage added.** This report evaluates the Rust application's data model, GUI editing, validation, CLI and persistence behavior. It does not diagnose the physical keyboard and does not authorize hardware access.

## 1. Executive conclusion

The application does **not** apply lighting settings to the keyboard. Its profile model and GUI are local-only, and the inspected GUI has no HID/USB transport path. The audit found no code path by which simply opening a profile or visiting the Lighting page converts an unspecified `enabled` value to disabled or writes a profile to disk.

One real presentation issue was found and fixed: when `configuration.lighting.color` was absent, the GUI displayed a white color-picker preview without saying that no color value was stored. The editor now has a **Set primary color** presence checkbox and explicitly displays **Not specified in local profile** when the value is absent. Enabling it creates an explicit white value; disabling it restores `None`.

The principal remaining model limitation is intentional strictness: unknown members inside `configuration.lighting` are rejected during JSON deserialization, not preserved. This prevents silent data loss but means a profile containing future/vendor-specific lighting fields cannot currently be imported or opened. The generic `effect_parameters` map preserves opaque values only when they are placed inside that known field.

**Answer to the project question:** the app had a misleading color presentation, but the audited implementation does not plausibly explain the physical keyboard's RGB-off state because it has no route to change device state. The remaining diagnosis must focus on firmware/device behavior and independently verified configuration provenance—not on applying this application's local profile values.

## 2. Findings ranked by severity

| Severity | Finding | Status | Impact |
|---|---|---|---|
| Medium | Missing primary color was visually represented by white with no presence indicator | **Confirmed; fixed** | Could make an unspecified value look like an explicitly configured white color. It did not mutate the model until the color picker changed. |
| Medium | Unknown fields inside `LightingConfig` are rejected | **Confirmed limitation; unchanged** | Future/vendor lighting JSON cannot be imported or loaded. It fails closed rather than silently dropping fields. |
| Low | Missing optional fields and explicit JSON `null` collapse to the same `Option::None` value | **Confirmed model behavior** | Re-serialization emits `null`; exact JSON shape is not preserved, but the application-level meaning remains unspecified. |
| Low | Brightness and speed use a normalized local 0–100 scale | **Confirmed policy, not firmware evidence** | A value is an application abstraction and cannot be assumed to map to vendor protocol bytes or hardware ranges. The UI labels this normalized. |
| Informational | Generic mode variants are not proven TH80 firmware effects | **Confirmed limitation** | `static`, `breathing`, `wave`, `reactive` and `custom` are application vocabulary, not a verified device capability list. |

No confirmed defect was found that changes `enabled: None` to `false`, changes valid stored RGB values merely by opening a profile, or writes a profile without an explicit profile-management action.

## 3. Core model and JSON semantics

### 3.1 Types, defaults and representations

Source: `crates/th80-core/src/config.rs`.

- `Configuration` (lines 94–106) has `#[serde(default)]` on `lighting`, so a missing lighting section deserializes to `LightingConfig::default()`.
- `LightingConfig` (lines 184–199) is `#[derive(Default, Serialize, Deserialize)]` and uses:
  - `enabled: Option<bool>`
  - `mode: Option<LightingMode>`
  - `brightness: Option<u8>`
  - `speed: Option<u8>`
  - `color: Option<RgbColor>`
  - `per_key: Vec<PerKeyColor>` with `#[serde(default)]`
  - `effect_parameters: BTreeMap<String, Value>` with `#[serde(default)]`
- `LightingMode` (lines 200–209) serializes with snake-case variant names and supports `Static`, `Breathing`, `Wave`, `Reactive`, and `Custom(String)`. The `Custom` payload is retained as text; it is not interpreted as a device opcode.
- `RgbColor` (lines 210–217) stores three `u8` channels. This is a generic 8-bit RGB representation, not proof of the device's color encoding.
- `PerKeyColor` (lines 218–224) stores a logical position string and an RGB value. The position is not proven to map to a physical matrix coordinate.

Serde's `Option` handling distinguishes `None` from explicit `Some(false)` and `Some(0)`. A missing Option field and an explicit JSON `null` both become `None`; serialization of `None` emits `null` because these fields do not use `skip_serializing_if`. Missing `per_key`/`effect_parameters` and explicitly empty arrays/maps likewise share the same in-memory empty representation.

### 3.2 Unknown fields and extension behavior

`LightingConfig`, `Configuration`, and the nested color structures use `#[serde(deny_unknown_fields)]`. An unrecognized lighting member therefore causes deserialization to fail. It is not silently discarded, but it is not preserved either. This is corroborated by the existing repository test `nested_unknown_fields_are_rejected_instead_of_discarded` and the new focused test `unknown_lighting_fields_are_rejected_not_silently_dropped`.

The top-level `Profile.extensions` map (config.rs lines 35–37) is flattened and preserves unknown top-level JSON members. That mechanism does **not** extend to nested `configuration.lighting` fields. `effect_parameters` is an explicit opaque JSON map, but unknown siblings of that map are still rejected.

### 3.3 Backward compatibility

The persisted schema remains version 1. The Phase 10 schema-v1 fixture still deserializes, validates and round-trips (`crates/th80-core/tests/phase10_compatibility.rs`). Missing `configuration.lighting` defaults cleanly. A profile with unknown nested lighting fields does not: import/load rejects it. Future schema versions are rejected by `ProfileRepository::validate`; there is no automatic migration layer in this phase.

## 4. GUI behavior and state mutation

Source: `crates/th80-gui/src/screens/lighting.rs`.

- The top-of-screen copy (lines 9–12) says settings are local and unapplied, and that firmware support/ranges are unverified.
- The enabled summary (lines 18–23) explicitly distinguishes `Some(true)`, `Some(false)` and `None`.
- The checkbox uses `lighting.enabled.unwrap_or(false)` only as its temporary egui widget value (lines 27–33). It assigns `Some(enabled)` only when the checkbox reports a user change. Therefore an unspecified value remains `None` merely by rendering the page; the temporary false is not persisted as a default.
- Mode handling (lines 35–80) maps absent mode to the visible `Unspecified` option and only writes a different mode after a selection/text edit. A custom mode string is retained. Empty custom strings are representable; the model currently does not reject them.
- Brightness/speed use `unwrap_or(50)` as local widget fallback values (lines 83–102), but their sliders are only shown when the corresponding `Option` is `Some`. The “Set” checkbox is the explicit action that turns an absent value into `Some(50)`. Valid persisted values are constrained to 0–100 by core validation, so the egui slider's default clamping cannot silently alter a profile that successfully loaded through the repository.
- Primary color was previously rendered from `unwrap_or(white)`, visually conflating absent and explicitly white. The audit changed this (lines 104–126): presence is now explicit, absent is labeled, and the white initial value is only written after the user enables the setting.
- Per-position colors and opaque effect parameters are edited directly in the active in-memory profile (lines 130 onward). Add/remove/update operations are tied to button or editor interaction. No hardware operation is performed.

The profile state in `crates/th80-gui/src/state.rs` tracks `active` against a cloned `baseline` (lines 82–84). `open_now` loads a profile and copies it into both fields (lines 105–129); it does not save or normalize lighting values. `save` validates then calls the repository and reloads the persisted profile only after the user invokes Save (lines 156–179). Dirty open/import/create paths use the existing save/discard/cancel guard in `crates/th80-gui/src/app.rs`.

The GUI now communicates the local-only boundary accurately. The status/navigation text also says offline/no device access/no firmware writes. The UI cannot report the keyboard's current lighting state.

## 5. Validation and numeric assumptions

Source: `crates/th80-core/src/validation.rs`.

- `APPLICATION_POLICY.normalized_lighting_max = 100` (lines 8–21) is explicitly described as application-side policy, not a firmware limit.
- Brightness and speed greater than 100 are rejected with `brightness_out_of_range` / `speed_out_of_range` (lines 274–296). Values 0 and 100 are accepted. The GUI sliders use 0–100 and label that scale normalized.
- RGB channels are `u8`, hence 0–255. No TH80-specific channel order, gamma, encoding, white balance or controller range is inferred.
- Per-key entries are checked for non-empty logical positions and duplicate position names (lines 298–316). No physical matrix position validation is possible from current evidence.
- Effect parameter keys must be non-empty (lines 318–325); parameter JSON values remain opaque.
- There is no firmware-backed mode allowlist or device-derived brightness/speed conversion. The validation messages explicitly state that 0–100 is not a firmware range.

These are generic local schema/policy constraints. They should not be described as TH80 firmware constraints.

## 6. CLI, persistence, import and export

Sources: `crates/th80-cli/src/main.rs`, `crates/th80-core/src/repository.rs`, and `crates/th80-gui/src/profile_service.rs`.

- CLI `profile lighting show` serializes the current model; `lighting set` only changes explicitly supplied flags. Brightness/speed are parsed as `u8` and then validated by the core before saving. Out-of-policy values fail validation instead of being stored.
- Repository `load` deserializes and validates without writing (repository.rs lines 81–100). `list` reads/validates profiles (lines 102–130).
- `save` validates, checks the loaded modification timestamp for stale-writer conflicts, updates metadata/provenance and atomically replaces the profile (lines 50–79). It does not transform valid lighting fields.
- `export` loads then serializes the model to a new output file (lines 149–158); `import` deserializes, validates, changes provenance to `ImportedFile`, then creates a profile (lines 160–168). Neither performs firmware I/O.
- Because absent Option fields become `None` and serialize as `null`, import/export may normalize JSON syntax (`missing` → `null`). This does not change the modeled optional setting. Unknown nested lighting fields prevent import instead of being dropped.
- `ProfileService` is a thin forwarding layer to `ProfileRepository`; it has no separate lighting conversion or serializer.

The only automatic persistence metadata changes are provenance on create/import/save and `modified_at_unix_ms` on save. Those do not change the lighting object.

## 7. Regression tests added

All fixtures use in-memory JSON or temporary directories. No real profile directory was read or written.

1. `config::lighting_serde_tests::absent_lighting_section_and_optional_values_remain_unspecified` — absent whole lighting section and absent optional values.
2. `config::lighting_serde_tests::explicit_false_zero_empty_custom_mode_and_black_color_are_not_defaults` — explicit false, zero brightness/speed, empty custom string and black RGB channels survive serialization.
3. `config::lighting_serde_tests::absent_optional_lighting_fields_and_explicit_null_share_none_semantics` — documents absent/null equivalence and serialized null behavior.
4. `config::lighting_serde_tests::unknown_lighting_fields_are_rejected_not_silently_dropped` — strict nested schema behavior.
5. `repository::tests::lighting_values_remain_stable_across_load_edit_save_export_and_import` — unrelated profile edit, save/reload, export and import preserve the lighting model exactly.
6. `state::tests::opening_profile_with_unspecified_lighting_does_not_mutate_it` — opening does not dirty the profile or populate absent lighting values.

The GUI color presence indicator is a presentation correction. The tests above verify the model and state invariants; they are not an automated pointer-level egui interaction test.

## 8. Answers to the ten audit questions

1. **Can unspecified lighting become disabled accidentally?** No in the inspected GUI path. `unwrap_or(false)` is temporary; the model is written only on checkbox change. Missing and explicit null both remain `None`; explicit false is `Some(false)`.
2. **Can opening/saving silently change RGB values?** Opening does not write. Saving serializes the in-memory model. Valid stored RGB channels are preserved; the former absent-color white preview was misleading but did not write white without a color-picker edit. It is now fixed with explicit presence.
3. **Are absent values distinguished from false/zero/empty?** `Option` distinguishes `None` from `Some(false)` and `Some(0)`. Empty custom strings are preserved as `Some(Custom(""))`. Missing and null collapse to `None`; missing/empty collections collapse to empty collections.
4. **Are unknown lighting fields preserved?** No. They are rejected by `deny_unknown_fields`. Top-level extensions are preserved, nested lighting extensions are not.
5. **Can GUI defaults differ from stored values?** Yes, as display/edit defaults: brightness/speed use 50 only when the user enables the field; color previously showed white for `None` and was corrected. No implicit brightness/speed assignment occurs while absent.
6. **Are values validated against evidence-backed constraints?** Not firmware constraints. The 0–100 brightness/speed scale is explicitly application-normalized policy. RGB uses generic 8-bit channels. No device-derived mode/range semantics are claimed.
7. **Can import/export change lighting semantics?** For supported schema, no modeled lighting value changes in the tested path. JSON shape may normalize absent Option fields to null. Unknown nested lighting fields cause import failure.
8. **Does the GUI accurately say local settings are not applied?** Yes. The Lighting page and global offline/no-device indicators state this. It cannot read hardware state.
9. **Are there initialization/callback/save bugs?** No additional confirmed lighting state or persistence bug was found. The absent-color display ambiguity was corrected. Opening is covered by a regression test; model round-trip/import/export are covered. Full automated egui pointer interaction coverage remains absent.
10. **Can a profile be modified without explicit user action?** No profile file write occurs on open/render. Create/import/save/save-as/delete/export are explicit UI/CLI actions. Normal refresh/list/load are read-only. Rendering may update the in-memory model only when an egui widget reports a user edit; persistence still requires an explicit save/create/import action.

## 9. Evidence boundary and what the application cannot know

Phase 5C/5D, Phase 7B, Phase 8 and Phase 12.1 remain the relevant evidence baseline:

- Phase 5C confirms frontend Feature wrapper shapes and the generic SET/GET transport envelope, but not complete device-side command semantics or a conclusive raw-route-to-handler edge.
- Phase 5D confirms buffer construction/checksum branches and the fixed Feature transfer envelope, but not the command grammar or firmware response semantics.
- Phase 7B identifies frontend lighting/per-key request candidates, while marking device-side meaning and hardware-test authorization unknown/not approved.
- Phase 8 closes static command semantics with no candidate approved as statically established read-only; no complete proven path from frontend through native handler to device-side semantics exists.
- Phase 12.1 identifies `0x87` as the strongest frontend lighting-parameter query candidate and `0x89` as overloaded between per-key picture/RGB and keymap paths. Neither is authorized for testing.

The application cannot determine whether the keyboard's LEDs are powered, whether firmware has disabled lighting, whether the Fn+Backspace handler is operating, which firmware/revision is installed, whether an active device-side profile/layer remaps a key, whether a separate controller is faulty, or whether the LED hardware/power path has failed. It cannot verify that a generic local mode/value is accepted by the physical device.

The user's report that RGB is off across USB and 2.4 GHz while USB mode switching still works is a physical observation, not something this application has independently verified. Because the application has no device-write path, a profile edit in this app cannot itself switch the physical LEDs off or on.

## 10. Verification and safety

All five requested commands passed after the code and documentation changes:

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | **PASS** |
| `cargo test --workspace` | **PASS — 99 tests passed, 0 failed** (including doc-tests) |
| `cargo clippy --workspace --all-targets -- -D warnings` | **PASS** |
| `cargo build --workspace --release` | **PASS** |
| `cargo doc --workspace --no-deps` | **PASS** |

The new test coverage consists of four focused lighting serde tests, one repository load/edit/save/export/import round-trip test, and one GUI profile-open non-mutation test. The GUI color presence indicator has not been manually pointer-tested in a native window during this audit.

Scope remained strictly offline:

- no HID enumeration/open, Feature GET/SET, USB transfer or `0x87`/`0x89` test;
- no device setting change, reset, pairing, calibration, firmware or bootloader operation;
- no real profile directory access;
- no Phase 13 work, commits, remotes, pushes, releases or Git history changes.

