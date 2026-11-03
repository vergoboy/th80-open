# Phase 15 — Native transport trace and closure boundary

Date: 2026-11-03  
Scope: static analysis of the V504 frontend bundle and macOS native `iot_driver`, followed by offline-only separation of frontend request metadata from the recovered native Feature envelope. No vendor process was launched and no physical I/O was performed.

## Artifact identity

| Artifact | SHA-256 |
|---|---|
| macOS `iot_driver` | `22c5dc67ba77e2373a5ffdbb0312ed8c8e73eb16e78037764eb676a030b2724b` |
| V504 `dist/index.js` | `7d53737095fe5756fb0a18d5aecb7593555b9bb61a9d5d160ae0b988694c1e87` |
| Windows V504 updater PE | `e9f3d8d2f3160be5e3817fb4eca8efe954cc854426351e3d0d21e73f39fce0fc` |

Local macOS evidence is under `.phase3-work/mac/EPOMAKER Driver 1.0.0/EPOMAKER Driver.app/Contents/Resources/app/`.

## 1. Exact frontend contract

The bundle offsets below are byte offsets, end-exclusive. They are stable against the recorded bundle SHA-256 above. The method bodies were extracted verbatim during the audit; this report records their bounded source locations and behavior rather than copying the vendor's minified implementation.

| Method | Bundle byte range | Recovered behavior |
|---|---:|---|
| `USBDevice.commomFeature` | `[2433583, 2433745)` | Calls `writeFeatureCmd` with default `CheckSumType=2`; after a successful write, calls `readFeatureCmd(5)`. |
| `USBDevice.writeFeatureCmd` | `[2433745, 2434269)` | Waits while blocked, applies caller-selected delay, optionally delays blue devices, computes `dangledevtype` from negative device-ID comparisons, then forwards the original `Buffer` to `sendMsg`. It does not mutate checksum bytes. |
| `USBDevice.readFeatureCmd` | `[2434269, 2434428)` | Waits while blocked, delays, then calls `readMsg(devicePath)`; no checksum field is sent on GET. |
| `USBDevice.writeRawFeatureCmd` | `[2434428, 2434854)` | Separate raw helper; computes `dangledevtype` using positive device-ID comparisons and calls `sendRawFeature`. Do not reuse normal-helper identity logic for this path. |
| `USBDevice.readRawFeatureCmd` | `[2434854, 2435023)` | Calls `readRawFeature(devicePath)` after its delay. |
| client `sendMsg` | `[2421106, 2421487)` | Constructs unary `SendMsg`, sets `device_path`, `msg`, `check_sum_type`, and `dangle_dev_type`, then calls the generated gRPC client. |
| client `readMsg` | `[2421487, 2421707)` | Constructs unary `ReadMsg` with only `device_path`; returns the response byte vector as a Buffer or `undefined`. |
| client `sendRawFeature` | `[2422179, 2422581)` | Same four protobuf fields as `sendMsg`, but calls the separate raw RPC. |
| client `readRawFeature` | `[2422581, 2422798)` | Calls the separate raw GET RPC and returns the response bytes or `undefined`. |

The generated service descriptor at approximately byte offset `2,335,490` registers ordinary `/driver.DriverGrpc/sendMsg` as unary `SendMsg -> ResSend` and `/driver.DriverGrpc/readMsg` as unary `ReadMsg -> ResRead`. Neither is streaming.

### Normal TH80 `dangledevtype`

The V504 runtime device record is ID `742`. The normal helper compares `danglecommondev.mouseId` and `.keyboardId` against `-deviceType.id`; neither matches `-742`, so normal TH80 requests pass `dangledevtype=0` (`DangleDevType::NONE`). The raw helper uses positive-ID comparisons and is a separate calculation.

### Per-method `CheckSumType`

The frontend enum is `BIT7=0`, `BIT8=1`, `NONE=2`. These values are protobuf request metadata; they are not evidence that JavaScript writes a checksum byte into the request Buffer.

| Command | Normal helper selection |
|---|---|
| C01 | `BIT7` |
| C05 GET / SET | `BIT7` |
| C06 GET / SET | `BIT7` |
| C07 GET / SET | `BIT7` |
| C08 GET | `BIT7` |
| C08 SET | `BIT8` |
| C09 upload SET | `BIT7` |
| C09 read helper | omitted argument → default `NONE` |
| C10 page GET / simple slot SET | `BIT7` |
| C10 separate bulk setter | omitted argument → default `NONE` |
| C11 single-slot SET | `BIT7` |
| C12 page GET / macro upload SET | `BIT7` |

