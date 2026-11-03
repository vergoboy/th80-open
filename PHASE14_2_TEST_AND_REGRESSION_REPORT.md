# Phase 14.2 — Test and Regression Report

**Run date:** 2026-11-02  
**Status:** Native Wayland / Hyprland review remains as recorded in section 9. The isolated Weston VNC desktop now has an authenticated loopback/TLS client connection (section 10), but remote input and a usable screenshot path remain unverified. Safe host-side driver improvements and forward-compatible profile persistence were implemented and fully validated in section 11. The unsaved-profile modal, in-app scale reset, accent-color interaction, Entropy comparison, AT-SPI/screen-reader runtime, and native cross-platform execution remain open. Phase 14.2 remains OPEN / NOT ACCEPTED.

## Fresh command results

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo test --workspace` | PASS — 136 passed, 0 failed |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo build --workspace --release` | PASS |
| `cargo doc --workspace --no-deps` | PASS |
| `cargo check -p th80-gui --target x86_64-pc-windows-msvc --locked` | PASS — Windows target type-check/compile; not a native linked executable or runtime test |
| `cargo build -p th80-gui --target x86_64-pc-windows-msvc --locked` | BLOCKED — GUI and dependencies compiled; final link requires missing Microsoft `link.exe` |

The workspace test count comprises CLI 9 (3 unit + 6 workflow), core 36 (30 unit + 1 compatibility + 5 process), GUI 44, probe 44 (26 library + 6 binary + 12 offline protocol), and HID research 3: 136 total. Doc-tests contain no additional tests.

## Relevant GUI regression coverage

Fresh GUI tests include:

- `action_picker_cancel_preserves_the_original_assignment`
- `action_picker_apply_updates_only_the_target_logical_position`
- `key_action_picker_suggests_only_codes_already_in_the_local_profile`
- `application_theme_accent_scale_and_file_dialog_preferences_persist`
- `left_and_right_keys_navigate_logical_layers_without_text_focus`
- `layer_arrow_navigation_remains_available_with_position_button_focus`
- `mouse_wheel_over_logical_board_navigates_layers`
- `keymap_positions_are_named_and_selectable_with_tab_and_enter`
- `tab_and_shift_tab_move_focus_between_navigation_controls`
- `enter_and_space_activate_focused_navigation_buttons`
- `modal_closure_restores_the_previous_keyboard_focus`
- `interactive_controls_on_all_screens_have_accessible_names`
- `long_option_values_remain_available_to_accessibility_and_inspection`
- `lighting_enabled_state_can_switch_between_unspecified_enabled_and_disabled`
- `lighting_pointer_toggles_preserve_unspecified_and_explicit_values`
- macro reorder, deletion confirmation/cancellation, async import/export, repository round-trip, stale-save conflict and unknown-extension preservation tests.

The picker cancellation and apply tests prove draft isolation and target-position isolation. They are egui test-context tests, not a substitute for native input testing.

## Native Linux evidence

Retained screenshots under `.phase14-2-evidence/th80/` demonstrate the rendered Layout, layer navigation, inspector, light/dark picker, search filtering, Advanced/Macros, Config/Lighting, Config/Keyboard options, Application settings and macro-delete dialog. The `wayland-` captures record representative keyboard traversal, Enter activation, layer-arrow/wheel interaction, picker search and Escape cancellation/focus restoration from the previous native Wayland interaction session.

An isolated X11 instance of the current Phase 14.2 build was launched with a copied synthetic profile at `/tmp/th80-phase14-2-acceptance/profiles` and settings at `/tmp/th80-phase14-2-acceptance/settings.json`. Existing Wayland and X11 application processes were preserved. New captures are under `.phase14-2-evidence/th80/`; each is 926 × 1012 and the visible scale control reads 100%.

Direct X11 mouse/keyboard observations:

| Interaction | Result | Evidence / limitation |
|---|---|---|
| Application startup | PASS | `layout-x11-acceptance.png`; current build rendered with synthetic profile |
| Main navigation | PASS | `x11-advanced-navigation.png` and `x11-config-navigation.png`; Layout, Advanced and Config switched by actual X11 clicks |
| Layer navigation | NOT TESTED | No new X11 layer change performed |
| Action picker open/close | PASS | `x11-action-picker-open.png`; opened from a logical position, closed with Escape |
| Action search | PASS | `x11-picker-search.png`; typing `macro` filtered to the Macros category/action |
| Action selection changes only target position | PARTIAL | Existing egui regression test covers target isolation; no X11 action was applied in this run |
| Picker cancellation preserves assignment | PASS | `x11-picker-cancel.png` shows the selected logical position still assigned `KEY_Q` after Escape |
| Layer renaming | NOT TESTED | No new X11 rename interaction performed |
| Theme switching | NOT TESTED | X11 run remained in Dark theme |
| Scaling interaction | NOT TESTED | Visible value was 100%; no slider change was made |
| Profile dialogs focus/dismissal | NOT TESTED | No new X11 profile dialog interaction performed |
| Unsaved-change protection | PARTIAL | Existing egui regression tests cover draft/confirmation behavior; not exercised in this X11 run |

This matrix is limited to the interactions actually performed. The older `layout-workspace-x11.png` remains rendering evidence only and is not upgraded to interaction evidence.

## Offline profile and persistence boundaries

Workspace tests exercise local profile create/list/load/save/import/export/delete, schema compatibility, atomic replacement, stale writers, process-level repository concurrency, unknown top-level extension preservation, lighting optional-value semantics, and CLI end-to-end offline editing. GUI tests exercise draft save/load, macro editing, layer/key assignment state, import/export results, unsaved-change handling and save conflicts. The Phase 14.2 GUI was launched with a synthetic profile root at `/tmp/th80-phase14-2/profiles`; real user profile data was not accessed.

## Hardware isolation

- `cargo tree -p th80-gui -e normal` was inspected; no HID, USB, serial, udev, Vial, RMK or libusb dependency appears in the normal GUI dependency tree.
- GUI source and manifest scan found no matching hardware transport API/reference.
- `th80-core` remains the GUI's domain/persistence dependency. No device communication was added.

## Accessibility and AT-SPI diagnostics

- `crates/th80-gui/Cargo.toml` enables eframe's `accesskit` feature. GUI source sets descriptive labels for controls and explicit Dialog/Root semantics; the test suite inspects generated AccessKit updates in egui test contexts.
- The session D-Bus exposes `org.a11y.Bus` and `GetAddress` returns `unix:path=/run/user/1000/at-spi/bus_0`. A local socket listener exists at that path, but direct connection attempts to it return `Connection refused`.
- `at-spi-bus-launcher` and `at-spi2-registryd` processes were present, but `org.a11y.atspi.Registry` was not available/activatable on the session bus during inspection. This indicates an AT-SPI session/registry infrastructure problem; it does not prove the TH80 application caused the failure.
- `orca`, `accerciser`, and the Python `pyatspi` module are not installed. Actual application registration, tree traversal through AT-SPI, and screen-reader speech/navigation could not be verified.
- AccessKit compilation and egui-generated tree tests are not evidence of successful native runtime bridge initialization. Accessibility runtime remains BLOCKED by the current session infrastructure/tooling; application semantics remain covered only by unit-level tree checks.

