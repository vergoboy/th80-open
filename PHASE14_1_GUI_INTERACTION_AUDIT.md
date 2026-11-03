# Phase 14.1 — Native GUI Interaction Audit

## Scope and environment

This audit exercised the existing Rust `eframe`/`egui` application. It did not add a browser frontend, hardware transport, or firmware functionality.

| Item | Observed value |
|---|---|
| Host | Linux x86_64, Hyprland |
| Primary interactive backend | Wayland (`xwayland=false`) |
| Wayland window | 1000 × 660, `TH80 Studio` |
| Executable | `target/release/th80-gui` |
| Synthetic profile root | `/tmp/th80-phase14-1-native.LvCNyT/import-target/profiles` |
| Input/capture | `ydotool`, `grim`, ImageMagick, Tesseract |

All GUI runs in this phase used temporary profile directories. The user's normal profile directory was not accessed. The active Wayland test process was PID 359824; a separate temporary-root X11-backend launch was PID 401922 and was terminated after its smoke capture.

## Native screen coverage

The actual application was captured on all seven pages: Dashboard, Keymap, Layers, Macros, Lighting, Options, and Profile. The evidence directory contains page captures such as `screen-dashboard.png`, `screen-keymap.png`, `screen-layers.png`, `screen-macros.png`, `screen-lighting.png`, `screen-options.png`, and `screen-profile.png`, plus more detailed interaction captures.

### Profile workflow

Observed through the native Wayland GUI:

- Create a profile and edit its name, description, and tags.
- Save metadata and observe the saved-local status.
- Duplicate a profile; the duplicate receives a distinct ID.
- Open a different profile and change the active local selection.
- Export through the native file chooser.
- Import through the native file chooser into a separate temporary repository.
- Exercise invalid JSON and an import collision where the same profile ID already exists.
- Display the profile deletion confirmation.
- Display duplicate-name scenarios; profile identity is ID-based, not name-based.

Representative evidence: `phase14-1-native-wayland-profile-create.png`, `phase14-1-native-wayland-keyboard-save-result.png`, `phase14-1-native-wayland-dashboard-duplicate.png`, `phase14-1-native-wayland-duplicate-result.png`, `phase14-1-native-wayland-export-result.png`, `phase14-1-native-wayland-import-success.png`, `phase14-1-native-wayland-invalid-json-error.png`, and `phase14-1-native-wayland-delete-profile-dialog.png`.

The imported record with the exported profile ID exists in the separate import repository and was subsequently edited there. Its later JSON therefore differs from the earlier export; this is not represented as a byte-identical export/import comparison. A separate, retained real-GUI cancellation-and-reopen proof for profile deletion was not captured. The deletion confirmation was shown, and the corresponding temporary profile ID was absent from the final source repository listing, but the cancellation branch remains an evidence gap.

### Unsaved changes, persistence and repository errors

- Switching profiles with an unsaved draft opened the confirmation modal.
- Cancel retained the original draft; discard opened the target profile.
- A controlled external write made the GUI's draft stale. The GUI displayed a stale-save conflict.
- Choosing **Reload saved version** loaded the externally saved description (`External desc 2`) and displayed the message that prior in-memory edits were discarded.
- A temporary profile directory was made non-writable. Saving produced `REPOSITORY_IO_ERROR` / permission denied (OS error 13); the UI still showed the unsaved in-memory draft. Permissions were restored afterward.

Evidence: `phase14-1-native-wayland-unsaved-cancel-retained.png`, `phase14-1-native-wayland-unsaved-discard-open.png`, `phase14-1-native-wayland-stale-conflict.png`, `phase14-1-native-wayland-reload-result-message.png`, and `phase14-1-native-wayland-save-failure.png`.

### Keymap and layers

- Inspected and selected opaque logical positions.
- Opened the action-type selector and changed action types/values.
- Exercised layer selection, layer creation/removal, and a referenced-layer warning.
- Saved and reopened the synthetic profile.

The UI explicitly labels positions as logical IDs and does not imply verified TH80 physical matrix geometry. Evidence includes `phase14-1-native-wayland-keymap-action-key.png`, `phase14-1-native-wayland-keymap-action-menu.png`, `phase14-1-native-wayland-keymap-saved.png`, `phase14-1-native-wayland-layer-switch.png`, `phase14-1-native-wayland-layer-referenced.png`, and `phase14-1-native-wayland-layer-after-remove.png`.

### Macros

- Created and renamed macros.
- Added key-down, key-up, delay and text steps.
- Edited delay values, reordered steps, removed a step and duplicated a macro.
- Opened macro deletion confirmation and observed the referenced-macro warning. The automated egui tests cover both cancellation and confirmed deletion paths.
- The confirmation captures the original macro ID. A regression test changes the current selection while the dialog is open and confirms only the captured target is deleted.

Evidence includes `phase14-1-native-wayland-macro-created.png`, `phase14-1-native-wayland-macro-three-steps.png`, `phase14-1-native-wayland-macro-delay-edited.png`, `phase14-1-native-wayland-macro-reordered-fixed.png`, `phase14-1-native-wayland-macro-duplicate.png`, `phase14-1-native-wayland-macro-delete-warning.png`, and `phase14-1-native-wayland-after-macro-delete.png`.

### Lighting and options

