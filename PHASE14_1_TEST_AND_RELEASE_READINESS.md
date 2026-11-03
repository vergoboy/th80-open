# Phase 14.1 — Test and Release Readiness

## Cargo verification

Final verification was run after formatting the test-only `ScreenRenderer` alias.

| Command | Exit | Result |
|---|---:|---|
| `cargo fmt --all --check` | 0 | PASS |
| `cargo test --workspace` | 0 | PASS — 123 tests, 0 failures |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | PASS |
| `cargo build --workspace --release` | 0 | PASS |
| `cargo doc --workspace --no-deps` | 0 | PASS |

The 123-test breakdown is:

| Target | Tests |
|---|---:|
| `th80-cli` unit tests | 3 |
| CLI offline workflow integration | 6 |
| `th80-core` unit tests | 30 |
| Core schema compatibility integration | 1 |
| Core repository process integration | 5 |
| `th80-gui` tests | 31 |
| `th80-probe` library tests | 26 |
| `th80-probe` binary tests | 6 |
| Probe offline protocol integration | 12 |
| Root legacy parser tests | 3 |
| **Total** | **123** |

## Native platform evidence

| Platform/backend | Result | Scope |
|---|---|---|
| Linux Wayland | PASS | Actual `TH80 Studio` Wayland window, profile workflows, screen navigation, native chooser, keyboard input and screenshot evidence. `xwayland=false`. |
| Linux X11 via Xwayland | PASS, limited | The release GUI was launched with `WAYLAND_DISPLAY` and `WAYLAND_SOCKET` empty and `DISPLAY=:0`; Hyprland reported `xwayland=true`. Dashboard screenshot captured. Full workflow suite was not repeated under X11. |
| Windows | Not run natively | No Windows GUI or screen-reader execution in this phase. |
| macOS | Not run natively | No macOS GUI or VoiceOver execution in this phase. |

The X11 test used a separate temporary `TH80_PROFILE_DIR`; the X11 test process was terminated after capture. The Wayland test process remained isolated under the temporary import-target repository. The user's normal profile directory was not used.

## Accessibility status

AccessKit and AT-SPI dependencies compile and egui accessibility-tree tests pass. The running TH80 app did not register in the session AT-SPI registry. The exact locked backend/property mismatch and independent private-bus transport failure are resolved in the Phase 14.1.2 accessibility addendum. Orca/Accerciser were not installed; there is no actual screen-reader pass. See [the accessibility report](PHASE14_1_ACCESSIBILITY_AND_KEYBOARD_NAVIGATION.md).

## Hardware isolation

- `cargo tree -p th80-gui -e normal` contained zero matches for HIDAPI, `rusb`, `nusb`, libusb, hidraw, `th80-probe`, Vial, QMK or RMK.
- The same forbidden-dependency scan for `th80-cli` returned zero matches.
- GUI/CLI/core source scan for HID/USB device APIs returned zero matches.
- The two observed GUI test processes used temporary `TH80_PROFILE_DIR` values.
- `/proc/<pid>/fd` inspection found no `/dev/hidraw*`, `/dev/input/*`, or `/dev/bus/usb/*` file descriptors for the running Wayland GUI.
- `ldd target/release/th80-gui` showed no unresolved shared libraries.
- No physical device was enumerated or opened by the GUI, no hardware report was sent, and no keyboard state was changed.
- Lighting is local profile data only. No physical-device state is claimed.

## Repository preservation

The repository has no Git `HEAD` and no remotes; all project files are represented as untracked entries. The Phase 14.1 starting inventory at `/tmp/th80-phase14-1-files-before.txt` contained 187 files. An earlier checkpoint had 297 files (110 new evidence captures); after the X11 captures and four report files were added, the final comparison found:

- 187 baseline files.
- 305 current files.
- 0 missing baseline files.
- 118 new files: 114 under `.phase14-1-evidence/` and the four Phase 14.1 reports.
- 0 other new paths.
- 0 staged changes.
- No commits or remotes.

The original inventory remains preserved. Because the repository has no commit history and the baseline files were untracked, a Git tracked-file diff cannot establish source-level before/after changes; the inventory comparison establishes path preservation, not byte-for-byte equality. No baseline file was deleted or overwritten.

## Evidence and limitations

Evidence is retained in `.phase14-1-evidence/`. The GUI interaction report lists representative screenshots and their tested workflows.

Known gaps:

1. No real screen-reader test or live TH80 AT-SPI tree was available.
2. Full keyboard-only traversal of every control and every dialog was not completed; automated egui keyboard tests supplement the native desktop pass.
3. Profile-delete confirmation was captured and the corresponding synthetic ID was absent from the final repository, but a separate delete-cancel/reopen proof was not retained.
4. X11 was launch/screen-smoke tested only; the complete workflow pass was performed under Wayland.
5. No Windows/macOS native GUI or accessibility execution occurred.
6. No exhaustive arbitrary-size resize/long-content stress matrix was performed.
7. Undo/redo remains intentionally deferred with a concrete design in the implementation report.

## Release-readiness conclusion

The workspace is build/test clean for Linux, and the product boundary remains offline/local-only with no normal GUI HID/USB dependency. The application is suitable for continued Linux development and manual evaluation. Phase 14.1 does **not** establish formal accessibility conformance, full keyboard-only completion, native Windows/macOS behavior, or complete X11 interaction coverage. Those limitations should be resolved before making broader accessibility or cross-platform release claims.

## Acceptance-closure update (2026-11-02)

The five Cargo commands and 123-test breakdown above are the last recorded complete workspace verification. No application source or test code changed in this continuation, so no new test count is claimed. The runtime investigation and existing evidence were reviewed without restarting the Wayland process.

### Linux runtime accessibility

