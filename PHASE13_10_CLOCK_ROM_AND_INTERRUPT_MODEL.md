# Phase 13.10 — Clock, ROM and Interrupt Model

## 1. Executive result

The exact V504 `yc_sysctrl.c/.h` and `yc_timer.c/.h` remain unavailable. The public YC3121 POS BSP and datasheets support a useful reference clock/register model, but do not close V504 clock-source initialization or timer interrupt acknowledgement.

- Firmware selects HCLK selector zero before timer initialization.
- Under the YC3121-D documented formula, selector zero corresponds to 96 MHz only when the required upstream clock source/PLL is active and stable.
- The firmware ROM delay thunk's literal `0x4239` matches the public Yichip `FUNC_DELAY_US_ADDR` definition (`0x4238 + 1`).
- Timer1 is disabled around the full RGB output routine, so 8,814 counts is not the RGB callback interval.
- Peripheral and NVIC acknowledgement remain `UNKNOWN`.

## 2. Available system-control/build sources

The public RT-Thread YC3121 POS BSP snapshot at commit `cbc19778d0ae5ef2533a28c0f62c6a399858010a` includes `libraries/core/system.c/.h`, `yc3121.h`, `misc.c/.h`, startup assembly and three toolchain project configurations. It does not include the original Yichip `yc_sysctrl.c/.h` or timer driver. Its project files describe that BSP, not the V504 application. See [source recovery](PHASE13_10_V504_SOURCE_PACKAGE_RECOVERY.md).

The Yichip YC3121-D V1.2 datasheet identifies HCLK as the timer clock source and describes the selector formula used in earlier phases. It documents a timer module clock close control bit in `SYSCTRL_CLK_CLS` bit 4. The timer interrupt-handling subsection does not specify pending/status acknowledgement behavior.

## 3. HCLK selector and source dependency

Firmware helper at file offset `0x1b362` (candidate runtime `0x0101b362`) updates the low nibble of `SYSCTRL_HCLK_CON` at `0x000f8560`. Startup calls this helper with argument zero at file offset `0x1028c`, before Timer1/Timer2 initialization.

The YC3121-D documented relation is:

`HCLK = 192 MHz / (HCLK_SEL + 2)`

Thus selector 0 maps to 96 MHz, selector 14 maps to 12 MHz, and those are the nominal clock assumptions used by the previous timing analysis. The selector write alone does not prove that the 192 MHz source/PLL has been initialized, locked or selected. The original V504 oscillator/PLL setup and stabilization sequence was not recovered, so actual HCLK remains conditional rather than measured.

Timer1 and Timer2 both use HCLK as timer clock in the public YC3121-D documentation. This supports calculations conditional on correct clock-source setup; it does not establish the exact physical MCU variant on the TH80 PCB.

## 4. Timer1 and Timer2 timing model

### Timer1

- Init file offset: `0x10240`.
- Channel: 1.
- PCNT: `0x226e` = 8,814.
- TIMER mode and auto-reload configured.
- NVIC index: 14.
- IRQ14 wrapper: `0x103b0`.
- Wrapper disables Timer1, calls output routine `0x135de`, then re-enables Timer1.

At nominal 96 MHz, 8,814 counts are 91.8125 microseconds (or 91.8229 microseconds for 8,815 inclusive clocks). Datasheet wording says current CNT is loaded when timer is enabled, supporting a fresh reload after re-enable as `STRONG_STATIC_INFERENCE`. It does not establish whether disable clears, masks, preserves or resets a pending request.

The callback interval includes output execution, the subsequent timer countdown, dispatch latency and any preemption. The output routine contains software loops and ROM delay calls, so the PCNT value is not a complete RGB frame/refresh period.

### Timer2

- Init file offset: `0x10214`.
- Channel: 2.
- PCNT computed as multiplier × 1,000,000.
- Startup passes 96 at nominal 96 MHz; lower-clock path passes 12 at nominal 12 MHz.
- NVIC index: 15.
- IRQ15 handler: `0x103c8`, which writes an application state byte and returns.

The nominal count is one second under the corresponding HCLK assumptions. Timer2 is a separate control case and does not resolve Timer1 pending semantics.

## 5. ROM delay API fingerprint

Public Yichip BSP file `libraries/core/rom_api.h` defines:

- `FUNC_DELAY_US_ADDR = 0x4238 + 1` = `0x4239`.
- `FUNC_DELAY_MS_ADDR = 0x425c + 1` = `0x425d`.

Firmware thunk at file offset `0x1a9a8` loads literal `0x00004239` and branches through it. The adjacent delay-ms thunk at `0x1a9ac` resolves to `0x425d`. The literal is stored at file offset `0x1ac0c`. This is an exact ROM entry address correspondence between firmware literal and public header definition; the relationship of the full V504 ROM image/revision to the public BSP remains `STRONG_STATIC_INFERENCE`.

