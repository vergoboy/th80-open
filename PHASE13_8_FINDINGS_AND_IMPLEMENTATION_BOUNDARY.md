# Phase 13.8 — Findings and Implementation Boundary

**Scope:** conclusions from static SDK provenance, timer register reconciliation, clock reconstruction, and interrupt analysis. No firmware changes or hardware interaction were performed.

**Related reports:**

- [SDK provenance](PHASE13_8_YICHIP_SDK_PROVENANCE.md)
- [Timer register map reconciliation](PHASE13_8_TIMER_REGISTER_MAP_RECONCILIATION.md)
- [Clock and interrupt semantics](PHASE13_8_CLOCK_AND_INTERRUPT_SEMANTICS.md)

## 1. Most important discoveries

1. **The apparent alternate timer register bank was a byte-order error.** The original literal bytes at `X+0x1ac3c` are `40 0c 0f 00`, or `0x000f0c40`, not `0x000f400c`. The false `0xf3fd4`/`0xf4014` register family is withdrawn.
2. **The firmware timer helpers use four-bit channel fields.** `0x1a9b2`, `0x1a9fa`, and `0x1aa50` calculate `channel*4`, not `channel*2`. This matches the Yichip header and datasheet field layout.
3. **The timer register model is internally coherent.** Timer1 PCNT is `0xf0c08`, Timer2 PCNT is `0xf0c10`, TIM_CTRL is `0xf0c48`, and TIM_CTRL1 is `0xf0c4c`.
4. **The datasheet prose contains repeated address typos.** Detailed tables and Yichip header formulas resolve TIM_NCNT, TIM_CTRL, and TIM_CNT locations. Both D and L datasheets repeat these typos.
5. **The startup HCLK is selected as 96 MHz.** Firmware writes HCLK selector zero before initializing Timer1/Timer2. Low-clock paths select 12 MHz and scale Timer2 reload accordingly.
6. **The nominal Timer1 count is approximately 91.8 μs at 96 MHz.** It is not the RGB callback period because Timer1 is disabled while the RGB output routine executes.
7. **Timer source acknowledgement remains unknown.** The available timer docs and header do not define a status/clear register or an ISR acknowledgement procedure.
8. **The strongest SDK provenance is family-level, not release-level.** Yichip-authored YC3121 POS BSP headers and datasheets match firmware semantics, but the original `yc_timer.c` source and a package version manifest remain unavailable.

## 2. SDK identification status

**Status:** exact SDK release not identified.

Best-supported classification: **matching Yichip YC3121 SDK family, exact version unknown** (`STRONG_STATIC_INFERENCE`).

The public RT-Thread YC3121 POS BSP contains Yichip-authored `yc3121.h`, `misc.h`, `system.h`, and `rom_api.h` files with 2018/2020 file-level version stamps. Their timer register formulas, control-bit positions, interrupt enum, HCLK formula, and clock gate semantics match the firmware. The official Yichip `YC3121` GitHub repository at the inspected commit contains only a README. The public BSP lacks `yc_timer.c` and `yc_sysctrl.c`; therefore there is no original timer implementation to compare line-for-line or byte-for-byte.

Do not claim the BSP's source dates form an exact SDK release. Do not claim the keyboard PCB uses the same exact YC3121 silicon as the POS development board.

See [SDK provenance](PHASE13_8_YICHIP_SDK_PROVENANCE.md) for URLs, commits, source hashes, datasheet hashes, and the search method.

## 3. Corrected timer register model

| Register/function | Address or expression | Established meaning |
|---|---:|---|
| Timer1 reload | `0x000f0c08` | `TIM_PCNT(1)` |
| Timer2 reload | `0x000f0c10` | `TIM_PCNT(2)` |
| Timer1 current count | `0x000f0c54` | `TIM_CNT(1)` |
| Timer2 current count | `0x000f0c58` | `TIM_CNT(2)` |
| Shared control | `0x000f0c48` | `TIM_CTRL`, channels 0–7 |
| Channel 8 control | `0x000f0c4c` | `TIM_CTRL1` |
| Timer module clock gate | `0x000f856c`, bit 4 | `SYSCTRL_CLK_CLS[4]`, 1 closes TIM module clock |
| HCLK selector | `0x000f8560`, bits 3:0 | `SYSCTRL_HCLK_CON[3:0]` |

Per-channel control nibble:

- bit 0: channel enable;
- bit 1: initial level;
- bit 2: TIMER/PWM mode;
- bit 3: auto-reload.

For Timer1 the nibble is bits `[7:4]`; for Timer2 it is `[11:8]`. Firmware configures mode and auto-reload and then enables each channel. The `0x1aa44` helper sets the global TIM module clock gate bit, but no direct caller was found; it is not part of the active Timer1 wrapper.

See [timer register map reconciliation](PHASE13_8_TIMER_REGISTER_MAP_RECONCILIATION.md) for every helper and datasheet discrepancy.

## 4. Clock and timing conclusions

