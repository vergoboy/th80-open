# Phase 13.8 — Clock and Interrupt Semantics

**Scope:** static firmware clock-flow, Yichip SoC definitions, timer semantics, and interrupt acknowledgement boundary. No hardware testing was performed.

**Related reports:**

- [SDK provenance](PHASE13_8_YICHIP_SDK_PROVENANCE.md)
- [Timer register map reconciliation](PHASE13_8_TIMER_REGISTER_MAP_RECONCILIATION.md)
- [Findings and implementation boundary](PHASE13_8_FINDINGS_AND_IMPLEMENTATION_BOUNDARY.md)

## 1. Executive conclusion

The clock analysis is materially stronger than Phase 13.7. Firmware function `0x1b362` writes the low four-bit HCLK selector field at `SYSCTRL_HCLK_CON=0x000f8560`. The main startup path calls it with argument zero at file offset `0x1028c`, before Timer1 and Timer2 are initialized. The YC3121 datasheet formula gives HCLK = 192 MHz / (selector + 2), so selector zero corresponds to **96 MHz**.

The firmware also switches to selector 14 (12 MHz) in selected low-clock paths and scales Timer2's reload argument from 96 to 12 accordingly. This strongly supports an intended one-second Timer2 interval in both clock states.

Timer1 reload `8814` at 96 MHz gives a nominal countdown of about 91.8 μs. This is not the observed RGB callback interval: the IRQ14 wrapper disables Timer1 while the RGB output routine runs, then re-enables it. The actual interval includes that disabled execution time and potentially reload/restart behavior.

Timer interrupt generation is documented at a high level, but peripheral acknowledgement remains unresolved. No timer status/clear register or clear sequence is present in the available header/datasheet, and no NVIC pending-clear write was found in the firmware.

## 2. Clock register source model

The Yichip `yc3121.h` header defines:

- `CLKGEN_BASEADDR = 0x000f8560`;
- `SYSCTRL_HCLK_CON = *(volatile int*)0x000f8560`;
- for POS_CHIP, `CPU_MHZ = 192,000,000 / ((SYSCTRL_HCLK_CON & 0x0f) + 2)`.

The YC3121-D V1.2 datasheet section 3.10.7, page 28, independently identifies `SYSCTRL_HCLK_CON` at `SYSCTRL_BASEADDR+0x60 = 0x000f8560`, describes selector bits `[3:0]`, and gives the same formula. It documents reset selector value `3` and dynamic-divider selection bit 5 reset to zero. YC3121-L V1.1 gives the same formula and reset defaults.

The datasheet's clock overview describes a 192 MHz internal oscillator and a DPLL `/2` 96 MHz system clock. The HCLK selector formula is the directly relevant equation for the timer block.

Classification: `CONFIRMED_STATIC` for header and datasheet definitions; `STRONG_STATIC_INFERENCE` for applying them to the firmware's active clock.

## 3. Firmware HCLK selector implementation

### 3.1 Function `0x1b362`

- File offset: `0x1b362`; candidate runtime: `0x0101b362`.
- Saves argument in `r4` (`0x1b364`).
- Accepts selector values 0, 2, 6, or 14; other values branch to an error/reporting path (`0x1b366–0x1b37c`).
- For values 2, 6, and 14: loads base `0x000f8540`, reads `[base+0x20] = 0x000f8560`, clears the low nibble by shifting right/left four bits, ORs the requested value, and writes it back (`0x1b380–0x1b38c`).
- For selector zero: takes a special ROM-assisted path (`0x1b38e–0x1b39a`) and calls `0x1b34a`; that helper also writes the HCLK selector field to zero at `0x000f8560` (`0x1b354–0x1b35e`). The ROM-assisted sub-operation is not fully identified, but the HCLK field write is explicit.

The field update preserves all HCLK register bits `[31:4]` and replaces `[3:0]` with the selector argument.

Classification: `CONFIRMED_STATIC` for field arithmetic and writes; `STRONG_STATIC_INFERENCE` for mapping the field to HCLK_SEL using the Yichip header/datasheet.

### 3.2 Startup ordering

At file offset `0x1028a`, the firmware loads `r0=0`; at `0x1028c` it calls `0x1b362`. Timer1 initialization follows at `0x102f8`, and Timer2 initialization follows at `0x102fe`. Thus the initial timer configuration occurs after the selector-zero write.

The relevant candidate runtime addresses are:

| Operation | File offset | Candidate runtime |
|---|---:|---:|
| HCLK selector zero call | `0x1028c` | `0x0101028c` |
| Timer1 init | `0x10240` | `0x01010240` |
| Timer2 init | `0x10214` | `0x01010214` |
| Timer1 init call from startup | `0x102f8` | `0x010102f8` |
| Timer2 init call from startup | `0x102fe` | `0x010102fe` |

The initial Timer1 and Timer2 configuration therefore runs with the firmware-selected 96 MHz HCLK, subject to the clock-source initialization succeeding as intended.

Classification: `CONFIRMED_STATIC` for call order and selector write; `STRONG_STATIC_INFERENCE` for the resulting 96 MHz timer clock.

