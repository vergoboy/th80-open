# Phase 13.7 — Findings and Hardware Boundary

## Executive conclusion

Phase 13.7 closes the firmware-side Timer1-to-IRQ14 software route but does **not** identify the exact MCU, establish HCLK, prove timer interrupt acknowledgement, or demonstrate physical RGB output. A material timer-register discrepancy was found between the Phase 13.6 datasheet-correlated labels (`0xf0c08`/`0xf0c48`) and the literal arithmetic in the shared firmware helpers (`0xf3fd4`/`0xf4014`). The new reports preserve that conflict rather than inventing a reconciliation.

This phase was static-only. No HID/USB operation, physical keyboard interaction, firmware execution, bootloader entry, flashing, current measurement, or hardware test was performed. No firmware, vendor driver, frontend, Rust source, Cargo dependency, or GUI file was modified.

## Findings matrix

| Topic | Finding | Classification | Boundary |
|---|---|---|---|
| Yichip SDK lineage | Embedded source-path strings include `yc_timer.c`, `yc_sysctrl.c`, `yc_gpio.c`, `yc_uart.c`, `yc_systick.c`, `yc_wdt.c`, `yc_ssc.c` | `CONFIRMED_STATIC` | Does not identify exact silicon |
| MCU family | YC3121 family is best-supported candidate, with YC3121-L a prior TH80-family review lead | `STRONG_STATIC_INFERENCE` family / `WEAK_INFERENCE` exact L variant | Exact PCB marking absent |
| ISA | Coherent ARM Thumb stream includes `CPSID`, `CPSIE`, `MSR MSP`, Thumb `BL`/return sequences | `STRONG_STATIC_INFERENCE` | Exact ARM core profile not proven |
| Cortex-M0 | Prior-session BSP clue was not reproducible in the currently supplied tree | `WEAK_INFERENCE` | Exact BSP/config package needed |
| Boot vectors | Executable boot/application stub banks plus secondary dispatch table | `CONFIRMED_STATIC` | Not conventional Cortex-M vector table |
| `0xf8564` | Software-visible mode state: `2` selects boot table; `1` is written before app-bank handoff | `STRONG_STATIC_INFERENCE` | Hardware remap/vector-fetch role unknown |
| Timer1 init | Channel index 1, reload/count literal `0x226e`, NVIC index 14 | `CONFIRMED_STATIC` | Physical timer semantics conditional |
| Timer1 helper registers | Literal `0x000f400c`; channel-1 count write computed at `0x000f3fd4`; packed control word at `0x000f4014` | `CONFIRMED_STATIC` for arithmetic / `STRONG_STATIC_INFERENCE` for register roles | Conflicts with earlier `0xf0cxx` mapping |
| Timer2 | Channel index 2, separate count configuration, NVIC index 15, handler `0x103c8` | `CONFIRMED_STATIC` | Does not independently prove hardware routing |
| Timer1 clock | HCLK, oscillator, PLL, prescaler and peripheral divider not recovered | `UNKNOWN` | No frequency calculation justified |
| Timer1 acknowledgement | No identified peripheral flag/status clear; no ICPR0 write found | `UNKNOWN` | Cannot say pending state is cleared |
| IRQ14 software dispatch | App IRQ14 stub → dispatcher → table[14] → `0x103b0` | `CONFIRMED_STATIC` for bytes/control flow | Actual peripheral delivery unobserved |
| Physical RGB | Output software path is present; electrical LED signaling is not proven | `UNKNOWN` | Requires separately authorized hardware evidence |

## Timer1 semantics and timing

The application Timer1 init at file offset `0x10240` passes channel `1` and count `0x226e` to generic helpers and enables NVIC index `14`. Handler `0x103b0` clears the channel-1 enable bit, invokes `0x135de`, then sets the bit again.

The current disassembly literal at `0x1ac3c` is `0x000f400c`. The helper computes a per-channel count address as `base - 0x40 + channel*8`; for channel 1 this is `0x000f3fd4`. The shared control/config word is at `0x000f4014`; channel-1 enable uses bit 2, and configuration operations touch bits 4 and 5. This conflicts with the earlier Phase 13.6 candidate map at `0x000f0c08`/`0x000f0c48`. The exact `yc_timer` SDK source or a matching primary datasheet revision is required before naming these registers definitively.

