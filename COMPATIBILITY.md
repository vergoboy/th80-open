# Compatibility matrix

| Device / mode | Evidence | Current support status |
|---|---|---|
| TH80 Pro wired USB 3151:4010 | Live USB enumeration and descriptors | **Detected**; read-only identification and descriptor capture supported |
| TH80 Pro standard keyboard input | Live interface 0 descriptor and one idle recorder smoke test | **Read-only input capture available**; ordinary Linux HID keyboard path remains in use |
| TH80 Pro extended input | Live interface 1 descriptor | **Descriptor documented**; raw input recording currently targets boot-keyboard interface only |
| TH80 Pro vendor interface 2 | Live descriptor; Windows/macOS static support entries | **Generic macOS Feature request fields recovered, but `wIndex=0` conflicts with interface 2**; no protocol operation executed |
| ROYUAN receiver 3151:4011 | Live USB enumeration; macOS support entry `dongle_common=true` | **Detected**; descriptor capture supported; separate native driver path exists; command semantics unknown |
| Bluetooth mode | BLE code and unrelated-looking BLE IDs in macOS driver | **Unverified for this TH80 Pro revision** |
| Bootloader 3151:4001 | Windows updater config | **Static identifier only**; not connected or tested |
| ANSI V5.04 firmware package | Supplied package and updater | **Not verified compatible** with the connected board; do not flash |
| QMK / ZMK / Vial | No MCU or compatible firmware evidence | **Unsupported / unproven** |

## Feature status

| Feature | Current status | Evidence needed to implement |
|---|---|---|
| Device identification | Read-only supported | Additional hardware revisions for broader compatibility |
| HID descriptors | Read-only supported | None for current USB revision |
| Keyboard input | Linux HID standard interface | No custom backend required for normal typing |
| Battery query | Static command name only | Passive vendor-app capture and response mapping |
| RGB configuration | Static command names / UI vocabulary only | Controlled passive trace and payload decode |
| Key remapping | Static command names / UI vocabulary only | Controlled passive trace, keymap model and read-back validation |
| Macros | Static command names / UI vocabulary only | Controlled passive trace and payload mapping |
| Sleep settings | Static command names / UI vocabulary only | Controlled passive trace and mode-specific validation |
| Profiles | Static command names / UI vocabulary only | Determine whether profile state lives on keyboard, receiver or host |
| Firmware backup | Not established | Verified complete readout and integrity comparison |
| Firmware flashing | Disabled / unsupported | Exact MCU, matching image, verified boot protocol and tested recovery |

No feature should be presented as functional in a GUI until its protocol and behavior have been tested on the matching hardware revision.

The Electron bundle's generic `YZW3151` frontend profile targets PID `0x4002`, not the connected TH80 Pro PID `0x4010`. Its command buffer layout and UI semantics are not accepted as device compatibility evidence for this keyboard without a verified native caller mapping or passive capture.