- AT-SPI bus and registry services are active; registry has no TH80 Studio connection.
- The session exposes `org.a11y.Status.IsEnabled=true` but not `ScreenReaderEnabled`; locked `accesskit_unix` 0.21.1 subscribes to the latter, while 0.24.1 subscribes to `IsEnabled` (see Phase 14.1.2 source comparison).
- `NO_AT_BRIDGE=1` is present in the app environment; this disables the GTK bridge but is not by itself the cause of AccessKit's separate backend not connecting.
- Toggling `IsEnabled` did not publish TH80. Orca, Accerciser and PyAT-SPI are not installed.
- Runtime tree `BLOCKED`; actual screen-reader test `BLOCKED`. The newer backend property selection does not solve the independent private AT-SPI socket `ECONNREFUSED` condition. No missing egui widget label has been established; Orca remains unavailable.

### Acceptance checklist

| Condition | Status | Evidence / limitation |
|---|---|---|
| Wayland interaction coverage | PASS | Seven screens and core profile workflows; actual Wayland window and screenshots retained. |
| X11 interaction coverage | PARTIAL | X11/Xwayland dashboard, navigation, Tab/Enter, profile create/edit and keymap captures exist; full import/export, macro, lighting, validation and save-failure matrix not run. |
| Profile deletion cancel and reopen | PARTIAL | Confirmation screenshot exists; no retained real Cancel/reopen/confirmed-delete sequence. |
| Full keyboard navigation | PARTIAL | Real Tab/Shift+Tab/Enter and profile editing plus simulated tests; not every control, dialog or chooser. |
| Runtime accessibility-tree availability | BLOCKED | Locked 0.21.1 property mismatch plus private AT-SPI socket connection refusal; see Phase 14.1.2 source/runtime resolution. |
| Screen-reader verification | BLOCKED | Orca absent; no TH80 AT-SPI tree available. |
| Dialog focus restoration | PASS (automated) / PARTIAL (runtime) | egui focus-restoration test and modal semantic tests pass; not every real dialog/chooser transition traversed. |
| Window-resizing behavior | PARTIAL | One 1000 × 660 macro-control bounds regression; no real multi-size matrix. |
| Long-content behavior | NOT_TESTED | No stress test for extreme names, large action lists, large option values or long validation messages. |
| Windows native execution | NOT_TESTED | MSVC Rust target installed; Wine 11.20 is present but no native Windows session was launched. Cross-target compile is not native execution. |
| macOS native execution | NOT_TESTED | No macOS target/runner or native GUI/VoiceOver session. |
| Hardware isolation | PASS | Existing dependency/source/fd audits; no hardware communication or real profile access. |

### Platform availability

The host is Linux x86_64/Hyprland. Installed Rust targets are `x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`, and `wasm32-unknown-unknown`. Wine 11.20 exists, but there is no Windows native desktop/runner or verified MSVC runtime execution environment. No Apple target, macOS host or VoiceOver environment is available. Windows UI Automation and macOS Accessibility/VoiceOver therefore remain unverified.

### Preservation note

The full Git status was captured before acceptance-closure work. The repository still has no `HEAD` and no remotes; all project files are untracked. No staged changes, commits, remotes, hardware changes or real profile access were introduced. Existing screenshots and reports were retained. This continuation did not perform a full content-hash comparison, so the earlier inventory/path-preservation claim must not be upgraded to byte-for-byte preservation.

### Release decision

Phase 14.1 remains **OPEN**. The confirmed blocker is the host AT-SPI status interface/runtime integration; screen-reader support cannot be certified here. The outstanding keyboard, profile-delete, full X11 and resize/long-content checks are incomplete test coverage, not proven product defects. No unsupported cross-platform or accessibility-compliance claim should be made.

### Re-run and final path inventory (2026-11-02)

All five requested commands were rerun after the report updates; each exited 0:

| Command | Exit | Result |
|---|---:|---|
| `cargo fmt --all --check` | 0 | PASS |
| `cargo test --workspace` | 0 | PASS — 123 passed, 0 failed (including 31 GUI tests) |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | PASS |
| `cargo build --workspace --release` | 0 | PASS |
| `cargo doc --workspace --no-deps` | 0 | PASS |

The final path inventory comparison against `/tmp/th80-phase14-1-files-before.txt` reports 187 baseline paths, 305 current project paths (excluding `.git` and `target`), zero missing baseline paths, and 118 additional paths. Those additions are exactly 114 evidence screenshots and the four Phase 14.1 reports. The evidence directory now contains 143 screenshots total (29 were already in the original inventory). The temporary invalid focus-capture attempt was removed; no other existing artifact was removed. `git diff --cached --quiet` exits 0; no `HEAD` or remote exists. The live Wayland GUI PID 359824 remains running with the synthetic root `/tmp/th80-phase14-1-native.LvCNyT`; no X11 GUI process remains.

## Phase 14.1.1 final native interaction status (2026-11-02)

### Runtime accessibility

A second Wayland instance was launched with `NO_AT_BRIDGE` unset and a distinct profile root. It still failed to register. The host's `at-spi2-core` is 2.62.1; `org.a11y.Status` exposes `IsEnabled` but not `ScreenReaderEnabled`, while locked `accesskit_unix` 0.21.1 waits for the latter. Phase 14.1.2 established that 0.24.1 instead listens for `IsEnabled`, but the private AT-SPI bus socket still refuses direct connections. Orca is absent; it was not installed because of its substantial dependency set. AT-SPI runtime tree and screen-reader interaction remain `BLOCKED`. The clean-env experiment demonstrates `NO_AT_BRIDGE=1` is not the sole cause.

### Native X11 interaction

An isolated X11/Xwayland app (PID 1346017) ran against `/tmp/th80-phase14-1.1-x11/profiles`. Verified real interaction includes app launch, profile creation, metadata edit/save, reopening, Keymap/Layers/Macros/Lighting/Options pages, macro creation, JSON option creation/save, profile deletion cancellation and confirmed deletion, and native import/export chooser appearance. The export dialog accepted a typed destination path but the Save action did not complete; there was no successful X11 import/export round-trip. Validation, save-failure and unsaved-change workflows were not completed in this X11 run. X11 interaction remains `PARTIAL`, not full backend parity.

### Profile deletion

