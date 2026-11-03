# Phase 14.1 — Accessibility and Keyboard Navigation

## Accessibility stack inspected

The GUI uses `eframe` 0.35 with its `accesskit` feature enabled. The resolved normal dependency tree includes:

- `accesskit` 0.24.1
- `accesskit_winit` 0.32.2
- `accesskit_unix` 0.21.1
- `accesskit_atspi_common` 0.18.1
- AT-SPI crates including `atspi` 0.29.0 and `atspi-common` 0.13.0

The application adds explicit labels to widgets through `screens::set_accessible_label`. Modal dialogs are assigned the AccessKit `Dialog` role, modal state, and a label through `mark_modal_dialog`.

## Automated AccessKit checks

The GUI test suite includes tests for:

- Accessible names on interactive controls across Dashboard, Keymap, Layers, Macros, Lighting, Options and Profile.
- Profile text-input accessible names.
- Modal dialog role, label and modal state.
- Macro deletion dialog content and referenced-macro warning.
- Keyboard focus and text editing in simulated egui input.
- Tab/Shift+Tab traversal and Enter/Space activation in simulated egui input.
- Focus restoration after modal closure.
- Macro confirmation remaining bound to the originally selected macro ID.
- Macro reordering controls remaining inside a 1000 × 660 viewport.

The accessible-name test checks the relevant interactive roles, including Button, TextInput, MultilineTextInput, ComboBox, CheckBox, Slider, SpinButton and ColorWell. These tests inspect egui's generated AccessKit update; they are not a native platform accessibility-tree or screen-reader test.

## Linux AT-SPI runtime investigation

The session had an AT-SPI bus and registry running. `gsettings get org.gnome.desktop.interface toolkit-accessibility` returned `true`. However:

- `orca` was not installed.
- `accerciser` was not installed.
- No `org.a11y.Status` / `ScreenReaderEnabled` service/property was exposed on the session bus.
- The AT-SPI registry's application list did not contain TH80 Studio while the Wayland GUI was running.

Consequently, the application did not expose a runtime tree that could be inspected through AT-SPI in this session, and no Orca/screen-reader interaction was possible. AccessKit being present in the dependency tree and egui unit tests passing do **not** establish end-to-end screen-reader compatibility. Focus announcements, live validation announcements, and screen-reader behavior remain unverified.

## Keyboard interaction findings

### Real desktop input

On the native Wayland application, `ydotool` sent Tab, Shift+Tab and Enter without mouse clicks during the recorded sequence. Enter activated a focused navigation control and moved from Keymap to Dashboard. Shift+Tab moved focus back to the toolbar; the focus indication was visible on the Validate control. Separate keyboard-driven profile metadata editing and saving are captured in `phase14-1-native-wayland-keyboard-profile.png`, `phase14-1-native-wayland-keyboard-save-focus.png`, and `phase14-1-native-wayland-keyboard-save-result.png`.

The keyboard-only sample captures are `phase14-1-native-wayland-keyboard-only-tab-sample.png`, `phase14-1-native-wayland-keyboard-only-tab-sample-2.png`, `phase14-1-native-wayland-keyboard-only-tab-sample-3.png`, and `phase14-1-native-wayland-keyboard-live-tab.png`, `phase14-1-native-wayland-keyboard-live-enter.png`, and `phase14-1-native-wayland-keyboard-live-shift-tab.png`.

### Automated egui input

The following tests pass:

- `tab_and_shift_tab_move_focus_between_navigation_controls`
- `enter_and_space_activate_focused_navigation_buttons`
- `keyboard_focus_and_text_input_editing_work_without_pointer_events`
- `modal_closure_restores_the_previous_keyboard_focus`
- `escape_closes_macro_confirmation_and_preserves_macro_data`

These establish simulated input behavior in egui, not a full physical keyboard-only desktop journey. The native desktop pass did not exhaustively traverse every control on every page, every dialog, or all file-picker controls. Focus behavior after every destructive list mutation was not separately measured.

## Semantics and presentation review

| Area | Finding |
|---|---|
| Accessible names | Explicit labels are assigned to many interactive widgets; automated tree tests cover the seven pages. |
| Roles | egui supplies widget roles; application dialogs are explicitly marked as modal Dialog nodes. |
| Toggle semantics | Lighting's three-state selection is a labeled combo box; the UI distinguishes unspecified, enabled and disabled. |
| Slider semantics | Brightness/speed controls are egui sliders. A dedicated value-description/announcement test is not present. |
| Validation | Validation messages are textual and visible in the application. Runtime live-region/screen-reader announcement is unverified. |
| Focus | egui keyboard focus is tested; modal closure requests restoration of the previously focused widget. |
| Color | Lighting colors are accompanied by RGB controls and textual labels; color is not the only representation of the setting. |
| Contrast | The app currently selects egui's dark visuals. A separate high-contrast theme or conformance measurement was not found. |
| Text scaling | No application-specific text-scale control was found or tested. System scaling behavior remains unverified. |
| Dialog labels | Status, unsaved-change, profile-delete and macro-delete dialogs have explicit AccessKit dialog labels. |

