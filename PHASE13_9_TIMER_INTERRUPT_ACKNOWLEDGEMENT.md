# Phase 13.9 — Timer Register and Interrupt Acknowledgement

## 1. Executive conclusion

The V504 firmware contains no positively identified Timer1/Timer2 interrupt-status read, peripheral flag clear, or NVIC pending-clear write. The closest public Yichip-authored YC3121 POS BSP confirms the timer register layout, channel-control fields and IRQ numbering, but contains no timer driver implementation. The YC3121-D and YC3121-L datasheets describe the timer count/reload model but leave their interrupt-handling subsection without acknowledgement semantics.

**Timer1 and Timer2 peripheral acknowledgement behavior remains UNKNOWN.** The code structure makes an automatically deasserting/pulsed timer request or another implicit hardware acknowledgement plausible, especially because Timer2 is left running and its handler simply returns. That is an inference about the intended design, not proof of the silicon behavior.

## 2. Evidence sources and confidence

| Evidence | Source | Classification |
|---|---|---|
| Timer register formulas and control field positions | RT-Thread BSP core/yc3121.h | CONFIRMED_STATIC for source contents |
| Nine independent timer channels and HCLK clocking | YC3121-D V1.2 / YC3121-L V1.1 datasheets | CONFIRMED_STATIC for documented family model |
| Timer mode defaults interrupt enable | YC3121-D V1.2 / YC3121-L V1.1 timer section | CONFIRMED_STATIC for documented model |
| Timer1/Timer2 init and wrapper accesses | V504 disassembly | CONFIRMED_STATIC |
| No explicit timer status/clear in analyzed V504 image | Disassembly and literal/address scan | CONFIRMED_STATIC for searched image |
| Exact interrupt acknowledgement mechanism | No timer source or complete peripheral description | UNKNOWN |

Public BSP commit and hashes are recorded in PHASE13_9_V504_SDK_RECOVERY.md.

## 3. Reconciled timer register map

The public header uses TIMER_BASEADDR=0x000f0c00 and PWM_TOTAL=9. The corresponding register model is:

| Register | Address | Documented purpose | Timer1/2 firmware use |
|---|---:|---|---|
| Timer1 PCNT | 0x000f0c08 | Reload value in timer mode | Written by generic channel configuration |
| Timer2 PCNT | 0x000f0c10 | Reload value in timer mode | Written by generic channel configuration |
| Timer1 NCNT | 0x000f0c0c | PWM low-level duration | No Timer1 timer-mode use identified |
| Timer2 NCNT | 0x000f0c14 | PWM low-level duration | No Timer2 timer-mode use identified |
| TIM_CTRL | 0x000f0c48 | Packed control nibbles for channels 0–7 | Read-modify-written for channels 1 and 2 |
| TIM_CTRL1 | 0x000f0c4c | Four control bits for channel 8 | Generic helper has channel-8 path; not used by Timer1/2 |
| Timer1 CNT | 0x000f0c54 | Current counter value, documented read-only | No direct firmware access found |
| Timer2 CNT | 0x000f0c58 | Current counter value, documented read-only | No direct firmware access found |

The header formula for TIM_CNT(x) evaluates to 0x000f0c50 + 4*x. The datasheet's detailed channel table independently lists Timer1 CNT at 0xf0c54 and Timer2 CNT at 0xf0c58.

The control words are not two copies of one shared interrupt register:

- TIM_CTRL at 0xf0c48 holds eight four-bit channel fields, channels 0 through 7.
- TIM_CTRL1 at 0xf0c4c holds the four-bit field for channel 8 only.
- Each channel field uses bit 0 enable, bit 1 initial output level, bit 2 TIMER/PWM mode, and bit 3 auto-reload.
- No timer interrupt-status or interrupt-clear field is documented in either control word.
- Timer mode is documented as enabling the interrupt by default; there is no separate per-channel interrupt-enable bit in the documented four-bit field.

TIM_CTRL1 is therefore not a shared interrupt control/status register. It is the channel-8 extension of the packed timer control model.

## 4. Firmware access inventory

All addresses below are candidate runtime addresses using the established image-base convention runtime = 0x01000000 + file offset. The raw file offsets are the primary reproducible evidence.