- Lighting's `Unspecified`, `Enabled`, and `Disabled` states were exposed and changed.
- Mode, brightness, speed, primary color, and RGB channels were edited. The color picker was used, and a previously configured color could be deselected.
- Saved JSON demonstrates absent lighting values remain `null`/unspecified; another saved synthetic profile contains explicit `enabled: true`, `mode: static`, and RGB `(255, 0, 0)`. A later edited import-target profile also contains explicit brightness `0`, speed `78`, and an opaque effect parameter with value `0`.
- Options JSON values were added/edited, invalid JSON was rejected, and an unknown top-level `future_vendor_extension` was displayed read-only. The import-target profile retained `custom_flag: true` and `zero_option: 0`.

These are local profile values only. No setting was sent to a keyboard. Evidence includes `phase14-1-native-wayland-lighting-three-state.png`, `phase14-1-native-wayland-lighting-brightness-speed.png`, `phase14-1-native-wayland-lighting-color-rgb-edited.png`, `phase14-1-native-wayland-options-two-values.png`, `phase14-1-native-wayland-options-invalid-json.png`, and `phase14-1-native-wayland-options-extension-expanded.png`.

## File chooser responsiveness

The GTK/portal chooser previously caused a synchronous wait that could make the application appear unresponsive. The current implementation creates an `rfd::AsyncFileDialog` future on the UI thread, awaits it on a worker, sends the selected path through an `mpsc` channel, and polls the receiver from the UI while requesting repaints. Real Wayland retesting showed the application remained responsive while the chooser was open, could cancel it, and could complete export. The regression tests cover import-result application, export-result application, and pending/cancel behavior.

Evidence: `phase14-1-native-wayland-async-open-profile.png`, `phase14-1-native-wayland-macro-after-cancel-portal.png`, `phase14-1-native-wayland-export-result.png`, and `phase14-1-native-wayland-import-success.png`.

## Visual and layout observations

- The 1000 × 660 Wayland window and all seven main pages were captured.
- Main content uses a vertical scroll area.
- The macro editor uses wrapped step rows and bounded editor widths. An egui regression test verifies the Move up/Move down controls remain within a 1000 × 660 viewport.
- Status and error messages are textual; local-only and unverified-hardware boundaries are visible on relevant screens.
- Confirmation and status dialogs have a bounded 360–560 px content width.
- A complete matrix of arbitrary window sizes, very long profile names, and every possible scroll position was not performed. No broad claim of responsive-layout conformance is made.

## Backend results

- **Wayland:** native interactive workflows were performed with the actual Wayland window (`xwayland=false`).
- **X11:** the release GUI was launched with Wayland environment variables cleared and `DISPLAY=:0`; Hyprland reported the TH80 window as `xwayland=true`. A dashboard capture was retained. The X11 run was a launch/screen smoke only; the full interaction suite was not repeated under X11.
- **Windows/macOS:** no native GUI execution was performed in this phase.

## Remaining interaction gaps

1. A dedicated native GUI profile-delete cancel/reopen capture was not retained.
2. The complete profile workflow was not repeated under X11.
3. A systematic arbitrary-size resize and long-content stress matrix was not completed.
4. Keyboard-only and accessibility limitations are detailed separately in [the accessibility report](PHASE14_1_ACCESSIBILITY_AND_KEYBOARD_NAVIGATION.md).

The evidence is genuine desktop capture from the running application, not a mockup. The screenshots remain under `.phase14-1-evidence/`.

## Acceptance-closure addendum (2026-11-02)

### Profile deletion

The existing real Wayland evidence `phase14-1-native-wayland-delete-profile-dialog.png` proves the confirmation was displayed, not that Cancel retained an unchanged profile or that the profile was reopened after cancellation. The current `Th80App` profile-delete modal uses `confirm_delete: bool` and calls `AppState::delete_active()` only from the Delete button; Cancel clears the flag. `delete_active()` captures the active profile ID at the moment of execution, clears active/baseline/editor selections after successful deletion, refreshes the list and returns to Dashboard. No dedicated native Cancel/reopen/confirmed-delete interaction was completed during this continuation. The profile-delete acceptance item remains **PARTIAL**; do not infer it from the macro-delete regression test.

### Keyboard-only traversal

The retained evidence establishes real Tab/Shift+Tab/Enter usage and keyboard profile editing/save. It does not establish complete keyboard-only operation on every screen, dialog, combo box, native chooser or destructive list mutation. The attempt to bring the existing Wayland window to the foreground was blocked by the current Hyprland `hyprctl dispatch` parser rejecting its dispatch syntax. The session was not restarted. See the accessibility report for the screen-by-screen matrix and exact limitations.

### X11 evidence review

The evidence directory contains actual X11/Xwayland captures for dashboard, navigation, Tab/Enter focus, profile creation/name editing and keymap (`phase14-1-native-x11-dashboard.png`, `phase14-1-native-x11-keyboard-tab.png`, `phase14-1-native-x11-keyboard-enter.png`, `03-x11-profile-created.png`, `x11-profile-name-edited.png`, `phase14-1-native-x11-keymap.png`, and related captures). These go beyond a single launch screenshot, but do not show the full requested workflow matrix. X11 is **PARTIAL**: basic navigation/profile/keymap interaction is evidenced; macro, lighting, import/export, validation, save failure and full chooser workflows were not verified in this backend during this continuation.

