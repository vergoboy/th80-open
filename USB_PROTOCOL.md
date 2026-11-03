# USB protocol inventory

## Scope and evidence

This document describes the live USB descriptors and static transport implementation found in the supplied macOS native driver. No vendor configuration operation was performed on the physical keyboard. No undocumented Feature report was read or written.

Evidence classes are marked **Physical observation**, **Static binary**, **Vendor documentation**, **Hypothesis**, or **Unknown**.

## USB device and endpoint layout

The connected keyboard (3151:4010) and receiver (3151:4011) each expose one USB configuration with three HID interfaces. Both run at Full Speed (12 Mbps). Their corresponding report descriptor SHA-256 values match:

| Interface | Descriptor size | SHA-256 | Endpoint | Max packet | Interval |
|---:|---:|---|---|---:|---:|
| 0 | 59 | a5395cb7e86bebce11aecb4cc6d73d5d2b23d1c63f28df3c7c90069490213dd8 | 0x81 IN | 8 | 1 ms |
| 1 | 177 | 3cc697106805cc7be08443be8dae9f4eda2069d7b5c8352e03abdea7e5a98c74 | 0x82 IN | 16 | 1 ms |
| 2 | 20 | 350fea62e225e3d92e9db3bbbd408fcebe386c10405454be661d3d2c3229b0b5 | 0x83 IN | 8 | 1 ms |

**Physical observation:** USB configuration and endpoint values were read from lsusb -v and sysfs. No interrupt OUT endpoint is declared. Feature and Output reports therefore use endpoint zero (control transfers), rather than the interrupt-IN endpoints above.

## Interface 0 — boot keyboard

- Top-level collection: Generic Desktop / Keyboard (Usage Page 0x01, Usage 0x06).
- Keyboard LEDs: LED usage page 0x08, usages 0x01 through 0x05, five output bits plus three constant padding bits.
- Modifier bitmap: Keyboard/Keypad page 0x07, usages 0xe0 through 0xe7.
- Six-key array: eight-bit values, logical range 0..255.
- Input report: 64 bits (8 bytes), implicit report ID 0.
- Output report: 8 bits (1 byte), implicit report ID 0; no interrupt OUT endpoint, so it uses EP0.
- Logical ranges: LED/modifier bits 0..1; key array 0..255.
- Physical ranges: not declared.

**Physical observation:** the interface subclass/protocol is HID Boot Interface / Keyboard (1/1), endpoint 0x81 IN, 8-byte maximum packet, 1 ms interval.

## Interface 1 — extended reports

The descriptor contains multiple top-level collections:

| Collection | Usage Page | Usage | Report ID | Payload | Meaning indicated by HID usages |
|---|---:|---:|---:|---:|---|
| Consumer Control | 0x0c | 0x01 | 3 | 16 bits | Consumer usages 0..0x33c |
| System Control | 0x01 | 0x80 | 2 | 8 bits | System control usages 0x81..0x83 plus padding |
| Keyboard | 0x01 / 0x07 | 0x06 | 1 | 120 bits | 120 one-bit key-state usages |
| Mouse / Pointer | 0x01 | 0x02 / 0x01 | 6 | 56 bits | Five buttons, X/Y, wheel and Consumer AC Pan |
| Vendor-defined | 0xffff | 0x01 | 5 | 24 bits | Three opaque vendor input bytes |

Report lengths including the report-ID byte are 3, 2, 16, 8 and 4 bytes respectively. The largest report is 16 bytes, matching the 0x82 IN endpoint's 16-byte maximum packet.

Logical ranges include Consumer 0..828, system and button bits 0..1, mouse X/Y -32767..32767, wheel and AC Pan -127..127, and vendor input bytes 0..255. Physical ranges are not declared.

**Unknown:** the semantics of the three vendor input bytes (Report ID 5) have not been observed.

## Interface 2 — vendor-defined Feature report

- Top-level collection: Usage Page 0xffff, Usage 0x02.
- One Feature item: 64 bytes (Report Size 8, Report Count 64).
- No explicit Report ID item: implicit ID 0.
- Logical range: signed byte -128..127.
- Physical range: not declared.
- No Input or Output report item is declared in this interface's report descriptor.
- USB interface has an interrupt-IN endpoint 0x83, max packet 8 bytes, interval 1 ms; it is not described as carrying an Input report.
- No interrupt OUT endpoint exists.

**Physical observation:** the descriptor declares a 64-byte Feature payload. This size is independent of the Windows updater's firmware transfer buffers and must not be assumed to define firmware packet framing.

### macOS native driver Feature transfer

**Static binary analysis:** disassembly of iot_driver's Rust HidLibusb::send_feature and HidLibusb::get_feature functions shows calls to libusb_control_transfer with these parameters:

| Operation | bmRequestType | bRequest | wValue | wIndex | wLength | Timeout |
|---|---:|---:|---:|---:|---:|---:|
| SET_FEATURE | 0x21 | 0x09 | 0x0300 | 0x0000 | 64 | 2000 ms |
| GET_FEATURE | 0xa1 | 0x01 | 0x0300 | 0x0000 | 64 | 2000 ms |

wValue=0x0300 encodes HID Feature report type (3) and report ID (0). The request types/directions and request codes are standard HID class control requests. The driver sends the first 64 bytes of its supplied buffer and allocates a 64-byte receive buffer. However, the observed transfer uses wIndex=0 while the live TH80 Pro vendor Feature report is declared on interface 2. The support configuration does select interface 2 for PID 3151:4010, but the caller path connecting that support entry to this generic transfer routine has not been resolved. This disassembly therefore does not yet prove that the routine is the active TH80 Pro interface-2 transport. No command has been sent to the keyboard.

