# Phase 14.2 — Entropy Visual Comparison

**Inspection date:** 2026-11-02  
**Assessment:** Recognizably Entropy-inspired structure; not pixel-equivalent. A controlled same-client-viewport capture was achieved for both applications, but Entropy v0.5.0 stopped at its no-device screen. Upstream source inspection found no supported device-free editor mode; equivalent interactive Entropy states remain BLOCKED (Outcome B).

## Evidence and dimensions

| Reference | Dimensions | TH80 Studio evidence | Dimensions / UI scale |
|---|---:|---|---:|
| Entropy Layout | 2798 × 1606 (1.742:1) | `layout-wayland.png` / `layout-light.png` / `layout-dark.png` | 926 × 492 (1.882:1), visible scale 100% |
| Entropy key picker light | 2798 × 1606 (1.742:1) | `key-picker-light.png` | 926 × 1012 (0.915:1), visible scale 100% |
| Entropy key picker dark | 2798 × 1606 (1.742:1) | `key-picker-dark.png` | 926 × 1012 (0.915:1), visible scale 100% |
| Entropy Matrix Tester (reference only) | 2798 × 1606 | No TH80 equivalent | Intentionally not implemented |

Other TH80 screen captures (`layer-selection.png`, `advanced-macros.png`, `config-lighting.png`, `config-keyboard-options.png`, `application-settings.png`, `dialog-macro-delete-confirmation.png`) are 926 × 1012. Native Wayland captures include `wayland-picker-search.png`, `wayland-picker-cancel-focus-restore.png`, `wayland-keyboard-focus.png`, `wayland-keyboard-enter-select.png`, `wayland-layer-arrow-key.png`, `wayland-layer-wheel.png`, and `wayland-keyboard-inspector-focus.png`.

Additional isolated X11 captures from the current Phase 14.2 build are retained as `layout-x11-acceptance.png`, `x11-advanced-navigation.png`, `x11-config-navigation.png`, `x11-action-picker-open.png`, `x11-picker-search.png`, and `x11-picker-cancel.png`. Each is 926 × 1012 pixels; the app's visible scale control reads 100%. The X11 run used a copied synthetic profile under `/tmp/th80-phase14-2-acceptance/profiles` and a separate settings file. No real profile directory was opened.

### Comparison methodology and limits

- Inspected the actual Entropy reference images and retained TH80 Wayland images, then compared their visible composition, hierarchy, controls and states. Image dimensions were verified with ImageMagick `identify`.
- Entropy references are 2798 × 1606. Existing TH80 Wayland captures are 926 × 492 for the compact Layout capture and 926 × 1012 for most tall captures. New X11 captures are 926 × 1012. The current X11 desktop reports 1920 × 1080, but its tiling layout assigned the isolated test window a 926 × 1012 client area.
- Both applications visibly report 100% scale in their respective screenshots. This does **not** establish equivalent physical-pixel/UI-point scale: the Entropy screenshot's source DPI/device-pixel ratio is unknown, and the source viewport differs substantially. No resizing, upscaling, cropping or padding is represented as a native matched-size capture.
- A genuinely matched comparison requires a live Entropy build or a reference capture made at the same client viewport and known display scale. Neither is available in this workspace. Therefore this priority remains PARTIAL rather than PASS.

## Structured comparison

