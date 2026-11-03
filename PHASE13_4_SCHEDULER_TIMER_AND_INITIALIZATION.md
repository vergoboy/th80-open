# Phase 13.4 — Scheduler, Timer and Initialization

## Scope

This report reconstructs only what is visible in the raw firmware disassembly. The firmware load address, full memory map, timer peripheral identity and runtime contents of callback/vector tables are not established. No frame rate or timer period is inferred without peripheral semantics.

## 1. Reset and startup sequence

### 1.1 Early entry

At `0x10200`:

```asm
10200: ldr r0, [pc, #0x0c]   ; literal at 0x10210
10202: mov sp, r0
10204: bl  0x1d488
10208: ldr r0, [pc, #0x00]   ; literal at 0x1020c
1020a: bx  r0
```

The stack literal at `0x10210` is `0x00030000`. The branch target literal at `0x1020c` is `0x0101047f`. The exact address-to-file mapping is not established; if the candidate image base is `0x01000000`, the branch enters file-offset-like `0x1047e`; if the relevant segment is based at `0x01010000`, the same literal instead maps to file offset `0x47e`. The analysis of instructions at `0x1047e` below is conditional on the first mapping and must not be treated as a proven reset target. `0x1d488` walks 16-byte runtime records and calls the callback pointer at `+0x0c`, passing the first three words as arguments. Its observed bounds are `0x0101c8fb` and `0x0101e8fb` (a 0x2000-byte range); the runtime contents and mapping remain unresolved.

At `0x1047e`, the code writes `0x0001c200` (115,200) to the stack configuration structure, zeroes the other shown fields, calls `0x1a652` with selector zero, then executes `pop {r0, r1, r2, r3, r4, pc}`. The popped PC comes from runtime stack memory at `0x00030014` under the observed stack value; its value is not present in the raw firmware. Consequently the reset stub's next control-flow edge after `0x1047e` is unknown. The helper derives `48,000,000 / 115,200 = 416` for its timer configuration, but the resulting interval is not known.

There is a second early stub at `0x200`: it sets SP from the same literal `0x00030000`, calls `0x2acc`, then branches through literal `0x01000899`. It walks a separate 16-byte runtime callback range with bounds `0x010034ec` to `0x0100350c` (two records). The ownership/relationship of the two entry stubs is unknown; do not assume they are two cores or one linear startup path.

**Classification:** `CONFIRMED_STATIC` for both entry stubs, literal operands, table-walker mechanics and the stack-pop sequence; `UNKNOWN` for runtime table contents, stack-supplied return PC, core ownership and the ultimate boot-entry semantics.

### 1.2 Candidate application initialization block

The code beginning at `0x1027e` performs the following visible operations before entering the polling loop at `0x10308`. No direct `BL` caller or static absolute pointer to `0x1027e` was recovered (the two candidate absolute pointer forms tested for the two simple segment bases were also absent). The reset stub at `0x10200` branches to literal `0x0101047f`; only under the candidate base `0x01000000` does this resolve to file offset `0x1047e`. If that mapping holds, the code at `0x1047e` returns through a PC popped from runtime stack memory; the stack-supplied PC was not recovered. Under the alternative segment mapping the literal resolves elsewhere. Therefore the edge from either reset stub to this block is plausible but unproven.

1. Configures a peripheral through `0x19f04` and invokes `0x1b362`, `0x1026a`, `0x1afbe`, and `0x1b562` helpers.
2. Sets priorities through `0x1d2b8` for indices `0`, `5`, `14`, and `-1` (the negative value takes the system-exception path in the helper).
3. Calls `0x18a00`, `0x18a3c`, `0x10554`, `0x11920`, `0x11308`, and `0x11318`.
4. Calls `0x173cc` at `0x102de` to configure the timer/peripheral path and enable interrupt index 5.
5. Initializes a global byte at the state block, calls `0x1122c`, `0x10750`, and `0x13400`.
6. Continues with `0x1b1bc`, `0x10240`, and `0x10214`, clears/sets further state bytes, and falls into the main polling loop at `0x10308`.

The exact hardware meaning of every helper in this sequence is not assumed. The ordering of the direct calls is `CONFIRMED_STATIC`; the block’s entry from either reset stub is `UNKNOWN`.

## 2. Timer and NVIC evidence

### 2.1 NVIC priority and enable helpers

`0x1d2b8` accesses literals corresponding to:

- `0xe000ed00` for the system-control/exception-priority path when the supplied index is negative.
- `0xe000e400` for external interrupt priority bytes when the supplied index is nonnegative.

Within the candidate `0x1027e` application-init block, the code supplies priority values for indices 0, 5, 14 and -1. This is consistent with Cortex-M NVIC/SCB address layout.

`0x1d304` loads `0xe000e100`, computes `1 << r0`, ORs it into the word at that address, and stores it. This is the NVIC Interrupt Set-Enable register address. The call at `0x173d0` supplies `r0=5`, so interrupt index 5 is enabled.

