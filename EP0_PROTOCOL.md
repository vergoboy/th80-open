# EP0 Feature Report protocol investigation

## Status and safety

This document records static evidence from the supplied EPOMAKER macOS driver and Electron application, the supplied Windows updater configuration, and physical USB descriptors read on Linux. It does **not** document a live vendor-command exchange. No vendor Feature report was read or written, no undocumented command was sent, and no firmware or mode transition was attempted.

Evidence labels:

- **[Physical observation]** Read from the connected keyboard/receiver, USB descriptors, sysfs, or a read-only host operation.
- **[Static binary]** Recovered from supplied executable, application JavaScript, or embedded configuration without executing a vendor command against hardware.
- **[Vendor documentation]** Stated in the supplied updater guide/package.
- **[Hypothesis]** A plausible interpretation requiring independent confirmation.
- **[Unknown]** Not established by available evidence.

### Inspected macOS artifacts

| Artifact | SHA-256 |
|---|---|
| `EPOMAKER_Driver(MAC2023021).dmg` | `d2db8fdf1cd3afa35c2c4035148187a5a009e0860107cb61371a268ef1091c48` |
| `EPOMAKER_Driver(MAC2023021).zip` | `ae5106ea1e80457507755246717ad4311ddd8e91df8c57e85a95972cb1d710f0` |
| Extracted `iot_driver` Mach-O | `22c5dc67ba77e2373a5ffdbb0312ed8c8e73eb16e78037764eb676a030b2724b` |

The ZIP contains one DMG; hashing the DMG stream inside the ZIP produced the same SHA-256 as the separately supplied DMG. The application bundle metadata identifies version `1.0.0`; `iot_driver` is Mach-O x86-64.

## 1. Device and report context

**[Physical observation]** The TH80 Pro is `3151:4010`; the ROYUAN 2.4 GHz receiver is `3151:4011`. Each currently exposes three HID interfaces. Interface 2 has Usage Page `0xffff`, Usage `0x02`, and one unnumbered Feature report containing 64 data bytes (512 bits). Its descriptor has no Input or Output report item. The interface also advertises an 8-byte interrupt-IN endpoint `0x83`, but that endpoint is not described by an Input report item.

The physical endpoint and report channels must not be conflated:

| Channel | Descriptor/USB fact | Role in this investigation |
|---|---|---|
| Interface 0, `0x81 IN` | 8-byte interrupt endpoint; boot-keyboard Input report | Ordinary keyboard input |
| Interface 1, `0x82 IN` | 16-byte interrupt endpoint; numbered Input reports | Extended keyboard/consumer/system/mouse/vendor input |
| Interface 2, `0x83 IN` | 8-byte interrupt endpoint; no Input item in this interface's descriptor | Not the 64-byte Feature transport |
| Interface 2 Feature report | 64-byte payload, implicit Report ID 0 | EP0 HID class control request candidate |
| Windows updater buffers | Length/format not recovered | Separate firmware-updater transport; do not equate to Feature payload |

**[Physical observation]** The report descriptors for the keyboard and receiver have matching SHA-256 hashes per interface. Matching descriptors establish matching report declarations only; they do not prove matching firmware, command handlers, storage, or command semantics.

### Report ID and lengths

**[Physical observation]** Interface 2 contains no `Report ID` item. Its Feature report therefore uses implicit Report ID `0`; the 64 bytes are report data, not 63 bytes plus a report-ID byte. The descriptor declares `Report Size=8`, `Report Count=64`.

**[Static binary]** The native Rust driver's direct libusb path passes a 64-byte data buffer to `libusb_control_transfer`. The Electron UI bundle contains 64-byte command buffers (`Buffer.alloc(64)`) in a YZW-common implementation. A separate frontend catalogue entry advertises `featureReportByteLength=65` for VID `3151`, PID `4002`; that is not the observed TH80 Pro PID `4010` and may reflect a HID API report-ID prefix convention. It must not be used to redefine the physical 64-byte descriptor payload.

## 2. Recovered generic macOS EP0 requests

**[Static binary]** Disassembly of the Rust Mach-O `iot_driver` identifies `HidLibusb::send_feature` and `HidLibusb::get_feature`. Both call `libusb_control_transfer` with fixed request fields:

| Field | Feature write (`send_feature`) | Feature read (`get_feature`) |
|---|---:|---:|
| `bmRequestType` | `0x21` | `0xA1` |
| `bRequest` | `0x09` | `0x01` |
| `wValue` | `0x0300` | `0x0300` |
| `wIndex` | `0x0000` | `0x0000` |
| `wLength` | `64` | `64` |
| Report type | Feature (`3`) | Feature (`3`) |
| Report ID | `0` | `0` |
| Data direction | Host to device (OUT) | Device to host (IN) |
| Data-stage payload | 64 bytes supplied by caller | 64-byte receive buffer |
| Expected data-stage response | 0 bytes | 64 bytes requested |
| Driver timeout | 2000 ms | 2000 ms |

`wValue=0x0300` is `(Feature report type << 8) | Report ID`. `0x21` is a host-to-device, class, interface request; `0xa1` is device-to-host, class, interface. `bRequest=0x09` is HID `SET_REPORT`; `bRequest=0x01` is HID `GET_REPORT`. These are standard HID class requests, not vendor-specific USB requests.

The write routine checks that its caller buffer is at least 64 bytes and submits exactly the first 64 bytes. The read routine allocates a 64-byte buffer and asks libusb for 64 bytes. A successful physical transfer has **not** been observed.

### Critical interface-index mismatch

**[Static binary]** The native driver's embedded support configuration selects interface `2` for both `3151:4010` (vendor HID) and `3151:4011` (`dongle_common=true`). The disassembled generic `send_feature` and `get_feature` routines nevertheless load literal zero into the `wIndex` argument passed to `libusb_control_transfer`.

**[Physical observation]** The Feature report is declared on interface `2`; interface `0` declares keyboard Input and LED Output reports, but no Feature report. Consequently, the recovered generic transfer parameters do not match the observed interface-2 Feature declaration. This is an unresolved implementation inconsistency, not evidence that interface 0 is the correct destination. The caller path, any alternate transfer implementation, and whether the routine is usable for this TH80 Pro remain unresolved.

| Operation | `bmRequestType` | `bRequest` | `wValue` | `wIndex` | `wLength` | Report | Status |
|---|---:|---:|---:|---:|---:|---|---|
| macOS generic Feature SET | `0x21` | `0x09` | `0x0300` | `0` | `64` | Feature ID 0 | **[Static binary]**; interface mismatch |
| macOS generic Feature GET | `0xA1` | `0x01` | `0x0300` | `0` | `64` | Feature ID 0 | **[Static binary]**; interface mismatch |
| Actual TH80 Pro interface-2 SET parameters | Unknown | Unknown | Unknown | Must address interface 2 if HID class request | Unknown | Feature ID 0 per descriptor | **[Unknown]**; do not synthesize |
| Actual TH80 Pro interface-2 GET parameters | Unknown | Unknown | Unknown | Must address interface 2 if HID class request | Unknown | Feature ID 0 per descriptor | **[Unknown]**; do not synthesize |

No vendor-specific `bmRequestType`/`bRequest` pair has been established.

## 3. Application command framing evidence

**[Static binary]** The native Mach-O contains a `YZW_CMD_LIST` symbol and command names/values listed below. The Electron bundle also contains YZW-family classes that allocate 64-byte buffers and put a command constant in byte offset `0`. This establishes a command-framing pattern in the vendor software family, but not a complete, verified TH80 Pro packet format.

**Important applicability limit:** the Electron frontend's explicit `YZW3151` catalogue entry is VID `0x3151`, PID `0x4002`, not the connected TH80 Pro PID `0x4010`. The native driver's support configuration separately includes `3151:4010`, but the frontend caller-to-device mapping for this exact revision has not been proven. Therefore, the byte-offset interpretation and individual command behavior below are **static family evidence**, not confirmed physical TH80 Pro operations.

### Native YZW command vocabulary

