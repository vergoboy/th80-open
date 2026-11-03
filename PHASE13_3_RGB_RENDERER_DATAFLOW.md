# Phase 13.3 — RGB Renderer Dataflow

## 1. Executive result

The previously missing producer edge is now recovered at the static-memory level:

```text
profile state 0x20400
   |  active-profile selection + mode/speed/value/RGB lookup
   v
0x13b92  [dispatch entry at 0x14810]
   |  per-cell matrix + runtime header
   v
0x2077c
   |  separate effect/update routine 0x153d4 [dispatch branch at 0x16336]
   |  per-cell scaling and channel calculation
   v
0x164a6  [literal base 0x20a78 at 0x168c0]
   v
0x20a78  [8 x 21 x 4-byte records]
   |  0x16c64 copies bytes +1..+3
   v
0x20d18  [8 x 21 x 4-byte destination records]
   |  0x13526 builds 16 words in 0x20380
   v
0x135de -> 16 x 0x134f2 -> 0x133d2/control
```

The `0x13b92` to `0x153d4` relation is a shared-state producer/consumer relationship, not a direct function call. The final edge from GPIO-like register writes to physical LEDs is still unproven.

## 2. Configuration storage at `0x20400`

`0x19afe` (opcode `0x08`) writes the active profile selector at `0x20400+0x1a`, mode at `0x20400+0x40+profile`, speed at `+0x60+profile`, value at `+0x80+profile`, and three color bytes at `+0xa0+3*profile`. The paired reader at `0x195ca` reads the same locations. The full seven-field command path is confirmed statically; its frontend caller/transport compatibility remains a separate unresolved issue documented in Phase 13.2.

The opcode handler calls `0x146f8` after writing. It does not call `0x13b92` or `0x153d4` directly. Thus the configuration write's direct refresh side effect is a partial buffer clear/copy; a full runtime render path is not proven to execute synchronously as a consequence of the opcode write.

## 3. Profile selection and intermediate matrix construction

At `0x13b92`:

1. Load `profile = *(0x20400+0x1a)`.
2. Read mode, speed and value from the three profile-indexed arrays at offsets `0x40`, `0x60`, `0x80`.
3. Read the profile's three bytes at `0xa0 + 3*profile`.
4. Write a runtime header to `0x2077c` and selected parameters to the adjacent state structure around `0x20a5c`.
5. Loop six rows and 21 columns, selecting per-cell data from a firmware table (pointer literal at `0x13c40`) and writing four-byte records into the matrix beginning at `0x2077c+3`, row stride `0x54`.

Mode branch: values `0..6` select three bytes from the lookup table; value `7` uses the profile's RGB triplet. Other values do not follow either of those two branches. This proves that the stored profile RGB is not necessarily the source for all modes. It does not establish user-facing names for modes or the precise meaning of each lookup entry.

`0x13b92` has a direct caller at `0x14810`, which is one branch in a dispatch/stub region around `0x14730–0x1483e`. This is static call evidence, not proof of periodic scheduling.

## 4. Intermediate consumer and renderer calculation

`0x153d4` is reached by a separate branch at `0x16336`. It reads and updates header bytes in `0x2077c`, including a decrement/reload field, and traverses a six-by-21 set of four-byte per-cell records. It also reads effect/profile state from `0x20400`, `0x20a5c`, and a firmware table. It computes a per-cell intensity/scale and calls `0x164a6` at `0x15398` with row/column and channel arguments. The `0x164a6` call writes the result to `0x20a78`.

The code therefore establishes an upstream data dependency from the profile-selected per-cell matrix through the effect/update routine to the candidate output buffer. Some transformations use lookup values and arithmetic; the exact meaning of every mode branch and time-dependent state remains partially unresolved. Do not describe `0x2077c` as the final output buffer.

## 5. `0x20a78` record representation

The `0x164a6` function stores one control/scale byte at offset zero and three calculated component bytes at offsets one through three. The destination address is `0x20a78 + row*0x54 + column*4`. The full 8×21 copy span is `0x2a0` bytes. The copy consumer independently proves that only bytes `+1..+3` are propagated to `0x20d18`; byte zero is not copied by `0x16c64`.

The shape is strongly consistent with three-channel colour records, but the channel order and physical interpretation are not proven. The record's first byte is demonstrably used as a scale/control value by `0x16ab2`.

## 6. Copy routine `0x16c64`

