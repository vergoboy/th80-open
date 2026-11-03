# Phase 13.6 — IRQ14 Peripheral Source Identification

## Executive finding

The strongest source attribution for external interrupt index 14 is **Timer1 count-to-zero/underflow**, configured in timer mode with auto-reload. This is a `STRONG_STATIC_INFERENCE`, not a dynamic observation: the firmware configures Timer1 channel 1, enables NVIC index 14 immediately afterward, stores a Timer1-specific callback at application table slot 14, and the callback disables/re-enables the same timer channel around the RGB output routine.

This attribution is conditional on the Yichip timer-block interpretation. The TH80-family YC3121-L identity is a lead, not confirmed silicon identification for the exact board. In addition, the candidate YC3121-L datasheet contains an internal inconsistency in the address assigned to `TIM_CTRL`; the firmware's per-channel bitfield operations are clear, but the physical register name/address mapping should not be treated as fully proven.

## 1. Candidate MCU and register-map qualification

The prior project report `PHASE13_FIRMWARE_RGB_ANALYSIS.md` records the YC3121-L as a TH80-family review lead and explicitly says that the exact board MCU is unconfirmed. The raw firmware contains Yichip SDK material, but no ELF metadata or trusted linker map.

The YC3121-L datasheet is therefore supporting evidence, not proof of exact part identity. It lists:

- Timer base `0x000f0c00`.
- UART0 control at `0x000f8b1c`, UART1 control at `0x000f8c1c`.
- GPIO base `0x000f8700`.
- USB base `0x000f6000`.
- DMA base `0x000f8800`.
- NVIC base `0xe000e100`.

