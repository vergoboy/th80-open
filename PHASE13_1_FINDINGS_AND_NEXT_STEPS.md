# Phase 13.1 — Findings, Evidence Closure and Next Steps

**Status:** static investigation and four requested reports completed. No hardware testing or implementation work was performed. Stop here pending a separate explicit hardware-test authorization.

## 1. Executive summary

Phase 13.1 substantially advances the firmware evidence beyond the earlier Phase 13 binary inventory:

1. The application command dispatcher at file offset `0x19c5e` is identified in executable code. It validates a shared command buffer using checksum helper `0x19c1e` and dispatches commands through direct comparisons and compact tables.
2. The full lighting parameter pair is now firmware-correlated at the static code level: `0x08` writes the seven-field profile-indexed structure at `0x19afe`, then calls grid/record processing routine `0x146f8` at `0x19bac`; `0x87` reads the same structure at `0x195ca` and prepares a 64-byte response.
3. Firmware `0x07` is a separate compact write operation at `0x19bb6`, paired with compact read `0x86` at `0x19614`. This does not match the frontend's seven-field `0x07` lighting setter layout.
4. The upper command table's `0x89` entry resolves to a return-only epilogue at `0x19cf8`. Neither frontend `0x89` context (per-key RGB/user-picture retrieval or keymap retrieval) has a handler established in this dispatcher.
5. A candidate USB setup/class-request path at `0x1cb48` is decoded. It handles HID-like idle/protocol requests and descriptor types, but `GET_REPORT`/`SET_REPORT` map to a common default path in this table. The exact Feature SET bridge into the application dispatcher remains unresolved.
6. The firmware checksum and native vendor checksum builder are not byte-for-byte equivalent. Firmware includes command byte zero and uses it as the span selector; native code excludes byte zero and uses `dangledevtype` to choose the checksum destination.
7. RGB-related runtime routines perform substantial component and buffer transformations. The full lighting writer calls `0x146f8`, which calls `0x164a6` and `0x16c64`; a separate 12-entry transform at `0x16d2c` has independent callsites. The physical LED output stage, LED controller, signalling method and power gating remain unknown.
8. The `0x08`/`0x87` lighting state is profile-indexed in a candidate RAM structure. No persistent flash/EEPROM save/load path has been recovered.
9. Fn key state, matrix scan, `Fn + Backspace`, RGB shortcut and USB-mode shortcut implementations remain unidentified.

The physical RGB failure still has no confirmed root cause. No operation is approved for hardware testing.

## 2. Evidence classification ledger

Only the required classifications are used below.

| Finding | Classification | Evidence / limitation |
|---|---|---|
| Firmware checksum helper at `0x19c1e` | `CONFIRMED_STATIC` | Selector-dependent complemented 8-bit sum directly decoded |
| Dispatcher at `0x19c5e` calls checksum at `0x19c68` | `CONFIRMED_STATIC` | Direct `BL` reference |
| Direct dispatcher callers `0x187d0`, `0x1c7b0` | `CONFIRMED_STATIC` | Two direct references in full disassembly |
| Caller `0x187b0` copies framed packet into global command buffer `0x00021240` | `CONFIRMED_STATIC` | Source `0x000211da`, destination `0x00021240`, copy loop and dispatcher call |
| `0x08` full lighting writer at `0x19afe` | `CONFIRMED_STATIC` | Direct dispatch table entry and seven field stores |
| `0x87` full lighting reader at `0x195ca` | `CONFIRMED_STATIC` | Direct opcode comparison/branch and matching indexed field reads |
| `0x07` compact writer at `0x19bb6` | `CONFIRMED_STATIC` | Lower table entry and compact state stores |
| `0x86` compact reader at `0x19614` | `CONFIRMED_STATIC` | Direct branch and matching compact state reads |
| `0x89` return-only entry in upper dispatch table | `STRONG_STATIC_INFERENCE` | Entry byte maps to `0x19cf8` epilogue; depends on candidate Thumb decode/table interpretation |
| `0x8c` page/data response candidate at `0x1951e` | `STRONG_STATIC_INFERENCE` | Request-indexed pointer calculation and 64-byte response length; semantic identity unknown |
| USB setup/class request handler at `0x1cb48` | `STRONG_STATIC_INFERENCE` | `bmRequestType`, `bRequest`, HID-like request table and descriptor-type handling; registration edge not recovered |
| GET_REPORT/SET_REPORT support in candidate class table | `UNKNOWN` | Both map to common default path; alternate handler/interface not excluded |
| Feature SET data stage → command buffer → dispatcher | `UNKNOWN` | Callback/transport candidate exists, but exact payload pointer/copy edge not proven |
| Firmware/native checksum equivalence | `STRONG_STATIC_INFERENCE` of mismatch | Distinct byte ranges and selector semantics |
| Frontend 0x87 and 0x89 layouts | `FRONTEND_ONLY` | Host request/parser behavior from existing vendor bundle analysis |
| RGB component/matrix runtime routines | `STRONG_STATIC_INFERENCE` | Memory transformations connected in part to lighting writer; no physical output proof |
| LED controller, bus, PWM/GPIO signalling and power gate | `UNKNOWN` | No hardware-facing RGB call chain identified |
| Persistent lighting storage | `UNKNOWN` | Candidate RAM state identified; no NVM save/load chain |
| Fn state, keymap layer and named shortcut handlers | `UNKNOWN` | No defensible static handler/data-flow recovered |
| Exact MCU, board revision and installed firmware match | `UNKNOWN` | Existing TH80-family external lead is not exact-board evidence |

