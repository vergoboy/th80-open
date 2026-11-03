# Reverse-engineering notes

## Supplied artifacts

The workspace contains:

- `TH80_ANSI_Firmware_V504.zip`
  - `TH80 Upgrade Firmware for v5xx.exe` (10,274,816 bytes)
  - `Upgrade Guide.pdf`
  - `selections_config.toml`
  - `support_config.toml`
  - `upgrade_config.toml`
  - `X` (opaque 184,608-byte file; format not identified)
- `EPOMAKER_Driver(MAC2023021).dmg` and its ZIP copy
  - Electron app `EPOMAKER Driver 1.0.0`
  - bundled `iot_driver` Mach-O x86-64 executable
  - bundled JavaScript UI and device catalogue

The files in the workspace have not been modified.

## Windows updater observations

Static strings identify the updater as a Rust application. Embedded source paths include:

- `src/data_source/yzw_upgrade.rs`
- `src/hid/detect_hid.rs`
- `src/hid/dj_device.rs`
- `src/hid/windows_bind.rs`
- `src/egui_ui/yzw_upgrade.rs`

The updater includes HIDAPI operations and Windows HID feature-report calls (`HidD_SetFeature`, `HidD_GetFeature`, `HidD_GetInputReport`). It also contains HTTP endpoints for update metadata and firmware downloads. Those network paths are vendor infrastructure and are not required for a local open-source Linux implementation.

### TH80 updater configuration

`support_config.toml` contains the following entries:

| Mode | VID:PID | Usage page | Usage | Interface |
|---|---|---:|---:|---:|
| Normal keyboard | `3151:4002` | `0x0001` | `0x06` | 0 |
| TH80 Pro vendor HID | `3151:4010` | `0xffff` | `0x02` | 2 |
| Bootloader | `3151:4001` | `0xff01` | `0x01` | -1 (wildcard) |

The same file contains other device-family identifiers. They must not be treated as confirmed TH80 interfaces.

`upgrade_config.toml` sets `before_start_delay=1000`, `start_delay=1000`, `per_delay=20`, and `end_delay=100`. These are updater timing settings, not enough to reconstruct the transfer protocol.

The guide says to close the driver, disconnect the 2.4 GHz receiver, put the keyboard in wired mode, and connect by cable. It describes reconnecting a failed keyboard so the updater can detect a bootloader state. This supports a wired USB bootloader path, but does not reveal packet contents or recovery guarantees.

The opaque `X` file is 184,608 bytes and has a mixed entropy profile: long zero-filled regions and several high-entropy regions. Its format and role remain unknown; it has not been identified as a firmware image. SHA-256: `4b22336ec9ffaaac3372791a1154a2c2a833e3305a4cadbbdcab9184252c0520`.

## macOS driver observations

The macOS package is an Electron application. Its UI bundle includes a broad device catalogue and a Rust native executable named `iot_driver`. The native executable contains modules for USB/HID discovery, HID sleep/status, Bluetooth discovery (`btleplug`), and vendor HID support configuration. This suggests that the driver architecture separates UI, native device access, and cloud/device metadata.

The UI's generic `YZWCommon` entry uses VID `0x3151`, PID `0x4002`, keyboard usage page `0x0001`, usage `0x0006`, and a 65-byte feature-report length. The catalogue entry is generic and does not uniquely prove that every TH80 feature is supported by that layout.

The macOS native support configuration contains more `0x3151` PIDs, including `4001`, `4002`, `4003`, `4007`, `4008`, `4010`, `4011`, `4012`, `4013`, `4014`, `4015`, `4016`, and `4017`. Several are vendor HID, dongle, or BLE entries. These are ecosystem clues only; the TH80 updater's explicit `support_config.toml` is the stronger source for the `4001`, `4002`, and `4010` entries in the table above.

## Live TH80 Pro USB observations

The Linux host used for this investigation has the TH80 Pro and its 2.4 GHz receiver connected. The read-only HIDAPI probe found:

| Device | VID:PID | `bcdDevice` reported by HIDAPI | Interfaces |
|---|---|---:|---|
| EPOMAKER TH80 Pro | `3151:4010` | `0x0505` | 0, 1, 2 |
| ROYUAN 2.4G Wireless Keyboard receiver | `3151:4011` | `0x0502` | 0, 1, 2 |

