# Phase 13.3 — LED Output Protocol Analysis

## 1. Scope and conclusion

This report covers only static instruction/data-flow evidence. No peripheral was accessed, no HID operation was performed, and no physical waveform or LED response was measured.

The firmware contains a repeatable bit-serial register-write sequence that consumes data prepared from `0x20d18`. The sequence is consistent with software-driven serial signalling using a candidate peripheral register area at `0x000f8720` plus a separate byte at `0x000f8705`. The exact peripheral, pin mapping, electrical role, attached LED controller and physical connection are **UNKNOWN**. Do not label it SPI, I2C, WS2812, PWM, or a particular LED protocol.

## 2. Frame-data path

`0x13526` receives an index, computes `4*index`, and uses literal `0x1379c = 0x20d18` as its source base. It gathers byte values from six index-relative blocks at source offsets `+0x01..03`, `+0x55..57`, `+0xa9..ab`, `+0xfd..ff`, `+0x151..153`, and `+0x1a5..a7`. Each byte is expanded to a repeated-byte halfword (`v | (v << 8)`). The first 15 expanded halfwords are written as 32-bit words at `0x20380+0x3c` down through `+0x08`; the final two words at `+0x04` and `+0x00` combine repeated-byte halfwords from the last two source groups. The routine therefore writes exactly 16 words (64 bytes) to `0x20380`. This exact gather/packing operation is `CONFIRMED_STATIC`; what each source lane means is not established.

`0x135de` calls `0x13526` in its active branch, then loops over 16 words from `0x20380`, calling `0x134f2` once per word. The direct call sequence and loop count are `CONFIRMED_STATIC`.

The output consumer also calls `0x104cc`, `0x1a9a8`, `0x1a9b2`, and `0x133d2`; alternate branches contain zero-word output loops and update state/counters. These helpers likely provide timing/control coordination, but their exact electrical semantics are not established.

## 3. Routine `0x134f2`: 32-bit bit loop

Relevant instructions: `0x134f2–0x13524`.

- Loads `0x000f8720` through literal pool at `0x13778`.
- Initializes byte `[base+0x0a]` and `[base+0x0b]` with `0x3e`.
- Computes a second pointer `base-0x20`, i.e. candidate address `0x000f8700`.
- Iterates `r1=0..31` (32 iterations).
- For each bit, tests the sign bit of the shifting `r0`; writes either `0x3f` or `0x3e` to `[base-0x20+5]`, candidate address `0x000f8705`.
- Writes `0x3f` to `[base+0x0b]` and shifts `r0` left by one.
- Ends by writing `0x3e` to `[base+0x0a]`, `[base-0x20+5]`, and `[base+0x0b]`.

The input word is therefore serialized MSB-first by sign-bit testing and left shifting. The repeated writes to the `+0x0a/+0x0b` bytes and the separate `+5` byte are static facts. Their roles as clock/data/enable lines are not established.

## 4. Routine `0x133d2`: 16-bit control-like loop

Relevant instructions: `0x133d2–0x133fe`.

- Uses the same literal `0x000f8720` at `0x13778`.
- Writes `0x3e` to `[base+0x0b]` before the loop.
- Iterates 16 times.
- Tests a high bit of the input by shifting `r0` left 16 and checking the sign condition; writes either `0x3f` or `0x3e` to `[base+0x0a]`.
- Writes `0x3f` to `[base+0x0b]`, shifts the input and repeats.
- Finishes by setting `[base+0x0a]` and `[base+0x0b]` to `0x3e/0x3f`-style idle values and writes `0x3f` at `0x000f8705`.

This is another fixed-length bit-pattern operation, but its exact relation to a frame latch/reset sequence is not established. It is called from `0x135de` and from other initialization/control routines.

## 5. Runtime ordering and boundaries

