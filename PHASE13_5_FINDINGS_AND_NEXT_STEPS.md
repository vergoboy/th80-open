# Phase 13.5 — Findings and Next Steps

## 1. Executive conclusion

Phase 13.5 recovered the static contents of the candidate handler table and substantially narrowed the runtime RGB invocation path.

**The original timer-IRQ-5-to-`0x103b0` hypothesis is rejected.** Under the strongly corroborated image mapping `0x01000200 + file_offset`, slot 5 (`+0x14`) contains `0x01010619`, resolving to `0x10418`, a routine that polls timer selector 0. Timer setup `0x173cc` configures selector 0 and enables index 5.

**The output wrapper is instead in slot 14 (`+0x38`).** That slot contains `0x010105b1`, resolving to `0x103b0`. The application initialization path separately enables NVIC index 14 through `0x10240`; priority 14 is also configured. The numeric correspondence strongly supports an IRQ14-to-`0x103b0` path, although the exact hardware vector/VTOR or boot-ROM entry into the 32 indexed stubs is not fully proven.

This is a significant static resolution of the previously missing registration: `0x103b0` is present as a static function pointer in the image-resident table. It is not a dynamically discovered callback assignment. Actual invocation through hardware IRQ14 is `STRONG_STATIC_INFERENCE`, not fully `CONFIRMED_STATIC`, until the vector-entry linkage and event source are independently established.

## 2. Strongest confirmed discoveries

1. **The table is present in the firmware image.** It occupies file offsets `0x100bc..0x1013b`, contains 32 32-bit Thumb-style pointers, and is read by the dispatcher at `0x1009c`. `CONFIRMED_STATIC`.
2. **The candidate image mapping is strongly corroborated.** Base `0x01000200` maps the table pointer, all valid table targets, the reset trampoline target `0x0101047f` to file offset `0x1027e`, the two-record startup table, and other in-image pointers consistently. `STRONG_STATIC_INFERENCE`; no linker map is available.
3. **Slot 5 does not contain `0x103b0`.** It contains `0x01010619`, resolving to `0x10418`. `CONFIRMED_STATIC` for the raw slot value; `STRONG_STATIC_INFERENCE` for the candidate target mapping.
4. **Slot 14 does contain `0x103b0`.** It contains `0x010105b1`, resolving to exact function start `0x103b0`. `CONFIRMED_STATIC` for the pointer bytes and `STRONG_STATIC_INFERENCE` for the mapped target.
5. **NVIC indices 5, 14 and 15 are enabled by the firmware.** `0x1d304` sets bits at `0xe000e100`; callsites pass 5 from `0x173cc`, 14 from `0x10240`, and 15 from `0x10214`. `CONFIRMED_STATIC` for instructions/arguments; `STRONG_STATIC_INFERENCE` for standard NVIC interpretation.
6. **The reset-like trampoline is statically wired to the init block.** `0x10200 -> 0x1d488` (record walker) -> `0x10208` -> `BX 0x0101047f` -> candidate file offset `0x1027e`. The target mapping is `STRONG_STATIC_INFERENCE`; actual completion of the `0x1d488` walk remains conditional because its runtime record region is unresolved.
7. **IRQ14 is dynamically disabled in transition paths.** `0x1d312` writes to `0xe000e180`; index 14 is disabled at `0x115e0` and `0x11796`. Selected transitions call `0x10240` to enable it again. `CONFIRMED_STATIC` for the register writes and arguments.
8. **Timer and output wrapper are separate table entries.** Slot 5 maps to selector-0 status routine `0x10418`; slot 14 maps to output wrapper `0x103b0`. `STRONG_STATIC_INFERENCE` for the associated interrupt roles.

## 3. Reconstructed dispatch and lifecycle

```text
Reset-like entry 0x10200
  -> startup callback walker 0x1d488 (runtime record completion unresolved)
  -> if walker completes: 0x10208 -> BX 0x0101047f
  -> candidate app init 0x1027e
       -> configure priorities 0, 5, 14
       -> 0x173cc: configure timer selector 0; enable index 5
       -> 0x13400: initialize candidate output subsystem
       -> 0x10240: enable index 14
       -> 0x10214: enable index 15
       -> main loop 0x10308

Indexed dispatch:
  stub for offset 0x14 (slot 5) -> table[5] -> 0x10418 (timer selector 0)
  stub for offset 0x38 (slot 14) -> table[14] -> 0x103b0 -> 0x135de

IRQ14 transition gating:
  disable via 0x1d312 at 0x115e0 / 0x11796
  selected transitions re-enable via 0x10240
```

The dispatch stubs and table are statically confirmed. The final hardware event-to-stub link remains `STRONG_STATIC_INFERENCE`; the precise IRQ14 source, periodicity and first firing time remain `UNKNOWN`.

## 4. Timer and RGB conclusions

- Timer selector 0 is configured by `0x173cc`; the routine enables index 5 and computes a quotient of 96 from `48,000,000 / 500,000`. Timer period and refresh frequency remain `UNKNOWN`.
- Table slot 5 points to `0x10418`, which polls timer selector 0. This is the strongest explanation of the IRQ5 route.
- Table slot 14 points to `0x103b0`; index 14 is enabled after `0x13400` in the candidate startup sequence. This is the likely output-wrapper interrupt route.
- `0x103b0` calls `0x135de`; `0x135de` skips normal frame transmission while state byte `0x20348` equals `0xfd`.
- The main loop dispatches state `0xfd` to `0x11328`, which conditionally clears the 672-byte candidate buffer and reconfigures timer selector 0. It does not prove table modification or IRQ14 disablement.
- Other transition routines explicitly disable IRQ14 and selected transitions later re-enable it. The table itself remains unchanged in the recovered static evidence.