### Resize and long-content stress

The existing macro-control bounds test checks one 1000 × 660 viewport. The main central content uses a vertical scroll area and macro rows wrap, but no new real resize matrix or long-profile/macro/validation/option stress was completed. Narrow, wide, reduced-height and large-content behavior therefore remains **NOT_TESTED** beyond the existing single viewport regression test. No minimum-size change or speculative layout fix was introduced.

### Final preservation and verification note

The 2026-11-02 continuation reran all five Cargo checks successfully (123 workspace tests; 31 GUI tests). The path inventory is 187 baseline paths and 305 current project paths, with zero missing baseline paths and 118 additions: 114 evidence screenshots plus the four updated Phase 14.1 reports. The evidence directory contains 143 screenshots total, including 29 present in the baseline. The original Wayland process remains alive on its isolated synthetic root; no X11 GUI process remains. This continuation added no product source changes.

## Phase 14.1.1 native X11 acceptance addendum (2026-11-02)

### Dedicated profile deletion cancellation, reopen and confirmed deletion

A separate X11 instance (PID 1346017) used only `/tmp/th80-phase14-1.1-x11/profiles`. It created a synthetic profile with ID `profile-1793598500930-0`, name `Phase 14.1.1 Delete Cancel Test`, and description `Synthetic metadata retained after cancellation.` The file was saved to disk. Its SHA-256 was recorded before cancellation.

The actual X11 confirmation dialog was captured as `phase14-1.1-x11-profile-delete-confirmation.png`. Clicking Cancel returned to the profile page; `sha256sum -c` verified the JSON bytes were identical to the pre-cancel file. The Dashboard then listed the same profile. Open profile reopened the same ID; the Profile page showed the same name and description, and the file hash still matched. This is direct native UI + repository evidence, not a simulated test.

The operation was repeated and Delete confirmed. The GUI returned to Dashboard with `Profile deleted`, and the targeted profile JSON was absent at that checkpoint (before the later, separate stress fixture was added). Captures: `phase14-1.1-x11-profile-delete-cancelled.png`, `phase14-1.1-x11-dashboard-after-delete-cancel.png`, `phase14-1.1-x11-profile-reopened-after-cancel.png`, and `phase14-1.1-x11-profile-delete-confirmed.png`.

**Profile deletion cancellation: PASS. Reopen/persistence after cancellation: PASS. Confirmed deletion: PASS.**

### Selection-change safety while confirmation is open

While the profile-delete confirmation was visible, a pointer click was sent to the Dashboard navigation control behind the modal. The modal dismissed as a cancellation; the underlying Dashboard was not activated and the profile remained unchanged (hash check passed). This establishes that the normal background click does not change the selected profile under the modal. It does not prove safety against arbitrary programmatic state mutation or every keyboard route; the implementation still resolves the active ID when Delete is activated rather than retaining a dedicated pending-delete ID. Selection-change hardening remains **PARTIAL**, not a proven defect or a complete invariant.

### X11 interaction workflow

The isolated X11/Xwayland run has evidence for startup, Dashboard, profile creation, metadata editing, Save, opening/reopening, Keymap screen, Layers screen, Macro creation, Lighting screen, Options screen, JSON option creation/save, profile delete Cancel/confirm, import chooser appearance/cancel, and export chooser appearance. It is meaningful real interaction, but not the complete requested X11 matrix: no successful X11 import/export file round-trip, exhaustive key assignment editing, full macro step/delete-confirm flow, validation-error sequence, save-failure sequence or unsaved-change dialog sequence was completed. See `phase14-1.1-x11-*` captures. X11 is **PARTIAL**.

### Window resizing and long-content stress

The separate X11 window was resized through 820 × 560, 1000 × 660, 1180 × 780 and 1600 × 900. Screenshots are `phase14-1.1-x11-resize-narrow.png`, `-standard.png`, `-wide.png`, `-reduced-height.png`, and `phase14-1.1-x11-resize-contact-sheet.png`. No overlap or immediate layout collapse was visible in the tested views. At 820 × 560, the long macro editor shows only the first several steps at once; wheel scrolling exposed subsequent steps, demonstrating that the list remains scrollable rather than fitting all 240 steps in the viewport.

A synthetic 247,741-byte profile was loaded from the isolated repository. It contains a 24-repeat layer set (24 layers / 960 logical assignments), a 240-step macro with a 20× repeated long title, a long profile name and description, eight long JSON option strings, a large lighting extension value and a large top-level extension payload. Evidence includes `phase14-1.1-x11-dashboard-refresh.png`, `phase14-1.1-x11-stress-profile-open.png`, `phase14-1.1-x11-stress-profile-details.png`, `phase14-1.1-x11-stress-macro-list.png`, `phase14-1.1-x11-stress-options.png`, `phase14-1.1-x11-stress-options-narrow.png`, `phase14-1.1-x11-stress-options-wide.png`, `phase14-1.1-x11-stress-macros-narrow.png`, `phase14-1.1-x11-stress-macros-wide.png`, and `phase14-1.1-x11-stress-macros-narrow-scrolled.png`.