The RGB output routine calls the delay-us thunk with branch-selected argument 7 or 15 and with argument 4; alternate branches also request 4. These are programmed API arguments. They are not measured GPIO pulse widths or total elapsed durations. No ROM implementation was recovered, and no physical timing measurement is authorized.

## 6. IRQ14 and IRQ15 dispatch

### IRQ14 / Timer1

The firmware configures NVIC index 14 and dispatches to wrapper `0x103b0`. The wrapper performs channel disable → output routine → channel re-enable. It contains no explicit NVIC disable, ICPR write or global interrupt mask. It contains no visible timer status read/clear.

### IRQ15 / Timer2

The firmware configures NVIC index 15. Handler `0x103c8` stores a state byte and returns. It contains no visible timer stop, peripheral status read/clear or NVIC pending clear.

### Public NVIC reference

The Yichip BSP's `yc3121.h` / `misc.h` model Cortex-M0 NVIC registers including:

- ISER `0xe000e100`
- ICER `0xe000e180`
- ISPR `0xe000e200`
- ICPR `0xe000e280`

Firmware helpers at `0x1d304` and `0x1d312` match enable/disable operations. No firmware ICPR write was identified in earlier static analysis. The BSP does not provide the missing timer-specific interrupt wrapper or driver source.

## 7. Peripheral versus NVIC acknowledgement

These are distinct state layers:

1. A timer peripheral event/request may have peripheral status or output semantics.
2. The interrupt controller may latch pending state independently.
3. Enabling/disabling a timer channel is not inherently equivalent to clearing either state.
4. Enabling/disabling an NVIC line is not inherently equivalent to clearing peripheral status.

Available evidence does not identify whether the timer request is pulse- or level-sensitive, whether a hidden peripheral latch exists, whether exception entry or counter reload deasserts it, or whether a shared SDK wrapper clears it. The public datasheet interrupt-handling section provides no substantive acknowledgement sequence. The missing `yc_timer.c/.h` and matching interrupt header prevent a source-level answer.

Timer2's auto-reload configuration and minimal handler make an automatically deasserting/pulsed request or implicit hardware acknowledgement plausible for intended periodic operation. This remains a `WEAK_INFERENCE`/possible explanation, not a resolved mechanism. It is not proof that the timer is edge-triggered, that no clear is needed, or that Timer1 re-enable acknowledges pending state.

**Conclusion: peripheral acknowledgement `UNKNOWN`; NVIC pending acknowledgement `UNKNOWN`.**

## 8. System-control helper and timer clock gating

Firmware helper at `0x1aa44` sets bit 4 in `SYSCTRL_CLK_CLS`. The YC3121-D datasheet identifies bit 4 as timer module clock control, where setting it closes the TIM module clock. No active caller in the recovered Timer1 lifecycle was established. It must not be interpreted as an interrupt clear or assumed safe while Timer1/Timer2 are active.

The missing system-control source leaves exact oscillator, PLL, clock-switch, stabilization, reset and low-power sequencing unresolved. Public BSP `system.c` is a related system implementation, not the missing original V504 SDK driver.

## 9. Build-provenance boundaries

The public BSP Keil project records ARMCC 5.06 update 5 build 528, and its IAR/GCC alternatives specify Cortex-M0 toolchain flags and reference linker layouts. This cannot identify the compiler used for V504 because the project is a separate RT-Thread BSP, supports multiple toolchains and has no demonstrated build relationship to the keyboard firmware. The raw V504 `X` image contains source path strings but no recovered ELF/AXF/DWARF/map/compiler stamp. Exact compiler version, optimization and linker behavior remain `UNKNOWN`.

## 10. RGB implementation boundary

Static facts available for documentation:

- Timer1 channel 1, PCNT 8,814, TIMER/auto-reload configuration and NVIC index 14.
- Timer1 stop/output/restart order.
- Timer2 channel 2, IRQ15 and nominal one-second count under clock assumptions.
- HCLK selector helper and conditional 96/12 MHz model.
- ROM delay-us entry address and programmed call arguments.
- Output routine software loops and register-write structure established in earlier phases.

Not safe to implement without additional source/documentation:

- Exact timer pending/acknowledgement semantics.
- Exact disable/re-enable behavior and pending-event retention.
- Exact MCU variant and timer peripheral revision fitted to the keyboard.
- Clock-source/PLL state and timing guarantees.
- Exact interrupt wrapper/registration behavior.
- GPIO pin mux, polarity, output enable and electrical behavior.
- Actual LED bus/latch/reset timing and sleep/wake dependencies.

No timer code, RGB control, transport, firmware or GUI code was written. No HID/USB access or hardware experiment occurred.

## 11. Cross-references

- [V504 source package recovery](PHASE13_10_V504_SOURCE_PACKAGE_RECOVERY.md)
- [Timer SDK source comparison](PHASE13_10_TIMER_SDK_SOURCE_COMPARISON.md)
- [Findings and next steps](PHASE13_10_FINDINGS_AND_NEXT_STEPS.md)
