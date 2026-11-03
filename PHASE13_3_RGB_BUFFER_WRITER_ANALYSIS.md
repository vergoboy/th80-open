# Phase 13.3 — RGB Buffer Writer Analysis

## Scope and evidence conventions

Static analysis only. All instruction addresses below are raw-file offsets in firmware `X`; the image load base is unknown. Candidate addresses such as `0x20a78` are firmware-referenced memory addresses, not independently mapped virtual addresses. No firmware execution, patching, HID/USB communication, or physical keyboard operation was performed.

Confidence labels use exactly: `CONFIRMED_STATIC`, `STRONG_STATIC_INFERENCE`, `WEAK_INFERENCE`, `FRONTEND_ONLY`, `UNKNOWN`.

## 1. Recovered layout of `0x20a78`

**`CONFIRMED_STATIC` — writer `0x164a6`, literal pool `0x168c0`.** The 32-bit little-endian word at file offset `0x168c0` is `0x00020a78`. At `0x164c8`, `ldr r1` loads this pointer. The function computes `row * 0x54 + base + (column * 4)` and writes:

| Record offset | Store | Provenance |
|---:|---|---|
| `+0` | `strb r4, [r1, r7]` at `0x164f2` | `r4`, the third argument (`r2` on entry), stored as a per-record control/scale byte |
| `+1` | `strb r0, [r5,#1]` at `0x164d2` | first calculated channel |
| `+2` | `strb r0, [r5,#2]` at `0x164e0` | second calculated channel |
| `+3` | `strb r0, [r5,#3]` at `0x164ee` | third calculated channel |

Each channel calculation multiplies a supplied channel byte by the third argument and a stack-passed scale, then calls `0x1d3b4` with divisor `100`. The result is stored as one byte. Rows are separated by `0x54` (84) bytes; columns by four bytes. `0x16c64` independently traverses 8 rows × 21 records, so the full accessed allocation is `8 * 0x54 = 672 = 0x2a0` bytes, representing 168 four-byte records. The 4-byte record shape and three calculated channels are confirmed; naming the channels R/G/B remains a `STRONG_STATIC_INFERENCE`, not a pin/channel-order proof.

The routine returns without writing if row or column equals `0xff` (`0x164b0–0x164b6`). No general bounds check for values above the valid grid is visible in this routine.

## 2. Complete direct-call inventory for the identified writer

The disassembly contains 67 direct `BL 0x164a6` call sites:

```text
0x10f2c 0x10f56 0x10f80 0x10faa 0x10fc6 0x10ff0 0x1101a 0x11044
0x1108a 0x110b2 0x110d8 0x110fe 0x11124 0x11142 0x11166 0x11186
0x111a6 0x111c6 0x113d0 0x1158e 0x13f80 0x1471a 0x14892 0x148c0
0x14902 0x14920 0x149f0 0x14b94 0x14ce2 0x14dc6 0x14e04 0x14e5e
0x14e7e 0x14ed6 0x14f2a 0x1506a 0x15132 0x15170 0x151dc 0x151fe
0x15258 0x152f8 0x15398 0x155a6 0x1565c 0x15758 0x157fa 0x15886
0x158e6 0x15a30 0x15b00 0x15bae 0x15c6c 0x15cbe 0x15d7a 0x15ef6
0x15f8c 0x16046 0x160e0 0x16230 0x166e8 0x16aac 0x16b14 0x17758
0x17ef2 0x17f3a 0x17f76
```

This is the complete direct-call inventory in the generated Thumb disassembly, not proof that no indirect call or pointer alias exists. The sites span several effect/update routines; they are not all descendants of the opcode-`0x08` handler. Examples:

- `0x1471a`: `0x146f8` calls it for a 7×15 region with zero-valued channel inputs and zero metadata; this is a partial clear/write, not a profile-colour render.
- `0x15398`: a call from the `0x153d4` state/effect routine; it writes output records derived from the `0x2077c` intermediate records.
- `0x16aac` and `0x16b14`: calls inside the `0x16ab2` per-record fade/decay path.
- `0x1158e`: looped 21×8 traversal with source values from a separate per-cell table.

For the complete instruction-level arguments, inspect each call site in `/tmp/th80-X-thumb.dis`; register and stack argument setup differs between callers.

## 3. Additional writers, clears, and aliases

### 3.1 Whole-buffer zero fills

The raw words at the following literal-pool offsets equal `0x00020a78`:

- `0x115ac` (used at `0x1133e`)
- `0x11918` (used at `0x118ca`)
- `0x1377c` (used at `0x13406` and `0x134ea`)
- `0x1c848` (used at `0x1c654`)
- `0x168c0` (writer `0x164a6`)
- `0x16cac` (reader/copy source `0x16c64` and read-modify-write `0x16ab2`)

