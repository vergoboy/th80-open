# Phase 13.7 — IRQ14 Event and Acknowledgement

## Scope

This report closes the firmware-side IRQ14 path as far as the static image permits. It does not claim a physical interrupt was observed. All analysis is static and does not access the keyboard, HID, USB, or firmware runtime.

## Event-to-handler path

The strongest firmware-side path is:

`Timer1 channel-1 condition (candidate) → peripheral interrupt request (not directly observed) → hardware/ROM interrupt entry (unknown) → application IRQ14 executable stub → secondary dispatcher → application handler table[14] → 0x103b0 → 0x135de`

### 1. Timer1 configuration

At file offset `0x10240` (candidate runtime `0x01010240`), application initialization builds a timer configuration object with channel index `1` and reload/count value `0x226e` (8814), calls generic channel configuration and mode helpers, enables the channel through `0x1a9b2(1,1)`, and enables NVIC index 14 through `0x1d304(14)`.

The helper-level register arithmetic is documented in [PHASE13_7_TIMER1_REGISTER_AND_CLOCK_ANALYSIS.md](PHASE13_7_TIMER1_REGISTER_AND_CLOCK_ANALYSIS.md). A key qualification is that the current literal-pool decode resolves the timer helper base to `0x000f400c`, not the `0xf0cxx` candidates used in Phase 13.6. The exact physical register names remain unresolved.

**Classification:** `CONFIRMED_STATIC` for channel argument, count literal, helper calls, and NVIC index; `STRONG_STATIC_INFERENCE` that the configured source is Timer1.

### 2. NVIC enable operation

At `0x1d304`, the firmware computes `1 << IRQ` and ORs it into the word at literal `0xe000e100`. For IRQ14 this selects bit 14 in the first 32-bit enable bank, consistent with NVIC ISER0. The paired helper at `0x1d312` writes the selected bit to `0xe000e180`, consistent with ICER0.

The literal values and writes are `CONFIRMED_STATIC`; identifying these as ARM NVIC ISER0/ICER0 is `STRONG_STATIC_INFERENCE`, supported by their canonical addresses and the Thumb system-instruction evidence. No firmware write to the canonical ICPR0 address `0xe000e280` was found in the searched disassembly. The NVIC pending state is therefore not shown to be explicitly cleared by the Timer1 wrapper.

### 3. Vector stub bank and secondary dispatcher

Phase 13.6 established the following static application dispatch structure:

- Application executable stub bank begins at file offset `0x10000`, candidate runtime `0x01010200`.
- Application dispatcher is candidate `0x0101029c`.
- Application handler table is candidate `0x010102bc` (file offset `0x100bc`).
- Application IRQ14 stub passes argument `0x38` (14 × 4) to the secondary dispatcher.
- Table slot 14 contains raw pointer `0x010105b1`; clearing the Thumb low bit resolves to handler `0x103b0`.

The boot-mode table slot 14 is a default trap, while the application table slot 14 resolves to the Timer1 wrapper. Boot dispatcher mode state `2` selects the boot-mode table; other values forward to the application dispatcher. The boot-stage function at file offset `0xf8` writes `1` to the state word and branches to the application stub bank.

This is a firmware-side secondary dispatch path, not proof of the hardware's reset-time vector fetch or peripheral-to-NVIC wiring. See [PHASE13_6_VECTOR_TABLE_AND_IRQ_ENTRY.md](PHASE13_6_VECTOR_TABLE_AND_IRQ_ENTRY.md) for the byte-level table evidence.

### 4. Handler wrapper `0x103b0`

The wrapper is:

```asm
103b0: push {r4, lr}
103b2: movs r1, #0
103b4: movs r0, #1
103b6: bl   0x1a9b2
103ba: bl   0x135de
103be: movs r1, #1
103c0: mov  r0, r1
103c2: bl   0x1a9b2
103c6: pop  {r4, pc}
```

The helper `0x1a9b2` uses channel index `1` and clears/sets the selected channel bit in the packed timer control word. In the currently decoded literal arithmetic that word is at `0x000f4014`, bit 2. The wrapper therefore disables the timer channel before invoking the RGB output routine and re-enables it after the call.

