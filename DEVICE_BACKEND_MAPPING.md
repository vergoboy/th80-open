# Device/backend mapping — EPOMAKER macOS driver Phase 4

## Mapping model

The native driver keeps several identities separate:

1. HID enumeration identity: VID, PID, usage, usage page, interface number and device path.
2. `DJDeviceInfo`: HID identity plus support-list-derived classification/transport flags.
3. `DJDevice`: owns the selected low-level sender backend.
4. Renderer model identity: a software model ID/name used to select YZW configuration logic.
5. `dangledevtype`: a frontend RPC hint for common dongle keyboard/mouse routing.

These fields are related but not interchangeable.

## Native identity preservation

**CONFIRMED — `DJDeviceInfo::from_hid_info`, `0x1000c8b00`:** reads vendor ID, product ID, usage, usage page, interface number and path through `hidapi::DeviceInfo` accessors. The interface number is stored at record offset `0x28`.

The function also consults native `SUPPORTDEVLIST` and stores support-derived bytes at offsets `0x39` and `0x3a`. The native `SupportDev` table explicitly labels `3151:4011` as `dongle_common=true`, while the `3151:4010` entry has no such flag. The exact mapping from support-record fields to every byte in `DJDeviceInfo` is not fully recovered.

**CONFIRMED — `DJDevice::new`, `0x1000c8ea0`:** `DJDeviceInfo[0x3a] == 2` selects `HidLibusb::new`; otherwise it selects `HidBLE::new`. This is a construction-time branch.

**CONFIRMED — `HidLibusb::new`, `0x100047d30`:** enumerates libusb devices, matches VID/PID against `DJDeviceInfo`, opens the match, and copies interface/usage metadata into backend state. The interface value at metadata offset `0x28` is retained.

## Keyboard and receiver

| Field | TH80 Pro keyboard | Wireless receiver |
|---|---|---|
| Physical VID:PID | `3151:4010` | `3151:4011` |
| Reported physical role | Vendor HID | `dongle_common` |
| Physical interface | 2 | 2 |
| Native HID identity fields | Read from HID metadata | Read from HID metadata |
| Native support-list match | **CONFIRMED:** usage 2, page `0xffff`, interface 2; no `dongle_common=true` | **CONFIRMED:** usage 2, page `0xffff`, interface 2, `dongle_common=true` |
| `DJDeviceInfo` transport tag (`+0x3a`) | **UNKNOWN** for this exact identity | **UNKNOWN** for this exact identity |
| Selected `DJDevice` backend | **UNKNOWN** until tag is established | **UNKNOWN** until tag is established |
| Raw RPC → Feature transport | **UNKNOWN** | **UNKNOWN** |

The native binary embeds a `SupportDev` table. Its readable record strings explicitly include:

```text
{ vid = 0x3151, pid = 0x4011, usage = 2, usage_page = 0xffff,
  interface_number = 2, dongle_common = true }
{ vid = 0x3151, pid = 0x4010, usage = 2, usage_page = 0xffff,
  interface_number = 2 }
```

These are **CONFIRMED native support-list records** and distinguish the receiver (`dongle_common=true`) from the keyboard (no `dongle_common=true` field in its record). The same table includes BLE entries (`4013`, `4012`) explicitly marked `ble=true`, as well as bootloader entries. The receiver and keyboard are therefore both supported vendor-HID records on interface 2, but only the receiver is marked `dongle_common`.

The native binary also has a `YZWDangleCommon::new` constructor at `0x10005fdd0`, called from the device-manager refresh state machine at `0x1000c7647`. This is **STRONGLY SUPPORTED** as the high-level implementation corresponding to the `dongle_common` device family, given the explicit support record and constructor. The exact branch predicate and the mapping of that specific constructor argument to PID `4011` still need to be proven. It does not yet prove that the receiver's raw Feature RPC reaches a specific sender method.

The frontend's `supportYZWDev` TH80 model record is ID 369 and uses logical `0x0461:0x4002` values. These are renderer model metadata, not the native physical keyboard VID/PID. Do not confuse that record with the `3151:4010` physical interface record.

## `dongledevtype` and `dongle_common`

**CONFIRMED:** the renderer's raw Feature helper derives `dangledevtype` from membership in `devArr[].danglecommondev` keyboard/mouse IDs (`0` for other devices, `1` for dongle keyboard, `2` for dongle mouse). It passes this value in the gRPC request.

**UNKNOWN:** the native raw Feature handler's use of `dangledevtype` has not been recovered. No conclusion is justified that `dongle_common` changes low-level transport, command serialization, configuration storage or persistence. The shared `YZW_CMD_LIST` metadata previously found in `YZWKeyboard` and `YZWDangleCommon` is not evidence of identical execution.

## `wIndex=0` implications

The generic libusb Feature SET and GET methods hardcode `wIndex=0`; they do not derive it from the preserved interface field. The physical keyboard and receiver are both described as interface 2. Whether either physical device's real RPC path reaches those generic methods remains **UNKNOWN**.

## Required follow-up to close mapping

1. Decode the matching native `SUPPORTDEVLIST` entries for `3151:4010` and `3151:4011`, including the fields that become offsets `0x39` and `0x3a`.
2. Resolve the raw gRPC handler's concrete sender-vtable target.
3. Determine whether the receiver's `dongle_common` class is created for PID `4011` by following the manager's matching support record and constructor call at `0x1000c7647`.
4. Locate libusb interface claiming and verify whether interface 2 is claimed while Feature control requests still use `wIndex=0`.

No hardware validation is authorized or required for these static questions.
