# Windows Rust firmware updater — reverse-engineering notes

## Scope and safety

The supplied updater was inspected statically. It was not executed, no firmware-update UI was opened, no bootloader was activated, and no USB command was sent. The supplied Windows updater is a firmware tool, not the macOS configuration driver.

Evidence labels are defined in [HARDWARE.md](HARDWARE.md): Physical observation, Static binary, Vendor documentation, Hypothesis, and Unknown.

## 1. Artifact identity

**[Static binary]** The executable is:

`TH80_ANSI_Firmware_V504/TH80 ANSI Firmware V504(ID 742)/TH80 Upgrade Firmware for v5xx.exe`

| Property | Value |
|---|---|
| Format | PE32+ Windows GUI executable |
| Architecture | x86-64 |
| Size | 10,274,816 bytes |
| SHA-256 | `e9f3d8d2f3160be5e3817fb4eca8efe954cc854426351e3d0d21e73f39fce0fc` |
| Language/runtime evidence | Rust paths, Rust panic/runtime strings, Tokio, Tonic, Hyper, Egui/Glium |

The adjacent `support_config.toml` SHA-256 is `95fd1124b020ceec89b987b5f4fa954f831b014e86b1c540f31d93263fca2593`.

PE sections reported by `objdump -h`:

| Section | Virtual size | RVA | File offset |
|---|---:|---:|---:|
| `.text` | `0x63366a` | `0x1000` | `0x400` |
| `.rdata` | `0x33787e` | `0x635000` | `0x633c00` |
| `.data` | `0x3600` | `0x96d000` | `0x96b600` |
| `.pdata` | `0x51894` | `0x972000` | `0x96ec00` |
| `.reloc` | `0xc13c` | `0x9c4000` | `0x9c0600` |

The current environment has GNU/LLVM object inspection utilities and `strings`, but no Ghidra or radare2 executable was available during this pass. The Rust binary does not expose ordinary Rust source-level function symbols in the inspected PE symbol output; source path strings and embedded configuration provide the main recovered architecture evidence.

## 2. Device discovery and HID API evidence

### SetupAPI imports

**[Static binary]** The PE import table includes these SetupAPI functions:

- `SetupDiGetClassDevsA`
- `SetupDiEnumDeviceInterfaces`
- `SetupDiGetDeviceInterfaceDetailA`
- `SetupDiEnumDeviceInfo`
- `SetupDiGetDeviceRegistryPropertyA`
- `SetupDiDestroyDeviceInfoList`

The executable also imports Windows file/device primitives such as `CreateFileA/W`, `ReadFile`, `WriteFile`, `DeviceIoControl`, `GetProcAddress`, and `LoadLibrary*` functions.

### HID symbols resolved at runtime

**[Static binary]** The executable contains the string `hid.dll` and function-name strings for a runtime HID API lookup table. The import table does not list `hid.dll` as a normal static import. The following names were found:

| API name | Evidence | Notes |
|---|---|---|
| `HidD_GetAttributes` | Static binary | Device identity helper |
| `HidD_GetSerialNumberString` | Static binary | Device metadata helper |
| `HidD_GetManufacturerString` | Static binary | Device metadata helper |
| `HidD_GetProductString` | Static binary | Device metadata helper |
| `HidD_SetFeature` | Static binary | Feature-report write API name |
| `HidD_GetFeature` | Static binary | Feature-report read API name |
| `HidD_GetInputReport` | Static binary | Input-report API name |
| `HidD_GetIndexedString` | Static binary | HID string helper |
| `HidD_GetPreparsedData` | Static binary | Preparsed descriptor metadata |
| `HidD_FreePreparsedData` | Static binary | Preparsed descriptor cleanup |
| `HidP_GetCaps` | Static binary | HID capability query |
| `HidD_SetNumInputBuffers` | Static binary | Input queue sizing |

The binary also contains diagnostic strings `Send Feature Report DeviceIoControl`, `Send Feature Report GetOverLappedResult`, `Send Input Report DeviceIoControl`, and `Send Input Report GetOverLappedResult`.

