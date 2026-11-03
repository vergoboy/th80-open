# Phase 4.2 — DJDeviceInfo metadata and SupportDev provenance

Date: 2026-11-01  
Binary: macOS x86-64 `iot_driver` from EPOMAKER Driver 1.0.0

## Binary provenance and address validation

Exact analyzed executable:

`../.phase3-work/mac/EPOMAKER Driver 1.0.0/EPOMAKER Driver.app/Contents/Resources/app/iot_driver`

- SHA-256: `22c5dc67ba77e2373a5ffdbb0312ed8c8e73eb16e78037764eb676a030b2724b`
- Size: `6,075,584` bytes
- Format: x86-64 Mach-O, PIE, base used for these VM addresses: `0x100000000`
- Only one `iot_driver` copy was found under `.phase3-work`.
- No IDA/Ghidra database was identified; analysis artifacts are `/tmp/th80_iot.dis` and the focused `.asm` files documented in the Phase 4.2 handoff.

The following VM addresses were re-disassembled from this exact fingerprinted file: `0x1000c8b00`, `0x1000c8ea0`, `0x100048290`, and `0x100048460`. These are virtual addresses, not file offsets.

For the Mach-O `__TEXT` segment (`vmaddr=0x100000000`, `fileoff=0`), the corresponding file offsets are `0xC8B00`, `0xC8EA0`, `0x48290`, and `0x48460`, respectively.

## Executive result

The fourth argument to `DJDeviceInfo::from_hid_info` is a stack copy of the currently matched 16-byte `SupportDev` record, not the global `SUPPORTDEVLIST` object.

The match is performed by `HidSender::get_support_dev` at `0x100036f60`. The function iterates 16-byte records and compares interface, VID, PID, usage and usage page against the enumerated HID `DeviceInfo`.

The constructor at `0x1000c8b00` writes the actual HID descriptor interface number to `DJDeviceInfo+0x28`. It derives `+0x39` and `+0x3a` from the copied support-record tail, with a special case when `report_id == 2`.

## SupportDev layout

The 16-byte stride is explicit in the matcher (`shl $4` and increments by `0x10`). The matching reads and embedded serde field names establish:

| Offset | Field | Width / representation |
|---:|---|---|
| `+0x00` | `interface_number` | i32 |
| `+0x04` | `vid` | u16 |
| `+0x06` | `pid` | u16 |
| `+0x08` | `usage` | u16 |
| `+0x0A` | `usage_page` | u16 |
| `+0x0C` | `report_id` | u8-like selector |
| `+0x0D` | padding | 1 byte |
| `+0x0E` | `dongle_common` | one-byte optional flag representation |
| `+0x0F` | `ble` | one-byte optional flag representation |

The field-name visitor is at `0x10011e920`; field ordinals are: vid=0, pid=1, usage=2, usage_page=3, interface_number=4, report_id=5, dongle_common=6, ble=7.

The support config's `dongle_common` and `ble` values are optional. The one-byte representation is consistent with Rust `Option<bool>` niche encoding: `Some(false)=0`, `Some(true)=1`, `None=2`. The SupportDev sequence visitor includes an `Option<bool>` element type. This representation explains the manager's comparisons against 2. The exact source-level Rust declaration is not available; the representation/type inference is based on serde visitor and field-layout evidence.

## Exact matching flow

Function: `HidSender::get_support_dev`, `0x100036f60`.

For each 16-byte SupportDev record, it compares:

| Record offset | HIDAPI getter | Comparison |
|---:|---|---|
| `+0x06` | `DeviceInfo::product_id` (`0x1001bef20`) | u16 equality |
| `+0x04` | `DeviceInfo::vendor_id` (`0x1001bef10`) | u16 equality |
| `+0x08` | `DeviceInfo::usage` (`0x1001bef40`) | u16 equality |
| `+0x0A` | `DeviceInfo::usage_page` (`0x1001bef30`) | u16 equality |
| `+0x00` | `DeviceInfo::interface_number` (`0x1001bef50`) | i32 equality |

