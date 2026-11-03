# Phase 5C — Feature Report correlation

Date: 2026-11-01  
Scope: static analysis of the EPOMAKER macOS `iot_driver`, its bundled renderer, and the Phase 5B descriptor facts. No hardware I/O.

## Executive result

- **CONFIRMED:** The `HidLibusb` Feature SET and GET functions are generic 64-byte HID class control-transfer wrappers. They pass a 64-byte data buffer and use Report Type Feature / Report ID 0 (`wValue=0x0300`).
- **CONFIRMED:** SET passes the caller's first 64 bytes unchanged at the low-level transport boundary; GET allocates a 64-byte zero-filled buffer and returns the bytes populated by libusb.
- **CONFIRMED:** The low-level libusb methods hardcode `wIndex=0`; they do not derive it from the saved HID interface number.
- **CONFIRMED:** The TH80 Pro and receiver support records both describe interface 2, and Phase 5B independently found the unnumbered 64-byte Feature report on interface 2 for both devices.
- **UNKNOWN:** Why the vendor chose `wIndex=0`, whether it is a legacy assumption or a defect, and whether an operation on either physical device succeeds through this generic implementation.
- **STRONGLY SUPPORTED:** The native candidate raw-RPC SET chain reaches the generic `DJDevice` Feature future, where `dangledevtype` controls checksum insertion. Exact raw URI → poll-function assignment remains unproven.

## Binary provenance

Analyzed binary: `../.phase3-work/mac/EPOMAKER Driver 1.0.0/EPOMAKER Driver.app/Contents/Resources/app/iot_driver`  
SHA-256: `22c5dc67ba77e2373a5ffdbb0312ed8c8e73eb16e78037764eb676a030b2724b`  
Mach-O x86-64, image base used for VM addresses `0x100000000`.

## Feature transport functions

### SET — `HidLibusb::send_feature`

Function VA `0x100048290`; `libusb_control_transfer` call at `0x10004831b`.

Disassembly at `0x1000482e0..0x10004831b`:

```asm
1000482e0: cmp     r12, 0x40           ; require at least 64 bytes
1000482ea: mov     rdi, [rbx+0x10]     ; DeviceHandle
1000482f7: mov     dword ptr [rsp+8], 0x7d0 ; timeout 2000 ms
1000482ff: mov     dword ptr [rsp], 0x40    ; wLength 64
100048306: mov     esi, 0x21           ; bmRequestType
10004830b: mov     edx, 0x09           ; bRequest
100048310: mov     ecx, 0x300          ; wValue
100048315: xor     r8d, r8d            ; wIndex = 0
100048318: mov     r9, r15             ; caller data pointer
10004831b: call    libusb_control_transfer
```

The Rust method returns success for non-negative libusb result and converts a negative result through `rusb::error::from_libusb` before logging. A caller slice shorter than 64 bytes triggers a bounds panic before transfer. A longer slice is truncated to its first 64 bytes by the fixed transfer length. **CONFIRMED.**

| Setup field | Value |
|---|---:|
| `bmRequestType` | `0x21` |
| `bRequest` | `0x09` |
| `wValue` | `0x0300` |
| `wIndex` | `0x0000` |
| `wLength` | `64` |
| timeout | `2000 ms` |

### GET — `HidLibusb::get_feature`

Function VA `0x100048460`; `libusb_control_transfer` call at `0x10004851b`.

The function creates a `BytesMut`, resizes it to 64 bytes with fill value zero (`0x1000484b8..0x1000484e3`), then calls libusb with:

| Setup field | Value |
|---|---:|
| `bmRequestType` | `0xa1` |
| `bRequest` | `0x01` |
| `wValue` | `0x0300` |
| `wIndex` | `0x0000` |
| `wLength` | `64` |
| timeout | `2000 ms` |

Negative transfer results become a one-byte error representation in the returned Rust result. Non-negative results are converted into a `Bytes` view whose length is based on the transfer result. **CONFIRMED.**

## Feature buffer layout

The low-level `HidLibusb` methods do not define a command grammar. They accept/return a byte buffer. The additional framing recovered in the higher `DJDevice::send_feature` future is limited to the following:

| Region / behavior | Evidence grade | What is actually established |
|---|---|---|
| Data stage length | **CONFIRMED** | Exactly 64 bytes submitted by `HidLibusb::send_feature`; GET requests 64 bytes. |
| Report ID | **CONFIRMED** | `wValue=0x0300` selects Feature Report ID 0. The interface-2 descriptor has no Report ID item, so it is an unnumbered report. No separate leading report-ID byte is added by the low-level libusb method. |
| Input buffer prefix | **CONFIRMED** | The higher SET future works on the supplied message bytes and uses `dangledevtype` to choose checksum insertion layout. |
| Checksum for `dangledevtype=0` | **CONFIRMED** | Byte-wise sum of a 7-byte prefix, followed by bitwise NOT; inserted at report offset 7 when metadata selector `+0x3A=2`. |
| Checksum for `dangledevtype=1` | **CONFIRMED** | Byte-wise sum of an 8-byte prefix, followed by bitwise NOT; inserted at report offset 8 when metadata selector `+0x3A=2`. |
| `dangledevtype=2` | **CONFIRMED** | Skips the checksum-generation/insertion branch. |
| Header/opcode semantics | **UNKNOWN** | No universal command/header grammar is established for these raw Feature requests. |
| Length field semantics | **UNKNOWN** | No general in-buffer length field has been proven for this raw path. |
| Sequence/counter | **UNKNOWN** | No general sequence field established. |
| Response semantics | **UNKNOWN** | The low-level GET returns bytes; command-specific interpretation is not resolved here. |
| Checksum algorithm beyond the two recovered selector branches | **UNKNOWN** | Do not extrapolate the two layouts to all possible request/device types. |

The checksum code is in the `DJDevice::send_feature` async poll body at `0x1000959e0`; the selector is read around `0x100095be2`. The native SendMsg-shaped handler normalizes `dangledevtype` at `0x1000aa13a..0x1000aa14b` and passes it to `Base::pure_send_feature` at call sites `0x1000aa799` and `0x1000aa7d7`. The Base future captures it at environment offset `+0x398`; the YZWDangleCommon factory is at `0x1000610d0`.

The frontend's `checksumtype` protobuf field is set by the renderer helper, but no corresponding native consumer has been identified in this Feature call chain. Its effect remains **UNKNOWN**. Do not confuse it with the native `dangledevtype` checksum-layout selector.

## HIDAPI / IOKit alternative-path audit

The Mach-O contains a compiled Rust `hidapi` crate and IOKit HID functions. This does **not** establish a second Feature-report transport:

- `hidapi::HidApi::open_path` at `0x1001beb90` calls `_hid_open_path` at `0x1001beba4` (call target `0x1001c1b70`) and wraps the resulting device handle.
- `hidapi::HidDevice::write` at `0x1001bf190` calls `_hid_write` at `0x1001bf1af` (target `0x1001c2290`). In the macOS HIDAPI backend, the recovered IOKit path is `IOHIDDeviceSetReport` with report type Output (1), not Feature (2).
- The existing binary audit found no `IOHIDDeviceGetReport` call and no Feature-type (`2`) `IOHIDDeviceSetReport` call. Therefore, **CONFIRMED within the inspected binary paths:** HIDAPI/IOKit accounts for enumeration, input and Output-report handling, not an identified alternate Feature GET/SET path.
- The native `HidSender` Feature methods recovered here are the separate libusb implementation at `0x100048290` and `0x100048460`.

## Device-specific transport correlation

Phase 4.2 recovered the support metadata and constructor path:

| Device | Support metadata | Base/API wrapper | Underlying DJDevice transport | Evidence |
|---|---|---|---|---|
| `3151:4010` TH80 Pro | interface 2; tags `(+0x39,+0x3A)=(2,2)` | `BleHid` | `HidLibusb` | **CONFIRMED** construction chain |
| `3151:4011` receiver | interface 2; `dongle_common=true`; tags `(1,2)` | `YZWDangleCommon` | `HidLibusb` | **CONFIRMED** construction chain |

`DJDevice::new` at `0x1000c8ea0` selects `HidLibusb::new` at `0x100047d30` when metadata byte `+0x3A == 2`. The manager wraps the normal keyboard path with `BleHid` and the receiver path with `YZWDangleCommon`. These are API wrappers; `BleHid` does not mean the transport is Bluetooth in this branch.

Thus the two devices share the same generic Feature transport implementation if the candidate raw route reaches it. That route-specific edge remains **STRONGLY SUPPORTED**, as described in `PHASE5C_RPC_DISPATCH_RESOLUTION.md`.

## Conclusion

The 64-byte transfer envelope and the selector-dependent checksum insertion behavior are understood at the level above. The full 64-byte command grammar is not. The `wIndex=0` discrepancy is a real mismatch between the generic libusb implementation and the physical interface-2 Feature descriptor; static analysis explains how the zero is produced, but not why it was chosen or whether it works on the target devices.

No Feature GET/SET, USB transfer, interface claim/detach, reset, updater execution, bootloader operation or firmware operation was performed.
