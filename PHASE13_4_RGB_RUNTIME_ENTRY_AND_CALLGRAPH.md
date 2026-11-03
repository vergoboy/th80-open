# Phase 13.4 — RGB Runtime Entry and Call Graph

## Scope and evidence rules

This report continues Phase 13.3. It treats disassembly addresses as firmware file offsets unless a virtual address is independently established. The firmware is a raw 184,608-byte image; no authoritative load address or complete MCU memory map was recovered. Peripheral-like addresses and callback-table bases are therefore reported as observed operands, not assigned undocumented device identities.

Evidence labels used here are exactly `CONFIRMED_STATIC`, `STRONG_STATIC_INFERENCE`, `WEAK_INFERENCE`, `FRONTEND_ONLY`, and `UNKNOWN`.

## 1. Function `0x103b0`

### Complete body

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

`0x103b0` has no explicit arguments. It invokes `0x1a9b2(1, 0)`, calls `0x135de`, then invokes `0x1a9b2(1, 1)` and returns. Its only direct callee other than the register helper is the previously recovered output-frame routine.

**Classification:** `CONFIRMED_STATIC` for the instruction sequence and direct call to `0x135de`; `STRONG_STATIC_INFERENCE` that this is an output wrapper which brackets frame transmission with a peripheral-control bit change. It is not itself the profile renderer or RGB record generator.

### Meaning of the bracket operation

At `0x1a9b2`:

- `r0` is retained as an index and must be below 9.
- For indices below 8, the helper computes `1 << (4 * index)` and read-modify-writes the word at `0x000f0c40 + 8`. For index 8, it instead sets or clears bit 0 in the word at `0x000f0c40 + 0x0c`.
- `r1 == 0` clears the selected bit; `r1 == 1` sets it.
- For index 1, the affected bit is bit 4 at observed address `0x000f0c48`.

Thus `0x103b0` clears bit 4 at `0x000f0c48`, runs `0x135de`, then sets bit 4 again. The register's peripheral identity and electrical meaning are unknown. This is **not** established as an NVIC interrupt mask: the separately recovered NVIC enable register is at `0xe000e100`.

**Classification:** `CONFIRMED_STATIC` for the bit arithmetic and address operand; `UNKNOWN` for register identity, pin mapping and physical effect.

## 2. Direct reference and caller inventory

A full textual scan of `/tmp/th80-X-thumb.dis` found:

| Target | Direct `BL` callers | Result |
|---|---:|---|
| `0x103b0` | 0 | No direct caller found |
| `0x135de` | 1 | `0x103ba` inside `0x103b0` |
| `0x1034c` | 0 | No direct caller found |
| `0x103c8` | 0 | No direct caller found |
| `0x103d0` | 0 | No direct caller found |
| `0x10418` | 0 | No direct caller found |

Raw firmware searches also found no aligned literal matching the candidate Thumb pointers `0x010103b1` (if image base is `0x01000000`) or `0x010203b1` (if the code at file offset `0x10000` belongs to a segment based at `0x01010000`), nor the unbased candidate forms `0x000103b0` / `0x000103b1`. This excludes a simple embedded absolute pointer reference in the searched image; it does not exclude runtime-populated tables, computed pointers, ROM/bootloader registration or another address mapping.

**Classification:** `CONFIRMED_STATIC` that the recovered disassembly contains no direct `BL` caller and no searched absolute pointer literal; `UNKNOWN` whether runtime dispatch invokes the function.

## 3. Indirect invocation candidates

### 3.1 External-interrupt vector dispatch

The region `0x10000–0x100a0` contains a vector-like dispatch stub:

- `0x1001c`, `0x10020`, …, `0x10098` load successive offsets `0x00, 0x04, …, 0x7c` into `r0` and branch to `0x1009c`.
- `0x1009c` loads a base pointer literal `0x010102bc` and executes `ldr r0, [r0, r1]` followed by `bx r0`.
- The stub at `0x10030` supplies offset `0x14`, corresponding to external interrupt index 5 if the table is the conventional zero-based external-IRQ vector sequence.

The vector-dispatch literal at `0x1013c` is `0x010102bc`. The table contents and any initialization/writer for that observed base address were not recovered. Its mapping to file-backed bytes is not established: under the two simple candidate image bases examined, the corresponding raw-file locations do not provide a validated handler-pointer table. The raw firmware does not contain the candidate pointer `0x010103b1` as a static literal. Consequently, the vector stub proves an indirect handler dispatch mechanism, but does not resolve IRQ 5 (or any other slot) to `0x103b0`.

**Classification:** `CONFIRMED_STATIC` for the indexed indirect dispatch instructions; `STRONG_STATIC_INFERENCE` that this is an external-interrupt handler table; `UNKNOWN` for the handler stored in slot `+0x14` and any relationship to `0x103b0`.

### 3.2 Startup callback-table walkers

Two routines walk 16-byte records and call a function pointer from offset `+0x0c` using `BLX`:

- `0x2acc` loads observed start/end pointer literals `0x010034ec` and `0x0100350c`, reads three argument words, loads the callback at record `+0x0c`, sets its Thumb bit, and invokes it. The routine is called from the early reset path around `0x204`.
- `0x1d488` similarly loads observed bounds `0x0101c8fb` and `0x0101e8fb`, walks records at 16-byte stride, passes the first three words as arguments and calls the fourth word as a function pointer. It is called from `0x10204` during startup.