Long profile descriptions wrapped across multiple lines; long macro titles wrapped in the list; 240 actions rendered and scrolled; long option values were represented in narrow single-line editors and visually clipped horizontally. The underlying JSON remained present. The full value was not independently verified through accessibility APIs or a keyboard-only caret traversal. Resizing and long-content behavior are therefore **PARTIAL**: no reproducible overlap/data-loss defect was found in the tested cases, but every screen/dialog/size combination and accessible full-value presentation was not certified.

The temporary X11 process was stopped after testing. The original Wayland PID 359824 remained alive.

### Final Phase 14.1.1 preservation and regression note

The original 143 screenshots present before this continuation are all still present by filename; 78 additional X11 evidence screenshots were added, for 221 screenshots total. The full project path inventory is 383 files excluding `.git` and `target`: 187 baseline paths, zero missing, and 196 additions (114 earlier Phase 14.1 evidence captures + 78 new captures + the four existing Phase 14.1 reports created in the prior phase). No product source file was added or modified in this continuation.

## Phase 14.1.2 final native interaction addendum (2026-11-02)

### Profile deletion target capture

The Profile screen now captures `profile.id.clone()` when **Delete profile…** is clicked and stores it in `pending_delete_profile`. The confirmation renders from that captured ID and calls `delete_profile(&profile_id)`; it no longer resolves the active profile at confirmation time. Regression test `profile_delete_confirmation_keeps_its_original_target_if_active_selection_changes` creates two isolated synthetic profiles, captures the first ID, changes `state.active` to the second profile, confirms deletion through the captured ID, and verifies the first is absent, the second remains active, and the pending target is cleared. This is a direct state-level regression test; the modal also prevents normal background pointer navigation (the previous X11 click-through attempt dismissed it as cancellation). **Profile selection-change safety: PASS** for the captured-target implementation and modeled state mutation.

### Phase 14.1.2 X11 profile deletion

The isolated X11 run used `/tmp/th80-phase14-1.2-x11/profiles` and a synthetic profile named `Phase 14.1.2 X11 Roundtrip` (ID retained in `/tmp/th80-phase14-1.2-x11/roundtrip-id.txt`). Actual X11 pointer interaction opened the profile, opened deletion confirmation, cancelled with Escape, reopened the same profile, reopened confirmation, and confirmed deletion. The JSON file was absent after confirmation. Captures include `phase14-1.2-x11-delete-confirmation.png`, `phase14-1.2-x11-delete-escape-cancelled.png`, `phase14-1.2-x11-reopened-after-cancel.png`, `phase14-1.2-x11-delete-confirmation-final.png`, and `phase14-1.2-x11-delete-confirmed-final.png`. Cancellation, reopen, and confirmed deletion are **PASS** for this synthetic X11 workflow.

### X11 file chooser and workflow parity

The GTK portal chooser was invoked from the X11/Xwayland app, but the chooser did not associate with its parent window. The user-session journal records `Unhandled parent window type` and `Failed to associate portal window with parent window` at 09:30, 09:31, 09:35, 09:53 and 09:57 on 2026-11-02. The chooser appeared separately, but no successful X11 export, import, or export/reimport JSON comparison was completed. The save-failure fixture and repository obstruction were restored, but the available X11 text-entry/focus evidence does not prove an end-to-end GUI save-failure/recovery sequence. X11 therefore remains **PARTIAL**; native file-dialog workflows are **BLOCKED** by the reproducible X11/GTK portal parent-association failure, and save-failure parity remains unverified.

### Keyboard and long-content findings

New X11 screenshots (`phase14-1.2-x11-keyboard-*` and `phase14-1.2-x11-tab-*`) show samples of Tab/Shift+Tab movement and several page states. The sample set does not establish the requested complete Tab, Shift+Tab, Enter, Space, arrow, Escape, and focus-restoration matrix across every screen and dialog. Some X11 text-field focus/coordinate attempts were unreliable. Full keyboard navigation remains **PARTIAL**.

Long option editors now expose an accessible name, an AccessKit description containing the complete JSON editor value, and a hover tooltip containing the full string. The regression test `long_option_values_remain_available_to_accessibility_and_inspection` renders an 80-repeat synthetic value and verifies both the full AccessKit node value and description. This proves the value is not lost from the AccessKit update, but does not prove live AT-SPI/screen-reader consumption, keyboard-only caret/copy/paste, or equivalent full-value inspection for profile descriptions, macro names and validation messages. Long-content accessibility remains **PARTIAL**.

### Phase 14.1.2 preservation

The normalized path comparison against the Phase 14.1.1 final inventory reports **383 baseline paths, 440 current paths, zero missing baseline paths, and 57 additions**. The evidence directory contains 278 valid PNG/JPEG screenshots; the prior 221 screenshot paths remain present. The 57 additions are the Phase 14.1.2 X11 PNG captures. No original report or screenshot was removed. The original Wayland PID 359824 remains alive; only the isolated X11 test process was stopped. No real profile data or hardware was accessed.

## Phase 14.1.3 continuation — 2026-11-02

### Entropy reference and TH80 adaptation

The read-only upstream reference review used the Ergohaven Entropy repository screenshot assets (Layout and light/dark Key Picker) and the official Layout, Layers, Quick Start and Settings documentation. The reference's strongest visual/interaction patterns are a keyboard-centred Layout workspace, compact top-level navigation, layer switching, a selected-key/action picker, grouped advanced features, restrained theme controls and a clean settings surface.

