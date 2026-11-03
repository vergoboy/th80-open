# Phase 13.6 — IRQ14 Lifecycle and RGB Trigger

## Executive conclusion

The application firmware statically configures Timer1 channel 1, enables NVIC external interrupt index 14, and stores `0x103b0` in application handler-table slot 14. The IRQ14 wrapper disables Timer1, invokes `0x135de`, and re-enables Timer1. This establishes a strong static Timer1-to-RGB-output callback path in application mode.

It does not establish that an interrupt physically arrived, that the output routine always sends a frame, or that physical LEDs receive the signal. State `0xfd` causes `0x135de` to skip its normal output path. Other transition routines also disable both Timer1 and NVIC14, but their exact relationship to state `0xfd` is not proven.

## 1. Application startup ordering

The application reset stub at file offset `0x10000` loads reset pointer `0x01010401`, which resolves to `0x10200`. The application reset routine sets SP to `0x00030000`, calls startup callback walker `0x1d488`, and transfers to application initialization at `0x1027e` under the candidate image mapping.

The relevant application initialization order is:

```text
0x1027e  application initialization
  ...    NVIC priority setup, including index 14
0x102ee  BL 0x13400       ; output subsystem initialization
0x102f8  BL 0x10240       ; configure Timer1 and enable NVIC14
0x102fe  BL 0x10214       ; configure Timer2 and enable NVIC15
0x10308  enter main polling loop
```

The Timer1 setup is therefore performed after the candidate output initialization in this startup path. No identified direct caller of `0x10240` precedes `0x13400`.

The same ordering is visible in three transition/reinitialization sequences:

| Timer1 init callsite | Preceding output init | Timer2 setup | Timer1 setup |
|---|---|---|---|
| `0x102f8` | `0x102ee -> 0x13400` | `0x102fe -> 0x10214` follows | `0x10240` |
| `0x1168e` | `0x1167e -> 0x13400` | `0x11684 -> 0x10214` | `0x10240` |
| `0x11704` | `0x116f4 -> 0x13400` | `0x116fa -> 0x10214` | `0x10240` |
| `0x11754` | `0x1173e -> 0x13400` | `0x1174a -> 0x10214` | `0x10240` |

**Classification:** `CONFIRMED_STATIC` for the call order at these sites. `STRONG_STATIC_INFERENCE` that these known application paths do not enable the Timer1 callback before output initialization. The boot-ROM/hardware reset path remains outside the image.

## 2. Timer1/NVIC14 initialization

Function `0x10240` configures channel 1 with reload value `0x226e`, writes PCNT1 at `0x000f0c08`, sets the channel's timer-mode/auto-reload control bits, enables channel 1, and calls `0x1d304(14)`.

`0x1d304` sets the corresponding bit in the NVIC-compatible set-enable register at `0xe000e100`. The priority helper `0x1d2b8` is also called with index 14 during application initialization.

The exact Timer1 control-register address remains qualified: the firmware manipulates the channel-1 field at `0x000f0c48`, while the candidate YC3121-L datasheet's `TIM_CTRL` address conflicts with its own PCNT/NCNT address table. The per-channel field behavior and helper arguments are static facts; the hardware naming is a strong inference.

## 3. IRQ14 handler execution and timer self-gating

The application vector stub for external IRQ14 at file offset `0x10054` executes:

```asm
10054: movs r0, #0x38
10056: b    0x1009c
```

The application dispatcher at `0x1009c` loads the pointer at `0x010102bc + r0` and branches through it. Since `0x38 / 4 = 14`, it selects slot 14. The slot contains raw `0x010105b1`, resolving to `0x103b0`.

The boot-stage IRQ14 stub at file offset `0x54` supplies the same `0x38` offset to `0x00ce`. In application mode (mode register `0x000f8564 != 2`), that dispatcher forwards to application dispatcher `0x1009c`. In boot mode (value 2), it uses the separate boot handler table at `0x0100032c`; its slot 14 is the default trap, not `0x103b0`.

The wrapper at `0x103b0` is:

```asm
103b0: push {r4, lr}
103b2: movs r1, #0
103b4: movs r0, #1
103b6: bl   0x1a9b2       ; disable timer channel 1
103ba: bl   0x135de       ; output-frame path / gate
103be: movs r1, #1
103c0: mov  r0, r1
103c2: bl   0x1a9b2       ; re-enable timer channel 1
103c6: pop  {r4, pc}
```

`0x1a9b2` is a per-channel timer control helper. It is not the NVIC enable/disable helper and does not modify the CPU's global interrupt mask. The wrapper's stop/output/restart sequence is consistent with avoiding timer retrigger during output work and restarting the countdown. No `DSB`, `DMB`, `ISB`, `MRS IPSR`, or NVIC pending-clear operation is present in this wrapper.

**Classification:** `CONFIRMED_STATIC` for the table entry, dispatch code, helper arguments and call order. `STRONG_STATIC_INFERENCE` that a Timer1 event on NVIC14 invokes the wrapper in application mode.

## 4. NVIC enable/disable lifecycle

The firmware has distinct source and NVIC controls:

- Timer1 source/channel enable: helper `0x1a9b2(1,1)`; disable: `0x1a9b2(1,0)`.
- NVIC IRQ14 enable: helper `0x1d304(14)`, writing the bit to `0xe000e100`.
- NVIC IRQ14 disable: helper `0x1d312(14)`, writing the bit to `0xe000e180`.
- Output suppression: state byte `0x00020348` checked by `0x135de`.

