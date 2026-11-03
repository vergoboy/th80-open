# Phase 5D — macOS Feature buffer construction and layout

Date: 2026-11-01  
Scope: static trace from the `DJDevice` Feature future to the libusb transport. No Feature operation was executed.

## Artifact identity

macOS `iot_driver` SHA-256: `22c5dc67ba77e2373a5ffdbb0312ed8c8e73eb16e78037764eb676a030b2724b` (Mach-O x86-64, base `0x100000000`).

## Executive result

- **CONFIRMED:** The `DJDevice` SET future at `0x1000959e0` constructs a mutable byte buffer from the supplied message slice, resizes it to at least 66 bytes with zero fill, and may insert a selector-dependent complemented additive checksum.
- **CONFIRMED:** For the target device metadata selector `+0x3A == 2`, `dangledevtype=0` writes the checksum at offset 7 after summing offsets 1–7; `dangledevtype=1` writes it at offset 8 after summing offsets 1–8; `dangledevtype=2` skips checksum insertion. Offset 0 is not included in either checksum loop.
- **CONFIRMED:** The `HidLibusb::send_feature` transport requires at least 64 bytes and sends exactly the first 64 bytes. It does not prepend a Report ID byte.
- **CONFIRMED:** The descriptor and `wValue=0x0300` identify an unnumbered Feature Report ID 0. No separate report-ID byte is inserted by the macOS libusb path. Buffer offset 0 is caller message content, not a framing byte supplied by this driver.
- **UNKNOWN:** General command/header semantics, the meaning of the first message bytes, a universal length/sequence field, and command-specific response interpretation.

## SET construction trace

The relevant generator poll is at `0x1000959e0`.

1. The saved input slice pointer and length are restored from the future environment (`+0x320` and `+0x328` at `0x100095bb9..0x100095bc2`).
2. An empty `BytesMut` is initialized on the stack at `0x100095b9b..0x100095bb3`: pointer/sentinel word `1` at `-0x48`, logical length `0` at `-0x40`, capacity `0` at `-0x38`, and the final representation word `1` at `-0x30`. The first word is the empty/dangling sentinel, not an allocated 1-byte message.
3. `BytesMut::put_slice` appends the caller-provided message bytes (`0x100095bcd`, target `0x1002da1a0`), reserving/growing storage as required. There is no fixed 64-byte allocation at this point, and no leading byte is inserted here.
4. `BytesMut::resize(0x42, 0)` at `0x100095bd6..0x100095bdd` sets the logical length to at least 66 bytes, zero-filling newly added bytes. `resize` does not truncate a longer input.
5. The future reads `dangledevtype` from saved environment byte `+0x332` and device metadata selector `+0x3A` from the captured device object.
6. For selector `+0x3A==2`, `dangledevtype=0` adds the seven bytes at buffer offsets 1 through 7, applies bitwise NOT modulo 256, and stores at offset 7. `dangledevtype=1` adds offsets 1 through 8 and stores at offset 8. `dangledevtype=2` skips checksum insertion. The byte at offset 0 is excluded. The sum includes the pre-store value at the checksum destination when that byte came from the caller slice; if the caller slice did not reach that offset, resize has zero-filled it.
7. It invokes the `HidSender` Feature SET vtable slot at `0x100095cff..0x100095d03` with the constructed buffer.
8. `HidLibusb::send_feature` at `0x100048290` checks the slice length is at least 64 (`0x1000482e0..0x1000482f1`), then passes the pointer and fixed `wLength=64` to libusb. Therefore only bytes `[0..64]` are transferred, even though the higher-level buffer is at least 66 bytes long.

**CONFIRMED:** There is no report-ID insertion in the macOS `DJDevice` construction or low-level libusb SET method. `wValue=0x0300` selects Feature Report ID 0.

## Byte-offset table — target metadata selector `+0x3A == 2`