TH80 Studio now uses Layout / Advanced / Config navigation; a local-profile selector rather than a connected-device selector; a logical-position keymap workspace; a selected-position/action inspector; layer controls; Advanced subpages for Macros, Lighting and Keyboard options; and Config settings for application behavior, theme, scale and local file-dialog location. The duplicate in-content TH80 Studio heading was removed so the native window title and compact navigation do not repeat the product name.

The keyboard canvas remains explicitly logical. The profile model does not provide verified TH80 Pro physical key geometry, so no physical switch arrangement or device state is fabricated. The selected-position inspector exposes the existing action categories and key-code editing, but it is not yet a dedicated Entropy-style modal key picker. This is a deliberate model-boundary adaptation and a remaining visual/interaction difference. No Entropy source, logo, branding, application asset or device transport was copied.

Visual evidence is retained in the existing evidence directory, including the Layout, selected-position, Advanced/Macros, Lighting, Keyboard options, Config, theme and dialog captures. Current eframe 0.35 X11 captures include:

- phase14-1.3-x11-current-layout.png
- phase14-1.3-x11-function-layer.png
- phase14-1.3-x11-layer-dropdown-open.png and phase14-1.3-x11-layer-escape.png
- phase14-1.3-x11-advanced-via-keyboard.png and phase14-1.3-x11-lighting-via-keyboard.png
- phase14-1.3-x11-gtk3-profile-library-keyboard.png
- phase14-1.3-x11-gtk3-profile-open-dialog.png and phase14-1.3-x11-gtk3-profile-open-dismissed.png

The reference comparison is **PARTIAL**: the information architecture and keyboard-centric editing direction are now recognizably Entropy-aligned, while the physical keyboard canvas and dedicated visual key picker are not reproduced. The former cannot be added without verified TH80 geometry; the latter remains a native-egui refinement.

### Current X11 interaction

A current-source eframe 0.35 X11 instance and a temporary GTK3 file-dialog backend build were launched only against synthetic profile repositories. Keyboard interaction verified profile selection, Escape dismissal of the “Profile opened offline” message, top-level navigation into Advanced, navigation to Lighting, profile-library navigation, and Base-to-Function layer selection using the layer selector's keyboard controls. Screenshots are X11 captures and are not reused Wayland evidence.

The temporary GTK3 build was made from a copy of the current source tree under /tmp; the main project dependencies were not changed. It did not complete a file chooser save/import operation. The existing portal-backed X11 attempt still reports “Unhandled parent window type” and “Failed to associate portal window with parent window”. No successful X11 export, reimport, serialized-data comparison, invalid-import recovery, unsaved-change cancel/discard sequence or controlled save-failure/retry sequence was established in this continuation. The earlier Phase 14.1.2 profile deletion cancel/reopen/confirm evidence remains valid, and the current profile/macro target-safety regression tests pass.

X11 interaction remains **PARTIAL**. The native keyboard path is demonstrably usable for the tested screens and layer controls, but the requested complete X11 workflow parity is not established.

### Current preservation

The normalized inventory comparison against the Phase 14.1.2 final inventory reports **440 baseline paths, 551 current paths, zero missing baseline paths and 111 additions**. The evidence directory contains **387 valid screenshots**; all 278 screenshots in the Phase 14.1.2 baseline remain present. The 111 additions comprise 109 screenshot files, the native Config/settings screen source file and one synthetic “Entropy Visual Review” profile fixture. ImageMagick identified all 387 current screenshots with zero errors. The original Wayland PID 359824 remains alive. No real profile data, physical keyboard, HID/USB device, firmware or vendor artifact was accessed or modified.


## Phase 14.1.4 continuation — 2026-11-02

### Corrected X11 import/export record

The later GTK3 X11 run supersedes the earlier Phase 14.1.3 statement that no GUI export/import operation completed. The actual X11 Save File chooser exported the 1,805-byte synthetic `Entropy Visual Review` profile. Export SHA-256 was `c5831b4f44a923d4d12e01142c8562a0d5318617bcc2b2ca9608607b16053864`, equal to the source profile. GUI reimport produced the 1,800-byte profile representation; semantic JSON comparison excluding provenance passed. A separate nested dynamically keyed extension fixture also survived semantic comparison. Malformed JSON returned `PROFILE_JSON_INVALID` with a parsing location, did not change existing repository hashes and wrote no partial profile. A corrected synthetic import then succeeded. These are real X11 GUI workflows, not CLI-only operations. They do not alone establish complete X11 parity.

### Unsaved draft and save recovery

In the GTK3 X11 instance, the local profile `Extension Preservation Review` was edited in memory (name and description). The profile details screen showed `Unsaved in-memory edits`. The selector's unsaved-changes modal was reached by selecting another profile. Clicking Cancel closed the modal while the edited values remained visible; the repository SHA-256 was captured and remained unchanged. This verifies cancellation and draft retention. Confirmed discard and post-discard reopen were not completed in this run.

A controlled write failure was introduced only in the synthetic import repository by changing its directory mode to 0555. Save displayed `Action failed` with `REPOSITORY_IO_ERROR — profile I/O error: Permission denied (os error 13)`. The profile file hash remained unchanged during the failed attempt. The directory mode was restored to 0755. Retrying Save wrote the edited name and description. The resulting file parsed as JSON; a separate fresh GTK3 app process (PID 1971100) loaded the same synthetic repository, showed the saved profile name/description in the profile library, opened it, and displayed the persisted values in Profile details. This establishes save-failure draft retention, retry persistence and reopen. No real profile directory or system-wide permissions were changed.