| File offset / runtime candidate | Operation | Register | Interpretation | Classification |
|---|---|---|---|---|
| 0x1a9fa / 0x0101a9fa | 32-bit write through computed channel address | PCNT at 0xf0c00 + channel*8 | Timer1 channel 1 writes 0xf0c08; Timer2 channel 2 writes 0xf0c10 | CONFIRMED_STATIC |
| 0x1a9b2 / 0x0101a9b2 | 32-bit read-modify-write | TIM_CTRL at 0xf0c48 for channels 0–7 | Sets/clears bit 0 of the selected four-bit channel field | CONFIRMED_STATIC |
| 0x1a9b2 / 0x0101a9b2 | 32-bit read-modify-write | TIM_CTRL1 at 0xf0c4c for channel 8 | Generic channel-8 enable branch | CONFIRMED_STATIC |
| 0x1aa50 / 0x0101aa50 | 32-bit read-modify-write | TIM_CTRL at 0xf0c48 | Sets/clears bit 2 of the selected channel nibble | CONFIRMED_STATIC |
| 0x1aa44 / 0x0101aa44 | 32-bit read-modify-write | SYSCTRL_CLK_CLS at 0xf856c | Sets clock-gate bit 4; no active caller in the recovered Timer1 lifecycle | CONFIRMED_STATIC |

The literal at file offset 0x1ac3c is 0x000f0c40. The helper subtracts 0x40 and adds channel-dependent offsets to reach the PCNT/control addresses. This corrects the byte-order error recorded in the older Phase 13.7 alternative map; the false 0xf3fdx/0xf401x timer candidates must not be reused.

No direct access to 0xf0c54 or 0xf0c58 was found in the firmware image. The generic timer helper paths do not read or write the current-count registers. No distinct status address or clear register is constructed by these helpers.

## 5. Control-field semantics

The Yichip header defines the channel nibble bit positions:

| Bit | Field | Documented meaning |
|---:|---|---|
| 0 | TIM_CTRL_ENABLE | Channel enable |
| 1 | TIM_CTRL_START_LEVEL | Initial output level |
| 2 | TIM_CTRL_MODE | 0 = PWM, 1 = TIMER |
| 3 | TIM_CTRL_AUTO_RELOAD | In TIMER mode, reload PCNT when CNT reaches zero |

Firmware helper behavior:

- 0x1a9b2 computes 1 << (channel*4), then sets or clears that enable bit.
- 0x1a9fa first disables the selected channel, writes PCNT, then ORs 0xC << (channel*4), setting mode and auto-reload bits.
- 0x1aa50 independently sets or clears 0x4 << (channel*4), the TIMER/PWM mode bit.

Timer1 and Timer2 initialization pass mode=1 and set auto-reload. Neither helper sets a separate interrupt-status or clear bit.

The control nibble interpretation is corroborated by the public header and datasheet, not by an exact recovered yc_timer.c implementation.

## 6. Peripheral interrupt acknowledgement

### 6.1 Datasheet evidence

The YC3121-D V1.2 timer chapter states that:

- The timer block contains nine independent timer channels.
- Each channel has its own interrupt source.
- The timer counts down from its configured value.
- HCLK supplies the timer clock.
- The current counter is loaded when the timer is enabled.
- TIMER mode defaults interrupt enable.

However, section 12.2.2, Interrupt Handling, contains no substantive flag, status, read-to-clear, write-one-to-clear, write-zero-to-clear or acknowledgement description. The corresponding YC3121-L V1.1 subsection is similarly uninformative.

The detailed timer register list exposes PCNT, NCNT, TIM_CTRL, TIM_CTRL1 and CNT. It does not identify a separate timer interrupt status/clear register.

### 6.2 Firmware ISR evidence

Timer1 path:

1. Init at file offset 0x10240 configures channel 1, PCNT=0x226e (8814), TIMER mode and auto-reload.
2. The same init enables NVIC index 14 via 0x1d304.
3. Application table slot 14 dispatches to 0x103b0.
4. 0x103b0 calls 0x1a9b2(1,0), calls 0x135de, then calls 0x1a9b2(1,1).
5. No timer status read, timer clear write, NVIC disable, NVIC pending clear or global interrupt mask occurs in this wrapper.

Timer2 path:

1. Init at file offset 0x10214 configures channel 2 and PCNT=96,000,000 in the startup configuration.
2. The same init enables NVIC index 15 via 0x1d304.
3. Application table slot 15 dispatches to 0x103c8.
4. 0x103c8 stores byte value 1 at object offset 0x0f and returns.
5. It does not stop Timer2, read a status register, clear a peripheral flag or clear NVIC pending state.