**`CONFIRMED_STATIC`.** Literal pool at `0x16cac` contains `0x00020a78`; pool at `0x16cd0` contains `0x00020d18`. The routine loops rows `0..7` with stride `0x54`, and columns `0..20` with stride four. For each record it loads source bytes `+1`, `+2`, `+3` and stores them at the corresponding destination offsets. It does not copy byte zero and performs no arithmetic or channel reordering.

Direct callers recovered in this disassembly:

- `0x10b1c`
- `0x10e20`
- `0x113d4`
- `0x1159e`
- `0x1472a`

Therefore the copy is a shared operation, not exclusive to opcode `0x08` or one renderer. The `0x146f8` caller clears only a 7×15 subset before copying all 8×21 records; other paths may populate or clear the entire source region.

## 7. Downstream frame preparation and output

`0x13526` indexes the `0x20d18` data, expands byte values into repeated-byte 16-bit patterns (`v | (v << 8)`), and stores a 16-word frame in `0x20380`. `0x135de` calls it in the active branch, iterates 16 words from `0x20380`, and calls `0x134f2` for each. It then calls timing/control helpers including `0x104cc`, `0x1a9a8`, `0x1a9b2`, and `0x133d2`. Alternate branches include zero-word output loops and state/counter changes.

The strongest recovered call chain is `0x103b0 -> 0x135de -> 0x13526 -> 16 x 0x134f2`, followed by `0x133d2` and timing/state helpers. The direct call `0x103ba -> 0x135de` is confirmed. The caller/registration of wrapper `0x103b0` is not identified, so scheduler semantics remain unknown.

## 8. Static, animated and refresh paths

- The `0x13b92`/`0x153d4` path uses the active profile, a six-by-21 per-cell matrix, a mode/effect state header and time-dependent update state. This is `STRONG_STATIC_INFERENCE` of a dynamic renderer path.
- Modes `0..6` use a firmware lookup table; mode `7` uses the selected profile RGB triplet. This branch behavior is `CONFIRMED_STATIC`; human-readable effect names are `UNKNOWN`.
- `0x146f8` clears `0x2077c` for `0x2fc` bytes, writes zero records to a 7×15 subset of `0x20a78`, then copies the complete 8×21 triple-channel area. It is a refresh/partial clear path, not evidence of a complete colour calculation.
- Five separate paths zero-fill all `0x2a0` bytes of `0x20a78` through `0x1d45e` (`0x1133e`, `0x118ca`, `0x13406`, `0x134ea`, `0x1c654`). Their trigger contexts differ.

## 9. Execution sequence and scheduling limits

The firmware contains configuration writes, profile-to-matrix preparation, separate effect/update routines, copy operations and a GPIO-like output consumer. However, the exact temporal relationship is not closed: `0x19afe` directly calls `0x146f8`, while `0x13b92` and `0x153d4` are reached through separate dispatch branches. No recovered direct call from the opcode handler to `0x13b92`/`0x153d4` proves an immediate full frame render. `0x103b0` calls the output loop, but its callback/timer/main-loop registration remains unresolved.

Classification:

- Profile state stores/reads: `CONFIRMED_STATIC`.
- `0x13b92` writes profile-derived matrix state: `CONFIRMED_STATIC`.
- `0x153d4` consumes that matrix and emits scaled records through `0x164a6`: `STRONG_STATIC_INFERENCE` for renderer semantics; direct memory/call operations are confirmed.
- `0x16c64` copy geometry: `CONFIRMED_STATIC`.
- `0x13526` frame preparation and `0x135de` output sequence: `CONFIRMED_STATIC` for data accesses/calls.
- Full periodic scheduler and immediate configuration-to-output ordering: `UNKNOWN`.

## 10. Evidence and references

- `0x19afe` / `0x195ca`: profile-indexed configuration writer/reader; see Phase 13.2 output-path report.
- `0x13b92–0x13cd6`: profile selection and matrix construction; `0x13c28=0x20400`, `0x13c30=0x2077c`, `0x14080=0x2077c`.
- `0x153d4–0x155d8`: effect/update consumer; `0x155b8=0x2077c`, `0x155cc=0x20400`.
- `0x164a6–0x164f4`: record writer; `0x168c0=0x20a78`.
- `0x16c64–0x16c94`: copy; `0x16cac=0x20a78`, `0x16cd0=0x20d18`.
- `0x13526–0x135dc`, `0x135de–0x136d0`: frame preparation and consumer.
- `0x103b0–0x103c6`: wrapper around output routine.