For the ordinary helper, `dangledevtype=NONE(0)` is separate from every row above. `readMsg` itself carries no checksum or dangle fields.

### Frontend response conversion

- `sendMsg`: if a `ResSend` response exists, `err == ""` resolves `true`, otherwise `false`. A gRPC callback error or absent response is logged and resolves `false`; the Promise does not reject.
- `readMsg`: any `ResRead` returns `Buffer.from(msg)` verbatim; missing response returns `undefined`. Callback errors are logged; the Promise does not reject. There is no frontend response-length or short-read validation.

## 2. Native ordinary gRPC route — exact stopping point

Native Mach-O symbols and disassembly establish:

- `DriverGrpcServer<T>::call`: VMA `0x100048d50`; dispatches registered HTTP/2 gRPC paths.
- `sendMsgSvc<GrpcServer>::call` adapter future/drop type: VMA `0x100040a60`.
- `readMsgSvc<GrpcServer>::call` adapter future/drop type: VMA `0x100040990`.
- Tonic codec future/drop types: `ResSend <- SendMsg` at `0x100086230`; `ResRead <- ReadMsg` at `0x100086110`.
- Named application handler future/drop types: `GrpcServer::send_msg_by_dev_path` at `0x10005bb30`; `GrpcServer::read_msg_by_dev_path` at `0x10005b9f0`.
- Device async future/drop types exist for `DJDevice::send_msg` (`0x1000257e0`), `DJDevice::read_msg` (`0x100025410`) and `DJDevice::send_read_msg` (`0x100025b20`). These are async future drop glue, not callable method bodies.

The handler poll state machines make their final device call through trait-object vtables:

- Send handler state machine around `0x1000a3ef0`: indirect calls at `0x1000a4525` (`call *0x38(%rcx)`) or `0x1000a4794` (`call *0x58(%rcx)`), selected by an internal state byte.
- Read handler state machine around `0x1000aa1da`: indirect calls at `0x1000aa799` (`call *0x30(%rax)`) or `0x1000aacb5` (`call *0x50(%rax)`).

**Unresolved:** the vtable target does not identify whether these handlers invoke `DJDevice::send_msg`, `read_msg`, `send_read_msg`, or another API implementation. The exact forwarding of protobuf `check_sum_type` and `dangle_dev_type` into native method arguments is therefore not proven. The presence of both protobuf fields in the frontend request does not prove their native consumers.

Consequently, the ordinary native route's checksum transformation, response length policy, short-read behavior, retry policy, acknowledgement mapping and native error-to-`ResSend`/`ResRead` conversion remain unknown. The frontend behavior above is confirmed; native behavior is not.

## 3. Separate macOS `HidLibusb` Feature method evidence

The binary contains distinct `HidLibusb::{send_feature,get_feature}` and `HidBLE::{send_feature,get_feature}` implementations. `DJDevice::new` selects `HidLibusb` when device metadata byte `+0x3A == 2`; the V504 support records identify the TH80 keyboard and receiver with selector 2. The HIDAPI/IOKit `HidBLE` methods are not a second Feature implementation: its SET prepends `0x06` and calls HIDAPI Output write, and its GET reads the HIDAPI input callback queue. No IOKit Feature GET or Feature SET path was found.

For the separately recovered `DJDevice::send_feature` → `HidLibusb::send_feature` method:

- The higher-level SET future appends the caller message to a `BytesMut`, resizes to a minimum of 66 bytes with zero fill (does not truncate longer inputs), and applies the `dangledevtype` branch for metadata selector 2.
- `dangledevtype=0`: complemented additive sum over offsets 1–7, stored at offset 7.
- `dangledevtype=1`: complemented additive sum over offsets 1–8, stored at offset 8.
- `dangledevtype=2`: skips this checksum insertion branch.
- The destination byte participates in the sum if it was supplied by the caller; otherwise it is zero-filled.
- `HidLibusb::send_feature` requires at least 64 bytes and transmits exactly the first 64. It does not prepend a Report ID. `wValue=0x0300` selects unnumbered Feature Report ID 0.

The literal low-level setup contracts recovered from the libusb call sites are:

| Field | Feature SET | Feature GET |
|---|---:|---:|
| Direction | Host → device | Device → host |
| `bmRequestType` | `0x21` | `0xa1` |
| `bRequest` | `0x09` | `0x01` |
| `wValue` | `0x0300` | `0x0300` |
| `wIndex` | `0` | `0` |
| `wLength` | `64` | `64` |
| Timeout | `2000 ms` | `2000 ms` |

GET allocates a 64-byte zero-filled buffer. A negative libusb result becomes a one-byte native error result; a non-negative short transfer still returns the original 64-byte buffer, with unread bytes remaining zero. No command parser runs in this low-level method.

**Important boundary:** the low-level Feature method above is recovered, but the ordinary gRPC handler's indirect vtable target is not. Do not claim that the normal TH80 `sendMsg`/`readMsg` route necessarily reaches these exact methods or that the native `dangledevtype` checksum branch is its final packet transformation.

## 4. Interface 2 versus `wIndex=0`

The vendor support/descriptor metadata retains physical interface number 2. The recovered libusb Feature SET and GET setup packets explicitly pass `wIndex=0`; the native methods do not derive this value from the retained interface number. This is not a normalization performed by the Rust host implementation. The reason is unknown: it may be an intentional vendor convention or may relate to another device/backend path, but neither explanation is proven. Keep interface identity and USB setup `wIndex` as separate fields; do not substitute interface 2 into the recovered setup.

## 5. Windows and Linux are separate evidence domains

### Windows V504 updater PE

The supplied Windows PE resolves `HidD_SetFeature` dynamically. Its direct caller is a generic Rust HID device Feature future, not a proven updater-specific packet routine. It prefixes `0x01`, appends the source payload, resizes to at least 65 bytes without truncating longer payloads, and computes a complemented sum over the entire source payload at offset 8 or 9 according to a device selector. The prefix's Report ID meaning is unverified. The direct caller is not proven to be invoked by the YZW firmware-upgrade flow; do not classify it as configuration transport or updater-only transport without the missing call edge/source.

This Windows path differs from macOS: Windows passes the full dynamic prefixed Vec to `HidD_SetFeature`, while macOS sends exactly 64 bytes with implicit report ID 0. Their checksum offsets correlate after accounting for the prefix, but checksum input ranges differ.

### Linux

No vendor Linux configuration backend has been established. This repository remains passive sysfs discovery plus fail-closed Feature/Input/Output operations. No physical Linux backend was added.

## 6. Source changes in this phase

- Added `FrontendChecksumType` (`BIT7=0`, `BIT8=1`, `NONE=2`) and a separate `DangleDeviceType` (`NONE=0`, `KEYBOARD=1`, `MOUSE=2`).
- Added `FrontendSendRequest`, which retains the exact 64-byte caller Buffer and the two independent protobuf metadata fields.
- Corrected C01/C05–C12 request builders to preserve the caller Buffer rather than inserting a checksum byte based on `CheckSumType`. In particular, C08 SET retains RGB bytes 5–7 and leaves byte 8 zero; `BIT8` remains separate metadata. This is a frontend-request model, not a claim about the final native wire bytes.
- Added explicit macOS Feature SET/GET control setup structures/constants. `wIndex=0` is represented independently from the discovered interface number; report ID remains implicit zero and the timeout is 2 seconds.
- Added a mockable `ControlTransferBackend` interface and a disabled default implementation. Offline tests capture exact SET bytes/setup, retain a short GET in a zero-filled 64-byte buffer, and propagate a synthetic native failure. No real backend is implemented.
- Existing `Transport` remains fail-closed. No HIDAPI/libusb/udev dependency was added to `th80-probe` or `th80-gui`.

## 7. C01–C12 readiness matrix

