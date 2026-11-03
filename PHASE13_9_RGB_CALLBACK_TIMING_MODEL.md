# Phase 13.9 — RGB Callback Timing Model

## 1. Executive result

The V504 Timer1 callback is not a free-running 91.8 microsecond RGB frame scheduler. Its wrapper disables Timer1 before running the entire output routine and re-enables it only after the output routine returns. The YC3121 timer documentation says that the current counter is loaded when a timer is enabled. Taken together, this strongly supports a model in which each callback's output work is followed by a fresh Timer1 countdown.

The output routine also contains explicit calls through the Yichip ROM delay-us entry. In its normal data path it requests either 7 or 15 microseconds, depending on the state byte, and then requests a further 4 microseconds. Alternate branches request 4 microseconds after their control sequence. These are programmed API delays, not a measured elapsed-time trace.

The exact RGB callback frequency remains UNKNOWN because the bit-serial GPIO-like loops, ROM delay implementation, interrupt-entry latency, bus wait states, possible higher-priority interrupt preemption and physical timer behavior have not been measured.

## 2. Startup timer and clock configuration

### 2.1 HCLK

At file offset 0x1028c (candidate runtime 0x0101028c), startup calls the HCLK selector helper 0x1b362 with argument 0 before Timer1 and Timer2 initialization.

The helper at file offset 0x1b362 (runtime candidate 0x0101b362) updates the low nibble of SYSCTRL_HCLK_CON at 0x000f8560. The YC3121-D datasheet gives:

HCLK = 192 MHz / (HCLK_SEL + 2)

Therefore selector 0 corresponds to 96 MHz under the documented oscillator/clock conditions. This is the nominal timer-clock assumption used below, not a measured keyboard clock.

### 2.2 Timer1

Timer1 init at file offset 0x10240 (runtime candidate 0x01010240):

- Passes channel index 1.
- Writes PCNT literal 0x226e = 8814 through 0x1a9fa.
- Sets TIMER mode through 0x1aa50.
- Enables channel 1 through 0x1a9b2.
- Enables NVIC index 14 through 0x1d304.

Nominal countdown at 96 MHz:

8814 / 96,000,000 = 91.8125 microseconds

If the hardware uses an inclusive N+1 convention, the corresponding value is 8815 / 96,000,000 = 91.8229 microseconds. The available datasheet does not settle the inclusive-count convention.

### 2.3 Timer2 comparison

Timer2 init at file offset 0x10214 (runtime candidate 0x01010214):

- Receives a multiplier argument.
- Multiplies it by literal 1,000,000.
- Configures channel index 2.
- Enables NVIC index 15.

Startup passes 96, yielding PCNT2=96,000,000 at HCLK=96 MHz, nominally one second. A low-clock path selects HCLK selector 14 (12 MHz) and passes 12, yielding 12,000,000 counts, also nominally one second.

Timer2 handler at 0x103c8 only writes a state byte and returns. It does not stop Timer2 or acknowledge a visible status flag. Timer2 is an independent control case; its one-second count does not prove Timer1's exact restart semantics.

## 3. IRQ14 dispatch and Timer1 wrapper

The software path is:

Timer1 channel 1 configuration
  -> NVIC index 14 enable
  -> application IRQ14 stub
  -> secondary dispatch table slot 14
  -> wrapper 0x103b0
  -> output routine 0x135de

The wrapper at file offset 0x103b0 (runtime candidate 0x010103b0) is:

| Offset | Operation |
|---:|---|
| 0x103b2 | Set r1=0 |
| 0x103b4 | Set r0=1 |
| 0x103b6 | Call 0x1a9b2(1,0), clearing Timer1 channel enable |
| 0x103ba | Call 0x135de, the complete RGB output routine |
| 0x103be | Set r1=1 |
| 0x103c0 | Set r0=1 |
| 0x103c2 | Call 0x1a9b2(1,1), setting Timer1 channel enable |
| 0x103c6 | Return |

This stop/output/restart order is CONFIRMED_STATIC. It is not a critical section: no CPSID i, NVIC IRQ14 disable, ICPR write or global mask is present in the wrapper.

## 4. Counter behavior on stop and restart

The datasheet states that when a timer is enabled, the current-count register is loaded. PCNT is the repeating reload value in TIMER mode. This makes a fresh PCNT load on the wrapper's re-enable a STRONG_STATIC_INFERENCE.

