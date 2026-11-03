# Phase 5C — `wIndex` discrepancy analysis

Date: 2026-11-01  
Scope: static comparison of Phase 5B HID descriptors, native support metadata, and macOS libusb code.

## Verdict

**The code-side value is resolved; the discrepancy's cause and operational correctness are not.**

- **CONFIRMED:** The physical keyboard `3151:4010` and receiver `3151:4011` expose a Feature report on HID interface 2 (Phase 5B live descriptor observation). Each report is 64 bytes, implicit Report ID 0.
- **CONFIRMED:** The macOS generic Feature SET and GET code pass literal `wIndex=0` to `libusb_control_transfer`.
- **CONFIRMED:** The native support records for both devices specify interface number 2; `DJDeviceInfo::from_hid_info` preserves the enumerated interface at offset `+0x28`, and `HidLibusb::new` copies the metadata.
- **CONFIRMED:** The generic libusb constructor opens a matching device by VID/PID but has no `libusb_claim_interface` or `libusb_set_interface_alt_setting` call. `ClaimedInterfaces::new` only initializes an empty tracking collection.
- **UNKNOWN:** Why zero was selected, whether it is intentional for a different interface layout, inherited from a generic implementation, or simply incorrect for these descriptors.
- **UNKNOWN:** Whether the raw RPC reaches these methods in actual operation and whether the host/device accepts the transfer.

## Relevant addresses and instructions

Analyzed Mach-O: `iot_driver`, SHA-256 `22c5dc67ba77e2373a5ffdbb0312ed8c8e73eb16e78037764eb676a030b2724b`, image base `0x100000000`.

### SET

`HidLibusb::send_feature` VA `0x100048290`; call site `0x10004831b`.

```asm
100048306: movl $0x40, (%rsp)      ; wLength 64
10004830b: movl $0x21, %esi        ; bmRequestType
100048310: movl $0x09, %edx        ; bRequest
100048315: movl $0x300, %ecx       ; wValue
100048315: xorl %r8d, %r8d         ; wIndex = 0
100048318: movq %r15, %r9          ; data pointer
10004831b: call _libusb_control_transfer
```

### GET

`HidLibusb::get_feature` VA `0x100048460`; call site `0x10004851b`.

```asm
100048501: movl $0x7d0, 0x8(%rsp)  ; timeout 2000 ms
100048509: movl $0xa1, %esi        ; bmRequestType
10004850e: movl $0x01, %edx        ; bRequest
100048513: movl $0x300, %ecx       ; wValue
100048518: xorl %r8d, %r8d         ; wIndex = 0
10004851b: call _libusb_control_transfer
```

The zero is not a loaded variable, not the result of interface metadata conversion, and not supplied by the RPC. It is an immediate register clear at each call site. **CONFIRMED.**

## Interface-number dataflow

1. `DJDeviceInfo::from_hid_info` at `0x1000c8b00` calls `hidapi::DeviceInfo::interface_number()` and stores the result at `DJDeviceInfo+0x28`.
2. `HidLibusb::new` at `0x100047d30` receives and copies the metadata into its backend state. It enumerates libusb devices and matches VID/PID before opening the matching device.
3. Neither `send_feature` nor `get_feature` reads `DJDeviceInfo+0x28` or the copied interface number when setting up the control transfer.
4. The target support records for `3151:4010` and `3151:4011` specify interface 2, usage 2, usage page `0xffff`. The receiver record additionally carries `dongle_common=true`.

The constructor's device-level open is not an interface selection. No interface claim or alternate-setting selection is made by this generic backend. **CONFIRMED.**

## Does another macOS implementation explain it?

The binary contains a Rust HIDAPI crate and IOKit. The recovered HIDAPI `HidDevice::write` wrapper at `0x1001bf190` calls `_hid_write` at `0x1001bf1af`; the macOS implementation routes this to IOKit Output-report handling. The audited IOKit code uses `IOHIDDeviceSetReport` with report type Output (1). No `IOHIDDeviceGetReport` or Feature-type (`2`) `IOHIDDeviceSetReport` path was found. **CONFIRMED for the inspected binary:** no alternate IOKit Feature path has been identified to reconcile `wIndex`.

`HidLibusb` is therefore the only Feature GET/SET implementation recovered in this binary. This does not prove that every frontend operation reaches it.

## Interpretation matrix

| Explanation | Result | Reason |
|---|---|---|
| Interface 2 is claimed, then Feature transfer uses index 0 | **NOT SUPPORTED** | No claim call; no claim symbol. |
| Interface metadata is discarded during discovery | **REFUTED** | Interface number is stored at `+0x28` and copied into backend metadata. |
| Control-transfer code derives `wIndex` from interface metadata | **REFUTED** | Literal zero at both call sites. |
| A separate IOKit Feature transport accounts for interface 2 | **NOT FOUND** | IOKit path identified is Output; no Feature GET/SET call recovered. |
| Zero is an intentional vendor design for this hardware | **HYPOTHESIS** | No source comment or corroborating interface mapping explains it. |
| Zero is a stale/generic assumption or implementation bug | **HYPOTHESIS** | Fits the mismatch, but static evidence cannot distinguish intent from defect. |
| The raw routes for both target devices reach the generic libusb methods | **STRONGLY SUPPORTED** | Construction/vtable/candidate handler chains are recovered; raw URI → poll body is not unique. |

## Is `wIndex=0` necessarily wrong?

For a HID class request addressed to an interface, the low byte of `wIndex` identifies the interface. Phase 5B's descriptor places the Feature report on interface 2, while the recovered generic requests address interface 0. This is a genuine static inconsistency. It is not valid to rewrite zero to two solely from the descriptor and claim that the vendor intended it; nor is it valid to assume that interface 0 is a synonym for interface 2.

Possible explanations include an incorrect generic implementation, stale support metadata, a device revision whose interface layout differs from the captured units, or an unobserved special transport path. These remain **HYPOTHESES/UNKNOWN**, not findings.

## Final answer

The discrepancy is only partially resolved: the origin of `wIndex=0` is proven (hardcoded literal in both generic libusb methods), and the conflicting interface-2 descriptor/support metadata is proven. The underlying reason and whether the requests are valid for `3151:4010` / `3151:4011` remain unknown. No hardware test was performed, and none is part of Phase 5C.
