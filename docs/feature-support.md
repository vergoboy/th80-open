# TH80 Studio feature support matrix

TH80 Studio is currently an offline profile configurator. “Complete” means the local profile workflow is implemented; it does not mean that the feature is supported by TH80 firmware.

| Area | Status | Current behavior and boundary |
|---|---|---|
| Create profile | Complete | Creates a fresh schema-v1 local profile. |
| Duplicate profile | Complete | GUI dashboard and CLI duplicate saved profile data with a new identity and fresh timestamps. |
| Rename profile | Complete | Editable in Profile details; CLI profile rename is also available. |
| Delete profile | Complete | GUI requires confirmation; CLI deletion is explicit and immediate. |
| Import/export | Complete | JSON import validates and refuses ID collisions; export never overwrites an existing destination. |
| Active profile | Complete for current session | A single unambiguous local profile opens automatically into Layout at startup; when multiple profiles exist, the user explicitly selects one. No persistent active-profile pointer is stored. |
| Metadata | Complete | Name, description, tags, creation time and last-modified time are local profile metadata. |
| Validation | Complete for schema/application policy | Structured diagnostics; does not establish firmware compatibility or device constraints. |
| Unsaved changes | Complete | Switching profiles prompts to save, discard or cancel. |
| Concurrent edits | Complete | Repository lock and stale-revision conflict errors prevent silent last-writer-wins. |
| Logical layers | Complete | Create, rename, select and delete layers; deletion is blocked while referenced. |
| Key assignments | Complete | Logical-position CRUD, selected-position inspector and a searchable categorized picker for the eight typed actions: key, modifier, combination, layer switch, momentary layer, macro, transparent and unassigned. |
| CLI typed key actions | Complete | set-action accepts the same tagged Action JSON variants as the shared core model; set-key remains a convenience command. |
| CLI layer management | Complete | Add, rename and remove operations use the same core validation/repository path; removing a referenced layer fails validation. |
| CLI macro step reordering | Complete | Macro step-move swaps zero-based indexes and validates/persists through the core repository. |
| Physical key layout | Blocked by missing evidence | No physical TH80 switch matrix is asserted or drawn. |
| Keymap undo/redo | Missing | No safe history/transaction model exists yet. |
| Macro editing | Complete | Create, duplicate, rename, edit ordered key-down/key-up/delay/text steps and reorder steps. Deletion requires confirmation and remains an unsaved local draft change until saved. |
| Macro playback/device encoding | Blocked by unverified compatibility | Macro steps are local data only; unknown serialized step types are rejected explicitly. |
| Lighting profile | Complete | Optional enabled/mode/brightness/speed/primary color, logical-position colors and opaque JSON effect parameters. |
| Lighting on device | Blocked by unverified protocol | No device state, RGB transfer or hardware application exists. |
| Keyboard options | Complete as generic JSON | Application-owned JSON values; no firmware meaning or device limits implied. |
| Unknown top-level extensions | Complete | Preserved through load/edit/save/import/export and shown read-only in the GUI. |
| Unknown nested schema fields | Explicitly rejected | Nested structs deny unknown fields; import/load returns a parse error rather than silently dropping data. |
| Error reporting | Complete for local operations | Structured core errors/diagnostics shown in GUI modal messages and CLI exit codes. |
| Keyboard navigation | Partial | Logical-position controls retain descriptive AccessKit names; layer Left/Right and wheel navigation and picker Escape/Apply are implemented. Comprehensive native traversal, dialog focus restoration, AT-SPI runtime registration and screen-reader interaction remain incomplete. |
| Responsive layout | Partial | The logical board and picker adapt to the available viewport and use scrollable content; broader window-size/platform interaction checks remain in progress. |
| Loading/saving indicators | Partial | Local synchronous file operations report completion/errors; no asynchronous progress indicator is needed for current small local JSON operations. |
| Cross-platform behavior | Partial | Linux native verification is performed in this phase. Windows/macOS builds and native filesystem behavior require their platform CI/native runs. |

## Validation categories

- **JSON/type validation:** JSON syntax, required typed fields, known enum/action variants and schema version.
- **Application policy:** non-empty names/IDs, valid references, balanced macro key state and local resource/normalized-lighting bounds.
- **Device constraints:** currently unknown unless a constraint has authoritative evidence. No current profile validator enforces a TH80-specific matrix, macro, RGB, timing or option limit.

## Duplicate action behavior

Assigning the same action to multiple logical positions is allowed. This is intentional: duplicate key assignments can be a valid profile design and there is no verified TH80 firmware rule that would justify rejecting them. Layer/macro references and structural validity are still checked.