## Windows and macOS

The cross-platform eframe/AccessKit dependency graph is present, but no native Windows UI Automation, Windows screen-reader, macOS Accessibility API, VoiceOver, or native GUI session was exercised in this phase. Prior cross-target compilation is not native accessibility verification. No Windows/macOS screen-reader support claim is made.

## Recommended next accessibility step

Before claiming screen-reader readiness, run the app on a Linux session with the AccessKit AT-SPI backend's screen-reader activation condition satisfied, install/use Orca, and inspect the live application tree for names, roles, states, values, focus events and modal transitions. Then repeat with native Windows UIA and macOS Accessibility/VoiceOver on their respective hosts. Do not add another accessibility framework until this runtime path has been diagnosed.

## Acceptance-closure runtime diagnosis (2026-11-02)

The live process was inspected without restarting it (PID 359824, native Wayland, 1000 × 660). Its environment includes `NO_AT_BRIDGE=1`. This variable disables the conventional GTK bridge; it does **not** by itself disable AccessKit's independent `accesskit_unix` backend, so it is not sufficient to explain the missing AccessKit application tree.

The AT-SPI infrastructure is running:

- Session service `org.a11y.Bus` is owned by `at-spi-bus-launcher` (PID 3230).
- Accessibility bus address: `/run/user/1000/at-spi/bus_0`.
- Registry service `org.a11y.atspi.Registry` is owned by `at-spi2-registryd` (PID 3269).
- `at-spi-dbus-bus.service` is active and `org.a11y.Status.IsEnabled` reads `true`.
- The registry bus contains the registry, launcher, portal and other applications, but no TH80 Studio connection.

The resolved `accesskit_unix` 0.21.1 backend subscribes to the `org.a11y.Status.ScreenReaderEnabled` property-change stream. The current session's `org.a11y.Status` introspection exposes only `IsEnabled`; attempts to read or set `ScreenReaderEnabled` return `InvalidArgs: No such property`. Phase 14.1.2 source inspection confirms this mismatch is specific to locked 0.21.1: `accesskit_unix` 0.24.1 subscribes to `IsEnabled` instead. Both implementations are transition-driven and do not first read the current property value.

This isolates the present failure to an AT-SPI status-interface/runtime compatibility or session integration gap: the registry and bus run, but the status property expected by the resolved AccessKit backend is not exported. The running process also has `NO_AT_BRIDGE=1`, which may affect GTK-native applications but is not the AccessKit status property. No code change was justified from this evidence. Orca, Accerciser and PyAT-SPI are not installed, so there was no screen-reader client to exercise even if the tree were published. No live TH80 roles, states, focus events or validation announcements could be inspected.

**Phase 14.1.1 classification (superseded by the source-level correction below):** runtime accessibility tree `BLOCKED` for the locked 0.21.1/property combination; actual screen-reader interaction `BLOCKED` (Orca absent and no published TH80 tree). Automated egui AccessKit update tests remain valid but do not close either runtime condition. No formal accessibility-conformance claim is made.

## Acceptance-closure keyboard traversal

The retained real desktop evidence remains limited to Tab, Shift+Tab and Enter on the Wayland app, including profile text editing/save. The prior screen-by-screen automated AccessKit/egui tests cover labels/roles on seven pages, navigation focus/activation, modal restoration and macro deletion target safety. The acceptance-closure attempt did not complete an exhaustive real keyboard-only pass of all seven pages and all modal/native chooser controls: the running window was on workspace 8 while the active desktop focus was elsewhere, and the installed Hyprland dispatcher rejected the available `hyprctl dispatch` syntax (`hl.dispatch(...)` parse errors). No keyboard focus was fabricated or inferred from mouse-driven captures.

