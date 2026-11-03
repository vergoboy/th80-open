# TH80 Pro hardware inventory

## Evidence labels

- **[Physical observation]** Read from the connected Linux host or its USB sysfs descriptors.
- **[Static binary]** Extracted from the supplied Windows updater or macOS driver without executing it.
- **[Vendor documentation]** Stated in the supplied updater guide or package configuration.
- **[Hypothesis]** A plausible interpretation that still needs independent confirmation.
- **[Unknown]** Not established by the available evidence.

## Connected devices

| Property | Physical keyboard | 2.4 GHz receiver |
|---|---|---|
| VID:PID | 3151:4010 | 3151:4011 |
| Manufacturer | ROYUAN | ROYUAN |
| Product | EPOMAKER TH80 Pro | 2.4G Wireless Keyboard |
| bcdDevice | 0x0505 | 0x0502 |
| USB speed | Full Speed, 12 Mbps | Full Speed, 12 Mbps |
| Configurations | 1 | 1 |
| Interfaces | 3 | 3 |
| Serial string | Not exposed | Not exposed |

**[Physical observation]** Both devices were present simultaneously. Their interface classes, endpoint layouts and corresponding report descriptor SHA-256 hashes match. The device release fields are USB bcdDevice values; they are not confirmed firmware versions.

## Interface topology

| Interface | Class / subclass / protocol | HID report descriptor | Endpoint | Transfer | Max packet | Interval |
|---:|---|---:|---|---|---:|---:|
| 0 | 03 / 01 / 01 (boot keyboard) | 59 bytes | 0x81 IN | Interrupt | 8 bytes | 1 ms |
| 1 | 03 / 00 / 00 | 177 bytes | 0x82 IN | Interrupt | 16 bytes | 1 ms |
| 2 | 03 / 00 / 00 | 20 bytes | 0x83 IN | Interrupt | 8 bytes | 1 ms |

**[Physical observation]** These values were read from lsusb -v and /sys/bus/usb/devices. All three endpoints are IN endpoints. There is no interrupt OUT endpoint on any interface. HID Output and Feature reports therefore use the default control pipe (EP0), as required by HID when no interrupt OUT endpoint is present.

The interface-2 endpoint `0x83 IN` is an 8-byte interrupt endpoint and is separate from the interface-2 64-byte Feature report. The interface-2 report descriptor declares no Input report item, so the endpoint's presence does not make it the transport for the Feature report. Feature access is through EP0 control transfers.

The interface 2 interrupt-IN endpoint exists in the USB configuration, but its report descriptor declares no Input report for that interface. Its descriptor declares a Feature report only.

## Report descriptor summary

The descriptors are byte-for-byte identical between the keyboard and receiver. Report sizes below are payload sizes from the HID descriptor; numbered reports also carry their report-ID byte on the interrupt endpoint.

| Interface | Report ID | Type | Payload | Host-visible input length |
|---:|---:|---|---:|---:|
| 0 | implicit 0 | Input | 64 bits | 8 bytes |
| 0 | implicit 0 | Output | 8 bits | Control transfer |
| 1 | 1 | Input | 120 bits | 16 bytes including ID |
| 1 | 2 | Input | 8 bits | 2 bytes including ID |
| 1 | 3 | Input | 16 bits | 3 bytes including ID |
| 1 | 5 | Input | 24 bits | 4 bytes including ID |
| 1 | 6 | Input | 56 bits | 8 bytes including ID |
| 2 | implicit 0 | Feature | 512 bits | 64-byte control payload |

Interface 1 does not declare Report ID 4. Interface 0 and interface 2 have no Report ID item; HIDAPI represents an unnumbered report with a leading zero byte in its API buffer. That API convention does not add a byte to the 64-byte USB Feature payload.

## Controller and PCB

- **[Unknown]** MCU manufacturer, part number, package, flash size and boot ROM.
- **[Unknown]** PCB revision and whether the supplied ANSI V5.04 package matches this exact board.
- **[Unknown]** Wireless chipset, matrix wiring, RGB controller, encoder circuitry and any separate nonvolatile configuration storage.
- **[Unknown]** Whether the receiver has independently updateable firmware or stores paired-device state/configuration.

No PCB photographs or readable controller markings have been supplied in this phase.

## Evidence files

- docs/hid-descriptors.md: raw descriptor bytes and hashes.
- USB_PROTOCOL.md: decoded report structure and control-transfer details.
- HID_PROTOCOL_ANALYSIS.md: protocol evidence and confidence labels.
- docs/firmware-roadmap.md: firmware identification prerequisites.
- EP0_PROTOCOL.md: static Feature transfer analysis and unresolved interface-index mismatch.
- UPDATER_REVERSE_ENGINEERING.md: Windows updater HID and bootloader support evidence.
- FIRMWARE_ANALYSIS.md: supplied firmware package inventory and unknowns.
