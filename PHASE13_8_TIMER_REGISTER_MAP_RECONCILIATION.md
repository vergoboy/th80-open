# Phase 13.8 — Timer Register Map Reconciliation

**Scope:** static disassembly, Yichip SoC header, and datasheet reconciliation. No firmware or hardware was executed.

**Related reports:**

- [SDK provenance](PHASE13_8_YICHIP_SDK_PROVENANCE.md)
- [Clock and interrupt semantics](PHASE13_8_CLOCK_AND_INTERRUPT_SEMANTICS.md)
- [Findings and implementation boundary](PHASE13_8_FINDINGS_AND_IMPLEMENTATION_BOUNDARY.md)

## 1. Executive conclusion

The timer register conflict is resolved. The firmware's literal at file offset `0x1ac3c` is little-endian `40 0c 0f 00`, i.e. **`0x000f0c40`**. Earlier Phase 13.7 work reversed the bytes within each 16-bit halfword and incorrectly read it as `0x000f400c`. This created false derived addresses `0x000f3fd4`, `0x000f3fdc`, `0x000f4014`, and `0x000f4018`.

The actual address calculations are:

- Timer1 PCNT: `0x000f0c40 - 0x40 + (1*8) = 0x000f0c08`.
- Timer2 PCNT: `0x000f0c40 - 0x40 + (2*8) = 0x000f0c10`.
- TIM_CTRL: `0x000f0c40 + 0x08 = 0x000f0c48`.
- TIM_CTRL1: `0x000f0c40 + 0x0c = 0x000f0c4c`.

These calculations match the Yichip `yc3121.h` macros and the detailed register tables in both YC3121-D V1.2 and YC3121-L V1.1. No second timer bank or remapping is required to explain the binary.

Classification: `CONFIRMED_STATIC` for literal bytes and instruction arithmetic; `STRONG_STATIC_INFERENCE` for the matching YC3121 register identity.

## 2. Address inventory and conflict resolution

### 2.1 Actual timer register map

The public Yichip header defines:

```c
#define TIMER_BASEADDR      0xf0c00
#define PWM_TOTAL           9
#define TIM_PCNT(x)         *(volatile int*)(TIMER_BASEADDR + x*8)
#define TIM_NCNT(x)         *(volatile int*)(TIMER_BASEADDR + 4 + x*8)
#define TIM_CTRL            *(volatile int*)(TIMER_BASEADDR + PWM_TOTAL*8)
#define TIM_CTRL1           *(volatile int*)(TIMER_BASEADDR + PWM_TOTAL*8 + 4)
#define TIM_CNT(x)          *(volatile int*)(TIMER_BASEADDR + PWM_TOTAL*8 + (PWM_TOTAL*4 + 31)/32*4 + x*4)
```

Substituting `PWM_TOTAL=9` gives:

| Register | Address | Meaning |
|---|---:|---|
| TIM0_PCNT | `0x000f0c00` | channel 0 reload / PWM high count |
| TIM1_PCNT | `0x000f0c08` | channel 1 reload / PWM high count |
| TIM2_PCNT | `0x000f0c10` | channel 2 reload / PWM high count |
| TIM1_NCNT | `0x000f0c0c` | channel 1 PWM low count |
| TIM2_NCNT | `0x000f0c14` | channel 2 PWM low count |
| TIM8_PCNT | `0x000f0c40` | channel 8 reload / PWM high count |
| TIM8_NCNT | `0x000f0c44` | channel 8 PWM low count |
| TIM_CTRL | `0x000f0c48` | four-bit fields for channels 0–7 |
| TIM_CTRL1 | `0x000f0c4c` | channel 8 control field in bits 3:0 |
| TIM0_CNT | `0x000f0c50` | current count / PWM period register |
| TIM1_CNT | `0x000f0c54` | current Timer1 count |
| TIM2_CNT | `0x000f0c58` | current Timer2 count |

Classification: `CONFIRMED_STATIC` for header arithmetic; `STRONG_STATIC_INFERENCE` for the firmware's actual silicon map because exact PCB part is unverified.

### 2.2 Firmware literal and computed accesses

The original firmware `X` has the following literal bytes:

| File offset | Raw bytes | Little-endian word | Use |
|---:|---|---:|---|
| `0x1ac3c` | `40 0c 0f 00` | `0x000f0c40` | Timer helper base literal |
| `0x1ac40` | `40 85 0f 00` | `0x000f8540` | SYSCTRL/clock helper base literal |
| `0x10490` | `40 42 0f 00` | `0x000f4240` | Timer2 reload multiplier (`1,000,000`) |
| `0x10494` | `6e 22 00 00` | `0x0000226e` | Timer1 reload (`8814`) |