| Screen / flow | Real keyboard-only coverage | Automated/simulated coverage | Closure status |
|---|---|---|---|
| Main navigation / Dashboard | Tab, Shift+Tab, Enter; retained Wayland captures | Tab/Shift+Tab and Enter/Space tests | PARTIAL |
| Profile fields and save | Keyboard text edit and save retained | Accessible text-input names and focus tests | PARTIAL |
| Keymap / Layers | Existing visual/pointer coverage; no exhaustive traversal | Seven-screen accessible-role/name tree tests | PARTIAL |
| Macros | Existing visual/pointer coverage; no exhaustive traversal of editor controls | Delete/cancel and captured-ID tests | PARTIAL |
| Lighting / Options | Existing visual/pointer coverage; no exhaustive traversal | Widget names/roles and value-state tests | PARTIAL |
| Validation / unsaved-change / delete dialogs | Some Tab/Enter dialog samples; not every path | Dialog semantics and focus restoration tests | PARTIAL |
| Native file chooser | Pointer-based async open/cancel/export test | Pending/result/cancel tests | PARTIAL; native chooser focus not audited |

No full keyboard-only traversal or absence-of-focus-traps claim is made. Arrow-key behavior for all combo boxes, lists and custom controls, Escape behavior for every modal, focus after profile deletion, and chooser-to-parent focus restoration remain unverified.

## Phase 14.1.1 clean-environment AT-SPI experiment (2026-11-02)

The previous runtime finding was tested with a second, separate Wayland instance rather than changing or restarting the preserved process:

- Original Wayland PID `359824` remained running with its original environment and synthetic repository.
- Test instance was launched with `NO_AT_BRIDGE` explicitly removed and `TH80_PROFILE_DIR=/tmp/th80-phase14-1.1-atspi-clean/profiles`.
- The clean-environment instance also failed to appear in the AT-SPI registry. It was stopped after the experiment; its log and isolated profile root remain under `/tmp/th80-phase14-1.1-atspi-clean`.
- The session AT-SPI Status interface still reports only `IsEnabled`; it does not expose `ScreenReaderEnabled`.
- Installed `at-spi2-core` package is 2.62.1. Resolved application dependencies are `eframe`/`egui-winit` 0.35.0, `accesskit_winit` 0.32.2 and `accesskit_unix` 0.21.1.

The clean test rules out `NO_AT_BRIDGE=1` as the sole cause. Exact Phase 14.1.2 source comparison refines the earlier diagnosis: locked `accesskit_unix` 0.21.1 listens for `ScreenReaderEnabled`, while 0.24.1 listens for `IsEnabled`. The installed `atspi-proxies` 0.13.0 source declares both proxy properties, but live `org.a11y.Status` introspection exposes only `IsEnabled`. The mismatch is specific to the locked backend's property subscription, not an inherent AccessKit requirement.

Orca remains absent. `pacman -Si orca` reports 5.24 MiB download / 37.07 MiB installed and a substantial dependency set including speech-dispatcher, liblouis, python-brltty and several Python/GObject components. It was not installed because this is not a small missing inspection utility and the underlying Status property mismatch would still prevent the AccessKit tree from appearing. No actual spoken output was tested.

**Phase 14.1.1 Linux accessibility classification (superseded by the Phase 14.1.2 compatibility resolution):** AT-SPI runtime registration `BLOCKED`; screen-reader interaction `BLOCKED`; automated egui/AccessKit semantics remain `PASS` only for existing test coverage. No screen-reader compliance claim is made.

## Phase 14.1.1 real X11 keyboard traversal evidence

A separate X11/Xwayland instance (PID 1346017) was used with `/tmp/th80-phase14-1.1-x11/profiles`; the original Wayland process was not touched. Real X11 keyboard input was sent with `xdotool`, and screenshots are retained as `phase14-1.1-x11-keyboard-dashboard-tab-1.png` through `-12.png`, `phase14-1.1-x11-keyboard-dashboard-shift-tab.png`, `phase14-1.1-x11-macro-tab-1.png` through `-8.png`, `phase14-1.1-x11-macro-shift-tab.png`, and `phase14-1.1-x11-macro-escape.png`.

Observed keyboard behavior:

- The profile-created / profile-deleted information dialog initially did not dismiss on the first Return from the initial focus state. Tab moved focus to the Dismiss button; Enter then dismissed it. This is a real focus-entry observation, not proof that every dialog has the same behavior.
- Dashboard Tab traversal moved a visible focus outline across interactive controls. Twelve forward Tab steps and a reverse Shift+Tab sample were captured. The contact sheet is `phase14-1.1-x11-keyboard-dashboard-tab-contact-sheet.png`.
- Macro editor Tab/Shift+Tab samples were captured while the 240-step synthetic macro was loaded. Escape was sent with no modal open; no modal-dismissal behavior can be inferred from that sample.
- The delete-confirmation modal was cancelled with its Cancel button; a click on the background dismissed the modal without activating the underlying Dashboard. No selection change occurred while it was open.