The address mapping and runtime contents of these regions are not established. Their apparent contents cannot safely be interpreted as ordinary file-backed callback records from the raw image alone. Neither table was resolved to a callback equal to `0x103b0`.

**Classification:** `CONFIRMED_STATIC` for the record-walk and indirect-call mechanics; `UNKNOWN` for the runtime record contents and whether either table contains `0x103b0`.

### 3.3 Other indirect calls

A whole-disassembly scan enumerated 102 `BLX` instruction sites. These include startup record walkers and indirect calls through runtime objects/vtables. Their call-time values are generally loaded from object fields or runtime tables; no recovered field value or registration site resolves one of those calls to `0x103b0`. This is an inventory of indirect-call sites, not proof that every runtime table value is statically recoverable. The USB transfer callback structures described in Phase 13.2 are not evidence for RGB scheduling.

**Classification:** `CONFIRMED_STATIC` that indirect-call mechanisms exist; `UNKNOWN` for any connection to `0x103b0`.

## 4. Nearby function cluster

The functions around `0x1034c` are separate routines, not a direct call chain into `0x103b0`:

- `0x1034c` calls `0x1b216(0)`, writes byte `0x20340+0x0b = 1`, invokes routines `0x12a7c`, `0x12efc`, `0x12fce`, `0x12e08`, `0x13160`, `0x12c7a`, `0x12b9e`, `0x12c02`, `0x108f4`, and `0x111dc`; it increments bytes at `0x20340+0x0c`, `+0x0d`, and `+0x0e` with limits 2/5/5, calls `0x1b216(1)`, and returns. It has no recovered direct caller.
- `0x103c8` writes `1` to byte `0x20340+0x0f` and returns. It has no recovered direct caller.
- `0x103d0` polls timer instance 1 through `0x1a582(1)` and conditionally calls `0x1a5b2(1)` or `0x1a758(1)`. It has no recovered direct caller. Its helpers inspect timer-like registers at `0x000f8c00`, not a demonstrated sleep/wake state.
- `0x10418` performs the analogous status checks for timer selector 0 through `0x1a582(0)`, `0x1a5b2(0)`, and `0x1a974(0)`, then updates a counter/flag pair. It also has no recovered direct caller.

These routines may be runtime callbacks or dormant entry points, but no registration or invocation edge was recovered.

## 5. Confirmed output-side graph

```text
0x103b0
  ├─ 0x1a9b2(1, 0): clear bit 4 at 0x000f0c48
  ├─ 0x135de
  │    ├─ checks byte 0x20348
  │    ├─ prepares/transmits the frame through 0x13526 / 0x134f2
  │    └─ performs state/timing and output-control operations
  └─ 0x1a9b2(1, 1): set bit 4 at 0x000f0c48
```

Phase 13.3 established the data path feeding the consumer: `0x20a78 → 0x16c64 → 0x20d18 → 0x13526`. Phase 13.4 does not establish that the runtime dispatcher invokes `0x103b0`, so the complete runtime edge remains open.

## 6. Call graph summary

```text
Two observed early entry stubs (relationship/ownership unknown):
  0x200
    -> 0x2acc -- indirect calls from runtime 16-byte records (targets unknown)
    -> BX 0x01000899

  0x10200
    -> 0x1d488 -- indirect calls from runtime 16-byte records (targets unknown)
    -> BX 0x0101047f
    -> candidate offset 0x1047e if base=0x01000000
    -> timer config scalar 115200
    -> POP PC from runtime stack (value unknown)

Separate candidate application-init block:
  0x1027e
    -> 0x173cc -- timer/peripheral configuration; enables NVIC IRQ 5
    -> 0x13400 -- candidate output initialization and 0x20a78 clear
    -> main polling loop at 0x10308
  Entry to 0x1027e from either reset stub is unresolved.
                 ├─ 0x1b74c
                 ├─ state dispatch to 0x11328 / 0x114f2 / 0x10b74 / 0x10eee
                 └─ 0x1076c

Candidate interrupt dispatch:
  IRQ vector stub (0x10030 for index 5)
    -> 0x1009c
    -> load handler from [0x010102bc + 0x14]
    -> BX unknown runtime target

Unresolved possible edge:
  runtime IRQ/callback table --?--> 0x103b0

Known output edge:
  0x103b0 -> 0x135de -> 0x13526 -> 0x134f2 / 0x133d2
```

## 7. Conclusions

- `0x103b0` is an output wrapper, not the renderer. `CONFIRMED_STATIC`.
- Its only direct output callee is `0x135de`; that direct edge is `CONFIRMED_STATIC`.
- No direct caller or static absolute pointer to `0x103b0` was found. `CONFIRMED_STATIC` for the searched image/disassembly; not proof of non-execution.
- The firmware has indirect startup callback walkers and an indexed interrupt-dispatch table. Their relevant runtime contents are missing. `CONFIRMED_STATIC` for dispatch mechanics; `UNKNOWN` for target resolution.
- The strongest unresolved edge is the runtime vector-table entry at `0x010102bc + 0x14` (candidate IRQ 5) and whether it resolves to `0x103b0`.

## Related reports

- [Scheduler, timer and initialization](PHASE13_4_SCHEDULER_TIMER_AND_INITIALIZATION.md)
- [Runtime gating](PHASE13_4_RGB_RUNTIME_GATING.md)
- [Findings and next steps](PHASE13_4_FINDINGS_AND_NEXT_STEPS.md)
- [Phase 13.3 LED output analysis](PHASE13_3_LED_OUTPUT_PROTOCOL_ANALYSIS.md)
