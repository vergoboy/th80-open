# HID protocol analysis — Phase 2

## Executive result

The TH80 Pro (3151:4010) and its ROYUAN 2.4 GHz receiver (3151:4011) expose matching three-interface USB HID layouts. Interface 2 is the vendor-specific interface selected by both the Windows updater and the macOS native driver's support configuration. Its descriptor contains one unnumbered 64-byte Feature report. Static disassembly of the macOS driver identifies a generic 64-byte Feature-report path using standard HID GET_REPORT and SET_REPORT requests with report ID 0. That routine passes interface index 0, however, while the TH80 Pro Feature report is on interface 2. Its caller path has not yet been traced to the TH80 Pro interface-2 support entry.

The macOS binary also contains a named YZW command vocabulary covering revision, information, battery, report/profile, keyboard options, LED/sleep parameters, key matrix, macros, user pictures and other settings. These command names are useful static evidence. They are not evidence that the corresponding commands were issued to this keyboard or that every command is supported by PID 4010 or 4011.

No undocumented command was sent. No Feature report was read or written. No firmware operation was attempted. The keyboard and receiver remained connected and functional during enumeration and the short input-recorder smoke test.

## Evidence classes

| Label | Meaning |
|---|---|
| **Physical observation** | Directly observed from the connected hardware, USB descriptors, sysfs or a read-only host operation. |
| **Static binary** | Extracted from supplied executable/configuration artifacts without running vendor code against the keyboard. |
| **Vendor documentation** | Stated in the supplied updater guide or package configuration. |
| **Hypothesis** | A reasoned interpretation that needs independent validation. |
| **Unknown** | Evidence is insufficient to answer. |

## 1. Physical devices and interfaces

**Physical observation:** the Linux host enumerates:

- 3151:4010, ROYUAN EPOMAKER TH80 Pro, bcdDevice 0x0505, Full Speed (12 Mbps), one configuration, three interfaces.
- 3151:4011, ROYUAN 2.4G Wireless Keyboard, bcdDevice 0x0502, Full Speed (12 Mbps), one configuration, three interfaces.

Both devices have these endpoints:

| Interface | Endpoint | Direction | Type | Max packet | Interval |
|---:|---:|---|---|---:|---:|
| 0 | 0x81 | IN | Interrupt | 8 | 1 ms |
| 1 | 0x82 | IN | Interrupt | 16 | 1 ms |
| 2 | 0x83 | IN | Interrupt | 8 | 1 ms |

No interrupt OUT endpoint is declared. HID Output and Feature reports therefore use EP0. Report descriptor hashes are identical between the two devices for each interface. See HARDWARE.md, USB_PROTOCOL.md and docs/hid-descriptors.md.

## 2. Report descriptor findings

### Interface 0

**Physical observation:** Generic Desktop Keyboard top-level collection, boot keyboard subclass/protocol. It has an 8-byte Input report and a 1-byte Output report for keyboard LEDs. Both are unnumbered (implicit report ID 0). The Output report has no interrupt OUT endpoint and is consequently a control-pipe report.

### Interface 1

**Physical observation:** the 177-byte descriptor defines five numbered Input reports:

- ID 1: 120-bit keyboard bitmap (15 payload bytes, 16 bytes including report ID).
- ID 2: 8-bit System Control report (2 bytes including ID).
- ID 3: 16-bit Consumer Control report (3 bytes including ID).
- ID 5: 24-bit vendor-defined input report (4 bytes including ID).
- ID 6: 56-bit mouse report (8 bytes including ID).

ID 4 is not declared. Mouse data includes five buttons, X/Y axes, wheel and Consumer AC Pan. The vendor input report's three payload bytes have unknown meaning.

### Interface 2

**Physical observation:** Usage Page 0xffff, Usage 0x02, one Feature report with 64 data bytes, implicit report ID 0, logical range -128..127. The descriptor contains no Input or Output report item. Physical ranges are not specified. An interrupt-IN endpoint exists in the interface descriptor but has no matching Input report definition.

The Feature payload size is a HID report descriptor property. It does not define the firmware updater's data block format.

## 3. Static macOS driver transport analysis

The supplied DMG contains an Electron application and a Rust x86-64 Mach-O executable named iot_driver. Its symbol table preserves Rust function names and module paths.

**Static binary:** HidLibusb::send_feature calls libusb_control_transfer with:

    bmRequestType = 0x21  (host-to-device, class, interface)
    bRequest      = 0x09  (HID SET_REPORT)
    wValue        = 0x0300 (Feature report, report ID 0)
    wIndex        = 0
    wLength       = 64
    timeout       = 2000 ms

The function checks that its input buffer is at least 64 bytes and transfers 64 bytes. HidLibusb::get_feature calls libusb_control_transfer with:

    bmRequestType = 0xa1  (device-to-host, class, interface)
    bRequest      = 0x01  (HID GET_REPORT)
    wValue        = 0x0300 (Feature report, report ID 0)
    wIndex        = 0
    wLength       = 64
    timeout       = 2000 ms