### 3.3 Dynamic clock states

The same HCLK selector function has callsites with immediate arguments:

| File offset | Argument | HCLK from documented formula | Context |
|---:|---:|---:|---|
| `0x1028c` | 0 | 96 MHz | application startup, before Timer1/2 init |
| `0x1095c` | 14 | 12 MHz | runtime mode transition |
| `0x10a9e` | 0 | 96 MHz | return to active clock |
| `0x10de8` | 0 | 96 MHz | runtime transition |
| `0x11606` | 14 | 12 MHz | lower-clock path; Timer2 reload set to 12 million |
| `0x1167a` | 0 | 96 MHz | active path; Timer2 reload set to 96 million and Timer1 reinitialized |
| `0x116f0` | 0 | 96 MHz | active path; Timer2 reload set to 96 million and Timer1 reinitialized |
| `0x1173a` | 0 | 96 MHz | active path; Timer2 reload set to 96 million and Timer1 reinitialized |
| `0x117bc` | 14 | 12 MHz | lower-clock path; Timer2 reload set to 12 million |
| `0x1b5e0` | 14 | 12 MHz | clock-state helper |
| `0x1bfac` | 0 | 96 MHz | return to active clock |

The source formula supports selector 2 = 48 MHz and selector 6 = 24 MHz, but the listed application callsites use only 0 and 14. No claim is made that 48 or 24 MHz is used by this firmware. The lower-clock paths visibly rescale Timer2, but this static review does not establish whether Timer1 remains enabled throughout every low-clock transition; do not assume its 91.8 μs nominal interval remains valid at 12 MHz.

Classification: `CONFIRMED_STATIC` for call arguments and writes; `STRONG_STATIC_INFERENCE` for the frequencies under the datasheet formula.

## 4. Timer module clock gate

The timer module clock gate is separate from per-channel enable:

- `SYSCTRL_CLK_CLS = 0x000f856c` (`CLKGEN_BASEADDR + 0x0c`).
- Bit 4 is the TIM module clock control: 0 = clock open, 1 = clock closed.
- The datasheet says bit 4 controls the Timer1/Timer2/Timer3 system clocks; external timer clocks are separately controlled.
- Reset value for bit 4 is zero (open).

Firmware helper `0x1aa44` at file offset `0x1aa44` loads base `0x000f8540`, reads `[+0x2c]`, ORs `0x10`, and writes it back. This is a direct implementation of disabling the TIM module clock. However, no direct call to `0x1aa44` and no raw pointer reference to its Thumb entry was found. It is not part of the observed Timer1 wrapper path.

The active Timer1 wrapper instead calls `0x1a9b2(1,0)` and `0x1a9b2(1,1)`, modifying only channel 1 enable bit 4 in TIM_CTRL. It does not gate the entire timer module clock.

Classification: `CONFIRMED_STATIC` for the register write; `STRONG_STATIC_INFERENCE` for clock-gate semantics; `UNKNOWN` for runtime reachability of the standalone helper.

## 5. Timer1 and Timer2 count semantics

### 5.1 Datasheet behavior

YC3121-D V1.2 section 12, pages 63–66, and YC3121-L V1.1 section 11, pages 52–56, document:

- nine independent down-counters;
- one independent interrupt source per timer;
- timer clock equal to HCLK;
- TIMER mode has interrupt enabled by default;
- PCNT is the repeated reload value;
- when auto-reload is set, reaching zero reloads from PCNT;
- TIM_CNT is the current count in TIMER mode and is loaded when the timer is enabled.

The detailed CNT table places Timer1 current count at `0x000f0c54` and Timer2 at `0x000f0c58`. The firmware initialization writes PCNT, not CNT.

### 5.2 Timer1 nominal count interval

- PCNT1 = `0x226e` = 8,814 (file literal offset `0x10494`).
- Timer1 is initialized after HCLK selector 0, giving 96 MHz under the documented formula.
- Nominal count interval: `8814 / 96,000,000 = 91.8125 μs`.
- If the hardware counts inclusively through zero, the alternative is `8815 / 96,000,000 = 91.8229 μs`.

The datasheet does not explicitly settle the inclusive endpoint convention, so report the nominal interval as approximately 91.8 μs and retain the one-tick ambiguity. This is not a measured frequency.

Classification: `STRONG_STATIC_INFERENCE`.

### 5.3 Timer2 nominal interval and clock compensation

Timer2 init at file offset `0x10214` multiplies its caller argument by `0x000f4240` (1,000,000).

- Startup call at `0x102fc`: argument `0x60` = 96, yielding 96,000,000 counts at 96 MHz: nominally one second.
- Lower-clock path at `0x11612` / `0x117c8`: argument `0x0c` = 12, yielding 12,000,000 counts at 12 MHz: nominally one second.
- Active paths at `0x11684`, `0x116fa`, and `0x1174a` pass 96 after selecting HCLK selector zero.

This is strong evidence that Timer2's reload is intentionally scaled with HCLK to maintain a one-second nominal count interval.

Classification: `CONFIRMED_STATIC` for the immediate arguments and multiplication; `STRONG_STATIC_INFERENCE` for the one-second timing intent.

