# Phase 13.10 — Timer SDK Source Comparison

## 1. Result

The original V504 timer driver was **not recovered**. Neither the exact `Librarier/sdk/yc_timer.c/.h` nor a source-equivalent Yichip YC3121 timer implementation exists in the inspected local source set or the public RT-Thread YC3121 POS BSP snapshot. Therefore this phase cannot provide an instruction-level source-to-binary match for the firmware timer helpers, reload sequence or interrupt acknowledgement.

The public BSP does provide a strong register-model reference in `libraries/core/yc3121.h` and `libraries/core/misc.h`. It is not the V504 timer source. The overall source classification remains `SAME_SDK_LINEAGE`; see [source package recovery](PHASE13_10_V504_SOURCE_PACKAGE_RECOVERY.md).

## 2. Source recovery inventory

Searched local TH80 project, Downloads, Documents, `.cache`, `.local/share`, Phase 13.8 and Phase 13.9 roots for exact timer/system-control filenames, SDK archives, Yichip/YC3121 identifiers and embedded project/build formats. The V504 firmware distribution archive contains no SDK source or project artifacts. The public RT-Thread checkout at `/tmp/th80-phase13-8/rt-thread`, commit `cbc19778d0ae5ef2533a28c0f62c6a399858010a`, contains:

- `libraries/core/yc3121.h`
- `libraries/core/misc.h` and `misc.c`
- `libraries/core/system.h` and `system.c`
- `libraries/core/rom_api.h`
- `libraries/sdk/yc_gpio.c/.h`
- `libraries/sdk/yc_uart.c/.h`
- `libraries/sdk/yc_systick.c/.h`
- `libraries/sdk/yc_dma.c/.h`
- `libraries/sdk/yc_wdt.c/.h`
- startup assembly and Keil/IAR/GCC project/linker configuration

It does **not** contain `yc_timer.c/.h` or `yc_sysctrl.c/.h`. There is no timer implementation or timer callback-registration source in the BSP inventory. The BSP checkout is shallow; its HEAD commit is not the SDK release identifier.

## 3. Firmware helper inventory

The following addresses and behavior are established in earlier Phase 13 static disassembly. Offsets are file offsets in raw firmware `X`; candidate runtime addresses follow the existing image mapping used in earlier reports. No original C implementation was recovered for any of these helpers.

| Firmware file offset | Candidate runtime address | Recovered behavior | Source comparison |
|---:|---:|---|---|
| `0x1a9b2` | `0x0101a9b2` | Set/clear selected channel enable bit using channel index shifted by four | Register-model match only; source function absent |
| `0x1a9fa` | `0x0101a9fa` | Disable selected channel, write PCNT, set TIMER mode and auto-reload fields | Register-model match only; source function absent |
| `0x1aa50` | `0x0101aa50` | Set/clear selected channel TIMER-mode bit | Register-model match only; source function absent |
| `0x1aa44` | `0x0101aa44` | Set bit 4 in `SYSCTRL_CLK_CLS` | No active timer lifecycle caller established |
| `0x1d304` | `0x0101d304` | Write selected bit to NVIC ISER | Public NVIC helper correspondence; not timer driver source |
| `0x1d312` | `0x0101d312` | Write selected bit to NVIC ICER | Public NVIC helper correspondence; not timer driver source |

The timer register map is:

| Register | Address |
|---|---:|
| Timer1 PCNT | `0x000f0c08` |
| Timer2 PCNT | `0x000f0c10` |
| TIM_CTRL | `0x000f0c48` |
| TIM_CTRL1 | `0x000f0c4c` |
| Timer1 CNT | `0x000f0c54` |
| Timer2 CNT | `0x000f0c58` |

The public YC3121 header derives `TIMER_BASEADDR=0xf0c00`, `TIM_PCNT(x)=base+x*8`, `TIM_NCNT(x)=base+4+x*8`, `TIM_CTRL=base+9*8`, `TIM_CTRL1=TIM_CTRL+4`, and timer counter addresses from `0xf0c50` with four-byte channel stride. The firmware helper address arithmetic agrees with this register model. Classification: source register definitions `CONFIRMED_STATIC`; firmware-to-source implementation relationship `STRONG_STATIC_INFERENCE`, not exact source match.

## 4. Timer1 firmware lifecycle

Timer1 initialization at file offset `0x10240`:

1. Selects channel index 1.
2. Programs PCNT `0x226e` = 8,814 through helper `0x1a9fa`.
3. Sets TIMER mode through `0x1aa50`.
4. Enables channel 1 through `0x1a9b2`.
5. Enables NVIC index 14 through `0x1d304`.

The IRQ14 wrapper at file offset `0x103b0`:

1. Calls `0x1a9b2(1,0)` to clear Timer1 channel enable.
2. Calls output routine `0x135de`.
3. Calls `0x1a9b2(1,1)` to set Timer1 channel enable.
4. Returns.

This stop/output/restart order is `CONFIRMED_STATIC`. The datasheet says the current counter is loaded when the timer is enabled. A fresh PCNT reload on re-enable is `STRONG_STATIC_INFERENCE`; exact behavior of a pending request during disable/re-enable remains `UNKNOWN`.

