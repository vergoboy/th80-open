# Phase 14 — Implementation Report

## Summary

Phase 14 productizes the existing native Rust configurator without introducing hardware communication or changing the firmware/transport boundary. The work is intentionally incremental: existing core persistence and schema design remain authoritative.

## Implemented behavior

### Profile workflow

- Added a dashboard duplicate action for any saved profile.
- Added CLI profile duplicate PROFILE_ID [--name NAME].
- Added CLI profile rename PROFILE_ID --name NAME.
- Added CLI typed set-action using the core tagged Action JSON model.
- Added CLI layer rename/removal and macro step reordering; referenced layer removal remains blocked by core validation.
- Duplicate operations allocate a new profile ID and fresh timestamps, and preserve description, model association, configuration, tags and top-level extension fields.
- The existing GUI Profile details editor remains the rename interface.
- Delete confirmation now states explicitly when the active in-memory draft will also be discarded.
- Profile-specific option and lighting editor buffers are cleared when opening/reloading/deleting a profile, preventing stale form text from crossing profile boundaries.
- The active profile remains a GUI-session selection; no global persistent active-profile marker was added.

### Conflict recovery

- AppState now tracks a detected stale-save conflict.
- The error modal offers Reload saved version to explicitly discard the draft and reload the repository copy.
- The same modal offers Save draft as new profile to preserve the local draft under a new identity.
- Conflict recovery has a state-level regression test using two repository snapshots.

### Keymap and macro UX

- Changing the active layer clears the selected logical position.
- The editor continues to display opaque logical positions and typed actions; it does not render a fabricated physical TH80 matrix.
- Macro library supports duplicate-selected using a new unique ID.
- Macro deletion now has an explicit confirmation dialog, captures the macro ID being confirmed, warns when key assignments reference it, and leaves the change in the unsaved local draft until the profile is saved.
- Empty macros display an explicit empty state.
- Macro steps can move up/down. The reorder operation is a bounds-checked slice swap with a focused unit test.
- Existing macro validation remains in core; no new undocumented action/step type was introduced.

### Validation and metadata

- Core validation now rejects empty/whitespace-only layer and macro display names with stable diagnostics invalid_layer_name and invalid_macro_name.
- Profile created/modified timestamps are displayed as UTC date/time strings using a dependency-free Gregorian conversion helper.

### Native application quality

- Main page content is wrapped in a vertical scroll area.
- The existing native eframe/egui layout and native file dialogs remain.
- Enabled eframe's AccessKit feature to expose accessibility semantics to supported platform bridges. This is not a claim that a screen-reader audit has been completed.
- No WebView, Electron or browser frontend was added.

### CLI

- Rename and duplicate commands use th80-core::ProfileRepository and the existing profile model.
- Duplicate integration test verifies preservation of a synthetic unknown top-level extension.
- CLI help and documentation include the new commands.

### Documentation and CI

- README now leads with the offline configurator and states the hardware boundary.
- Added feature support, profile JSON format and installation/development guides.
- Updated CLI/core API and firmware-roadmap documentation.
- Extended the existing Linux/Windows/macOS repository workflow to include th80-gui in test, Clippy, release-build and documentation jobs.

## Design decisions

- No new transport trait or device backend was introduced. There is no current integration point that needs one.
- No persistent active profile file was added. The GUI's selected profile is an application-session concept; CLI commands remain explicit by profile ID.
- No undo/redo system was added. The application has no edit transaction/history abstraction, and a shallow UI-only undo would give misleading coverage.
- Duplicate key assignments remain valid. A repeated action can be intentional and no firmware-specific duplicate restriction is established.
- Unknown top-level extensions remain round-trippable. Unknown nested typed fields remain explicitly rejected.
- AccessKit was enabled through an existing eframe feature. No direct dependency version or hardware dependency was added.
- File operations remain synchronous because current profile JSON operations are small and already bounded by local filesystem operations; error/success messages are shown.

## Modified existing paths

- .github/workflows/repository-platform.yml
- Cargo.lock (AccessKit feature dependency graph)
- README.md
- docs/cli.md
- docs/core-api.md
- docs/firmware-roadmap.md
- crates/th80-cli/src/main.rs
- crates/th80-cli/tests/offline_workflow.rs
- crates/th80-core/src/validation.rs
- crates/th80-gui/Cargo.toml
- crates/th80-gui/src/app.rs
- crates/th80-gui/src/state.rs
- crates/th80-gui/src/screens/dashboard.rs
- crates/th80-gui/src/screens/keymap.rs
- crates/th80-gui/src/screens/macros.rs
- crates/th80-gui/src/screens/profile.rs

## New paths

- docs/feature-support.md
- docs/profile-format.md
- docs/installation.md
- PHASE14_APPLICATION_AUDIT.md
- PHASE14_IMPLEMENTATION_REPORT.md
- PHASE14_TEST_AND_PLATFORM_REPORT.md
- PHASE14_HARDWARE_INTEGRATION_READINESS.md

No core repository implementation, profile schema, vendor firmware, updater, native vendor driver, firmware analysis artifact or hardware protocol implementation was modified.

## Known limitations

- No physical device communication or real hardware state.
- No physical TH80 matrix geometry.
- No verified firmware compatibility for keymap, macros, lighting or options.
- No undo/redo.
- No persistent active profile.
- No screen-reader interaction audit or formal accessibility conformance test.
- No pointer-level GUI interaction test harness.
- Windows and macOS native GUI execution remain to be verified on their platforms.

See [application audit](PHASE14_APPLICATION_AUDIT.md), [test/platform report](PHASE14_TEST_AND_PLATFORM_REPORT.md) and [hardware integration readiness](PHASE14_HARDWARE_INTEGRATION_READINESS.md).