## Cross-platform target diagnostics

- `x86_64-pc-windows-msvc` is installed. `cargo check -p th80-gui --target x86_64-pc-windows-msvc --locked` **passed**. This verifies Rust compilation/type checking for the Windows target, not linking, packaging, or Windows runtime behavior. No MSVC linker/toolchain or Windows runner was present for native execution.
- A subsequent `cargo build -p th80-gui --target x86_64-pc-windows-msvc --locked` compiled the GUI and its Windows dependencies but stopped at final linking with `linker link.exe not found`. Wine is installed, but it does not supply the Microsoft linker or a native Windows desktop environment.
- Neither `aarch64-apple-darwin` nor `x86_64-apple-darwin` is installed. macOS target compilation and native execution were not performed. No Apple SDK or macOS runner is available in this Linux session.
- Linux-only `cfg` paths and platform settings paths exist in `app.rs`; the Windows target check compiled the Windows-selected code path. No unsupported platform API was identified by this limited compile check, but runtime compatibility remains unverified.
- All seven commands in the fresh validation batch exited with code 0. The GUI-only test command separately passed 44/44 tests.

## Phase 14.2 acceptance matrix

| Acceptance area | Status | Evidence / outstanding condition |
|---|---|---|
| Core native UI structure and profile workflows | PASS | Existing implementation and GUI/workspace regression suite; no redesign rewrite was needed |
| Automated Linux Rust validation | PASS | Workspace 136/136; GUI 44/44; formatting, strict Clippy, release build and docs all pass |
| GUI hardware-transport isolation | PASS | Normal GUI dependency tree and source/manifest scans remain free of hardware transport APIs/dependencies |
| Entropy visual comparison at matched viewport and known equivalent scale | PARTIAL | Actual reference and app screenshots inspected; dimensions/scales recorded; native matched capture unavailable |
| X11 interaction matrix | PARTIAL | Startup, primary navigation, picker open/search/cancel verified; layer rename/navigation, theme/scale controls, profile dialogs and full unsaved-change path remain untested in X11 |
| AccessKit semantics in egui-generated tree tests | PASS | GUI tests inspect names, roles, dialogs and generated tree updates; this is not runtime bridge verification |
| AT-SPI registration and screen-reader interaction | BLOCKED | AT-SPI socket refuses connection, registry service unavailable, and Orca/Accerciser/PyAT-SPI absent |
| Windows target compilation check | PASS | `cargo check` for `x86_64-pc-windows-msvc` passed; no link/runtime claim |
| Windows target linking and native execution | BLOCKED | Missing Microsoft `link.exe`/Visual Studio Build Tools and no Windows host/runner; Wine alone is insufficient |
| macOS target compilation and native execution | BLOCKED | Apple targets not installed; no Apple SDK or macOS runner in this Linux session |

## Not verified / still open

- AT-SPI registration and actual Orca/screen-reader interaction are blocked by the current session registry/socket state and missing screen-reader tools. AccessKit labels/tree generation tests do not establish runtime screen-reader accessibility.
- Native Windows and macOS GUI execution was not performed. Windows target `cargo check` passed; macOS target compilation was not attempted because the target is not installed.
- Full native keyboard-only navigation and dialog focus restoration are not exhaustively verified; existing Wayland evidence, new X11 mouse/keyboard interactions and egui tests cover representative paths only.
- The X11 matrix above remains partial; layer navigation/rename, theme/scale changes, profile-dialog focus and end-to-end unsaved-change handling were not exercised in this X11 run.
- A matched-size native Entropy/TH80 screenshot set was not produced. The source screenshots differ in viewport dimensions and the Entropy source display scale/DPI is unknown; see `PHASE14_2_VISUAL_COMPARISON.md`.
- Theme preference loading/saving is implemented and screenshot evidence exists, but a dedicated automated process-restart persistence test was not added in this run.

Phase 14.1 remains OPEN (9 PASS, 5 PARTIAL, 4 BLOCKED). Phase 14.2 does not upgrade those statuses or claim screen-reader/platform certification. The current X11 interaction evidence improves the native Linux matrix but does not close it.

## Controlled assessment update — 2026-11-02

This section supersedes the earlier X11 interaction table and earlier viewport-only statement where they conflict. The original results above remain historical evidence for that earlier run.

### Repository and source-change boundary

- Repository root: `/home/vergoboy/Projects/projects/th80/th80-open/` (shared workspace `/th80/th80-open`).
- `git status` still reports an unborn `main` branch and all existing project files as untracked. No file was staged; no commit, remote, push, reset, clean or destructive cleanup was performed.
- No application source, Cargo manifest/lockfile, README, profile fixture or user profile was modified in this assessment round. The only intended repository changes for this round are the two existing Phase 14.2 assessment reports and added evidence screenshots under `.phase14-2-evidence/controlled-x11/`.
- Existing Wayland TH80 process PID `359824` and active user browser sessions were not manipulated. The controlled TH80 process used a separate copied synthetic profile/settings directory under `/tmp/th80-phase14-2-controlled/`.
- No physical keyboard was connected, queried or communicated with. The isolated Weston instance used `--fake-seat`; Entropy ran without USB/HID device access.

### Fresh X11 evidence and results

An isolated Weston headless compositor was run at `1800 × 1033`, scale `1`, with XWayland display `:1`. The TH80 test window client area was `1762 × 974` pixels. The app's settings file records `ui_scale: 1.0`; the visible control reads `100%`. A temporary `105%` interaction was captured, then the settings were restored to `1.0` / `100%`. Theme was toggled to Light and Dark and the temporary preference was restored to Dark before teardown.