**Classification:** `CONFIRMED_STATIC` for the exact call sequence and arguments; `STRONG_STATIC_INFERENCE` for channel stop/start semantics based on generic helper behavior and its paired init use.

## Peripheral event generation and acknowledgement

The static image establishes Timer1 channel configuration and NVIC enable. It does not expose a positively identified Timer1 interrupt-status/flag register or a flag clear sequence in the wrapper. No W1C/W0C acknowledgement operation has been identified. The handler does not read a timer status register, write a timer status bit, call an identifiable interrupt-clear helper, disable the NVIC line, or write ICPR0.

Possible designs still consistent with the observed code include:

- Peripheral source is automatically cleared by exception entry or by a counter/reload condition.
- A status/acknowledgement helper is called by hidden ROM or hardware glue not present in this image.
- The timer source is level-sensitive and the channel disable/re-enable sequence affects request assertion, while the actual pending semantics are elsewhere.
- The datasheet or current address mapping is for a different silicon revision, and the actual source status register has not yet been mapped.

None of these alternatives is proven. **Peripheral flag generation, read semantics, clear semantics, and pending-state behavior are `UNKNOWN`.** Do not infer that disabling Timer1 clears a latched request.

## Timer2 independent comparison

Timer2 init at `0x10214` configures channel `2`, calls the same generic helpers, and enables NVIC index `15`. Table slot 15 maps to handler `0x103c8`. That handler writes a state byte at object offset `0x0f` and returns; it does not share the RGB wrapper. This independently supports the channel-index-to-IRQ numbering used by the application but does not prove hardware routing for either channel.

## Wrapper concurrency and timing

The wrapper's channel disable/re-enable is not a demonstrated critical section:

- It does not mask global interrupts (`CPSID i` is absent from `0x103b0`).
- It does not disable NVIC IRQ14 through `0x1d312`.
- It does not clear NVIC pending state.
- Other interrupts may therefore remain eligible during the output routine, subject to priority/masking state established elsewhere.

The operation may prevent the Timer1 source from generating repeated requests while the output routine runs, but whether it stops the counter, pauses it, masks only the interrupt source, resets the count, or clears a pending request depends on unresolved control-register semantics. If the output routine runs longer than the timer interval, the result could be a deferred or immediate retrigger on re-enable, but this is a possibility rather than an observed outcome. No output-duration or timer-frequency measurement exists.

**Classification:** `CONFIRMED_STATIC` for channel bit clear/call/set ordering; `UNKNOWN` for counter restart-versus-resume, pending-event retention, nested IRQ behavior under actual priorities, and immediate retrigger.

## Relationship to RGB output

`0x103b0` calls `0x135de`, the previously reconstructed output routine. Earlier Phase 13 work established that state byte `0x20348 == 0xfd` bypasses the normal frame-transmission path. That software guard does not establish whether Timer1 continues to run, whether the handler still occurs, or whether any electrical LED waveform is produced. The firmware output path remains distinct from proof of a physical LED protocol.

## Hardware boundary

The static firmware proves the software chain from Timer1 initialization to NVIC index 14, application stub argument `0x38`, table slot 14, and handler `0x103b0`. It does not prove:

- that the exact keyboard PCB contains the candidate YC3121-family MCU;
- that the peripheral source is physically wired to NVIC input 14 on that silicon;
- that the ROM/hardware selects the analyzed application stub bank on reset;
- that the timer event occurs at runtime;
- that the handler has ever executed on the user's device;
- that `0x135de` produces the expected electrical LED signal.

Those boundaries require exact part documentation, matching SDK source, or separately authorized hardware evidence. They are outside this static-only phase.

## Cross-references

- [Silicon and boot architecture](PHASE13_7_SILICON_AND_BOOT_ARCHITECTURE.md)
- [Timer1 register and clock analysis](PHASE13_7_TIMER1_REGISTER_AND_CLOCK_ANALYSIS.md)
- [Findings and hardware boundary](PHASE13_7_FINDINGS_AND_HARDWARE_BOUNDARY.md)
- Historical lifecycle: [Phase 13.6 IRQ14 lifecycle and RGB trigger](PHASE13_6_IRQ14_LIFECYCLE_AND_RGB_TRIGGER.md)
