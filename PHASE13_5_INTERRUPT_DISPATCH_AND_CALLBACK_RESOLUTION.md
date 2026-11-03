# Phase 13.5 — Interrupt Dispatch and Callback Resolution

## Executive result

The supplied firmware contains a 32-slot indexed dispatch mechanism at file offsets `0x1001c..0x100a0`, backed by a static 32-entry table at `0x100bc`. Under candidate image base `0x01000200`, slot 5 resolves to `0x10418`, slot 6 to `0x103d0`, slot 14 to `0x103b0`, and slot 15 to `0x103c8`.

The application-init path statically enables NVIC indices 5, 14, and 15. It sets priorities for indices 0, 5, and 14. The timer configuration routine enables index 5, whose table slot points to the timer-selector-0 status routine `0x10418`. Separately, helper `0x10240` enables index 14, whose table slot points to the output wrapper `0x103b0`. Thus the original claim that timer IRQ 5 directly dispatches to `0x103b0` is rejected under the candidate mapping. The stronger explanation is a separate IRQ/index-14 route to the wrapper.

The numeric correspondence between table slot and NVIC index is compelling but not fully closed: the actual Cortex-M vector-table/VTOR entry that enters each code stub was not recovered. Accordingly, `slot 14 -> 0x103b0` is `CONFIRMED_STATIC` as a table value under the mapping, while `NVIC IRQ14 -> stub14 -> slot14` is `STRONG_STATIC_INFERENCE`, not a fully proven hardware vector chain.

## 1. Indexed stub and dispatch mechanism

The 32 stubs begin at file offset `0x1001c` and are four bytes apart. Each loads a byte offset into `r0` and branches to `0x1009c`. Examples:

```asm
10030: movs r0, #0x14       ; candidate index 5
10032: b    0x1009c

10054: movs r0, #0x38       ; candidate index 14
10056: b    0x1009c

1009c: ldr  r1, [pc, #0x9c] ; raw pointer literal 0x010102bc at 0x1013c
1009e: ldr  r0, [r0, r1]
100a0: bx   r0
```

The `r0` values range from `0x00` through `0x7c`, step four. The dispatcher performs an unvalidated word lookup and tail-branches through the loaded pointer. There is no explicit bound check, pointer validation, or handler replacement logic in this path.

The 32-slot table and all entries are detailed in [handler table layout and initialization](PHASE13_5_HANDLER_TABLE_LAYOUT_AND_INITIALIZATION.md).

**Classification:** `CONFIRMED_STATIC` for stub offsets, indexed load and `BX`; `STRONG_STATIC_INFERENCE` for a secondary interrupt dispatch table.

## 2. NVIC enable and priority setup

### Enable helper `0x1d304`

```asm
1d304: ldr  r1, [pc, ...] ; literal 0xe000e100
1d306: ldr  r2, [r1]
1d308: movs r3, #1
1d30a: lsls r3, r0
1d30c: orrs r2, r3
1d30e: str  r2, [r1]
1d310: bx   lr
```

It sets bit `r0` in the word at `0xe000e100`, the Cortex-M NVIC Interrupt Set-Enable register address. This is a static address/bit-operation fact; the standard NVIC interpretation is `STRONG_STATIC_INFERENCE` from the architectural address.

All direct callers found:

| Callsite | Supplied index | Context |
|---|---:|---|
| `0x1023a` | 15 | Helper `0x10214`; candidate init and later transition callsites |
| `0x10264` | 14 | Helper `0x10240`; candidate init and later transition callsites |
| `0x173d0` | 5 | Timer configuration `0x173cc` |

No other direct `BL` caller of `0x1d304` was found in the disassembly.

### Priority helper `0x1d2b8`

`0x1d2b8` uses the literal `0xe000e400` for nonnegative indices and `0xe000ed00` for the negative/system-exception path. The candidate application-init block at `0x1027e` calls it with:

- `(r0=0, r1=1)` at `0x102a4`.
- `(r0=5, r1=2)` at `0x102ac`.
- `(r0=14, r1=3)` at `0x102b4`.
- `(r0=-1, r1=3)` at `0x102bc`.

The fixed addresses and index arguments are `CONFIRMED_STATIC`; their interpretation as NVIC/SCB priority registers is `STRONG_STATIC_INFERENCE`.

### Disable helper `0x1d312`

```asm
1d312: movs r1, #1
1d314: lsls r1, r0
1d316: ldr  r0, [pc, ...] ; 0xe000e100
1d318: adds r0, #0x80
1d31a: str  r1, [r0]
1d31c: bx   lr
```

This writes the bit mask to `0xe000e180`, the NVIC Interrupt Clear-Enable register address. Thus it disables, rather than pends, the selected interrupt. Direct callers found:

- `0x115e0`: disables index 14 inside routine `0x115d8`.
- `0x11796`: disables index 14 inside routine `0x1176c`.

These callsites prove runtime disable operations for index 14. They do not by themselves prove the exact condition or duration of the disabled interval.

## 3. Table slot resolution

Under candidate base `0x01000200`:

| Slot | Raw table value | Candidate target | Target behavior |
|---:|---:|---:|---|
| 5 (`+0x14`) | `0x01010619` | `0x10418` | Polls timer selector 0 through `0x1a582(0)`; conditional timer control/status operations |
| 6 (`+0x18`) | `0x010105d1` | `0x103d0` | Polls timer selector 1 through `0x1a582(1)` |
| 14 (`+0x38`) | `0x010105b1` | `0x103b0` | Brackets `0x135de` with peripheral bit clear/set |
| 15 (`+0x3c`) | `0x010105c9` | `0x103c8` | Writes one to a state byte at candidate block `0x20340+0x0f` |
| 31 (`+0x7c`) | `0x0101054d` | `0x1034c` | Counter/state work and call to `0x111dc` |

The target halfwords at these candidate file offsets are valid instruction starts: `0x10418: b570`, `0x103d0: b510`, `0x103b0: b510`, `0x103c8: 4935`, and `0x1034c: b510`. All table values have the Thumb bit set.

The common base mapping is independently corroborated by the reset trampoline, init target and startup callback-record pointers. See the mapping evidence in the table-layout report.

## 4. Does index 5 invoke `0x103b0`?

No, not under the recovered table bytes and candidate mapping.

```text
candidate NVIC index 5
  -> stub at 0x10030 supplies offset 0x14
  -> dispatcher 0x1009c
  -> table[5] raw 0x01010619
  -> candidate target 0x10418
```

`0x173cc` enables index 5 and configures timer selector 0. Slot 5 points to `0x10418`, which also polls selector 0. These independent correspondences make `IRQ5 -> 0x10418` a `STRONG_STATIC_INFERENCE` and a plausible timer-service path. The precise timer interrupt event semantics and timing are not known.

Therefore the former hypothesis `IRQ5 -> 0x103b0` is rejected. The slot `+0x14` is not the RGB output wrapper.

## 5. Does index 14 invoke `0x103b0`?

The static evidence is substantially stronger:

```text
candidate NVIC index 14
  -> corresponding indexed stub at 0x10054 supplies offset 0x38
  -> dispatcher 0x1009c
  -> table[14] raw 0x010105b1
  -> candidate target 0x103b0
  -> 0x103b0 calls 0x135de
```

The candidate application-init block calls `0x10240` at `0x102f8`; `0x10240` calls `0x1d304` at `0x10264` with `r0=14`. This occurs after the `0x13400` call at `0x102ee`. The same helper is called in later transition routines at `0x1168e`, `0x11704`, and `0x11754`.