| Interaction | Updated result | Evidence / boundary |
|---|---|---|
| Application startup and Layout rendering | PASS | `controlled-x11/th80-layout-1762x974.png`; isolated current release binary, synthetic profile |
| Primary Layout / Advanced / Config navigation | PASS | Direct X11 clicks were already verified in the earlier X11 evidence (`x11-advanced-navigation.png`, `x11-config-navigation.png`) |
| Layer navigation | PASS | Direct navigation to layer 2/2; `controlled-x11/th80-layer-navigation-1762x974.png` |
| Layer rename | PASS | Double-clicked the layer name, replaced it with `1. Navigation - X11 Verified`, and pressed Enter. The committed in-app layer title is visible in `th80-layer-rename-committed-1762x974.png`. The change remains an unsaved synthetic-profile draft; no real profile was written. |
| Theme switching | PASS | Light and dark states captured (`th80-theme-light-1762x974.png`, `th80-action-picker-open-dark-1762x974.png`); settings file restored to dark |
| Scaling interaction | PASS | 105% visible in `th80-scale-105-1762x974.png`; settings restored to 100% (`ui_scale: 1.0`) |
| Profile selector / profile switching | PASS | Selector menu opened and synthetic fixture profile selected (`th80-profile-selector-menu.png`, `th80-fixture-profile-restored.png`) |
| Profile switching | PASS | Synthetic fixture profile and empty `My TH80 profile` were selected in the isolated profile selector; `th80-profile-selector-menu.png` and `th80-profile-open-dismissed.png` |
| Profile-dialog focus/dismissal | PARTIAL | The unsaved-changes dialog was displayed and dismissed; no exhaustive focus traversal or independent modal keyboard-focus audit was completed |
| Action picker open and category/action selection | PASS | Light/dark picker states and key selection captured (`th80-action-picker-open-light-1762x974.png`, `th80-picker-key-selected-KEY_E-light-1762x974.png`) |
| Action application and target-position isolation | PASS | `KEY_E` applied to `logical-01`; subsequent `logical-02` remained `KEY_HOME` (`th80-action-applied-position-01.png`, `th80-action-target-isolation-position-02.png`). This is a direct UI observation using the synthetic profile. |
| Unsaved-change protection | PASS | Attempt to open another profile while the fixture was dirty displayed the explicit Save and open / Discard and open / Cancel dialog (`th80-create-profile-unsaved-guard.png`). The dialog was dismissed without writing to a real profile. |
| Picker cancellation preserves assignment | PASS | Earlier direct X11 capture `x11-picker-cancel.png` retained `KEY_Q`; automated cancellation regression also passes |

All these X11 captures are from the isolated display, not the user's active desktop. Direct mouse interactions are not represented as keyboard-only/screen-reader verification.

### Matched-viewport Entropy attempt

- A local Entropy v0.5.0 AppImage was found and launched under bubblewrap on the same isolated XWayland display as TH80.
- Both Entropy and TH80 client windows were captured at `1762 × 974` pixels; compositor output was `1800 × 1033`, scale `1`.
- Entropy log: `Guessed window scale factor: 1`; `Found 0 Vial device(s)` repeated. The captured Entropy state (`controlled-x11/entropy-v0.5.0-no-device-1762x974.png`) says “Waiting for a device — Connect a Vial device.”
- The sandbox had no USB/HID device access by design. No physical device was used and no fabricated device was introduced. Entropy's Layout, inspector, layers and picker are consequently unreachable in this safe test.
- The same-pixel viewport attempt is real and directly evidenced, but equivalent interactive states are not. This does not satisfy visual-comparison acceptance. See the updated `PHASE14_2_VISUAL_COMPARISON.md` for the full method and state inventory.

### Accessibility and platform recheck

- `org.a11y.Bus` still advertises `unix:path=/run/user/1000/at-spi/bus_0`, but a direct connection to that socket fails with `Connection refused`; direct bus/registry introspection also fails. Orca, Accerciser and `pyatspi` remain unavailable. Do not claim AT-SPI registration or screen-reader acceptance. Keep runtime accessibility BLOCKED; AccessKit/egui tree tests remain PASS only at the test-context level.
- Windows target `x86_64-pc-windows-msvc` remains installed. The previous `cargo check` passed. No `link.exe` or `cl.exe` was found; `/usr/bin/link` is GNU `link`, not the Microsoft linker. Windows linking/native runtime remains BLOCKED.
- No Apple Rust targets or `xcrun` were found; no Apple SDK/macOS runner is present. macOS compilation/runtime remains BLOCKED.

### Current acceptance matrix

| Acceptance area | Current status | Evidence / remaining gap |
|---|---|---|
| Core native UI structure and offline profile workflows | PASS | Existing Phase 14.2 implementation and previously passing automated tests |
| Automated Linux Rust validation | PASS (latest prior source run) | 136 workspace tests and 44 GUI tests; fmt, strict Clippy, release build and docs passed in the previous source-validation batch. No source changes since then. |
| GUI hardware-transport isolation | PASS | Prior dependency/source audit; current Entropy test sandbox intentionally had no USB/HID access |
| Same client-pixel viewport capture | PASS | Both live app windows captured at `1762 × 974` on the same isolated XWayland display, scale 1 |
| Equivalent Entropy/TH80 interactive visual states | BLOCKED | Entropy v0.5.0 waits for a Vial device and reports zero devices; no safe Layout/picker state can be reached |
| Evidence-backed visual refinements | NOT TESTED | No source change justified by a no-device Entropy screen; do not alter layout speculatively |
| X11 primary navigation, layer navigation and rename | PASS | Direct X11 interactions; layer 2/2 capture and committed visible rename to `1. Navigation - X11 Verified` |
| X11 theme and scaling | PASS | Both themes toggled; 105% tested then restored to 100% |
| X11 profile selector/dialog and unsaved-change guard | PASS / PARTIAL | Profile switching and unsaved prompt/dismissal directly observed; exhaustive focus traversal remains partial |
| X11 action apply and target isolation | PASS | `KEY_E` applied to `logical-01`; `logical-02` remained `KEY_HOME` |
| AccessKit semantics in egui-generated tree tests | PASS | Existing GUI tests inspect names, roles and generated updates |
| AT-SPI registration / screen-reader interaction | BLOCKED | AT-SPI socket refuses connection; registry cannot be introspected; Orca/Accerciser/PyAT-SPI unavailable |
| Windows target Rust check | PASS | Prior `cargo check` passed |
| Windows link/native execution | BLOCKED | Microsoft `link.exe`/MSVC toolchain and native Windows runner unavailable |
| macOS target compilation/native execution | BLOCKED | Apple targets, SDK and macOS runner unavailable |

Phase 14.1 remains OPEN at **9 PASS / 5 PARTIAL / 4 BLOCKED**; no new direct evidence in this round changes an individual Phase 14.1 status. Phase 14.2 remains **unaccepted** because matched interactive Entropy comparison and runtime accessibility/platform gaps remain.

## Final acceptance investigation — upstream Entropy, accessibility, focus and platform prerequisites (2026-11-02)

This section is the final investigation update and supersedes earlier statements where they describe Entropy device-free support, AT-SPI socket state, or the remaining dialog-focus work differently.

### 1. Entropy device-free mode: Outcome B

The official upstream repository `https://github.com/ergohaven/entropy` was inspected from a shallow checkout at commit `336a5dcb0ad587caab938b32e4700568ecd21ba8`. Official docs inspected: `https://docs.eh.industries/software/entropy/`, including Quick start and navigation, Installation, and the repository README.

Source paths and findings:

| Upstream path | Finding |
|---|---|
| `src/main.rs` (normal startup around lines 365–486) | `eframe::run_native` creates the normal window and calls `EntropyApp::new`; no demo/preview launch entry point or device-free CLI flag is present. |
| `src/app_init.rs` (around lines 15–53) | Normal `EntropyApp::new` constructs `DeviceManager::new()`. |
| `src/device.rs` (around lines 507–560) | `DeviceManager::new()` calls `scan()` on non-macOS; the scan initializes `hidapi::HidApi` and enumerates real HID devices. macOS defers the initial scan for run-loop safety, not to expose a demo editor. |
| `src/app_init.rs` (around lines 55–74) | `new_headless()` is explicitly the no-window `--export-layout` path, constructs an empty manager and marks the app headless. It does not render the editor. |
| `src/headless_export.rs` | Headless export is a device-backed layout snapshot/export operation, not an offline preview. The public README likewise describes selecting a connected Vial device and exporting its layout. |
| `src/ui/layout_view.rs` | The production Layout renderer receives a `KeyboardLayout`; no standalone sample-editor entry point is exposed. |
| `tests/fixtures/display-menu-catalog.README.md`, `tests/fixtures/k04-vial-b4709ab9.json` | Source-derived characterization/test data. These fixtures are not a documented sample keyboard profile or production editor launch mode. |
| `src/app_init.rs` (`new_inert_for_test`) and `src/ui/*` test/preview files | Test-only constructors and egui fixture tests exist. `new_inert_for_test` is behind `cfg(test)`; isolated SM62 `no HID` previews are specialized component fixtures, not the full Entropy editor or picker. |
| `README.md` (around lines 282–285) | Browser-only configuration is explicitly out of scope. |

The official Quick Start instructs users to connect a compatible Vial device, launch Entropy, select it from the device list, and wait for its layout to load. No documented demo mode, offline editor, UI-only mode, sample-editor profile, supported discovery-disable environment variable/argument, or browser preview was found. No fake device, virtual HID, spoofing, event injection, or runtime bypass was attempted. The existing no-device live screenshot at `1762 × 974` remains the only newly captured Entropy application state.

**Conclusion:** a genuine equivalent-state live comparison cannot currently be completed without a compatible device under the required safety restrictions. Existing Entropy screenshots and upstream source/docs remain useful qualitative and architectural evidence, but neither is a same-state live reference. This is an external reference limitation, not evidence that TH80 needs speculative styling changes. See the dedicated source-path and evidence-class sections in `PHASE14_2_VISUAL_COMPARISON.md`.

### 2. Accessibility infrastructure: still BLOCKED

Fresh checks in the current Hyprland/Wayland session establish:

- `org.a11y.Bus` is reachable on the session bus and `GetAddress` returns `unix:path=/run/user/1000/at-spi/bus_0`.
- The advertised path exists and `ss -xlpn` reports a listening Unix stream socket owned by the AT-SPI dbus-broker. However, a direct GDBus connection to that advertised address still returns `Could not connect: Connection refused`.
- `at-spi-bus-launcher`, its dbus-broker, and an `at-spi2-registryd --use-gnome-session` process are present. The session bus does not expose `org.a11y.atspi.Registry`; `gdbus introspect` reports `ServiceUnknown` / not activatable. Additional orphaned registry/launcher processes exist but do not establish a working registry.
- `at-spi2-core 2.62.1-1` is installed. `orca`, `accerciser`, and `python-pyatspi` are not installed. No system packages or user session services were changed.

Thus the infrastructure has changed from the earlier observation of no socket listener to a listener being present, but it is **not functional end-to-end**: the advertised AT-SPI socket refuses a client connection and the registry name is unavailable. The desktop session was not restarted or modified, and no system-wide service installation was attempted.

Minimum future runtime-accessibility prerequisites: a supported desktop session with a usable AT-SPI bus address/socket; a functioning `org.a11y.atspi.Registry` on that bus; Orca or another compatible screen reader; and an isolated TH80 instance whose accessible tree can be traversed and whose labels, roles, focus changes, modal announcement, and screen-reader navigation can be heard/observed. AccessKit enablement, egui tree-generation tests and accessible labels remain source/test-level evidence only. They do not satisfy runtime registration or screen-reader acceptance.

### 3. Dialog focus traversal: remains PARTIAL

The exact outstanding direct-interaction requirement is keyboard-only traversal of the **Unsaved changes** profile-switch modal, not merely opening it or clicking its buttons. `crates/th80-gui/src/app.rs` creates this modal with three actions in order: `Save and open`, `Discard and open`, and `Cancel` (around lines 521–545). The previous X11 run directly displayed the dialog and dismissed it, but did not record keyboard focus entering the modal, sequential Tab movement through all three actions, Shift+Tab in reverse, wrap/escape behavior, Enter/Space activation, or focus restoration to the pre-modal control.

Existing tests do not close this gap: `tab_and_shift_tab_move_focus_between_navigation_controls` verifies the main navigation controls; `modal_closure_restores_the_previous_keyboard_focus` exercises closure of the macro-delete modal, not the unsaved-profile-open modal. The unsaved-change workflow tests cover state/confirmation behavior but do not establish native keyboard focus traversal. The isolated X11 assessment process has been torn down; the only remaining TH80 process is the user's pre-existing session, which was deliberately left untouched. No new live interaction was run against that user session.

Required direct acceptance sequence in a fresh isolated synthetic-profile instance: focus a known control before making the profile dirty; open the unsaved-change modal via keyboard; record initial focus; press Tab until all three actions have been visited in order and determine whether focus wraps or leaves the modal; repeat with Shift+Tab; verify Enter/Space activates the focused action without accidental selection; verify Escape/Cancel closes without changing the active profile; verify focus returns to the original control. Separately verify Save and open and Discard and open outcomes using disposable synthetic profiles. Keep this PARTIAL until those observations are captured. Do not infer a pass from egui tests alone.

### 4. Windows and macOS prerequisites

| Target | Fresh finding | Minimum prerequisite to close |
|---|---|---|
| Windows MSVC | `x86_64-pc-windows-msvc` remains installed; the previous `cargo check -p th80-gui --target x86_64-pc-windows-msvc --locked` remains valid. No `link.exe` or `cl.exe` was found in standard executable search paths or inspected development locations. Wine is not an MSVC linker or native Windows desktop. | Install/use Visual Studio Build Tools with the Desktop development with C++ workload (MSVC linker, compiler and Windows SDK), then link the native target. For runtime acceptance, use a real Windows host/VM/runner and exercise the GUI there. Do not call target `cargo check` native execution. |
| macOS | No `xcrun`, Apple SDK, Xcode.app or Command Line Tools installation was found; neither Apple Rust target is installed. | A macOS host/runner with Xcode or Command Line Tools and the relevant Apple SDK; install `aarch64-apple-darwin` (Apple Silicon) or `x86_64-apple-darwin` (Intel) as appropriate, then compile and run natively. |

No CI expansion or cross-platform infrastructure was added.

### 5. Final acceptance matrix