The synthetic profile `profile-1793598500930-0` was saved with a recognizable name/description. SHA-256 before and after Cancel matched exactly. The same ID/name/description were verified after reopening. A second confirmation/Delete removed the JSON and returned the GUI to Dashboard. Cancellation, reopen/persistence and confirmed deletion are `PASS` for the tested native X11 flow. A background click while the modal was open dismissed it as cancellation without changing the selected page; programmatic selection mutation was not tested, so selection-change safety remains `PARTIAL`.

### Keyboard traversal

Real X11 Tab/Shift+Tab evidence now covers 12 Dashboard Tab steps and eight macro-editor Tab steps, plus reverse traversal samples. Tab reached the information dialog's Dismiss button, and Enter dismissed it. The macro editor's 240-step list scrolled to later actions at narrow size. This does not cover every control, every dialog, all arrow-key combo/list behavior, all Escape paths, or full native chooser traversal. Full keyboard-only acceptance remains `PARTIAL`.

### Resize and long content

Real X11 window sizes tested: 820 × 560, 1000 × 660, 1180 × 780 and 1600 × 900. The 820 × 560 macro editor showed a limited viewport but scrolling exposed later rows. A 247,741-byte synthetic profile with 24 layers, 960 assignments, a 240-step long-title macro, long profile name/description, eight long option values and large extension payloads loaded and rendered. Text wrapped or scrolled where appropriate; option text remains horizontally clipped in single-line editors, though its JSON value persisted. No data-loss or overlap defect was reproduced. Resize and long-content acceptance remain `PARTIAL` because not all page/dialog combinations and accessible full-value behavior were audited.

### Platform feasibility

The host is Linux x86_64/Hyprland. Installed Rust targets: `x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`, `wasm32-unknown-unknown`. Wine 11.20 and QEMU are installed, but no native Windows runner/desktop or booted Windows VM was available. A cross-target MSVC compile and Wine presence do not constitute native Windows execution. The host is Linux; no macOS host, Apple SDK, macOS runner or VoiceOver session is available. Windows and macOS native GUI execution remain `BLOCKED` by unavailable native environments.

### Final acceptance matrix

| Condition | Status | Evidence / exact limitation |
|---|---|---|
| Wayland interaction | PASS | Previously completed real Wayland workflows; original PID 359824 preserved. |
| X11 interaction | PARTIAL | Real X11 profile/macro/options/screen/chooser interaction; no complete import/export, validation, save-failure or unsaved-change matrix. |
| Profile deletion cancellation | PASS | Real X11 Cancel; file SHA-256 unchanged. |
| Profile deletion reopen | PASS | Same profile ID/name/description reopened and verified. |
| Selection-change deletion safety | PARTIAL | Background click dismissed modal without navigating; arbitrary state mutation not tested and no captured-ID invariant exists. |
| Full keyboard-only traversal | PARTIAL | Dashboard 12× Tab, Shift+Tab, macro 8× Tab, Shift+Tab and Dismiss focus; not every screen/control/dialog. |
| AT-SPI runtime registration | BLOCKED | Locked backend watches absent `ScreenReaderEnabled`; candidate watches `IsEnabled`, but private AT-SPI socket returns `ECONNREFUSED`. |
| Screen-reader interaction | BLOCKED | Orca absent; no TH80 AT-SPI tree or spoken output. |
| Dialog focus restoration | PARTIAL | Existing automated restoration tests pass; native X11 Dismiss entry/activation observed, not every dialog/chooser return path. |
| Window resizing | PARTIAL | 820×560, 1000×660, 1180×780 and 1600×900; not all screens/dialogs at all sizes. |
| Long-content behavior | PARTIAL | 24 layers/960 assignments/240 steps/long text loaded and scrolled; single-line option display and accessible full-value traversal not certified. |
| Native Windows execution | BLOCKED | No native Windows runner/desktop/booted VM; Wine 11.20 and MSVC target are not native execution. |
| Native macOS execution | BLOCKED | No macOS host, SDK, runner or VoiceOver. |
| Hardware isolation | PASS | All new test repositories synthetic; no hardware communication introduced. |

The X11 test process was stopped. The original Wayland PID 359824 remains alive. The stress profile and logs remain in `/tmp/th80-phase14-1.1-x11`; no real profile data was accessed.

**Phase 14.1.1 release decision: OPEN.** The Phase 14.1.2 addendum below supersedes this checkpoint with two source fixes, a refined AccessKit diagnosis and current acceptance results.

### Final preservation and verification scope

The 143 screenshots present at the beginning of Phase 14.1.1 are all still present by filename. There are 78 additional X11 evidence screenshots, 221 total. Final project inventory (excluding `.git` and `target`) is 383 paths: 187 baseline, zero missing, 196 additions. The 196 additions comprise the earlier 114 Phase 14.1 evidence captures, these 78 new X11 captures, and the four Phase 14.1 reports. No application source or test code changed in this continuation; the five Cargo checks were therefore not rerun, in accordance with the instruction not to repeat them without code changes. The last complete run remains 123 workspace tests (31 GUI tests) with formatting, strict Clippy, release build and docs all passing.

## Phase 14.1.2 final verification and acceptance (2026-11-02)

### Current Cargo verification

All five required commands were rerun after the Phase 14.1.2 source changes. Every command exited 0:

| Command | Exit | Result |
|---|---:|---|
| `cargo fmt --all --check` | 0 | PASS |
| `cargo test --workspace` | 0 | PASS — 125 passed, 0 failed |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | PASS |
| `cargo build --workspace --release` | 0 | PASS |
| `cargo doc --workspace --no-deps` | 0 | PASS |

The 125 tests include 33 GUI tests (up from 31). The two additional GUI regressions cover captured profile-deletion target safety under active-selection mutation and complete long-option AccessKit value/description exposure.

### Final 16-item acceptance matrix