The number `8814` is a configured count only. HCLK and prescaler are unknown, and counter inclusion/reload semantics are not confirmed. **No refresh rate, callback period, duty cycle, output duration, or timer-to-RGB ratio is reported.**

## IRQ14 and acknowledgement

The software path is coherent: Timer1 initialization enables NVIC bit 14 at the canonical `0xe000e100` bank; the application stub passes `0x38`; the secondary dispatcher indexes application table slot 14; the raw handler pointer resolves to `0x103b0`; and that wrapper calls the RGB output routine `0x135de`.

This establishes static routing within the supplied firmware image. It does not prove the hardware peripheral request reaches input 14 or that ROM/hardware enters the analyzed stub bank. No peripheral flag register or W1C/W0C clear sequence was identified. The wrapper's channel stop/start sequence does not directly disable the NVIC line or clear pending state. Counter restart/resume and immediate retrigger behavior remain unknown.

## RGB implementation implications

The firmware contains enough software structure to specify the **software-side** RGB callback relationship: channel 1 is configured, IRQ14 dispatches to `0x103b0`, and that wrapper calls `0x135de`. It does not yet provide an independent hardware implementation specification because the following remain open:

1. Exact MCU and timer peripheral register map.
2. HCLK and timer prescaler/reload semantics.
3. Timer interrupt flag and acknowledgement behavior.
4. Hardware/ROM vector selection and peripheral-to-NVIC routing.
5. Electrical LED interface, signaling protocol, pin mapping, and any external LED controller.
6. Actual runtime timing and whether the output routine completes before the next timer event.

A native Rust application can continue to implement and test the already established software dataflow and frontend protocol independently, but it should not write guessed MMIO registers or claim hardware RGB control from this analysis. Hardware transport and register-level implementation should remain blocked pending the missing evidence.

## Power and scheduling opportunities — ideas only

The static path suggests future opportunities to investigate, not changes authorized or validated in this phase:

- Stop the RGB timer when lighting is disabled, if source and counter semantics permit.
- Avoid transmitting unchanged frames.
- Reduce update frequency for static effects only after timing requirements are measured.
- Gate peripheral clocks during sleep where the SoC supports it.
- Avoid redundant frame generation and output calls.

No battery-current model or measurement exists; no power savings are estimated. State `0xfd` suppresses the normal output transmission path in software, but that alone does not prove the timer is stopped or that the device consumes less power.

## Most valuable next investigation

Obtain and correlate the **exact Yichip SDK release used to build this firmware**, especially `Librarier/sdk/yc_timer.c`, `yc_timer.h`, `yc_sysctrl.c`, `yc_sysctrl.h`, SOC selection config, compiler target flags, and linker script. The timer helper literal arithmetic and the `0xf0cxx` datasheet interpretation currently disagree; the matching SDK would resolve register naming, channel layout, mode bits, and clock API semantics in one step. The same package may establish whether the selected core is Cortex-M0 and which YC3121 variant the image targets.

If that SDK cannot be recovered, the next most valuable external evidence is a clear marking photograph of the exact PCB MCU and board revision, followed by the matching primary datasheet. Do not disassemble the keyboard solely for this purpose; obtain such evidence only if safely and independently authorized.

## Hardware boundary and authorization

Phase 13.7 does not authorize or perform PCB probing, electrical measurements, USB control transfers, HID feature requests, firmware execution, bootloader entry, pairing/reset/calibration, or flashing. Any such work must be a separate explicitly authorized phase with a defined safety plan. Static report conclusions must not be presented as observed hardware behavior.

## Cross-references

- [Silicon and boot architecture](PHASE13_7_SILICON_AND_BOOT_ARCHITECTURE.md)
- [Timer1 register and clock analysis](PHASE13_7_TIMER1_REGISTER_AND_CLOCK_ANALYSIS.md)
- [IRQ14 event and acknowledgement](PHASE13_7_IRQ14_EVENT_AND_ACKNOWLEDGEMENT.md)
- Historical synthesis: [Phase 13.6 findings and next steps](PHASE13_6_FINDINGS_AND_NEXT_STEPS.md)