| Acceptance area | Status | Evidence / closure condition |
|---|---|---|
| Core native UI structure and offline profile workflows | PASS | Existing implementation and regression evidence; no redesign rewrite. |
| Automated Linux Rust validation | PASS (latest source run) | 136 workspace tests, 44 GUI tests, formatting, strict Clippy, release build and docs passed; no source code changed in this investigation. |
| GUI hardware-transport isolation | PASS | Existing dependency/source audit; no hardware transport introduced. |
| Matched-pixel live viewport attempt | PASS | Entropy and TH80 client windows were both captured at 1762 × 974 on the same isolated XWayland display/compositor scale 1. |
| Equivalent-state live Entropy/TH80 comparison | BLOCKED | Upstream has no supported device-free editor path; live Entropy shows no-device state. Reopen only with an upstream-supported preview or genuine user-supplied equivalent-state captures with known scale. |
| Screenshot/source-based reference analysis | PARTIAL | Retained screenshots and upstream source/docs support qualitative analysis, not live equivalent-state acceptance. |
| Evidence-backed visual refinements | NOT TESTED | No matched editor-state evidence supports a change; no source modification is justified. |
| X11 primary navigation, layer navigation/rename, theme, scaling, picker, profile switch, target isolation and unsaved guard | PASS | Direct isolated X11 interactions documented in the controlled assessment appendix. |
| Unsaved-profile modal keyboard focus traversal | PARTIAL | The modal was opened/dismissed; Tab/Shift+Tab sequence, activation, trapping/wrap and focus restoration have not been directly verified for this specific modal. |
| AccessKit/egui-generated tree semantics | PASS | Existing source and test-context assertions. |
| AT-SPI bus/socket, registry registration and screen-reader interaction | BLOCKED | Advertised socket still refuses connections; registry D-Bus name unavailable; no Orca/Accerciser/PyAT-SPI. Requires working bus+registry and screen reader in a supported session. |
| Windows MSVC Rust target check | PASS | Previously completed `cargo check`; compile/type-check only. |
| Windows linking and native runtime | BLOCKED | Requires MSVC Build Tools/link.exe and a Windows host/runner. |
| macOS compilation and native runtime | BLOCKED | Requires Apple SDK/Xcode Command Line Tools, matching Rust target and macOS runner. |

Phase 14.1 remains OPEN at **9 PASS / 5 PARTIAL / 4 BLOCKED**. Phase 14.2 remains **OPEN / NOT ACCEPTED**. No blocked item was promoted based on indirect evidence.

### 6. External closure checklist

1. **Entropy visual reference:** obtain an upstream-supported device-free editor/preview if Ergohaven adds one, or provide authentic Entropy Layout, inspector, layer and light/dark picker captures with recorded client viewport and scale. Do not fabricate a device or infer state geometry from a no-device screen.
2. **Dialog focus:** run the explicit unsaved-profile modal Tab/Shift+Tab/activation/focus-return sequence in a separate synthetic-profile X11/Wayland test instance and retain captures/logs.
3. **Runtime accessibility:** restore a working AT-SPI socket/bus and registry in a supported session; use Orca or another compatible screen reader to verify actual TH80 registration, tree traversal, announcements and modal/focus behavior.
4. **Windows:** provide Visual Studio Build Tools/MSVC linker + Windows SDK, then native Windows link and GUI runtime execution.
5. **macOS:** provide an Apple host with Xcode/Command Line Tools and the matching Apple SDK, then compile and run on macOS.
6. **Regression:** if any source change is justified after these checks, rerun the full required Linux regression suite and applicable cross-target checks before acceptance.

### 7. Files and safety

- Modified existing reports only: `PHASE14_2_VISUAL_COMPARISON.md` and `PHASE14_2_TEST_AND_REGRESSION_REPORT.md`.
- No application source, Cargo manifest/lockfile, profile fixture, README, firmware or hardware-isolation code was changed. No dependencies were added.
- Upstream Entropy source was inspected in a separate shallow checkout outside the TH80 repository; no upstream files were copied into the project.
- No physical keyboard or HID/USB device was accessed; no device was emulated or spoofed. Existing user TH80 session was not manipulated. No package installation, system service change, Git staging/commit/remote/push, reset or clean operation occurred.
- The previous complete Linux regression results remain the latest because no application source changed; they were not redundantly rerun.

### Regression decision

No source code changed in this round. The full source-validation batch was therefore not rerun: the latest recorded results remain `cargo fmt --all -- --check` PASS, `cargo test --workspace --quiet` 136/136 PASS, `cargo test -p th80-gui --quiet` 44/44 PASS, strict workspace Clippy PASS, workspace release build PASS, workspace docs PASS, and Windows-target `cargo check` PASS. If any source file is changed in a later refinement, rerun the complete required regression list before acceptance.

### Precise remaining actions

1. Obtain an Entropy build/reference environment that can display the requested Layout, inspector, layers and light/dark picker without violating the no-hardware-communication boundary. A user-provided authorized reference capture at the same viewport/known scale is also acceptable.
2. Historical X11 rename text-entry/commit remains unverified. The synthetic layer rename is now directly verified under Native Wayland in section 9; do not infer X11 behavior from that result.
3. Restore a functioning AT-SPI registry/socket and install/use a screen reader in a controlled environment before claiming runtime accessibility.
4. Run native Windows build/runtime with Microsoft Build Tools and a Windows host/runner; run macOS checks only on an Apple SDK/macOS runner.

### 8. Native Wayland / Hyprland runtime investigation (2026-11-02)

The user explicitly designated the actual Wayland + Hyprland desktop as the primary development and acceptance environment. A separate isolated TH80 process was therefore launched natively on the existing session; no X11/XWayland or nested compositor was used for this new evidence.

**Native Wayland proof**

- Isolated process PID during capture: `2862597` (`./target/release/th80-gui`).
- Process environment: `XDG_SESSION_TYPE=wayland`, `WAYLAND_DISPLAY=wayland-1`, `XDG_RUNTIME_DIR=/run/user/1000`; `DISPLAY` was unset for this process.
- `hyprctl clients -j` reported class/title `TH80 Studio`, PID `2862597`, and `xwayland: false`.
- Synthetic profile storage: `/tmp/th80-phase14-2-wayland/profiles`; isolated settings/config/data roots: `/tmp/th80-phase14-2-wayland/config` and `/tmp/th80-phase14-2-wayland/data`. The fixture was copied from the prior controlled synthetic assessment; the user's real profile directory was not used.
- Monitor: `HDMI-A-5`, 1920 × 1080, 60 Hz, Hyprland scale 1, transform 0.
- Client viewport during the new capture: 926 × 1012 pixels, tiled at `[974, 48]`. This is the actual client size in the current desktop layout, not the earlier 1762 × 974 controlled X11 viewport. No resize/crop/upscale was applied to the client screenshot; `grim -g` captured the exact Hyprland client rectangle.
- Native screenshot PNGs are in `.phase14-2-evidence/wayland/`. The folder is separate from `.phase14-2-evidence/controlled-x11/` and the older mixed historical `.phase14-2-evidence/th80/` folder.

**Direct Wayland observations**

