# Research status — TH80 Pro

## Scope

This report covers the connected TH80 Pro (3151:4010), the ROYUAN 2.4 GHz receiver (3151:4011), the supplied Windows firmware updater, the macOS EPOMAKER Driver 1.0.0 package and the read-only Rust probe.

## Confirmed experimentally

- The physical keyboard and 2.4 GHz receiver are simultaneously enumerated on Linux.
- Both enumerate as Full Speed USB HID devices with three interfaces.
- Their matching report descriptors have lengths 59, 177 and 20 bytes and identical SHA-256 hashes per interface.
- Interface 0 is a boot keyboard; interface 1 provides extended keyboard/consumer/system/mouse/vendor input reports; interface 2 declares one 64-byte vendor Feature report.
- All USB endpoints are IN endpoints. No interrupt OUT endpoint is declared.
- The read-only probe enumerated six unique HID interface paths and exported a JSON diagnostic snapshot.
- A one-second input recorder smoke test opened only the standard keyboard interface, received zero reports during the idle interval and closed successfully. No keystrokes were generated for the test.
- In this Phase 3 pass, `th80ctl list` enumerated all six HID interface paths for `3151:4010` and `3151:4011`; `th80ctl descriptors` completed and produced a 168-line report.
- `th80ctl diagnostics /tmp/th80-phase3-diagnostics.json` exported a valid JSON snapshot containing six interface records across both device IDs. No Feature/Output report or control transfer was performed.
- `cargo test --locked` passed all three report-descriptor parser tests.

## Confirmed by static binary analysis

- The Windows updater is a Rust PE32+ program. It contains HID feature/input-report APIs and identifies the physical 3151:4010 vendor interface as page 0xffff, usage 0x02, interface 2. Its bootloader match is PID 0x4001, page 0xff01, usage 0x01.
- The macOS DMG contains an Electron UI and a Rust Mach-O native driver iot_driver.
- The macOS native support configuration identifies 3151:4010 interface 2 as vendor HID and 3151:4011 interface 2 as dongle_common.
- Disassembled macOS native code contains a generic 64-byte HID Feature control-transfer path with report ID 0, standard HID GET_REPORT/SET_REPORT request codes, interface index 0 and a 2-second timeout. This does not yet match the TH80 Pro vendor report's interface 2; its caller path remains unresolved.
- The macOS binary contains a YZW command vocabulary for revision, information, battery, reports, profiles, keyboard options, LED/sleep parameters, key matrix, key enable, macros, user pictures, audio and OS mode.
- The macOS native binary includes USB/HID and BLE modules. The presence of BLE support does not prove the observed TH80 Pro uses a particular BLE PID.
- Focused disassembly confirms the native macOS driver's generic Feature SET/GET request fields (`0x21/0x09/0x0300/0/64` and `0xa1/0x01/0x0300/0/64`, 2-second timeout). The literal `wIndex=0` conflicts with the physical vendor Feature report's interface number 2; no live Feature request was made.
- The native macOS support configuration marks receiver `3151:4011` as `dongle_common=true`, and the binary contains separate `YZWKeyboard` and `YZWDangleCommon` API classes.
- The Electron bundle contains 64-byte YZW command buffers with opcode at offset 0 and a `YZW3151` catalogue entry for `3151:4002`. This is not a direct model match for the TH80 Pro `3151:4010` and is not treated as proof of its packet layout.
- The Windows updater's HID function names are runtime-resolved from `hid.dll`; its support config matches `3151:4010` interface 2 and bootloader `3151:4001`, but does not list receiver `3151:4011`.
- PE disassembly recovered an actual `HidD_SetFeature` wrapper (VA `0x140615b30`) and a compiled caller (VA `0x14022b510`) that passes through a caller-provided buffer and length. Its connection to a named YZW upgrade phase is still unresolved. `HidD_GetFeature` and `HidD_GetInputReport` are resolved but no calls through those pointers were found in the examined disassembly.
- The macOS native binary uses IOKit/IOHIDManager for HID enumeration and input callbacks. Its one recovered `IOHIDDeviceSetReport` call is report type 1 (Output); no IOHID Feature GET/SET path was found. Feature control transfers use the separate libusb path.
- Both native `YZWKeyboard::get_cmd_list` and `YZWDangleCommon::get_cmd_list` return the same static `YZW_CMD_LIST` object, although the receiver class has additional light-type methods. Shared command metadata does not establish identical behavior or storage ownership.

## Confirmed by vendor documentation

- The supplied updater guide instructs the user to close the driver, disconnect the 2.4 GHz receiver, switch to wired mode and connect the keyboard by cable before updating.
- The updater configuration contains timing delays of 1000/1000/20/100 ms.

## Hypotheses

- The receiver's dongle_common path may implement receiver-side settings or device communication. This is inferred from the support configuration label only.
- The keyboard and receiver may share a common vendor Feature report layout while implementing different command semantics or storage ownership.

## Unknown

- MCU, PCB revision, matrix wiring, wireless chipset, separate RGB controller and flash layout.
- Whether 3151:4012/4013 BLE entries correspond to this exact TH80 Pro hardware.
- Which settings persist in the keyboard, receiver or host application.
- Exact vendor command packet structure, response framing, checksums and command applicability by PID.
- Whether the macOS driver's generic `wIndex=0` transfer routine is actually used for the interface-2 TH80 Pro support entry, or whether another call path exists.
- Whether any YZW command or payload layout from the `3151:4002` frontend profile applies to the TH80 Pro `3151:4010` or receiver `3151:4011`.
- Actual firmware update packet format, checksum/signature rules, flash addresses and recovery procedure.
- Whether a complete firmware/configuration backup is possible and restorable.

## Vendor application traffic

No vendor configuration operation was run, and no USB traffic was captured. The current Linux session has tshark/wireshark installed but cannot access usbmon or USB tracing interfaces. The macOS app cannot be run natively in this environment; the supplied Windows executable is an updater. This phase therefore has static transport analysis but no dynamic configuration trace.

## Project and reference review

- Vial/VIA are protocol ecosystems that require matching firmware support. The TH80 Pro has not been shown to implement them.
- Vialite is a frontend reference; its WebHID access does not make it compatible with an unrelated vendor protocol.
- The current architecture should keep standard keyboard input with Linux HID and put any eventual vendor support in a separate userspace backend.
- Firmware replacement remains blocked by missing MCU/PCB identity and absent restorable backup/recovery evidence.

## Artifacts

- HARDWARE.md
- USB_PROTOCOL.md
- EP0_PROTOCOL.md
- HID_PROTOCOL_ANALYSIS.md
- UPDATER_REVERSE_ENGINEERING.md
- FIRMWARE_ANALYSIS.md
- COMPATIBILITY.md
- SAFETY.md
- docs/hid-descriptors.md
- docs/reverse-engineering.md
- docs/firmware-roadmap.md
- src/main.rs

## Next work

Collect a passive capture from the official configuration application on a supported OS, correlate 4010 and 4011 paths, and identify the MCU from PCB photographs. Do not add vendor Feature commands to the CLI until their packet structures and side effects are independently understood.