| Area | Entropy reference | TH80 Studio | Assessment / remaining difference |
|---|---|---|---|
| Primary navigation | Compact centered Layout / Advanced / Config tabs | Same three primary sections, with local profile selector and profile action adjacent | Strong structural match; TH80 shell is more profile-centric and less centered |
| Header | Undo left, scale right, no device/profile context in the shown reference | Local profile, local save state, Files, Save and scale controls | Appropriate offline adaptation; denser and more controls than Entropy |
| Layout hierarchy | Large layer name with arrows, central keyboard, quiet footer | Layer arrows/name, logical-position canvas, inspector, contextual hints/footer | Similar editing hierarchy; TH80 canvas has an additional inspector and logical-ID label |
| Key styling | Uniform light keycaps, thin outlines, compact labels | Uniform key-like buttons with ID and assignment legend, selected accent state | Related key-centric language; current keys are wider/taller and less tightly packed |
| Geometry | Device-specific split physical geometry | Wrapped logical-position controls | Intentionally different; physical TH80 mapping is unverified and must not be invented |
| Layer interaction | Arrows, wheel, Left/Right, click-to-rename | Arrows, wheel over board, Left/Right, click-to-rename | Strong interaction-pattern match; TH80 uses its profile layer list |
| Picker structure | Large centered modal, title, hint, category row, key/action surface | Large centered modal, search, five categories, action list/suggestions and details editor | Strong modal hierarchy match; TH80 uses profile-derived suggestions and model editing instead of a physical key matrix |
| Picker taxonomy | Basic/Symbols/Mods/Special/Macros/Tap Dance/Custom | Keys/Modifiers/Layers/Macros/Other | Correct adaptation to the eight TH80-core variants; no unsupported firmware catalog |
| Light/dark | Same geometry and hierarchy with contrasting surfaces | Same modal/page layout with egui light/dark visuals and accent | Consistent theme pair visible in captures; detailed pixel-level contrast audit not performed |
| Advanced | Device/firmware-specific advanced sections | Existing macro editor | Intentionally scoped to supported TH80 model; no unsupported Entropy features |
| Config | Configuration tabs and application settings | Lighting, Keyboard options, Profile library/details, App settings | Functional grouping retained; TH80 profile editor pages are more form-oriented |
| Footer/theme | Context footer; theme at lower-right | Save state and contextual hint footer; theme toggle lower-right | Similar placement concept; footer carries more local persistence context |

### Measurable composition observations

- The Entropy Layout reference is 2798 × 1606 (aspect 1.742); the original Wayland TH80 Layout capture is 926 × 492 (aspect 1.882). These are not viewport-matched. The new X11 Layout view is 926 × 1012 (aspect 0.915), with profile, section tabs and scale/save controls sharing a single top row.
- In the 926 × 1012 X11 Layout capture, the logical-position workspace occupies approximately x=8–614 (606 px) and the selected-position inspector x=635–917 (282 px), giving the inspector about 32% of content width. Nine logical positions wrap into six controls on the first row and three on the second. This is an explicit profile-defined collection, not an inferred physical matrix.
- Entropy places the three primary sections near the horizontal center of the top edge and layer title/navigation near the upper center. TH80 places profile and file/save controls in the same top band as left-of-center section tabs, and layer controls at the left of the workspace. This is a material shell-composition difference, not just a color difference.
- The Entropy light picker is a full-width keyboard catalog with seven categories and a large key-grid surface. TH80's picker uses five categories, a searchable model-driven action list and a separate details/editor column. This is a deliberate data-model adaptation; reproducing Entropy's physical key grid would misrepresent the available TH80 profile model.
- At 926 × 1012, the TH80 picker dialog spans approximately x=40–886 (846 px) and y=150–862 (712 px), about 91% of viewport width and 70% of viewport height. Its category row, action selection, details editor and Apply/Cancel controls remain visible. The Entropy screenshot's modal is approximately x=232–2568 and y=92–1460 in its 2798 × 1606 source, so absolute pixel dimensions are not comparable.
- In the X11 search capture, entering `macro` reduces the category row to `Macros · 1` and hides unrelated action suggestions while preserving the current action preview and detail editor. Escape closes the modal; the post-cancel capture still shows the selected position's `KEY_Q` assignment.
- The dark and light TH80 picker captures retain the same modal hierarchy and category/action structure. The Entropy references also include both themes, but there is no common native viewport/scale pair for pixel-level contrast or typography comparison.

## Visual findings