- The native application rendered the Layout workspace and the synthetic `Phase 14.2 Visual Fixture` profile. The captured layer content included `0. Main` and `1. Navigation`, with the fixture's logical positions/actions visible.
- Native keyboard input was sent using `wtype` through the Wayland virtual-keyboard protocol. The profile selector was opened with keyboard focus and the synthetic fixture was selected; subsequent Tab navigation visibly outlined the global `Advanced` and `Config` navigation controls. Screenshots `th80-native-tab-4.png` and `th80-native-tab-5.png` preserve those focus states. `th80-native-tab-1.png` and `th80-native-tab-2.png` preserve the two fixture layer views observed during keyboard traversal.
- Native dark-theme Layout screenshot: `th80-layout-fixture-native-wayland.png`. The separate `th80-layout-dark-native-wayland-client.png` was captured before selecting the populated fixture and shows the initial no-profile state; it is not evidence of fixture content.
- An attempted keyboard activation did not establish a reliable transition into Config or Advanced, and the nominal `th80-config-native-wayland.png` / `th80-advanced-native-wayland.png` captures show Layout content. Those two misleadingly named captures were removed; no Config/Advanced page rendering is claimed from this run.
- No direct native Wayland verification was completed for the action picker, selected-position inspector, profile-layer rename, theme toggle, scale change, or the unsaved-profile dialog. In particular, no native Wayland Tab/Shift+Tab/Enter/Space/Escape sequence was run against the three-action unsaved-profile modal. The dialog focus requirement therefore remains PARTIAL.
- The new run establishes a native Wayland launch and Layout/layer rendering, not full Wayland acceptance. No reproducible Wayland-specific defect was established; no source changes were justified.

**Acceptance matrix update**

| Acceptance area | Native Wayland evidence/status |
|---|---|
| Native Wayland launch | PASS — process environment and Hyprland `xwayland: false` agree. |
| Synthetic profile isolation | PASS — separate `TH80_PROFILE_DIR` and XDG config/data roots. |
| Layout rendering | PASS — native capture of the populated synthetic profile. |
| Global navigation keyboard focus | PARTIAL — focus rings observed on Advanced and Config; activation/page rendering not established. |
| Layer view/navigation | PARTIAL — both fixture layer views captured during keyboard traversal; complete layer-operation acceptance not repeated. |
| Inspector, picker, Config/Advanced rendering | NOT VERIFIED in this native run. |
| Native Wayland theme/scaling | NOT VERIFIED in this native run. |
| Unsaved-profile dialog keyboard focus | PARTIAL — no native dialog traversal or activation test completed. |

Existing X11 evidence and prior X11 PASS results remain X11-only. They are not relabeled as native Wayland evidence. Entropy, AT-SPI, Windows and macOS blockers are unchanged. No application source, manifests, dependencies or real user profile files were changed. The automated Linux suite was not rerun because there were no source changes.

### 9. Native Wayland functional and visual review — 2026-11-02

This is a separate, later Native Wayland pass. It supersedes the earlier section 8 statement that Advanced/Config, inspector, picker, theme and scaling had not yet been exercised. It does **not** supersede the still-open unsaved-profile-dialog requirement.

**Runtime identity and isolation**

- Assessment process: PID `2984032`, `target/release/th80-gui`.
- Environment: `XDG_SESSION_TYPE=wayland`, `WAYLAND_DISPLAY=wayland-1`, `DISPLAY` unset, `XDG_RUNTIME_DIR=/run/user/1000`.
- Hyprland client: `class=TH80 Studio`, `xwayland=false`.
- Monitor: `HDMI-A-5`, 1920 × 1080, scale 1.0.
- Client when tiled alone: 1880 × 1012 at `[20,48]`.
- Isolated profile/config/data roots: `/tmp/th80-phase14-2-wayland/final-review2/{profiles,config,data}`. The directory contains only copied synthetic fixture profiles. The user's actual TH80 profile directory and profile data were not read or modified.
- Input was sent to the native Wayland application with `wtype` and Hyprland's window-targeted keyboard shortcut path. No physical keyboard, HID, USB, serial or firmware communication occurred.

**Direct functional results**

| Area | Native Wayland result | Evidence / qualification |
|---|---|---|
| Layout | PASS | `01-layout-dark-full.png`, `01-layout-light.png`. Full monitor captures at 1920 × 1080. |
| Advanced activation/rendering | PASS | `03-advanced-dark.png` shows the actual Advanced → Macros page. |
| Config activation/rendering | PASS | `04-config-app-settings.png` and `04-config-light.png` show Config → App Settings. Other Config subpages were not all verified. |
| Inspector | PASS | `02-layout-inspector.png`, `02-layout-inspector-layer2.png`, and `02-layout-inspector-logical03.png` show selected positions and corresponding inspector values across positions/layers. |
| Layer navigation | PASS | `07-layer-navigation.png` shows the second synthetic layer selected. |
| Layer rename | PASS | `08-layer-rename-state.png` shows the synthetic layer renamed to `1. Navigation QA`. This was an in-memory synthetic-profile edit. |
| Action picker, dark theme | PASS | `05-picker-dark-open.png` shows the actual picker in dark appearance. |
| Action picker, light theme | PASS | `05-picker-light-open.png`, `05-picker-light-search.png`, `05-picker-light-keys.png`. Search and category switching were exercised. |
| Action selection | PASS | The `KEY_Q` option was selected in the picker; `05-picker-action-selected.png` and `05-picker-light-keys.png` retain selection states. |
| Apply / target isolation | PASS | The paired dark-theme captures `05-picker-dark-open.png` and `05-picker-action-applied.png` show logical-02 changing from `KEY_HOME` to `KEY_Q`; logical-01 (`Switch · base`), logical-03 (`KEY_UP`), logical-04 (`KEY_DOWN`), and the other visible positions remain unchanged. The selected logical-02 inspector also shows `KEY_Q` after apply. |
| Picker cancellation | PASS | `picker-cancel-restored.png` shows Escape closing the picker while the prior `KEY_HOME` assignment remains visible. This is picker cancellation, not the unsaved-profile dialog's Escape behavior. |
| Light/dark theme | PASS | `10-config-light-theme.png`, `10-config-dark-theme.png`, `01-layout-light.png`, and `02-layout-inspector-layer2.png`. Both appearances rendered. |
| Accent color | NOT VERIFIED | No color change was made. |
| UI scaling increase | PASS | `10-scale-increased.png` shows 105% and the correspondingly enlarged controls. |
| UI scaling restore | PARTIAL | A trustworthy in-app Reset-to-100% screenshot was not obtained. The clean initial Layout capture shows 100%; a later attempt to restore after increasing scale was inconclusive. The isolated settings file will be reset to 1.0 after the assessment process exits; that is cleanup, not proof of the in-app Reset control. |
| Profile selector | PARTIAL | The selector opened and displayed both synthetic profiles. Keyboard selection of the second option was not reliable in this session. `exploratory/09-unsaved-profile-dialog-initial.png` is a selector popup, not the unsaved-profile dialog. |
| Unsaved-profile dialog focus | PARTIAL / NOT VERIFIED | The three-action dialog was not successfully opened in this Native Wayland round. No initial-focus, Tab, Shift+Tab, wrap/escape, Enter/Space, Escape semantics or focus-restoration results are claimed. |

**Visual evidence index**

