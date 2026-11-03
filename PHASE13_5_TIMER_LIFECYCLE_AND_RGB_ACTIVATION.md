# Phase 13.5 — Timer Lifecycle and RGB Activation

## Executive result

The timer path and RGB output wrapper are distinct in the recovered dispatch table.

- Timer setup `0x173cc` enables candidate NVIC index 5, configures timer selector 0 at candidate register base `0x000f8b00`, and uses scalar `500000`. Table slot 5 contains `0x01010619`, mapping to `0x10418`, which polls timer selector 0. This is a `STRONG_STATIC_INFERENCE` of a timer-IRQ-5 service path.
- The RGB output wrapper `0x103b0` is in table slot 14 (`0x010105b1`). Candidate NVIC index 14 is enabled by `0x10240`, separately from the timer setup. The IRQ14-to-wrapper chain is a `STRONG_STATIC_INFERENCE` pending proof of the hardware vector-entry linkage.
- No timer frequency, RGB refresh interval, or IRQ14 cadence can be calculated from the available evidence.

## 1. Timer configuration and enablement

At file offset `0x173cc`:

```asm
173cc: push {r0, r1, r2, r3, r4, lr}
173ce: movs r0, #5
173d0: bl   0x1d304             ; NVIC ISER bit 5
173d4: movs r2, #9
173d6: movs r1, #2
173d8: movs r0, #0
173da: bl   0x19f04
173de: movs r2, #8
173e0: movs r1, #1
173e2: movs r0, #0
173e4: bl   0x19f04
173e8: ldr  r0, [pc, ...]       ; 0x0007a120 = 500000
173ea: str  r0, [sp, #4]
173ec: movs r0, #0
173ee: mov  r1, sp
173f0: str  r0, [sp, #8]
173f2: strb r0, [r1]
173f4: bl   0x1a652             ; selector 0, candidate base 0x000f8b00
173f8: movs r2, #1
173fa: movs r1, #2
173fc: movs r0, #0
173fe: bl   0x1a514
17402: movs r1, #1
17404: movs r0, #0
17406: bl   0x1a92c
1740a: pop  {r0, r1, r2, r3, r4, pc}
```

The helper `0x1a652` selects candidate timer-like base `0x000f8b00` for selector 0 and `0x000f8c00` for selector 1. It divides a `48,000,000` constant by the scalar through `0x1d3b4`; `48,000,000 / 500,000 = 96`. It writes timer-like register offsets including `+0x04`, `+0x08`, `+0x0c`, `+0x0f`, and `+0x1c`. The exact clock source, prescaler, reload, counter direction, interrupt generation and period are not established.

**Classification:** `CONFIRMED_STATIC` for instruction sequence, arguments, constants, arithmetic and observed register offsets; `STRONG_STATIC_INFERENCE` for timer-like peripheral configuration; `UNKNOWN` for actual period/frequency and RGB cadence.

## 2. Timer and table-slot correlation

The table entry at slot 5 (`file+0x100d0`) is raw `0x01010619`; under candidate base `0x01000200`, it resolves to `0x10418`. The body at `0x10418` calls `0x1a582(0)` and subsequent selector-0 timer helpers. The entry at slot 6 resolves to `0x103d0`, which similarly polls selector 1.

This gives two independent matching clues for slot 5: `0x173cc` configures selector 0 and enables index 5; slot 5 points to a routine that polls selector 0. The strongest static model is therefore:

```text
timer selector 0 configuration (0x173cc)
  -> enable NVIC index 5
  -> indexed slot 5 (offset +0x14)
  -> 0x10418 (selector-0 status service)
```

The table's exact NVIC hardware entry mechanism remains unproven, so the final IRQ edge is `STRONG_STATIC_INFERENCE`, not `CONFIRMED_STATIC`.

Crucially, slot 5 does **not** point to `0x103b0`. The timer path is not the recovered direct route to the RGB wrapper.

## 3. Separate IRQ14 output-wrapper route