The helper at `0x1d45e` sets the fill value to zero and branches to `0x1d450`. Each identified clear call supplies length `0x2a0` (672 bytes), covering the full 8×21×4-byte region. Confirmed call sites are `0x1133e`, `0x118ca`, `0x13406`, `0x134ea`, and `0x1c654`. These are block writes, distinct from record writes. Their caller conditions determine when the buffer is cleared; they are not unconditional startup proof.

### 3.2 Conditional read-modify-write at `0x16ab2`

`0x16ab2` loads base `0x20a78` through pool word `0x16cac`, loops six rows and 23 candidate columns, and rejects columns `>= 21`. For valid entries it reads record byte `+0`; if zero it skips the record. Otherwise it subtracts a caller-supplied amount with floor-at-zero behavior, stores the updated byte `+0`, reads channel bytes `+1..+3`, and calls `0x164a6` with the updated scale/control byte and scale argument `1`. The writer therefore recalculates the three channel bytes as `channel * updated_byte / 100` while retaining the updated byte at `+0`. This is a confirmed per-record read/modify/rewrite path; its exact visual effect (fade, decay, or another effect) is `STRONG_STATIC_INFERENCE`.

### 3.3 Reference classification

| Reference | Classification | Evidence |
|---|---|---|
| `0x168c0 → 0x20a78` | Pointer setup / direct writer base | `0x164c8`, followed by four byte stores |
| `0x16cac → 0x20a78` | Pointer setup / read reference | `0x16ae2`, `0x16c66` |
| `0x115ac`, `0x11918`, `0x1377c`, `0x1c848` | Pointer setup / block-clear destination | Calls to zero-fill helper with `0x2a0` length |
| `0x16ab2` | Indirect-in-the-dataflow RMW writer | Reads and rewrites addressed records through `0x164a6` |
| `0x16c64` | Read source, not writer | Loads source bytes `+1..+3`; writes destination `0x20d18` |

No separate alias address for the same `0x20a78` storage was established. The six literal occurrences above are references to the same numeric base, not proof of additional buffers.

## 4. Upstream provenance and record construction

`0x13b92` reads the active profile selector from `0x20400+0x1a`, then reads the profile's mode (`+0x40+profile`), speed (`+0x60+profile`), value (`+0x80+profile`) and RGB triplet (`+0xa0+3*profile`). It writes a small runtime header at `0x2077c` and copies per-cell data into the matrix area beginning at `0x2077c+3`, with six rows, 21 columns, four bytes per cell and row stride `0x54`. The per-cell source is a firmware table at the literal-pool address referenced at `0x13c40`; for modes `0..6`, it selects a three-byte lookup-table entry, while mode `7` copies the selected profile RGB triplet. This establishes profile-dependent matrix construction, but not a semantic name for every matrix byte.

The `0x13b92` routine is reached from the mode/state dispatch stub at `0x14810`. `0x153d4` is a separate state/effect routine reached from a dispatch branch at `0x16336`; it consumes `0x2077c` records and calls `0x164a6` at `0x15398`. There is no direct `BL 0x153d4` inside `0x13b92`; the relationship is shared-state producer/consumer, not a direct call chain. The `0x153d4` path decrements/reloads state in the `0x2077c` header, walks the six-by-21 matrix, derives an update scale using mode/effect state and writes calculated records to `0x20a78` through `0x164a6`.

`0x146f8` is a separate refresh helper. It clears `0x2077c` for `0x2fc` bytes, writes zero-valued records to a 7×15 subset of `0x20a78` through `0x164a6`, then invokes `0x16c64`. It is not evidence that all 168 records in `0x20a78` are freshly rendered by that call.

## 5. Limits and unresolved items

- The 67 direct call sites are exhaustive for direct `BL` references in this disassembly; indirect calls and all pointer-derived aliases are not exhaustively disproven.
- Several callers are effect-specific and require individual argument tracing before assigning semantic names.
- The `0x2077c` intermediate has a three-byte header followed by a six-by-21 set of four-byte records; its complete lifetime, all writers, and interrupt/main-loop sharing remain unresolved.
- Channel labels/order are not established by the record shape alone.
- No source-level C types or reliable load address are available.

## 6. Evidence index

- Writer: `0x164a6–0x164f4`; base literal at `0x168c0`.
- Copy reader: `0x16c64–0x16c94`; source literal `0x16cac`, destination literal `0x16cd0`.
- RMW: `0x16ab2–0x16b24`.
- Profile-to-matrix: `0x13b92–0x13cd6`; state base literal `0x13c28`, matrix base literal `0x14080`.
- Effect consumer: `0x153d4–0x155d8`; matrix literal `0x155b8`.
- Partial refresh/clear: `0x146f8–0x1472e`.
- Zero-fill helper: `0x1d45e` tail-branches to `0x1d450`.