| Name as embedded | Value | Static interpretation only |
|---|---:|---|
| `SET_REV` | `0x00` | Name present; semantics unknown |
| `SET_RESERT` | `0x02` | Name present; spelling retained; semantics unknown |
| `SET_REPORT` / `GET_REPORT` | `0x04` / `0x84` | Report-rate/configuration family in frontend code |
| `SET_PROFILE` / `GET_PROFILE` | `0x05` / `0x85` | Profile selection/query family in frontend code |
| `SET_KBOPTION` / `GET_KBOPTION` | `0x06` / `0x86` | Keyboard-option family in frontend code |
| `SET_LEDPARAM` / `GET_LEDPARAM` | `0x07` / `0x87` | LED-parameter family in frontend code |
| `SET_SLEDPARAM` / `GET_SLEDPARAM` | `0x08` / `0x88` | Sleep-parameter family by name |
| `SET_KEYMATRIX` / `GET_KEYMATRIX` | `0x09` / `0x89` | Key-matrix family by name |
| `SET_KEYENABLE` / `GET_KEYENABLE` | `0x0a` / `0x8a` | Key-enable family by name |
| `SET_MACRO` / `GET_MACRO` | `0x0b` / `0x8b` | Macro family by name |
| `SET_USERPIC` / `GET_USERPIC` | `0x0c` / `0x8c` | User-picture family by name |
| `SET_AUDIO` | `0x0d` | Audio setting family by name |
| `SET_WINDOS` | `0x0e` | Name retained as embedded; semantics unknown |
| `GET_BATTERY` | `0x83` | Battery query by name |
| `GET_REV` | `0x80` | Revision/version query by name |
| `GET_INFOR` | `0x8f` | Information query by name |

These values are **[Static binary]** evidence only. Names are not proof of side effects, persistence, support on both PIDs, or command-to-payload mapping. In particular, do not assume `SET_RESERT` is a reset command based only on its spelling.

### Query versus transport direction

**[Static binary]** The Electron YZW-common frontend implements a helper named `commomFeature`: it sends a 64-byte command buffer through its `sendMsg` path, waits, then obtains a Feature response through `readMsg`. Read methods such as firmware-version, profile, report-rate, keyboard-option and LED-setting getters use this helper. The native driver separately exposes `send_feature` and `get_feature` backed by HID `SET_REPORT` and `GET_REPORT` requests.

This means a semantic `GET_*` query may still require a host-to-device `SET_REPORT` to submit the query opcode, followed by a device-to-host `GET_REPORT` to retrieve the response. A HID `SET_REPORT` transaction is therefore not automatically a persistent-setting write. Conversely, the existence of a `SET_*` name does not prove the setting persists across power cycles.

**Classification:**

- Candidate query opcodes: `GET_REV`, `GET_BATTERY`, `GET_REPORT`, `GET_PROFILE`, `GET_KBOPTION`, `GET_LEDPARAM`, `GET_SLEDPARAM`, `GET_KEYMATRIX`, `GET_KEYENABLE`, `GET_MACRO`, `GET_USERPIC`, and `GET_INFOR`. **Static query intent only; none is confirmed read-only on the physical TH80 Pro.**
- Candidate configuration-write opcodes: `SET_REPORT`, `SET_PROFILE`, `SET_KBOPTION`, `SET_LEDPARAM`, `SET_SLEDPARAM`, `SET_KEYMATRIX`, `SET_KEYENABLE`, `SET_MACRO`, `SET_USERPIC`, `SET_AUDIO`, and `SET_WINDOS`. **Static write intent only; persistence and target-device applicability are unknown.**
- `SET_REV` and `SET_RESERT`: semantics unknown.
- No command is classified as physically confirmed read-only or physically confirmed persistent-write in this phase.

### Payload structures and remaining unknowns

**[Static binary]** The frontend's YZW-common profile constructs 64-byte command buffers with the opcode at byte `0`. Examples include a profile byte at offset `1` for profile/report-rate operations, a report-rate enum at offset `2`, and a keyboard-option bitfield at offset `2` with an additional field at offset `3`. Its version getter interprets response bytes `1..2` as a little-endian 16-bit value. These are examples from the PID `0x4002` frontend profile and must not be assumed to describe PID `0x4010` until the exact caller path is established.

**[Unknown]** For the TH80 Pro and receiver: complete field layouts, payload-length rules, checksum, sequence number, response status, command pagination, retries, command-specific delays, error handling, configuration storage location, persistence, and mode-specific behavior. The native driver exposes separate `YZWKeyboard` and `YZWDangleCommon` types and both return the same static command metadata list, but the receiver's support entry is marked `dongle_common=true`; shared metadata does not establish identical command handling.

## 4. Keyboard versus receiver