1. The app now presents a clear keyboard-configuration workspace rather than a dashboard-first landing page. The three primary sections and layer-centric editing loop are recognizable Entropy concepts.
2. The picker has the clearest visual resemblance: a large centered modal, compact title/instructions, centered category row, two-column action/detail composition, selected-action indication and consistent light/dark surfaces.
3. The Layout screen is intentionally not a physical keyboard drawing. Its wrapped logical-position buttons are less visually rich and less spatially informative than Entropy's hardware-specific key layout. This is a model-evidence constraint, not a missing geometry implementation.
4. The inspector is a TH80-specific addition that makes the selected opaque logical position and action parameters explicit.
5. The shell has more profile and file controls than the Entropy reference capture. That is necessary to preserve TH80's offline profile workflow and explicit Save semantics.
6. Existing captures show the main content at smaller viewports than the Entropy reference. A matched-size native TH80/Entropy capture set is still required before making reliable claims about relative control density, panel sizing and typography at the reference scale.

## Acceptance judgment

The result is a substantial structural/interaction adaptation rather than a palette-only refresh. It is **not** a pixel-faithful clone and should not be described as visually equivalent. No cosmetic source changes were made in this acceptance pass: current screenshots do not provide a fair same-viewport basis for deciding whether changing panel proportions, header spacing or typography would improve fidelity. The next visual step is to capture Entropy and TH80 at a shared client viewport and known display scale, then make only evidence-backed refinements.

The Matrix Tester remains reference material only and has no TH80 implementation.

## Controlled matched-viewport attempt — 2026-11-02 (supersedes earlier viewport-only limitation)

### Capture setup

- A separate headless Weston compositor was used, with XWayland display `:1`, output `1800 × 1033`, scale `1`, pixman renderer and a synthetic/fake seat. This did not resize or take over the user's active desktop, browser windows, or existing TH80 process.
- TH80 was launched as a separate process with a copied synthetic profile directory and separate settings file under `/tmp/th80-phase14-2-controlled/`. Its actual X11 client window was `1762 × 974` pixels (window geometry `X=38`, `Y=59`); app scale was `100%` (`ui_scale: 1.0`).
- Entropy v0.5.0 was extracted from the locally available AppImage under `/tmp` and launched in a bubblewrap sandbox on the same XWayland display. Its client window was set to `1762 × 974` pixels. Its log explicitly says `Guessed window scale factor: 1`; this is a runtime guess, not proof of the original reference screenshot's DPI/device-pixel ratio.
- Both new captures are native screenshots from the same XWayland display and compositor scale, not resized, cropped, padded or upscaled images. PNG metadata reports `1762 × 974`; its physical pixel density is not embedded in the PNG (`Units: Undefined`).
- TH80's own app scale is directly visible as `100%`; Entropy's guessed scale factor is `1`. Both therefore used nominal scale 1 in this controlled run, but this does not retroactively establish the source scale of the historical `2798 × 1606` Entropy references.

### Result: Entropy has no usable keyboard state without a device

`entropy-v0.5.0-no-device-1762x974.png` is a real, same-viewport capture of Entropy, but the entire app view is the startup message **“Waiting for a device — Connect a Vial device.”** Its log repeatedly reports `Found 0 Vial device(s)`. The sandbox deliberately has no USB/HID device access and no physical keyboard was connected or queried. No synthetic/fabricated Vial device was introduced.

Consequently, same-viewport live captures for Entropy Layout, selected-position inspector, layer navigation, and light/dark action picker cannot be produced under the user's hardware-isolation constraints. The Entropy startup screen is not a substitute for those states. The earlier 2798 × 1606 screenshots remain useful historical reference material, but their original DPI/device-pixel ratio is unknown and they cannot be claimed as equivalent-scale captures.

### TH80 state captures in the same viewport

All listed captures are `1762 × 974` pixels, captured from the isolated X11 TH80 instance on the same `:1` display:

| State | Evidence | Verified setting/state |
|---|---|---|
| Layout workspace | `controlled-x11/th80-layout-1762x974.png` | Dark theme, 100%, synthetic profile with profile-defined logical positions |
| Selected-position inspector | `controlled-x11/th80-selected-inspector-1762x974.png` | Dark theme, 100% |
| Layer navigation | `controlled-x11/th80-layer-navigation-1762x974.png` | Second layer selected; layer count visible |
| Light action picker | `controlled-x11/th80-action-picker-open-light-1762x974.png` | Light theme |
| Dark action picker | `controlled-x11/th80-action-picker-open-dark-1762x974.png` | Dark theme |
| Scale interaction | `controlled-x11/th80-scale-105-1762x974.png` | 105% shown; settings were subsequently restored to `1.0` / 100% |
| Light theme | `controlled-x11/th80-theme-light-1762x974.png` | Light theme; settings subsequently restored to dark for test teardown |

The new viewport allows repeatable TH80 measurements and direct visual inspection at a practical desktop-sized viewport. It does **not** permit fair visual measurements against Entropy's unavailable interactive states. Do not compare the blank/no-device Entropy view's panel dimensions or content density with TH80's editor.

### Measurements and visual judgment

- Viewport/client bounds: both windows `1762 × 974` pixels, same XWayland display, compositor output `1800 × 1033`, compositor scale `1`.
- TH80 Layout capture: the profile-defined logical-position workspace and selected-position inspector are both visible at once. This is a logical collection of profile-defined positions, not a physical switch matrix; no attempt was made to make it resemble Entropy's device-specific physical geometry.
- The header retains profile, section navigation, saved state, Files/Save and scale controls. These controls reflect TH80's local/offline profile workflow; no evidence in the no-device Entropy state justifies changing their dimensions or alignment.
- The picker retains its TH80-specific searchable action taxonomy and editor/details composition. Since Entropy's picker cannot be opened without a Vial device, picker density, category spacing, typography and modal proportions cannot be fairly compared at this viewport.
- No visual source refinement is justified by this evidence. The matched viewport has removed the raw pixel-size mismatch for the attempted live run, but the missing Entropy application state—not a screenshot manipulation issue—is the remaining blocker.

**Updated acceptance:** controlled equal-pixel viewport capture = PASS; same compositor/display scale for the live attempt = PASS; equivalent Entropy/TH80 interactive state comparison = BLOCKED; evidence-backed visual refinement decision = NOT TESTED (no change warranted by the available comparison evidence). Phase 14.2 remains unaccepted.

## Final Entropy device-free reference investigation — 2026-11-02

### Upstream revision and inspection scope

- Upstream: `https://github.com/ergohaven/entropy`, inspected from a shallow source checkout at commit `336a5dcb0ad587caab938b32e4700568ecd21ba8` (the `main` branch revision obtained during this investigation).
- Official documentation: `https://docs.eh.industries/software/entropy/`, including Quick start and navigation, Installation, and the project README published at the repository root.
- Source files inspected include `src/main.rs`, `src/app_init.rs`, `src/device.rs`, `src/ui/layout_view.rs`, `src/headless_export.rs`, `tests/fixtures/display-menu-catalog.README.md`, `tests/fixtures/k04-vial-b4709ab9.json`, and the `src/ui/*` test/preview entry points.

### Supported-mode findings