This is broader real X11 keyboard evidence, but it is **not** a full keyboard-only traversal of all controls on every screen. Arrow-key behavior for all combos/lists, every dialog entry/exit path, native chooser focus, keyboard activation of all sidebar pages, focus after deleting the selected profile, and all custom controls remain unverified. The per-screen result remains `PARTIAL`; no claim of full keyboard accessibility or no focus traps is made.

| Screen / workflow | Native keyboard evidence in this continuation | Result |
|---|---|---|
| Dashboard | 12 Tab steps, Shift+Tab, visible focus outlines | PARTIAL |
| Profile create / status dialog | Tab reached Dismiss; Enter dismissed; profile fields were edited with pointer in this X11 run | PARTIAL |
| Profile delete confirmation | Real Cancel click and outside-click dismissal; keyboard traversal of every dialog control not completed | PARTIAL |
| Keymap / Layers | Screens displayed; no exhaustive keyboard traversal or assignment editor traversal | PARTIAL |
| Macro editor | Eight Tab steps, Shift+Tab and Escape sample with 240-step profile | PARTIAL |
| Lighting / Options | Screens displayed; no exhaustive keyboard-only control traversal | PARTIAL |
| Import/export chooser | Native GTK portal chooser appeared; no full keyboard-only path/selection/save traversal | PARTIAL |
| Validation / unsaved changes | Prior Wayland evidence and automated tests exist; not repeated as a complete X11 keyboard sequence | PARTIAL |

The X11 file chooser is a separate Wayland portal window (`xdg-desktop-portal-gtk`), not an X11 child. Its presence is real; its full keyboard accessibility and save/selection workflow were not verified here.

### Preservation and scope

All 143 screenshots present at the start of Phase 14.1.1 remain present by filename; the new evidence adds 78 screenshots (221 total). The original Wayland process remains live. The X11 and clean-environment Wayland test instances were stopped after their isolated tests. No source-code accessibility fix was made because the reproduced blocker is the host AT-SPI Status interface, not a confirmed missing accessible name/role in TH80.

## Phase 14.1.2 — exact AccessKit and AT-SPI compatibility resolution (2026-11-02)

### Exact locked and candidate dependency source

The main `Cargo.lock` remains unchanged and resolves:

| Crate | Locked version |
|---|---:|
| `eframe` | 0.35.0 |
| `accesskit` | 0.24.1 |
| `accesskit_unix` | 0.21.1 |
| `accesskit_atspi_common` | 0.18.1 |
| `atspi` | 0.29.0 |
| `atspi-proxies` | 0.13.0 |
| `zbus` | 5.19.0 |

The isolated upgrade copy resolves `eframe` 0.37.0, `accesskit_winit` 0.34.2, `accesskit` 0.25.2, `accesskit_unix` 0.24.1, `accesskit_atspi_common` 0.21.1, with `atspi` still 0.29.0 and `zbus` still 5.19.0. That isolated graph compiled. It was not merged: it is a framework-level eframe/egui/winit upgrade, and the candidate did not establish successful runtime AT-SPI registration or complete GUI regression compatibility.

The installed `atspi-proxies` 0.13.0 source declares both `IsEnabled` and `ScreenReaderEnabled` properties on `org.a11y.Status`. The locked `accesskit_unix` 0.21.1 `context.rs` subscribes specifically to `receive_screen_reader_enabled_changed()`. The candidate 0.24.1 `context.rs` subscribes to `receive_is_enabled_changed()`. These are version-specific activation choices; the properties are not used for separate purposes by the old backend.

### Adapter activation, D-Bus addresses and registration

Both versions first establish a connection to the normal session bus and create a `StatusProxy` for service `org.a11y.Bus`, object `/org/a11y/bus`, interface `org.a11y.Status`. The source subscribes to property-change events; it does not first call `Get` to read the current enabled value. Consequently, an already-true property at subscription time does not by itself prove that the event loop has observed an activation transition.

On an enabled property-change event, AccessKit calls `org.a11y.Bus.GetAddress` on the same service/object, connects a second `zbus::Connection` to the returned private AT-SPI bus address (or `AT_SPI_BUS_ADDRESS` if explicitly set), creates its application root object at `/org/a11y/atspi/accessible/root`, and calls `org.a11y.atspi.Socket.Embed` on the registry service `org.a11y.atspi.Registry` at that root path. This `Embed` handshake is the application registration step; the app's AT-SPI objects are exported on the private AT-SPI bus, not directly on the ordinary session bus. The registry returns its desktop/root reference and sets the application's `org.a11y.atspi.Application.Id` as part of the handshake. AccessKit then invokes the adapter activation handler to request the initial tree; if it returns no tree, the adapter remains pending until an update supplies one.