| Acceptance dimension | Status | Evidence / exact limitation |
|---|---|---|
| Wayland interaction | PASS | Existing native Wayland profile, keymap, layer, macro, lighting, options, chooser, stale-save and save-error workflows; original PID 359824 remains alive. |
| X11 interaction | PARTIAL | Real X11 screen/profile interaction and delete Cancel/reopen/confirm; import/export blocked by GTK portal parent association; X11 save-failure recovery not conclusively verified. |
| Profile deletion cancellation | PASS | Real X11 confirmation cancellation; same synthetic profile reopened afterward. |
| Profile deletion reopen | PASS | Same synthetic ID/profile was reopened after cancellation; prior X11 evidence also includes persistence checks. |
| Profile selection-change safety | PASS | Pending profile ID is captured when confirmation opens; regression changes active selection and proves deletion still targets the captured ID. Modal background interaction cancels rather than navigating. |
| Macro deletion safety | PASS | Existing captured-macro-ID implementation and regression tests cover cancellation and selection mutation before confirmation. |
| Full keyboard navigation | PARTIAL | Real Tab/Shift+Tab samples and several screen captures; not every control/dialog with Tab, Enter, Space, arrows, Escape and focus restoration. |
| AccessKit D-Bus activation | BLOCKED | Locked 0.21.1 listens for absent `ScreenReaderEnabled`; 0.24.1 listens for `IsEnabled`, but no verified live activation/registration and no initial property read is performed by either inspected source. |
| AT-SPI registration | BLOCKED | `GetAddress` returns `/run/user/1000/at-spi/bus_0`; direct `gdbus`/`busctl` connections fail `ECONNREFUSED`; no TH80 registry entry. |
| Accessibility tree inspection | PARTIAL | Automated egui AccessKit updates and long-option node value/description are inspected in tests; no live AT-SPI application tree is reachable. |
| Actual screen-reader verification | BLOCKED | Orca not installed; no accessible TH80 AT-SPI tree or spoken interaction. |
| Dialog focus restoration | PARTIAL | Automated modal focus restoration passes; native X11 Dismiss entry/activation sampled, not all dialogs/chooser return paths. |
| Long-content accessibility | PARTIAL | Full option editor value and description are asserted in AccessKit regression test; hover tooltip added; live AT-SPI, keyboard caret/copy/paste and other long text fields not fully verified. |
| Windows native execution | BLOCKED | No native Windows desktop/runner or booted VM; MSVC target and Wine do not count. |
| macOS native execution | BLOCKED | No macOS host, Apple SDK, native runner or VoiceOver. |
| Hardware isolation | PASS | Normal dependency scans for GUI/CLI and source scans show no HID/USB device access in the GUI path; no hardware was opened or communicated with. |

### Runtime and platform blockers

The locked AccessKit backend/property mismatch and the private AT-SPI socket transport failure are documented in the accessibility report. The `IsEnabled` value was restored to its original `true`; `ScreenReaderEnabled` was never forced. The candidate eframe 0.37.0 / AccessKit Unix 0.24.1 dependency graph compiled in isolation but was not merged because runtime registration remained blocked and the framework upgrade is broader than a targeted backend fix.

The X11 GTK chooser journal repeatedly reports `Unhandled parent window type` and `Failed to associate portal window with parent window`. The portal window appears, but X11 import/export round-trip is not established. The X11 save-failure obstruction was restored, but unreliable text-entry/focus evidence means no end-to-end GUI recovery claim is made. Full keyboard traversal remains incomplete test coverage, not a reproduced focus trap.

### Final preservation and scope

Normalized inventory comparison: 383 Phase 14.1.1 baseline paths, 440 current paths excluding `.git` and `target`, zero missing baseline paths, 57 additions. The evidence directory contains 278 valid PNG/JPEG screenshots; the prior 221 screenshot paths remain present. The 57 new paths are Phase 14.1.2 X11 screenshots. The original Wayland PID 359824 was preserved; isolated X11 test PID 1440436 was stopped. The temporary roots `/tmp/th80-phase14-1.2-x11/` and `/tmp/th80-phase14-1.2-accesskit-upgrade/` remain for inspection. No real profile data, hardware, firmware or vendor artifacts were accessed or modified. The repository has no commits or remotes and nothing was staged.

**Release decision: Phase 14.1 remains OPEN.** The profile/macro deletion-safety regressions and current workspace checks pass, but full X11 workflow parity, the full native keyboard matrix, current-baseline AT-SPI registration, actual screen-reader interaction and the remaining long-content matrix are not closed. Do not claim accessibility certification or close Phase 14.1.

## Phase 14.1.3 final verification and authoritative acceptance matrix

### Main-project regression verification

The current main project remains on eframe 0.35.0. After the final native-egui top-bar refinement, all five requested workspace checks exited successfully:

| Command | Result |
|---|---|
| cargo fmt --all --check | PASS |
| cargo test --workspace --offline | PASS — 126 passed, 0 failed, including 34 GUI tests |
| cargo clippy --workspace --all-targets -- -D warnings | PASS |
| cargo build --workspace --release --offline | PASS |
| cargo doc --workspace --no-deps --offline | PASS |

The eframe 0.37.0 candidate was not merged. Its earlier isolated runtime registration/tree success is documented in the accessibility report, but its full candidate test/Clippy/docs and complete native interaction acceptance were not established. The temporary GTK3 file-dialog candidate was built only from a copy under /tmp; the main Cargo.toml and Cargo.lock were not changed.

### Preservation and isolation

The final normalized inventory comparison against the Phase 14.1.2 inventory reports 440 baseline paths, 551 current paths, zero missing baseline paths and 111 additions. The evidence directory contains 387 valid screenshots, with all 278 baseline screenshots preserved and zero ImageMagick identification errors. The 111 additions are 109 screenshot files, the native Config/settings source file and one synthetic “Entropy Visual Review” profile fixture. The fixture description explicitly identifies it as synthetic offline data with no device data.

The original Wayland process PID 359824 remains alive and untouched. All GUI tests in this continuation used synthetic profile/config roots. No physical keyboard, HID/USB device, firmware, vendor artifact or real profile data was accessed. The repository has no HEAD or remotes; nothing was staged, committed, pushed or published.

### Authoritative 17-item acceptance matrix

This matrix supersedes the earlier Phase 14.1.1/14.1.2 summary matrices above. It evaluates exactly the 17 dimensions required by the Phase 14.1.3 specification.

