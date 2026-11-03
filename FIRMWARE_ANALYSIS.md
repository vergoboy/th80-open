# Firmware package analysis — TH80 ANSI V5.04

## Scope

This is a static inventory of the supplied V5.04 firmware package and updater metadata. No firmware was flashed, unpacked onto the device, or used to trigger a bootloader. No firmware backup or recovery capability is claimed.

## Package contents

Directory: `TH80_ANSI_Firmware_V504/TH80 ANSI Firmware V504(ID 742)/`

| File | Size | Current finding |
|---|---:|---|
| `TH80 Upgrade Firmware for v5xx.exe` | 10,274,816 bytes | Rust PE32+ updater; see [UPDATER_REVERSE_ENGINEERING.md](UPDATER_REVERSE_ENGINEERING.md) |
| `support_config.toml` | 1,234 bytes | HID normal-mode and bootloader support rules |
| `selections_config.toml` | 539 bytes | Updater UI selection labels |
| `upgrade_config.toml` | 78 bytes | Update timing values only |
| `Upgrade Guide.pdf` | 154,048 bytes | Vendor instructions for wired update setup and reconnect after failure |
| `X` | 184,608 bytes | Raw binary data; format and target MCU not established |

## Raw payload `X`

**[Static binary]**

- Size: `184608` bytes.
- SHA-256: `4b22336ec9ffaaac3372791a1154a2c2a833e3305a4cadbbdcab9184252c0520`.
- `file` identifies it only as generic `data`; no recognized ELF/PE/Intel HEX container was established.
- It contains readable strings including `Gaming Keyboard`, `Joystick in HS mode`, `ROYUAN`, `HID Config`, and `HID Interface`, as well as source-path remnants such as `yc_gpio.c`, `yc_uart.c`, `yc_timer.c`, and `yc_systick.c`.
- The final 64 bytes inspected are `0xff` padding.

These observations are insufficient to establish the CPU architecture, memory map, image entry point, application/bootloader boundary, flash address, signature scheme, or whether the file is the exact firmware for the currently connected board.

## Update configuration

`upgrade_config.toml` records `before_start_delay=1000`, `start_delay=1000`, `per_delay=20`, and `end_delay=100` (milliseconds). These values are timing configuration, not protocol framing.

`support_config.toml` identifies normal keyboard `3151:4010` on vendor HID interface 2 and bootloader PID `3151:4001` with Usage Page `0xff01`, Usage `0x01`, interface wildcard `-1`. The receiver `3151:4011` is not listed in the Windows updater configuration.

The updater embeds HID API names and source path strings for a YZW upgrade module, but the firmware transfer sequence, block format, checksums, flash addresses, and acknowledgements remain unknown. See [UPDATER_REVERSE_ENGINEERING.md](UPDATER_REVERSE_ENGINEERING.md).

## Compatibility and recovery

**[Unknown]** The exact PCB revision and MCU in the connected TH80 Pro have not been identified. The package filename/version does not prove compatibility with the connected board. There is no demonstrated complete firmware readout, no validated backup manifest, and no tested restore procedure.

Firmware modification remains unsupported. Do not flash this package, use the updater's hidden modes, activate bootloader mode, erase flash, or experiment with undocumented commands.
