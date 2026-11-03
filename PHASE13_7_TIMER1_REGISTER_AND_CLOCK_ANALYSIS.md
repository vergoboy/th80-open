# Phase 13.7 — Timer1 Register and Clock Analysis

## Scope and confidence

This report traces the Timer1/Timer2 initialization functions, shared channel helpers, literal-pool values, clock-related routines, and NVIC helpers in the supplied V504 image. All conclusions are static. Candidate runtime addresses use the Phase 13.6 application mapping (`0x01010000 + file offset`). No peripheral was accessed on hardware.

A significant discrepancy emerged: the Phase 13.6 reports described Timer1 reload/control candidates at `0x000f0c08` and `0x000f0c48`, whereas the currently inspected helper literal pool at `0x1ac3c` decodes to `0x000f400c`. The helper's arithmetic therefore addresses a reload-like channel block at `0x000f3fcc` onward and control words at `0x000f4014`/`0x000f4018`. The earlier `0xf0cxx` mapping is not silently retained as fact. Until an exact SDK header or matching datasheet revision reconciles these addresses, the register map below is the literal address arithmetic observed in the disassembly, with the prior mapping recorded as a conflict.

## Initialization call sites

| Function | File offset | Candidate runtime address | Observed behavior | Classification |
|---|---:|---:|---|---|
| Timer2 init | `0x10214` | `0x01010214` | Builds channel-2 config structure; calls `0x1a9fa`, `0x1aa50`, `0x1a9b2`; enables NVIC index 15 via `0x1d304` | `CONFIRMED_STATIC` |
| Timer1 init | `0x10240` | `0x01010240` | Builds channel-1 config structure with reload literal `0x226e`; calls the same helpers; enables NVIC index 14 | `CONFIRMED_STATIC` |
| Timer1 IRQ wrapper | `0x103b0` | `0x010103b0` | Disables channel 1 through `0x1a9b2(1,0)`, calls `0x135de`, then enables channel 1 through `0x1a9b2(1,1)` | `CONFIRMED_STATIC` |
| Timer2 IRQ handler | `0x103c8` | `0x010103c8` | Sets state byte at offset `0x0f` in a global object; no Timer1-like output call | `CONFIRMED_STATIC` |

Timer2 initialization at `0x10214` computes a reload from a literal multiplier and a caller-supplied value. The Phase 13.6 analysis identified the resulting configured value as `0xf4240 * 96 = 96,000,000`; this timer has its own channel index and IRQ15 mapping. Timer1 initialization uses the fixed `0x226e` (8814) reload value and IRQ14. These are distinct channels and interrupt indices; the old IRQ5-as-timer interpretation is incorrect. IRQ5 is UART0 (`0x10418`), IRQ6 is UART1 (`0x103d0`); the `500000`/`96` calculation in the UART path is baud configuration, not Timer0 frequency.

## Helper-level register arithmetic

### Channel enable helper `0x1a9b2`

At file offset `0x1a9b2` (candidate `0x0101a9b2`):

1. Channel index is in `r0`; requested enable state is in `r1`.
2. It forms `1 << (channel * 2)`.
3. It loads the literal at `0x1ac3c`, whose two halfwords are `0x400c` and `0x000f`, i.e. the 32-bit address `0x000f400c`.
4. For channels 0–7, it reads/writes `[0x000f400c + 0x08]` (`0x000f4014`); channels 8 and above use the word at `0x000f4018`.
5. It sets or clears the selected bit in that word.

For channel 1, the bit is `1 << 2`, i.e. bit 2 of `0x000f4014`. `0x103b0` calls this helper with `(1,0)` and `(1,1)`, directly establishing the software's disable/enable operation on that bit.

**Classification:** `CONFIRMED_STATIC` for the arithmetic, literal value, and read-modify-write; `STRONG_STATIC_INFERENCE` that the selected bit is the channel enable control, based on call arguments and lifecycle use.

### Channel configuration helper `0x1a9fa`

At file offset `0x1a9fa` (candidate `0x0101a9fa`):

- Calls `0x1a9b2(channel,0)` first.
- Writes the caller-provided reload/config value to `base - 0x40 + channel*8`, where `base` is the same `0x000f400c` literal.
- For channel 1, the computed address is `0x000f3fd4`.
- It then ORs `0x0c << (channel*2)` into the control word at `base + 8` (`0x000f4014`) for channels 0–7, or `base + 0x0c` for the upper bank.

