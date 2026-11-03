# Phase 13.5 — Handler Table Layout and Initialization

## Scope and result

Static analysis only. No hardware, HID, USB, firmware execution, firmware modification, or application implementation was performed.

The observed base `0x010102bc` is a direct pointer to a 32-entry, four-byte-stride table embedded at firmware file offset `0x100bc`, under the strongly corroborated but not linker-map-proven mapping `candidate_address = 0x01000200 + file_offset`. The dispatcher reads the table directly; the table is not a pointer variable or descriptor array. It contains Thumb-style code pointers, including slot 5 -> `0x10418`, slot 6 -> `0x103d0`, slot 14 -> `0x103b0`, and slot 15 -> `0x103c8`.

The table is statically present in the supplied firmware image. No runtime writer, copy-to-RAM initializer, relocation pass, or replacement operation for this table was identified. `CONFIRMED_STATIC` applies to its bytes and read mechanism; `STRONG_STATIC_INFERENCE` applies to the candidate address mapping and its role as a firmware-managed interrupt dispatch table. The exact hardware-vector-to-stub entry mechanism remains unproven.

## 1. Address mapping and corroboration

The dispatcher at file offset `0x1009c` loads the literal at `0x1013c`, whose raw value is `0x010102bc`. The following instruction reads `[r0 + r1]`, where each entry stub supplies a byte offset. Subtracting candidate image base `0x01000200` gives:

| Observed address/pointer | Candidate file offset | Independent check |
|---|---:|---|
| Table base `0x010102bc` | `0x100bc` | Exact start of the 128-byte pointer array |
| Slot 14 pointer `0x010105b1` | `0x103b0` | Exact function start (`push {r4,lr}`) |
| Slot 5 pointer `0x01010619` | `0x10418` | Exact function start (`push {r4,r5,r6,lr}`) |
| Slot 6 pointer `0x010105d1` | `0x103d0` | Exact function start (`push {r4,lr}`) |
| Default pointer `0x010102bb` | `0x100ba` | Exact infinite-loop instruction (`b 0x100ba`) |
| Slot 27 pointer `0x01010401` | `0x10200` | Exact reset-like startup trampoline |
| Startup target `0x0101047f` | `0x1027e` after clearing Thumb bit | Exact candidate application-init block |
| Startup record bounds `0x010034ec..0x0100350c` | `0x32ec..0x330c` | Two complete 16-byte records in image |
| Early target `0x01000899` | `0x698` after clearing Thumb bit | In-image executable code |

The common offset relationship across the table, multiple handler starts, the reset trampoline, application-init target, and startup-record region is much stronger than the earlier single-base guesses. The candidate base `0x01000200` is therefore `STRONG_STATIC_INFERENCE`; it is not elevated to `CONFIRMED_STATIC` because no authoritative linker map, MCU memory map, or loader mapping document is available.

The startup pointer at `0x0101047f` is particularly useful: `0x10200` calls the startup walker and then branches through this literal; with the candidate mapping it enters exactly at `0x1027e`. This resolves the earlier reset-to-application-init uncertainty at the static control-flow level, subject to the candidate mapping.

## 2. Table geometry and complete contents

The table begins at file offset `0x100bc` and occupies `0x80` bytes, ending immediately before the dispatcher literal at `0x1013c`. The stubs at `0x1001c, 0x10020, ... 0x10098` supply offsets `0x00, 0x04, ... 0x7c`: 32 distinct four-byte offsets. The dispatcher uses the supplied byte offset directly, not an index multiplied at runtime. The observed data supports 4-byte entries and a 4-byte stride.

All raw entries have bit 0 set, consistent with Thumb function pointers. The table has no descriptor fields or embedded arguments. No length/bounds check or pointer validation is performed by the dispatcher.