Error handling differs. In 0.21.1, the event loop is awaited with `.unwrap()`, so an error after the session-bus connection can panic the accessibility worker; the session-bus builder failures are silently skipped. In 0.24.1, recoverable errors from AT-SPI bus creation/operations are mapped to `None`/default values and ignored; only `InterfaceExists` and `MissingParameter` are classified as unrecoverable and panic the worker. The newer source does not implement a timer-based retry: another enabled-property change is needed to attempt `Bus::new` again after a recoverable failure. Neither inspected version waits for an additional event-registration signal before adapter activation; the newer source's gate is the `IsEnabled` property-change event followed by successful `Embed` registration.

### Live D-Bus evidence and remaining blocker

Read-only introspection of the current session confirms `org.a11y.Bus` at `/org/a11y/bus`, `org.a11y.Status.IsEnabled=true`, and no `ScreenReaderEnabled` property. `GetAddress` returns `unix:path=/run/user/1000/at-spi/bus_0`. `gdbus` and `busctl --address` attempts to connect directly to that address both fail with `ECONNREFUSED`, including a recorded `strace` `connect()` result. `org.a11y.atspi.Registry` is not activatable/visible from the ordinary session bus, and the private registry cannot be introspected while the advertised socket rejects connections. The registry process is present, but no TH80 application registration was observable; the private registry bus could not be queried while its advertised socket refused connections. The earlier documented `IsEnabled=true` set attempt recorded the original value as true and restored/verified true; no `ScreenReaderEnabled` override was used. The host's AT-SPI launcher also has repeated core-dump records during this boot, although the exact cause of the socket refusal was not isolated.

This is now two independently supported compatibility conditions: (1) the locked AccessKit 0.21.1 watches a property absent from the live status interface; (2) the advertised private AT-SPI socket rejects direct connections, so even a backend listening to `IsEnabled` cannot complete its private-bus `Embed` registration in this session. The candidate 0.24.1 corrects the first property selection but does not resolve the second host transport failure. No dependency upgrade is justified until a compatible, reachable AT-SPI bus and runtime GUI test are available.

**Phase 14.1.2 accessibility result:** AccessKit D-Bus activation `BLOCKED`; AT-SPI registration `BLOCKED`; live accessibility-tree inspection `BLOCKED`; actual screen-reader verification `BLOCKED`. The automated egui AccessKit update test for long option values passes, but is not a live AT-SPI or screen-reader test. Orca was not installed; no accessibility certification is claimed.

## Phase 14.1.3 continuation — runtime and keyboard evidence

### Isolated AT-SPI registration evidence

A disposable D-Bus/AT-SPI environment previously succeeded with the eframe 0.37.0 candidate. The isolated session used D-Bus address unix:path=/tmp/dbus-uQFYSneIqS,guid=64fb028bd122822a92d50a236ae83316 and private AT-SPI socket /tmp/th80-phase14-1.3-atspi-isolated/runtime/at-spi/bus_0. Candidate process PID 1607928 appeared on the private bus as :1.6876; /org/a11y/atspi/accessible/root was reachable and its tree was traversable. The tree exposed named controls including Layout, Advanced, Config, Local profile selector, Keymap, Layers, Selected key, Key code, Action, Layer, Lighting, Keyboard options and Macros. AT-SPI Action.DoAction was used to activate Layout, Advanced, Lighting and Keyboard options.

This is positive evidence that the 0.37 candidate can register and expose an AccessKit tree in that isolated runtime. It is not evidence that the main eframe 0.35 binary registers, and it is not screen-reader evidence. The original isolated D-Bus process/socket has since exited. A fresh disposable-session retry could create a private socket, but the app-side connection attempt logged “A connection to the bus can't be made”; neither the current 0.35 binary nor the retry 0.37 binary produced a new application registration. The host session continues to advertise /run/user/1000/at-spi/bus_0, which returns ECONNREFUSED. No host accessibility setting was changed.

The main dependency baseline remains eframe 0.35.0 / accesskit_unix 0.21.1 / accesskit_atspi_common 0.18.1. The isolated candidate graph was eframe 0.37.0 / accesskit_unix 0.24.1 / accesskit_atspi_common 0.21.1. The candidate was not merged: its full tests, strict Clippy, docs, pointer/keyboard acceptance and complete runtime revalidation were not established. The historical 0.37 registration success is retained as evidence, not treated as a release decision.

The main app's explicit root label/description call compiles and is covered by the current source build, but its actual root properties were not independently verified at runtime because the current 0.35 app did not register on a reachable private AT-SPI bus.

### Screen-reader status

Orca and Accerciser are not installed in the current environment. No speech output or actual screen-reader announcement was captured. Launch announcement, control announcements, value announcements, dialog announcements, validation/error announcements and focus restoration therefore remain `BLOCKED` for screen-reader verification. No accessibility certification is claimed.

