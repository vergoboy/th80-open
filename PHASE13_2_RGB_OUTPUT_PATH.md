# Phase 13.2 — RGB configuration and physical output-path investigation

**Scope:** static disassembly and data-reference analysis only. No firmware execution, patching, flashing, USB/HID operation, or physical LED test was performed.

## 1. Address and confidence conventions

All addresses are raw-file offsets in firmware `X`; the image load base and actual virtual-address mapping remain unknown. Candidate peripheral addresses are retained as raw constants. Confidence labels are exactly `CONFIRMED_STATIC`, `STRONG_STATIC_INFERENCE`, `WEAK_INFERENCE`, `FRONTEND_ONLY`, and `UNKNOWN`.

## 2. Complete lighting configuration writer and state layout

### 2.1 Opcode `0x08` handler at `0x19afe`

The dispatcher routes opcode `0x08` to the handler at `0x19afe`. It reads the shared command buffer through the pointer literal `0x21240` (pool at `0x19b4c`) and uses the profile selector at command byte `[1]`.

The profile-indexed state base is calculated from pointer literal `0x20420` minus `0x20`, yielding candidate state base `0x20400`. The handler stores the profile selector plus mode, speed, value, and three color bytes:

| Command byte | Field | State location | Operation |
|---:|---|---|---|
| `[1]` | Profile index | `0x20400 + 0x1a` | Stores active profile selector |
| `[2]` | Mode | `0x20400 + 0x40 + profile` | Direct byte store |
| `[3]` | Speed | `0x20400 + 0x60 + profile` | Direct byte store |
| `[4]` | Value/intensity | `0x20400 + 0x80 + profile` | Direct byte store |
| `[5]` | Red / first color byte | `0x20400 + 0xa0 + 3*profile` | Direct byte store |
| `[6]` | Green / second color byte | `0x20400 + 0xa1 + 3*profile` | Direct byte store |
| `[7]` | Blue / third color byte | `0x20400 + 0xa2 + 3*profile` | Direct byte store |

The code stores bytes directly; no normalization or clamp is visible in this handler. Any validation must occur earlier or elsewhere and is not established here.

After storing the fields, it updates flags through globals around `0x19ee8–0x19ef0`, calls `0x146f8` at `0x19bac`, then sets a completion/status byte. The handler does not itself call the lowest hardware-facing routines.

Classification: `CONFIRMED_STATIC` for the seven byte stores (profile selector plus six data bytes), profile-indexed address arithmetic, and the subsequent call to `0x146f8`. The firmware stores no separate option byte in this handler. The frontend layout includes a packed option before RGB, so the two layouts must not be silently equated.

### 2.2 Paired getter at `0x195ca` (opcode `0x87`)

The getter reads the same profile selector at `0x20400+0x1a`, then reads mode, speed, value, and the three color bytes from the corresponding offsets in the table above. No separate option byte is read by this firmware handler. It writes response bytes `[1]` through `[7]` in the shared response structure and sets a 64-byte response length at the response descriptor (`0x200d8` region).

Classification: `CONFIRMED_STATIC` for the read offsets and response-field writes. This independently corroborates the state layout but does not prove physical LED output.

### 2.3 Separate compact parameter pair

The compact setter `0x07` around `0x19bb6` and getter `0x86` at `0x19614` are separate from the complete profile-indexed pair `0x08`/`0x87`. Existing vendor correlation shows the frontend's TH80 light-setting path uses opcode `0x07`, while the firmware also contains the distinct complete writer at `0x08`. Do not merge these command layouts.

## 3. Renderer and intermediate matrix data

### 3.1 Refresh/transform routine at `0x146f8`

`0x19afe` calls `0x146f8` after updating the profile fields. The routine:

1. Initializes loop state and iterates seven rows (`r5=0..6`) and fifteen columns (`r4=0..14`).
2. Calls `0x164a6` for each coordinate.
3. Calls `0x16c64` after the nested loops.

At `0x164a6`, the firmware writes three calculated byte components into a four-byte-stride matrix record based at `0x2077c`, using row stride `0x54` and column offset `4*column`. The fourth byte stores the row/coordinate value. The call arguments supplied by `0x146f8` include zero-valued parameters; the routine's precise role should be treated as a matrix initialization/transform rather than assumed to be the final LED output.

### 3.2 Bulk matrix copy at `0x16c64`

The routine uses source base `0x20a78` and destination base `0x20d18`, both loaded through literal pools. It iterates eight rows with stride `0x54` (84 bytes) and 21 entries per row. For each entry it copies bytes at offsets `+1`, `+2`, and `+3` from source to destination; it leaves the first byte untouched.

This establishes a structured three-byte-per-entry data copy. It is consistent with RGB triples but the channel semantics and the origin of source buffer `0x20a78` are not independently proven by this copy alone.

`0x146f8` calls `0x16c64`, and `0x19afe` calls `0x146f8`. Thus an opcode-`0x08` write triggers this refresh/copy sequence. However, `0x164a6` writes to `0x2077c`, whereas `0x16c64` reads from `0x20a78`; no direct equality or copy from `0x2077c` to `0x20a78` was recovered. The exact data dependency from the seven configuration fields into the bytes copied to `0x20d18` remains incomplete.

Classification: `CONFIRMED_STATIC` for the call sequence and exact copy geometry; `STRONG_STATIC_INFERENCE` that the three-byte records represent color data; `UNKNOWN` for the upstream producer of `0x20a78` and the precise influence of the profile selector and mode/speed/value fields on it.