The Yichip HCLK formula is `192 MHz / (HCLK_SEL + 2)`. Firmware startup selects `HCLK_SEL=0`, corresponding to 96 MHz, before Timer1/2 setup. Selected lower-clock transitions set selector 14, corresponding to 12 MHz.

- Timer1 PCNT = `0x226e` = 8,814. At 96 MHz, nominal countdown is 91.8125 μs, or 91.8229 μs if the terminal count is inclusive. Datasheet text does not settle that one-tick detail.
- Timer2 reload is `caller_argument * 1,000,000`. Firmware passes 96 at 96 MHz and 12 at 12 MHz, both corresponding to a nominal one-second count interval.
- Timer1 is disabled around the output callback. Re-enable likely reloads the current counter from PCNT, based on the documented load-on-enable behavior. Therefore actual IRQ14 spacing includes callback execution time and is not equal to 91.8 μs.

No physical RGB refresh rate, waveform, or output duration is established.

See [clock and interrupt semantics](PHASE13_8_CLOCK_AND_INTERRUPT_SEMANTICS.md).

## 5. IRQ14/IRQ15 implications

The Yichip IRQ enum assigns Timer1 to IRQ14 and Timer2 to IRQ15. Firmware initialization enables those same NVIC indices. The existing software dispatch maps Timer1 to `0x103b0` and Timer2 to `0x103c8`.

The datasheet says TIMER mode enables interrupt generation by default and the timer counts down. This supports a count-zero event source. However, neither the available datasheets nor the public header documents a timer-specific interrupt status/clear register. The application handler and wrapper do not show an explicit source acknowledgement, and no NVIC ICPR write was found.

Do not implement an assumed W1C/W0C flag clear, do not assume disable/re-enable clears pending state, and do not treat the software dispatch chain as proof of physical interrupt delivery.

## 6. Independent RGB implementation boundary

### 6.1 Established enough to specify (static software target)

- Timer1 and Timer2 channel numbers and corresponding IRQ enum values.
- PCNT reload addresses and per-channel stride (`0x8`).
- TIM_CTRL/TIM_CTRL1 locations and four-bit channel control fields.
- Enable, TIMER mode, and auto-reload bit positions.
- HCLK selector register and documented formula.
- Firmware's startup HCLK selector and the Timer2 reload scaling strategy.
- Timer1's software disable/output/re-enable wrapper structure.
- Existing RGB software output routine entry (`0x135de`) and the fact that Timer1 is disabled around it.

These facts support a written architecture/register specification, not a safe live driver.

### 6.2 Still required before a safe Rust RGB driver

| Driver responsibility | Minimum missing information |
|---|---|
| Timer initialization | Exact MCU/revision and confirmation that the TH80 PCB uses the YC3121 register map; reset/clock state and any required module-clock delay/ordering |
| Timer scheduling | Exact enable-edge reload behavior, inclusive count convention, and behavior of pending events during disable/re-enable |
| RGB frame timing | Measured or source-defined duration of the output routine, actual output peripheral clock, required inter-frame timing, and physical LED protocol/electrical topology |
| Output peripheral control | Exact GPIO/PWM/SPI/bit-bang peripheral and pin mux used by the PCB, polarity, voltage domain, and output-driver requirements |
| Interrupt acknowledgement | Exact timer IRQ flag/status semantics, automatic vs explicit acknowledgement, W1C/W0C behavior if any, and NVIC pending/exception behavior for the exact core |

The single most critical blocker for implementing IRQ14 safely is the timer source acknowledgement contract. The most important board-specific blocker for RGB signaling is identifying the physical output path and LED interface from the PCB/schematic or an independently authorized hardware investigation.

## 7. Power-management opportunities (ideas only)

The following are engineering opportunities, not implementation instructions or measured savings:

- Keep the documented global TIM module clock gate open only while Timer1/2/3 are needed; the firmware contains a helper that sets `SYSCTRL_CLK_CLS[4]` to close it, but no direct call was found.
- Keep Timer1 disabled when no output update is required, provided state transitions and restart behavior are fully specified.
- Avoid repeated RGB frame generation when the buffer has not changed.
- Use lower HCLK only in modes where active RGB output and required timer cadence are suspended or compensated.
- Preserve the observed Timer2 reload scaling when HCLK changes if its one-second cadence is required.

No current model or power savings estimate is available. No firmware change is authorized in this phase.

## 8. Single most valuable next investigation

**Obtain the exact V504 Yichip SDK package (or a vendor-authenticated source archive) containing `Librarier/sdk/yc_timer.c` and `yc_sysctrl.c`, with its release metadata.**

This would directly resolve whether the binary helpers are built from the same timer implementation as the public YC3121 POS header, expose any omitted timer status/acknowledgement API, and permit a proper source-to-binary semantic comparison. If that package cannot be recovered, the next most valuable evidence is the exact PCB MCU marking plus an official matching datasheet/errata; neither should be guessed from the firmware alone.

## 9. Stop condition

Phase 13.8 stops at static research and reporting. No Rust driver, firmware patch, transport, GUI, HID, USB, or hardware test is part of this deliverable.