### Keyboard and dialog scope

Native X11 evidence now includes the import/export chooser, invalid-import feedback, unsaved-change modal cancellation, save-error dismissal with Escape, save retry and fresh-process reopen. Full keyboard-only traversal remains incomplete: not every control on Profile library/details, Keymap/Layers, Macros, Lighting, Keyboard options, validation, import/export and confirmation/error dialogs has a recorded Tab/Shift+Tab/Enter/Space/arrow/Escape result. Native focus restoration after each dialog class is also not fully verified. Do not treat a successful pointer interaction as keyboard acceptance.

### Preservation snapshot

Phase 14.1.4 inventory was captured before further changes at `/tmp/th80-phase14-1.4-baseline-normalized.txt` (23,590 paths). The final comparison is `/tmp/th80-phase14-1.4-current-normalized.txt` (23,614 paths): 0 baseline paths missing, 24 additions. The evidence directory has 444 image files, all identified successfully by ImageMagick (0 errors). The additional Phase 14.1.4 captures are retained alongside all prior evidence. PIDs 359824, 1888578 and the separate fresh-reopen test PID 1971100 were alive at the final preservation check. No real profile, physical device, firmware or vendor artifact was accessed or changed.

### Phase 14.1.4 interaction decision

The X11 import/export and invalid-import protections, unsaved-change cancellation, and controlled save failure/retry/reopen are now supported by direct GUI evidence. Confirmed discard, comprehensive keyboard traversal, complete native dialog focus restoration and the wider long-content matrix remain incomplete. X11 interaction therefore remains PARTIAL and Phase 14.1 is not closed.

## Phase 14.1.5 continuation — final interaction evidence (2026-11-02)

### Confirmed unsaved-change discard

The GTK3 X11 session used the isolated synthetic repository `/tmp/th80-phase14-1.3-x11-gtk3-roundtrip2/import-profiles`. Profile `profile-invalid-corrected` was opened with saved name `Corrected Synthetic Import` and saved description `Corrected after invalid JSON validation; synthetic offline fixture.` The live Profile details page was edited to the visibly distinct draft name `PHASE14.1.5 DISCARD DRAFT` and description `UNSAVED TEST VALUE - MUST BE DISCARDED`. Selecting another profile opened the native unsaved-changes modal. The actual **Discard and open** button was focused and activated; `Entropy Visual Review` opened and displayed the offline-open notification. The original profile was then reopened and its saved name and description were visible again.

Evidence: `phase14-1.5-x11-unsaved-before-discard.png`, `phase14-1.5-x11-discard-dialog.png`, `phase14-1.5-x11-discard-focus-1.png`, `phase14-1.5-x11-discard-focus-2.png`, `phase14-1.5-x11-discard-confirmation.png`, `phase14-1.5-x11-post-discard.png`, `phase14-1.5-x11-original-profile-details.png`, and `phase14-1.5-x11-original-reopened.png`.

The original repository JSON SHA-256 was `ef1dd91a514ad379297261de6e59545271bac7463423e218e511300eca3d04a8` before and after the UI action. The reopened JSON still contains the saved original name and description. No discarded draft value was written. The native UI evidence directly covers name and description; the state regression was also strengthened to change and verify tags and a JSON option value before discard, then assert that all four original values remain after persistence and reopen. **Confirmed discard: PASS.** The already-proven Cancel branch was not repeated.

The unsaved-change guard is associated with changing the active profile (`pending_open`), not ordinary Layout / Advanced / Config page navigation. Moving between application sections keeps the same in-memory active draft; it does not silently replace the profile and does not require a discard confirmation. The tested destructive transition was opening a different profile.

### Native X11 keyboard traversal matrix

| Screen / workflow | Real X11 keyboard evidence | Remaining gap | Result |
|---|---|---|---|
| Dashboard / profile library | Existing 12-step Tab sequence, reverse Shift+Tab sample and keyboard navigation evidence from earlier X11 runs | Create, duplicate, open, refresh and every row action not exhaustively traversed/activated | PARTIAL |
| Profile metadata | 24-step Tab capture sequence plus Shift+Tab; focus reached profile selector, top-level navigation, sub-navigation, name, description, tags and associated-model control | Not every action button and model-combo arrow/Space path was recorded | PARTIAL |
| Keymap editor | Keyboard activation reached Layout; existing X11 evidence includes keyboard layer switching | Every logical-position button, action type/value editor and assignment action was not traversed | PARTIAL |
| Layers | Existing keyboard layer selection/switching evidence | Add, rename, remove and referenced-layer warning controls were not all traversed and activated | PARTIAL |
| Macro editor | Keyboard navigation reached Advanced/Macros; the 240-step list and long macro-name editor were inspected; Home/End and full selection/copy were tested | Full keyboard add/delete/reorder and all 240 action controls were not traversed | PARTIAL |
| Lighting | Existing keyboard navigation reached Lighting | Full combo, slider, color and RGB keyboard operation was not recorded | PARTIAL |
| Keyboard options | Keyboard navigation reached the options view; a full 4,558-byte JSON option value was selected, copied, pasted back unchanged and copied back byte-identically | New-option, Update, Remove and nested-extension controls were not all traversed | PARTIAL |
| Validation | Existing validation and error presentation evidence remains valid | Keyboard-only traversal/activation of all validation outcomes and dismissal paths is incomplete | PARTIAL |
| Dialogs | Unsaved-change modal focus entry and Discard activation; save-error Escape dismissal; existing deletion, chooser and modal tests | Not every Create/Rename/Duplicate/Import/Export/Delete/Validation/Conflict dialog was traversed with Tab, Shift+Tab, Enter, Space, arrows and Escape | PARTIAL |