## 3. Key command semantics matrix

| Opcode | Firmware evidence in this image | Frontend evidence | Result |
|---:|---|---|---|
| `0x07` | Compact state writer at `0x19bb6`; consumes bytes `[1..4]` into a compact indexed structure | TH80 frontend uses `0x07` for seven-field lighting setter `[1..7]` | Structural mismatch; do not conflate |
| `0x08` | Full lighting state writer at `0x19afe`; consumes bytes `[1..7]`, calls `0x146f8` | No corresponding TH80 frontend setter established in reviewed evidence | Firmware write handler confirmed; host invocation unknown |
| `0x86` | Compact state reader at `0x19614` | Frontend keyboard-option request also uses `0x86` in inherited model code | No semantic match established from the numeric opcode alone |
| `0x87` | Full lighting state reader at `0x195ca`; prepares 64-byte response | TH80 frontend parses `[1..7]` as mode/speed/value/option/RGB; its helper does SET then GET | Strong field-layout correlation; full transaction safety/transport unknown |
| `0x89` | Upper-table entry maps to return-only epilogue `0x19cf8` | Per-key picture and keymap operations both use `0x89` with different layouts | No firmware handler established in this dispatcher |
| `0x8a` | Table target `0x19dca` calls `0x1956e` | Frontend constant exists | Handler semantics not named |
| `0x8b` | Table target `0x19dd0` calls `0x1954e` | No specific TH80 frontend operation correlated | Handler semantics not correlated |
| `0x8c` | Table target `0x19dd6` calls `0x1951e`; request-indexed pointer and 64-byte response | Generic frontend user-picture constant exists; TH80 per-key picture path uses literal `0x89` | Page/data response candidate; identity unknown |
| `0x8d` | Table target `0x19ddc` calls `0x194e0` | No specific TH80 operation correlated | Handler semantics not named |

Numeric opcode overlap is not enough to assign semantics. Frontend class/method names are not firmware symbols.

## 4. Checksum compatibility verdict

### Firmware helper

- File offset `0x19c1e`.
- `buffer[0]` is the selector.
- If byte zero is `7` or `8`, the routine accumulates bytes `0..7`, then includes checksum byte `8`.
- Otherwise it accumulates bytes `0..6`, then includes checksum byte `7`.
- It uses wrapping 8-bit addition and one's-complement validation.

### Vendor native builder

Prior Phase 5D static disassembly established:

- `dangledevtype=0`: sum offsets `1..7`, write checksum at 7.
- `dangledevtype=1`: sum offsets `1..8`, write checksum at 8.
- `dangledevtype=2`: skip checksum insertion.
- Offset zero is excluded from both native sums.

### Verdict

The two algorithms are not byte-for-byte equivalent. The firmware uses the command opcode byte as a selector and includes that byte in its sum. The native builder uses a device-type selector and excludes byte zero. Even where the checksum destination matches, the byte ranges differ. The exact deployed route and device metadata selection remain unresolved, so this discrepancy must not be promoted to a proven explanation of the physical failure.

## 5. HID reception and transport verdict

The candidate setup handler at `0x1cb48` parses standard/class USB requests. Its class request table contains handlers for GET_IDLE, GET_PROTOCOL, SET_IDLE and SET_PROTOCOL. GET_REPORT and SET_REPORT map to the common default path at `0x1cc24`, which calls `0x1cdd4` and returns value 2. The standard request branch recognizes descriptor types `0x21` and `0x22`.

