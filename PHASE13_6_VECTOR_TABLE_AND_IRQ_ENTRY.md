# Phase 13.6 — Vector Table and IRQ Entry Resolution

## Scope and conclusion

This report resolves the **firmware-side vector and secondary-dispatch path**. It does not claim that the supplied image proves the SoC's architectural vector-fetch base or that the exact keyboard PCB contains the YC3121-L candidate MCU.

The firmware contains two code-stub vector banks: a boot-stage bank at candidate runtime address `0x01000200` (file offset `0x0000`) and an application bank at `0x01010200` (file offset `0x10000`). In application mode, the boot-stage external-IRQ stubs forward to the application dispatcher at `0x0101029c`; the application stubs also enter that dispatcher directly. The dispatcher indexes the 32-word application handler table at `0x010102bc` and tail-branches to the selected pointer.

For external interrupt index 14, the stub supplies byte offset `0x38`, the dispatcher loads table entry 14, and that entry contains raw pointer `0x010105b1`, which maps to `0x103b0` under the candidate image mapping. The firmware-side route is therefore statically reconstructed. The remaining hardware dependency is how the SoC/boot ROM selects or aliases the first vector bank for actual exception entry; no standard Cortex-M VTOR/vector-table setup is present in this image.

## Confidence vocabulary

The labels below use the required exact classifications: `CONFIRMED_STATIC`, `STRONG_STATIC_INFERENCE`, `WEAK_INFERENCE`, `FRONTEND_ONLY`, and `UNKNOWN`.

## 1. Image mapping and independent pointer checks

Candidate image mapping:

```text
candidate runtime address = 0x01000200 + firmware file offset
```

This mapping is not derived from a linker map. It is strongly supported by independent absolute pointers:

| Purpose | File offset containing pointer | Raw pointer | Normalized target | Candidate file offset |
|---|---:|---:|---:|---:|
| Boot reset entry | `0x0198` | `0x01000401` | `0x01000400` | `0x0200` |
| Application vector-bank base | `0x00f4` | `0x01010201` | `0x01010200` | `0x10000` |
| Application reset entry | `0x10128` | `0x01010401` | `0x01010400` | `0x10200` |
| Application IRQ table base | `0x1013c` | `0x010102bc` | data address, no Thumb normalization | `0x100bc` |
| Application IRQ14 entry | `0x100f4` | `0x010105b1` | `0x010105b0` | `0x103b0` |

The odd function-pointer values are consistent with Thumb-state pointers in the candidate decode. Bit zero is retained in the stored pointer and is cleared only when describing the aligned instruction address. The dispatcher itself branches using the unmodified stored value.

**Classification:** `STRONG_STATIC_INFERENCE` for the global image mapping; `CONFIRMED_STATIC` for the raw pointer bytes and their arithmetic. The exact CPU ISA and exact board MCU remain unconfirmed. Earlier project evidence identifies YC3121-L only as a TH80-family lead, not as a verified marking on this exact board.

## 2. Two code-stub vector banks

The bytes at the beginning of the image are executable Thumb-like stubs, not a conventional Cortex-M vector table made of an initial MSP word followed by exception-handler pointers.

### 2.1 Boot-stage bank

Candidate runtime base `0x01000200`, file offset `0x0000`.

- Offset `0x0000`: `ldr r0, [pc, #...] ; bx r0`; its literal at file `0x0198` is `0x01000401`, targeting reset/startup code at file `0x0200`.
- Offsets `0x0004`, `0x000a`, `0x0010`, `0x0016`: four special-vector stubs. They load `r0` with offsets `0`, `6`, `12`, `18` and branch to common routine `0x00b4`.
- Offsets `0x001c` through `0x0098`: 32 consecutive four-byte external-interrupt stubs. Each loads `r0` with `0, 4, 8, ... 0x7c` and branches to `0x00ce`.
- The index-14 stub is at file offset `0x0054`:

```asm
0054: 2038    movs r0, #0x38
0056: e03a    b    0x00ce
```

The four special entries use six-byte spacing; the 32 external entries use four-byte spacing. This is a code-vector layout, not a Cortex-M architectural word-vector layout.

### 2.2 Application bank