The curated visual set and capture metadata are documented in `.phase14-2-evidence/wayland/final-review/README.md`. Primary evidence is stored in `.phase14-2-evidence/wayland/final-review/`; exploratory compositor/focus captures are in its `exploratory/` subdirectory. Full-monitor captures are 1920 × 1080; exact client-region captures are 1880 × 1012. No screenshot was upscaled. The screenshot named `09-unsaved-profile-dialog-initial.png` was moved into `exploratory/` because it did not show the requested dialog.

**Assessment limitations and safety record**

- One isolated TH80 attempt displayed a Hyprland “Application Not Responding” prompt and later exited. This was not reproduced as a deterministic TH80 defect; no source change was made.
- A compositor pointer-injection experiment did not reliably target TH80 and was abandoned. A browser microphone-permission prompt appeared during that experiment; it was dismissed without granting microphone access. No credentials were entered into any prompt. A separate package-authentication prompt for `pacman -S --needed neatvnc tigervnc` was left untouched.
- One existing `mpv` window was inadvertently moved from workspace 7 to workspace 9 during an incorrect window-target test and was returned to workspace 7. Playback was not terminated. The pre-existing TH80 process/session was not terminated or manipulated. No user profile files were accessed.
- The isolated assessment process PID `2984032` was closed after evidence capture. No other TH80 process was signaled. A previously observed `th80-gui --help` process (PID `2824651`) was absent at the final check; its exit was not initiated by this task and its cause was not established.
- No source, Cargo manifest/lockfile, dependencies, firmware, or user profile data changed. No Git staging, commit, reset, clean, remote, push or publish operation occurred.

**Updated Native Wayland acceptance matrix**

| Requirement | Status |
|---|---|
| Native Wayland process identity / XWayland exclusion | PASS |
| Layout | PASS |
| Advanced page | PASS |
| Config App Settings page | PASS; remaining Config subpages not fully exercised |
| Selected-position inspector | PASS |
| Layer navigation and rename | PASS |
| Action picker open/search/category switch in both themes | PASS |
| Action selection and apply to a changed target with proven isolation | PASS |
| Picker Escape cancellation | PASS |
| Light/dark theme | PASS |
| UI scale increase | PASS |
| In-app UI scale reset | PARTIAL |
| Accent color | NOT VERIFIED |
| Unsaved-profile modal initial focus and complete keyboard traversal | PARTIAL / NOT VERIFIED |
| Modal action semantics and focus restoration | PARTIAL / NOT VERIFIED |
| Runtime screen-reader accessibility | BLOCKED — unchanged AT-SPI prerequisite |
| Entropy equivalent interactive visual reference | BLOCKED — unchanged Outcome B |
| Native Windows and macOS execution | BLOCKED — unchanged host/toolchain prerequisites |

No application source changed; the inherited automated regression results remain the latest results and were not rerun. Phase 14.2 remains **OPEN / NOT ACCEPTED**. The unsaved-profile dialog, in-app scale reset, accent color and external blockers remain outstanding.

### 10. Isolated Weston VNC desktop bootstrap — 2026-11-02 (GUI interaction pending)

This section records the separate virtual-compositor setup requested after section 9. The VNC client is now authenticated and connected, but this setup is still not counted as GUI acceptance because no reliable remote input or usable visual evidence has been established.

**Weston VNC runtime**

- Weston `15.0.1` was launched as the normal unprivileged `vergoboy` user (UID/GID 1000), PID `3072739`, using the installed VNC backend. It was not run as root.
- Dedicated test root: `/tmp/th80-agent-acceptance-20261102/`; dedicated runtime directory: `/tmp/th80-agent-acceptance-20261102/runtime`; Wayland socket: `wayland-th80-agent`.
- Virtual output requested at 1920 × 1080, with the backend's default scale 1. No primary monitor or Hyprland output was changed.
- VNC address and port: `127.0.0.1:5901`. `ss -ltnp '( sport = :5901 )'` showed the exact loopback listener; it did not show a wildcard or LAN listener.
- TLS certificate and key are dedicated to this temporary test root. Weston logged `TLS support activated`. A raw RFB/VeNCrypt negotiation selected the X509Plain subtype and completed a TLS 1.3 handshake using `TLS_AES_256_GCM_SHA384`.
- The existing `/etc/pam.d/weston-remote-access` service was left unchanged. The backend authenticates VNC clients through this PAM service. The user opened TigerVNC and authenticated interactively; Weston logged `New VNC client connected` at 21:54:59 and `ss` showed an established connection from `127.0.0.1:50316`. PAM authentication is therefore verified for this viewer session. The password was not inspected, captured, stored, or transmitted by the agent.
- The VNC server offered RFB security types `[19, 129, 5]`; VeNCrypt exposed X509Plain subtype `262`. No unauthenticated viewer session was treated as a successful input test.
- A `grim` capture attempt reported that the compositor does not support `zwlr_screencopy_manager_v1`. `weston-screenshooter` reported an unauthorized capture and the saved 1920 × 1080 framebuffer is entirely black; it is retained only as `.phase14-2-evidence/wayland-isolated/diagnostic-black-framebuffer.png`, not acceptance evidence. The exact screenshooter test process was terminated. No usable screenshot was obtained.

**Native TH80 process**

