# Phase 13.2 — opcode `0x8c` byte-level analysis

**Scope:** static firmware disassembly and frontend source correlation only. No command was sent to a device, and no Feature GET/SET or other hardware operation was performed.

## 1. Dispatch and handler

The recovered upper dispatch table entry for opcode `0x8c` points to a small dispatch stub at `0x19dd6`, which calls the handler body at `0x1951e`. The command buffer pointer is `0x21240` (literal at `0x196f0`). The response descriptor is based at `0x200d8` (literal at `0x196e8`).

Classification: `CONFIRMED_STATIC` for the dispatch-table target and the handler's code-level buffer accesses.

## 2. Byte-level input layout

The handler reads:

| Request byte | Use in handler | Evidence |
|---:|---|---|
| `[0]` | Opcode is selected by dispatcher; not otherwise used in this body | Dispatcher table at `0x19cda` and stub `0x19dd6` |
| `[1]` | First page/bank index; copied to response descriptor byte `+4` | `0x19520–0x19524` |
| `[2]` | Second page/chunk index; copied to response descriptor byte `+5` | `0x19526–0x19528` |

It calculates a source pointer as:

```text
source = 0x01029100 + (request[1] << 8) + (request[2] << 6)
length = 0x40
```

The base constant is loaded from literal pool word at `0x1973c`. The calculated pointer is stored in response descriptor field `0x200e0` (`0x200d8 + 8`), and length `64` is stored at `0x200e4` (`+0x0c`).

The handler clears response descriptor byte `0x200d8`. It sets byte `0x200d9` to `1` only when request byte `[2] <= 4`; values above 4 leave the flag clear. The body returns through `0x1954c`.

Classification: `CONFIRMED_STATIC` for the byte reads, address arithmetic, response pointer/length writes, and the `request[2] <= 4` condition. `STRONG_STATIC_INFERENCE` that this is a paged 64-byte data-read response. The physical meaning of the pointed-to data is not established.

## 3. Source region and address caveat

The literal is `0x01029100`. Under the unconfirmed `0x01000000` image-base hypothesis, this would correspond to file offset `0x29100`. The bytes at raw file offset `0x29100` are zero-filled, but the image-base mapping is not proven and must not be used to assert that the handler reads a zero-filled firmware region. No runtime memory map or flash alias was recovered. The physical storage region and contents remain `UNKNOWN`.

## 4. Frontend correlation and an important non-match

The generic frontend constant table contains `FEA_CMD_GET_USERPIC = 140` (`0x8c`) and `FEA_CMD_SET_USERPIC = 12` (`0x0c`). Some generic model code constructs a 64-byte GET_USERPIC request with byte `[1]=0`, loops byte `[2]` from `0` through `5`, and concatenates each returned 64-byte response.

However, the TH80-specific `YZWCommon` `_getLightPic` override uses literal opcode `0x89`, not `0x8c`. This distinction is documented in `PHASE8_STATIC_EVIDENCE_CLOSURE.md`, `PHASE13_VENDOR_PROTOCOL_RGB_CORRELATION.md`, and `PHASE13_1_RGB_HANDLER_ANALYSIS.md`. The firmware's `0x89` upper-table entry resolves to a return-only epilogue, while the separate `0x8c` handler exists and has the paged-data behavior described above.

Consequently:

- The existence of a generic frontend `0x8c` GET_USERPIC constant is `FRONTEND_ONLY` evidence.
- The six-chunk frontend loop is `FRONTEND_ONLY` evidence for generic model behavior.
- The firmware handler's five-valid-index condition (`[2] <= 4`) differs from the generic frontend's loop through index 5. This is a byte-level compatibility discrepancy, not proof of which side is wrong.
- It is not justified to rename the TH80 firmware `0x8c` handler as the TH80 user-picture getter, keymap getter, or per-key RGB reader.

Classification: `CONFIRMED_STATIC` for the frontend constant and the TH80-specific literal `0x89`; `STRONG_STATIC_INFERENCE` for paged-data response behavior; `UNKNOWN` for semantic identity and compatibility with any specific model's user-picture API.

## 5. Relationship to other handlers

- `0x08`/`0x87` are the complete profile-indexed lighting write/read pair (profile selector plus mode/speed/value/RGB).
- `0x07`/`0x86` are a separate compact parameter pair.
- `0x89` is a separate command-table entry that resolves to a return-only epilogue in the recovered dispatcher.
- No shared parser or data structure connecting `0x8c` with `0x08`, `0x87`, or `0x89` was established.

The pointer arithmetic is compatible with a generic 64-byte page/chunk reader, but no source-level structure or firmware flash map identifies the data.

## 6. Next useful static investigation

The most valuable follow-up is to identify the memory mapping and initialization/ownership of the region addressed by `0x01029100`, then trace any other handlers that write or select that region. This must be done through static references only. Do not test `0x8c` against the keyboard or receiver.