### Native keyboard evidence

Current-source eframe 0.35 X11 interaction verified keyboard activation of top-level navigation, entry into Advanced and Lighting, profile-library navigation, Escape dismissal of the profile-open message, and layer selection from Base to Function using the layer selector. The test screenshots are phase14-1.3-x11-advanced-via-keyboard.png, phase14-1.3-x11-lighting-via-keyboard.png, phase14-1.3-x11-layer-dropdown-open.png, phase14-1.3-x11-layer-escape.png and the phase14-1.3-x11-gtk3-* profile/dialog captures.

This does not complete the required full Tab/Shift+Tab/Enter/Space/arrow/Escape matrix across all profile operations, all macro/lighting/options controls, file choosers, validation, unsaved-change confirmation, save conflict and error dialogs. The first post-dismissal Tab sequence also required another focus traversal event before the next top-level action was activated. The egui regression test modal_closure_restores_the_previous_keyboard_focus passes, but native focus restoration has not been verified for every dialog class. Full keyboard navigation and native dialog focus restoration remain `PARTIAL`.

### Long-content boundary

The test long_option_values_remain_available_to_accessibility_and_inspection still proves that a synthetic long JSON option value is retained in the AccessKit value/description and tooltip. It does not cover long profile descriptions, long profile/macro names, long macro actions, large extension objects, long validation messages, narrow/wide-window caret navigation or copy/paste. Long-content accessibility remains `PARTIAL`; the automated option-value result is not a live AT-SPI or screen-reader result.


## Phase 14.1.4 continuation — runtime and interaction assessment

### Keyboard-only status

The new X11 run verified a pointer-driven unsaved-change modal and its Cancel action, a native save error and Escape dismissal, and the profile chooser's selection/open flow. The unsaved modal was opened by changing the local-profile selector while a draft was dirty. Cancel returned to Profile details with the edited description still visible; the saved repository hash was unchanged. This proves the cancellation transition but does not prove that all modal controls are reachable/activatable with keyboard alone. Confirmed Discard and focus restoration after that action were not established.

The screen-by-screen keyboard matrix remains incomplete. Existing automated tests cover navigation Tab/Shift+Tab, Enter/Space activation, text input without pointer events and modal focus restoration. Existing native X11 captures cover selected navigation and layer controls. They do not collectively prove all controls and dialog workflows. Profile details, profile library, keymap/layer management, macros, lighting, options, validation, file chooser, destructive confirmations, unsaved-change dialog and save-error dialog still need systematic keyboard evidence. Full keyboard-only navigation remains PARTIAL.

### Dialog focus

The save error was dismissed with Escape and the app returned to Profile details; the unsaved-change Cancel action returned to the edited profile screen. These are useful native observations. They do not establish focus restoration for every chooser, validation, deletion, unsaved-change and save-error dialog. The automated `modal_closure_restores_the_previous_keyboard_focus` regression remains positive, but native dialog focus restoration remains PARTIAL.

### Long-content boundary

The existing stress screenshot set from Phase 14.1.1 remains preserved (profile details, macro list, options and narrow/wide captures). The current automated `long_option_values_remain_available_to_accessibility_and_inspection` test verifies the complete serialized JSON string in the AccessKit node value and description. This Phase 14.1.4 continuation did not re-run a complete current-source live inspection of long profile names/descriptions, macro names/actions, nested extension objects and long validation errors at narrow/wide sizes. Caret/copy/paste and full AccessKit descriptions for those other content classes remain unverified. Long-content accessibility remains PARTIAL.

### AT-SPI and screen reader

A fresh read-only call during this continuation still returned `unix:path=/run/user/1000/at-spi/bus_0` from `org.a11y.Bus.GetAddress`. Orca and Accerciser are still unavailable. No new successful current eframe 0.35 registration or actual screen-reader speech was obtained. The previously recorded direct connection refusal (`ECONNREFUSED`), old `ScreenReaderEnabled` versus newer `IsEnabled` property mismatch, and failure of removing `NO_AT_BRIDGE` alone remain the applicable evidence. The historical eframe 0.37 isolated tree remains historical candidate evidence only. AccessKit integration is present in the build; current-baseline AT-SPI registration remains BLOCKED, current-baseline tree inspection remains PARTIAL on the strength of historical candidate evidence, and actual screen-reader interaction remains BLOCKED. No certification is claimed.

## Phase 14.1.5 final accessibility disposition (2026-11-02)

### Keyboard-only results