For channel 1, this sets bits 4 and 5 of `0x000f4014`. The init function passes the stack structure with channel index 1 and reload `0x226e`; therefore the binary's direct write destination for that value is `0x000f3fd4`, not `0x000f0c08`.

**Classification:** `CONFIRMED_STATIC` for address arithmetic and bit masks; `STRONG_STATIC_INFERENCE` that `0x000f3fd4` is the Timer1 reload/period register, because it receives the channel's configured count. The hardware register name and count semantics remain conditional.

### Mode selector `0x1aa50`

At file offset `0x1aa50` (candidate `0x0101aa50`):

- Uses channel index `r0` and a mode selector from the stack argument.
- Forms `0x4 << (channel*2)` and sets/clears that bit in `0x000f4014` (or `0x000f4018` for the upper bank).
- Timer1 initialization calls it with channel `1` and selector `1`, so bit 4 of `0x000f4014` is set.

This overlaps the bit-4 portion of the pair set by `0x1a9fa`; it is not evidence for an independent four-bit-per-channel field. The safest description is a packed control word whose channel-related bits are addressed by two-bit strides, with enable at `2*channel` and configuration mask bits at `2*channel+2` and `2*channel+3`. The precise hardware meaning of each configuration bit cannot be established solely from this helper. Bit 4 is selected by a helper called as a mode selector; bit 5 is also set by configuration initialization. Auto-reload is a plausible interpretation from the timer API/datasheet context, but remains conditional.

**Classification:** `CONFIRMED_STATIC` for bit operations and call arguments; `WEAK_INFERENCE` for the named timer mode/auto-reload meanings until matched to an exact SDK header.

### `0x1aa44` is not established as a timer clock gate

The function at `0x1aa44` reads a separate literal `0x000f4085`, reads a word at offset `+0x2c`, sets bit `0x10`, and writes it back. No direct call to this function was identified in the inspected helper xrefs. The odd literal and absence of a confirmed caller mean it must not be described as the Timer1 clock gate. `UNKNOWN` for its peripheral identity and effect.

## Register map currently justified by the binary

| Candidate address | Access evidence | Conservative interpretation | Classification |
|---|---|---|---|
| `0x000f3fcc + channel*8` | `0x1a9fa` writes caller-supplied 32-bit count at `base - 0x40 + channel*8` | Per-channel count/reload-like register bank | `STRONG_STATIC_INFERENCE` |
| `0x000f3fd4` | Timer1 channel 1 receives `0x226e` | Timer1 count/reload-like register | `STRONG_STATIC_INFERENCE` |
| `0x000f3fdc` | Timer2 channel 2 config structure uses same helper arithmetic | Timer2 count/reload-like register | `STRONG_STATIC_INFERENCE` |
| `0x000f4014` | Read-modify-write in helpers `0x1a9b2`, `0x1a9fa`, `0x1aa50` | Packed control/config word for channels 0–7 | `STRONG_STATIC_INFERENCE` |
| `0x000f4018` | Same helpers route channel indices >=8 here | Upper-channel packed control/config word | `STRONG_STATIC_INFERENCE` |
| Timer status/interrupt flag | No positively identified read/clear sequence in Timer1 wrapper or init | Unknown | `UNKNOWN` |
| Prescaler / timer clock select | Not positively identified in the Timer1 init path | Unknown | `UNKNOWN` |

## Datasheet conflict

The candidate YC3121-L datasheet previously cited in Phase 13.6 describes timer channel registers in the `0xF0Cxx` area and gives inconsistent labels/addresses for `TIM_CTRL`, `TIM_NCNT`, and `TIM_CTRL1` (historical references: pages 52–55; NVIC/system map page 11). Phase 13.6 mapped `0x000f0c08` and `0x000f0c48` to Timer1 reload/control based on that candidate map. The current direct helper literal and pointer arithmetic instead resolve to `0x000f400c`, `0x000f3fd4`, and `0x000f4014`.

This is a substantive unresolved discrepancy, not a harmless naming difference. Possibilities include an earlier transcription/address-decoding error, a different register bank or alias, a different silicon variant, or a disassembly/image mapping assumption that needs independent corroboration. No original `yc_timer.h`/`yc_timer.c` source was present in the currently extracted workspace to resolve it. The Phase 13.6 `0xf0cxx` labels should therefore be treated as candidate datasheet-correlated labels, not confirmed register identities.

