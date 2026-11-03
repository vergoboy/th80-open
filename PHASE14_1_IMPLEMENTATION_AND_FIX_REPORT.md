# Phase 14.1 — Implementation and Fix Report

## Summary

Phase 14.1 retained the existing native Rust/egui architecture. The phase verified several prior interaction fixes, added focused regression coverage already present in the GUI suite, and made one additional test-code cleanup to restore strict Clippy compliance. No hardware transport, firmware, vendor binary, or profile-schema change was made.

## Confirmed interaction fixes

### Non-blocking native file chooser

**Original defect:** the native GTK/portal chooser could block the GUI thread long enough for the desktop to report the application as unresponsive.

**Implementation:** `Th80App` now holds a pending flag and an `mpsc::Receiver<FileDialogResult>`. `import_dialog` and `export_dialog` create an `rfd::AsyncFileDialog` future on the UI thread, move the future to a worker thread, and use `pollster::block_on` there. The worker sends the selected path/result to the UI thread. `poll_file_dialog` consumes the channel, clears pending state, applies import/export on the UI thread, and requests repaint after 100 ms while the chooser remains open. Export is disabled while a chooser is pending; the status bar indicates that the chooser is open.

**Regression tests:**

- `async_import_dialog_result_is_applied_on_the_ui_thread`
- `async_export_dialog_result_writes_profile_and_clears_pending_state`
- `async_file_dialog_waiting_state_does_not_clear_before_result_arrives`

**Native retest:** Wayland interaction confirmed the application remained responsive while the portal chooser was open, could cancel, and could complete export/import. No Hyprland unresponsive prompt appeared during the recorded two-second wait.

### Macro deletion target safety

The confirmation stores `pending_delete_macro` as the macro ID captured when the dialog opens. It does not consult the later current selection to determine the deletion target. The dialog computes the referenced-key warning against that captured ID. The test `macro_confirmation_deletes_original_target_after_selection_changes` changes selection from macro A to macro B while confirmation is open, activates Delete for the dialog, and asserts only macro A is removed. The cancellation test confirms both macros and the key reference remain unchanged.

### Focus restoration and dialog semantics

`sync_modal_focus` records the previously focused egui widget when a modal opens and requests focus restoration when all modal state closes. The test `modal_closure_restores_the_previous_keyboard_focus` covers the state transition. Dialogs are explicitly assigned AccessKit Dialog role, modal state and accessible label.

### Narrow macro editor and dialogs

Macro step controls use wrapped rows and bounded edit widths. `macro_step_reordering_controls_remain_inside_narrow_window_bounds` checks the Move up/Move down AccessKit bounds within a 1000 × 660 viewport. Dialog content is constrained to a 360–560 px width; this avoids a narrow error/status modal. These checks do not replace an exhaustive resize matrix.

## Additional test-code cleanup in this continuation

The first strict Clippy run found `clippy::type_complexity` in the GUI test's seven-screen renderer table. The function-pointer type is now named `ScreenRenderer`, leaving the test behavior unchanged. `cargo fmt --all` was run afterward and the full formatting/Clippy/test/build/doc suite passed.

## Save/error recovery observations

- Stale-save conflict is detected by the repository and surfaced in the GUI.
- Reload explicitly replaces the draft with the latest repository copy and reports that unsaved edits were discarded.
- Save-as creates a new profile identity to preserve a stale draft.
- A permission-denied save displays a structured `REPOSITORY_IO_ERROR`; the in-memory draft remains marked unsaved.
- Invalid JSON import and duplicate-ID import failures are surfaced as errors rather than silently ignored.

## Undo/redo assessment and deferred design

No undo/redo feature was implemented. `AppState` holds an `active` profile and a `baseline` profile; `dirty()` compares them. Screen widgets mutate the active `Profile` directly, while repository save/reload establishes a new baseline. There is no mutation-command abstraction or transaction boundary shared by all screens.

### Why not add a superficial global history

- UI-only undo would miss changes made through other screens and cross-feature references.
- Per-widget inverse operations would be fragile for layer IDs, macro references, option extensions, and lighting's absent-versus-explicit values.
- Serializing snapshots through a reduced view could discard unknown top-level extension fields.
- History must not bypass repository stale-save conflict detection or make undo silently write to disk.