**Classification:** `CONFIRMED_STATIC` for the addresses, bit operation and index arguments; `STRONG_STATIC_INFERENCE` for the standard NVIC/SCB interpretation based on the fixed Cortex-M register addresses.

### 2.2 Timer configuration routine `0x173cc`

The routine is:

```asm
173cc: push {r0, r1, r2, r3, r4, lr}
173ce: movs r0, #5
173d0: bl   0x1d304          ; enable NVIC interrupt index 5
173d4: movs r2, #9
173d6: movs r1, #2
173d8: movs r0, #0
173da: bl   0x19f04
173de: movs r2, #8
173e0: movs r1, #1
173e2: movs r0, #0
173e4: bl   0x19f04
173e8: ldr r0, [pc, ...]     ; literal 0x0007a120 = 500000
173ea: str r0, [sp, #4]
173ec: movs r0, #0
173ee: mov r1, sp
173f0: str r0, [sp, #8]
173f2: strb r0, [r1]
173f4: bl   0x1a652
173f8: movs r2, #1
173fa: movs r1, #2
173fc: movs r0, #0
173fe: bl   0x1a514
17402: movs r1, #1
17404: movs r0, #0
17406: bl   0x1a92c
1740a: pop {r0, r1, r2, r3, r4, pc}
```

The `0x1a652` configuration helper selects the literal base `0x000f8b00` when its first argument is zero (and `0x000f8c00` for selector one). It consumes the configuration scalar at structure offset `+4`, validates several byte fields, divides the literal `48,000,000` (`0x02dc6c00`) by that scalar through the software division routine `0x1d3b4`, shifts the quotient into the upper half of a configuration word, and writes timer-like registers including base offsets `+0x04`, `+0x08`, `+0x0c`, `+0x0f`, and `+0x1c`.

For the call at `0x102de` in the candidate application-init block, the scalar is `500000`; the integer division result is `96`. The code therefore derives a hardware configuration value from `48,000,000 / 500,000 = 96`. The exact timer clock, prescaler semantics, counter direction, reload interpretation and interrupt period are not established. The literal must not be reported as a 500 ms interval, 500 kHz refresh rate or 96 Hz frame rate without the peripheral register specification.

`0x1a514` and `0x1a92c` then update additional fields in the selected timer-like register block. The candidate application-init block passes selector zero to both.

**Classification:** `CONFIRMED_STATIC` for calls, arguments, constants, arithmetic and register offsets; `STRONG_STATIC_INFERENCE` that `0x000f8b00` is a timer peripheral block; `UNKNOWN` for the timer identity and resulting period/frequency.

### 2.3 Timer reconfiguration callsites

`0x173cc` has three direct callers:

| Caller | Context | Evidence |
|---|---|---|
| `0x102de` | Candidate `0x1027e` init block, before its `0x13400` output initialization call | `CONFIRMED_STATIC` |
| `0x10974` | Called inside `0x1094c` after its state-dependent branch and setup operations | `CONFIRMED_STATIC` |
| `0x11360` | Called inside `0x11328`, which the main loop dispatches for state byte `0xfd` | `CONFIRMED_STATIC` |

`0x1094c` itself is called from `0x10aa6`, `0x10e04`, and `0x1bfca`. Those callers show multiple runtime contexts, but they do not by themselves identify USB, wireless or RGB ownership.

The repeated calls show that timer configuration occurs in the candidate init block and on later state transitions. Whether the candidate block is reached from reset remains unresolved. They do not prove that the timer invokes `0x103b0`.

## 3. Main-loop scheduler evidence

The candidate application-init block at `0x1027e` reaches a cooperative polling loop at `0x10308`:

```asm
10308: bl 0x1b74c
       ... read byte [0x20340 + 8]
       ... branch by state value
       ... call one of 0x11328 / 0x114f2 / 0x10b74 / 0x10eee
10346: bl 0x1076c
1034a: b  0x10308
```

This is a persistent polling loop. It repeatedly calls `0x1b74c`, dispatches according to the state byte at `0x20348`, calls `0x1076c`, and branches back. No `BLX` occurs in the inspected loop body, and there is no direct call to `0x103b0`, `0x1034c`, `0x103c8` or `0x103d0` from this loop.

State dispatch includes:

- Values `0`, `1`, `2`, and `5`: `0x10eee`.
- Value `6`: `0x10b74`.
- Values `3`, `4`, `7..0xfc`, and `0xff`: `0x1076c` path.
- `0xfd`: `0x11328`.
- `0xfe`: `0x114f2`.
- Other values: the path through `0x1076c`.

**Classification:** `CONFIRMED_STATIC` for the polling and state dispatch. It is not evidence of periodic RGB rendering.

## 4. Software-counter candidate

`0x1034c` calls `0x111dc` once, then increments three bytes in the `0x20340` state block. `0x111dc` updates a countdown-like structure using fields at offsets including `+4`, `+8`, and `+0x10`. However:

- `0x1034c` has no recovered direct caller.
- `0x111dc` has only the recovered direct call from `0x1037e`.
- No period, tick source or scheduler registration for this pair was found.