| Index | Table file offset | Raw 32-bit value | Candidate target file offset | Interpretation |
|---:|---:|---:|---:|---|
| 0 | `0x100bc` | `0x0101bf55` | `0x1bd54` | Code pointer |
| 1 | `0x100c0` | `0x010102bb` | `0x100ba` | Default loop |
| 2 | `0x100c4` | `0x010102bb` | `0x100ba` | Default loop |
| 3 | `0x100c8` | `0x010102bb` | `0x100ba` | Default loop |
| 4 | `0x100cc` | `0x010102bb` | `0x100ba` | Default loop |
| 5 | `0x100d0` | `0x01010619` | `0x10418` | Timer-selector-0 status routine |
| 6 | `0x100d4` | `0x010105d1` | `0x103d0` | Timer-selector-1 status routine |
| 7 | `0x100d8` | `0x010102bb` | `0x100ba` | Default loop |
| 8 | `0x100dc` | `0x010102bb` | `0x100ba` | Default loop |
| 9 | `0x100e0` | `0x010102bb` | `0x100ba` | Default loop |
| 10 | `0x100e4` | `0x010102bb` | `0x100ba` | Default loop |
| 11 | `0x100e8` | `0x010102bb` | `0x100ba` | Default loop |
| 12 | `0x100ec` | `0x010102bb` | `0x100ba` | Default loop |
| 13 | `0x100f0` | `0x010102bb` | `0x100ba` | Default loop |
| 14 | `0x100f4` | `0x010105b1` | `0x103b0` | Output wrapper |
| 15 | `0x100f8` | `0x010105c9` | `0x103c8` | State-byte setter |
| 16 | `0x100fc` | `0x010102bb` | `0x100ba` | Default loop |
| 17 | `0x10100` | `0x010102bb` | `0x100ba` | Default loop |
| 18 | `0x10104` | `0x010102bb` | `0x100ba` | Default loop |
| 19 | `0x10108` | `0x010102bb` | `0x100ba` | Default loop |
| 20 | `0x1010c` | `0x010102bb` | `0x100ba` | Default loop |
| 21 | `0x10110` | `0x010102bb` | `0x100ba` | Default loop |
| 22 | `0x10114` | `0x010102bb` | `0x100ba` | Default loop |
| 23 | `0x10118` | `0x010102bb` | `0x100ba` | Default loop |
| 24 | `0x1011c` | `0x010102bb` | `0x100ba` | Default loop |
| 25 | `0x10120` | `0x010102bb` | `0x100ba` | Default loop |
| 26 | `0x10124` | `0x010102bb` | `0x100ba` | Default loop |
| 27 | `0x10128` | `0x01010401` | `0x10200` | Reset-like startup trampoline; anomalous for an ordinary IRQ handler |
| 28 | `0x1012c` | `0x010102a7` | `0x100a6` | Infinite-loop instruction |
| 29 | `0x10130` | `0x010102a9` | `0x100a8` | Infinite-loop instruction |
| 30 | `0x10134` | `0x010102ab` | `0x100aa` | Infinite-loop instruction |
| 31 | `0x10138` | `0x0101054d` | `0x1034c` | Counter/state routine |

The unusual slot 27 target (`0x10200`) and the three distinct loop addresses at slots 28–30 are reasons not to claim a fully documented NVIC ABI from table shape alone. They do not invalidate the direct indexed lookup or the slot-14 pointer value.

## 3. Dispatcher reads and address calculation

The relevant instructions are:

```asm
1001c: movs r0, #0
1001e: b    0x1009c
...
10030: movs r0, #0x14
10032: b    0x1009c
...
10054: movs r0, #0x38
10056: b    0x1009c
...
10098: movs r0, #0x7c
1009a: mov  r8, r8
1009c: ldr  r1, [pc, #0x9c]  ; literal at file+0x1013c = 0x010102bc
1009e: ldr  r0, [r0, r1]
100a0: bx   r0
```

`0x10030` supplies `0x14` (slot 5); `0x10054` supplies `0x38` (slot 14). The handler table is read-only from this code path. The dispatch is a tail branch (`BX`), not a `BLX`, and has no explicit bounds or Thumb-bit validation.

**Classification:** `CONFIRMED_STATIC` for the 32 immediate-offset stubs, literal load, indexed word read and indirect branch. `STRONG_STATIC_INFERENCE` for their role as firmware-managed interrupt entry stubs.

## 4. Initializers, writers, and memory placement

### Static image initialization

The 32 words are already present in the raw firmware at file offsets `0x100bc..0x1013b`. The base literal is at `0x1013c`. A raw-byte search finds:

- `0x010102bc` only at file offset `0x1013c`.
- `0x010105b1` only at file offset `0x100f4` (slot 14).
- `0x01010619` only at file offset `0x100d0` (slot 5).
- `0x010102bb` at the default-handler slots listed above.

The generated analysis object `/tmp/th80-X-analysis.o` is ELF32 ARM relocatable with one `.text` section marked `AX` (not writable). It is a synthetic analysis object derived from the raw image, not the vendor's original link product; its section flags support but do not independently prove target flash placement.