| # | Acceptance dimension | Status | Evidence / exact limitation |
|---:|---|---|---|
| 1 | Wayland interaction | PASS | Previously completed native Wayland workflows remain valid; original PID 359824 is preserved. |
| 2 | X11 interaction | PARTIAL | Current-source X11 keyboard navigation, profile selection, layer switching and page interaction were exercised. No successful X11 export/reimport round-trip, invalid-import recovery or complete file-dialog workflow was established. |
| 3 | Profile deletion cancellation | PASS | Phase 14.1.2 synthetic X11 confirmation was cancelled; repository content remained unchanged. |
| 4 | Profile deletion reopen | PASS | The same synthetic profile was reopened and verified before the confirmed deletion step in the earlier X11 run. |
| 5 | Profile selection-change safety | PASS | Regression test profile_delete_confirmation_keeps_its_original_target_if_active_selection_changes verifies the captured profile ID remains the deletion target. |
| 6 | Macro deletion safety | PASS | Regression tests macro_confirmation_deletes_original_target_after_selection_changes and macro_confirmation_cancel_preserves_both_macros_and_reference verify captured macro ID and cancellation behavior. |
| 7 | Full keyboard navigation | PARTIAL | Real X11 Tab/Enter navigation reached Layout, Advanced, Lighting and Profile library; Escape closed a message dialog and layer controls were exercised. The full required key/control/dialog matrix is not complete. |
| 8 | AccessKit activation | PARTIAL | AccessKit is compiled/enabled and the 0.37 candidate activated in a historical isolated session; current main eframe 0.35 runtime activation/registration is not established. |
| 9 | AT-SPI socket connectivity | BLOCKED | The host-advertised private socket /run/user/1000/at-spi/bus_0 returns ECONNREFUSED. A fresh isolated retry created a private socket, but the app-side connection did not complete. |
| 10 | AT-SPI application registration | PARTIAL | Candidate 0.37 historically registered as :1.6876 and exposed a tree in an isolated session; current eframe 0.35 registration is not established and the isolated success is not reproducible in the current session. |
| 11 | Accessibility tree inspection | PARTIAL | The historical 0.37 candidate tree was traversed and Layout/Advanced/Lighting/Keyboard options actions were invoked; no current-baseline 0.35 tree was obtained. |
| 12 | Actual screen-reader interaction | BLOCKED | Orca/Accerciser are unavailable; no speech or actual screen-reader announcement was captured. |
| 13 | Dialog focus restoration | PARTIAL | Automated egui focus-restoration regression passes and native Escape dismissal was exercised; restoration after every required chooser/error/confirmation dialog is not verified. |
| 14 | Long-content accessibility | PARTIAL | Full synthetic JSON option value is present in AccessKit value/description and tooltip; profile descriptions, long names/actions, extension objects, validation messages, caret/copy/paste and screen-reader exposure remain unverified. |
| 15 | Windows native GUI execution | BLOCKED | No native Windows desktop/runner or booted Windows VM; MSVC cross-target and Wine do not count as native execution. |
| 16 | macOS native GUI execution | BLOCKED | No macOS host, Apple SDK, native runner or VoiceOver environment. |
| 17 | Hardware isolation | PASS | No GUI HID/USB/serial transport dependency or physical-device communication was introduced; all test repositories were synthetic. |

### Final acceptance decision

Phase 14.1 remains **OPEN**. The completed Linux regressions and current Cargo checks are PASS, but X11 workflow parity, full keyboard-only traversal, current-baseline AT-SPI connectivity/registration, actual screen-reader interaction, dialog focus coverage and long-content coverage are not all PASS. The historical eframe 0.37 registration result is useful evidence but does not justify a dependency upgrade without full candidate validation. No Windows/macOS native verification is claimed. Stop here without commits, publication, hardware integration or firmware work; resume only on the specific outstanding acceptance dimensions.


## Phase 14.1.4 continuation — final acceptance matrix (supersedes earlier matrices)

Inventory snapshot: 23,590 paths before Phase 14.1.4 evidence; 23,614 afterward; 0 missing baseline paths; 24 additions. There are 444 image files in `.phase14-1-evidence/`; ImageMagick identified all 444 with 0 errors. Original Wayland PID 359824 and GTK3 X11 PID 1888578 remain alive. Separate fresh-process reopen verification used PID 1971100. All three were alive at final process check.

| # | Acceptance dimension | Status | Evidence / limitation |
|---:|---|---|---|
| 1 | Wayland interaction | PASS | Previously verified Wayland workflows retained; original PID 359824 preserved. |
| 2 | X11 interaction | PARTIAL | GUI export/import, invalid import, unsaved Cancel, save failure/retry and fresh-process reopen verified. Complete keyboard, discard, chooser cancellation/focus and all-screen parity remain incomplete. |
| 3 | Profile deletion cancellation and reopen | PASS | Existing synthetic X11 evidence and regression remain valid; not unnecessarily repeated. |
| 4 | Macro deletion safety | PASS | Captured-ID and cancellation regression tests remain valid. |
| 5 | Import/export semantic round trip | PASS | Real GTK3 X11 export/reimport; source/export hash match and imported semantic JSON equality excluding provenance; extension fixture preserved. |
| 6 | Invalid-import protection | PASS | GUI rejected malformed JSON with `PROFILE_JSON_INVALID` and parse location; repository hashes unchanged/no partial write; corrected fixture imported. |
| 7 | Unsaved-change cancellation | PASS | Dirty draft triggered modal via profile selector; Cancel closed modal, retained edited description in memory and left repository hash unchanged. |
| 8 | Confirmed discard | PARTIAL | Modal was reached; the Discard and open action and post-discard disk/reopen verification were not conclusively completed. |
| 9 | Save failure and recovery | PASS | Directory permission obstruction yielded `REPOSITORY_IO_ERROR`/Permission denied; disk hash unchanged; draft retained; mode restored; retry saved valid JSON; fresh app reopened and displayed persisted values. |
| 10 | Keyboard-only navigation | PARTIAL | Automated navigation/text/focus regressions and sampled native X11 navigation exist; comprehensive per-control/per-dialog matrix is not complete. |
| 11 | Dialog focus restoration | PARTIAL | Escape dismissal and unsaved Cancel returned to useful screens; automated modal focus test passes; native restoration across every dialog/chooser is not established. |
| 12 | Long-content inspection | PARTIAL | Full JSON option value is in AccessKit value/description; older stress screenshots remain. Other long fields, extension/validation content, narrow/wide live inspection and caret/copy/paste not comprehensively reverified. |
| 13 | AT-SPI registration | BLOCKED | Current host address still advertises `/run/user/1000/at-spi/bus_0`; prior direct socket connection returned ECONNREFUSED; no new current eframe 0.35 registration. |
| 14 | Accessibility tree inspection | PARTIAL | Historical isolated eframe 0.37 tree was traversed; current baseline eframe 0.35 tree is not available on the refusing host bus. |
| 15 | Actual screen-reader interaction | BLOCKED | Orca/Accerciser unavailable; no speech or announcement evidence. |
| 16 | Windows native execution | BLOCKED | No native Windows environment/runner. Cross-compilation and Wine do not count. |
| 17 | macOS native execution | BLOCKED | No native macOS environment/runner. |
| 18 | Hardware isolation | PASS | No physical keyboard, USB, HID, hidraw, serial, firmware or vendor-artifact interaction. Synthetic repositories only. |