| Command | Frontend request / policy | Offline implementation | Remaining boundary |
|---|---|---|---|
| C01 | `0x80`; `BIT7`; `dangledevtype=0` | 64-byte caller request + 64-byte scalar parser (bytes 1–2 LE16) | Native normal route and firmware byte interpretation not physically verified |
| C02 | `0x8f` response parser candidate; no target request helper/checksum policy | Candidate response parser only | Request dispatch unsupported |
| C03 | Battery helper found only in unrelated Dk2017 class | Structured unsupported | Not a TH80 command contract |
| C04 | Host telemetry/watch stream not tied to TH80 HID | Structured unsupported | Not established as host-driver scope |
| C05 | GET `0x85`, SET `0x05`; `BIT7` | Request buffers and profile parser/setter | Native SET acknowledgement/error mapping unknown |
| C06 | GET `0x84`, SET `0x04`; `BIT7` | V504-specific rate mapping and parser | Native SET acknowledgement/error mapping unknown |
| C07 | GET `0x86`, SET `0x06`; `BIT7` | Opaque options fields and request buffers | Native acknowledgement unknown; no flag reinterpretation |
| C08 | GET `0x87` `BIT7`; SET `0x07` `BIT8` | RGB bytes 5–7 retained, byte 8 zero; metadata separate | Native ordinary `check_sum_type` consumer unknown; do not claim byte-8 wire checksum |
| C09 | Upload `0x0c`, seven reports, `BIT7`; read helper defaults `NONE` | Seven 64-byte requests, 384 RGB bytes | Read-back unreliable; persistent writes not sent |
| C10 | GET `0x89` and simple SET `0x13`, `BIT7`; bulk helper defaults `NONE` | Page GET and single-slot SET | Bulk write unsupported: 512-byte TH80 matrix vs inherited nine 56-byte pages |
| C11 | Single-slot SET `0x15`, `BIT7` | Setter only | No verified getter; candidate `0x95` remains unsupported |
| C12 | GET `0x88` four pages and SET `0x16` up to five pages, `BIT7` | Page framing and raw 256-byte aggregation | Macro event grammar/semantics unknown |

No operation is physically interoperable-verified. Offline tests establish deterministic construction and validation only.

## 8. Fresh validation

All seven required commands completed with exit code 0:

| Command | Exit |
|---|---:|
| `cargo fmt --all -- --check` | 0 |
| `cargo test --workspace --locked` | 0 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 |
| `cargo build --workspace --release --locked` | 0 |
| `cargo doc --workspace --no-deps --locked` | 0 |
| `cargo tree -p th80-gui -e normal` | 0 |
| `cargo tree -p th80-probe -e normal` | 0 |

The fresh workspace test count is **175 passed, 0 failed, 0 ignored**, up from the historical 168-test baseline (+7 new regression tests). Unique test counts by target:

| Test target | Passed |
|---|---:|
| root `th80-hid-research` unit-test target (test harness only; binary not executed) | 3 |
| `th80-cli` offline workflow | 6 |
| `th80-core` library | 32 |
| `th80-core` Phase 10 compatibility | 1 |
| `th80-core` repository process | 5 |
| `th80-gui` | 44 |
| `th80-probe` library | 51 |
| `th80-probe` CLI | 6 |
| `th80-probe` offline protocol | 12 |
| `th80-probe` offline sysfs | 12 |
| **Total** | **175** |

The repository-process tests launch filtered test subprocesses; their repeated one-test summaries are not additional unique tests. The four `th80-probe` targets account for 81 tests (51 + 6 + 12 + 12).

Both required normal dependency trees are free of `hidapi`, `rusb`, `libusb`, `udev` and `hidraw`. The root `th80ctl` package still has its pre-existing `hidapi 2.6.7` / `udev` dependency for the separate legacy `th80-hid-research` binary; that binary was not executed. No physical transport dependency was added to `th80-gui` or `th80-probe`.

The fresh protocol CLI output was written to `/tmp/th80-protocol-cli-native-closure.json` (schema v1, 15 command entries, 7 structured unsupported entries; SHA-256 `0fe0b71968e3dd4e5e62b6ae944b8d93aa93e2c378524e4f710c510b2aa702c0`). The previously existing `/tmp/th80-protocol-cli-current.json` was not overwritten.

## 9. Remaining exact blockers

1. Matching native Rust source, symbol-rich build, or PDB/DWARF for the exact `iot_driver` hash to resolve the ordinary `send_msg_by_dev_path` / `read_msg_by_dev_path` handler vtable targets and argument forwarding.
2. Native schema/source proving whether ordinary `check_sum_type` and `dangle_dev_type` reach distinct parameters and what transformations occur.
3. Explanation from vendor material for interface-2 metadata versus the hardcoded `wIndex=0` Feature setup.
4. Windows source/call edge proving whether the generic `HidD_SetFeature` future is used by normal configuration, firmware update, or both.
5. Physical interoperability evidence is not available and is not authorized in this task; no hardware test should be attempted.