The numerical match among the stub's `0x38` byte offset, table index 14, priority index 14, and NVIC enable/disable helper argument 14 is compelling. It supports `STRONG_STATIC_INFERENCE` that NVIC IRQ14 dispatches to `0x103b0`. The missing proof is the hardware vector-table or boot/ROM trampoline that enters stub `0x10054`; no definitive VTOR write or full Cortex-M vector table was recovered. Also, slot 27's pointer to the reset-like `0x10200` trampoline is atypical for a plain external-IRQ table and is retained as a caveat.

**Conclusion:** `0x103b0` is statically registered in table slot 14 and NVIC index 14 is statically enabled in the candidate init/transition paths. Actual hardware invocation through IRQ14 is a `STRONG_STATIC_INFERENCE`, not `CONFIRMED_STATIC` until the vector-entry linkage is proven.

## 6. Registration, initialization, and dynamic replacement

No callback-registration function or table writer is required to explain the recovered table: its pointer values are embedded in the firmware image. The table is read directly and no writer/copy/fixup was identified. The startup walkers at `0x2acc` and `0x1d488` are separate 16-byte-record callback systems; neither has been shown to register `0x103b0`.

The static IRQ lifecycle is:

1. Reset-like stub `0x10200` sets SP to `0x00030000` and calls `0x1d488`.
2. `0x1d488` walks records and then calls `0x10208`; `0x10208` branches through raw pointer `0x0101047f`, which maps to file offset `0x1027e` under the candidate base.
3. `0x1027e` sets priorities for indices 0, 5 and 14.
4. `0x173cc` enables index 5 and configures timer selector 0.
5. `0x13400` initializes candidate output state.
6. `0x10240` enables index 14; `0x10214` enables index 15.
7. Later paths can disable index 14 through `0x1d312`; other transition paths call `0x10240` to enable it again.

This sequence makes the IRQ14 output-wrapper path eligible only after candidate output initialization in the startup path. Reset reachability of the `0x10200` trampoline is visible, and its target `0x1027e` is strongly mapped; the runtime record contents of the `0x1d488` walker remain an uncertainty. Later transition callsites provide independent enable/reinitialization paths.

## 7. Alternate callback/task systems

- Main polling loop `0x10308` dispatches state and calls `0x1076c`; it has no direct call to `0x103b0` and no `BLX` in the loop body.
- Startup walker `0x2acc` has two file-backed 16-byte records mapping to callbacks `0x2af0` and `0x2b7e`; neither resolves to `0x103b0`.
- Startup walker `0x1d488` has an unresolved record-range mapping; no record is proven to contain `0x103b0`.
- The 102-site `BLX` inventory includes these startup walkers and object/callback dispatches. No additional indirect-call target set was recovered that resolves to `0x103b0`.
- `0x1034c`/`0x111dc` form a software-counter candidate but `0x1034c` has no direct caller and no recovered scheduler registration.

This is not a claim of complete indirect-control-flow coverage. The static slot-14 pointer is the only recovered registration directly explaining `0x103b0`.

## 8. Remaining dependencies

1. The precise CPU vector-table/VTOR or boot-ROM mechanism that enters each `0x1001c..0x10098` stub.
2. The exact peripheral/event source assigned to NVIC index 14.
3. Whether IRQ14 fires periodically, on a peripheral event, or through software pending; no cadence is proven.
4. The runtime contents and semantics of the second startup callback region used by `0x1d488`.
5. Whether any external boot stage remaps or patches the static table before application execution.

## Related reports

- [Handler table layout and initialization](PHASE13_5_HANDLER_TABLE_LAYOUT_AND_INITIALIZATION.md)
- [Timer lifecycle and RGB activation](PHASE13_5_TIMER_LIFECYCLE_AND_RGB_ACTIVATION.md)
- [Findings and next steps](PHASE13_5_FINDINGS_AND_NEXT_STEPS.md)
- [Phase 13.4 runtime entry and call graph](PHASE13_4_RGB_RUNTIME_ENTRY_AND_CALLGRAPH.md)
- [Phase 13.4 scheduler, timer and initialization](PHASE13_4_SCHEDULER_TIMER_AND_INITIALIZATION.md)