### Phase 14.1 acceptance decision

**Phase 14.1 remains OPEN.** The X11 data workflows and controlled save recovery are now strong PASS results. However, confirmed discard is not proven, comprehensive keyboard-only navigation and native dialog focus restoration remain partial, long-content inspection is incomplete, and current-baseline AT-SPI registration/screen-reader execution remain environmentally blocked. Do not label the Linux GUI acceptance closed while feasible interaction items remain partial. Windows/macOS native execution are external platform blockers and remain BLOCKED.

### Phase 14.2 readiness

The mandatory design reference is Ergohaven Entropy: https://github.com/ergohaven/entropy. Phase 14.2 must be a recognizable Entropy-inspired native-egui redesign with actual screenshot comparisons, centered on the Layout/keymap workspace and the Layout / Advanced / Config information architecture. It must adapt only real TH80 profile capabilities, preserve offline profile management, avoid copying upstream code/assets/branding/device transport, and avoid inventing physical key geometry. No Phase 14.2 redesign work was started in this acceptance continuation.

## Phase 14.1.5 — final verification and authoritative 18-dimension matrix (2026-11-02)

### Cargo regression verification

The confirmed-discard state regression was strengthened to mutate and then verify name, description, tags and a JSON option field. All required commands were run after that source/test change; each exited `0`:

| Command | Exit | Result |
|---|---:|---|
| `cargo fmt --all --check` | 0 | PASS |
| `cargo test --workspace` | 0 | PASS — 128 passed, 0 failed |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | PASS |
| `cargo build --workspace --release` | 0 | PASS |
| `cargo doc --workspace --no-deps` | 0 | PASS |

The 128 tests comprise 9 CLI tests, 36 core tests, 36 GUI tests, 44 probe tests and 3 research CLI tests. The GUI suite includes 36 passing tests, including confirmed discard, modal focus restoration, all-screen accessible names, long profile-name navigation bounds and full long-option AccessKit value/description coverage.

### Preservation, inventory and isolation

The Phase 14.1.4 normalized baseline contains 23,590 paths. The final Phase 14.1.5 normalized inventory contains 26,320 paths: 3,526 additions and 796 paths no longer present. All 796 missing paths are generated files under `target/debug/incremental`; there are zero missing baseline paths outside `target/`. Cargo regenerated the active build outputs during the required checks; no source, report, fixture or evidence path was lost.

The evidence directory now contains 533 validated PNG images. All 444 image filenames in the Phase 14.1.4 baseline remain present; 89 new images were added. ImageMagick decoded all 533 images with zero errors. The newly retained Phase 14.1.5 evidence includes confirmed-discard screenshots, keyboard focus captures, stress-screen captures and long-value editing/copy/paste captures.

The original Wayland PID 359824 and named GTK3 X11 PID 1888578 remain alive. Other already-existing isolated GUI sessions observed during the final process check (PIDs 1971100, 2042117 and 2113082) were also left running. The confirmed-discard and stress-profile hashes match their pre-test values. All test repositories were synthetic under `/tmp`; no real profile directory, physical keyboard, HID/USB device, serial transport, firmware or vendor binary was accessed. Static inspection confirms `th80-gui` has no HID/USB/serial/udev dependency in its normal dependency graph and no hardware transport API in its source.

The repository still has no commit/HEAD and no remotes; nothing is staged, committed or published. The complete repository inventory includes Cargo-generated `target/` churn; the 796 missing paths are disposable incremental compiler-cache artifacts, not user-authored project data.

### Final 18-dimension acceptance matrix