This is direct static evidence for a generic HID Feature transport routine in the macOS binary. There is an unresolved interface mismatch: the disassembled routine passes wIndex=0, whereas the live TH80 Pro's vendor Feature report is declared on interface 2 and the native support config selects interface 2 for PID 3151:4010. The caller path and interface selection need further tracing before this routine can be attributed to the TH80 Pro. It is not a live observation and does not reveal the contents of any Feature report.

The binary includes modules and symbols for hid_libusb, hid_ble, YZWKeyboard, YZWDangleCommon, YZWBoot, and FEA_CMD_*. It exposes generic HID methods including pure_get_feature and pure_send_feature, plus methods named get_id, send_msg and get_24_status.

## 4. Static command vocabulary

The native binary contains a YZW_CMD_LIST symbol and the following named constants:

| Command | Value | Command | Value |
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

The names indicate likely feature areas, but payload layouts and exact semantics remain unknown. The binary's support configuration associates 3151:4010, interface 2, Usage Page 0xffff, Usage 0x02 with the vendor HID path. This is evidence that the native driver intends to support the observed vendor interface. It does not independently prove that all commands in the generic command table are valid for the TH80 Pro. The interface-index mismatch above also means the command table and generic Feature transfer routine have not yet been tied to the same active call path.

The JavaScript frontend contains a separate YZWCommon command table whose values differ for multiple commands. Its generic model entry is 3151:4002. Treat that as a separate protocol profile until a code path proves otherwise. Do not combine the two command maps.

## 5. Windows updater analysis

**Static binary:** the supplied Windows updater is a Rust PE32+ application. Embedded strings include HidD_SetFeature, HidD_GetFeature, HidD_GetInputReport, WriteFile, and HID-related source paths (src/hid/detect_hid.rs, src/hid/dj_device.rs, src/hid/windows_bind.rs, src/data_source/yzw_upgrade.rs).

Its support_config.toml identifies:

| Mode | VID:PID | Usage Page | Usage | Interface |
|---|---|---:|---:|---:|
| Normal keyboard | 3151:4002 | 0x0001 | 0x06 | 0 |
| TH80 Pro vendor HID | 3151:4010 | 0xffff | 0x02 | 2 |
| Bootloader | 3151:4001 | 0xff01 | 0x01 | wildcard -1 |

The receiver 3151:4011 is not listed in this updater config. The updater's timing configuration is 1000 ms before start, 1000 ms start delay, 20 ms per transfer unit, and 100 ms end delay. No firmware write or bootloader request was attempted. The updater's use of HID feature APIs does not prove its firmware data chunks are identical to the 64-byte configuration Feature report.

## 6. Device relationship and modes

### Wired USB

**Physical observation:** the keyboard enumerates as 3151:4010 with the three interfaces above.

**Vendor documentation:** the supplied updater guide instructs the user to close the driver, disconnect the 2.4 GHz receiver, switch the keyboard to wired mode and connect the cable before updating. This supports a vendor-documented wired USB updater path.

### 2.4 GHz

**Physical observation:** the receiver independently enumerates as 3151:4011, and its three HID descriptors match the keyboard's.

**Static binary:** the macOS native support configuration lists 3151:4011 as Usage Page 0xffff, Usage 0x02, interface 2, with dongle_common = true. This is a stronger link between the receiver and the vendor HID stack than the updater's preliminary PID list, which omits 4011.

**Hypothesis:** the receiver may expose a vendor control channel for receiver-side functions. The dongle_common label suggests a distinct driver path. It does not establish which settings are stored in the receiver or keyboard.

### Bluetooth

**Static binary:** the macOS driver includes a BLE HID module and btleplug/CoreBluetooth support. Its native support config also contains BLE entries 3151:4013 (Usage Page 0xff55, Usage 0x0202) and 3151:4012 (Usage Page 0xff66, Usage 0x0202).

**Unknown:** no live Bluetooth-mode TH80 Pro device was captured, and no model-specific mapping ties those BLE PIDs to this exact keyboard revision. The driver's general BLE support does not prove the TH80 Pro uses those IDs or that its Bluetooth configuration protocol matches the USB Feature report.

### Where settings are stored

**Unknown:** no configuration read, write, vendor-app action, or traffic trace was performed. We cannot currently attribute keymaps, RGB, macros, profiles, sleep settings, pairing data, or battery state to persistent keyboard storage, receiver storage, or host-side storage.

## 7. Passive monitoring

No passive USB capture was collected in this phase. tshark and wireshark are installed, but usbmon is not loaded, /dev/usbmon* is absent, and this session cannot access the usbmon/debugfs and USB tracing paths. The only supplied macOS configuration application cannot be run natively on this Linux host. The supplied Windows program is a firmware updater, not the vendor configuration UI.

