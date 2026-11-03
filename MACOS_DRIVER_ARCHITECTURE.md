# macOS EPOMAKER Driver architecture — Phase 4

## Artifact layout

The extracted application is an Electron application containing:

- `dist/main.js` — Electron main process bundle;
- `dist/index.js` — bundled renderer, model classes, gRPC client and UI logic;
- `iot_driver` — native Rust Mach-O x86-64 executable.

The native executable links IOKit, CoreFoundation, Foundation and CoreBluetooth. It also contains libusb/rusb transport code.

## Frontend/native boundary

**CONFIRMED:** `dist/index.js` creates a Tonic-compatible gRPC client for `[::]:3814` and registers separate `sendMsg`, `readMsg`, `sendRawFeature`, and `readRawFeature` unary methods. The raw methods are not aliases in the frontend descriptor: they have their own RPC paths.

The request fields are `devicepath`, `msg`, `checksumtype`, and `dangledevtype` for `SendMsg`; `ReadMsg` carries `devicepath`. The renderer's helper resolves success/error into booleans for writes and `Buffer | undefined` for reads.

The native binary includes generated Tonic routes for `sendMsgSvc`, `readMsgSvc`, `sendRawFeatureSvc`, and `readRawFeatureSvc`. Native path lookup is centralized in `get_dev_from_path` at `0x1000603a0`, which queries `DJDeviceManger::get_dev_list`.

The Phase 4.2/5C static analysis recovered candidate SendMsg/ReadMsg helper futures, device-path lookup, Base Feature vtable dispatch, `DJDevice` Feature futures, and concrete `HidLibusb` methods for both `3151:4010` and `3151:4011`. Phase 5D confirms the exact raw URI → Tonic wrapper-poll mapping (`sendRawFeature` → `0x1000a0f50`; `readRawFeature` → `0x100092ec0`). The final wrapper → concrete `GrpcServer` handler future remains **UNKNOWN** because the inner `UnaryService::call` uses an indirect vtable call at `0x1000ac5ed`, whose route-specific target is not recovered.

## Renderer device model

`YZW_CY_TH80` inherits from `YZW_3098`. The shared USB device implementation includes raw Feature helpers that call the raw RPCs, alongside ordinary command helpers that call `sendMsg`/`readMsg`.

Do not conflate these API families. Normal YZW command operations may use `sendMsg`/`readMsg`; raw Feature operations use separate RPCs. The existence of both does not prove all TH80 operations use both.

The renderer's `supportYZWDev` entry for TH80 is logical model metadata (ID 369, `vid=0x0461`, `pid=0x4002`, usage 6/page 1). It is not the physical descriptor identity `3151:4010` and must not be substituted for the native HID support record.

## Native device layers

### Discovery and metadata

`DJDeviceInfo::from_hid_info` at `0x1000c8b00` reads VID, PID, usage, usage page, interface number and path from `hidapi::DeviceInfo`. It preserves the interface number at metadata offset `0x28`. It also consults `SUPPORTDEVLIST` and stores additional classification/transport flags, including a byte at offset `0x3a`.

### Transport selection

`DJDevice::new` at `0x1000c8ea0` checks metadata byte `+0x3a`:

- value `2` → constructs `HidLibusb`;
- other values → constructs `HidBLE`.

This selection happens when the `DJDevice` is created, rather than on every request. The `dangledevtype` RPC field has not been shown to alter this native transport branch.

### libusb backend

`HidLibusb::new` at `0x100047d30` enumerates libusb devices, compares descriptor VID/PID to the metadata, opens the matching device, and copies the metadata (including interface number) into its backend state.

`HidLibusb::send_feature` at `0x100048290` and `HidLibusb::get_feature` at `0x100048460` call `libusb_control_transfer` directly. Both use 64-byte report transfers and hardcode `wIndex=0`. Their exact setup tuples are recorded in [PHASE4_CALL_CHAIN.md](PHASE4_CALL_CHAIN.md).

