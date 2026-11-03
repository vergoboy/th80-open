# Phase 4.1 — libusb interface resolution

**Analysis date:** 2026-11-01  
**Artifact:** EPOMAKER Driver 1.0.0 macOS `iot_driver` Mach-O x86-64  
**Scope:** static analysis of device opening, interface claims and Feature control transfers.

## Verdict

- **CONFIRMED:** `DJDeviceInfo` retains the enumerated HID interface number at offset `0x28`.
- **CONFIRMED:** `HidLibusb::new` opens a matching libusb device by VID/PID and retains the copied device metadata.
- **CONFIRMED:** the libusb Feature GET and SET implementations hardcode `wIndex=0` and use 64-byte transfers.
- **CONFIRMED:** this Mach-O contains no `libusb_claim_interface` or `libusb_set_interface_alt_setting` symbol/call implementation. The `rusb::ClaimedInterfaces::new` call only initializes an empty tracking collection; it does not claim an interface.
- **UNKNOWN:** whether the physical TH80 Pro's actual raw Feature RPC reaches these generic Feature methods.

## 1. Interface metadata

`DJDeviceInfo::from_hid_info` at `0x1000c8b00` reads `hidapi::DeviceInfo::interface_number()` and stores the result at offset `0x28`.

`HidLibusb::new` at `0x100047d30` copies this metadata into the `HidLibusb` object. The interface number therefore survives discovery and backend construction. This does **not** imply that every later operation uses it.

## 2. libusb opening

**CONFIRMED:** `HidLibusb::new` enumerates libusb devices, compares descriptor VID and PID with the stored device identity, opens the matching `rusb::Device` and stores the resulting handle. The open path is device-level; the VID/PID match does not select a USB interface number.

The constructor also calls `rusb::device_handle::ClaimedInterfaces::new` at `0x1001b1440` (call site `0x100047ec1`). The implementation of `ClaimedInterfaces::new` only initializes its four-word collection state to zero. It does not call into libusb and does not claim interface 2.

## 3. Interface claim and alternate setting

The complete native symbol table was searched for the libusb interface operations. It contains:

- `_libusb_open` at `0x1001b9db0`;
- `_libusb_release_interface` at `0x1001ba290`;
- `_libusb_control_transfer` at `0x1001bdd90`.

It does **not** contain:

- `_libusb_claim_interface`;
- `_libusb_set_interface_alt_setting`.

No alternate claim/alt-setting implementation was found in the native binary. `ClaimedInterfaces::new` is an empty collection initializer, not a claim call.

Therefore:

**CONFIRMED:** the generic `HidLibusb` constructor does not claim interface 2.  
**CONFIRMED:** the inspected native binary has no libusb claim-interface or set-alt-setting call available to the `HidLibusb` path.

The presence of `_libusb_release_interface` alone is not evidence of a claim. It is a release function in the linked libusb implementation; the corresponding claim entry point is absent.

## 4. Feature SET

Function: `HidLibusb::send_feature`  
Address: `0x100048290`  
Transport: `libusb_control_transfer` at call site `0x10004831b`.

| Setup field | Value | Meaning |
|---|---:|---|
| `bmRequestType` | `0x21` | Host-to-device, class, interface recipient |
| `bRequest` | `0x09` | HID SET_REPORT |
| `wValue` | `0x0300` | Feature report type, report ID 0 |
| `wIndex` | `0x0000` | Interface index 0 |
| `wLength` | `0x0040` | 64 bytes |
| timeout | `0x07d0` | 2000 ms |

The function rejects buffers shorter than 64 bytes and submits the first 64 bytes. `wIndex` is an immediate constant; it is not loaded from `DJDeviceInfo[0x28]`.

## 5. Feature GET

Function: `HidLibusb::get_feature`  
Address: `0x100048460`  
Transport: `libusb_control_transfer` at call site `0x10004851b`.

| Setup field | Value | Meaning |
|---|---:|---|
| `bmRequestType` | `0xa1` | Device-to-host, class, interface recipient |
| `bRequest` | `0x01` | HID GET_REPORT |
| `wValue` | `0x0300` | Feature report type, report ID 0 |
| `wIndex` | `0x0000` | Interface index 0 |
| `wLength` | `0x0040` | 64 bytes |
| timeout | `0x07d0` | 2000 ms |

The method allocates a 64-byte read buffer and converts the control-transfer result into a byte buffer or rusb error. Again, `wIndex` is a literal zero, not a value derived from the preserved interface number.

## 6. Interface discrepancy matrix

| Question | Static result | Evidence |
|---|---|---|
| Is physical interface metadata preserved? | **CONFIRMED — yes** | `DJDeviceInfo` offset `0x28`; copied by `HidLibusb::new` |
| Does `HidLibusb::new` select an interface by number? | **CONFIRMED — no** | libusb enumeration match is VID/PID; open is device-level |
| Does it claim interface 2? | **CONFIRMED — no claim call found** | no `_libusb_claim_interface`; `ClaimedInterfaces::new` zero-initializes only |
| Does it set an alternate interface setting? | **CONFIRMED — no call found** | no `_libusb_set_interface_alt_setting` |
| Does Feature SET use interface 2? | **CONFIRMED — no** | hardcoded `wIndex=0` |
| Does Feature GET use interface 2? | **CONFIRMED — no** | hardcoded `wIndex=0` |
| Does a real TH80 RPC reach these methods? | **UNKNOWN** | raw service-to-device-to-HidSender dispatch edge unresolved |

## 7. Which explanation fits A–E?

- **A. Claims interface 2 but sends Feature transfers with `wIndex=0`: NOT SUPPORTED.** No interface claim call exists in the inspected native binary.
- **B. Opens a different interface or device handle: PARTLY REFUTED.** The constructor opens a device handle by VID/PID, not a handle selected by interface number. No evidence of an alternate interface-specific handle was found.
- **C. Uses a separate path for Feature reports: UNKNOWN.** The generic Feature methods are direct libusb control transfers, but their reachability from TH80 RPC is unresolved. IOKit Output report handling is a separate path and does not prove a Feature path.
- **D. Never reaches this generic implementation for the TH80: UNKNOWN.** The static call chain is not complete enough to prove non-reachability.
- **E. Remains unresolved: YES, for TH80 RPC reachability.** The generic implementation's interface behavior itself is resolved: it opens the device, does not claim interface 2, and sends Feature control transfers with `wIndex=0`.

## 8. Why is `wIndex=0`?

**CONFIRMED:** because both Feature methods pass a hardcoded zero to `libusb_control_transfer`. The implementation does not derive the field from the preserved interface number.

**UNKNOWN:** the developer's reason for choosing zero, and whether this was intended for another device/interface layout. No source comment, configuration mapping or alternate code path in the inspected function explains the choice.

This is not a reinterpretation of interface 0 as interface 2. USB control-transfer `wIndex=0` remains interface index zero.

## 9. Safety

This report is based exclusively on static artifacts. No Feature GET/SET, Output report, control transfer, interface claim, reset, bootloader operation, firmware write or physical-device probe was performed.
