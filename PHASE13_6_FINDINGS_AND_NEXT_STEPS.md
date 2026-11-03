# Phase 13.6 — Findings and Next Steps

## Overall result

Phase 13.6 closes most of the firmware-side interrupt path and identifies Timer1 as the strongest source for NVIC external interrupt index 14. It does **not** prove the SoC's physical vector-fetch base, actual interrupt arrival, or electrical LED output. The correct conclusion is conditional and does not require treating the device as a standard Cortex-M implementation.

## Strongest discoveries

1. **Two vector-stub banks exist.** The boot-stage bank begins at file offset `0x0000` (candidate runtime `0x01000200`); the application bank begins at file offset `0x10000` (candidate runtime `0x01010200`). Each has a reset entry, four special-vector stubs and 32 external-interrupt stubs. `CONFIRMED_STATIC` for the code and layout; `STRONG_STATIC_INFERENCE` for their role as boot/application vector banks.

2. **The boot/application forwarding mechanism is recovered.** Boot external stubs at file `0x1c..0x98` pass offsets `0..0x7c` to dispatcher `0xce`. That dispatcher checks `0x000f8564`: value 2 selects the separate boot table at `0x0100032c`; any other value forwards to application dispatcher `0x0101029c`. The handoff routine at `0x00f8` writes mode value 1 and branches to application vector-bank base `0x01010200`. `CONFIRMED_STATIC` for instructions; `STRONG_STATIC_INFERENCE` for mode-1 application and mode-2 boot semantics.

3. **The application secondary handler table is real firmware data.** The application dispatcher at `0x1009c` loads a word from `0x010102bc + r0` and tail-branches through it. The table is at file offset `0x100bc`, has 32 four-byte entries, and is static in the image. No writer, relocation fixup or runtime replacement was identified. `CONFIRMED_STATIC` under the candidate mapping.

4. **IRQ14's application stub selects slot 14 exactly.** Stub `0x10054` (and the boot-bank equivalent at `0x54`) supplies `r0=0x38`; the dispatcher uses this as a byte offset, so `0x38 / 4 = 14`. Slot 14 contains raw `0x010105b1`, resolving to `0x103b0`. `CONFIRMED_STATIC` for the indexed dispatch when the stub is entered.

5. **Timer1 is the strongest IRQ14 source.** Function `0x10240` configures Timer1 channel 1, writes PCNT1 value `0x226e` at `0x000f0c08`, sets timer-mode/auto-reload fields, enables channel 1 and enables NVIC index 14. The handler `0x103b0` disables and re-enables that same channel around `0x135de`. The candidate YC3121-L datasheet describes nine independent down-counting timer interrupt sources and HCLK timer clocking. `STRONG_STATIC_INFERENCE` because exact board MCU identity and a Timer1-to-NVIC mapping table are not proven.

6. **The earlier IRQ5 timer interpretation was wrong.** Function `0x173cc` configures UART0 at base `0x000f8b00`, uses a `500000` baud parameter and derives divisor 96 from the fixed 48 MHz UART clock, then enables NVIC index 5. Table slot 5 resolves to the UART0 polling routine `0x10418`; slot 6 resolves to UART1 routine `0x103d0`. This corrects Phase 13.4/13.5's characterization of `0x173cc` as timer initialization. The quotient 96 is a UART baud divisor, not an RGB timer period.

7. **Timer2 is separately configured for IRQ15.** `0x10214` configures channel 2 and enables NVIC index 15; application table slot 15 resolves to `0x103c8`. This further supports the timer-channel/IRQ pairing pattern.

8. **The wrapper's control helper is not an interrupt lock.** `0x1a9b2` toggles a per-channel timer-control field. It does not modify NVIC enable state or global CPU interrupt masks. The datasheet's `TIM_CTRL` address is internally inconsistent with its PCNT/NCNT map, so the precise physical register name remains qualified.

## Required claim-by-claim status