On a match, the function clones the HIDAPI DeviceInfo and copies both qwords of the matched SupportDev record into a local 16-byte stack copy (`0x1000371be..0x1000371cb`). It passes the local copy's address in RCX to `from_hid_info` at `0x1000371e6`.

There are three call sites within this matching routine, with caller-supplied class values 1, 0 and 2 at `0x1000371e6`, `0x100037421`, and `0x1000375ac`, respectively. They all pass the copied record pointer. Do not collapse these distinct enumeration/class paths.

## from_hid_info arguments and writes

Function: `DJDeviceInfo::from_hid_info`, `0x1000c8b00`.

The x86-64 SysV argument evidence at the caller establishes:

- RDI: output `DJDeviceInfo` destination.
- RSI: cloned `hidapi::DeviceInfo`.
- EDX: caller-supplied class/type (observed values 1, 0, 2 across three call sites).
- RCX: pointer to the copied 16-byte SupportDev record.

The function obtains the actual interface number by calling `hidapi::DeviceInfo::interface_number` and stores it at `DJDeviceInfo+0x28` (`0x1000c8c35..0x1000c8c3a`). The support record's interface number is used for matching, but the value written into +0x28 comes from the HIDAPI descriptor getter.

It writes the caller-supplied class/type at `+0x38`.

Metadata derivation:

```text
if support.report_id == 2:
    info[0x39] = 2
    info[0x3a] = 2
else:
    info[0x39] = support.dongle_common
    info[0x3a] = support.ble
```

Relevant instructions: `0x1000c8bee..0x1000c8c08`; stores at `0x1000c8c61` and `0x1000c8c65`.

## Target SupportDev records and metadata

Embedded support configuration at file offset `0x3756b6` includes:

| PID | VID | Usage | Usage page | Interface | dongle_common | ble | report_id |
|---|---:|---:|---:|---:|---|---|---|
| 4010 | 3151 | 2 | 0xffff | 2 | omitted | omitted | omitted |
| 4011 | 3151 | 2 | 0xffff | 2 | true | omitted | omitted |

The target TOML omits `report_id`; the SupportDev deserializer materializes the scalar field as zero (effective default 0, not an Option sentinel). The constructor special case is exactly value 2. The omitted optional flags use the separate `Option<bool>` None sentinel 2. The optimized binary does not retain a source-level declaration, so describe `report_id` as a u8-like selector based on its one-byte layout and consumer, not as a proven Rust source type.

Derived metadata:

| Device | Support report_id | support +0x0E | support +0x0F | DJDeviceInfo +0x39 | +0x3A |
|---|---:|---:|---:|---:|---:|
| 3151:4010 | 0 / absent→0 | 2 / None | 2 / None | 2 | 2 |
| 3151:4011 | 0 / absent | 1 / Some(true) | 2 / None | 1 | 2 |

The report_id default is derived from the deserializer's omitted-field default and the record representation; if later source-level type metadata becomes available, retain that as a cross-check.

## Field provenance table

| Field | Offset | Source | Transformation | Consumers | Proven meaning |
|---|---:|---|---|---|---|
| interface number | `0x28` | HIDAPI `DeviceInfo::interface_number()` | direct i32 store | manager/device construction; HID metadata | actual enumerated HID interface number |
| class/type | `0x38` | caller's EDX argument (1/0/2 across call sites) | direct store | manager branch | enumeration class/type; exact enum labels not recovered |
| dongle-common tag | `0x39` | SupportDev `dongle_common` byte at +0x0E unless report_id=2 | copy optional-bool niche byte; special-case 2 | manager branch; DJDevice::new metadata normalization at `0x1000c90f1..0x1000c9118` | optional dongle-common flag encoding; selects YZWDangleCommon when not 2 and +0x3A is 2 |
| BLE tag | `0x3A` | SupportDev `ble` byte at +0x0F unless report_id=2 | copy optional-bool niche byte; special-case 2 | manager branch; `DJDevice::new` at `0x1000c8eba`; SET checksum selector at `0x100095bff` / `0x100095c25` | optional BLE flag encoding; transport selector input and checksum insertion-position selector |