`0x135de` uses the output state byte at `0x20340+8`, a counter/index at `0x20076`, and output words at `0x20380`. In the active path it prepares one indexed frame, transmits 16 words via `0x134f2`, calls timing/state helpers, then calls `0x133d2`. Alternate branches transmit zero words and adjust state. `0x103b0` wraps `0x135de` with calls to `0x1a9b2`; the direct call from `0x103b0` to `0x135de` is confirmed.

No direct call to `0x103b0` was recovered in the inspected direct-call inventory. The routine may be invoked through a callback, timer dispatch, table or indirect call; the registration edge remains unresolved. The static code does not establish whether this output sequence is periodic, event-driven, or conditionally enabled by a subsystem state.

## 6. Protocol classification

| Candidate | Evidence | Classification |
|---|---|---|
| GPIO-like bit-banging | Repeated byte stores to two nearby register offsets and a separate offset, selected by data bits | `STRONG_STATIC_INFERENCE` |
| Clocked serial | Two outputs toggle in a fixed bit loop, but clock/data polarity and edge relationship are unknown | `WEAK_INFERENCE` |
| Single-wire addressable LED protocol | No measured timing, reset interval, pulse-width classes, or encoded frame format | `UNKNOWN` |
| SPI/I2C peripheral transfer | No proven SPI/I2C register block, peripheral API call, or bus transaction | `UNKNOWN` |
| PWM-driven LED output | Timer/PWM SDK strings/capability alone do not link PWM to this path | `UNKNOWN` |
| External LED controller | No controller identity or bus/device reference established | `UNKNOWN` |

No protocol name should be assigned. The instruction-level bit loops support a GPIO-like serial candidate, but static instruction spacing is insufficient to infer actual nanosecond/microsecond pulse timing without a verified clock and execution context.

## 7. Peripheral identity and physical connection

The literal pool word at `0x13778` is exactly `0x000f8720`. Code also accesses `0x000f8705` by subtracting `0x20` from that base. This proves the constants and writes in the raw binary; it does not prove that the address is a particular MCU peripheral register block. No authoritative register map, pin mux, board schematic, LED-controller datasheet correlation, or electrical evidence was available in this phase.

A successful software store to these addresses would not by itself prove a physical signal reaches the LED chain. Output enable, GPIO mux, power rail, external controller and board routing remain unverified.

## 8. RGB-off failure implications

The static output path yields several testable *hypotheses*, not a diagnosis:

- **Output routine not scheduled:** supported by the unresolved `0x103b0` caller/registration; contradicted by the existence of the callable routine only in the sense that code is present. Missing: actual runtime scheduling evidence.
- **Output-enable/peripheral initialization absent or conditional:** the code uses state branches and control helpers; missing: full initialization and register semantics.
- **Output source data zero/stale:** whole-buffer clears and zero-word branches exist; missing: current runtime contents and path frequency.
- **Power/physical LED chain fault:** cannot be confirmed or contradicted by this static binary; missing: board-level evidence, intentionally not collected.
- **Configuration/profile mismatch:** the active profile and mode determine matrix construction; missing: current device state and proof the user's host command reaches opcode `0x08`.
- **USB-only transport fault:** reported failure across USB and 2.4 GHz makes an exclusively USB-side fault less likely, but does not distinguish shared firmware logic, receiver behavior, power or LED hardware.

No arbitrary reset, firmware flash, command execution, setting change or hardware test is recommended by this report.

## 9. Evidence index

- `0x134f2–0x13524`: 32-bit bit-serial candidate.
- `0x133d2–0x133fe`: 16-bit control-like sequence.
- `0x13526–0x135dc`: `0x20d18` to `0x20380` frame preparation.
- `0x135de–0x136d0`: runtime frame loop and control calls.
- `0x103b0–0x103c6`: wrapper calling `0x135de`.
- `0x13778`: little-endian literal `0x000f8720`.
- Phase 13.2 baseline: `PHASE13_2_RGB_OUTPUT_PATH.md`, sections 4–5.
