# Phase 14 — Application Audit

## Executive summary

The repository already contains a viable native, offline configurator architecture. The product boundary is clear in code: the GUI depends on the hardware-independent core, while the passive probe and legacy HID research executable are separate workspace packages. The core repository is substantially more mature than the GUI surface and already supplies schema validation, atomic persistence, lock files, stale-save detection and structured diagnostics.

Phase 14 therefore focused on correcting specific workflow/UX defects and closing gaps rather than replacing the architecture.

## Existing architecture

| Component | Responsibility | Hardware boundary |
|---|---|---|
| th80-core | Schema-v1 profile model, local validation, capability evidence, repository persistence | Hardware-independent; denies unsafe code |
| th80-gui | Native seven-screen eframe/egui editor and application state | Depends on core, not probe/HID research |
| th80-cli | Offline th80ctl profile commands | Depends on core; no transport |
| th80-probe | Separate legacy passive sysfs/HID descriptor research | Not in GUI dependency graph |
| Root th80ctl package / th80-hid-research binary | Separate legacy HIDAPI research utility | Not in GUI dependency graph |

The normal dependency tree of th80-gui contains no HIDAPI, USB transport, th80-probe, th80-hid-research, Vial or QMK package.

## Feature matrix

| Feature | Audit status before Phase 14 | Phase 14 outcome |
|---|---|---|
| Profile creation | Complete | Retained |
| Profile duplication | Partial (Save as only for active profile) | Dashboard duplicate action and CLI duplicate command |
| Profile rename | GUI editor only; CLI absent | CLI profile rename; GUI inline edit retained |
| Profile deletion | Complete with confirmation | Confirmation now explicitly warns when unsaved edits will be discarded |
| Import/export | Complete | Retained; duplicate IDs still fail closed |
| Active local profile | Complete for GUI session | Documented as session-only; no persistent active-profile pointer |
| Metadata/description/tags | Complete | Last-modified and created timestamps now human-readable UTC |
| Validation | Partial | Added non-empty layer/macro display-name diagnostics |
| Unsaved-change protection | Complete for switching | Retained; deletion warning made explicit |
| Concurrent edits | Complete | Repository lock and stale-revision conflict errors retained |
| Conflict handling | Partial | Conflict now offers reload from repository or save draft as a new profile |
| Logical layer editing | Complete | Retained; no physical matrix implied |
| Key assignment | Complete | Retained; switching layers clears stale position selection |
| CLI typed key actions | Complete | set-action accepts the same tagged Action JSON variants as core; set-key remains a convenience command |
| CLI layer management | Complete | Add, rename and remove use core validation; referenced layer removal fails |
| Action clearing/reassignment | Complete | Unassigned remains the clear action; typed action editor retained |
| Duplicate action handling | Complete by permissive design | Same action may be assigned to multiple logical positions; no unsupported restriction added |
| Keymap undo/redo | Missing | Remains deferred; no history/transaction model exists |
| Macro create/edit/delete | Complete | Macro deletion now requires confirmation; referenced assignments are called out before removal |
| Macro duplication | Missing | Duplicate-selected macro action added |
| Macro step ordering | Partial | Move-up/move-down controls and pure ordering test added |
| CLI macro step reordering | Complete | Zero-based step-move operation uses core validation and persistence |
| Empty macros | Supported by model | Explicit empty-state message added |
| Lighting profile | Complete | Optional enabled/mode/speed/brightness/color, logical-position colors and opaque JSON effect parameters remain local only |
| Lighting hardware application | Blocked | Still absent; no live state or RGB transfer |
| Generic keyboard options | Complete | Retained as JSON values, not firmware options |
| Unknown top-level fields | Complete | Preserved and visible read-only |
| Unknown nested typed fields | Explicitly rejected | Retained fail-closed behavior; never silently dropped |
| Error reporting | Partial | Conflict recovery actions added; core structured errors remain authoritative |
| Keyboard accessibility | Partial | AccessKit integration enabled; formal assistive-technology audit remains outstanding |
| Resizing/scrolling | Partial | Central page now scrolls; fixed navigation and minimum window size remain |
| Loading/saving progress | Partial | Local synchronous file operations report completion/errors; no asynchronous progress UI was justified by current operation size |
| Cross-platform | Partial | Linux native checks run here; Windows/macOS native GUI verification remains pending |

