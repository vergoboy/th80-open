# Phase 5D — `wIndex=0` resolution

Date: 2026-11-01  
Scope: static reconstruction of the libusb open/control-transfer path and its interface handling. No device I/O.

## Artifact identity

macOS `iot_driver` SHA-256: `22c5dc67ba77e2373a5ffdbb0312ed8c8e73eb16e78037764eb676a030b2724b` (Mach-O x86-64, base `0x100000000`).

## Verdict

- **CONFIRMED:** Both application Feature methods pass literal `wIndex=0` to `_libusb_control_transfer`; each explicitly clears `r8d` immediately before the call. This is a call-site constant, not a libusb wrapper default.
- **CONFIRMED:** The low-level libusb control-transfer routine copies the `wIndex` argument directly into bytes 4–5 of the eight-byte USB setup packet. There is no interface-number rewrite or transformation.
- **CONFIRMED:** `HidLibusb::new` opens a device handle by VID/PID and retains the HID metadata, including interface number, but does not claim an interface or select an alternate setting in the recovered constructor path.
- **CONFIRMED:** The `ClaimedInterfaces::new` called by the constructor only zero-initializes its tracking collection; it does not claim an interface.
- **UNKNOWN:** Why the vendor implementation selected zero, whether it reflects another product/interface arrangement, or whether it is an implementation defect for the captured TH80 units.

## Application call sites

### Feature SET

`HidLibusb::send_feature` VA `0x100048290`; call at `0x10004831b`:

```asm
1000482f7: movl $0x7d0, 0x8(%rsp) ; timeout 2000 ms
1000482ff: movl $0x40, (%rsp)    ; wLength 64
100048306: movl $0x21, %esi      ; bmRequestType
10004830b: movl $0x09, %edx      ; bRequest
100048310: movl $0x300, %ecx     ; wValue
100048315: xorl %r8d, %r8d       ; wIndex = 0
100048318: movq %r15, %r9        ; data pointer
10004831b: call _libusb_control_transfer
```

### Feature GET

`HidLibusb::get_feature` VA `0x100048460`; call at `0x10004851b`:

```asm
1000484b8: BytesMut::resize(64, 0)
100048501: movl $0x7d0, 0x8(%rsp) ; timeout 2000 ms
100048509: movl $0xa1, %esi      ; bmRequestType
10004850e: movl $0x01, %edx      ; bRequest
100048513: movl $0x300, %ecx     ; wValue
100048518: xorl %r8d, %r8d       ; wIndex = 0
10004851b: call _libusb_control_transfer
```

Both are standard HID class/interface requests, Feature report type 3, Report ID 0 (`wValue=0x0300`), data length 64.

## Device discovery and metadata flow

1. `DJDeviceInfo::from_hid_info` at `0x1000c8b00` reads `hidapi::DeviceInfo::interface_number()` and stores it at metadata offset `+0x28`.
2. `HidLibusb::new` at `0x100047d30` enumerates libusb devices, reads device descriptors, matches VID/PID, calls `_libusb_open` at `0x100047e61`, and copies the saved metadata into the sender state.
3. The target support records for keyboard `3151:4010` and receiver `3151:4011` specify interface number 2; Phase 5B captured the 64-byte implicit-ID-zero Feature descriptor on interface 2 for both.
4. Neither Feature method reads the copied interface-number field. Each call site clears `r8d` immediately before `_libusb_control_transfer`.

**CONFIRMED:** Interface metadata survives discovery, but the Feature transfer setup does not consume it.

## Claim/configuration analysis

`HidLibusb::new` invokes `rusb::device_handle::ClaimedInterfaces::new` at `0x100047ec1`. The constructor at `0x1001b1440` writes zero to its four collection fields and returns. No claim call is made there.

The statically linked libusb implementation does contain Darwin interface-management internals, including `_darwin_claim_interface` (`0x1001b7020`), `_darwin_capture_claim_interface` (`0x1001b2710`), `_darwin_set_interface_altsetting` (`0x1001b28a0`) and `_darwin_set_configuration` (`0x1001b2530`). The observed `_darwin_claim_interface` call sites are inside libusb configuration/alternate-setting/state-restore routines, not the application `HidLibusb` constructor or Feature methods. The app does not import/call a public `libusb_claim_interface` symbol, and no configuration/alternate-setting operation was recovered in the target constructor path.

The `_libusb_control_transfer` implementation at `0x1001bdd90` builds an eight-byte setup packet. It stores `bmRequestType` at byte 0, `bRequest` at byte 1, `wValue` at bytes 2–3, `wIndex` from `r8w` at bytes 4–5 (`0x1001bde43`), and `wLength` at bytes 6–7 (`0x1001bde48`). The argument is copied directly. This routine allocates/submits a control transfer; it does not derive `wIndex` from HID metadata.

**CONFIRMED:** There is no app-level interface claim or interface-number mapping in the recovered Feature path. Internal libusb Darwin claim routines exist for other library operations; their presence does not imply that the Feature path claims interface 2.

## Interpretation matrix

| Explanation | Finding |
|---|---|
| `wIndex` is derived from `DeviceInfo::interface_number` | **REFUTED** — literal zero at both call sites |
| `wIndex=0` is a generic control-transfer helper default | **REFUTED** — caller explicitly supplies zero in both Feature methods; wrapper copies it unchanged |
| Interface number is lost during discovery | **REFUTED** — stored at metadata `+0x28` and copied |
| libusb control-transfer wrapper remaps zero | **REFUTED** — setup bytes 4–5 receive the `r8w` argument directly |
| Constructor claims interface 2 before Feature I/O | **REFUTED for the recovered constructor path** — tracking collection starts empty; no claim call |
| Library contains internal Darwin claim support | **CONFIRMED** — used by configuration/alternate-setting/state-restore routines |
| Zero is an intentional product-specific design | **HYPOTHESIS** — no source comment or corroborating mapping |
| Zero is inherited generic behavior or a defect | **HYPOTHESIS** — plausible, but intent cannot be distinguished statically |

## Final classification

The strongest static conclusion is that `wIndex=0` is an explicit, hardcoded application-level implementation choice in both Feature methods—not a generic helper default, a transformed interface number, or a consequence of losing interface metadata. The discrepancy is not explained at the design/intent level: static evidence cannot distinguish an intentional product-specific assumption from a copied assumption or defect. The captured keyboard/receiver Feature descriptors are on interface 2, while the generic implementation addresses interface 0. Do not change the value or claim it is operationally wrong without authorized validation.

No interface claim/detach, control transfer, USB reset, or hardware I/O was performed.