Timer helper references to the `0x1ac3c` pool occur at `0x1a9ce`, `0x1aa1a`, `0x1aa28`, `0x1aa70`, `0x1aac0`, `0x1aaf0`, `0x1ab08`, `0x1ab6c`, `0x1abb4`, and `0x1abc0`. The helpers calculate channel addresses dynamically; the per-channel addresses need not appear as separate raw literals.

The false Phase 13.7 candidates `0x000f3fd4`, `0x000f3fdc`, `0x000f4014`, and `0x000f4018` are not timer references in the original binary. They resulted from misreading the halfword sequence `0x0c40 0x000f` as `0x400c 0x000f`.

The older `0x000f0c08` / `0x000f0c48` interpretation was the correct register bank. The remaining issue is not a remap but several datasheet prose errors described below.

Classification: `CONFIRMED_STATIC`.

## 3. Datasheet discrepancies

Both YC3121-D V1.2 (section 12.4, pages 64–66) and YC3121-L V1.1 (section 11.4, pages 53–56) repeat the same inconsistencies:

1. **TIM_NCNT prose base:** says `0xf0c48`, but the per-channel table gives `TIM0_NCNT=0xf0c04`, `TIM1_NCNT=0xf0c0c`, etc. The header formula `TIMER_BASEADDR + 4 + x*8` agrees with the detailed table. `0xf0c48` is TIM_CTRL, not the NCNT bank base.
2. **TIM_CTRL prose address:** says `0xf0c30`. The PCNT/NCNT table identifies `0xf0c30` as `TIM6_PCNT`; a control register there would overlap channel 6's reload value. The header computes `TIM_CTRL = 0xf0c00 + 9*8 = 0xf0c48`, and the documented `TIM_CTRL1=0xf0c4c` follows immediately. Firmware uses `0xf0c48` as the control word. The coherent correction is `TIM_CTRL=0xf0c48`.
3. **TIM_CNT prose base:** says `PWM_BASEADDR+0x50 (0xf0c34)`, but the detailed table gives `TIM0_CNT=0xf0c50`, `TIM1_CNT=0xf0c54`, etc. The header formula yields `0xf0c50 + 4*x`. `0xf0c34` is in the channel PCNT/NCNT region, not the CNT bank.
4. **Timer8 control:** both datasheets identify `TIM_CTRL1=0xf0c4c` and channel 8 control in bits 3:0. This supports the header's split control-word layout.

The same typos in two documents do not independently prove the header is correct, but the header formulas, the nine-channel layout, and firmware instruction arithmetic all agree on the corrected addresses.

Classification: `STRONG_STATIC_INFERENCE` for the corrected address interpretation; the source of the documentation transcription error remains unknown.

## 4. Four helper functions reconstructed independently

**Important correction to the task's shorthand:** the actual disassembly uses `channel*4`, not `channel*2`, in the shift calculations. For example, `0x1a9c8` is `lsls r1, r4, #2`, then `0x1a9cc` is `lsls r0, r1`. The same `#2` shift precedes the register-controlled shift in `0x1aa32–0x1aa36` and `0x1aa68–0x1aa6e`. This is a four-bit field stride and matches the original header and datasheet.

### 4.1 `0x1a9b2` — per-channel enable/disable

- File offset: `0x1a9b2`; candidate runtime: `0x0101a9b2`.
- Arguments: `r0=channel`, `r1=enabled` (1 enables, 0 disables).
- Range check: channel must be `<9` (`cmp r0,#9` at `0x1a9b8`).
- Field mask: `1 << (channel*4)` (`lsls r1,r4,#2` at `0x1a9c8`; `lsls r0,r1` at `0x1a9cc`).
- Base literal: `0x000f0c40` loaded from pool `0x1ac3c`.
- Channels 0–7: read/modify/write `[base+8] = TIM_CTRL` (`0x1a9d8–0x1a9dc` for clear; `0x1a9e4–0x1a9e8` for set).
- Channel 8: read/modify/write `[base+0xc] = TIM_CTRL1`, bit 0 (`0x1a9ea–0x1a9f6`).
- This helper does not touch NVIC ISER/ICER/ICPR.
- Header match: `TIM_CTRL_ENABLE=0`, meaning bit 0 of each four-bit field.