See the [YC3121-L datasheet](https://atta.szlcsc.com/upload/public/pdf/source/20211111/30894FF75D776498D9212EBEB2236820.pdf), pages 11, 31–32, 52–55 and 61–63.

## 2. Direct Timer1 initialization evidence

Function `0x10240` (candidate runtime address under image base `0x01000200`; file offset `0x10240`) is called from the application initialization block and three transition/reinitialization paths.

Its channel/reload setup is:

```asm
; struct channel = 1
; struct reload  = 0x0000226e
; 0x1a9fa(config_struct)
; 0x1aa50(1, 1)
; 0x1a9b2(1, 1)
; 0x1d304(14)
```

The helper at `0x1a9fa`:

1. Calls `0x1a9b2(channel, 0)` to disable the selected timer channel.
2. Writes the supplied period/reload word to `0x000f0c00 + channel * 8`.
3. Sets the channel's packed control bits corresponding to timer mode and auto-reload.

For channel 1, the PCNT write destination is `0x000f0c08`, which is `TIM1_PCNT` in the candidate datasheet. The stored reload value is `0x226e` (decimal 8814).

The helper at `0x1aa50(1,1)` sets bit 2 in the channel-1 nibble. The helper at `0x1a9b2(1,1)` sets bit 0 in that nibble. The function then calls `0x1d304` with `r0=14`, enabling NVIC bit 14 at `0xe000e100`.

**Classification:** `CONFIRMED_STATIC` for the literal channel index, reload value, PCNT destination, helper arguments and NVIC enable argument. `STRONG_STATIC_INFERENCE` that this is Timer1 in timer/auto-reload mode and that its interrupt source is mapped to NVIC external index 14.

## 3. `0x1a9b2` semantics and register caveat

The helper at `0x1a9b2` takes a timer-channel index and an enable/disable flag. It calculates a four-bit field position (`index * 4`) and sets or clears bit 0 of that channel field. For channels 0–7 it performs the read-modify-write at `0x000f0c48`; for channel 8 it uses `0x000f0c4c`.

For channel 1, the operation is therefore a set/clear of mask `0x10` at `0x000f0c48`. The same helper is called by `0x103b0` with `(1,0)` before `0x135de`, and `(1,1)` afterward. This is a timer-channel control operation, not a write to NVIC ISER/ICER and not a global CPU interrupt mask.

The candidate datasheet describes the timer control nibble as:

- bit 0: timer module enable;
- bit 2: PWM versus TIMER mode (`1` = TIMER mode);
- bit 3: automatic reload.

However, the datasheet's register table is internally inconsistent: it lists `TIM_CTRL` at `0x000f0c30`, which also appears in its PCNT table as `TIM6_PCNT`; it lists `TIM_NCNT` beginning at `0x000f0c48`; and it lists `TIM_CTRL1` for Timer8 at `0x000f0c4c`. The firmware consistently treats `0x000f0c48` as the packed control word for channels 0–7 and `0x000f0c4c` as the channel-8 control word. The bitfield semantics match the firmware helper's use, but the datasheet address conflict prevents unqualified naming of `0x000f0c48` as `TIM_CTRL`.

**Classification:** `CONFIRMED_STATIC` for the bitfield read-modify-write behavior and addresses. `STRONG_STATIC_INFERENCE` for its interpretation as per-channel timer enable control. Physical register semantics remain qualified by the datasheet inconsistency and MCU identity uncertainty.

## 4. Timer mode, event and reload semantics

The candidate datasheet says the timer unit has nine independent timers, each with an independent interrupt source; timers count down; HCLK clocks the timer; TIMER mode enables interrupt generation by default; and `TIM_PCNT` is the repeat reload value. It also says that enabling a timer loads the current counter from the programmed period register.

Combining that documentation with the firmware's channel-1 setup gives this likely event sequence:

```text
Timer1 enabled
  -> down-counter runs from PCNT1 = 0x226e
  -> counter reaches zero
  -> Timer1 interrupt source requests NVIC index 14
  -> vector stub / secondary dispatcher selects table[14]
  -> 0x103b0
```

The auto-reload bit is set in the firmware's channel-1 configuration. The callback then disables and re-enables channel 1, which—if the candidate datasheet behavior applies—reloads the counter from PCNT1. This is consistent with a periodic timer interrupt whose handler restarts the interval.

No frequency is calculated. HCLK's runtime frequency and any effective prescaling are not recovered. The raw reload value `0x226e` alone is insufficient to establish a period or RGB refresh rate.

**Classification:** `STRONG_STATIC_INFERENCE` for Timer1 underflow as the event source and auto-reload behavior. `UNKNOWN` for actual event timing and pending/acknowledgement semantics.

## 5. Timer2 comparison

Function `0x10214` configures channel 2 and enables NVIC index 15. During the startup call at `0x102fe`, `r0=0x60` (96); the helper multiplies that by `0x000f4240` (1,000,000), giving a Timer2 PCNT value of 96,000,000, and writes it to `0x000f0c10` (`TIM2_PCNT` under the candidate datasheet map). It configures timer mode/auto-reload, enables channel 2 and then calls `0x1d304(15)`.

Application table slot 15 contains a pointer to `0x103c8`, a routine that sets a state byte at `0x0002034f`. This is a second timer/channel-to-NVIC pairing and supports the Timer1/IRQ14 interpretation. Timer2 is not the source of IRQ14.

**Classification:** `CONFIRMED_STATIC` for channel-2 setup and NVIC index-15 enable. `STRONG_STATIC_INFERENCE` that Timer2 is the peripheral source for index 15.

## 6. Corrected interpretation of IRQ5 and IRQ6

The Phase 13.4/13.5 interpretation of `0x173cc` as timer setup was incorrect. The actual base addresses used by its helpers are UART0/UART1 bases, not the timer block.

- `0x173cc` calls `0x1d304(5)`, configures UART selector 0 using base `0x000f8b00`, and invokes UART helpers at register offsets including `+0x1c`.
- The scalar `500000` is used with the UART's 48 MHz clock to calculate divisor `48,000,000 / 500,000 = 96`; it is a baud-rate configuration value, not a timer period.
- The candidate datasheet identifies UART0 control at `0x000f8b1c` and UART1 control at `0x000f8c1c`, with a fixed 48 MHz UART clock.
- Application table slot 5 resolves to `0x10418`, which polls UART0 registers.
- Application table slot 6 resolves to `0x103d0`, which polls UART1 registers.

Therefore IRQ5 is associated with UART0 in this firmware, and slot 6 is the UART1 service path. Neither establishes a Timer0-to-RGB relationship. This corrects the earlier Phase 13.4/13.5 reports without rewriting those historical files.

**Classification:** `CONFIRMED_STATIC` for the UART base literals, UART helper calls, scalar/divisor calculation and table targets. `STRONG_STATIC_INFERENCE` for UART0-to-index-5 mapping.

## 7. Other candidate sources

| Candidate | Relevant evidence | Assessment for IRQ14 |
|---|---|---|
| Timer1 | Channel 1 PCNT write at `0xf0c08`; timer-control operations; NVIC14 enable; table14 -> `0x103b0`; handler toggles channel 1 | **Strongest source; `STRONG_STATIC_INFERENCE`** |
| Timer2 | Channel 2 PCNT write at `0xf0c10`; NVIC15; table15 -> `0x103c8` | Separate IRQ15 source, not IRQ14 |
| UART0 | Base `0xf8b00`; NVIC5; table5 -> UART0 poller | IRQ5, not IRQ14 |
| UART1 | Base `0xf8c00`; table6 -> UART1 poller | Slot 6; no evidence it drives IRQ14 |
| GPIO | GPIO/output register region around `0xf8700`; RGB output code writes peripheral registers | No IRQ14-specific GPIO interrupt-source setup recovered |
| USB | Candidate base `0xf6000`; USB interrupt masks exist in datasheet | No IRQ14-specific USB source enable/handler correlation recovered |
| DMA | Candidate base `0xf8800`; DMA channels have completion interrupts | No DMA channel setup tied to NVIC14 or table14 recovered |
| RTC/LPM | Candidate datasheet documents separate low-power/RTC sources | No corresponding IRQ14 setup or handler correlation recovered |

The table does not prove that other peripheral sources can never be multiplexed onto the same line. No such multiplexing is established by the supplied firmware or candidate datasheet.

## 8. Final source assessment

**Most likely IRQ14 hardware event:** Timer1 count-down reaching zero in TIMER mode, with automatic reload configured. The firmware associates Timer1 channel 1 with NVIC index 14 and registers `0x103b0` in application table slot 14. The wrapper disables Timer1 while executing `0x135de` and re-enables it afterward.

This is a `STRONG_STATIC_INFERENCE`, not a runtime observation. The remaining qualifications are exact MCU identity, the datasheet's timer-control address inconsistency, hardware vector-base selection, and the lack of a documented Timer1-to-NVIC-number table.

## Related reports

- [Vector table and IRQ entry](PHASE13_6_VECTOR_TABLE_AND_IRQ_ENTRY.md)
- [IRQ14 lifecycle and RGB trigger](PHASE13_6_IRQ14_LIFECYCLE_AND_RGB_TRIGGER.md)
- [Findings and next steps](PHASE13_6_FINDINGS_AND_NEXT_STEPS.md)
- [Phase 13.5 timer lifecycle (historical; corrected here)](PHASE13_5_TIMER_LIFECYCLE_AND_RGB_ACTIVATION.md)