| # | Acceptance dimension | Status | Evidence / exact limitation |
|---:|---|---|---|
| 1 | Wayland interaction | PASS | Previously verified native Wayland flows remain valid; original PID 359824 remains alive. |
| 2 | X11 interaction | PARTIAL | X11 profile CRUD, macro/lighting/options editing, semantic import/export, invalid-import protection, unsaved Cancel and confirmed discard, save failure/retry/reopen are evidenced. Complete keyboard traversal and native dialog focus restoration remain incomplete. |
| 3 | Profile deletion cancellation and reopen | PASS | Existing actual X11 cancel/reopen/persistence evidence and captured-target regression remain valid. |
| 4 | Macro deletion safety | PASS | Captured macro ID, cancellation and selection-change regressions pass. |
| 5 | Import/export semantic round trip | PASS | Actual GTK3 X11 1,805-byte export/reimport; semantic JSON comparison excluding provenance passes; nested dynamically keyed extension survives. |
| 6 | Invalid-import protection | PASS | Actual X11 malformed JSON yields `PROFILE_JSON_INVALID` with parse location; repository hashes unchanged and no partial profile written; corrected import succeeds. |
| 7 | Unsaved-change cancellation | PASS | Actual X11 Cancel retains the in-memory draft and leaves repository hash unchanged. |
| 8 | Confirmed discard | PASS | Actual X11 Discard and open; original profile hash unchanged; original saved name/description reopened. Regression additionally covers tags and JSON option data. |
| 9 | Save failure and recovery | PASS | Controlled synthetic repository write failure returns `REPOSITORY_IO_ERROR`; original hash unchanged, draft retained, permissions restored, retry persisted valid JSON and fresh process reopened it. |
| 10 | Keyboard-only navigation | PARTIAL | Native Tab/Shift+Tab/Enter and selected-screen navigation evidence expanded; Profile details 24-step traversal and selected long-value editing verified. Not every control on all screens/dialogs, Space, arrows and Escape paths is covered. |
| 11 | Dialog focus restoration | PARTIAL | Unsaved-modal focus entry/Discard activation and save-error Escape dismissal are evidenced; automated modal focus restoration passes. Exact native focus restoration after every chooser, deletion, validation and conflict dialog is not established. |
| 12 | Long-content inspection | PARTIAL | Full 4,558-byte JSON option and 350-byte macro name copy/caret selection verified; same option pasted back byte-identically; 240-step list rendered/scrollable. Other descriptions, validation messages and nested extension values are not fully inspected. |
| 13 | AT-SPI registration | BLOCKED | `GetAddress` advertises `/run/user/1000/at-spi/bus_0`; direct connection returns `ECONNREFUSED`; locked backend property mismatch remains. |
| 14 | Accessibility tree inspection | PARTIAL | AccessKit updates, all-screen accessible-name tests and full option-value description tests pass; no live current eframe 0.35 AT-SPI tree is reachable. Historical 0.37 tree is not current-baseline proof. |
| 15 | Actual screen-reader interaction | BLOCKED | Orca unavailable; no speech, announcements or screen-reader navigation captured. |
| 16 | Windows native execution | BLOCKED | No native Windows host/runner/booted VM. Cross-compilation and Wine do not count. |
| 17 | macOS native execution | BLOCKED | No native macOS host/runner/Apple SDK or VoiceOver environment. |
| 18 | Hardware isolation | PASS | Synthetic repositories only; no hardware transport dependency in GUI normal graph or API in GUI source; no device communication occurred. |

Totals: **9 PASS · 5 PARTIAL · 4 BLOCKED · 0 NOT_TESTED**.

### Final acceptance decision

**Phase 14.1 remains OPEN.** Confirmed discard, save recovery, import/export, invalid-import protection and previously passing profile/macro safety workflows are PASS. The external AT-SPI/screen-reader and Windows/macOS blockers are accurately documented. However, full native keyboard traversal, exact focus restoration across all native dialog classes and the broad long-content matrix are feasible Linux application-level requirements that remain PARTIAL. Do not close Phase 14.1 or claim full keyboard accessibility/screen-reader compatibility until those gaps are closed or explicitly accepted by the user.

### Mandatory Phase 14.2 design handoff

Reference: https://github.com/ergohaven/entropy. The prior `PHASE12_ENTROPY_REFERENCE_AUDIT.md` is complete and the retained actual reference captures are `.phase14-1-evidence/entropy-reference/layout.png`, `key-picker-light.png`, `key-picker-dark.png` and `matrix-tester.png`. Phase 14.2 must be a substantial, recognizable Entropy-inspired redesign, not a generic polish pass. Revisit the upstream application and screenshots, then create side-by-side comparisons against TH80 Studio's corresponding Layout, key picker/selected-key editor, layer and settings views. Keep native Rust/egui and TH80's offline profile functionality. Rework the Layout / Advanced / Config hierarchy around a central keymap workspace, layer navigation, selected-key editor and action/keycode picker, structured settings panels, menus, dialogs, spacing, themes and scaling. Adapt only capabilities present in TH80's profile model; do not invent physical switch geometry, firmware features or device transport. Do not copy Entropy source, branding, assets or device transport. No Phase 14.2 redesign work was started in Phase 14.1.5.

## Phase 14.1.6 — Final focused acceptance addendum (2026-11-02)

### Native keyboard/focus results

The complete source-derived inventory is recorded in `PHASE14_1_GUI_INTERACTION_AUDIT.md` and the detailed accessibility disposition in `PHASE14_1_ACCESSIBILITY_AND_KEYBOARD_NAVIGATION.md`. It covers all actual GUI pages and interactive controls, including shared navigation, profile operations, keymap/layers, macro editor, lighting, options, application settings, egui modal classes and rfd portal dialogs. No absent control (device picker, firmware screen or physical TH80 matrix) was invented.

Real X11 input in the isolated updated-release process (PID `2164192`, synthetic profile root `/tmp/th80-phase14-1.6-x11/profiles`) used Tab and Enter to navigate into Profile library. It did not establish full keyboard-only profile opening and subsequent editing. The preserved older X11 process on PID `1971100` was used for real Tab/Shift+Tab and Enter samples on its existing synthetic Profile/Keymap state. These are limited samples, not a complete per-control matrix. No pre-existing process was restarted or terminated.

The source fix adds descriptive AccessKit names to keymap layer navigation, position selection and position removal. The new egui test verifies Tab reaches a logical position and Enter selects it. This closes a small semantic-label gap, not the full native keymap acceptance requirement. Actual native layer switching, action-category selection, action value editing and clearing through keyboard alone remain **PARTIAL**.

### Dialog and chooser focus matrix disposition

| Dialog/flow | Implementation/test evidence | Native focus-restoration disposition |
|---|---|---|
| Profile deletion cancel/confirm | Prior real X11 data-safety evidence; egui target-ID regression | Exact keyboard focus before/inside/after both branches not fully captured; PARTIAL |
| Macro deletion cancel/confirm | Prior X11 delete safety and egui Escape/target tests | Exact native focus after both branches not fully captured; PARTIAL |
| Unsaved changes cancel/discard | Prior real X11 dialog button activation and discard/reopen proof | Exact originating widget focus identity after every branch not captured; PARTIAL |
| Import chooser cancel/complete | Prior native XDG portal cancel and successful import | Portal keyboard traversal and parent focus return not established; PARTIAL |
| Export chooser cancel/complete | Prior native XDG portal export completion/cancel evidence | Portal keyboard traversal and parent focus return not established; PARTIAL |
| Invalid import / validation / save failure | Existing native status/error dialogs and save-error Escape sample | All message lengths and post-dismissal focus identities not exhaustively captured; PARTIAL |
| Save conflict recovery | Existing stale-save reload/recovery workflow and egui state coverage | Native focus return after Reload/Save as/Dismiss not exhaustively captured; PARTIAL |

