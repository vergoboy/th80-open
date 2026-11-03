# Phase 5C — Windows updater Feature correlation

Date: 2026-11-01  
Scope: static analysis of the supplied V504 Windows updater. The executable was not run.

## Executive result

- **CONFIRMED:** The updater is a Rust PE32+ x86-64 GUI executable. Its CodeView record points to `rust_hid.pdb` (GUID `{C215AEE3-87E2-43DB-B791-B1A9DD29DCEB}`, age 55) from `C:\Users\Windows\Documents\rust-hid\target\release\deps\rust_hid.pdb`.
- **CONFIRMED:** It dynamically resolves `HidD_SetFeature` from `hid.dll`; a wrapper at VA `0x140615b30` calls the resolved function pointer. A caller routine at VA `0x14022b510` calls that wrapper at `0x14022b52e`.
- **CONFIRMED:** The updater's HID support configuration lists normal-mode keyboard `3151:4010` on interface 2 and bootloader PID `3151:4001` with usage page `0xff01`, usage `0x01`, wildcard interface. Receiver `3151:4011` is not listed.
- **STRONGLY SUPPORTED:** The updater includes a firmware-upgrade subsystem that uses the HID/device layer; embedded source paths include `src/data_source/yzw_upgrade.rs`, `src/hid/dj_device.rs`, and `src/egui_ui/yzw_upgrade.rs`, alongside YZW/bootloader types and the Feature API.
- **UNKNOWN:** Whether the recovered `HidD_SetFeature` caller is on the TH80 V504 update path, which exact `YZWBoot` function calls it, what buffer it sends, its length, Report ID, header, checksum, chunking, response behavior, and how it compares byte-for-byte with the macOS 64-byte Feature payload.

## Artifact identity

`../TH80_ANSI_Firmware_V504/TH80 ANSI Firmware V504(ID 742)/TH80 Upgrade Firmware for v5xx.exe`

- PE32+ x86-64 Windows GUI; size 10,274,816 bytes.
- SHA-256: `e9f3d8d2f3160be5e3817fb4eca8efe954cc854426351e3d0d21e73f39fce0fc`.
- PE image base `0x140000000`; `.text` RVA `0x1000`, raw offset `0x400`; `.rdata` RVA `0x635000`, raw offset `0x633c00`.
- CodeView PDB reference: `rust_hid.pdb`, GUID `{C215AEE3-87E2-43DB-B791-B1A9DD29DCEB}`, age 55. No matching PDB file is present in the supplied workspace.

## HID API resolution and call chain

The PE import table includes SetupAPI, `CreateFileA/W`, `ReadFile`, `WriteFile`, `DeviceIoControl`, `GetProcAddress`, and `LoadLibrary*`. `hid.dll` is dynamically loaded; the HID API names are not ordinary static imports.

Previously recovered static addresses, retained here:

| Function / object | VA | Finding |
|---|---:|---|
| HID API resolver | `0x140615490` | Loads `hid.dll`, resolves function names through `GetProcAddress`. |
| Resolved `HidD_SetFeature` function pointer slot | `0x140970810` | Populated by resolver. |
| Resolved `HidD_GetFeature` function pointer slot | `0x140970818` | Populated by resolver; no active call through it recovered in prior pass. |
| `HidD_SetFeature` wrapper | `0x140615b30` | Calls resolved pointer with device handle, buffer pointer and caller-supplied length. On success returns supplied length; on failure follows `GetLastError`/error-formatting path. |
| Caller routine | `0x14022b510` | Calls wrapper at `0x14022b52e`, passes through handle, buffer and requested length, then checks returned length and builds a result/error state. |

This establishes a real `HidD_SetFeature` call path in the executable. It does not by itself prove that this caller belongs to the YZW V504 upgrade operation.

The string table contains `HidD_SetFeature`, `HidD_GetFeature`, `HidD_GetInputReport`, `hid_send_feature_report`, and diagnostic text such as `Send Feature Report DeviceIoControl` / `Send Feature Report GetOverLappedResult`. String presence alone is not evidence of an executed call. The `HidD_GetFeature` pointer was resolved, but the prior disassembly pass did not recover an active call through it.

