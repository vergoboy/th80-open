# Phase 13.9 — Findings and Implementation Boundary

## 1. Executive summary

Phase 13.9 recovered the closest publicly verifiable Yichip source lineage and improved the RGB timing model, but did not recover the original V504 SDK package or resolve timer interrupt acknowledgement.

The most important new finding is a direct ROM API fingerprint: the V504 thunk at file offset 0x1a9a8 branches through literal 0x00004239, matching the Yichip SDK's FUNC_DELAY_US_ADDR definition. The RGB output routine calls this thunk with branch-selected 7- or 15-microsecond arguments and with a 4-microsecond argument. Combined with the Timer1 stop/output/restart wrapper, this proves that the nominal 91.8-microsecond reload is not the RGB callback interval.

## 2. Strongest discoveries

### 2.1 SDK provenance

- Firmware source-path strings establish Yichip SDK lineage.
- The public Yichip YC3121 repository contains only README.md in the inspected public state.
- The closest public Yichip-authored source is the RT-Thread YC3121 POS BSP at commit cbc19778d0ae5ef2533a28c0f62c6a399858010a.
- That BSP contains the matching timer register formulas, channel bit definitions, IRQ enum and Cortex-M0/NVIC source.
- It does not contain yc_timer.c, yc_timer.h, yc_sysctrl.c or yc_sysctrl.h.
- Component source banners are V1.0 with dates spanning 2018–2020; they do not identify one SDK package release.

Conclusion: matching Yichip/YC3121 register and ROM API lineage is STRONG_STATIC_INFERENCE; exact V504 SDK identity remains UNKNOWN.

### 2.2 Corrected timer map

| Register | Address |
|---|---:|
| Timer1 PCNT | 0x000f0c08 |
| Timer2 PCNT | 0x000f0c10 |
| TIM_CTRL | 0x000f0c48 |
| TIM_CTRL1 | 0x000f0c4c |
| Timer1 CNT | 0x000f0c54 |
| Timer2 CNT | 0x000f0c58 |

TIM_CTRL packs channels 0–7 in four-bit nibbles. TIM_CTRL1 is the four-bit control field for channel 8, not a shared interrupt register. Bits 0–3 of each channel nibble are enable, start level, TIMER/PWM mode and auto-reload.

The Phase 13.7 alternative 0xf3fdx/0xf401x timer map resulted from reversing bytes within a little-endian halfword. The actual literal at file offset 0x1ac3c is 0x000f0c40.

### 2.3 Interrupt acknowledgement

- No Timer1/Timer2 status read or peripheral clear write is identified in V504.
- No write to NVIC ICPR at 0xe000e280 is identified.
- The Timer1 wrapper disables the channel, runs the output routine and re-enables the channel.
- The Timer2 handler only writes an application state byte and returns.
- The public BSP has explicit clear functions for watchdog and DMA but no timer driver source.
- Both YC3121 datasheets omit substantive interrupt acknowledgement details.

Conclusion: the exact peripheral and NVIC pending lifecycle remains UNKNOWN. Timer2's design makes automatic deassertion or an implicit acknowledgement plausible, but does not prove the mechanism.

### 2.4 Timer1 callback timing

The Timer1 wrapper at 0x103b0 disables channel 1, invokes 0x135de and re-enables channel 1. The datasheet says the current counter is loaded when a timer is enabled. A fresh PCNT1 countdown after the wrapper re-enables Timer1 is therefore a STRONG_STATIC_INFERENCE.

At HCLK=96 MHz and PCNT1=8814, the nominal countdown is approximately 91.8125 microseconds. The output routine itself contains ROM delay-us calls requesting 7 or 15 microseconds and a further 4 microseconds in the normal data path, or 4 microseconds in alternate paths. It also executes 16 32-bit bit loops, a 16-bit control loop, table-driven register writes and Timer0 channel gating.

The actual callback interval is therefore approximately:

Toutput + 91.8125 us + dispatch/interrupt overhead + any preemption delay.

No exact callback frequency or physical LED refresh rate is established.

## 3. Evidence classifications

| Finding | Classification | Limitation |
|---|---|---|
| Yichip source-path lineage in firmware | CONFIRMED_STATIC | Does not identify package revision |
| Public YC3121 timer register formulas and field positions | CONFIRMED_STATIC for source definitions | Exact keyboard MCU still unconfirmed |
| Firmware helper register arithmetic matches the public timer model | STRONG_STATIC_INFERENCE | Exact yc_timer.c absent |
| Timer1 channel stop/output/restart sequence | CONFIRMED_STATIC | Does not prove pending-flag behavior |
| Timer CNT reload on enable | STRONG_STATIC_INFERENCE | Datasheet wording does not explain every stop/restart edge case |
| ROM delay-us thunk mapping | STRONG_STATIC_INFERENCE | Public ROM API map matches literal; actual silicon timing unmeasured |
| Timer1 output routine's programmed delays and bit-loop counts | CONFIRMED_STATIC | Actual elapsed duration unknown |
| Timer interrupt acknowledgement | UNKNOWN | Requires exact source or authoritative peripheral documentation |
| Physical RGB protocol and successful LED signaling | UNKNOWN | Requires separately authorized hardware evidence |

Use only these classifications in subsequent reports: CONFIRMED_STATIC, STRONG_STATIC_INFERENCE, WEAK_INFERENCE, FRONTEND_ONLY and UNKNOWN.