Slot 14 (`file+0x100f4`) contains raw `0x010105b1`, which maps to `0x103b0`. The init block calls `0x10240` after `0x13400`; `0x10240` calls `0x1d304` with `r0=14`, enabling candidate NVIC index 14. The block also assigns priority index 14 through `0x1d2b8`.

```text
candidate application init 0x1027e
  -> timer setup 0x173cc / enable index 5
  -> output initialization 0x13400
  -> helper 0x10240 / enable index 14
  -> candidate IRQ14 stub at 0x10054
  -> table slot 14
  -> 0x103b0
  -> 0x135de
```

The first, second, third and fifth code edges are statically visible; the hardware vector-to-stub link and actual event/cadence for IRQ14 remain inferred/unknown. The numerical match and static table pointer make the route a `STRONG_STATIC_INFERENCE`.

`0x103b0` itself executes:

```asm
103b0: push {r4, lr}
103b2: movs r1, #0
103b4: movs r0, #1
103b6: bl   0x1a9b2
103ba: bl   0x135de
103be: movs r1, #1
103c0: mov  r0, r1
103c2: bl   0x1a9b2
103c6: pop  {r4, pc}
```

It clears bit 4 at observed address `0x000f0c48`, calls `0x135de`, then sets the bit again. That bit is not proven to be an NVIC mask. `0x135de` checks state byte `0x20348`; when it equals `0xfd`, normal frame preparation/transmission is skipped. Otherwise it calls `0x13526` and performs the 16-word output loop through `0x134f2`, followed by control operations including `0x133d2`.

## 4. Timer reconfiguration lifecycle

Direct callers of `0x173cc`:

| Callsite | Context | Static consequence |
|---|---|---|
| `0x102de` | Candidate application init | Enables index 5 and configures selector 0 before `0x13400` |
| `0x10974` | Inside `0x1094c` | Reconfigures the same timer path during a later state-dependent routine |
| `0x11360` | Inside `0x11328` | Reconfigures the same timer path when main loop dispatches state `0xfd` |

`0x1094c` is called from `0x10aa6`, `0x10e04`, and `0x1bfca`; their ownership (USB, wireless, or other) is not established. Repeated calls to `0x173cc` show reconfiguration and re-enable of index 5, but do not establish a refresh rate or why each transition occurs.

## 5. IRQ14 enable/disable lifecycle

`0x10240` enables index 14 through `0x1d304`. Direct callers include:

- `0x102f8` in the candidate startup block, after `0x13400` at `0x102ee`.
- `0x1168e`, `0x11704`, and `0x11754` in later transition paths.

`0x1d312` writes `1 << index` to `0xe000e180`, the NVIC Interrupt Clear-Enable register. Index 14 is disabled at:

- `0x115e0` inside routine `0x115d8` (which is called from `0x11884`).
- `0x11796` inside routine `0x1176c` (which is called from `0x11810`).

This proves IRQ14 is not merely enabled once; the firmware has state-dependent disable and re-enable paths. The enable helper's callsites show reinitialization after output setup in several transition routines. The disable callsites do not establish a table rewrite: the table remains static, and the operation is an NVIC enable-state change.

No evidence shows the table pointer or slot 14 being changed. No explicit unregister/replace function was found. The IRQ14 event source and exact periods when enabled are unknown.

## 6. Output initialization and ordering

Candidate init block `0x1027e` is reached from the reset-like trampoline under the candidate mapping:

```text
0x10200: set SP = 0x00030000
  -> 0x1d488 startup record walker
  -> 0x10208
  -> BX raw pointer 0x0101047f
  -> candidate file offset 0x1027e
```

The init block orders operations as follows:

1. NVIC priority setup for 0, 5, 14 and the system-exception path.
2. Timer setup at `0x102de` (`0x173cc`), enabling index 5.
3. Output initialization at `0x102ee` (`0x13400`), including a full `0x2a0`-byte clear of candidate buffer `0x20a78` and output-control register-like writes.
4. IRQ14 enable via `0x10240` at `0x102f8`.
5. IRQ15 enable via `0x10214` at `0x102fe`.
6. Main polling loop at `0x10308`.