Real GTK3 X11 input expanded the evidence beyond the earlier Tab samples. The retained 24-step Profile details sequence traversed the profile selector, top-level and sub-navigation controls, name/description/tags fields and associated-model control, with reverse Shift+Tab samples. Enter activated top-level navigation; Escape dismissed the save-error message and closed tested dropdown/modal states. Keyboard navigation reached Layout, Advanced/Macros and Config/Keyboard options; existing X11 evidence includes Base/Function layer switching. The Macro editor exposed its 240-step list and long name field. These are native input observations, not only egui simulation.

| Screen | Native keyboard evidence | Status |
|---|---|---|
| Dashboard / profile library | Prior 12-step Tab/Shift+Tab sample and navigation evidence | PARTIAL |
| Profile metadata | 24-step Tab sequence; name, description, tags and associated-model focus reached | PARTIAL |
| Keymap | Layout reached by keyboard; layer navigation evidence exists | PARTIAL |
| Layers | Base/Function keyboard selection evidence | PARTIAL |
| Macros | Advanced/Macros navigation; long name editor and 240-step list inspected | PARTIAL |
| Lighting | Keyboard navigation reached Lighting; controls not exhaustively traversed | PARTIAL |
| Keyboard options | Options page reached; long JSON value copy/paste and caret selection tested | PARTIAL |
| Validation and dialogs | Unsaved modal focus entry/activation and save-error Escape dismissal; remaining dialog paths incomplete | PARTIAL |

The full requirement remains **PARTIAL**. There is no complete native record of every meaningful control on every screen, nor every dialog's Tab/Shift+Tab/Enter/Space/arrow/Escape behavior. Focus after deletion, validation, chooser cancellation, save conflict and every modal close is not exhaustively established. Automated egui tests for Tab/Shift+Tab, Enter/Space, text editing and modal focus restoration remain positive, but simulated input is not substituted for native coverage.

### Dialog focus restoration

The unsaved-change modal showed focus entry on **Save and open** and then **Discard and open**; confirmed discard opened the target profile. The earlier save-error path was dismissed with Escape and returned to Profile details. The egui `modal_closure_restores_the_previous_keyboard_focus` regression passes. Native focus identity after dismissal is not recorded for every dialog class, especially GTK chooser cancellation, profile/macro deletion, validation and save-conflict recovery. Native dialog focus restoration is **PARTIAL**.

### Long-content values

The live X11 Keyboard options editor exposed a 4,558-byte serialized JSON value. Ctrl+A/C copied the full value; Home→Shift+End and End→Shift+Home each selected the full line; pasting the same value and copying it back was byte-identical. A 350-byte macro name was also copied byte-identically and selected in both caret directions. The 240-step macro list is rendered and scrollable. The AccessKit regression asserts the complete option value in the node value and description, and the existing tooltip contains the complete editor string.

This does not establish complete live inspection of long profile descriptions, all profile/macro labels, validation messages or nested extension objects through AT-SPI. The legacy GTK3 stress window clips very long selector/layer labels at narrow width; the current-source compact profile-selector helper and its 926-pixel navigation-bounds test pass, but no current-baseline live AT-SPI description can be inspected. Long-content accessibility/usability remains **PARTIAL**, not a serialization defect.

### AT-SPI final targeted check

The session bus still reports `org.a11y.Status.IsEnabled=true` and exposes no `ScreenReaderEnabled` property. `org.a11y.Bus.GetAddress` still returns `/run/user/1000/at-spi/bus_0`; a direct connection to that advertised private socket again returns `ECONNREFUSED`. Multiple registry processes being present does not make the advertised socket usable. Orca is not installed, and no working registry/screen-reader client or new live TH80 registration was found. No repeated property mutation experiment was performed.

Final distinctions:

| Accessibility layer | Disposition | Evidence |
|---|---|---|
| AccessKit integration | PASS | `eframe` 0.35.0 has AccessKit enabled; the GUI emits AccessKit updates and 36 GUI tests pass, including all-screen accessible names and complete long-option value/description assertions. |
| Runtime tree publication | PARTIAL | Application-side AccessKit generation is tested, but current eframe 0.35 does not publish a reachable live tree in this session. Historical isolated eframe 0.37 tree evidence is not current-baseline proof. |
| AT-SPI registration | BLOCKED | Advertised private socket returns `ECONNREFUSED`; locked `accesskit_unix` 0.21.1 waits for `ScreenReaderEnabled`, while the session exports only `IsEnabled`. |
| Accessibility inspection | PARTIAL | Automated AccessKit tree updates and value assertions exist; no live current-baseline AT-SPI application tree is available. |
| Actual screen-reader interaction | BLOCKED | Orca unavailable; no spoken output, focus announcement or screen-reader navigation was tested. |