## 4. Candidate runtime consumer and GPIO-facing output

### 4.1 Frame preparation at `0x13526`

`0x13526` receives an index in `r0`, computes `4*index`, and reads fields from the table at `0x20d18`. It reads bytes at several offsets separated by `0x40`, `0xa0`, `0xe0`, and further blocks, duplicates 8-bit values into 16-bit words (`value + (value << 8)`), and stores a sequence of 16 32-bit words into the buffer based at `0x20380` (loaded from the literal pool at `0x137a0`). This is a concrete transformation from the `0x20d18` table into a 16-word output buffer.

Classification: `CONFIRMED_STATIC` for source/destination accesses and byte-expansion arithmetic; `STRONG_STATIC_INFERENCE` that this is preparation of a serial hardware frame.

### 4.2 Runtime loop at `0x135de`

`0x135de` reads state at `0x20340+8`, a counter/index at `0x20076`, and a pointer to the output words at `0x20380`. In its active branch it calls `0x13526`, then iterates 16 words from `0x20380`, calling `0x134f2` once per word. It then calls timing/control helpers (`0x104cc`, `0x1a9a8`, `0x1a9b2`) and `0x133d2`. Alternate branches also call `0x134f2` in 16-word loops and update state/counters.

A wrapper at `0x103b0` gates the call to `0x135de` with `0x1a9b2` operations. The direct call `0x103ba -> 0x135de` is confirmed; the scheduler or callback registration that invokes `0x103b0` is not.

Classification: `CONFIRMED_STATIC` for the data reads, loop count, and direct call relationships; `STRONG_STATIC_INFERENCE` that `0x135de` is a periodic/frame-update routine.

### 4.3 Lowest identifiable GPIO-like functions

`0x133d2`:

- Loads a peripheral-like base constant `0x000f8720` from literal `0x13778`.
- Writes control values to base offsets `+0x0a` and `+0x0b` in a 16-iteration loop.
- Shifts the input word between iterations and selects one of two values (`0x3e`/`0x3f`) based on the current bit.
- Also writes a control byte at `0x000f8705` (base `0x000f8700` plus 5).

`0x134f2`:

- Uses the same peripheral-like base `0x000f8720`.
- Runs a 32-iteration bit loop, shifting the input word and toggling the same `+0x0a`/`+0x0b` register bytes.
- Writes a control byte at `0x000f8705`.

These routines perform GPIO-like register writes in bit-serial patterns, and `0x135de` directly feeds 16 words to `0x134f2`. This is strong evidence for a GPIO-driven serial output mechanism rather than a conventional SPI/I2C transfer call. The register identity, actual pin numbers, electrical polarity, timing, attached controller, and whether the signal is directly connected to addressable LEDs are unknown.

Classification: `CONFIRMED_STATIC` for the register addresses, write sequence, and bit-loop behavior; `STRONG_STATIC_INFERENCE` for GPIO bit-banging/serial frame output; `UNKNOWN` for pin/controller/protocol identity.

## 5. Reconstructed output sequence and remaining gap

The best-supported static sequence is:

```text
opcode 0x08
  -> 0x19afe writes profile-indexed lighting state at 0x20400
  -> 0x146f8
       -> 0x164a6 updates a coordinate matrix at 0x2077c
       -> 0x16c64 copies 8 x 21 x 3 bytes from 0x20a78 to 0x20d18

runtime/update path:
  0x103b0 -> 0x135de
       -> 0x13526 reads 0x20d18 and builds 16 words at 0x20380
       -> 16 x 0x134f2 bit-serial output operations
       -> 0x133d2 control/latch-like operation
       -> timing/state helpers
```

The bridge between the profile selector and mode/speed/value fields at `0x20400` and the RGB triples at `0x20a78` is not yet established. The copy into `0x20d18` is invoked by the `0x08` refresh path, and the runtime consumer reads `0x20d18`, but that alone does not prove that the profile selector and six command data bytes determine the output values. The source producer for `0x20a78`, plus any mode/effect calculations that feed it, is the single most important remaining RGB data-flow gap.

No actual timer/PWM, SPI, I2C, or DMA RGB transfer was established. SDK strings such as `TIM_PWMInit`, `TIM_SetPWMPeriod`, and `TIM_PWMDifferential` prove SDK capability only and are not used here as proof of LED control.

## 6. USB/wireless and failure interpretation

The same host-captured HID report descriptors are recorded for keyboard `3151:4010` and receiver `3151:4011`, but this does not establish that the same firmware image or identical RGB runtime is used in both devices. No independent wireless-side RGB initialization or separate radio command path was proven in this phase.

The user-reported failure over both USB and 2.4 GHz makes an exclusively USB transport failure less likely, but it does not identify a hardware fault. The static code shows a software-controlled GPIO-like output stage exists; it cannot establish whether that stage is being scheduled, whether the relevant data buffer is valid, whether the GPIO peripheral is powered/configured, or whether the LED hardware is electrically functional.

## 7. Safe implementation boundary

The evidence supports additional offline Rust functionality such as parsing and displaying the profile selector plus mode/speed/value/RGB state, decoding captured descriptor metadata, and visualizing known matrix/page structures. It does **not** justify enabling a hardware transport, issuing opcodes, changing settings, or implementing an active RGB controller. Any future hardware test requires a separately reviewed protocol and safety plan; no test is authorized by this report.