This is broader real X11 evidence, including forward/reverse focus movement and Enter activation, but it is not a complete per-control keyboard-only acceptance matrix. Space and arrow-key behavior across every custom control is not established. No claim of complete keyboard accessibility or absence of focus traps is made.

### Dialog focus restoration

The unsaved-change modal visibly accepted keyboard focus on **Save and open** and then **Discard and open**. After discard, the target opened and its information notification was dismissed; the original profile was reopened. The save-error dialog was dismissed with Escape in the earlier X11 evidence and returned to Profile details. Automated `modal_closure_restores_the_previous_keyboard_focus` still passes. However, exact post-dismissal focus identity was not captured for every native modal, and focus return from GTK chooser cancellation, profile/macro deletion, validation errors and save-conflict recovery was not comprehensively recorded. **Native dialog focus restoration remains PARTIAL.**

### Long-content inspection and editing

The retained 247,741-byte synthetic stress profile contains 24 layers, 960 logical assignments, a 240-step macro, long profile/layer/macro labels, eight long JSON option strings and nested extension data. The live X11 options editor exposed a 4,558-byte serialized JSON string. Ctrl+A/C copied the complete value; Home→Shift+End and End→Shift+Home each selected the full string; pasting the same value and copying it back produced a byte-identical 4,558-byte result. The 350-byte macro name was also copied byte-identically and both Home/End selection directions covered the full value. The UI showed all 240 macro steps and the list remains vertically scrollable.

The full option value is also asserted in the AccessKit node value/description by `long_option_values_remain_available_to_accessibility_and_inspection`. The nested extension fixture remained valid JSON and was not rewritten. The stress-profile file hash remained `13e131cd178f613a19854c9f696e2f9d8761d9c8974935f410a008d7109eaa6d`. These results establish practical inspection/copy/paste for long option and macro-name fields, but not every long profile description, validation message or nested extension value. The legacy GTK3 stress window visibly clips the long selected-profile/layer labels at narrow widths; the current source's compact profile-selector label and narrow-window AccessKit-bounds regression pass, but a full current-source native resize/accessibility matrix is not established. **Long-content inspection remains PARTIAL.** No serialization semantics were changed.

### X11 and acceptance disposition

The confirmed-discard workflow closes the last previously unverified profile-switch data transition. Together with the earlier actual GTK3 X11 semantic import/export round trip, invalid-import protection, unsaved Cancel, controlled save failure/retry/reopen, profile/macro safety and screen-specific editing, the remaining X11 data workflows are covered. X11 interaction is still **PARTIAL** because the full keyboard-only screen/dialog matrix and exact native focus restoration are not complete. No temporary GTK3 candidate was merged and no main GUI dependency was upgraded.

Phase 14.1 remains **OPEN**: confirmed discard is now PASS, but keyboard-only navigation, native dialog focus restoration and the broader long-content matrix remain feasible Linux acceptance gaps. The external blockers remain AT-SPI registration, actual screen-reader interaction, native Windows execution and native macOS execution. Phase 14.2 is design-ready but must not begin until the remaining Phase 14.1 Linux acceptance gaps are either closed or explicitly accepted by the user.

## Phase 14.1.6 — Keyboard inventory and launch-preparation addendum (2026-11-02)

### Source-derived interactive-control inventory

The inventory was derived from `crates/th80-gui/src/app.rs` and every current `screens/*.rs` renderer, not inferred from screenshots. Actual pages are Dashboard/Profile library, Profile details, Keymap (Layout), Layers, Macros (Advanced), Lighting, Keyboard options and Application settings (Config). There is no separate physical keyboard matrix, device selector, firmware page or connected-device action.