For the next capture, run the original configuration application on a supported OS and capture the USB bus passively. Capture one user-initiated action at a time (initialization, read-only status display, RGB setting, key remap, sleep option) and save both the original pcapng and application/device version. Avoid any firmware/update or factory-reset UI. Classify setup packets by interface, request type, request, value, index and length before interpreting data bytes.

## 8. Read-only probe

The Rust CLI is th80ctl in this repository. It:

- Enumerates the two known live device IDs and deduplicates HIDAPI collection paths.
- Reads USB product/manufacturer/release/usage/interface information exposed by HIDAPI.
- Reads each hidraw report descriptor from sysfs and parses report IDs, Input/Output/Feature bit lengths, usages, collections and logical/physical ranges.
- Exports a JSON diagnostic snapshot.
- Offers an explicit, bounded input-report recorder for the boot-keyboard interface.

It has no Feature-report read/write, Output-report write, arbitrary control-transfer, or firmware-flashing operation. Input recording is opt-in and captures raw key data; users should treat the output as sensitive.

## 9. Unknowns and next safe work

1. Obtain a passive trace of the vendor configuration app while performing one non-destructive setting change at a time.
2. Compare 4010 and 4011 traces to identify whether each endpoint/feature path is used.
3. Capture the keyboard's BLE advertisements and GATT metadata while it is explicitly in Bluetooth mode; do not pair or write characteristics during discovery.
4. Inspect the macOS binary's higher-level YZW command handlers and the Windows updater's firmware transfer implementation to recover exact payload structures, without invoking them on hardware.
5. Identify the MCU and board revision from PCB photographs before considering replacement firmware.
6. Keep firmware flashing unsupported until a complete restorable backup and recovery route are demonstrated.

No firmware backup or recovery capability has been established.

## 10. Phase 3 — EP0 and command framing refinement

### Generic native Feature transfer, not a confirmed TH80 transaction

**Static binary:** focused disassembly of `HidLibusb::send_feature` and `HidLibusb::get_feature` confirms the exact generic libusb request fields:

| Operation | `bmRequestType` | `bRequest` | `wValue` | `wIndex` | `wLength` | Timeout |
|---|---:|---:|---:|---:|---:|---:|
| Feature SET | `0x21` | `0x09` | `0x0300` | `0` | `64` | 2000 ms |
| Feature GET | `0xa1` | `0x01` | `0x0300` | `0` | `64` | 2000 ms |

The report is Feature type 3, Report ID 0. The SET path checks a minimum 64-byte source buffer and transfers 64 bytes; the GET path allocates a 64-byte buffer and requests 64 bytes. These are standard HID class requests. No live transaction was captured.

**Static binary/physical descriptor mismatch:** the native support config selects interface 2 for `3151:4010` and `3151:4011`, while both generic transfer functions pass `wIndex=0`. The physical Feature report is declared on interface 2, and interface 0 has no Feature report. This discrepancy prevents attributing the generic function as a working TH80 Pro interface-2 transport until its caller path or an alternate implementation is established.

### Command names versus verified behavior

**Static binary:** the native `YZW_CMD_LIST` contains command names and values for revision, battery, information, report rate, profile, keyboard options, LED/sleep parameters, key matrix, key enable, macros, user pictures, audio, and OS mode. The Electron bundle shows a YZW-family implementation using 64-byte buffers with an opcode at offset 0, but the explicit `YZW3151` frontend catalogue entry is PID `0x4002`, not the observed TH80 Pro PID `0x4010`.

The frontend's query helper sends the query buffer through `sendMsg` and then reads a response through `readMsg`. Thus a semantic `GET_*` command can use a host-to-device HID `SET_REPORT` request to submit the query and a device-to-host `GET_REPORT` request for the response. This is static frontend behavior, not physical traffic. No `GET_*` command is yet confirmed read-only on the TH80 Pro, and no `SET_*` command is confirmed to persist settings.

The native driver has separate `YZWKeyboard` and `YZWDangleCommon` classes. The `3151:4011` support entry carries `dongle_common=true`; receiver command behavior is therefore independently unresolved.

Additional Mach-O inspection confirms that the native binary does use IOKit: `IOHIDManager` creates/matches/enumerates devices, `IOHIDDevice` opens devices and registers input callbacks, and the generic `_hid_write` routine calls `IOHIDDeviceSetReport` with report type `1` (Output). No `IOHIDDeviceGetReport` symbol/call or Feature-type (`2`) `IOHIDDeviceSetReport` call was found. This IOKit/HIDAPI path is not the recovered Feature-report path; Feature GET/SET remains the separate libusb implementation with the unresolved `wIndex=0` mismatch.

Both `YZWKeyboard::get_cmd_list` and `YZWDangleCommon::get_cmd_list` were disassembled and return the same static `YZW_CMD_LIST` object. The receiver class also exposes distinct light-type methods. Shared command metadata does not prove identical command execution, payload handling, or configuration ownership.

See [EP0_PROTOCOL.md](EP0_PROTOCOL.md) for the complete static opcode inventory, confidence labels, setup packet fields, and remaining unknowns.