## Core model and compatibility findings

- Persisted JSON schema version is 1 and independent of crate version.
- Unknown fields are preserved at the profile, metadata, configuration, keymap, macro-set, lighting, and application-options extension boundaries. The GUI displays retained extension maps read-only; CLI and GUI duplicate/Save As workflows preserve them.
- Configuration options.values and lighting.effect_parameters are explicitly extensible JSON maps.
- Unsupported leaf structures and tagged Action/MacroStep variants deny unknown fields. Unknown fields at those unsupported boundaries produce a deserialization error rather than being discarded or assigned guessed semantics.
- Optional lighting values preserve explicit false, zero and black RGB values. Missing/null Option values deserialize to None; explicit null is not distinguished from missing.
- Generic lighting modes, logical positions, key names and macro steps are local abstractions. They are not proven firmware encodings.
- Application limits (macro step/delay and normalized lighting bounds) are documented as local policy, not device limits.
- Duplicate actions are permitted because there is no evidence-based reason to reject them.
- Empty layer and macro display names were previously accepted; Phase 14 adds explicit validation diagnostics for both.
- Future schema versions are rejected. There is no implicit migration path.

## Repository and reliability findings

ProfileRepository provides create/load/save/list/delete/import/export/validate operations. It uses exclusive per-profile lock files, no-clobber creation/export, same-directory temporary files, file synchronization, platform replacement primitives and stale revision checks. Cross-process and crash/failure tests already existed and passed in Phase 14.

Stale detection compares the persisted modification timestamp. Repository writers update that timestamp. An out-of-band editor that changes file contents while preserving the timestamp is outside the repository's complete conflict-detection guarantee; the GUI's reload action handles detected conflicts but does not introduce a content-hash revision protocol.

## GUI workflow defects identified and corrected

1. An options editor buffer keyed only by option name could survive a profile switch and overwrite the newly opened profile with stale text. Profile-specific editor buffers are now cleared on open/reload/delete.
2. Switching keymap layers retained a selected logical position ID. If the new layer had the same ID, the editor could unexpectedly continue editing it. Selection now resets when the layer changes.
3. Deleting a dirty profile could discard the in-memory draft after the ordinary delete confirmation. The confirmation now explicitly says the unsaved edits will be discarded.
4. A stale-save conflict only displayed an error. The conflict modal now offers reload or save-as-new-profile recovery.
5. The main content area could overflow vertically. A central vertical scroll area now handles long profiles and forms.
6. Profile timestamps were shown as raw epoch milliseconds. They are now rendered as UTC date/time strings.

## Prioritized remaining work

1. Complete native Windows and macOS GUI CI/runtime verification.
2. Perform a formal keyboard and screen-reader accessibility audit.
3. Add UI-level interaction tests for dialogs and pointer flows when a stable headless harness is available; the new macro deletion state transition is covered at the application-state level only.
4. Consider undo/redo only with an explicit, tested edit-history design.
5. Keep hardware integration blocked until the separate readiness report's evidence gates are met.

## Related documentation

- [Feature support matrix](docs/feature-support.md)
- [Profile JSON format](docs/profile-format.md)
- [Installation and development](docs/installation.md)
- [Implementation report](PHASE14_IMPLEMENTATION_REPORT.md)
- [Test and platform report](PHASE14_TEST_AND_PLATFORM_REPORT.md)
- [Hardware integration readiness](PHASE14_HARDWARE_INTEGRATION_READINESS.md)