The original V504 firmware SDK/build request remains in `PHASE13_12_VENDOR_SDK_REQUEST_DOSSIER.md`. The narrow host-driver request is in `PHASE15_NATIVE_TRANSPORT_VENDOR_REQUEST.md`.

## 10. Exact native-source acquisition attempt — 2026-11-03

### Binary identity and debug state

The exact local macOS executable was re-hashed and its Mach-O load commands inspected:

- SHA-256: `22c5dc67ba77e2373a5ffdbb0312ed8c8e73eb16e78037764eb676a030b2724b` (unchanged).
- UUID: `E66C5739-F4BF-3DB7-AA10-01600946BF11`.
- x86-64 Mach-O executable; `LC_BUILD_VERSION` platform `macos`, SDK 13.0, minimum OS 12.0; `LC_SOURCE_VERSION` 0.0; linker `ld` 820.1.
- The enclosing bundle is `com.rongyuan.driverElectron`, EPOMAKER Driver 1.0.0.
- `LC_SYMTAB`: 22,512 nlist entries (20,333 local, 1,874 externally defined, 305 undefined). This is a symbol table, not DWARF debug information.
- No `__DWARF`/debug sections, dSYM, or matching external debug companion were found.
- Embedded build paths identify `/Users/jiandong/Documents/dj_hid_sdk_rs`; relevant source modules include `src/grpc_server/mod.rs`, `src/dj_dev_api/mod.rs`, `src/dj_dev_api/cmd_list.rs`, `src/dj_hid_device/mod.rs`, `src/dj_hid_device/dj_hid/mod.rs`, `hid_ble.rs`, and `hid_libusb.rs`. The generated service file path is `target/release/build/iot_driver-bf78ff4bc4bf14df/out/driver.rs`.

### Local package inventory

Inspected the extracted macOS EPOMAKER Driver 1.0.0 app, its local macOS ZIP, the TH80 ANSI V504 firmware ZIP and extracted directory, and all corresponding local executable/debug/protobuf/source candidates under the project parent. No matching Rust crate, `driver.proto`, `iot_driver` companion build, dSYM, DWARF, PDB, or CodeView companion for the macOS binary was present.

The only local Windows executable is `TH80 Upgrade Firmware for v5xx.exe` (SHA-256 `e9f3d8d2f3160be5e3817fb4eca8efe954cc854426351e3d0d21e73f39fce0fc`). Its PE CodeView record points to `C:\Users\Windows\Documents\rust-hid\target\release\deps\rust_hid.pdb`, GUID `C215AEE3-87E2-43DB-B791-B1A9DD29DCEB`, age 55, PE timestamp `2021-10-23 03:00:25`. This is a separate firmware-upgrade executable/build, not a matching debug companion for the macOS `iot_driver`; the PDB was not found locally and is not accepted as evidence for the ordinary macOS gRPC vtable.

### Public-source search disposition

Public searches for the exact macOS SHA-256, Mach-O UUID, `dj_hid_sdk_rs`, `send_msg_by_dev_path`, and native module paths did not locate matching source or debug artifacts. Two superficially relevant public projects were rejected as substitutes:

- `Aiacos/ajazz-control-center` documents AJAZZ Driver 2.1.94 and a Windows `i686-pc-windows-msvc` `iot_driver.exe`, not this EPOMAKER Driver 1.0.0 macOS x86-64 build. Its reverse-engineering notes are useful evidence that the Rongyuan driver family has multiple RPC and transport paths, but do not establish the vtable implementation for this UUID.
- `echtzeit-solutions/monsgeek-akko-linux` is an independent Linux reimplementation/research project for a different MonsGeek/Akko device family, not the matching vendor Rust source.

No candidate was accepted based on similar names or shared vendor branding. No vendor binary was uploaded, executed, or modified; no hardware was accessed.

### Decision

The bounded local and public search did not recover the exact native source or compatible debug symbols. The concrete targets behind send call sites `0x1000a4525` / `0x1000a4794` and read call sites `0x1000aa799` / `0x1000aacb5` therefore remain unresolved. No Rust source or protocol builder was changed in this acquisition attempt. The prior 175-test result remains the last validated implementation baseline; this documentation-only update does not claim a new test run.

Physical transport remains unsupported and fail-closed. No independently implementable host-side transport work remains justified by the currently available evidence; the next implementation step requires the exact crate/debug companion and schema requested in `PHASE15_NATIVE_TRANSPORT_VENDOR_REQUEST.md`.