## 6. Timer1 wrapper and callback timing

The IRQ14 wrapper at file offset `0x103b0`:

1. calls `0x1a9b2(1,0)` to clear Timer1 channel enable;
2. calls `0x135de` (the existing RGB/output routine);
3. calls `0x1a9b2(1,1)` to set Timer1 channel enable again.

It does not call NVIC disable/enable, NVIC pending clear, or the global TIM clock-gate helper.

The datasheet says enabling a timer loads TIM_CNT and PCNT is the reload value. This supports the inference that re-enabling Timer1 restarts the countdown from its reload rather than resuming a partially elapsed count. The exact edge timing of the reload is not specified. The callback interval is therefore not just 91.8 μs: it includes the time spent in `0x135de` while the timer is disabled, plus the next countdown interval.

This is not proof that `0x135de` emits a particular electrical LED waveform. It only explains the software timer schedule around the existing output routine.

Classification: `CONFIRMED_STATIC` for the disable/callback/enable sequence; `STRONG_STATIC_INFERENCE` for reload-on-enable behavior; `UNKNOWN` for exact callback spacing and physical output timing.

## 7. Interrupt generation and acknowledgement

### 7.1 Source and NVIC mapping

The Yichip `misc.h` interrupt enum defines `TIM1_IRQn=14` and `TIM2_IRQn=15`. Firmware Timer1 init enables NVIC index 14; Timer2 init enables index 15. The existing application dispatch table maps those entries to `0x103b0` and `0x103c8` respectively.

The datasheet says timer channels have independent interrupt sources and TIMER mode enables interrupts by default. This supports the event path `channel counter reaches zero → timer IRQ source → NVIC IRQ index`.

Classification: `CONFIRMED_STATIC` for enum, init arguments, and software dispatch; `STRONG_STATIC_INFERENCE` for peripheral event-to-NVIC routing.

### 7.2 Peripheral acknowledgement

No timer status/flag/clear register is listed in the available YC3121-D or YC3121-L timer register tables. The timer interrupt-handling subsection is blank in both extracted PDFs. The public YC3121 BSP does not include `yc_timer.c`, and its timer register header contains PCNT, NCNT, CTRL, CTRL1, and CNT definitions but no timer IRQ status or clear macro.

Firmware helper `0x1a9b2` modifies channel enable only. The RGB wrapper has no visible status read, W1C/W0C write, or source acknowledgement. The `0x1aa44` module clock-gate helper is not called from the wrapper and is not an acknowledgement operation.

Thus source flag acknowledgement remains `UNKNOWN`. It is not valid to infer that disabling/re-enabling the counter clears an interrupt flag.

### 7.3 NVIC pending state

The Yichip header defines:

- `NVIC_ISER = 0xe000e100`;
- `NVIC_ICER = 0xe000e180`;
- `NVIC_ISPR = 0xe000e200`;
- `NVIC_ICPR = 0xe000e280`.

Its `enable_intr()` writes ISER; `disable_intr()` writes ICER. Firmware's Timer1/2 init calls the NVIC helper with 14/15. No ICPR write or explicit pending-clear sequence was found in the firmware disassembly. A pending NVIC request may therefore survive a peripheral channel disable; actual hardware behavior and exception-acceptance semantics require the exact core implementation.

Classification: `CONFIRMED_STATIC` for header addresses and firmware absence of an identified clear path; `UNKNOWN` for actual pending behavior at runtime.

## 8. What is and is not established

| Question | Result | Classification |
|---|---|---|
| Timer clock source | HCLK | `CONFIRMED_STATIC` in Yichip datasheets; `STRONG_STATIC_INFERENCE` for firmware silicon |
| Startup HCLK selector | firmware writes 0 before Timer1/2 init | `CONFIRMED_STATIC` |
| Startup HCLK | 96 MHz under documented 192 MHz formula | `STRONG_STATIC_INFERENCE` |
| Low-clock selector | firmware writes 14 (12 MHz) in selected paths | `CONFIRMED_STATIC` write; `STRONG_STATIC_INFERENCE` frequency |
| Timer1 nominal count | ~91.8 μs at 96 MHz | `STRONG_STATIC_INFERENCE` |
| Timer2 nominal count | ~1 second with reload scaled 96M/12M | `STRONG_STATIC_INFERENCE` |
| Actual RGB callback period | includes timer-disabled callback duration | `UNKNOWN` quantitatively |
| Timer source acknowledgement | no documented or observed clear mechanism | `UNKNOWN` |
| NVIC pending clear | no firmware ICPR write found | `UNKNOWN` runtime pending semantics |
| Physical IRQ delivery | not observed | `UNKNOWN` |

## 9. Hardware boundary

A future software implementation can model the documented counter, control, HCLK, and NVIC setup without inventing register names. It still must not implement timer ISR acknowledgement until the exact silicon revision's interrupt behavior is confirmed. It must also not treat the nominal 91.8 μs countdown as the RGB frame rate.

The minimum additional requirements are listed in [findings and implementation boundary](PHASE13_8_FINDINGS_AND_IMPLEMENTATION_BOUNDARY.md).