Thus the static startup ordering places IRQ14 enable after output initialization. The reset-to-init mapping is strongly corroborated by the same image-base relationship used by the table. However, the `0x1d488` startup record contents are not fully resolved; any boot-stage dependency outside the image remains possible.

## 7. State `0xfd` and reinitialization

The main loop at `0x10308` reads byte `0x20348`. For `0xfd` it calls `0x11328`. In that routine:

- A pending flag at `[state+0x16]` can cause the full 672-byte candidate buffer at `0x20a78` to be cleared.
- The routine calls `0x173cc` at `0x11360`, reconfiguring selector 0 and re-enabling IRQ5.
- It changes other state bytes and calls additional output/control routines.
- A later conditional branch calls `0x164a6` and `0x16c64` if `[state+0x1c]` is set.

Separately, `0x135de` skips its normal frame path whenever `0x20348 == 0xfd`. The state therefore gates output at the software consumer and triggers timer/buffer transition work. There is no evidence that `0x11328` changes the static table or directly disables IRQ14; IRQ14 disable operations were found in other routines (`0x115d8`, `0x1176c`). Do not equate `0xfd` with sleep/power state or infer complete restoration from timer reconfiguration alone.

## 8. Alternate schedulers and timing

The main loop is cooperative and state-driven, but does not call `0x103b0`. `0x1034c` and `0x111dc` resemble a software-counter path but have no recovered registration/cadence. The startup callback walkers use separate 16-byte records and have not been shown to register the output wrapper.

The table resolves an interrupt-side output wrapper path, but no proven source clock, timer divider/reload interpretation, IRQ14 event source, or event frequency is available. The quotient 96 is arithmetic only. No RGB refresh rate or frame period is reported.

## 9. RGB activation lifecycle graph

```text
Reset-like entry 0x10200
  -> startup walker 0x1d488
  -> candidate app init 0x1027e
       -> priority indices 0, 5, 14
       -> 0x173cc: timer selector 0 + enable index 5
       -> 0x13400: clear/init candidate output subsystem
       -> 0x10240: enable index 14
       -> 0x10214: enable index 15
       -> main loop 0x10308

Candidate IRQ5 path:
  -> stub offset 0x14 -> table slot 5 -> 0x10418 (timer selector 0)

Candidate IRQ14 path:
  -> stub offset 0x38 -> table slot 14 -> 0x103b0
       -> 0x135de
            -> if state != 0xfd: 0x13526 -> 16 x 0x134f2 -> 0x133d2/control
            -> if state == 0xfd: skip normal frame transmission

Transition paths:
  -> disable IRQ14 through 0x1d312 at 0x115e0 / 0x11796
  -> selected transitions later call 0x10240 to enable IRQ14 again
  -> state 0xfd calls 0x11328, clears buffer conditionally and reconfigures IRQ5/timer
```

The timer-to-IRQ5 service and IRQ14-to-output-wrapper paths are separate. This is the central Phase 13.5 correction.

## Related reports

- [Handler table layout and initialization](PHASE13_5_HANDLER_TABLE_LAYOUT_AND_INITIALIZATION.md)
- [Interrupt dispatch and callback resolution](PHASE13_5_INTERRUPT_DISPATCH_AND_CALLBACK_RESOLUTION.md)
- [Findings and next steps](PHASE13_5_FINDINGS_AND_NEXT_STEPS.md)
- [Phase 13.4 runtime gating](PHASE13_4_RGB_RUNTIME_GATING.md)
- [Phase 13.3 renderer dataflow](PHASE13_3_RGB_RENDERER_DATAFLOW.md)
- [Phase 13.3 LED output protocol analysis](PHASE13_3_LED_OUTPUT_PROTOCOL_ANALYSIS.md)
