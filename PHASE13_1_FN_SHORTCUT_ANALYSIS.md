# Phase 13.1 — Fn Shortcut and Key-Input Path Analysis

**Scope:** static analysis of the supplied firmware and prior TH80 protocol/application evidence. No key was pressed for testing, no keyboard or receiver was accessed, and no hardware state was changed.

## 1. Executive conclusion

The exact firmware path for `Fn + Backspace`, RGB shortcuts, Fn-layer selection and USB-mode switching has **not** been recovered. The available image contains a substantial application dispatcher and generic buffer/matrix/color-processing routines, but no reliable symbol, string, dispatch entry or data-flow chain identifies a Fn state machine or the shortcut actions.

The frontend's `0x89` keymap request is not firmware-correlated: in the recovered application command dispatch table, opcode `0x89` resolves to a return-only epilogue at `0x19cf8`. This is strong evidence that the inspected dispatcher has no `0x89` handler, but it does not prove that all firmware keymap handling is absent or that another build/path does not implement it.

The user's reported symptoms therefore remain unexplained by static evidence. The working USB-mode shortcut shows only that some input/mode path remains functional; it does not establish that the RGB shortcut, Fn-layer lookup or LED output path shares that code path.

## 2. User-reported behavior retained as context

The existing Phase 13 root-cause report records these user observations:

- `Fn + Backspace` does not restore lighting.
- Other RGB shortcuts reportedly do not work.
- Most Fn shortcuts reportedly stopped responding.
- The USB-mode switching shortcut still works.
- The behavior persists over USB and 2.4 GHz.

These are user-reported observations, not reproduced hardware results. No active device query or key test was performed in this phase.

## 3. Relevant firmware command evidence

### 3.1 Shared command dispatcher

The dispatcher at file offset `0x19c5e` validates the global command buffer and selects command handlers. It is reached by direct callers at `0x187d0` and `0x1c7b0`; checksum validation occurs at `0x19c68` through helper `0x19c1e`.

This proves that multiple internal ingress paths can converge on application command dispatch. It does not identify a Fn key event or a shortcut action.

### 3.2 Keymap opcode `0x89`

The upper command table begins at `0x19cda`. The index for opcode `0x89` is zero; its entry byte is `0x0f`, which resolves to `0x19cf8`. That address contains only `POP {r4, pc}`.

The TH80 frontend uses `0x89` for two different host-side operations:

1. Per-key RGB/user-picture retrieval, with request `[0x89,0]` and subsequent GET reads parsed as RGB triples.
2. Keymap retrieval, with request `[0x89,profile,page]` for pages 0–7.

The frontend layouts are `FRONTEND_ONLY`. The firmware dispatch-table result is `STRONG_STATIC_INFERENCE` that neither operation has a command-specific handler through this dispatcher. A separate handler or another firmware revision remains `UNKNOWN`.

The adjacent `0x8c` dispatch entry reaches `0x1951e`, which builds a 64-byte response using request bytes and a pointer calculation. It is a possible follow-up for data/page retrieval, but there is insufficient evidence to call it a keymap or per-key RGB handler.

### 3.3 Lighting command handlers are not Fn shortcut handlers

- `0x08 → 0x19afe` writes the full profile-indexed lighting parameter structure and calls `0x146f8` at `0x19bac`.
- `0x87 → 0x195ca` reads that structure and prepares a 64-byte response.
- `0x07 → 0x19bb6` and `0x86 → 0x19614` operate on a separate compact structure.

These are command-level configuration paths. No direct reference ties them to matrix key events, the Fn key, Backspace, or a shortcut lookup table.

## 4. Matrix, input and rendering candidates

Several routines are relevant to a future static key-input investigation but cannot currently be assigned Fn semantics:

| File offset | Observed behavior | Assessment |
|---:|---|---|
| `0x1c5b6` | Long event/transfer-state routine; allocates a 64-byte local buffer, accesses peripheral-like state and eventually contains a dispatcher call through `0x1c7b0` | `STRONG_STATIC_INFERENCE` of a transport/event-processing role; exact HID and key-event semantics `UNKNOWN` |
| `0x1c7b0` | Direct call to application dispatcher `0x19c5e` | `CONFIRMED_STATIC` direct call; source event and command-buffer origin `UNKNOWN` |
| `0x146f8` | Iterates a 6×15 grid, calls `0x164a6`, then copies a structured buffer via `0x16c64` | `STRONG_STATIC_INFERENCE` of grid/record processing; no proven key matrix or Fn role |
| `0x164a6` | Scales components and writes indexed records using a `0x54` stride | `STRONG_STATIC_INFERENCE` of component/record transformation; no proven physical LED/key semantics |
| `0x164f6` | Scales and stores three component bytes plus an index in a structured record | `STRONG_STATIC_INFERENCE` of component/record transformation; Fn role `UNKNOWN` |
| `0x16566` | Handles record selector cases `0..3` and modifies adjacent bytes with bounded arithmetic | `WEAK_INFERENCE` of a generic record/state helper; Fn role `UNKNOWN` |
| `0x16c64` | Copies three bytes for 21 entries across eight iterations | `STRONG_STATIC_INFERENCE` of structured matrix/frame copy; exact data identity `UNKNOWN` |
| `0x16d2c` | Loops 12 entries and calls `0x164f6`; direct callsites at `0x17352` and `0x19af2` | `CONFIRMED_STATIC` routine/callsite facts; `0x19af2` is immediately before the `0x19afe` handler and is not proven to belong to the full lighting writer |

These routines contain no recovered string or direct branch identifying `Fn`, `Backspace`, an RGB shortcut, or a USB-mode shortcut. Their presence must not be used to infer the physical key matrix or key layout.

## 5. Fn state and shortcut dispatch evidence

The firmware string scan recovered general strings such as `Gaming Keyboard`, `HID Config`, `HID Interface`, and Yichip SDK paths/APIs. It did not recover readable Fn, keymap, RGB or named shortcut strings. The binary is stripped/raw, so this negative string result is weak and does not prove that the functionality is absent.

No defensible static evidence currently identifies:

- the physical matrix scan routine and row/column mapping;
- the Fn key's matrix/HID code and state bit;
- the function-layer selector or keymap override table;
- the `Fn + Backspace` key-combination comparison;
- RGB shortcut handlers or their call chain into the lighting state;
- the USB-mode switching shortcut handler;
- whether the working USB-mode shortcut and failing RGB shortcuts share a common dispatch stage.

Classification: `UNKNOWN` for the complete Fn path and all named shortcut handlers.

## 6. What the symptom pattern does and does not imply

The working USB-mode shortcut is evidence only that at least one input/mode path remains functional. It does not prove that the Fn-layer lookup, RGB command handler, LED state machine or LED output path is functional.

The fact that RGB and several Fn shortcuts reportedly fail over both USB and 2.4 GHz makes a purely USB-host-side failure less likely, but it does not isolate the shared keyboard MCU, receiver, firmware, keymap state, LED power rail or LED hardware.

No static result in this phase proves a single common cause for the Fn and RGB symptoms. The prior ranked hypotheses remain hypotheses, not diagnoses:

1. Fn-layer/keymap or input-dispatch abnormality.
2. Lighting state/configuration issue.
3. Firmware revision/configuration mismatch or corruption.
4. LED initialization/power-management problem.
5. LED driver, connector, chain or power-path fault.
6. Host/transport issue (deprioritized by the cross-transport symptom, not ruled out).

## 7. Required next static investigation

1. Recover the USB HID report descriptor contents and identify keyboard input report layout from the descriptor's actual initialized data, not from generic HID assumptions.
2. Trace the input-report receive buffer from its USB/2.4 GHz ingress through the matrix/key event parser.
3. Identify the Fn key state and layer-selection data-flow; search for callsites that branch on that state and map to key actions.
4. Trace the working USB-mode shortcut and candidate RGB shortcuts separately, then compare their shared ancestors only when the call graph proves a relationship.
5. Trace any keymap storage/load function and determine whether the `0x89` frontend command is unsupported by this image or handled through a separate route.
6. Connect any RGB shortcut handler to the already identified `0x08`/`0x87` lighting state only if actual stores/calls establish the edge.

No hardware test is part of these next steps. The exact PCB and installed firmware identity remain unconfirmed, and the supplied V504 image must not be flashed or executed.

## 8. Safety statement

No key combination was tested. No HID device was enumerated or opened. No Feature report, Output report, interrupt transfer or USB control transfer was sent or received. No keyboard setting, keymap, profile, pairing state or persistent data was changed. Firmware was not executed, patched, flashed or updated. Hardware testing remains unauthorized.