### Recommended model

Use a profile-scoped snapshot history first, not command history. The current UI already mutates the complete typed `Profile` in several screens, so a full `Profile::clone()` snapshot is the smallest compatible unit and retains unknown top-level extension values.

1. Scope history to `(ProfileId, loaded repository revision/baseline)`; clear it on opening a different profile, import/open, reload, delete, and save-as identity change.
2. Capture snapshots at semantic edit boundaries (create/remove layer, assignment change, macro step add/remove/reorder, lighting state/value change, option add/update/remove). Coalesce continuous text edits into one transaction on focus loss/commit rather than recording every frame.
3. Keep bounded undo and redo deques (initial proposal: 50 snapshots, with a memory budget and oldest-snapshot eviction). Do not serialize snapshots to a lossy intermediate format.
4. Undo/redo only changes the in-memory draft and recomputes `dirty()` against the repository baseline. It never performs I/O.
5. Successful Save updates the baseline but can retain history; undoing a saved transaction makes the profile dirty again. Redo can restore the saved state.
6. A stale-save conflict must remain authoritative. Do not apply history as an automatic overwrite. Reload discards history; Save as creates a new identity and resets the history scope.
7. Validate after applying a snapshot, but do not discard intentionally incomplete drafts solely because they are currently invalid. Preserve the existing validator's diagnostics.
8. Add tests for extension preservation, absent versus explicit lighting values, macro references, layer identity, save/undo/redo dirty state, stale conflict, reload, and save-as.

Implement only after all profile mutations are routed through a consistent transaction boundary and memory limits are measured. The current phase's correct outcome is to defer this feature.

## Scope and preservation

- No GUI rewrite or framework migration.
- No new hardware dependency or device API.
- No firmware or vendor binary changes.
- No user profile access.
- No README/feature-support update was necessary: the verified local/offline feature boundary did not materially change.

The only additional source edit made during the final strict-Clippy pass was the `ScreenRenderer` type alias in `crates/th80-gui/src/app.rs` test code.

## Acceptance-closure investigation (2026-11-02)

No additional product source fix was made in this continuation. The live Wayland app remained running (PID 359824); no firmware, vendor, hardware or real profile files were accessed.

### AT-SPI runtime finding

The Linux session exposes `org.a11y.Status.IsEnabled` but not `ScreenReaderEnabled`. Locked `accesskit_unix` 0.21.1 subscribes to `ScreenReaderEnabled`; candidate 0.24.1 subscribes to `IsEnabled`. The newer source corrects the property mismatch, but the private AT-SPI socket returned by `org.a11y.Bus.GetAddress` rejects direct connections with `ECONNREFUSED`, preventing the `Embed` registration handshake. `NO_AT_BRIDGE=1` affects the GTK bridge but is not by itself proof that AccessKit is disabled. Orca/Accerciser are absent. See the Phase 14.1.2 accessibility addendum for exact source and runtime evidence.

### Profile deletion safety review

At the Phase 14.1.1 checkpoint the profile-delete dialog resolved the active profile at confirmation time. Phase 14.1.2 hardened this path: the Profile screen now captures the profile ID when opening the confirmation, and confirmation deletes that captured ID. The regression `profile_delete_confirmation_keeps_its_original_target_if_active_selection_changes` proves the target remains stable when the active profile changes in state. This is documented in the Phase 14.1.2 implementation addendum.

### Remaining unmodified items

- No live Orca/AT-SPI tree inspection or screen-reader interaction.
- No exhaustive real keyboard-only traversal.
- No dedicated real profile-delete Cancel/reopen/confirmed-delete run.
- Existing X11 evidence covers startup, navigation, basic keyboard focus, profile creation/edit and keymap, not the full requested X11 workflow.
- No exhaustive resize/long-content matrix.
- No native Windows or macOS GUI session was available. `x86_64-pc-windows-msvc` is an installed Rust target and Wine 11.20 is present, but neither constitutes a native Windows desktop. No Apple target/runner is installed.