## HCLK and clock initialization

The image contains SDK strings `SYSCTRL_HCLKConfig`, `SYSCTRL_AHBPeriphClockCmd`, and `SYSCTRL_EnterSleep`, plus `yc_sysctrl.c` and `yc_systick.c` source-path remnants. This proves that the binary was linked with code or strings associated with system-clock APIs; it does not prove the active HCLK setting at Timer1 enable.

Functions around `0x1b0e8` and `0x1b168` were inspected. They access a separate system/peripheral block, derive a divisor-like value from a field in a register at candidate literal `0x000f8540` plus offset `0x20`, program count/compare/control words in another block, and poll a status bit before returning. The observed loops and register writes are consistent with a hardware timing/delay service, but do not establish the system oscillator, PLL multiplier, HCLK source, HCLK divider, or Timer1 clock tree. The neighboring functions `0x1b1bc` onward manipulate the same peripheral control word, but no defensible oscillator/PLL/HCLK frequency reconstruction has been completed.

The prior handoff's proposed SDK configuration clue was not reproducible from the current tree, and no original SYSCTRL headers were found. Therefore:

- Oscillator source: `UNKNOWN`.
- PLL configuration/frequency: `UNKNOWN`.
- Core clock: `UNKNOWN`.
- HCLK at Timer1 enable: `UNKNOWN`.
- Timer peripheral divider/prescaler: `UNKNOWN`.
- Sleep/wake clock transition: `UNKNOWN` beyond the presence of `SYSCTRL_EnterSleep` SDK naming.

No timer interval is calculated. `0x226e = 8814` is a configured count, not a refresh rate. Even if HCLK were known, reload inclusion, prescaling, count direction, and update timing would still need to be confirmed before applying a period formula.

## Interrupt source enable, NVIC, and acknowledgement

Timer1 init calls `0x1d304(14)`. At `0x1d304`, the code writes `1 << IRQ` to the word at the literal `0xe000e100`, consistent with NVIC ISER0. The paired helper `0x1d312` writes the selected bit to `0xe000e180`, consistent with NVIC ICER0. No corresponding write to `0xe000e280` (ICPR0) was found in the examined disassembly search.

These NVIC operations are distinct from the timer channel enable helper `0x1a9b2`. The Timer1 IRQ wrapper does not call `0x1d312`, does not write an observed NVIC pending-clear register, and does not contain an explicit peripheral status read/clear. It only toggles the channel enable bit around the output routine.

No Timer1 status/interrupt-flag register, W1C/W0C sequence, or explicit source acknowledgement has been positively identified. The possibility of automatic peripheral acknowledgement on exception entry or a vendor ROM/SDK helper outside the examined image cannot be excluded. Disabling the channel is not proof that pending state is cleared.

## Timer2 comparison

Timer2 initialization at `0x10214` independently exercises the same generic helpers with channel index `2`, computes a large count using the timer multiplier path, selects mode, enables the channel, and enables NVIC index `15`. Its handler at `0x103c8` stores `1` into a state byte at global-object offset `0x0f`. It does not execute the Timer1 RGB output path. This supports a generic multi-channel timer driver interpretation but does not prove all channels share identical interrupt acknowledgement semantics.

## Conclusions

The binary establishes a generic channel configuration path, a Timer1 channel-1 count write of `0x226e`, packed control-bit manipulation, and NVIC index 14 enable. It does not establish HCLK, actual timer frequency, a peripheral interrupt flag/clear mechanism, or physical interrupt delivery. Most importantly, the firmware literal arithmetic conflicts with the earlier `0xf0cxx` datasheet-based register labels. The exact SDK header matching this image is the highest-value artifact needed to close the map.

## Cross-references

- [Silicon and boot architecture](PHASE13_7_SILICON_AND_BOOT_ARCHITECTURE.md)
- [IRQ14 event and acknowledgement](PHASE13_7_IRQ14_EVENT_AND_ACKNOWLEDGEMENT.md)
- [Findings and hardware boundary](PHASE13_7_FINDINGS_AND_HARDWARE_BOUNDARY.md)
- Historical timer source attribution: [Phase 13.6 IRQ14 peripheral source](PHASE13_6_IRQ14_PERIPHERAL_SOURCE.md)
