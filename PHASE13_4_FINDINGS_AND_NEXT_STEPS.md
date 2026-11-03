# Phase 13.4 — Findings and Next Steps

## 1. Strongest discoveries

1. **`0x103b0` is an output wrapper, not the renderer.** It clears bit 4 at `0x000f0c48` through `0x1a9b2`, calls `0x135de`, then sets that bit again. The bit's peripheral/pin identity is unknown. `CONFIRMED_STATIC` for instruction behavior; `STRONG_STATIC_INFERENCE` for output-wrapper role.
2. **No direct caller of `0x103b0` was found.** A complete direct-call scan found zero `BL` callers. No searched absolute Thumb pointer literal to `0x103b0` exists in the raw image. `CONFIRMED_STATIC` for the static search; not proof that runtime dispatch never invokes it.
3. **The firmware has a real interrupt-dispatch skeleton.** The vector-like stubs at `0x1001c–0x10098` route indices through `0x1009c`, which loads a handler pointer from observed base `0x010102bc` plus the vector offset and branches to it. The index-5 slot is `+0x14`; its runtime target is unknown.
4. **A candidate application-init block configures a timer-like peripheral and enables NVIC IRQ 5.** `0x173cc` is called at `0x102de` inside the `0x1027e` block and later from transition paths (`0x10974`, `0x11360`). The block’s entry from reset is unresolved. It enables IRQ 5 through `0x1d304`, configures the block selected at `0x000f8b00`, and uses `500000` with a `48,000,000` constant to derive 96. Timer period and frequency remain unknown.
5. **Output initialization code exists.** `0x13400` is called at `0x102ee` inside the candidate application-init block; the block's reachability from reset is unresolved. It clears the 672-byte candidate buffer, configures register-like state, invokes `0x133d2` and other output helpers, and is also called from later transitions.
6. **A shared state byte can suppress output.** Both the main loop and `0x135de` read `0x20348`. The main loop dispatches `0xfd` to `0x11328`; `0x135de` skips its normal frame path when the same byte equals `0xfd`. `CONFIRMED_STATIC` for the branch behavior.

## 2. Reconstructed runtime sequence — proven and unresolved edges

```text
Two observed early entry stubs (ownership/relationship unresolved):
  0x200 -> 0x2acc runtime callback walker -> BX 0x01000899
  0x10200 -> 0x1d488 runtime callback walker -> BX 0x0101047f
       -> if base=0x01000000: candidate offset 0x1047e configures timer scalar 115200
       -> if that mapping holds: POP PC from runtime stack (value unknown)
       -> alternate segment mapping may resolve target to offset 0x47e; unresolved

Separate candidate application-init block:
  0x1027e -> 0x173cc timer-like peripheral setup + NVIC IRQ 5 enable
         -> 0x13400 candidate output initialization + clear 0x20a78
         -> main polling loop at 0x10308
  The entry edge into 0x1027e is not proven.
       -> dispatch state byte 0x20348
       -> state 0xfd: 0x11328 (buffer clear / timer reconfiguration path)
       -> state 0xfe: 0x114f2
       -> other state handlers

Candidate interrupt route:
  IRQ 5 vector stub at 0x10030
    -> 0x1009c
    -> handler pointer [0x010102bc + 0x14]
    -> unknown target
    -> ? 0x103b0

Known output path:
  0x103b0
    -> bit-control operation at 0x000f0c48
    -> 0x135de
    -> 0x13526 / 16-word frame preparation
    -> 0x134f2 / 0x133d2 output routines
```

The final `? 0x103b0` edge is the central unresolved point. Timer initialization and vector dispatch are both present, but no runtime handler-table entry was recovered.

## 3. Scheduler conclusion

The firmware has at least two execution mechanisms:

- A cooperative main polling loop at `0x10308`, which repeatedly dispatches a shared state byte and calls `0x1076c`.
- A vector-like interrupt dispatch framework that resolves handlers through a runtime table at `0x010102bc`.