The nearby routines `0x103d0` and `0x10418` separately poll timer selectors 1 and 0 through `0x1a582`, then conditionally update timer control/status and counter fields. Neither has a recovered direct caller. Therefore these are timer-status/counter candidates, not proof of the RGB refresh scheduler.

**Classification:** `CONFIRMED_STATIC` for the counter/status operations and direct calls; `UNKNOWN` for their invocation cadence and RGB relationship.

## 5. Output initialization ordering

`0x13400` is called at `0x102ee` within the candidate application-init block, after that block’s call to `0x173cc` and before its main polling loop. This is local code order only; the block’s reset reachability is unresolved. Its visible behavior includes:

- Calls `0x1d45e` with length `0x2a0` to clear the full 672-byte region based at the literal at `0x1377c` (the Phase 13.3 candidate `0x20a78`).
- Reads configuration halfwords from the structure based at `0x20340` and calls `0x19f04` with peripheral selector 2.
- Writes `0x3e` to several bytes in the register-like region based at `0x000f8700`.
- Calls `0x1a9ac`, `0x133d2`, `0x1a9b2`, `0x13398`, and related output-control routines.
- Resets an output-related byte through the pointer literal at `0x13798`.

`0x13400` is also called at `0x1096a`, `0x1167e`, `0x116f4`, and `0x1173e`, establishing additional runtime callsites independent of the unresolved reset-to-`0x1027e` edge. The exact peripheral and pin meanings remain unknown.

**Classification:** `CONFIRMED_STATIC` for the clear, register stores and calls; `STRONG_STATIC_INFERENCE` that `0x13400` initializes the candidate output subsystem; `UNKNOWN` for physical LED enable and board connection.

## 6. First renderer invocation and runtime order

The following two early entry stubs are independently visible:

```text
0x200:
  set SP = 0x00030000
  -> 0x2acc callback-table walker (runtime records unresolved)
  -> BX 0x01000899 (target mapping/ownership unresolved)

0x10200:
  set SP = 0x00030000
  -> 0x1d488 callback-table walker (runtime records unresolved)
  -> BX 0x0101047f
  -> if base=0x01000000: file offset 0x1047e configures timer selector 0 with scalar 115200
  -> if that mapping holds: POP PC from runtime stack (value unknown)
  -> under alternate segment mapping, target may instead be file offset 0x47e; unresolved
```

Separately, the code block at `0x1027e` performs this internally ordered initialization before its polling loop:

```text
0x1027e candidate app-init block
  -> NVIC priority setup
  -> 0x173cc timer/peripheral configuration; NVIC IRQ 5 enabled
  -> 0x13400 candidate output initialization; 0x20a78 cleared
  -> main polling loop at 0x10308
```

The edge from the reset trampoline/stack-supplied PC to `0x1027e` is not proven. Do not collapse the two reset stubs or assume the 0x1027e block is entered immediately after 0x1047e.

The following runtime edge remains unresolved:

```text
NVIC IRQ 5 / vector dispatch slot +0x14
  -> [runtime handler table at observed base 0x010102bc]
  -> unknown handler
  -> ? 0x103b0
```

The firmware has an indirect interrupt-dispatch mechanism, but the runtime table entry and its initializer are not present as a resolvable static pointer in the supplied raw image. The first actual invocation of `0x103b0` therefore cannot be timed or ordered relative to the candidate output-initialization block. The reset/stack handoff into the `0x1027e` block is also unresolved.

## 7. Timing conclusion

No defensible RGB refresh interval or frame rate was recovered. The candidate application-init call to the timer routine contains a scalar `500000` and computes a quotient of `96` from a `48 MHz` literal, but the timer's clock source, divider interpretation, reload behavior and interrupt-to-callback mapping are unknown. The safe conclusion is that the firmware contains a routine which configures a timer-like peripheral and enables NVIC interrupt index 5; execution of the candidate init block and the interrupt handler mapping remain unresolved.

## 8. Remaining blockers

1. Recover the runtime contents and writer/initializer of the vector-handler table at `0x010102bc`.
2. Resolve slot `+0x14` (candidate IRQ 5) to its actual function pointer.
3. Determine whether that handler is `0x103b0`, `0x1034c`, another routine or a ROM/bootloader callback.
4. Identify the timer peripheral at `0x000f8b00`, its clock source and register semantics before deriving an interval.
5. Determine whether the runtime callback table is initialized by the boot ROM or an omitted firmware component.
6. Trace wake/sleep and USB/wireless transition ownership without assuming the observed state byte is a power state.

## Related reports

- [Runtime entry and call graph](PHASE13_4_RGB_RUNTIME_ENTRY_AND_CALLGRAPH.md)
- [Runtime gating](PHASE13_4_RGB_RUNTIME_GATING.md)
- [Findings and next steps](PHASE13_4_FINDINGS_AND_NEXT_STEPS.md)
- [Phase 13.3 renderer dataflow](PHASE13_3_RGB_RENDERER_DATAFLOW.md)