Classification: `CONFIRMED_STATIC` for code; `STRONG_STATIC_INFERENCE` for naming it the timer enable bit.

### 4.2 `0x1a9fa` — configure channel reload and auto-reload/mode field

- File offset: `0x1a9fa`; candidate runtime: `0x0101a9fa`.
- Argument: pointer to a small configuration structure; byte 0 is channel, word at `+4` is reload value.
- Range check: channel `<9`.
- First calls `0x1a9b2(channel,0)` to disable the channel (`0x1aa10–0x1aa16`).
- Loads `0x000f0c40` from pool `0x1ac3c`.
- Computes `base - 0x40 + channel*8` (`0x1aa1a–0x1aa22`) and stores the supplied reload word (`0x1aa22–0x1aa24`). This equals `TIMER_BASEADDR + channel*8`, i.e. `TIM_PCNT(channel)`.
- ORs `0xC << (channel*4)` into TIM_CTRL or TIM_CTRL1 (`0x1aa28–0x1aa40`). This sets field bits 2 and 3: TIMER mode and auto-reload.
- The function does not write TIM_NCNT or TIM_CNT.

Classification: `CONFIRMED_STATIC` for arithmetic and stores; `STRONG_STATIC_INFERENCE` for the field names based on Yichip definitions.

### 4.3 `0x1aa50` — select TIMER/PWM mode bit

- File offset: `0x1aa50`; candidate runtime: `0x0101aa50`.
- Arguments: `r0=channel`, `r1=mode boolean`.
- Computes `mask = 0x4 << (channel*4)` (`0x1aa68–0x1aa6e`).
- For channels 0–7, RMW TIM_CTRL at `0x000f0c48`; for channel 8, RMW TIM_CTRL1 at `0x000f0c4c`.
- If `r1==1`, sets the bit; if `r1==0`, clears it (`0x1aa72–0x1aaa6`).
- Header match: `TIM_CTRL_MODE=2`, bit 2 of each four-bit field; 1 selects TIMER mode, 0 selects PWM mode.

Classification: `CONFIRMED_STATIC` for code; `STRONG_STATIC_INFERENCE` for semantic name.

### 4.4 `0x1aa44` — global timer module clock gate disable

- File offset: `0x1aa44`; candidate runtime: `0x0101aa44`.
- Loads `0x000f8540` from pool `0x1ac40`.
- Reads `[base+0x2c] = 0x000f856c = SYSCTRL_CLK_CLS` (`0x1aa46`).
- ORs `0x10` and writes back (`0x1aa48–0x1aa4c`).
- Yichip header: `SYSCTRL_CLK_CLS = CLKGEN_BASEADDR + 0x0c`; `disable_clock(id)` ORs `1<<id`.
- Datasheet: `SYSCTRL_CLK_CLS[4]` is the TIM module clock gate; 0 opens the timer module clock, 1 closes it. It specifically documents bit 4 as affecting Timer1/Timer2/Timer3 system clocks.
- This is a global module clock gate, not the per-channel enable bit manipulated by `0x1a9b2`.
- No direct BL caller or raw Thumb function-pointer reference to `0x1aa45` was found. Thus its implementation semantics are established, but active use by the application is not.

Classification: `CONFIRMED_STATIC` for address and bit write; `STRONG_STATIC_INFERENCE` for the TIM gate meaning; `UNKNOWN` for runtime reachability/use.

## 5. Timer1 and Timer2 setup

### 5.1 Timer1

- Initialization function: file offset `0x10240`, candidate runtime `0x01010240`.
- Loads literal `0x226e` from file offset `0x10494` and passes channel 1 to `0x1a9fa`.
- `0x1a9fa` writes `TIM_PCNT(1)=0x000f0c08` and sets bits 6–7 of TIM_CTRL.
- Calls `0x1aa50(1,1)` to set TIMER mode bit 6.
- Calls `0x1a9b2(1,1)` to set enable bit 4.
- Calls NVIC helper `0x1d304(14)` to enable IRQ14.
- Timer1's field in TIM_CTRL is bits `[7:4]`: bit4 enable, bit5 start level, bit6 TIMER mode, bit7 auto-reload.

Classification: `CONFIRMED_STATIC` for init sequence and values; `STRONG_STATIC_INFERENCE` for register semantics.

### 5.2 Timer2