The TH80 Pro and receiver expose matching report-descriptor layouts. Interface 0 is the standard keyboard input report; interface 1 carries consumer, system-control, keyboard, mouse, and a small vendor input collection; interface 2 exposes a vendor-defined feature report. The interface-2 descriptor declares usage page `0xffff`, usage `0x02`, 64 data bytes, and a feature item. HIDAPI's conventional feature-report buffer includes a report-ID byte, which explains the vendor UI's 65-byte buffer setting; the descriptor itself has no explicit Report ID item for this feature collection.

The receiver's `4011` PID is visible in the live host and in the macOS native support configuration. The updater's `4010` vendor-interface entry matches the live TH80 Pro. This is stronger evidence for the actual Pro revision than the generic `4002` catalogue entry.

Raw descriptor bytes are recorded in [hid-descriptors.md](hid-descriptors.md). The host probe did not open either device or issue HID requests.

## Phase 2 macOS HID transport findings

The macOS DMG's iot_driver Mach-O symbol table exposes Rust modules for hid_libusb, hid_ble, YZWKeyboard, YZWDangleCommon and YZWBoot. Disassembly of HidLibusb::send_feature and HidLibusb::get_feature shows standard HID class control transfers over EP0:

| Operation | bmRequestType | bRequest | wValue | wIndex | Length | Timeout |
|---|---:|---:|---:|---:|---:|---:|
| SET_FEATURE | 0x21 | 0x09 | 0x0300 | 0 | 64 bytes | 2000 ms |
| GET_FEATURE | 0xa1 | 0x01 | 0x0300 | 0 | 64 bytes | 2000 ms |

The report type in wValue is Feature (3) and report ID is zero. The native driver transfers exactly 64 bytes. This is static implementation evidence; no request was sent to the physical device during this investigation.

The native binary contains a YZW_CMD_LIST symbol and names/constants for revision, information, battery, reports, profiles, keyboard options, LED parameters, sleep parameters, key matrix, key enable, macros, user pictures, audio and OS mode. See USB_PROTOCOL.md for the recovered constant table. Payload layouts and command-to-device applicability remain unknown.

The native support config associates 3151:4010 interface 2, page 0xffff, usage 0x02 with vendor HID. It separately associates receiver 3151:4011 interface 2 with dongle_common=true. This supports separate device backend paths, but does not prove where settings are stored.

The macOS JavaScript frontend's generic YZWCommon catalogue entry is 3151:4002 and its embedded command constants differ from the native YZW_CMD_LIST. Do not merge those command tables until the relevant call paths and device model are proven equivalent.

## Passive capture status

tshark and wireshark are installed, but usbmon is not loaded, /dev/usbmon* is absent, and the current session cannot access the usbmon/debugfs or USB tracing paths. The macOS configuration application cannot run natively in this Linux environment; the supplied Windows executable is a firmware updater. No USB traffic capture or vendor configuration action was performed.

## What remains unknown

- Exact TH80 Pro board revision and whether the supplied ANSI V5.04 package matches it.
- MCU vendor/model, flash size, boot ROM, and debug interface availability.
- Matrix row/column wiring, diode direction, encoder/OLED/battery/RGB controllers, and wireless chipset.
- Firmware payload format and whether `X` is firmware, metadata, or an unrelated resource.
- HID report IDs, command opcodes, packet framing, checksums, acknowledgements, and retry behavior.
- Whether the 65-byte feature-report size applies to the actual TH80 hardware revision.

No write command or flash operation has been attempted.

## Vial / QMK compatibility assessment

Vial is a firmware and host-application ecosystem. Its host transport can be implemented over HID, but the keyboard firmware must implement the Vial/VIA protocol: a fixed 32-byte packet format, Vial command prefix, protocol/UID handshake, keyboard definition responses, and feature-specific read/write and unlock handlers. The TH80 Pro's vendor feature descriptor exposes a 64-byte payload, and no Vial handshake or QMK-compatible firmware evidence has been observed. Therefore Vialite cannot configure the stock TH80 Pro merely because it can access WebHID.

Vialite is a React/TypeScript WebHID frontend whose protocol layer ports Vial GUI behavior. It may be a useful UI reference or a future frontend if replacement firmware implements Vial. Its GPL-2.0 licensing and protocol coupling need review before code reuse. For the stock TH80 Pro, the host application needs a separately implemented client for the Epomaker/Rongyuan vendor protocol once that protocol is understood.

References: [Vialite](https://github.com/tcyeee/vialite), [Vial QMK](https://github.com/vial-kb/vial-qmk), [Vial porting guide](https://get.vial.today/docs/porting-to-vial.html).