| Offset | Width | Value/source | Confidence | Evidence |
|---:|---:|---|---|---|
| 0 | 1 byte | Caller message byte 0, unchanged; not a Report ID inserted by the driver | **CONFIRMED** | `put_slice` at `0x100095bcd`; no prefix write in future |
| 1–6 | 6 bytes | Caller message bytes 1–6, unchanged before checksum insertion | **CONFIRMED** | `put_slice`; checksum reads at `0x100095c56..0x100095c67` / `0x100095ca2..0x100095cb3` |
| 7 | 1 byte | For `dangledevtype=0`: `~(buffer[1]+...+buffer[7]) mod 256`; destination byte's pre-store value participates if supplied, otherwise zero-fill | **CONFIRMED** | Sum at `0x100095c9e..0x100095cb6`; store at `0x100095cc7` |
| 8 | 1 byte | For `dangledevtype=1`: `~(buffer[1]+...+buffer[8]) mod 256`; destination byte's pre-store value participates if supplied, otherwise zero-fill | **CONFIRMED** | Sum at `0x100095c52..0x100095c6d`; store at `0x100095ce5` |
| 9–63 | 55 bytes | Remaining caller message bytes and/or zero-fill; no universal meaning established | **CONFIRMED / UNKNOWN semantics** | `resize(66,0)` at `0x100095bd6`; transfer length 64 at `0x1000482ff` |
| 64–65 | 2 bytes | Logical buffer tail; zero-filled if appended by resize; not included in the 64-byte libusb transfer | **CONFIRMED** | resize to minimum 66; low-level fixed `wLength=64` |

`BytesMut::resize(66, 0)` is a minimum-length operation: it does not truncate a longer caller message. Allocation size is therefore input-dependent (the append/reserve path), while the logical length is raised to at least 66. Thus the transport's first 64 bytes are the wire image; bytes 64 onward are never sent by this method. The checksum byte is overwritten in-place after summing the preexisting byte at that offset if it was part of the original caller message.

## Transfer envelope

| Field | Value | Grade |
|---|---:|---|
| Report type | Feature (`wValue` high byte `0x03`) | **CONFIRMED** |
| Report ID | 0 (implicit/un-numbered descriptor) | **CONFIRMED** |
| SET request type | `0x21` | **CONFIRMED** |
| SET request | `0x09` | **CONFIRMED** |
| SET `wValue` | `0x0300` | **CONFIRMED** |
| SET `wIndex` | `0` | **CONFIRMED** |
| SET data length | 64 bytes | **CONFIRMED** |
| Timeout | 2000 ms | **CONFIRMED** |

The caller buffer must be at least 64 bytes. A shorter slice panics before transfer. A longer slice is truncated to the first 64 bytes at the transport boundary. The higher-level `DJDevice` future passes the entire logical `BytesMut` (minimum 66 bytes), so the transport truncation is explicit.

## GET allocation, response and errors

`HidLibusb::get_feature` at `0x100048460` initializes an empty `BytesMut`, then `resize(64, 0)` at `0x1000484d8..0x1000484e3`. It issues GET_REPORT with `bmRequestType=0xa1`, `bRequest=0x01`, `wValue=0x0300`, `wIndex=0`, `wLength=64`, timeout 2000 ms.

- **CONFIRMED:** A negative libusb result is converted with `rusb::error::from_libusb` and represented by a one-byte error result.
- **CONFIRMED:** On any non-negative result, the code converts the original `BytesMut` into `Bytes` with its existing logical length of 64. The transfer return value is tested only for negativity; a positive short-transfer count is not used to shorten the returned byte string. Because the buffer was zero-filled before transfer, bytes not overwritten by a short transfer remain zero.
- **CONFIRMED:** No command-specific response parser runs in `HidLibusb::get_feature`; it returns the 64-byte buffer as opaque bytes.
- **UNKNOWN:** The meaning of individual response bytes, command-specific status fields, or any universal response grammar.

## What is not established

- **UNKNOWN:** Universal command ID/opcode, header layout, in-buffer length semantics, sequence counter, payload schema, and command-specific status meanings.
- **UNKNOWN:** Native consumption of the renderer's protobuf `checksumtype` field. The native checksum branch described here consumes `dangledevtype`; do not conflate them.
- **UNKNOWN:** Whether every frontend raw Feature request reaches this future; exact URI-to-poll mapping is unresolved.

No Feature GET/SET, control transfer, HIDAPI I/O, Output/Interrupt transfer, interface claim/detach, reset, firmware, or bootloader operation was performed.