The strongest supported model is therefore:

1. Timer1 reaches its timer event and the IRQ14 path dispatches.
2. The wrapper clears Timer1's enable bit.
3. The complete output routine runs while Timer1 is disabled.
4. The wrapper sets Timer1's enable bit.
5. The timer's current count is loaded from PCNT1.
6. A new nominal countdown begins.

The exact peripheral behavior during disable remains UNKNOWN. The datasheet does not explicitly say whether disabling freezes the current count, resets it, or affects a pending request. The documented load-on-enable behavior is the reason a restart-from-PCNT model is favored, but it is not a hardware measurement.

## 5. Output routine execution and explicit timing

### 5.1 Frame preparation and bit-serial loops

The output routine at file offset 0x135de (runtime candidate 0x010135de) calls 0x13526 to prepare a 16-word buffer from data at 0x20d18, then sends 16 words through 0x134f2.

Routine 0x134f2 iterates 32 times per call and writes data-dependent values to the candidate GPIO-like addresses 0x000f872a, 0x000f872b and 0x000f8705. Sixteen calls therefore execute 512 bit iterations. Routine 0x133d2 performs a separate 16-iteration control-like bit sequence using the same candidate register area.

The exact peripheral, pin mapping, electrical roles and external LED protocol remain UNKNOWN. Do not identify this as SPI, I2C, WS2812 or another specific protocol.

### 5.2 ROM delay-us mapping

The public Yichip core/rom_api.h defines:

- FUNC_DELAY_US_ADDR = 0x4238 + 1
- FUNC_DELAY_MS_ADDR = 0x425c + 1

The firmware literal at file offset 0x1ac0c is 0x00004239. The thunk at 0x1a9a8 loads that value and branches through it. The neighboring thunk at 0x1a9ac adds 0x24 and resolves to 0x425d. This is a strong source-to-binary ROM API address match.

Within 0x135de:

- At file offset 0x1361a, the ROM delay-us thunk is called with r0 selected as 7 or 15 by the branch at 0x13610–0x13618.
- At file offset 0x13638, the same thunk is called with r0=4.
- Alternate paths call it with r0=4 at offsets 0x1369a, 0x1370e and 0x13762.

Thus the normal data path requests 7 or 15 microseconds plus a further 4 microseconds. The requested delay sum is nominally 11 or 19 microseconds. The alternate control branches request 4 microseconds. These values are source/API-backed programmed delays, not measured elapsed times.

### 5.3 0x104cc is not the delay routine

The call at 0x13608 passes argument 7 to function 0x104cc. That function iterates 16 entries, selects byte values according to the argument, and writes them through a table-derived register pointer. It contains no delay loop or call to the ROM delay-us thunk. It is a table-driven register update, not the delay_us function.

This distinction matters: the 7/15/4 microsecond waits come from the 0x1a9a8 ROM thunk, not from 0x104cc.

### 5.4 Timer0 is also gated inside the output routine

The normal and alternate output paths call 0x1a9b2(0,0), execute 0x133d2, then call 0x1a9b2(0,1). This clears and restores Timer0 channel enable around the 16-bit control-like sequence.

This is CONFIRMED_STATIC for Timer0 channel gating. It does not establish that Timer0 is the output timebase, nor does it reveal Timer0's PCNT/CNT state or IRQ13 behavior. No Timer2 call or Timer2 register access occurs inside the analyzed 0x135de path.

## 6. Callback timing equation

Let:

- T1 = nominal Timer1 countdown after re-enable, approximately 91.8125 microseconds at 96 MHz.
- Tout = complete execution time of 0x135de, including the 16-word/512-bit data path, control writes, ROM delay calls and any branch-dependent work.
- Tdispatch = interrupt entry, secondary dispatch, wrapper and return overhead not included in Tout.
- Tpreempt = any delay introduced by higher-priority interrupt service while the output routine is executing.

Under the restart-from-PCNT inference:

Tcallback-to-callback ≈ Tout + T1 + Tdispatch + Tpreempt

For the normal data path, Tout includes nominal programmed delay calls totalling 11 or 19 microseconds, depending on the state branch, plus the instruction and register-write execution time. The actual Tout is therefore greater than the programmed delay sum. For alternate branches, the explicit delay-us call is 4 microseconds plus the remaining work.