The copied interface number is present in backend metadata, but the control-transfer calls do not read it to form `wIndex`. `HidLibusb::new` initializes an empty `ClaimedInterfaces` tracking collection and the recovered application path makes no interface-claim or alternate-setting call. The statically linked libusb library contains Darwin claim routines for configuration/alternate-setting/state-restore paths, but they are not called by this constructor or either Feature method. `_libusb_control_transfer` copies the caller `wIndex` argument directly into setup-packet bytes 4–5. See [Phase 5D wIndex resolution](PHASE5D_WINDEX_RESOLUTION.md).

### BLE backend

`HidBLE` implements the same `HidSender` trait with methods named `send_feature` and `get_feature`, but these names do not describe HID Feature reports on the wire. `send_feature` prepends `0x06` and calls HIDAPI `HidDevice::write`, which reaches `IOHIDDeviceSetReport` with Output type 1; `get_feature` drains the HIDAPI input-report queue. `DJDevice::new` selects `HidLibusb` when metadata byte `+0x3A == 2`; the recovered support metadata for `3151:4010` and `3151:4011` selects that branch. The separate BLE-class entries use the other branch. See [Phase 5D HIDAPI/IOKit audit](PHASE5D_HIDAPI_FEATURE_AUDIT.md).

### IOKit

The binary imports `IOHIDDeviceOpen`, `IOHIDDeviceRegisterInputReportCallback`, and `IOHIDDeviceSetReport`. The exhaustive Phase 5D audit found one `IOHIDDeviceSetReport` call site, from `_hid_write`, with report type Output (1), and no `IOHIDDeviceGetReport` import. No IOKit Feature GET/SET path was identified. The identified Feature GET/SET implementation remains the separate libusb path; do not treat IOKit Output handling or `HidBLE` method names as Feature transport.

## Evidence boundaries

- **CONFIRMED:** gRPC endpoint separation, TH80 renderer inheritance, native metadata preservation, transport-tag branch, libusb Feature GET/SET control-transfer parameters.
- **CONFIRMED:** raw URI → Tonic wrapper poll (`sendRawFeature` → `0x1000a0f50`; `readRawFeature` → `0x100092ec0`).
- **STRONGLY SUPPORTED:** the common device-path helper futures perform Feature SET/GET dispatch through Base vtable slots `+0x30` / `+0x38`; their unique association with the raw rather than ordinary RPC handler remains unproven.
- **CONFIRMED:** the macOS SET future starts with an empty `BytesMut`, appends caller bytes, resizes to at least 66 bytes, computes selector-specific checksums over offsets 1–7 or 1–8, and the libusb transport sends the first 64 bytes. GET returns 64 bytes for any non-negative transfer result. Windows prepends `0x01`, retains dynamic payload length with minimum 65, and checksums the full payload; checksum destinations align relative to payload but checksum domains differ.
- **UNKNOWN:** why the generic libusb Feature methods hardcode `wIndex=0` despite interface-2 metadata/descriptors; command grammar and applicability; complete error conversion; exact Windows prefix/report-ID semantics and firmware-upgrade invocation; final indirect raw URI → handler vtable target.

No source edits or runtime interaction with the keyboard/receiver were performed.

## Phase 5D static-analysis reports

- [HIDAPI / IOKit Feature transport audit](PHASE5D_HIDAPI_FEATURE_AUDIT.md)
- [`wIndex=0` resolution](PHASE5D_WINDEX_RESOLUTION.md)
- [macOS Feature buffer layout](PHASE5D_FEATURE_BUFFER_LAYOUT.md)
- [Windows V504 updater Feature buffer](PHASE5D_WINDOWS_UPDATER_BUFFER.md)
- [RPC dispatch audit](PHASE5D_RPC_DISPATCH_AUDIT.md)

Phase 5D is static-only. The Windows buffer's relationship to firmware update operations and the exact raw RPC route-to-poll edge remain unproven. No device or updater was executed.