These are not interchangeable operations.

### 4.1 IRQ14 disable paths

Two direct callsites disable NVIC index 14:

| File offset | Operation | Context |
|---|---|---|
| `0x115e0` | `0x1d312(14)` | Called from transition path `0x11884` |
| `0x11796` | `0x1d312(14)` | Called from transition path `0x1176c` |

Both paths also call `0x1a9b2(1,0)` to disable Timer1. They then execute other peripheral/output transition work and configure Timer2. They do not rewrite the application handler table.

The corresponding Timer1/NVIC14 reinitialization occurs through `0x10240` at startup and at `0x1168e`, `0x11704`, and `0x11754`. Those transition sequences initialize output through `0x13400` before invoking `0x10240`.

The exact high-level names of these state transitions are not established. They should not be labeled sleep, wake, USB or wireless transitions without further evidence.

### 4.2 Pending state and acknowledgement

No literal/reference to NVIC ICPR0 at `0xe000e280` was found, and no explicit pending-clear write is present in the identified IRQ14 paths. The handler disables and re-enables Timer1; the candidate datasheet says enabling a timer loads its current counter from PCNT. That is consistent with restarting the timer, but the datasheet does not specify whether this also clears every peripheral or NVIC pending condition.

Therefore:

- Timer1 source restart is `CONFIRMED_STATIC` as firmware behavior.
- Peripheral interrupt acknowledgement by restart is `WEAK_INFERENCE`.
- NVIC pending-bit clearing is `UNKNOWN`.
- Whether enabling NVIC14 can immediately deliver an already-pending request is `UNKNOWN`.

## 5. `0x1a9b2` is not an interrupt lock

The helper computes a channel nibble offset and sets/clears bit 0 in a packed timer-control word. For channel 1 it modifies mask `0x10` at `0x000f0c48`. The same helper is used by timer initialization and several mode/transition paths. It does not access `0xe000e100`, `0xe000e180`, `0xe000e280`, `PRIMASK`, or `BASEPRI`.

It is therefore a timer-channel enable/disable helper under the candidate timer-register interpretation, not a global interrupt lock. No memory barrier or synchronization primitive is performed by the wrapper around the output call.

**Classification:** `CONFIRMED_STATIC` for the read-modify-write and target address. `STRONG_STATIC_INFERENCE` for timer-channel enable semantics. `UNKNOWN` for the physical meaning of every control bit on the exact board MCU.

## 6. State `0xfd` and output suppression

The main loop at `0x10308` checks state byte `0x00020348`; value `0xfd` dispatches to `0x11328`. The output routine `0x135de` also checks the same byte and skips its normal frame-transmission path when the value is `0xfd`.

The `0x11328` path does not directly call `0x1d312(14)` or `0x1a9b2(1,0)`. Therefore the state `0xfd` branch itself is not proof that Timer1 or NVIC14 is disabled. Separate transition paths do disable both, but their predicates have not been tied conclusively to the `0xfd` state byte.

If Timer1 remains enabled while state is `0xfd`, the interrupt wrapper can still be entered, disable Timer1, call `0x135de`, have that routine skip the frame, and re-enable Timer1. This is a `STRONG_STATIC_INFERENCE` conditional on continued Timer1/NVIC enablement. Whether the transition state machine instead disables the source in a particular runtime state remains `UNKNOWN`.

## 7. Output path and physical limitation

When state is not `0xfd`, `0x135de` proceeds through the existing renderer/output path documented in Phase 13.3, including `0x13526` and the candidate peripheral writes. This confirms a firmware-level output attempt, not the electrical success of the output pin or receipt by the LED chain.

| Claim | Classification | Reason |
|---|---|---|
| App table slot 14 contains pointer to `0x103b0` | `CONFIRMED_STATIC` | Raw pointer and candidate mapping |
| App dispatcher selects slot 14 for offset `0x38` | `CONFIRMED_STATIC` | Exact stub immediate and indexed load |
| Hardware NVIC IRQ14 enters the matching stub | `STRONG_STATIC_INFERENCE` | NVIC index and vector-stub index agree; hardware vector base remains external |
| IRQ14 source is Timer1 | `STRONG_STATIC_INFERENCE` | Timer1 channel setup, NVIC14 enable and wrapper channel-1 gating |
| Wrapper invokes `0x135de` | `CONFIRMED_STATIC` | Direct call in `0x103b0` |
| Normal output path writes the candidate output peripheral | `CONFIRMED_STATIC` | Static control/data flow when state is not `0xfd` |
| Physical LEDs receive a valid frame | `UNKNOWN` | No electrical/hardware observation is authorized or performed |

## Related reports

- [Vector table and IRQ entry](PHASE13_6_VECTOR_TABLE_AND_IRQ_ENTRY.md)
- [IRQ14 peripheral source](PHASE13_6_IRQ14_PERIPHERAL_SOURCE.md)
- [Findings and next steps](PHASE13_6_FINDINGS_AND_NEXT_STEPS.md)
- [Phase 13.5 RGB gating (historical)](PHASE13_5_TIMER_LIFECYCLE_AND_RGB_ACTIVATION.md)