The timer setup in the candidate application-init block enables NVIC interrupt index 5; the `0x10200` reset trampoline branches to `0x0101047f`; only under the `0x01000000` base assumption does that resolve to code at `0x1047e`, which configures timer selector zero with scalar 115200 before returning through an unknown stack-supplied PC. It is plausible that this timer/interrupt framework participates in RGB updates, but the IRQ 5 handler has not been resolved. The main polling loop itself does not directly call `0x103b0`.

No defensible RGB refresh period or frame rate can be calculated. The scalar `500000` and quotient `96` are timer-configuration inputs, not a proven time interval.

## 4. Implications for the RGB failure

The static code supports several independent failure points:

- Runtime callback/vector table may not route to `0x103b0`.
- State `0xfd` explicitly suppresses the normal output path.
- The candidate init block and transition code clear the candidate output buffer; the later repopulation path may or may not run.
- Output initialization code exists, but its register and physical pin semantics are unknown.
- Timer initialization is present, but the interrupt-to-handler edge and actual timer period are unresolved.

None of these is established as the cause of the user's physical symptom. The USB/2.4 GHz observation does not identify the shared firmware, receiver, power or LED hardware failure point.

## 5. Can an RGB engine implementation specification be written?

A **partial static software specification** can be written for:

- The 4-byte records and 672-byte candidate buffer from Phase 13.3.
- The copy from `0x20a78` to `0x20d18`.
- The 16-word frame preparation and candidate bit-serial output routines.
- The `0x20348 == 0xfd` output suppression condition.
- The `0x103b0` output wrapper's bit-clear / transmit / bit-set sequence.

A safe, complete runtime specification cannot yet be written because:

- No proven caller or registration for `0x103b0` exists.
- IRQ 5's vector-table target is unknown.
- The timer interval and clock semantics are unknown.
- The runtime callback/vector table contents are missing.
- Physical GPIO/pin mapping, LED power and LED-controller connection remain unconfirmed.

Therefore no RGB hardware transport or application implementation should be based on this phase alone.

## 6. Most valuable next investigation

**Recover the initialization and runtime contents of the interrupt-handler table at `0x010102bc`, especially the slot at `+0x14` for enabled NVIC interrupt index 5.**

The investigation should:

1. Find every writer or initializer for the vector-table base and its entries.
2. Determine whether the table is RAM, ROM, bootloader-owned or a memory-mapped dispatch structure; do not assume the current firmware file-offset mapping applies to it.
3. Resolve the `+0x14` entry to its function pointer.
4. Compare that target with `0x103b0`, `0x1034c`, `0x103d0`, `0x10418` and other nearby callback-like routines.
5. Trace the source of the runtime handler-table contents, including any boot ROM or companion firmware component if available as a static artifact.
6. Only after resolving the target, establish whether the timer setup at `0x173cc` is the scheduler that activates RGB output.

This investigation is more valuable than guessing a refresh interval or LED protocol.

## 7. Preservation and safety

- Original firmware, native driver and frontend were fingerprinted before report creation; their expected hashes matched.
- Prior Phase 13/13.1/13.2/13.3 reports were fingerprinted before writing and were not edited.
- No firmware, driver, frontend, Rust source, Cargo dependency or GUI changes were made.
- No HID enumeration, device open, HID report, USB transfer, vendor configurator, firmware execution, reset, pairing, calibration, bootloader or update operation was performed.
- No files were staged or committed; no remote was added.

Phase 13.4 stops at static analysis and reporting.

## Related reports

- [Runtime entry and call graph](PHASE13_4_RGB_RUNTIME_ENTRY_AND_CALLGRAPH.md)
- [Scheduler, timer and initialization](PHASE13_4_SCHEDULER_TIMER_AND_INITIALIZATION.md)
- [Runtime gating](PHASE13_4_RGB_RUNTIME_GATING.md)
- [Phase 13.3 findings](PHASE13_3_FINDINGS_AND_NEXT_STEPS.md)
