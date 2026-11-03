# Phase 5D — Windows V504 updater Feature buffer recovery

Date: 2026-11-01  
Scope: static analysis of the supplied Windows PE and its Feature-report wrapper/caller. The updater was not executed.

## Artifact identity

- Path: `../TH80_ANSI_Firmware_V504/TH80 ANSI Firmware V504(ID 742)/TH80 Upgrade Firmware for v5xx.exe`
- SHA-256: `e9f3d8d2f3160be5e3817fb4eca8efe954cc854426351e3d0d21e73f39fce0fc`
- PE32+ x86-64; image base `0x140000000`.
- CodeView PDB path: `C:\Users\Windows\Documents\rust-hid\target\release\deps\rust_hid.pdb`; GUID `{C215AEE3-87E2-43DB-B791-B1A9DD29DCEB}`, age 55. No matching PDB is present in the supplied workspace.

## Executive result

- **CONFIRMED:** The wrapper at `0x140615b30` dynamically calls the resolved `HidD_SetFeature` pointer with the caller's handle, data pointer and length. It does not construct or modify the buffer.
- **CONFIRMED:** The only direct caller of the wrapper is the method at `0x14022b510`, called from the device Feature future at `0x14013d4ff`.
- **CONFIRMED:** The caller passes a Rust Vec buffer whose first byte is `0x01`, appends the source payload slice, then resizes to at least 65 bytes with zero fill.
- **CONFIRMED:** If the source payload length is N bytes, the Vec length passed to `HidD_SetFeature` is `max(65, N + 1)`. For N ≤ 64, the length is exactly 65 bytes; for N > 64, the longer prefixed payload is retained. The code does not truncate a longer source payload.
- **CONFIRMED:** The checksum is the complemented 8-bit additive sum over the entire dynamic source payload slice, inserted at offset 8 or 9 according to the device selector byte. This differs from the macOS target branch, whose checksum loops cover fixed 7-byte or 8-byte windows beginning at message offset 1 (offset 0 is excluded).
- **STRONGLY SUPPORTED:** The caller is the generic Rust HID device Feature-send future (the `rust-hid` `DJDevice` layer), not an updater-specific buffer-construction routine. The binary contains a YZW firmware-upgrade subsystem, but the direct call graph recovered here does not prove that this particular Feature send is invoked by the V504 firmware-update flow.
- **UNKNOWN:** Whether the leading `0x01` is the actual Report ID for the target descriptor or a generic prefix used by a distinct device/report mode. No matching Windows report descriptor for the exact runtime device was available to prove its semantics.

## HID API wrapper and caller

| Item | VA | Evidence |
|---|---:|---|
| Runtime resolver | `0x140615490` | Resolves `HidD_SetFeature` from `hid.dll` |
| Function pointer slot | `0x140970810` | Called indirectly by wrapper |
| `hid_send_feature_report` wrapper | `0x140615b30` | Calls pointer at `0x140615b43`; forwards buffer and length |
| Generic HID method | `0x14022b510` | Calls wrapper at `0x14022b52e` |
| Only direct caller of `0x14022b510` | `0x14013d4ff` | Device Feature future call site |

At `0x14022b510`, Windows x64 argument forwarding is explicit:

```asm
14022b51f: movq %r9, %rdi       ; preserve requested length
14022b522: movq (%rdx), %rbx     ; load device handle
14022b525: movq %rbx, %rcx       ; HidD_SetFeature handle
14022b528: movq %r8, %rdx        ; data pointer
14022b52b: movq %r9, %r8         ; byte length
14022b52e: call 0x140615b30
```

The wrapper at `0x140615b30` loads the raw handle from the device wrapper and calls the dynamically resolved function pointer with the same data pointer and length. The wrapper returns the supplied length on success or `-1` on failure. Its direct caller compares the returned byte count with the requested length.

## Caller buffer construction

The caller function at `0x14013d137` constructs the Vec in the state object at `rbx+0x80`:

1. It allocates/initializes the Vec backing storage; its pointer, length and capacity are stored at `rbx+0x80`, `rbx+0x88`, and `rbx+0x98` (`0x14013d164..0x14013d17f`).
2. It sets a one-byte prefix at `rbx+0x48` to `1` (`0x14013d137..0x14013d148`).
3. It appends that byte to the Vec using the helper at `0x1405c7ef0` with length 1 (`0x14013d18a..0x14013d19b`).
4. It appends the source slice whose pointer is device-state `+0x2A0` and length is `+0x2A8` (`0x14013d1a0..0x14013d1b9`).
5. It calls the Vec resize helper at `0x1405c7a00` with target length `0x41` (65) and fill byte zero (`0x14013d1be..0x14013d1cd`). The helper reserves capacity if necessary, zero-fills the newly exposed region, and sets the Vec length to the requested target. As with Rust `Vec::resize`, this does not truncate a buffer already longer than 65 bytes.
6. It reads the device selector at `+0x2B2` and computes/inserts the checksum.
7. It passes the Vec pointer and length to the generic HID method at `0x14022b510` (`0x14013d4e6..0x14013d4ff`).