## Device selection and upgrade context

The adjacent `support_config.toml` contains:

| Mode | VID:PID | Usage page | Usage | Interface |
|---|---|---:|---:|---:|
| Normal keyboard | `3151:4010` | `0xffff` | `0x02` | 2 |
| Bootloader | `3151:4001` | `0xff01` | `0x01` | `-1` wildcard |

There is no `3151:4011` receiver entry. **CONFIRMED.** This is consistent with an updater targeting the keyboard and a distinct bootloader identity, but does not prove the exact transition sequence.

The binary includes source-path strings:

- `src/data_source/yzw_upgrade.rs`
- `src/egui_ui/yzw_upgrade.rs`
- `src/hid/dj_device.rs`
- `src/hid/detect_hid.rs`
- `src/hid/windows_bind.rs`

It also contains `YZWBoot`, `YZWDev`, `SupportDev`, `UpgradeConfig`, `upgrade_firmware`, `upload_firmware`, `devId`, `version`, and `lowestAppVersion`. Selection ID 0 is the visible `yzw 升级` UI mode; hidden selections include `fw upload`, `cmd test`, and others. These are architecture/context clues, not protocol command identifiers.

**STRONGLY SUPPORTED:** The application has a YZW upgrade subsystem built on its HID layer. **UNKNOWN:** The exact call edge from `upgrade_firmware` / `YZWBoot` to the caller at `0x14022b510`.

## Packet comparison with macOS

| Property | macOS generic Feature path | Windows updater |
|---|---|---|
| API | `libusb_control_transfer` | dynamically resolved `HidD_SetFeature` |
| Report type / ID | Feature, ID 0 (`wValue=0x0300`) | Abstracted by HID API; ID not recovered |
| Transfer length | 64 bytes | Caller passes a length, exact value at relevant YZW caller unknown |
| Data buffer | Caller buffer, first 64 bytes passed unchanged by low-level SET method | Caller buffer contents not recovered |
| Interface | `wIndex=0` hardcoded | Windows HID path matched to interface 2 in normal support config; actual opened handle path/call-specific interface not yet proven |
| Checksum | Native `dangledevtype` branch can insert complemented additive checksum at offsets 7 or 8, or skip | No packet checksum/layout recovered |
| Firmware semantics | Generic Feature transport used by macOS driver; command semantics unresolved | Firmware upgrade subsystem exists; exact SetFeature caller relation to update unknown |

There is currently **no evidence sufficient to claim packet equivalence**. The common HID Feature concept and shared VID/PID are not enough: the macOS code uses a fixed 64-byte Feature ID 0 transfer, while the Windows call's report ID, length and payload are still unknown. The updater may be sending bootloader-specific packets that differ from the macOS configuration protocol.

## Remaining unknowns

1. Exact caller chain from `0x14022b510` to the YZW firmware-upgrade operation.
2. Actual caller buffer allocation and writes before the `HidD_SetFeature` call.
3. API buffer length and leading report-ID byte.
4. Any per-packet header, command, address, sequence, payload length, checksum/CRC or acknowledgement.
5. Whether this SetFeature call targets normal VID:PID `3151:4010`, bootloader PID `3151:4001`, or another supported device.
6. Whether a different HIDAPI wrapper / Feature helper is used for firmware chunks.
7. Whether any updater Feature packet matches the macOS Feature report envelope or checksum layout.

## Conclusion and safety

The Windows updater contains a real dynamically resolved `HidD_SetFeature` call path, and the same executable contains the YZW upgrade subsystem and TH80 Pro support configuration. Their exact call-graph connection and packet bytes are not recovered. Therefore updater Feature packets **cannot currently be correlated** with the macOS 64-byte Feature implementation.

The updater was not executed. No HID open, Feature GET/SET, DeviceIoControl, USB transfer, interface claim, reset, configuration change, bootloader operation, or firmware operation was performed.