At HCLK 96 MHz, the nominal 8,814-count interval is approximately 91.8125 microseconds (8,815 clocks would be approximately 91.8229 microseconds if the peripheral uses an inclusive convention). This is not the callback period: the timer is stopped while the output routine executes. See [clock, ROM and interrupt model](PHASE13_10_CLOCK_ROM_AND_INTERRUPT_MODEL.md).

## 5. Timer2 firmware lifecycle

Timer2 initialization at file offset `0x10214`:

1. Receives a multiplier.
2. Multiplies it by 1,000,000.
3. Configures channel index 2 through the timer helpers.
4. Enables NVIC index 15.

The startup path passes 96 for the nominal 96 MHz configuration, yielding PCNT2=96,000,000. The lower-clock path passes 12 for HCLK=12 MHz, yielding 12,000,000. Both imply a nominal one-second count if the timer clock equals HCLK and the assumed clock is active.

The IRQ15 handler at file offset `0x103c8` writes an application state byte and returns. It does not visibly stop Timer2, read a timer status register, clear a peripheral flag or clear NVIC pending state. This is `CONFIRMED_STATIC` about the handler body, not evidence that acknowledgement is unnecessary.

## 6. Timer source-to-binary comparison status

| Required source behavior | V504 binary evidence | Original source evidence | Result |
|---|---|---|---|
| Init/channel indexing | Timer1 channel 1, Timer2 channel 2 | Public register macros only | `STRONG_STATIC_INFERENCE` |
| Start/stop | Enable bit set/clear helper | No `yc_timer.c` | `UNKNOWN` at source level |
| Counter reload | PCNT writes; timer enabled | Datasheet says CNT loaded when enabled | `STRONG_STATIC_INFERENCE` |
| Auto-reload | Control nibble bit 3 set | Public header field definition | `STRONG_STATIC_INFERENCE` |
| TIMER/PWM mode | Control nibble bit 2 set/cleared | Public header field definition | `STRONG_STATIC_INFERENCE` |
| Interrupt configuration | NVIC 14/15 enabled | Public IRQ enum/NVIC definitions | `STRONG_STATIC_INFERENCE` |
| Callback registration | App dispatch table routes IRQ14/15 | No SDK registration/wrapper source | `UNKNOWN` |
| Peripheral status/ack | No visible status/clear in app handlers | No timer driver or complete timer spec | `UNKNOWN` |
| NVIC pending clear | No firmware ICPR write identified | BSP defines ICPR address but no timer-specific clear | `UNKNOWN` |
| Channel indexing validation | Firmware uses channels 1 and 2 | Public macro packing | `STRONG_STATIC_INFERENCE` |

No function can be labelled `EXACT_BINARY_MATCH` because the matching C source is absent. Semantic similarity of register arithmetic is not a byte-level source match.

## 7. Interrupt acknowledgement boundary

The public BSP defines Cortex-M0-style NVIC ISER, ICER, ISPR and ICPR addresses, including ICPR `0xe000e280`. Its generic helpers implement enable/disable; no timer driver is present. Firmware `0x1d304` and `0x1d312` correspond to enable/disable behavior. No write to ICPR was identified in the firmware image during prior analysis.

Neither IRQ14 nor IRQ15 application handler contains an explicit peripheral status read/clear or NVIC pending clear. This does not establish that the SDK wrapper or hardware does not acknowledge the event. The missing source files prevent determination of whether a shared ISR wrapper, peripheral auto-deassertion, exception-entry side effect, ROM routine or another controller mechanism is involved.

Do not treat Timer1 re-enable as acknowledgement. Do not assume Timer2's periodic behavior proves an edge-triggered request. Peripheral and NVIC pending semantics remain `UNKNOWN`.

## 8. System-control source gap

`yc_sysctrl.c/.h` were not recovered. The public BSP has `core/system.c/.h` and the SoC register header, but this is not an equivalent complete system-control SDK driver. The firmware helper at `0x1b362` selects HCLK by updating the low nibble of `SYSCTRL_HCLK_CON` at `0x000f8560`. The helper at `0x1aa44` sets bit 4 in `SYSCTRL_CLK_CLS` (timer module clock close bit in the YC3121 datasheet), but no active Timer1 lifecycle caller was established. No original C function signatures, source-level initialization ordering or peripheral gate policy can be reconstructed from the missing driver.

## 9. Evidence classifications and limitations

- Public Yichip register definitions and IRQ enum: `CONFIRMED_STATIC` as statements in that BSP source snapshot.
- Firmware helper register operations and ISR instruction order: `CONFIRMED_STATIC` as recovered from firmware disassembly.
- Mapping those helpers to the public Yichip timer model: `STRONG_STATIC_INFERENCE`.
- Exact source implementation, interrupt registration wrapper and peripheral acknowledgement: `UNKNOWN`.
- No timer source or project was recovered; absence applies only to the explicitly searched roots and inspected BSP snapshot.

No application, firmware, Rust, frontend or native driver code was modified. No hardware was accessed. Continue with [clock/ROM/interrupt model](PHASE13_10_CLOCK_ROM_AND_INTERRUPT_MODEL.md) and [findings](PHASE13_10_FINDINGS_AND_NEXT_STEPS.md).