No refresh rate, physical pin mapping, electrical output protocol, LED power state, or physical RGB failure cause is established.

## 5. Rejected and qualified hypotheses

| Hypothesis | Result | Reason |
|---|---|---|
| IRQ/index 5 directly calls `0x103b0` | **Rejected** | Slot 5 raw pointer is `0x01010619`, mapping to `0x10418` |
| `0x103b0` has no static registration | **Rejected under candidate mapping** | Slot 14 contains raw pointer `0x010105b1`, mapping to `0x103b0` |
| Timer setup is the proven RGB refresh scheduler | **Rejected** | Timer setup enables index 5; slot 5 points to timer-selector-0 status routine, not output wrapper |
| IRQ14 invokes `0x103b0` | **Strongly supported, not fully proven** | Matching stub/table index, NVIC priority and enable index; hardware vector-to-stub entry is missing |
| Table is dynamically populated in RAM | **Not supported** | Table bytes are embedded in image; no writer/copy/fixup found |
| `0xfd` disables/replaces handler slot 14 | **Unknown / not demonstrated** | `0xfd` gates output and triggers timer/buffer work; explicit IRQ14 disables occur in other transition routines |
| Timer quotient 96 is refresh Hz | **Rejected** | Clock and timer register semantics are not established |

## 6. Physical RGB failure implications

The investigation removes one major uncertainty: there is a plausible static interrupt dispatch route from index 14 to the output wrapper. This makes a total absence of any registration less likely than Phase 13.4 suggested, but it does not establish that IRQ14 is firing on the user's device.

Remaining software-side possibilities include:

- IRQ14 source not asserted, disabled by a transition, or not entered through the expected vector stub.
- `0x135de` is reached but suppressed while `0x20348 == 0xfd`.
- The candidate output buffer is cleared and not repopulated by the producer path.
- Output-control register configuration or a shared peripheral condition is not restored.
- `0x103b0` executes but the candidate GPIO-like writes do not reach the physical LED chain.

The actual physical cause remains `UNKNOWN`. No hardware test is authorized or implied by these static findings.

## 7. Remaining transitions / exact dependencies

1. Recover the authoritative vector-table base/VTOR configuration or boot-ROM trampoline proving which hardware exception enters stub `0x10054` for IRQ14.
2. Identify the peripheral/event source assigned to IRQ14 and whether it is periodic, edge-triggered, level-triggered or software-pended.
3. Determine whether the two IRQ14 disable paths (`0x115d8`, `0x1176c`) correspond to the same state transitions as the `0xfd` path; current evidence does not prove this.
4. Resolve the second startup record walker `0x1d488`'s record-range mapping and contents; the reset-to-`0x1027e` branch target is nevertheless statically identifiable under the candidate mapping.
5. If future static artifacts become available, compare the firmware's memory map/linker script and boot ROM to validate the candidate base and vector entry semantics.

No further application implementation or hardware work should proceed from this phase alone.

## 8. Integrity and repository preservation

Before report creation, the three protected artifact hashes matched the user-provided values:

| Artifact | SHA-256 |
|---|---|
| Firmware `X` | `4b22336ec9ffaaac3372791a1154a2c2a833e3305a4cadbbdcab9184252c0520` |
| Native vendor driver `iot_driver` | `22c5dc67ba77e2373a5ffdbb0312ed8c8e73eb16e78037764eb676a030b2724b` |
| Frontend `index.js` | `7d53737095fe5756fb0a18d5aecb7593555b9bb61a9d5d160ae0b988694c1e87` |

The initial Git inventory showed no commits on `main`, no remotes, no staged changes, and extensive pre-existing untracked files. Those files were preserved; the four Phase 13.5 reports are additional untracked files. Final verification results are recorded after report creation in the completion response.

No firmware, driver, frontend, Rust source, Cargo dependency, GUI, or historical report was modified. No HID enumeration, physical device open, HID report, USB transfer, firmware execution/patch/flash, reset, pairing, calibration, bootloader, or vendor configurator operation was performed.

## Related reports

- [Handler table layout and initialization](PHASE13_5_HANDLER_TABLE_LAYOUT_AND_INITIALIZATION.md)
- [Interrupt dispatch and callback resolution](PHASE13_5_INTERRUPT_DISPATCH_AND_CALLBACK_RESOLUTION.md)
- [Timer lifecycle and RGB activation](PHASE13_5_TIMER_LIFECYCLE_AND_RGB_ACTIVATION.md)
- [Phase 13.4 runtime entry and call graph](PHASE13_4_RGB_RUNTIME_ENTRY_AND_CALLGRAPH.md)
- [Phase 13.4 scheduler, timer and initialization](PHASE13_4_SCHEDULER_TIMER_AND_INITIALIZATION.md)
- [Phase 13.4 runtime gating](PHASE13_4_RGB_RUNTIME_GATING.md)
- [Phase 13.3 renderer dataflow](PHASE13_3_RGB_RENDERER_DATAFLOW.md)
- [Phase 13.3 LED output protocol analysis](PHASE13_3_LED_OUTPUT_PROTOCOL_ANALYSIS.md)