No table population loop, assignment, copy destination, relocation fixup, callback registration API, or runtime replacement of this table was identified. No second reference to the base literal was found. The dispatcher only reads it. Because the table is part of the firmware's code/data image and there is no identified writer, a static image-resident table is the strongest explanation.

**Classification:** `CONFIRMED_STATIC` for the raw bytes and single read path; `STRONG_STATIC_INFERENCE` for image-resident, effectively read-only runtime contents; `UNKNOWN` for any boot ROM, loader, memory remap, or hardware alias that could alter the mapping before execution.

### Startup callback tables are separate

`0x2acc` walks 16-byte records bounded by raw pointers `0x010034ec` and `0x0100350c`. Under the candidate mapping these are file offsets `0x32ec..0x330c`, containing two records:

```text
file+0x32ec: 0100350c 00020000 00000130 01002cf0
file+0x32fc: 010035c8 00020130 00000ae0 01002d7e
```

The walker loads the first three words as arguments, loads the fourth as a Thumb callback, and executes `BLX`. These callback targets map to `0x2af0` and `0x2b7e`; neither is `0x103b0`. This is a distinct startup-record mechanism, not the 32-slot table at `0x100bc`.

The second walker `0x1d488` similarly reads 16-byte records using bounds `0x0101c8fb..0x0101e8fb`. Under the candidate mapping that range overlays file offsets `0x1c6fa..0x1e6fa`, whose first apparent record does not resolve cleanly as three arguments plus an in-image Thumb callback. Its mapping/contents remain unresolved; it is not evidence for registration of `0x103b0`.

## 5. Access inventory and unresolved writers

| Reference | Kind | Evidence / result |
|---|---|---|
| `0x1009c`, `0x1013c` | Table pointer setup + read | Literal `0x010102bc`; direct indexed load |
| `0x1001c..0x10098` | Address calculation | 32 stubs supply offsets `0..0x7c` in steps of four |
| `0x100bc..0x10138` | Static table data | 32 raw 32-bit Thumb-style pointers |
| `0x100a4..0x100ba` | Default handler code | Repeating `B .` loop; multiple entries target it or adjacent loop instructions |
| `0x1027e` startup block | Initialization context | No table write; establishes priorities and enables selected IRQs |
| `0x1d488`, `0x2acc` | Separate startup callback table reads | 16-byte records; do not write the 32-slot table |
| Firmware copy/relocation scans | Unresolved | No copy or fixup targeting `0x010102bc` found |
| Runtime replacement / alias writes | Unresolved | No alias or generic writer can be excluded without authoritative MCU memory map/boot-stage image |

There is no evidence that `0x010102bc` is a RAM pointer variable. It is the literal base address used directly by the dispatcher. There is no identified RAM mirror. The exact flash bank, aliasing rules, and boot-stage relocation policy remain unknown.

## 6. Conclusions

1. The table layout is recovered: 32 entries, four-byte stride, 128-byte span, raw Thumb pointers. `CONFIRMED_STATIC` for the bytes and access mechanics.
2. The `0x01000200` base mapping is strongly corroborated by multiple unrelated in-image pointers and startup control flow, but remains a mapping inference pending a linker map or authoritative memory map.
3. Slot `+0x14` is `0x01010619` -> candidate file offset `0x10418`, not `0x103b0`.
4. Slot `+0x38` (index 14) is `0x010105b1` -> candidate file offset `0x103b0`.
5. No dynamic table writer, copy, relocation or registration function was found. The supplied image itself provides the pointer values.
6. Whether the 32 stubs are entered directly by NVIC hardware or through an external/ROM vector trampoline remains an explicit dependency. See [interrupt dispatch and callback resolution](PHASE13_5_INTERRUPT_DISPATCH_AND_CALLBACK_RESOLUTION.md).

## Related reports

- [Interrupt dispatch and callback resolution](PHASE13_5_INTERRUPT_DISPATCH_AND_CALLBACK_RESOLUTION.md)
- [Timer lifecycle and RGB activation](PHASE13_5_TIMER_LIFECYCLE_AND_RGB_ACTIVATION.md)
- [Findings and next steps](PHASE13_5_FINDINGS_AND_NEXT_STEPS.md)
- [Phase 13.4 runtime entry and call graph](PHASE13_4_RGB_RUNTIME_ENTRY_AND_CALLGRAPH.md)
- [Phase 13.4 scheduler, timer and initialization](PHASE13_4_SCHEDULER_TIMER_AND_INITIALIZATION.md)
- [Phase 13.3 renderer dataflow](PHASE13_3_RGB_RENDERER_DATAFLOW.md)