No undo/redo implementation was justified by these findings. Keep the profile-scoped snapshot-history design deferred as documented above.

### Final verification and preservation note

The five requested Cargo checks were rerun on 2026-11-02 and all exited 0; workspace tests remain 123 passed, including 31 GUI tests. No source/test changes were needed for the acceptance-closure diagnosis. Final path inventory: 187 baseline, 305 current (excluding `.git`/`target`), zero missing, 118 additions consisting of 114 screenshot files and four Phase 14.1 reports. The Wayland process was preserved; no X11 GUI process remains. No staged changes, commit, remote, hardware access or real profile access occurred.

## Phase 14.1.1 acceptance-closure results (2026-11-02)

No application source change was justified by the new evidence. The separate X11 instance demonstrated profile-delete cancellation and persistence, then confirmed deletion; the delete modal also dismissed on a background click without activating the background navigation. Because the modal prevented a selection change through the tested normal GUI path, there is no reproduced selection-race defect. A pending-delete captured-ID field could be a defensive improvement, but is not being added without evidence of a reachable mutation path.

The clean-environment AccessKit experiment removed `NO_AT_BRIDGE` from a separate Wayland app process and used its own profile root. The app still did not register in AT-SPI. The host has `at-spi2-core 2.62.1`; resolved `accesskit_unix 0.21.1` waits for `ScreenReaderEnabled`, which the host's `org.a11y.Status` interface does not expose. This is classified as a session/backend compatibility blocker rather than a confirmed application widget-tree defect. Orca was not installed because its package brings 37.07 MiB installed size and a substantial dependency set; no actual spoken interaction was performed.

Real X11 evidence was expanded: profile creation/edit/save, delete Cancel/reopen/confirm, macro creation, option creation/save, multiple screen displays, native chooser appearance, and long-content/resize testing. The X11 import/export round-trip, full validation and unsaved-change workflows remain incomplete. The real keyboard samples now include 12 Dashboard Tab steps, reverse Tab, macro editor Tab/Shift+Tab and a Dismiss-button focus sequence; full traversal of all controls remains incomplete.

No product defects were reproduced in the additional tests. The narrow 820 × 560 macro view displayed only a subset of 240 actions at a time, but wheel scrolling exposed later rows; this is expected scroll behavior, not clipping of the data. Long option values are visually clipped in single-line controls, while underlying JSON remains intact; no full-value accessible/keyboard caret audit was possible. No arbitrary minimum-size or framework changes were made.

Undo/redo remains deferred. These tests did not establish a requirement for global or cross-profile history; the previously documented profile-scoped snapshot-history design remains the appropriate future proposal if undo becomes a product requirement.

The separate X11 process was stopped. The original Wayland PID 359824 remains running. All new profiles were synthetic and isolated under `/tmp/th80-phase14-1.1-x11/profiles`; the 247,741-byte stress profile remains there for reproducibility.

### Final Phase 14.1.1 inventory

The path inventory is 383 project files excluding `.git` and `target`: 187 baseline, zero missing, 196 additions. The evidence directory has 221 screenshots; all 143 screenshots present before this continuation remain present by filename, and 78 new X11 captures were added. No source or test file changed. No Cargo checks were rerun because the continuation made no code changes; the last complete verification remains 123 workspace tests (31 GUI tests) and all five Cargo checks passing in Phase 14.1.

## Phase 14.1.2 implementation and fix addendum (2026-11-02)

### Profile deletion target capture

`crates/th80-gui/src/screens/profile.rs` now captures `profile.id.clone()` when the Delete profile action opens the confirmation and stores it in `AppState.pending_delete_profile`. `crates/th80-gui/src/app.rs` renders the confirmation from that pending ID and invokes `delete_profile(&profile_id)`, rather than resolving the mutable active profile at confirmation time. The obsolete `request_profile_deletion()` method was removed from `state.rs`. Regression test `profile_delete_confirmation_keeps_its_original_target_if_active_selection_changes` creates two synthetic profiles, captures the first ID, changes the active profile to the second, deletes by the pending target, and proves the first is removed while the second remains active. The tested modal background click path also cancels rather than activating underlying navigation. This is a justified target-safety hardening, not a reproduced normal-UI race.