Application confirmation/status dialogs are egui Modal overlays. Import/export use rfd 0.17.2 asynchronous XDG/GTK portal dialogs in separate native surfaces. The rfd API has optional `set_parent` for XDG, but the current application does not set an explicit parent. The app worker/channel/pending/repaint lifecycle is covered by existing tests and native responsiveness evidence. No native chooser ownership or focus bug was reproduced, so no file-dialog rewrite was performed.

### Long content

Retained byte-identical copy/paste and caret evidence for the 4,558-byte option and 350-byte macro name remains valid. The 240-step macro remains scrollable. Long descriptions wrap and option editors expose tooltip/AccessKit descriptions, but full nested extension inspection, every long description, validation-message scrollability and all tooltip/accessibility combinations remain partial. No data loss or serialization mutation was reproduced.

### Final 18-dimension acceptance matrix

| # | Dimension | Status | Phase 14.1.6 disposition |
|---:|---|---|---|
| 1 | Wayland interaction | PASS | Existing native Wayland workflows remain valid; no regressions or process restart. |
| 2 | X11 interaction | PARTIAL | Multiple actual X11 workflows and this phase's Tab/Shift+Tab/Enter samples; not every screen/dialog/control. |
| 3 | Profile deletion cancellation and reopen | PASS | Prior native X11 cancel/hash/reopen evidence remains intact. |
| 4 | Macro deletion safety | PASS | Captured target identity and cancel/confirm regression tests remain passing. |
| 5 | Import/export semantic round trip | PASS | Existing native and core semantic round-trip evidence retained. |
| 6 | Invalid-import protection | PASS | Existing invalid JSON/collision protection remains passing. |
| 7 | Unsaved-change cancellation | PASS | Existing native X11 cancellation evidence remains intact. |
| 8 | Confirmed discard | PASS | Native X11 discard and original-profile hash preservation remain intact. |
| 9 | Save failure and recovery | PASS | Existing permission failure, retained draft and recovery evidence remain intact. |
| 10 | Keyboard-only navigation | PARTIAL | Inventory completed; new keymap AccessKit labels and simulated Tab/Enter position test pass; comprehensive native per-control and keymap action workflow not established. |
| 11 | Dialog focus restoration | PARTIAL | egui modal focus restoration test and selected native cases exist; all 14 native dialog/chooser paths and exact focus identity are not established. |
| 12 | Long-content inspection | PARTIAL | Existing byte-identical option/macro checks and 240-step scrolling retained; descriptions, nested extensions and validation messages not exhaustive. |
| 13 | AT-SPI registration | BLOCKED | Locked backend/session status-property mismatch and refused advertised socket remain. |
| 14 | Accessibility tree inspection | PARTIAL | AccessKit generated updates and new keymap labels are testable; no reachable live current-baseline AT-SPI tree. |
| 15 | Actual screen-reader interaction | BLOCKED | Orca/live screen-reader operation remains unavailable and unverified. |
| 16 | Windows native execution | BLOCKED | No native Windows runner. |
| 17 | macOS native execution | BLOCKED | No native macOS runner. |
| 18 | Hardware isolation | PASS | GUI normal dependency graph/source remain hardware-transport-free; synthetic profiles only. |

Totals remain **9 PASS · 5 PARTIAL · 4 BLOCKED · 0 NOT_TESTED**. The unchanged totals are intentional: this phase added a narrow keymap semantics improvement, but did not overstate native keyboard/focus coverage.

### Regression verification and preservation

| Command | Exit | Result |
|---|---:|---|
| `cargo fmt --all --check` | 0 (after applying rustfmt) | PASS |
| `cargo test --workspace` | 0 | **129 passed, 0 failed** (th80ctl 9; core 36; GUI 37; probe 44; HID research 3) |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | PASS |
| `cargo build --workspace --release` | 0 | PASS |
| `cargo doc --workspace --no-deps` | 0 | PASS |

The first format check surfaced only rustfmt differences in the new test; `cargo fmt --all` corrected them and the final check passed. Normalized repository inventory excluding `.git` and `target` grew from 721 to 741 paths (20 additions, zero missing baseline paths). Nineteen new phase-specific screenshots are retained under `.phase14-1-evidence/phase14-1.6/`; the evidence tree now contains 552 image files. Existing GUI processes `359824`, `1888578`, `1971100`, `2042117` and `2113082` remained alive; new isolated X11 PID `2164192` also remains alive. No real profile data, device communication, firmware/vendor artifacts, commits, remotes or staged changes were introduced. The workspace still has no initial HEAD and no remotes.

### Final decision and Phase 14.2 handoff

**Phase 14.1 remains OPEN.** The native keyboard-only control matrix, complete keymap operation, exact native dialog focus restoration and full long-content inspection are not closed. The application-level focus and file-dialog implementation has been traced, and no reproducible focus defect justified a risky rewrite. AT-SPI registration, actual screen-reader interaction, Windows and macOS remain accurately BLOCKED. Do not mark the phase PASS or claim keyboard/screen-reader certification.

Phase 14.2 is ready to start only after the remaining acceptance gaps are explicitly resolved/accepted. Mandatory reference: https://github.com/ergohaven/entropy and `PHASE12_ENTROPY_REFERENCE_AUDIT.md`, with retained captures in `.phase14-1-evidence/entropy-reference/`. Build a substantial recognizable Entropy-inspired native egui redesign around Layout / Advanced / Config, a central model-driven logical-position workspace, adjacent layer navigation, selected-position inspector, searchable/category-based action picker and structured settings panels. Preserve TH80's offline profile repository, serialization and hardware isolation. Do not copy Entropy source, assets, branding, Vial/RMK models, physical geometry or transport. No Phase 14.2 implementation was started in this phase.