| Screen | Actual interactive controls | Keyboard disposition |
|---|---|---|
| Shared navigation | Local-profile ComboBox, Profiles, Layout/Advanced/Config, page tabs, Validate, Save as, Save, Export; status-bar theme and scale buttons | Tab/Shift+Tab and Enter navigation are proven in earlier X11/Wayland samples; Space activation is covered by egui tests. Full control-by-control native traversal and all combo arrow semantics remain unverified. |
| Dashboard / Profile library | New profile name/Create; Import JSON; Refresh; per-profile Open and Duplicate | Controls are standard egui text fields/buttons and enter the normal Tab order. Actual keyboard-only completion of every row action was not established in this phase. |
| Profile details | Name, multiline description, tags, Associated model ComboBox, Validate profile, Delete profile | Text input/save keyboard workflow exists. ComboBox arrow selection and both destructive-dialog branches were not exhaustively traversed natively. |
| Layout / Keymap | Previous/next layer, Active logical layer ComboBox, Manage layers, logical-position buttons, New logical position ID/Add slot, eight action-category selectors, action-specific text fields/ComboBoxes, Remove logical position | Position buttons are focusable egui Buttons; simulated Tab-to-position + Enter selection is now a regression test. No complete native end-to-end keymap edit/clear workflow is claimed. |
| Layers | Selectable layer list, New layer ID/name, Add layer, Selected layer name, Remove layer, Edit assignments | Selectable labels and buttons use normal egui keyboard focus behavior. Full native selection/create/delete traversal not completed. |
| Macros / Advanced | Macro list, New macro ID/name, Create, Duplicate, selected name, per-step key fields/delay DragValue/text field, remove/move buttons, new key-down/key-up/delay/text controls, Delete macro | Existing X11 Tab samples and egui macro-delete/Escape/focus tests remain. Full 240-step keyboard operation and every reorder/delete activation were not newly established. |
| Lighting | Enabled-state and Mode ComboBoxes, custom mode field, brightness/speed/primary-color toggles, sliders, primary/per-position color pickers, position fields, add/remove position colors, JSON parameter key/value and add/update/remove | Standard egui widgets expose keyboard focus; RGB values also have numeric/textual representation. Native keyboard operation of popup color selection and every slider/combo was not completed. |
| Keyboard options | JSON value editors, Update/remove, new option key/value/Add, collapsible unknown extensions | JSON editors retain full values and have AccessKit descriptions/tooltips; no complete keyboard traversal of arbitrarily many options or extension disclosure nodes was captured. |
| Application settings | Dark/Light selectable values, interface scale Slider/Reset, chooser start-directory field | Keyboard-capable egui controls; no full native test of scale slider keyboard increments or theme persistence sequence. |
| egui dialogs | Unsaved changes, profile delete, macro delete, status/validation/save failure/save conflict; buttons Save and open, Discard and open, Cancel, Delete, Dismiss, Reload saved version, Save draft as new profile | Real X11 unsaved-discard and save-error dismissal evidence exists; egui tests cover Escape and focus restoration for macro deletion. Every listed dialog branch was not individually tested on the native desktop. |
| Native file chooser | Async rfd import pick-file and export save-file, including cancellation/result handling | GTK/XDG portal chooser is a separate native surface. Existing native open/cancel/export/import results and async pending/result tests remain; full chooser Tab/Shift+Tab/Enter/Escape and parent-focus return are not established. |

### Phase 14.1.6 native evidence and source change

An isolated new X11 process (PID `2164192`) was launched from the current release binary with `TH80_PROFILE_DIR=/tmp/th80-phase14-1.6-x11/profiles` and a copied synthetic stress fixture. It was not pointed at the user's profile directory. Actual Tab/Enter input navigated into the Profile library; further traversal did not establish a complete keyboard-only Open-profile sequence. The process was left alive. Screenshots are under `.phase14-1-evidence/phase14-1.6/`.

The existing X11 process on PID `1971100` was also used for real Tab, Shift+Tab and Enter samples on its synthetic Profile/Keymap state. The samples show keyboard-driven top-level navigation and real keymap content, but the position/action operation could not be certified as a complete native keyboard-only workflow from those captures. This is recorded as partial rather than extrapolated from egui simulation.

Source inspection found that the keymap's symbolic previous/next layer buttons (`‹`, `›`), position-selection buttons and remove-position button lacked explicit descriptive AccessKit names. `screens/keymap.rs` now assigns meaningful labels to those controls. A new test, `keymap_positions_are_named_and_selectable_with_tab_and_enter`, verifies the layer navigation names, reaches a logical-position button through simulated Tab traversal, activates it with Enter, and checks the selected position. This is an egui/AccessKit regression test, not native screen-reader proof. No physical matrix or new keyboard shortcut was introduced.

### Dialog and chooser implementation findings

Application dialogs are egui `Modal` overlays, not native OS message boxes. `sync_modal_focus` captures `memory.focused()` on the first modal frame and requests that ID again after the last application modal closes. The native file pickers are `rfd 0.17.2` `AsyncFileDialog` instances awaited on a worker thread; an mpsc result is polled on the UI thread, the pending flag is cleared on completion/cancellation/disconnection, and an empty channel requests a repaint after 100 ms. This is an application-owned async lifecycle, not a synchronous UI-thread wait.

The Linux chooser is the XDG/GTK portal surface (a separate native window). `rfd` supports optional `set_parent` on XDG, but the current builder does not set an explicit parent window. No actual chooser open/close focus identity or accidental underlying-action activation was recorded for every import/export path. No parent-handle or file-dialog architecture rewrite was justified without a reproduced failure. Native chooser focus restoration remains PARTIAL.

### Long content and disposition

The previously retained 4,558-byte JSON and 350-byte macro-name byte-identical copy/paste tests remain valid and were not repeated. The 240-step action list remains scrollable. Long profile descriptions wrap, JSON single-line editors use horizontal caret navigation and tooltip/AccessKit descriptions, and unknown top-level extensions remain read-only/serialized. Long nested extension inspection, every long description, long validation-message scrollability, and all tooltip/AccessKit value combinations were not exhaustively exercised. No truncation or serialization defect was reproduced; broader inspection remains PARTIAL.

Phase 14.1 remains **OPEN**. This addendum closes a small AccessKit naming gap and adds a simulated keymap keyboard-selection regression, but it does not satisfy the required comprehensive native per-control traversal or exact native focus-restoration matrix.