### Full-value inspection for JSON option editors

`crates/th80-gui/src/screens/mod.rs` adds `set_accessible_description`, implemented through egui's `accesskit_node_builder(... node.set_description(...))`. `crates/th80-gui/src/screens/options.rs` retains the existing single-line editor and visible layout, gives it the accessible name `JSON value for option {key}`, adds a description containing `Full JSON value: {edit}`, and attaches a hover tooltip with the complete editor string. Regression test `long_option_values_remain_available_to_accessibility_and_inspection` renders an 80-repeat synthetic option and asserts that the AccessKit node value equals the full serialized value and that its description contains the same value. The serialized model is not truncated. This does not prove live AT-SPI/screen-reader exposure or full keyboard copy/paste behavior, and equivalent enhancements were not applied speculatively to other long text fields.

### AccessKit dependency decision

The exact source investigation is documented in `PHASE14_1_ACCESSIBILITY_AND_KEYBOARD_NAVIGATION.md`. Locked `accesskit_unix` 0.21.1 listens for `ScreenReaderEnabled`; candidate 0.24.1 listens for `IsEnabled`. The latter is part of a newer dependency graph requiring eframe 0.37.0 and newer AccessKit/Winit components. That isolated graph compiled, but runtime registration remained blocked by the host private AT-SPI socket returning `ECONNREFUSED`. No `Cargo.toml` or main `Cargo.lock` dependency change was made. This avoids introducing a broad framework upgrade without a functioning AT-SPI runtime or demonstrated application-level compatibility.

### Deferred / unchanged

- No speculative keyboard-navigation, profile-description, macro-name or validation-message changes were made without a reproduced defect.
- No X11 file-dialog workaround or portal dependency was added; GTK portal parent association failed in the current X11/Xwayland environment.
- No firmware, vendor artifact, device transport or hardware behavior was changed.
- No Windows/macOS code changes were made because native platform runners are unavailable.

## Phase 14.1.3 continuation — implementation and design update

### Native Entropy-aligned information architecture

The current native egui source now organizes the application around Layout / Advanced / Config. Layout provides the local profile selector, logical-position workspace, layer controls and selected-position/action inspector. Advanced groups Macros, Lighting and Keyboard options. Config contains application settings, appearance/theme, UI scale and local file-dialog directory preferences. The top-level local profile selector is explicitly a local/offline profile control, not a physical-device selector. The keymap canvas remains a logical-position grid because the current TH80 profile model does not establish verified physical key coordinates or switch geometry.

The current top bar no longer repeats “TH80 Studio” inside the application content; the native window title supplies the product name and the compact navigation begins directly with local profile controls. The visual comparison against the actual Entropy Layout and light/dark Key Picker screenshots confirms the same high-level organization and keyboard-centric workflow, but TH80's selected-position inspector remains text/action-category based rather than a dedicated modal visual key picker. That remaining visual difference is recorded as PARTIAL; no unverified physical geometry or unsupported firmware feature was introduced.

### Source changes retained

- Profile deletion captures the active profile ID before rendering the confirmation action; the pending confirmation retains that ID if active selection changes.
- Macro deletion captures the selected macro ID into pending confirmation state; the confirmation acts on that captured ID, not a later editor selection.
- Regression coverage includes profile_delete_confirmation_keeps_its_original_target_if_active_selection_changes, macro_confirmation_deletes_original_target_after_selection_changes and macro_confirmation_cancel_preserves_both_macros_and_reference.
- The shared screen helper sets complete AccessKit descriptions. The JSON option editor exposes its full value through its accessible description and hover tooltip; the long-option regression verifies the full synthetic value.
- The main app root receives the accessible label “TH80 Studio main window” and the description that the editor is offline/local and does not connect to a physical keyboard or write firmware.
- Theme spacing is set per egui light/dark theme. The renderer-independent AccessKit tests clear unused texture deltas through the test-only finish_output helper.
- The new Config/settings screen and the top-bar refinement are native Rust/egui; no web UI, Electron, React or additional GUI framework was introduced.