**Limit:** strings establish that the executable knows/resolves these API names, not that every function is called on the TH80 Pro in the normal upgrade path. The `HidD_SetFeature` call-site addresses are recovered below; the actual caller buffer contents, report IDs, API buffer lengths, and complete error branches remain unknown. No `HidD_SetOutputReport`, `HidP_GetButtonCaps`, or `HidP_GetValueCaps` name was found in the inspected string table; this is not proof that equivalent behavior is impossible through lower-level APIs.

### Resolved Feature API call path

Focused x86-64 disassembly recovered more than the API-name table:

- A resolver at VA `0x140615490` loads `hid.dll` and resolves HID entry points with the imported `GetProcAddress` pointer. The `HidD_SetFeature` pointer is stored at VA `0x140970810`; `HidD_GetFeature` is stored at `0x140970818`.
- A wrapper at VA `0x140615b30` calls the resolved `HidD_SetFeature` pointer with a device handle, caller-provided report-buffer pointer, and caller-provided length. On success it returns the supplied length; on failure it follows a `GetLastError`/error-formatting path.
- A compiled routine at VA `0x14022b510` calls that wrapper at `0x14022b52e`, passing through a handle, report buffer, and requested length, then checks the returned length and constructs a result/error state.
- The higher-level call chain from that routine to a named `YZWBoot`/upgrade operation has not yet been mapped. This proves a real `HidD_SetFeature` invocation path exists in the executable, but does not prove the exact update phase, device mode, report ID, payload, or buffer length used for TH80 Pro V5.04.
- The `HidD_GetFeature` and `HidD_GetInputReport` function pointers are resolved, but no call through either pointer was found outside the resolver in the disassembly pass. Do not describe them as confirmed active calls in the updater path.
- `HidD_GetPreparsedData` and `HidP_GetCaps` do have call sites in the device-capability path; `HidD_FreePreparsedData` is used to release the preparsed data.

The Windows HID API abstracts the underlying control request. No exact Windows-side `bmRequestType`, `bRequest`, `wValue`, `wIndex`, report ID, or report-buffer length is recovered here.

## 3. Embedded support configuration

The adjacent `support_config.toml` is directly inspectable and is also reflected in updater strings.

### Normal-mode support

| VID:PID | Usage Page | Usage | Interface | Static role implied by config |
|---|---:|---:|---:|---|
| `3151:4002` | `0x0001` | `0x06` | 0 | Keyboard HID |
| `3151:4003` | `0x0001` | `0x06` | 0 | Keyboard HID |
| `3151:4007` | `0xffff` | `0x02` | 2 | Vendor HID |
| `3151:4010` | `0xffff` | `0x02` | 2 | TH80 Pro vendor HID |

The file also contains entries for other vendors/products. It does **not** contain receiver PID `0x4011`.

### Bootloader support

| VID:PID | Usage Page | Usage | Interface selector |
|---|---:|---:|---:|
| `3151:4001` | `0x0001` | `0x06` | `-1` (wildcard) |
| `3151:4001` | `0xff01` | `0x01` | `-1` (wildcard) |

The configuration also lists other bootloader IDs for other vendors. The `3151:4001` entries are static bootloader matching rules. They do not prove that the connected TH80 Pro can safely enter that mode or that the mode has been tested.

**Comparison:** the Windows updater explicitly selects the keyboard's normal vendor interface 2 and a bootloader PID `4001`; it does not list receiver `4011`. The macOS native driver has separate support entries for `4010` and `4011`.

## 4. Rust architecture recovered from strings

**[Static binary]** Useful embedded Rust source-path and type strings include:

- `src/main.rs`
- `src/hid/dj_device.rs`
- `src/data_source/yzw_upgrade.rs`
- `src/egui_ui/yzw_upgrade.rs`
- `SupportDev`, `SupportDevList`, `UpgradeConfig`
- `YZWBoot`, `YZWDev`, `DJDeviceInfo`, `device_info`
- `Status24`, `is_ready`, `battery`, `is_online`
- `upgrade_firmware`, `upload_firmware`, `devId`, `version`, `lowestAppVersion`

This supports a high-level architecture of a Rust desktop application with a HID/device layer, a YZW upgrade data source, an Egui upgrade UI, and device/version metadata. It does not recover the full function graph or establish that every string belongs to the TH80 Pro path.

### Selection modes

`selections_config.toml` defines:

| Selection ID | Name | Visible by default |
|---:|---|---|
| 0 | `yzw 升级` | Yes |
| 1 | `fw upload` | No |
| 2 | `cmd test` | No |
| 3 | `24 new test` | No |
| 4 | `holy rob` | No |
| 5 | `set into test` | No |

These are UI labels only. They are not protocol command identifiers and do not authorize testing hidden modes.

## 5. Firmware update timing and network metadata

`upgrade_config.toml` contains:

| Setting | Value |
|---|---:|
| `before_start_delay` | 1000 ms |
| `start_delay` | 1000 ms |
| `per_delay` | 20 ms |
| `end_delay` | 100 ms |

These are timing values, not packet lengths, checksums, addresses, or acknowledgements.

**[Static binary]** Strings include the download endpoint `http://api.rongyuan.tech:3000/v1/downloadDevUpgradeFile`, an admin upload endpoint, and metadata fields such as `devId`, `version`, and `lowestAppVersion`. These strings suggest remote firmware metadata/download integration. They do not establish the contents, authenticity checks, or actual image selected for the connected device.

## 6. Bootloader/update state machine

The following are the only defensible statements from the current evidence:

1. The normal-mode support configuration includes `3151:4010`, Usage Page `0xffff`, Usage `0x02`, interface 2.
2. The bootloader support configuration includes `3151:4001`, Usage Page `0xff01`, Usage `0x01`, wildcard interface `-1`.
3. The updater has a visible `yzw 升级` UI mode and embedded YZW upgrade module strings.
4. The updater includes HID Feature and Input report API names.
5. The package's timing configuration specifies 1000/1000/20/100 ms delays.

**[Unknown]** The actual transition trigger, bootloader handshake, Feature Report IDs used in bootloader mode, command opcodes, packet header, chunk size, sequence number, checksum/CRC, flash address, erase operation, acknowledgements, retries, completion response, and recovery behavior. No firmware-update state machine is claimed as recovered.

## 7. Vendor guide

**[Vendor documentation]** `Upgrade Guide.pdf` instructs the operator to close the driver, disconnect the 2.4 GHz receiver, place the keyboard in wired mode, and connect the keyboard by cable. It says only one updater should be open at a time and describes disconnect/reconnect steps if an update fails.

These instructions are recorded for provenance only. They were not followed. They are not a verified recovery procedure and do not establish that the keyboard can be restored after interrupted flashing.

## 8. Windows/macOS comparison

| Area | Windows updater | macOS driver |
|---|---|---|
| Artifact | Rust PE32+ GUI updater | Electron app + Rust x86-64 Mach-O `iot_driver` |
| Normal TH80 Pro match | `3151:4010`, interface 2 | `3151:4010`, interface 2 |
| Receiver `3151:4011` | Not in updater support list | Present, `dongle_common=true` |
| Bootloader match | `3151:4001`, page `0xff01`, usage `0x01`, wildcard interface | `YZWBoot` symbols exist; no equivalent live transition established |
| HID APIs | Runtime-resolved `HidD_*`, `HidP_GetCaps`; DeviceIoControl diagnostic strings | libusb control-transfer functions in `HidLibusb` |
| Feature setup fields | Not recovered; Windows HID API abstracts EP0 | Generic code uses `21/09/0300/0000/64` and `a1/01/0300/0000/64` |
| Firmware packet format | Unknown | Unknown |
| Configuration command protocol | Not established by updater | YZW command vocabulary in native binary; target command applicability unresolved |

Do not infer that the updater's Feature API buffers are the same as the macOS driver's 64-byte Feature payload or its firmware transfer blocks.

## 9. Remaining work

1. Recover the Windows Rust `yzw_upgrade` and `dj_device` call graph with a PE-aware decompiler and record exact function RVAs.
2. Trace the compiled `HidD_SetFeature` caller at VA `0x14022b510` through to the named `yzw_upgrade`/`YZWBoot` operation and recover the actual caller buffer, report ID and length.
3. Recover the updater's firmware validation and update state machine statically, without invoking it.
4. Establish the bootloader descriptor from a vendor-provided descriptor or an independently supplied static artifact; do not activate the physical bootloader to discover it.
5. Obtain passive traces only from a supported host using the official app, and only for non-update operations.

No firmware write, erase, bootloader activation, receiver modification, cross-flash, or recovery test has been performed. No complete restorable firmware backup is established.