The earlier handoff's suggestion that `0x1cd08` is the descriptor itself is corrected: it is a literal pool containing RAM-like pointers. The descriptor bytes and exact report IDs/lengths are not decoded.

A USB/control callback candidate exists around `0x1c5b6`, called from wrapper `0x1bd54`; that routine contains a call to the application dispatcher at `0x1c7b0`. This proves a dispatcher call exists in that processing region, but not that it was caused by the vendor HID `SET_REPORT` request or that the Feature payload was copied into global buffer `0x00021240`.

The vendor native driver uses Feature SET `bRequest=0x09` and Feature GET `bRequest=0x01`, `wValue=0x0300`, `wIndex=0`, length 64. The support metadata identifies interface 2. The mismatch and route uncertainty remain material blockers.

## 6. RGB runtime and persistence verdict

The full `0x08` handler writes a profile-indexed structure based at candidate address `0x00020400`; `0x87` reads it. The write handler calls `0x146f8` at `0x19bac`; that routine iterates a 6×15 grid, calls `0x164a6` and then `0x16c64`. Separately, `0x16d2c` iterates 12 records and calls `0x164f6`, but its direct callsites (`0x17352` and `0x19af2`) do not prove a relationship to the `0x08` handler. These functions support a color/matrix-processing interpretation, but they do not establish a physical LED controller or output bus.

The handlers do not visibly call a flash/EEPROM/NVM routine. No clear flash/NVM API strings were recovered. This is not proof that the firmware lacks persistence; the persistence owner and lifecycle remain unknown.

## 7. Fn shortcut verdict

No exact Fn state bit, matrix scan routine, Fn-layer selector, `Fn + Backspace` comparison, RGB shortcut handler or USB-mode shortcut handler was recovered. The reported cross-transport symptom pattern does not establish a shared failure path. The working USB-mode shortcut only demonstrates that at least one input/mode action remains functional.

The `0x89` keymap request has no handler in the recovered dispatcher, which is a meaningful compatibility lead but not a complete explanation of the Fn symptoms.

## 8. Concrete next steps — static only

1. **Resolve USB setup callback registration.** Trace the function-pointer/descriptor construction that registers `0x1cb48`, then follow the data stage for `SET_REPORT` and `GET_REPORT`. Prove or reject the bridge to `0x00021240` without sending any request.
2. **Inspect the 0x8c data path.** Trace the source pointer/table at `0x1973c` and determine whether the `0x8c` handler is a user-picture, keymap, or unrelated page-data response.
3. **Continue RGB output call graph.** Trace callers of `0x16d2c` (including `0x17352`) and locate any subsequent memory-to-peripheral operation. Require an actual GPIO/PWM/SPI/I2C/DMA call/data path before naming the LED signalling method.
4. **Find persistence implementation.** Identify any flash/NVM functions and trace all references to candidate state base `0x00020400`, including startup/default loading and save triggers.
5. **Trace matrix/Fn handling.** Recover actual HID input report descriptors, then follow input buffer parsing, keycode conversion, Fn state and shortcut lookup. Compare USB-mode and RGB shortcut paths only after their code references are established.
6. **Resolve host route compatibility.** Continue static analysis of ordinary `/sendMsg` and `/readMsg` native route-to-handler binding and the interface-index discrepancy. Do not execute the native driver.
7. **Confirm firmware applicability from existing evidence.** The supplied V504 image is an ANSI package with explicit family/revision restrictions. The exact board revision and installed firmware remain unknown. Do not flash or update.

These steps require only offline artifacts, disassembly and static documentation. If the needed callback tables, descriptors or source artifacts are unavailable, document the blocker rather than attempting hardware experiments.

## 9. Safety and project hygiene closure

The following remained untouched:

- original firmware payload and package sidecars;
- original vendor native driver and frontend bundle;
- historical Phase 5C/5D/7A/7B/8/9/10/11/12/13 reports;
- GUI/core/Rust source files and existing Phase 12 work;
- Git history and repository publication state.

No commits, pushes, remotes, releases, builds or implementation changes were made. The repository already contained extensive untracked work before Phase 13.1; it was preserved.

The three source artifact hashes matched the recorded baselines before analysis and were rechecked after the four reports were created. No hardware operation was performed. Hardware testing remains unauthorized; wait for a separate explicit, narrowly scoped authorization before any device-facing action.

## 10. Companion reports

- `PHASE13_1_HID_DISPATCH_ANALYSIS.md`
- `PHASE13_1_RGB_HANDLER_ANALYSIS.md`
- `PHASE13_1_FN_SHORTCUT_ANALYSIS.md`