### Dependency and test-build decision

The main project remains on eframe 0.35.0 and the existing AccessKit dependency graph. The eframe 0.37.0 candidate demonstrated one historical isolated AT-SPI registration/tree success, but its full test/Clippy/docs and complete native pointer/keyboard acceptance were not established. It was not merged. A separate temporary GTK3 file-dialog candidate was built from a copy of the current source to investigate X11 chooser behavior; its Cargo feature change exists only under /tmp and was not copied into the main Cargo.toml or Cargo.lock.

No physical-device transport, USB/HID access, firmware behavior, vendor artifact or serialized profile schema was added or changed. The test repositories and the “Entropy Visual Review” profile fixture are synthetic and explicitly contain no device data.


## Phase 14.1.4 continuation — no new product-source changes

No main Rust source or dependency changes were made during this acceptance continuation. The temporary GTK3 backend remains confined to `/tmp/th80-phase14-1.3-x11-current/` and its separate target directory. The main project remains on eframe 0.35.0, accesskit_unix 0.21.1 and the existing AccessKit graph. No reason to merge eframe 0.37 or alter the main file-dialog backend was established by this test-only continuation.

The X11 save-failure experiment used only `/tmp/th80-phase14-1.3-x11-gtk3-roundtrip2/import-profiles`. Directory mode 0555 caused a deterministic repository permission error; mode was restored to 0755 immediately afterward. The draft remained available, retry persisted the edited name/description, and a separate fresh app process reopened the saved profile. No serialization/schema changes were needed.

The unsaved-change modal is tied to `pending_open`, which is populated by profile open/import/duplicate flows when the active draft is dirty. Arbitrary top-level section navigation directly changes `state.page`; it is not the transition that exercises this modal. The selector path successfully reached the modal and Cancel preserved the in-memory draft. Confirmed discard still needs direct verification. No new source fix is justified by the evidence collected so far.

### Phase 14.2 design handoff — mandatory

Phase 14.2 is explicitly a substantial native Rust/egui redesign following the actual Ergohaven Entropy interface at https://github.com/ergohaven/entropy. It is not a generic polish pass. The next phase must study actual upstream screenshots and interaction patterns, then adapt the Layout / Advanced / Config hierarchy, central keymap workspace, layer navigation, selected-key/action editing, panels, menus, dialogs, spacing, typography, themes and scaling to verified TH80 Studio capabilities. It must compare actual TH80 screenshots against corresponding Entropy references. Preserve offline profile features and native egui; do not copy Entropy code, assets, branding, logo, proprietary resources or device transport. Do not invent TH80 physical geometry or firmware features. The redesign was not started during Phase 14.1.4.

## Phase 14.1.5 continuation — regression and implementation record (2026-11-02)

### Confirmed-discard regression strengthening

The only main-project source edit in this continuation is test-only: `crates/th80-gui/src/state.rs`, test `confirmed_discard_opens_target_without_persisting_the_original_draft`. The fixture now begins with saved profile name, description, metadata tag and JSON option values; the in-memory draft changes all four before requesting another profile. The test confirms `pending_open`, exercises the explicit `open_now` discard transition, verifies the target becomes active and clean, reloads the original repository record and asserts the saved name, description, tag and JSON option value are unchanged, then reopens the original and repeats those assertions. This strengthens coverage of a real profile-wide draft discard without changing application behavior or serialization.

The actual X11 workflow separately demonstrated name/description discard and post-discard persistence. Its original profile JSON hash remained `ef1dd91a514ad379297261de6e59545271bac7463423e218e511300eca3d04a8` before and after the interaction. Screenshots are listed in the GUI interaction audit.

### Long-value inspection results

No UI or serialization code change was needed. In the existing GTK3 X11 stress session, the 4,558-byte serialized JSON option value was copied in full, selected using both Home/Shift+End and End/Shift+Home, pasted back unchanged and copied again byte-identically. The 350-byte macro name was copied in full and selected in both caret directions. The 240-step macro list remained visible and scrollable. Existing AccessKit test `long_option_values_remain_available_to_accessibility_and_inspection` continues to assert the complete option value and description. The 247,741-byte stress profile hash remained `13e131cd178f613a19854c9f696e2f9d8761d9c8974935f410a008d7109eaa6d`.