## Manager branch and backend consequence

Manager state machine: `DJDeviceManger::refresh_dev_list`, around `0x1000c66ad..0x1000c66bd`.

- If `info[0x3A] != 2`: alternate BLE path.
- If `info[0x3A] == 2 && info[0x39] != 2`: construct `YZWDangleCommon`.
- If both are 2: normal path constructs `BleHid` around the `DJDevice` (the `BleHid` name is a Base/API wrapper, not the `HidBLE` transport).

`DJDevice::new` at `0x1000c8ea0` independently tests `info[0x3A] == 2` at `0x1000c8eba`:

- equal to 2 → `HidLibusb::new` at `0x100047d30`
- otherwise → `HidBLE::new` at `0x100035fc0`

Therefore, under the recovered SupportDev optional-field representation:

- PID 4010: tags (2,2), `BleHid` Base wrapper around DJDevice, HidLibusb transport.
- PID 4011: tags (1,2), `YZWDangleCommon` Base wrapper around DJDevice, HidLibusb transport.

## YZWDangleCommon construction

`BleHid::new` at `0x100035f20` wraps an `Arc<DJDevice>` and adds a movable mutex and poison flag; the manager calls it at `0x1000c758e` on the normal `(2,2)` path. `YZWDangleCommon::new` at `0x10005fdd0` copies a `0xf8`-byte DJDevice into its wrapper and initializes synchronization/poison state; the manager calls it at `0x1000c7647` on the `(1,2)` path. The manager calls it at `0x1000c7647` after copying the DJDeviceInfo/device object into the constructor input.

This is an **indirect** association:

`SupportDev.dongle_common → DJDeviceInfo[0x39] → manager branch → YZWDangleCommon::new`.

The flag is not passed as a direct constructor argument. It influences the manager branch through metadata.

The wrapper is not a separate USB transport: its Base Feature futures call the wrapped DJDevice Feature methods. Its purpose is a higher-level synchronized/common-protocol wrapper.

## Writes and reads

Proven writes in `from_hid_info`:

- `+0x28`: `0x1000c8c35..0x1000c8c3a`
- `+0x39`: `0x1000c8c61`
- `+0x3A`: `0x1000c8c65`

Proven consumers:

- manager branch compares `+0x39` and `+0x3A` around `0x1000c66ad..0x1000c66bd`.
- `DJDevice::new` compares `+0x3A` at `0x1000c8eba`.
- `DJDevice::new` later reads both `+0x39` and `+0x3A` at `0x1000c90f1..0x1000c9118` and normalizes nonzero tags while preserving value 2.

A complete whole-binary audit of every constructor/copy/write to these offsets remains outstanding. The confirmed critical-path stores, reads and copies are: `+0x28` is written by `from_hid_info` and copied into `HidLibusb` metadata during `HidLibusb::new` (around `0x100047f98`); `+0x39`/`+0x3A` are written by `from_hid_info`, compared by the manager, and read/normalized by `DJDevice::new` around `0x1000c90f1..0x1000c9118`; `+0x3A` is additionally read by the SET checksum state machine at `0x100095bff` and `0x100095c25`. The libusb control-transfer functions do not read `+0x28`; they use hardcoded `wIndex=0`. No claim is made that this is an exhaustive cross-binary xref census.

## Remaining uncertainty

- Source-level declaration/type spelling for `report_id`, `dongle_common`, and `ble` is unavailable; the binary representation and deserializer behavior are the evidence.
- The exact names/semantics of the three caller-supplied class values 1/0/2 are unknown.
- A complete audit of all possible writes/copies of the three requested offsets remains open.
