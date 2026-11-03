# Hardware safety policy

## Current guarantees

The th80ctl tool currently provides device enumeration, HID report-descriptor reads from sysfs, diagnostic export and an explicit standard keyboard Input-report recorder. It does not implement:

- HID Feature-report reads or writes.
- HID Output-report writes.
- Arbitrary USB control transfers.
- Interface detachment or kernel-driver unbinding.
- Bootloader entry or firmware flashing.
- Flash erase, configuration reset or pairing changes.

The one-second recorder smoke test received zero reports and did not change device state. No vendor application action or vendor command was issued.

During Phase 3, the CLI's `list` and `descriptors` commands were run against the currently connected keyboard and receiver. They enumerated six HID interface paths and read descriptor data only. `cargo test --locked` passed all three parser tests. No Feature GET/SET, Output report, arbitrary control transfer, or input-recorder session was performed during this pass.

## Input capture privacy

record-input records raw keyboard input bytes and timestamps. It can contain the user's typed text. It is opt-in, bounded to 1–300 seconds and writes only to the caller-selected file. Treat recordings as sensitive data and delete or protect them after analysis.

## Prohibited during protocol discovery

- Do not send undocumented Feature or Output reports.
- Do not experiment with SET_* command names extracted from binaries.
- Do not invoke reset, profile, keymap, macro, LED, sleep, audio or OS-mode commands directly.
- Do not enter bootloader mode or change USB mode to test PID transitions.
- Do not flash, erase, or modify keyboard or receiver firmware.
- Do not detach the Linux HID driver from standard keyboard interfaces.
- Do not interrupt normal typing or receiver operation for a capture.

## Passive capture rules

Capture only the vendor application's normal initialization and user-requested configuration actions. Keep the original capture immutable. Record application version, device PID, connection mode and exact UI action timestamps. Avoid firmware update, reset, pairing and factory service pages. Classify USB setup packets before decoding report bytes.

## Firmware safety gates

Firmware writes must remain unavailable until all of these are demonstrated:

1. Exact PCB revision and MCU identified.
2. Exact matching image and address map validated.
3. Checksum/signature rules understood.
4. Full original firmware and configuration backup proven complete.
5. Backup restoration demonstrated on recoverable hardware.
6. Bootloader entry, acknowledgements, power-loss handling and recovery verified.
7. Keyboard and receiver firmware ownership clearly separated.

No complete firmware backup or recovery capability has been established. Firmware replacement is currently unsupported.

The macOS native driver's generic Feature functions use `wIndex=0`, despite the TH80 Pro vendor Feature report being on interface 2. This is an unresolved static mismatch; it is not permission to try alternative indices or requests on the physical device.