- TH80 PID `3075879` (`target/release/th80-gui`) was launched with `XDG_SESSION_TYPE=wayland`, `XDG_RUNTIME_DIR=/tmp/th80-agent-acceptance-20261102/runtime`, and `WAYLAND_DISPLAY=wayland-th80-agent`.
- `DISPLAY` was absent from `/proc/3075879/environ`. The process uses isolated `HOME`, `XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `XDG_CACHE_HOME`, `TH80_PROFILE_DIR`, and `TH80_SETTINGS_PATH` paths, all under the dedicated `/tmp` root.
- Two disposable profiles, `synthetic-profile-a.json` and `synthetic-profile-b.json`, were created from a copied synthetic fixture. The original repository fixture was not modified. Initial settings are `ui_scale: 1.0`, `accent_color: [190, 126, 145]`, and light appearance.
- The dedicated Wayland socket exists and has established connections. TH80 remains running; no real profile directory was supplied to it.

**Direct acceptance not yet performed**

| Requirement | Status | Reason |
|---|---|---|
| Independent VNC mouse and keyboard input | NOT VERIFIED | An authenticated TigerVNC client is connected, but no input event has been confirmed. `wtype -k Tab` fails because Weston does not advertise `zwp_virtual_keyboard_manager_v1`; global/primary-session input injection is prohibited. |
| Unsaved-profile modal initial focus and Tab/Shift+Tab traversal | NOT VERIFIED | No visible interaction was performed. |
| Enter/Space, Escape/Cancel, Save and open, Discard and open, and focus restoration | NOT VERIFIED | No visible interaction was performed. |
| In-app 100% → 105% → Reset to 100% | NOT VERIFIED | No visible interaction was performed; the settings file remains at its original 100% value. |
| Accent-color edit, rendered update, and restoration | NOT VERIFIED | No visible interaction was performed; the settings file remains at its original accent value. |
| Genuine isolated Weston screenshots | NOT VERIFIED | The available Weston capture is all black; the connected viewer has not yielded a verified screenshot. The black diagnostic image is explicitly excluded from acceptance evidence. |

**Safety and repository record**

- The primary Hyprland process remained PID `1921`; its `HDMI-A-5` monitor remained 1920 × 1080 at scale 1. The previously observed `ydotoold` PID `2233780` was not used or terminated. The existing `mpv` process remained present.
- No primary-session input injection, workspace/window manipulation, physical input-device access, X11/XWayland acceptance, hardware communication, firmware operation, or real profile access was performed in this bootstrap.
- The Weston server, TH80 process, and user's authenticated TigerVNC viewer are intentionally left running. Lifecycle script: `/tmp/th80-agent-acceptance-20261102/bin/th80-weston` (`start`, `status`, `logs`, `connect`, `stop`). The `screenshot` subcommand is not a verified capture path on this backend and must not be treated as evidence.
- No application source, manifest, lockfile, or existing profile fixture changed. The report was updated only to record the observed bootstrap. Git remains an unborn `main` with the existing work untracked; nothing was staged, committed, reset, cleaned, or deleted.

**Next required step:** use the already connected TigerVNC viewer to establish visible mouse/keyboard interaction and a screenshot path, or have the user provide genuine captures from that viewer. Do not inject input into the primary Hyprland session, and do not treat `wtype` failure or the black diagnostic image as GUI evidence.

### 11. Host-side driver implementation and fresh validation — 2026-11-02

**Implemented**

- Split passive discovery from active transport. `DiscoveryProvider` and `DescriptorSource` live in `discovery.rs`; `Transport` contains only active Feature/Input/Output operations, all of which remain fail-closed through `DisabledTransport`.
- Added `SysfsDiscovery` and `SysfsDescriptorSource`, reusable Linux providers that read USB sysfs metadata and readable HID report-descriptor files only. The typed inventory retains ephemeral instance/interface/endpoint paths, descriptor source/bytes, parsed capability models and fingerprints. The CLI now consumes the same provider models rather than independently re-enumerating interfaces or reparsing descriptor hex.
- Hardened configured sysfs roots: custom-root device entries, attributes, and descriptor symlinks are canonicalized and constrained to their configured boundary; the system provider is constrained to `/sys` to accommodate normal USB bus symlinks. Device identity is revalidated after enumeration. Descriptor reads are capped at 4096 bytes; empty, truncated, malformed, and oversized descriptors return explicit errors. No fallback to real `/sys` occurs when synthetic roots are configured.
- Reused the shared `DeviceIdentity`/`DeviceKind`, `DeviceMetadata`, `InterfaceRole`, `EndpointInfo`, `InterfaceInfo`, `DescriptorFingerprint`, and `CapabilityProfile` models while retaining the existing CLI JSON role labels and schema version. Unknown USB IDs are excluded; interface/endpoint ordering is deterministic. No test enumerated or opened the physical keyboard or receiver.
- Kept custom-root CLI runs hermetic beyond sysfs: the optional `/dev/bus/usb` existence check is enabled only when both default system sysfs roots are selected. The synthetic CLI fixture supplies bus/device numbers and asserts that no host USB-node path is exposed.
- Added synthetic sysfs/CLI integration coverage for both verified VID/PID roles, exclusion of an unrelated unknown device, typed provider results, descriptor-derived Feature capability, identity reuse/disconnect handling, malformed/oversized descriptor reporting, USB-root/attribute/descriptor symlink escapes, and JSON inventory/capability output.
- Expanded forward-compatible profile preservation to unknown fields at Profile, metadata, configuration, keymap, macro-set, lighting, and options section boundaries. The GUI shows retained extension maps read-only. CLI duplicate and GUI duplicate/Save As paths now retain metadata extensions as well as configuration/top-level extensions. Unknown fields inside unsupported leaf/action schemas are explicitly rejected rather than silently discarded or interpreted.
- No command IDs, opcode semantics, response formats, checksum branches, firmware behavior, or device capabilities were invented. The existing evidence-backed in-memory macOS/Windows packet transformations remain pure-memory only.

**Fresh validation after source changes**

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo test --workspace --locked` | PASS — 150 tests, 0 failures |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo build --workspace --release --locked` | PASS |
| `cargo doc --workspace --no-deps --locked` | PASS |
| `cargo tree -p th80-gui -e normal` | PASS — no HIDAPI, rusb/libusb, udev, or hidraw transport dependency matches |
| `cargo tree -p th80-probe -e normal` | PASS — no HIDAPI, rusb/libusb, or udev transport dependency matches |

The 150 tests comprise CLI 9, core 38 (32 unit + 1 compatibility + 5 process), GUI 44, probe 56 (30 library + 5 binary + 12 protocol + 9 sysfs integration), and legacy HID research 3. The GUI test subset remains 44/44. These are fresh post-change results and supersede the earlier 136-test count in the historical table above.

**Dependency and boundary audit**

- `cargo tree -p th80-gui --edges normal` contains no HIDAPI, rusb/libusb, udev, or hidraw transport dependency.
- `cargo tree -p th80-probe --edges normal` contains no HIDAPI, rusb/libusb, or udev dependency. The sysfs provider uses only Rust standard-library filesystem access plus the existing descriptor parser.
- The root/attribute/descriptor escape regression tests use only temporary synthetic trees; the end-to-end CLI integration test injects both synthetic roots via `TH80_USB_SYSFS_ROOT` and `TH80_HIDRAW_SYSFS_ROOT`. No physical keyboard, receiver, host `/sys` inventory, or `/dev/bus/usb` node was accessed by the test suite.
- The separate root `th80-hid-research` binary still has its pre-existing HIDAPI dependency; it is not linked into `th80-gui`, `th80-cli`, or `th80-probe` and was not executed against hardware.
- Feature GET/SET, Input read, and Output write remain disabled. C01–C12 device command semantics, exact target firmware applicability, and transport setup remain unverified. No physical-device communication, firmware operation, or persistent keyboard change occurred.

**Remaining acceptance and hardware blockers**

- The three Native Wayland GUI checks remain NOT VERIFIED: the unsaved-profile modal's actual focus/traversal/action semantics; the in-app 100% → 105% → Reset sequence; and accent-color edit/restoration. The viewer is authenticated, but agent-driven input is blocked by the missing Weston virtual-keyboard protocol and the black framebuffer capture. No primary-session input injection was used.
- AT-SPI runtime, equivalent Entropy interactive visual comparison, Windows native linking/runtime, and macOS native execution remain blocked as recorded above.
- Active hardware transport and all device commands remain blocked pending exact target-specific request/response evidence and a separately authorized hardware phase. This implementation completes the safe passive/offline host-side functionality; it does not complete firmware or physical-device integration.

No Git staging, commit, push, reset, clean, or destructive cleanup occurred. Existing untracked project files and user profile data were preserved. The isolated Weston/TH80/VNC processes remain running for the pending GUI checks.