**[Static binary]** The macOS native support configuration includes:

- `3151:4010`, Usage Page `0xffff`, Usage `0x02`, interface `2`.
- `3151:4011`, Usage Page `0xffff`, Usage `0x02`, interface `2`, `dongle_common=true`.

The same native executable contains distinct `YZWKeyboard` and `YZWDangleCommon` API types. Disassembly of both `get_cmd_list` methods shows that each returns the same static `YZW_CMD_LIST` object. `YZWDangleCommon` additionally exposes its own light-type methods. This is evidence that both code paths share the driver's command metadata list, but it does **not** prove that all commands are supported identically, that payloads/transport are identical, whether commands are forwarded, or which device owns a setting.

**[Static binary]** The Windows updater's support list includes `3151:4010` but not `3151:4011`. Its bootloader list includes PID `0x4001`. The macOS driver includes 4011; the Windows updater's omission is not evidence that the receiver cannot communicate with the macOS driver.

## 5. macOS application architecture

The supplied macOS package contains an Electron application and an x86-64 Rust Mach-O executable `iot_driver`. The Electron bundle declares gRPC methods including `sendMsg`, `readMsg`, `sendRawFeature`, and `readRawFeature`; the native executable contains the corresponding gRPC driver/server symbols. The native executable links libusb and calls `libusb_control_transfer` for the generic Feature operations above.

The native Mach-O links IOKit and contains actual `IOHIDManager`/`IOHIDDevice` call sites. These implement the generic HIDAPI backend: manager creation/matching/enumeration, device open/close, and input-report callback registration. The one recovered `_hid_write` call site invokes `IOHIDDeviceSetReport` with report type `1` (Output), after splitting the report-ID byte from the HIDAPI buffer when it is zero. No `IOHIDDeviceGetReport` import/call or Feature-type (`2`) `IOHIDDeviceSetReport` call was found in this binary. Thus IOKit is used for HID enumeration/input and generic Output reports, while the identified Feature GET/SET implementation is the separate libusb path described above. The application was not executed on this Linux host; no app-driven traffic was captured.

## 6. Evidence and confidence summary

| Finding | Evidence class | Confidence / limitation |
|---|---|---|
| Interface 2 declares one unnumbered 64-byte Feature report | Physical observation | High; descriptor-derived |
| Interface 2's report ID is 0 | Physical observation | High; no Report ID item |
| macOS generic SET request fields `21/09/0300/0000/64` | Static binary | High for this function; not a physical transaction |
| macOS generic GET request fields `a1/01/0300/0000/64` | Static binary | High for this function; not a physical transaction |
| Native support config selects interface 2 for 4010 and 4011 | Static binary | High for embedded config |
| Generic transfer uses `wIndex=0` | Static binary | High; conflicts with interface-2 support/descriptor |
| Native YZW command names and values | Static binary | High that names/values are embedded; semantics/applicability remain unverified |
| Byte 0 holds command opcode in YZW frontend buffers | Static binary | High for inspected frontend profile; frontend profile PID is 4002, not 4010 |
| `GET_*` commands are physically read-only | Unknown | No device trace or read-back validation |
| `SET_*` commands persist configuration | Unknown | No power-cycle/persistence evidence |
| Receiver shares keyboard command protocol | Partially supported by static metadata | Both `YZWKeyboard::get_cmd_list` and `YZWDangleCommon::get_cmd_list` return the same static `YZW_CMD_LIST`; actual command handling, payloads and storage remain unknown |
| Safe read-only configuration API | Not established | Transport mismatch and command applicability unresolved |

## 7. Safe next steps

1. Obtain a passive USB capture from the official application on a supported macOS host, without triggering firmware, reset, pairing, or factory-service operations.
2. Correlate the application UI operation with the exact USB device, interface, setup packet and data stage.
3. Capture one query at a time, then distinguish the query opcode submission from the Feature GET response.
4. Capture the keyboard and receiver separately; do not infer receiver semantics from the keyboard trace.
5. Resolve the `wIndex=0` versus interface-2 mismatch through static caller analysis or passive traces before implementing any Feature operation.
6. Keep the Linux CLI limited to enumeration, descriptor reads, device metadata, and explicitly requested input capture. Do not add Feature or Output writes.

No firmware backup, restoration, or recovery capability is established by this protocol work.
