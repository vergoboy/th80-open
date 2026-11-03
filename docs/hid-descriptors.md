# Captured HID report descriptors

Captured read-only from /sys/class/hidraw/hidraw*/device/report_descriptor on the connected Linux host. These are descriptor bytes, not traffic captures. The TH80 Pro and its 2.4 GHz receiver expose identical descriptors.

## TH80 Pro `3151:4010`

| HID interface | Descriptor size | Linux HID collection |
|---:|---:|---|
| 0 | 59 bytes | Keyboard input |
| 1 | 177 bytes | Consumer/system/keyboard/mouse/vendor input collections |
| 2 | 20 bytes | Vendor-defined feature report, usage page `0xffff`, usage `0x02`, 64-byte payload |

## Descriptor hashes

| Interface | SHA-256 |
|---:|---|
| 0 | a5395cb7e86bebce11aecb4cc6d73d5d2b23d1c63f28df3c7c90069490213dd8 |
| 1 | 3cc697106805cc7be08443be8dae9f4eda2069d7b5c8352e03abdea7e5a98c74 |
| 2 | 350fea62e225e3d92e9db3bbbd408fcebe386c10405454be661d3d2c3229b0b5 |

## USB endpoints (both devices)

| Interface | Class/subclass/protocol | Endpoint | Direction | Type | Max packet | Interval |
|---:|---|---|---|---|---:|---:|
| 0 | 03/01/01 | 0x81 | IN | Interrupt | 8 | 1 ms |
| 1 | 03/00/00 | 0x82 | IN | Interrupt | 16 | 1 ms |
| 2 | 03/00/00 | 0x83 | IN | Interrupt | 8 | 1 ms |

No interrupt OUT endpoint is present. HID Output and Feature reports use EP0 control transfers.

Interface 0:

```text
05 01 09 06 a1 01 05 08 15 00 25 01 19 01 29 05
75 01 95 05 91 02 95 03 91 01 05 07 19 e0 29 e7
75 01 95 08 81 02 95 08 81 01 15 00 26 ff 00 19
00 2a ff 00 75 08 95 06 81 00 c0
```

Interface 1:

```text
05 0c 09 01 a1 01 85 03 19 00 2a 3c 03 15 00 26
3c 03 95 01 75 10 81 00 c0 05 01 09 80 a1 01 85
02 05 01 19 81 29 83 15 00 25 01 95 03 75 01 81
02 95 01 75 05 81 01 c0 05 01 09 06 a1 01 85 01
05 07 15 00 25 01 19 00 29 77 95 78 75 01 81 02
c0 05 01 09 02 a1 01 85 06 09 01 a1 00 05 09 15
00 25 01 19 01 29 05 75 01 95 05 81 02 95 03 81
01 05 01 16 01 80 26 ff 7f 09 30 09 31 75 10 95
02 81 06 15 81 25 7f 09 38 75 08 95 01 81 06 05
0c 0a 38 02 95 01 81 06 c0 c0 06 ff ff 09 01 a1
01 85 05 09 01 15 00 26 ff 00 75 08 95 03 81 02
c0
```

Interface 2:

```text
06 ff ff 09 02 a1 01 09 02 15 80 25 7f 95 40
75 08 b1 02 c0
```

## 2.4 GHz receiver `3151:4011`

The three interface descriptors are byte-for-byte identical to the corresponding 3151:4010 descriptors on this host. The receiver has the same three interface numbers and corresponding report descriptor layout.

## Decoded report structure

| Interface | Report ID | Type | Payload | Notes |
|---:|---:|---|---:|---|
| 0 | implicit 0 | Input | 8 bytes | Boot keyboard, modifiers and six-key array |
| 0 | implicit 0 | Output | 1 byte | Five LED bits and three padding bits; EP0 |
| 1 | 1 | Input | 15 bytes | Keyboard bitmap; 16 bytes including ID |
| 1 | 2 | Input | 1 byte | System control; 2 bytes including ID |
| 1 | 3 | Input | 2 bytes | Consumer control; 3 bytes including ID |
| 1 | 5 | Input | 3 bytes | Vendor-defined input; 4 bytes including ID |
| 1 | 6 | Input | 7 bytes | Mouse; 8 bytes including ID |
| 2 | implicit 0 | Feature | 64 bytes | Vendor page 0xffff, usage 0x02; EP0 |

Interface 1 has no Report ID 4. Interface 2 has no Input or Output report item despite exposing an interrupt-IN endpoint. Physical ranges are not declared in any of these report descriptors.

## Interpretation limits

- HIDAPI represents an unnumbered Feature report with a leading report-ID byte (zero) in its caller buffer. That API buffer can be 65 bytes while the USB report payload remains 64 bytes.
- Static disassembly of the supplied macOS native driver contains GET_REPORT and SET_REPORT control transfers with report ID 0 and a 64-byte length. The observed generic routine uses wIndex 0, which does not match the TH80 Pro vendor report's interface 2; its caller path remains unresolved. See USB_PROTOCOL.md.

- Descriptor structure identifies report types and lengths, not command opcodes or semantics.
- The vendor feature report's 64-byte payload does not reveal whether commands use a leading command byte, subcommand, checksum, sequence number, or response framing.
- No feature report was read or written during this capture.
- The host's `bcdDevice` values are USB release descriptors; they should not be assumed to be firmware version strings without corroboration.