The approximate lower envelope, ignoring all code and interrupt overhead, is therefore:

- Normal branch: 91.8125 us + 11 us = 102.8125 us, or 91.8125 us + 19 us = 110.8125 us.
- Alternate branch: 91.8125 us + 4 us = 95.8125 us.

These are not callback periods or measured frequencies. They are arithmetic floors conditional on 96 MHz HCLK, nominal ROM delay behavior and the restart-from-PCNT inference. The actual callback interval is longer because the bit-serial loops and other code execute while Timer1 is stopped.

No exact RGB refresh frequency can be calculated from the available static evidence.

## 7. Other-interrupt interference

The public Yichip BSP identifies a Cortex-M0 NVIC model with two implemented priority bits. V504's startup priority setup at file offsets 0x102a0–0x102bc assigns:

- IRQ0 priority 1.
- IRQ5 priority 2.
- IRQ14 priority 3.
- IRQ15 priority 3.

The firmware also enables IRQ5 in the UART0 setup path at 0x173d0. Timer1 IRQ14 is enabled by its init function at 0x10264.

Under the matching Cortex-M0 priority model, IRQ5 has higher urgency than IRQ14 and can preempt the Timer1 output handler if its source asserts. IRQ15 is assigned the same priority as IRQ14 and would not normally preempt it. Whether UART0 actually asserts during an RGB output transaction is UNKNOWN.

The wrapper does not mask global interrupts. Therefore software bit-loop timing is not statically guaranteed to be uninterrupted. A higher-priority IRQ5 service could stretch an output bit interval and introduce jitter. This is a STRONG_STATIC_INFERENCE conditional on the exact keyboard silicon implementing the public BSP's Cortex-M0 priority model.

## 8. Timer2 as independent control case

Timer2 has a nominal one-second interval in both the 96 MHz / 96 million-count startup configuration and the 12 MHz / 12 million-count low-clock configuration. Its handler at 0x103c8 only sets a state byte. It does not share Timer1's RGB wrapper and does not stop its timer channel.

This comparison supports the interpretation that the firmware expects a timer interrupt event to be acknowledged or deasserted without a visible application-level clear. It does not identify whether that happens through an edge/pulse request, hardware auto-clear, exception-entry behavior or another undocumented mechanism.

## 9. Power-management observations

The firmware has low-clock paths that select HCLK selector 14 and scale Timer2's reload input to 12 million counts. It also has paths that disable Timer1 or NVIC IRQ14 during mode transitions. These are evidence of an intended separation between the frequent RGB output service and the slower state/timekeeping path.

The helper at 0x1aa44 sets bit 4 of SYSCTRL_CLK_CLS at 0x000f856c. The public datasheet identifies this as the timer-module clock-closure bit. No active caller in the recovered Timer1 lifecycle was found, so this is not evidence that the running keyboard gates the timer module during ordinary RGB operation. Gating the shared timer module while Timer2 is needed would be unsafe without further source evidence.

## 10. What remains uncalculable

- Actual Timer1 interrupt arrival latency and callback interval.
- Exact timer counter value at disable and exact restart edge.
- Actual elapsed delay of the ROM delay-us function on the keyboard's clock tree.
- Total instruction and bus time for the bit-serial output path.
- Actual UART0 preemption occurrence and jitter.
- Exact electrical pulse widths at the output pins.
- Whether the output writes reach a powered LED chain.
- The physical LED protocol and successful LED frame latch behavior.

## 11. Cross-references

- [V504 SDK recovery and ROM API provenance](PHASE13_9_V504_SDK_RECOVERY.md)
- [Timer interrupt acknowledgement and register map](PHASE13_9_TIMER_INTERRUPT_ACKNOWLEDGEMENT.md)
- [Findings and implementation boundary](PHASE13_9_FINDINGS_AND_IMPLEMENTATION_BOUNDARY.md)
- [Phase 13.3 LED output protocol analysis](PHASE13_3_LED_OUTPUT_PROTOCOL_ANALYSIS.md)
- [Phase 13.7 IRQ14 lifecycle analysis](PHASE13_7_IRQ14_EVENT_AND_ACKNOWLEDGEMENT.md)