The broader inspection matrix remains partial for long profile descriptions, every long label, long validation text and full nested-extension inspection. No schema, serializer, unknown-extension or vendor-specific behavior was changed to address visual clipping.

### Source/dependency boundary

No product behavior fix, GUI framework change, dependency upgrade or temporary GTK3 candidate merge was made. The main project remains on eframe 0.35.0 and accesskit_unix 0.21.1. The only edit is the regression-test strengthening above. `th80-gui`'s normal dependency graph contains no HID, USB, serial or udev transport crate, and `crates/th80-gui/src` contains no hardware transport API. The workspace-level HID research/probe dependencies remain outside the GUI crate's normal dependency graph.

### Deferred work

- Complete native keyboard traversal and exact focus restoration across every dialog remain acceptance work, not a justified speculative shortcut or focus-management rewrite.
- Full live AT-SPI/screen-reader verification remains blocked by the current host socket/property mismatch and missing Orca.
- Long validation messages, every extension value and every long description still need a complete native inspection matrix.
- Windows/macOS native verification requires their own native runners.
- Phase 14.2 remains a separate substantial Entropy-inspired native-egui redesign; no redesign implementation began here.

## Phase 14.1.6 — Minimal keyboard semantics fix and verification (2026-11-02)

### Reproduced source-level finding

The keymap's previous/next layer buttons displayed only `‹` and `›`; the logical-position selection buttons exposed visible text but did not set explicit AccessKit labels; and Remove logical position had no explicit label. These are concrete semantics gaps in otherwise keyboard-focusable egui Buttons. They make the intent less descriptive to assistive technology even though no live AT-SPI tree is available.

### Minimal fix

Updated `crates/th80-gui/src/screens/keymap.rs` to provide descriptive labels for previous/next logical layer, each logical-position button (including its current action label), and removal of the selected logical position. No widget architecture, interaction model, key bindings, profile schema, serializer, dependency, or framework was changed.

Added `app::ui_tests::keymap_positions_are_named_and_selectable_with_tab_and_enter` in `crates/th80-gui/src/app.rs`. It builds a synthetic two-position layer, inspects AccessKit labels, traverses focus with Tab until the logical-position button is focused, activates it with Enter, and asserts the selected logical ID. The targeted test passed; it is simulated egui input and must not be represented as a native desktop or screen-reader test.

### Dialog and file-dialog code review

`Th80App::dialogs` draws egui Modal instances for unsaved profile changes, profile deletion and macro deletion. The status-message modal covers success/error, validation, save failure, stale-save conflict recovery and import/export error feedback. `sync_modal_focus` saves the previous egui focus ID on modal entry and requests it after the final modal closes. Existing tests cover macro-dialog Escape cancellation and egui focus restoration; prior native X11 captures cover unsaved-dialog keyboard activation and save-error Escape dismissal. The complete native 14-case focus matrix remains unverified.

Import/export are async `rfd 0.17.2` XDG/GTK portal operations. The worker awaits the picker and sends an optional path to the UI thread; pending state remains set while the channel is empty, requests repaint after 100 ms, and clears on completion/cancellation or disconnection. rfd offers optional XDG `set_parent`, but TH80 does not currently set an explicit parent handle. Native portal focus return was not measured exhaustively; no speculative ownership rewrite was made.

### Regression checks

After the source change, the targeted keymap regression passed. The workspace test run passed **129 tests, zero failures**: th80ctl 9, th80-core 36, th80-gui 37, th80-probe 44, and th80-hid-research 3. Strict Clippy, release build and documentation commands exited 0. The initial `cargo fmt --all --check` reported only formatting differences in the new test; `cargo fmt --all` was applied and the subsequent format check exited 0. No dependency update occurred.

The GUI normal dependency tree still contains no HID/USB/serial/udev transport dependency, and `crates/th80-gui/src` contains no hardware transport API. All new test profiles were synthetic. No firmware/vendor artifacts or existing user profiles were accessed.