| Candidate | Upstream finding | Acceptance consequence |
|---|---|---|
| Demo / offline preview / UI-only launch flag | No documented mode, CLI option or environment variable was found. `src/main.rs` launches the normal `EntropyApp::new` GUI; the documented CLI features include minimized launch and headless layout export, not a demo editor. | Not supported; do not invent a flag. |
| Built-in sample keyboard configuration | `tests/fixtures/k04-vial-b4709ab9.json` is a test fixture, not a production demo profile or documented sample-import flow. | Useful as source/test evidence only; it does not make the GUI editor available without a device. |
| Development/test fixture | The source contains test-only constructors and many egui tests using fixture state. `new_inert_for_test` is compiled only under `cfg(test)`; fixtures and mocked HID recorders are test infrastructure. | Not a supported standalone executable entry point. Reusing those internals would require an upstream test harness or a custom source modification, neither of which qualifies as an official device-free mode. |
| Headless mode | `EntropyApp::new_headless` is documented in source as the no-window `--export-layout` path. It creates an empty `DeviceManager`, sets `headless = true`, and is not an editor renderer. Export itself requires a connected keyboard snapshot. | Does not render the editor and does not supply an offline layout preview. |
| Browser preview | Upstream README explicitly lists browser-only configuration as out of scope. | No official browser editor reference exists. |

### Source-level execution path

1. The normal native window is created by `eframe::run_native` in `src/main.rs`; its app-creation closure calls `EntropyApp::new` (`src/main.rs`, around lines 365 and 486).
2. `EntropyApp::new` in `src/app_init.rs` initializes the normal application with `DeviceManager::new()` (around lines 15–53).
3. `DeviceManager::new()` in `src/device.rs` calls `manager.scan()` on non-macOS platforms (around lines 518–524). The scan path initializes `hidapi::HidApi` and iterates the actual HID device list (`src/device.rs`, around lines 542–560). macOS defers the initial scan for run-loop safety, but that is not a device-free editor mode.
4. The real Layout renderer in `src/ui/layout_view.rs` accepts a `KeyboardLayout` reference; it is not a self-contained sample layout page. The application fixture data is used by tests and does not populate the production `EntropyApp::new` path.
5. The headless constructor in `src/app_init.rs` is explicitly for `--export-layout`, uses `DeviceManager::empty()`, and marks the app headless (around lines 55–74). `src/headless_export.rs` describes export as connecting to a keyboard and writing its layout; the README documents the same device-dependent operation.

### Exact checks and result

- Inspected the current upstream README, official Entropy docs and the source checkout above; searched startup/CLI code for demo, preview and test entry points; inspected fixture documentation and the K:04 JSON fixture; traced normal app initialization to HID enumeration and separately traced the headless export constructor.
- No documented demo, offline preview, built-in sample-editor, UI-only, or browser editor mode was found.
- No production application entry point that renders the actual Layout editor with a fixture while bypassing device discovery was found. Test-only egui construction and unrelated `no HID` display-preview fixtures do not provide the complete Entropy editor/picker.
- No attempt was made to inject data, alter Entropy runtime behavior, add a virtual HID device, spoof a keyboard, or bypass discovery. No physical keyboard was accessed.
- The previously captured `entropy-v0.5.0-no-device-1762x974.png` remains the only new live Entropy capture. The earlier upstream/retained Entropy screenshots remain useful screenshot references, and the inspected source/docs provide architectural and interaction-pattern context. Neither is an equivalent-state live comparison.

**Final decision: Outcome B.** Under the stated no-hardware/no-spoofing constraints, equivalent interactive Entropy Layout, inspector, layer and picker states are not reachable through an officially supported device-free mode in the inspected upstream revision. A future upstream-provided demo fixture/preview, or user-provided genuine reference captures at a known scale and matching viewport, would reopen this comparison. Do not modify TH80 visual styling based solely on the no-device screen or architectural inference.

### Evidence classes (keep separate)

- **Direct live application comparison:** only the same-display, same-client-pixel viewport attempt; Entropy shows its no-device screen, so the requested editor-state comparison is blocked.
- **Screenshot-based reference:** retained Entropy screenshots and upstream documentation images can inform qualitative design reasoning, but their viewport/scale provenance must be stated and they are not a live equivalent-state comparison.
- **Source-code architectural analysis:** upstream initialization, HID discovery, headless export and test fixtures establish the supported execution paths; this is not a visual comparison.
- **Inference:** any conclusion about relative editor density, exact proportions, or state-specific colors not visible in a same-state capture remains an inference and must not drive speculative refinements.