Keep eframe 0.35.0 and accesskit_unix 0.21.1. Do not upgrade to the isolated 0.37 candidate without demonstrated runtime benefit and full regression acceptance. Do not claim screen-reader compatibility or accessibility certification.

## Phase 14.1.6 — Keyboard operation and AccessKit naming addendum (2026-11-02)

### Keymap-specific keyboard semantics

The Layout renderer uses actual egui `Button` widgets for each profile-defined logical position, `ComboBox` for active layer and target layer/macro selection, and `selectable_value` widgets for the eight TH80-core action categories. The action editor supports Key, Modifier, Combination, Layer switch, Momentary layer, Macro, Transparent and Unassigned. There is no keyboard geometry or inferred physical-position map. Position selection is therefore available through the normal Tab/Enter focus model, but a complete native sequence covering layer switch → position select → category select → edit/apply → another position → Unassigned/clear was not captured.

The previous/next layer controls were rendered as glyph-only buttons, and position selection/remove controls had no explicit descriptive accessible label. The source now labels these as `Previous logical layer`, `Next logical layer`, `Select logical position {id}: {action}`, and `Remove selected logical position`. Regression test `keymap_positions_are_named_and_selectable_with_tab_and_enter` verifies the AccessKit names and simulated Tab/Enter position selection. This is a narrow, justified semantics improvement; it does not add speculative shortcuts or assert that every custom widget is fully accessible.

### Keyboard inventory coverage and native limits

The source inventory in the GUI interaction report covers all eight actual pages, shared navigation, all present profile/keymap/layer/macro/lighting/options/settings controls, egui modals and native file pickers. egui's standard buttons, text edits, checkboxes, sliders, drag values, ComboBoxes and selectable values participate in keyboard focus; the keymap logical positions are standard buttons. However, actual X11 input in this phase established Tab/Shift+Tab and Enter samples and navigation to the Profile library, not a complete native keyboard-only workflow for each control. Space, arrow navigation of every ComboBox/list, keyboard popup color selection, long-list traversal, every destructive action and each nested modal remain unverified. Focus visibility was observed on earlier X11 captures, but not independently measured for every control.

The updated release was launched as isolated X11 PID `2164192`, using only `/tmp/th80-phase14-1.6-x11/profiles`; existing PIDs `359824`, `1888578`, `1971100`, `2042117`, and `2113082` were preserved. Keyboard input navigated to Profile library. This fresh process did not produce a complete keyboard-only profile-open/keymap-edit sequence. Screenshots are retained in `.phase14-1-evidence/phase14-1.6/`. The native keyboard result remains **PARTIAL**; simulated egui test success is not substituted for native proof.

### Dialog focus and native file chooser

The application-level dialog classes are all egui `Modal` overlays: unsaved-profile switch, profile deletion, macro deletion, and status/error/information (including validation, save failure, stale-save conflict and chooser-result errors). `sync_modal_focus` captures the focused egui ID when the first modal becomes active and requests that ID when the last modal closes. Existing tests prove macro-delete Escape cancellation and restoration of the prior egui focus ID. Real X11 evidence proves selected unsaved-dialog buttons and save-error Escape dismissal in prior captures. It does not prove exact native focus return for all deletion, validation, save conflict, or chooser cases, nor that each modal initially places focus on its first action.

Import/export use `rfd 0.17.2` `AsyncFileDialog`; completion/cancel is sent from a worker through mpsc and applied/polled on the UI thread with a 100 ms repaint cadence while pending. Linux XDG/GTK portal pickers are separate native windows. Although rfd supports optional `set_parent` for XDG, TH80's current builder does not pass an explicit parent. Existing native portal cancel and successful import/export evidence plus async state tests are retained; a full chooser keyboard traversal and originating-control focus restoration were not observed. No architecture change was made without a reproduced defect. Dialog focus restoration remains **PARTIAL**.

### Long content

The full 4,558-byte option value and 350-byte macro name copy/caret-selection evidence remains retained; the 240-step list is scrollable. The new change does not affect text serialization. Long descriptions, nested extension values, validation messages, per-control tooltip exposure and all AccessKit value descriptions are not completely inspected. The full long-content acceptance dimension remains **PARTIAL**.

### Runtime/platform boundary

No new working AT-SPI application registration, private bus connection, Orca installation or screen-reader interaction was obtained. Keep AT-SPI registration and actual screen-reader interaction **BLOCKED**, runtime tree inspection **PARTIAL**, and do not repeat D-Bus property mutation experiments. Keep `eframe 0.35.0` and `accesskit_unix 0.21.1`; do not merge the isolated eframe 0.37 candidate. Windows and macOS native execution remain **BLOCKED** pending native runners.