HIDAPI's Feature API uses a report-ID byte in its caller buffer; for an unnumbered report that byte is zero. Thus a caller may provide a 65-byte HIDAPI buffer while the USB Feature payload remains 64 bytes. This is an API buffer convention, not a 65-byte descriptor payload.

## Vendor command vocabulary in the macOS native binary

**Static binary analysis:** the native Rust binary contains a YZW_CMD_LIST symbol and these command names/values. This is a decoded static command vocabulary, not a trace of commands sent to the connected keyboard.

| Name | Value | Name | Value |
|---|---:|---|---:|
| SET_REV | 0x00 | GET_REV | 0x80 |
| SET_RESERT | 0x02 | GET_INFOR | 0x8f |
| GET_BATTERY | 0x83 | SET_REPORT | 0x04 |
| GET_REPORT | 0x84 | SET_PROFILE | 0x05 |
| GET_PROFILE | 0x85 | SET_KBOPTION | 0x06 |
| GET_KBOPTION | 0x86 | SET_LEDPARAM | 0x07 |
| GET_LEDPARAM | 0x87 | SET_SLEDPARAM | 0x08 |
| GET_SLEDPARAM | 0x88 | SET_KEYMATRIX | 0x09 |
| GET_KEYMATRIX | 0x89 | SET_KEYENABLE | 0x0a |
| GET_KEYENABLE | 0x8a | SET_MACRO | 0x0b |
| GET_MACRO | 0x8b | SET_USERPIC | 0x0c |
| GET_USERPIC | 0x8c | SET_AUDIO | 0x0d |
| SET_WINDOS | 0x0e | | |

Names imply command categories such as revision, information, battery, reports, profiles, keyboard options, LED parameters, sleep parameters, key matrix, key enable, macros, user pictures, audio and OS mode. Exact payload structures, checksums, pagination, state transitions, response codes and command-to-PID applicability remain **Unknown**. The misspelling RESERT and WINDOS are retained as found in the binary.

The JavaScript YZWCommon frontend bundle contains a separate command table with different values for several operations (for example report/profile/LED/reset). It is a different protocol profile or device implementation and must not be merged with the native YZW_CMD_LIST without proving their relationship. The generic frontend catalogue entry is for 3151:4002, not the observed 3151:4010 device.

### Phase 3 framing and applicability clarification

**Static binary:** the Electron frontend contains 64-byte YZW command buffers with the command opcode placed at byte offset 0. Its generic YZW3151 catalogue entry is `3151:4002`, not this TH80 Pro's `3151:4010`; this buffer layout is therefore vendor-family evidence, not a verified TH80 Pro packet structure. The native `iot_driver` Feature functions pass exactly 64 bytes to libusb. The frontend's separate `featureReportByteLength=65` entry is not evidence for a 65-byte USB report.

**Static binary:** the frontend query helper sends a command buffer through its `sendMsg` path and then retrieves a Feature response through `readMsg`. A semantic `GET_*` query can therefore use the HID `SET_REPORT` transport request to submit the query and a HID `GET_REPORT` request to retrieve the response. Do not equate transport direction with command side effect.

**Static binary:** the native driver has separate `YZWKeyboard` and `YZWDangleCommon` implementations. Its `3151:4011` support entry is marked `dongle_common=true`. Both classes return the same static command metadata list, but the receiver must not be assumed to implement every command identically.

Further disassembly shows both classes' `get_cmd_list` methods return the same static `YZW_CMD_LIST` object. The receiver-specific class also has separate light-type methods. Shared command metadata is not proof of identical transport or command behavior. The Mach-O uses IOKit/IOHIDManager for HID enumeration and input callbacks; its one recovered `IOHIDDeviceSetReport` call is report type 1 (Output), not Feature. No IOHID Feature GET/SET path was found; the Feature implementation identified here is libusb-based.

See [EP0_PROTOCOL.md](EP0_PROTOCOL.md) for request-field tables, command evidence classifications, and unresolved payload details.

## Windows updater

**Static binary analysis:** the updater is Rust, includes HIDAPI and Windows HID feature APIs (HidD_SetFeature, HidD_GetFeature, HidD_GetInputReport), and its support configuration explicitly selects 3151:4010, Usage Page 0xffff, Usage 0x02, interface 2. It identifies bootloader PID 3151:4001 with Usage Page 0xff01, Usage 0x01, wildcard interface -1.

The Windows updater's upgrade_config.toml contains delays of 1000 ms before start, 1000 ms at start, 20 ms per transfer unit, and 100 ms at end. These are timing parameters only. They do not identify firmware packet framing or prove a relationship to the 64-byte feature report.

## Passive monitoring status

- tshark and wireshark are installed on the Linux host.
- usbmon is not loaded and /dev/usbmon* is absent.
- Access to /sys/kernel/debug/usb/usbmon and /sys/kernel/tracing/events/usb is denied in the current session.
- No Linux-native vendor configuration application is available in the supplied materials. The supplied macOS app cannot be run natively on this Linux host; the Windows artifact is a firmware updater, not the configuration UI.
- No USB traffic capture was made. No official configuration action was performed.

The next passive capture must run on a host where the official configuration app can run and USB capture permissions are available (for example macOS with a passive USB capture tool, or Linux after usbmon access is provisioned and a compatible official app is available). Capture only user-initiated settings changes and retain the original pcapng plus device/firmware identity.