## Byte-offset table

| Offset | Width | Value/source | Confidence | Evidence |
|---:|---:|---|---|---|
| 0 | 1 byte | Literal `0x01` prefix | **CONFIRMED** | `0x14013d137..0x14013d19b` |
| 1 onward | Dynamic | Caller-provided payload slice `[device+0x2A0, device+0x2A8)` | **CONFIRMED** | `0x14013d1a0..0x14013d1b9` |
| 8 | 1 byte | For selector branch 0: `~sum(payload bytes) mod 256` | **CONFIRMED** | `0x14013d20f..0x14013d23d`, checksum/store `0x14013d47c..0x14013d49b` |
| 9 | 1 byte | For selector branch 1: `~sum(payload bytes) mod 256` | **CONFIRMED** | `0x14013d1e3..0x14013d205`, checksum/store `0x14013d37c..0x14013d39b` |
| Remaining bytes through 64 | Variable | Source payload followed by zero fill to minimum length 65; longer payloads are retained | **CONFIRMED** | Vec append and resize calls above |

The checksum loops perform byte-wise addition modulo 256 and `notb` before storing. Both selector branches iterate over the dynamic source payload slice (pointer `device+0x2A0`, length `device+0x2A8`), not a statically fixed 7- or 8-byte prefix. Selector 0 stores at buffer offset 8; selector 1 stores at offset 9. The selector is loaded from the device state byte `+0x2B2` at `0x14013d1d6`.

The literal prefix `0x01` occupies the first byte of the buffer supplied to the Windows HID Feature API. **HYPOTHESIS:** This is the Report ID byte, as expected by the Windows HID Feature API contract. The exact descriptor/report-ID semantics for the runtime device are not established by this PE disassembly alone.

## Operation classification

The wrapper is a generic HIDAPI-compatible `hid_send_feature_report` function. Its only direct caller is a generic device Feature future that also reads a per-device selector and inserts the same style of complemented additive checksum found in the macOS `DJDevice` Feature future. The executable separately contains `src/data_source/yzw_upgrade.rs`, `src/egui_ui/yzw_upgrade.rs`, `YZWBoot` and firmware-upgrade strings.

- **STRONGLY SUPPORTED:** This is the generic `DJDevice` Feature-send path in the `rust-hid` application.
- **UNKNOWN:** Whether the V504 upgrade flow invokes this exact future for firmware chunks, configuration messages, or another operation. No unique direct edge from the YZW upgrade routine to `0x14013d4ff` was proven.
- **CONFIRMED:** The normal-mode updater support table includes `3151:4010` on interface 2 and bootloader PID `3151:4001`; it does not list receiver PID `3151:4011` in the prior Phase 5C inventory.

## Correlation with macOS

There is a strong structural relationship, but not a proven byte-for-byte packet equivalence:

- macOS builds from the message slice without a leading byte; Windows prepends `0x01`.
- macOS target-device checksum offsets are 7 (`dangledevtype=0`) and 8 (`dangledevtype=1`); Windows offsets are 8 and 9, exactly one byte later. The corresponding payload-relative destinations therefore align because Windows prepends one byte.
- macOS logical buffer is resized to at least 66 bytes but the libusb transport transmits the first 64 bytes. Windows logical buffer is resized to at least 65 bytes and the Windows HID Feature wrapper passes the full Vec length.
- The one-byte checksum-offset difference is consistent with the Windows leading prefix, but the checksum domains are not identical in the recovered code: macOS sums buffer offsets 1–7 or 1–8 (excluding buffer offset 0), while Windows sums the complete dynamic source payload slice (which begins at Windows buffer offset 1). The Windows prefix report-ID meaning and matching target descriptor are not proven.

**STRONGLY SUPPORTED:** Both binaries implement selector-dependent complemented additive checksums and place the checksum at corresponding payload-relative positions after accounting for the Windows leading byte. However, their recovered checksum input ranges differ: macOS sums a fixed 7/8-byte window beginning at message offset 1, whereas Windows sums the complete dynamic payload beginning at its buffer offset 1. The macOS checksum window includes the destination byte before overwriting it when the original message is long enough.  
**UNKNOWN:** Whether the Windows prefix is a Report ID, whether the same target report/firmware operation is involved, and whether all transmitted bytes match the macOS 64-byte envelope.

## Final result

The Windows `HidD_SetFeature` caller buffer is statically recovered as a dynamic payload preceded by `0x01`, padded to at least 65 bytes, with a complemented additive checksum over the complete source payload at offset 8 or 9. Its exact length is `max(65, payload_length + 1)`. The direct caller is a generic device Feature-send path; its specific association with V504 firmware upgrade remains unproven. The macOS/Windows checksum structures correlate strongly, but exact packet equivalence is not confirmed.

The updater was not executed. No HID/USB device was opened and no Feature or firmware operation was performed.