- Initialization function: file offset `0x10214`, candidate runtime `0x01010214`.
- Loads multiplier `0x000f4240` (`1,000,000`) from file offset `0x10490`; multiplies by caller's `r0` argument.
- Stores reload into `TIM_PCNT(2)=0x000f0c10` via `0x1a9fa`.
- Calls `0x1aa50(2,1)`, `0x1a9b2(2,1)`, and NVIC helper `0x1d304(15)`.
- Timer2's field in TIM_CTRL is bits `[11:8]`: bit8 enable, bit9 start level, bit10 TIMER mode, bit11 auto-reload.
- Main startup passes `0x60` (96), yielding `96,000,000` counts. A low-clock transition passes `0x0c` (12), yielding `12,000,000` counts.

Classification: `CONFIRMED_STATIC` for code and values; `STRONG_STATIC_INFERENCE` for the register/IRQ mapping.

### 5.3 Independent source match

The Yichip `misc.h` enum maps `TIM1_IRQn=14` and `TIM2_IRQn=15`. The timer helper's four-bit fields align channel 1 and channel 2 to the same channel numbers used by the interrupt enum. This is a strong independent consistency check, but does not prove physical interrupt delivery.

## 6. Timer counter, reload, and control semantics

The datasheets state:

- timer unit contains nine independent timers;
- each has its own interrupt source;
- counters count downward;
- timer clock equals HCLK;
- when enabled, the current `TIM_CNT` value is loaded;
- in TIMER mode, interrupt is enabled by default;
- PCNT is the repeating reload value;
- when auto-reload bit 3 is set, reaching zero reloads from PCNT.

`TIM_CNT(x)` is the current count register in timer mode and is read-only in the detailed table. The Yichip header's formula yields Timer1 `TIM_CNT=0x000f0c54` and Timer2 `TIM_CNT=0x000f0c58`. The firmware initialization helpers write PCNT, not CNT. The hardware loading of CNT when channel enable is asserted is described in the datasheet; whether a disable/re-enable transition reloads immediately or on the next timer clock edge is not specified precisely.

The four-bit control field is:

| Field bit | Meaning | Firmware evidence |
|---:|---|---|
| 0 | channel/module enable | `0x1a9b2` set/clear mask `1 << channel*4` |
| 1 | initial output level | no assignment by Timer1/Timer2 init helpers |
| 2 | mode: 0 PWM, 1 TIMER | `0x1aa50` set/clear `4 << channel*4` |
| 3 | auto-reload; in TIMER mode reload PCNT at zero | `0x1a9fa` ORs `0xC << channel*4` |

Classification: `STRONG_STATIC_INFERENCE` for semantics due independent source and datasheet match; `CONFIRMED_STATIC` for observed writes.

## 7. Interrupt acknowledgement boundary

Neither the public Yichip BSP nor the inspected datasheet provides a timer-specific status/clear register or a `yc_timer.c` interrupt-clear implementation. Datasheet section 12.2.2 (D) / 11.2.2 (L), titled interrupt handling, contains no usable flag-clear procedure. The timer register tables list PCNT, NCNT, CTRL/CTRL1, and CNT only.

The SDK header defines NVIC_ISER, NVIC_ICER, NVIC_ISPR, and NVIC_ICPR. Firmware enables Timer1/2 with NVIC helper index 14/15, but no write to NVIC_ICPR (`0xe000e280`) was found in the firmware disassembly. The RGB wrapper only clears/re-sets the per-channel enable bit; it does not clear NVIC pending state.

Therefore:

- source event: timer count reaches zero in TIMER mode — `STRONG_STATIC_INFERENCE`;
- source flag existence/clear method: `UNKNOWN`;
- automatic hardware acknowledgement vs explicit source acknowledgement: `UNKNOWN`;
- NVIC pending state after channel disable: `UNKNOWN`;
- wrapper re-enable does not establish acknowledgement.

See [clock and interrupt semantics](PHASE13_8_CLOCK_AND_INTERRUPT_SEMANTICS.md) for the full boundary.

## 8. Implementation implications

The register map, channel stride, control field, PCNT reload, CNT readback address, HCLK dependency, module clock gate, and IRQ index mapping are sufficiently supported to document an independent implementation target. They are **not sufficient to safely implement a live hardware driver** until the exact MCU revision and interrupt acknowledgement semantics are confirmed. The minimum remaining requirements are listed in [findings and implementation boundary](PHASE13_8_FINDINGS_AND_IMPLEMENTATION_BOUNDARY.md).