Candidate runtime base `0x01010200`, file offset `0x10000`.

- `0x10000`: direct reset stub; its literal at file `0x10128` is `0x01010401`, targeting application reset code at file `0x10200`.
- `0x10004`, `0x1000a`, `0x10010`, `0x10016`: four special-vector stubs, each loading a pointer and branching through it.
- `0x1001c` through `0x10098`: 32 external-interrupt stubs, each supplying an offset `0..0x7c` and branching to `0x1009c`.
- Application IRQ14 stub, file offset `0x10054`:

```asm
10054: 2038    movs r0, #0x38
10056: e021    b    0x1009c
```

**Classification:** `CONFIRMED_STATIC` that both code-stub banks and their immediate values exist. `STRONG_STATIC_INFERENCE` that they are the boot/application vector-entry banks associated with the NVIC-compatible interrupt interface. The SoC's actual vector-fetch base/remap is not stated by the firmware.

## 3. Boot-to-application handoff and mode-dependent forwarding

The boot-stage dispatcher uses a memory-mapped word at `0x000f8564` as a mode selector. Its silicon meaning is not identified in the available product brief.

At file offset `0x00f8`:

```asm
00f8: ldr  r0, [pc, ...]    ; literal address 0x010002f0
00fa: ldr  r0, [r0]        ; -> 0x000f8564
00fc: movs r1, #1
00fe: str  r1, [r0]        ; mode = 1
0100: ldr  r0, [pc, ...]  ; literal address 0x010002f4
0102: ldr  r0, [r0]        ; -> 0x01010201
0104: bx   r0              ; enter application vector-bank base
```

The boot reset stub at file `0x0200` sets SP to `0x00030000`, calls startup walker `0x2acc`, and branches to candidate file offset `0x0698`. The boot-stage path at `0x0710` calls the handoff routine at `0x00f8` on its selected branch. The handoff writes mode value 1 and branches to the application vector-bank base `0x01010200` (Thumb bit set in the literal).

The code at file `0x0108` writes value 2 to the same `0x000f8564` register and returns. No direct branch/call reference to this small routine was found; its runtime reachability is unresolved.

### 3.1 Special-vector forwarding

Routine `0x00b4` reads `0x000f8564` and compares it with 2:

- If equal to 2, it adds the supplied six-byte-spaced offset to local special-vector base pointer `0x0100029d` and branches there.
- Otherwise, it loads application vector-bank pointer `0x01010201`, adds 4 plus the supplied offset, and branches to the corresponding application special-vector stub.

### 3.2 External-IRQ forwarding

Routine `0x00ce` reads the same mode register and compares it with 2:

- If equal to 2, it loads a handler pointer from the separate boot-mode table at runtime `0x0100032c` plus the supplied byte offset, then branches through it.
- Otherwise, it loads application vector-bank pointer `0x01010201`, adds `0x9c`, and branches to application dispatcher `0x0101029c` (file offset `0x1009c`). The supplied `r0` offset is preserved.

Thus the mode-1 handoff path does not require the boot-stage stub to rewrite the vector table: it forwards the external interrupt directly into the application dispatcher. The boot-mode table is separate from the application table; its entry 14 is the default trap pointer `0x0100032b`, not the RGB wrapper.

**Classification:** `CONFIRMED_STATIC` for the writes, comparisons, branch destinations, and separate boot/application tables. `STRONG_STATIC_INFERENCE` that mode 1 is application dispatch mode and mode 2 is boot dispatch mode. The hardware meaning of `0x000f8564` remains `UNKNOWN`.

## 4. Application secondary dispatcher and index calculation

The application dispatcher at file offset `0x1009c` is:

```asm
1009c: 4927    ldr r1, [pc, #0x9c]   ; literal at 0x1013c
1009e: 5840    ldr r0, [r0, r1]      ; word at table_base + byte_offset
100a0: 4700    bx  r0                ; tail-branch to handler
```

The literal at file offset `0x1013c` is raw `0x010102bc`, which maps to file offset `0x100bc` under the candidate image base. The 32 entries are four-byte words. The vector stubs supply a byte offset equal to `external_irq_index * 4`; therefore:

```text
table_index = supplied_r0_offset / 4
IRQ14: r0 = 0x38; 0x38 / 4 = 14
```

There is no `IPSR` read, no subtraction of 16, no runtime bounds check, no pointer validation and no `BLX`. The dispatcher performs an indexed load and a tail `BX`. The finite set of 32 stubs constrains normal vector-originated offsets to `0..0x7c`.

The application table entry at index 14 is:

```text
file offset 0x100f4
raw word    0x010105b1
Thumb bit   1
normalized  0x010105b0
file target 0x103b0
```

This is an exact function boundary in the candidate disassembly.

**Classification:** `CONFIRMED_STATIC` for the table lookup and raw slot-14 value under the candidate mapping. `CONFIRMED_STATIC` that a call path entering stub 14 reaches `0x103b0` through this dispatcher. The hardware-to-stub fetch remains conditional on the SoC vector mechanism.

## 5. NVIC-compatible register operations

The helper at `0x1d304` computes `1 << r0` and sets that bit at `0xe000e100`. The helper at `0x1d312` writes `1 << r0` to `0xe000e180`. The helper at `0x1d2b8` writes priority fields using the NVIC/SCB-compatible priority address ranges. Application initialization supplies priority indices 0, 5, 14 and -1; enable callers include 5, 14 and 15; disable callers include 14.

The candidate YC3121-L product datasheet lists `SYSTICK_BASEADDR = 0xe000e010` and `NVIC_BASEADDR = 0xe000e100` (memory/peripheral map, page 11). That supports the NVIC-compatible interpretation, but YC3121-L is not proven to be the exact MCU on this board.

No literal/reference to `0xe000ed08` (Cortex-M VTOR) or `0xe000e280` (NVIC ICPR0) was found. No `MRS IPSR` instruction was found. The firmware therefore does not itself reveal a conventional VTOR write or active-exception-number conversion. This absence does not exclude a fixed vector base, alias/remap, or boot-ROM vector trampoline.

## 6. Architectural distinction and exact blocker

The supplied image does **not** contain a conventional Cortex-M architectural vector table: there is no initial stack-pointer word followed by the standard 16 core-exception entries and external IRQ words. Instead, it contains executable stubs with five leading reset/special-vector entries and 32 external-interrupt stubs. The candidate YC3121-L brief describes a 32-bit RISC core and an NVIC register block, but does not establish a Cortex-M core or document the hardware vector-fetch format. Earlier Phase 13 evidence also explicitly leaves the exact ISA and exact board MCU unconfirmed.

The firmware-side route is recovered, including the boot/application forwarding condition. The exact remaining dependency is the SoC/boot-ROM mechanism that causes an enabled external interrupt to enter the boot-stage vector bank at candidate address `0x01000200` (or selects the application bank at `0x01010200`). No vector-base register write or hardware alias configuration is present in the supplied image. A boot-ROM image, exact silicon programming manual, or authoritative vector-remap documentation would close this final hardware-entry edge.

## Related reports

- [IRQ14 peripheral source](PHASE13_6_IRQ14_PERIPHERAL_SOURCE.md)
- [IRQ14 lifecycle and RGB trigger](PHASE13_6_IRQ14_LIFECYCLE_AND_RGB_TRIGGER.md)
- [Findings and next steps](PHASE13_6_FINDINGS_AND_NEXT_STEPS.md)
- [Phase 13.5 handler table layout](PHASE13_5_HANDLER_TABLE_LAYOUT_AND_INITIALIZATION.md)
- [Phase 13.5 interrupt dispatch](PHASE13_5_INTERRUPT_DISPATCH_AND_CALLBACK_RESOLUTION.md)
- [Phase 13 firmware mapping and ISA caveats](PHASE13_FIRMWARE_RGB_ANALYSIS.md)

## External reference

- [Yichip YC3121-L datasheet (third-party hosted PDF)](https://atta.szlcsc.com/upload/public/pdf/source/20211111/30894FF75D776498D9212EBEB2236820.pdf) — peripheral map and NVIC base, page 11; timer description and registers, pages 52–55. The datasheet is used conditionally because exact board MCU identity is not confirmed.