## 4. Safe future native RGB implementation boundary

### 4.1 What can be specified statically

The following can be documented as a firmware reference model without writing or executing a hardware driver:

- Candidate timer register addresses and four-bit control fields.
- Timer1 channel index 1, PCNT=8814, TIMER mode and auto-reload.
- Timer1's NVIC index 14 mapping through the application dispatch table to wrapper 0x103b0.
- Timer1 channel disable before output and enable after output.
- Timer2 channel index 2, IRQ15 and nominal one-second count in the two recovered clock configurations.
- The candidate HCLK selector values and nominal 96 MHz / 12 MHz timer-clock assumptions.
- The software bit-loop lengths, candidate output register writes and ROM delay-us call arguments.
- The existence of Timer0 channel gating around the 16-bit control-like sequence.

These are reverse-engineering specifications, not authorization to execute the functions or access the physical device.

### 4.2 What is not safe to implement yet

Do not implement physical timer or RGB output control based only on this evidence:

- Peripheral interrupt acknowledgement and pending-state clearing.
- Exact stop/restart semantics and pending-event retention.
- Exact MCU variant and timer peripheral revision on the keyboard PCB.
- Exact timer interrupt routing and request polarity.
- GPIO pin mux, output-enable and electrical polarity.
- The physical LED controller, bus protocol and latch/reset requirements.
- The output timing under actual CPU/bus clock and interrupt load.
- Sleep/wake interactions and whether Timer2 must remain active during RGB-off states.

### 4.3 Minimum evidence required before a Rust RGB driver

At minimum, obtain:

1. The exact V504 yc_timer.c/yc_timer.h and yc_sysctrl.c/yc_sysctrl.h, or an independently verified source revision that matches the V504 helper binaries.
2. An authoritative timer/NVIC description for the exact keyboard MCU, including request polarity, pending status and clear semantics.
3. Exact stop/restart counter behavior, including whether disable affects a pending request and whether re-enable loads PCNT.
4. Exact PCB MCU identification and pin-mux mapping for the candidate 0xf8700/0xf8720 output region.
5. The output routine's clock and preemption constraints, including whether IRQ5 may interrupt the GPIO bit loop.
6. A separately authorized hardware validation plan before any transport or device-control implementation is attempted.

No hardware testing or transport is authorized by this phase.

## 5. Power-management opportunities

Static firmware evidence suggests several future optimization questions:

- Timer1 is the frequent RGB output source; mode-transition paths disable it and/or IRQ14. A native engine could eventually avoid unnecessary RGB callbacks when the renderer is inactive, but only after confirming state transitions and wake requirements.
- Timer2 is configured to preserve an approximately one-second interval when HCLK is reduced from 96 MHz to 12 MHz. This supports a lower-clock housekeeping timebase design.
- The SYSCTRL_CLK_CLS bit-4 helper can close the timer module clock, but it has no confirmed active caller in the recovered Timer1 lifecycle. It must not be used while Timer2 or another timer channel is required.
- Timer0 is toggled around the 16-bit output control sequence. Its role must be identified before optimizing or removing that gate.

These are investigation opportunities, not implementation recommendations.

## 6. Single most valuable next investigation

Recover the exact V504 SDK archive/build project, prioritizing the original yc_timer.c/yc_timer.h and matching SoC timer/NVIC headers. This is the only source likely to settle the timer helper semantics and peripheral acknowledgement without guessing from an incomplete public BSP or relying on hardware experiments.

If the original package is unobtainable, search for a preserved Yichip YC3121 SDK archive whose file hashes, source banners and function-level disassembly match the V504 image. Do not substitute the YC16xx SDK or a different Yichip family.

## 7. Integrity and project hygiene

Protected original artifacts:

| Artifact | Expected SHA-256 |
|---|---|
| Firmware X | 4b22336ec9ffaaac3372791a1154a2c2a833e3305a4cadbbdcab9184252c0520 |
| Native vendor iot_driver | 22c5dc67ba77e2373a5ffdbb0312ed8c8e73eb16e78037764eb676a030b2724b |
| Frontend dist/index.js | 7d53737095fe5756fb0a18d5aecb7593555b9bb61a9d5d160ae0b988694c1e87 |

Post-analysis validation confirmed all three hashes above. Phase 13.1–13.5 report files match the saved baseline manifest at /tmp/th80-phase13-6/phase13-reports-before.sha256. Phase 13.6–13.8 reports were read only and were not edited during this phase; no pre-Phase-13.9 hash manifest for those later reports was present, so a cryptographic before/after comparison for those specific files is unavailable.

The repository has no commits and no configured remotes. No tracked files were modified and nothing was staged. The four Phase 13.9 reports are the only files created by this phase; all pre-existing untracked project files were left in place.

## 8. Cross-references

- [V504 SDK recovery and provenance](PHASE13_9_V504_SDK_RECOVERY.md)
- [Timer interrupt acknowledgement and register map](PHASE13_9_TIMER_INTERRUPT_ACKNOWLEDGEMENT.md)
- [RGB callback timing model](PHASE13_9_RGB_CALLBACK_TIMING_MODEL.md)
- [Phase 13.8 findings and implementation boundary](PHASE13_8_FINDINGS_AND_IMPLEMENTATION_BOUNDARY.md)