| Question | Result | Classification |
|---|---|---|
| Is there a firmware vector-stub path? | Yes: boot/app code-stub banks and dispatchers are present | `CONFIRMED_STATIC` |
| Is there a conventional Cortex-M architectural vector table/VTOR setup in the image? | No such table or VTOR write was found; exact core/vector architecture is not proven | `CONFIRMED_STATIC` for absence in analyzed image; hardware format `UNKNOWN` |
| Is the application table a secondary IRQ dispatch table? | Yes, the vector stubs supply byte offsets and dispatcher indexes its 32 words | `CONFIRMED_STATIC` |
| Does vector stub index 14 select table entry 14? | Yes, `0x38 / 4 = 14` | `CONFIRMED_STATIC` |
| Does application table entry 14 resolve to `0x103b0`? | Yes, raw `0x010105b1` under candidate mapping | `CONFIRMED_STATIC` |
| Does hardware external IRQ14 enter the corresponding stub? | Strongly supported by matching vector-stub and NVIC indices, but the SoC/ROM vector-base fetch is not documented in this image | `STRONG_STATIC_INFERENCE` |
| Is IRQ14's peripheral source Timer1? | Timer1 channel 1 is initialized alongside NVIC14; its handler gates Timer1 around output | `STRONG_STATIC_INFERENCE` |
| Does IRQ14 invoke `0x103b0` in application mode? | The complete firmware-side dispatch path is present; actual interrupt arrival remains unobserved | `STRONG_STATIC_INFERENCE` |
| Does `0x103b0` call the RGB output routine? | Yes, it directly calls `0x135de` | `CONFIRMED_STATIC` |
| Is a frame sent when state is `0xfd`? | No; `0x135de` skips its normal frame path | `CONFIRMED_STATIC` |
| Do physical LEDs receive the frame? | Not established | `UNKNOWN` |

## Exact remaining blockers

1. **Hardware vector fetch/base:** The supplied firmware contains code-stub vector banks and an explicit boot-to-application forwarding path, but no VTOR write, IPSR read or documented SoC vector-base register. The hardware/boot-ROM rule that enters the boot-stage bank at candidate runtime `0x01000200` (or directly selects the application bank) is not included. The Yichip family brief is not sufficient to prove the exact silicon core or vector-fetch format.

2. **Mode register identity:** `0x000f8564` is written with 1 by the application handoff, written with 2 by a small helper at `0x0108`, and compared with 2 by boot dispatchers. Its physical name/meaning is not present in the candidate datasheet text.

3. **Timer register-map inconsistency:** The firmware uses `0x000f0c48` as a packed channel-control word for Timer0–Timer7 and `0x000f0c4c` for Timer8. The candidate datasheet lists `TIM_CTRL` at `0x000f0c30`, overlapping its own `TIM6_PCNT`, and lists `TIM_NCNT` starting at `0x000f0c48`. A corrected silicon register manual or matching SDK header is needed to prove the control register address and pending-clear semantics.

4. **Timer1 timing:** The reload count is known (`0x226e`) and the candidate datasheet says HCLK clocks the timers, but runtime HCLK/divider configuration is not recovered. No period or RGB refresh rate is claimed.

5. **Pending interrupt semantics:** No NVIC ICPR0 write at `0xe000e280` or separate Timer1 interrupt-clear sequence was found. Timer1 disable/re-enable reloads the counter according to the candidate datasheet, but whether it clears a pending NVIC request is unknown.

6. **State `0xfd`:** `0x135de` suppresses output in this state. The main-loop `0xfd` path does not directly disable NVIC14 or Timer1. Separate transition paths disable both, but no exact equivalence between their predicates and `0xfd` has been proven.

7. **Exact board silicon:** The project history treats YC3121-L as a family-level lead, not a verified PCB marking. Datasheet-based register semantics remain conditional until exact MCU identity is established.

## Most valuable next investigation

The next static-only investigation should obtain a **matching YC3121-L SDK header/source package or boot-ROM/vector documentation** and compare it against:

- the boot/app vector-stub banks at file offsets `0x0000` and `0x10000`;
- the mode register at `0x000f8564`;
- the timer control field used at `0x000f0c48`;
- the documented Timer1-to-NVIC interrupt number;
- Timer1 interrupt pending/clear behavior and the actual HCLK divider.

If an authoritative boot-ROM image or map is available, resolve the fixed vector-fetch base without executing it. No hardware experiments are required or authorized by this phase.

## Preservation and safety

This phase is static analysis only. No firmware, driver, frontend, Rust source, Cargo dependency or GUI was modified. No HID/USB operation, hardware access, firmware execution, flashing or device configuration was performed. The four Phase 13.6 reports are the only intended new repository files; no staging, commit, push or remote change is authorized.

## Related reports

- [Vector table and IRQ entry](PHASE13_6_VECTOR_TABLE_AND_IRQ_ENTRY.md)
- [IRQ14 peripheral source](PHASE13_6_IRQ14_PERIPHERAL_SOURCE.md)
- [IRQ14 lifecycle and RGB trigger](PHASE13_6_IRQ14_LIFECYCLE_AND_RGB_TRIGGER.md)
- [Phase 13.5 findings (historical)](PHASE13_5_FINDINGS_AND_NEXT_STEPS.md)
