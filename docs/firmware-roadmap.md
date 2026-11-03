# Firmware and Linux driver roadmap

## Future host-side hardware integration (not implemented)

TH80 Studio currently edits local profiles only. No hardware transport is implemented or included in the GUI dependency tree. Any future work is blocked until the device identity, firmware compatibility, protocol semantics and recovery path are independently established. The following is a gated research sequence, not a current product feature or authorization to perform device operations.

Use a userspace HID stack. Standard keyboard input should remain with the Linux HID/input subsystem. A separate `th80d` service can claim only the vendor-specific HID interface once its exact interface and report descriptors are confirmed. A CLI and desktop UI can communicate with that service over a local Unix socket or D-Bus.

Initial feature sequence:

1. Enumerate the device and preserve USB/HID descriptor data.
2. Capture passive input reports and identify vendor interfaces without detaching the standard keyboard interface.
3. Discover read-only identification, status, and battery commands.
4. Map lighting and key configuration commands with controlled request/response captures.
5. Implement macro/keymap writes with validation, backups, and read-back verification.
6. Add firmware update support only after the bootloader protocol and recovery process are independently verified.

## Firmware replacement

The supplied updater is not firmware source code. It provides device matching, timing values, and clues about the bootloader identity, but does not identify the MCU or expose the complete update protocol. A replacement firmware project needs at minimum:

- Clear photographs of both PCB sides and readable MCU markings.
- Exact model, layout, wired/2.4 GHz/Bluetooth variant, and current firmware version.
- MCU datasheet and toolchain/SDK availability.
- Matrix continuity map and peripheral inventory.
- Bootloader entry and recovery behavior.
- A legally redistributable firmware baseline or a clean-room implementation specification.

Once the MCU is identified, evaluate an existing open firmware platform (for example QMK, ZMK, or Embassy-based Rust) against the actual MCU, wireless requirements, RGB/OLED hardware, and bootloader. Do not select a platform based on the product name alone.

## Firmware flashing safety gates

Before enabling any flash command, require all of the following:

- Exact board/firmware compatibility check.
- Parsed image with known address ranges and length limits.
- Verified checksum/signature behavior where applicable.
- Explicit bootloader handshake and acknowledgement parsing.
- Power-loss/reconnect recovery procedure tested on a recoverable board.
- A preserved, hash-identified original image and vendor recovery package.

Until those conditions are met, tooling remains read-only.