This is particularly important: Timer2 is configured for auto-reload and its handler does not stop the channel. If the source remained continuously asserted as an uncleared level request, repeated immediate re-entry would be expected under a conventional level-sensitive interrupt controller. The intended periodic behavior therefore makes an automatically deasserting/pulsed request or implicit hardware acknowledgement plausible. This is only a STRONG_STATIC_INFERENCE about intended design; the actual request polarity and clear semantics remain UNKNOWN.

### 6.3 NVIC pending state

The public Yichip BSP source:

- Defines NVIC ISER at 0xe000e100.
- Defines NVIC ICER at 0xe000e180.
- Defines NVIC ISPR at 0xe000e200.
- Defines NVIC ICPR at 0xe000e280.
- Includes an NVIC_Type structure with ICPR at offset 0x180.
- Implements NVIC_EnableIRQ and NVIC_DisableIRQ through enable_intr/disable_intr.
- Does not implement NVIC_ClearPendingIRQ in the inspected misc.c/misc.h.

The V504 function at 0x1d304 writes the selected bit to ISER. The function at 0x1d312 writes the selected bit to ICER. No write to ICPR address 0xe000e280 was found in the searched image. No ICPR literal was found in the firmware image.

Disabling the peripheral channel is not equivalent to clearing NVIC pending state. Disabling the NVIC line is also not equivalent to clearing a peripheral flag. No such clear is visible in 0x103b0 or 0x103c8.

The public BSP's explicit Cortex-M0/NVIC definitions strongly support a Cortex-M0-style NVIC for the YC3121 POS BSP. The exact keyboard silicon is still unconfirmed. Generic Cortex-M pending-state behavior cannot identify the timer peripheral's request polarity or prove that the timer output is deasserted on exception entry.

## 7. Rejected acknowledgement hypotheses

| Hypothesis | Why it is not established |
|---|---|
| Entering the handler clears the timer peripheral flag | No timer flag/status register or peripheral behavior description is available |
| Disabling Timer1 clears a pending timer request | 0x1a9b2 only clears the channel enable bit; no documented pending side effect |
| Re-enabling Timer1 clears NVIC pending | The helper only sets the channel enable bit; it does not write ICPR |
| Reading TIM_CNT clears the interrupt | Firmware does not read Timer1/2 CNT, and datasheet calls it current count |
| TIM_CTRL1 contains a shared pending flag | Public header/datasheet define it as Timer8's four-bit control field |
| TIM_CTRL bit 0 is an interrupt-status acknowledgement | It is the channel enable bit |
| W1C/W0C semantics apply to PCNT/NCNT | No source or documentation supports this |
| Timer2 proves all timers use edge-triggered IRQs | It supports an intended auto-deasserting periodic source but does not prove the silicon implementation |

## 8. Comparative evidence from other SDK drivers

The public BSP's watchdog driver implements WDT_GetITStatus and WDT_ClearITPendingBit; the clear function writes 1 to WD_CLEAR. The DMA driver implements DMA_ClearITPendingBit by setting a documented bit in DMA_START.

These examples show that the SDK uses explicit per-peripheral pending-clear APIs where documented. The timer driver source is absent, so the lack of a timer clear function in this BSP cannot be treated as proof that no timer clear mechanism exists.

## 9. Exact remaining unknowns

The following are still UNKNOWN:

1. Whether Timer1/Timer2 requests are pulse- or level-sensitive.
2. Whether a peripheral status latch exists outside the documented timer register map.
3. Whether exception acceptance, counter reload or another hardware event deasserts the request.
4. Whether stopping the channel clears or merely masks a request.
5. Whether a pending request survives stop/restart.
6. Whether the exact keyboard MCU implements the same timer/NVIC behavior as the YC3121 POS BSP.

The single most useful missing source is the exact V504 yc_timer.c/yc_timer.h plus the matching SoC interrupt-controller header. Without it, this phase cannot claim a resolved acknowledgement sequence.

## 10. Cross-references

- [V504 SDK recovery and source provenance](PHASE13_9_V504_SDK_RECOVERY.md)
- [RGB callback timing model](PHASE13_9_RGB_CALLBACK_TIMING_MODEL.md)
- [Findings and implementation boundary](PHASE13_9_FINDINGS_AND_IMPLEMENTATION_BOUNDARY.md)
- [Phase 13.8 corrected timer map](PHASE13_8_TIMER_REGISTER_MAP_RECONCILIATION.md)
- [Phase 13.7 historical IRQ14 analysis](PHASE13_7_IRQ14_EVENT_AND_ACKNOWLEDGEMENT.md)
