# Phase 7B — Read-only command discovery

Date: 2026-11-01  
Scope: static inspection of the EPOMAKER macOS renderer/native driver and TH80 V5.04 Windows updater/firmware package. No original executable was launched and no device communication was performed.

## Executive result

The renderer exposes a family of model-specific query methods for firmware revision, battery, active profile, report rate, keyboard options, lighting, keymap, Fn keymap, macros, and user-picture/RGB data. The most concrete candidates are firmware revision (`0x80`), active profile (`0x82`), lighting parameters (`0x84` in one keyboard dialect), key matrix (`0x89`), and macro retrieval (`0x8b` in one dialect; other model implementations differ).

**No candidate is approved for hardware testing.** The critical finding is that the frontend's `commomFeature` helper is not a GET-only transaction: it first submits the command buffer through a Feature SET RPC (`sendMsg`), then issues a Feature GET RPC (`readMsg`). The SET may be the protocol's command-request mechanism, but it is still a device write, and no firmware-side semantic proof establishes that these opcodes are side-effect-free. The generic transport's GET method only returns opaque bytes; semantic response parsing occurs in the frontend.

The frontend bundle also contains a `GET_INFOR=0x8f` constant but no located method/call site using it. Battery has both a device-specific `getBattery` method (`0x83`) and status fields in the driver's `SystemInfo` protobuf stream. A `watchSystemInfo` frontend wrapper is present, but no confirmed consumer of that wrapper was located in this pass.

The Windows updater and firmware payload provide no independently verified read-only keyboard command. The updater is update-oriented; `X` remains an unidentified raw binary payload. No command semantics should be inferred from its strings.

## Artifact provenance

| Artifact | Size | SHA-256 | Static finding |
|---|---:|---|---|
| macOS renderer `dist/index.js` | 4,370,837 | `7d53737095fe5756fb0a18d5aecb7593555b9bb61a9d5d160ae0b988694c1e87` | Bundled JavaScript; source offsets below are character offsets in this exact file |
| macOS `iot_driver` | 6,075,584 | `22c5dc67ba77e2373a5ffdbb0312ed8c8e73eb16e78037764eb676a030b2724b` | Mach-O x86-64; VM base `0x100000000` |
| Windows V504 updater | 10,274,816 | `e9f3d8d2f3160be5e3817fb4eca8efe954cc854426351e3d0d21e73f39fce0fc` | PE32+ x86-64 updater; not executed |
| Firmware-sidecar `X` | 184,608 | `4b22336ec9ffaaac3372791a1154a2c2a833e3305a4cadbbdcab9184252c0520` | Generic raw data; architecture/container/entry point unknown |

Renderer path: `../.phase3-work/mac/EPOMAKER Driver 1.0.0/EPOMAKER Driver.app/Contents/Resources/app/dist/index.js`  
Native path: `../.phase3-work/mac/EPOMAKER Driver 1.0.0/EPOMAKER Driver.app/Contents/Resources/app/iot_driver`  
Windows package: `../TH80_ANSI_Firmware_V504/TH80 ANSI Firmware V504(ID 742)/`

## Frontend operation inventory

The bundle is minified into long lines. Locations below therefore use exact character offsets rather than misleading line numbers. Offsets refer to the renderer file identified above.

### Shared frontend transport and RPC

- RPC descriptors at offset `2,334,500` onward declare unary `/driver.DriverGrpc/sendMsg` (`SendMsg → ResSend`) and `/driver.DriverGrpc/readMsg` (`ReadMsg → ResRead`). Separate `/sendRawFeature` and `/readRawFeature` routes also exist, but the model methods listed below predominantly use the ordinary `sendMsg`/`readMsg` helpers.
- RPC wrapper implementation at offset `2,337,609` onward constructs `SendMsg`, assigns `devicepath`, `msg`, `checksumtype`, `dangledevtype`, calls `client.sendMsg`, and returns success when `ResSend.err` is empty. The read wrapper constructs `ReadMsg`, assigns `devicepath`, calls `client.readMsg`, and returns `ResRead.msg` as a Buffer or `undefined`.
- `USBDevice` helpers at offsets `2,350,331`–`2,351,700`: `commomFeature(request)` invokes `writeFeatureCmd(request, ...)` and, if that returns true, invokes `readFeatureCmd(...)`. `writeFeatureCmd` calls the ordinary `sendMsg` wrapper; `readFeatureCmd` calls `readMsg`. Thus a query method using `commomFeature` has a Feature SET request followed by a Feature GET response. This is a command/write transaction followed by a read, not a transport-only GET.
- `writeRawFeatureCmd` / `readRawFeatureCmd` are separate helpers around offsets `2,351,000`–`2,351,800`; the existence of raw RPCs must not be conflated with model methods using `commomFeature`.
- The gRPC client listens on `[::]:3814` with insecure local credentials (offset around `2,337,700`).

### Candidate operations

| Area / candidate | Frontend method and location | Request construction / response | RPC chain | Evidence |
|---|---|---|---|---|
| Firmware revision | `getFirmwareVersion`, offsets `2,359,986`, `2,415,714`, `2,733,417`, and additional model implementations; invoked during device insertion around `3,747,314` | 64-byte zero buffer; byte 0 = `0x80`; `commomFeature`; parses `readUInt16LE(1)` | `sendMsg` then `readMsg` | **CONFIRMED** frontend behavior; command semantics **UNKNOWN** |
| Device information | `FEA_CMD_GET_INFOR=0x8f` constant at `2,357,830` and other class constructors | No `getDeviceInfo` method or command call site located; no request/response layout recovered | None established | **UNKNOWN**; constant alone is not an operation |
| Battery | `getBattery` method at `2,380,229` | 64-byte buffer; byte 0 = `131` (`0x83`); `commomFeature`; reads response byte 1 as battery and byte 2 as charge-state enum (`0/1/other`) | `sendMsg` then `readMsg` | **CONFIRMED** frontend method; opcode collision/model dependence noted below |
| Battery/status stream | `Status24` protobuf at `2,311,718`; `Device` protobuf battery/isOnline fields around `2,315,338`; `watchSystemInfo` descriptor/wrapper around `2,335,267` / `2,339,661` | Streaming `SystemInfo` response; protobuf includes battery and online status fields | `/driver.DriverGrpc/watchSystemInfo` | **CONFIRMED** schema and wrapper; native source of values and UI consumer **UNKNOWN** |
| Active profile | `getCurrentProfile` at `2,361,031` and related implementations around `2,416,000` | 64-byte buffer; byte 0 = `0x82` in this dialect; response byte 1 updates `currentProfile` | `sendMsg` then `readMsg` | **CONFIRMED** frontend behavior; command semantics **UNKNOWN** |
| Report rate / config | `getReportRate` around `2,360,000` / `2,416,000` | Byte 0 = model-specific GET_REPORT (`0x81` in one dialect); byte 1 = current profile; response byte 2 maps 1/2/4/8 to 1000/500/250/125 Hz | `sendMsg` then `readMsg` | **CONFIRMED** frontend behavior; model-specific opcode mapping |
| Keyboard options | `getKeyboardOption` around `2,361,400` | Byte 0 = GET_KBOPTION; byte 1 = profile; parses response bytes 2–4 into option flags | `sendMsg` then `readMsg` | **CONFIRMED** frontend behavior; opcode varies by model |
| RGB / lighting mode | `getLightSetting` around `2,362,000` and model-specific copies | Byte 0 = GET_LEDPARAM (`0x84` in one dialect); response bytes 1–4 map mode, speed, value, option | `sendMsg` then `readMsg` | **CONFIRMED** frontend behavior; semantics **UNKNOWN** at firmware/native layer |
| RGB / per-key picture | `getLightPic` around `2,374,026`; another implementation `_getLightPic` around `2,394,234` | One dialect sends byte 0 = `137` (`0x89`), byte 1 = 0 then performs an additional `readFeatureCmd`; another sends GET_USERPIC with page index in byte 2 and concatenates multiple responses. Returned bytes become per-key RGB records | `sendMsg` then `readMsg` (some variants issue repeated reads) | **CONFIRMED** frontend behavior; opcode/layout differs across models |
| Keymap | `getKeyConfig` around `2,370,865` / `3,708,082`; `_getKeyMatrix` around `3,708,763` | Byte 0 = GET_KEYMATRIX (`0x89` in this dialect); byte 1 = profile; some variants page data via byte 2 and repeat up to 8 times. `matrixToConfigValues` converts returned 4-byte entries into UI config objects | `sendMsg` then `readMsg` per page | **CONFIRMED** frontend behavior; firmware semantics **UNKNOWN** |
| Fn keymap | `getFnKeyConfig` around `2,371,285` / `3,708,502` | Calls `_getFnKeyMatrix`; distinct model implementations exist, but no single universal request layout was established in the extracted evidence | `sendMsg` then `readMsg` where using common helper | **STRONGLY SUPPORTED** frontend model operation; exact per-model layout incomplete |
| Macro configuration | `getMacro` around `2,374,508`, `2,403,061`, and `3,711,192` | Model implementations use GET-like command bytes including `0x88` or `0x8b`; index in byte 1, page/chunk index in byte 2; responses concatenated (four chunks in one path) and parsed by `buffToMacroEvents` | `sendMsg` then `readMsg`, repeated by chunk | **CONFIRMED** frontend behavior; opcode/model semantics **UNKNOWN** |

### Model-dialect warning

The renderer is a multi-device driver, not a TH80-only protocol specification. Command values are class properties and vary between model implementations. For example, `0x83` is used as `GET_BATTERY` in one class and is also assigned to `GET_KBOPTION` in another; `0x89` is used for key-matrix retrieval in one implementation and user-picture retrieval in another. Macro retrieval also appears with differing opcode values. Do not transfer an opcode from one renderer class to another without proving that the connected TH80 model instantiates that class.

### UI/model call flow recovered

- Device insertion subscription begins around offset `3,747,000`. On an inserted supported device it creates a model instance, may call `setLightType(..., 2, ...)` to stop a lighting stream, then calls `getFirmwareVersion()`, stores the returned version in device metadata, and persists the metadata. The firmware query is therefore demonstrably reached from the device-insertion lifecycle. Note that the preceding `setLightType(2)` is a separate state-changing stream-control operation and is not part of the revision query's packet.
- The device-to-config aggregation method `devToConfig` is visible around offset `3,750,000` in the same model/store region. It reads report rate, key or Fn-key configuration, lighting settings, SLED settings, debounce/sleep/auto settings where supported, and per-key lighting or DPI. This is a configuration-read UI/model workflow, but its calls still use the SET-then-GET helper for Feature commands.
- Profile and keyboard-option methods are called in model/configuration workflows around offsets `3,759,000`–`3,774,000`; exact component-level JSX event handlers were not mapped from the minified bundle. The model API and store orchestration are present.
- `getBattery` was located as a model method but a direct caller in the renderer was not conclusively identified. Battery status is also represented in the protobuf stream schema.

## Native correlation and packet boundary

The macOS native binary implements generic Feature transport, not command-aware response parsing:

- Candidate SendMsg-shaped handler poll: `0x1000aa0a0`; candidate ReadMsg-shaped handler poll: `0x1000a3ef0`. They perform device-path lookup at `0x1000603a0`, then dispatch Base Feature SET slot `+0x30` / GET slot `+0x38`.
- `DriverGrpcServer::call` at `0x100048d50` routes RPC URIs. Phase 5D proves exact raw-route URI → Tonic wrapper-poll descriptors (`sendRawFeature` → `0x1000a0f50`; `readRawFeature` → `0x100092ec0`) but the final indirect `UnaryService::call` target remains unresolved. Ordinary `sendMsg` / `readMsg` use distinct URIs with shared request/response types and folded closure glue; no unique concrete ordinary-route poll assignment is claimed here.
- Downstream candidate Base-to-device Feature chain: `DJDevice::send_feature` future `0x1000959e0`; `DJDevice::get_feature` future `0x1000b4060`; `HidLibusb::send_feature` `0x100048290`; `HidLibusb::get_feature` `0x100048460`. This is a generic candidate transport chain, not proof that every renderer operation reaches a particular route/device.
- Feature SET construction for target metadata selector `+0x3a == 2`: append caller bytes; resize logical buffer to at least 66 bytes with zero fill; for `dangledevtype=0`, checksum offsets 1–7 and store at 7; for `dangledevtype=1`, checksum offsets 1–8 and store at 8; for `dangledevtype=2`, skip checksum. First 64 bytes are transferred. Do not generalize these selector-specific checksum branches.
- Feature GET requests 64 bytes into a zero-filled buffer and returns opaque bytes. It has no command-specific parser. The frontend performs the parsers described above.
- The physical HID descriptor evidence identifies a Feature report with implicit Report ID 0. The macOS libusb requests use `wValue=0x0300`, `wIndex=0`, length 64. The target interface descriptor is interface 2, so the reason for `wIndex=0` remains unexplained and operational success is unverified.

For model methods that use `commomFeature`, the frontend's SET-before-GET behavior is directly confirmed. The native raw-route handler uncertainty must not be silently treated as proof that the ordinary `sendMsg` route reaches the same candidate poll body. Accordingly, the exact per-command native handler and final command-specific native address are **UNKNOWN**. The generic downstream Feature transport addresses above are the strongest static correlation currently available.

## Safety assessment

The five checks below are applied to the semantic operation, not merely to the fact that the second half of the transaction is a Feature GET. Since each `commomFeature` call first submits a Feature SET command buffer and the firmware's behavior is not available for inspection, unknowns fail the approval gate.

| Candidate | Avoid persistent config modification? | Avoid reset/mode transition? | Avoid bootloader/update mode? | Avoid pairing/calibration changes? | Avoid undocumented side effects? | Decision |
|---|---|---|---|---|---|---|
| Firmware revision | Unknown: request is sent via Feature SET | Unknown | Unknown | Unknown | Unknown | **NOT APPROVED FOR HARDWARE TESTING** |
| Device information (`0x8f` constant only) | Unknown | Unknown | Unknown | Unknown | Unknown | **NOT APPROVED; operation itself unconfirmed** |
| Battery query | Unknown | Unknown | Unknown | Unknown | Unknown | **NOT APPROVED FOR HARDWARE TESTING** |
| `watchSystemInfo` status stream | Native source/effects unknown; no device command identified | Unknown | Unknown | Unknown | Unknown | **NOT APPROVED FOR HARDWARE TESTING** |
| Active profile | Unknown; profile query is a Feature SET request followed by GET | Unknown | Unknown | Unknown | Unknown | **NOT APPROVED FOR HARDWARE TESTING** |
| Report rate / keyboard options | Unknown | Unknown | Unknown | Unknown | Unknown | **NOT APPROVED FOR HARDWARE TESTING** |
| Lighting mode / per-key RGB | Unknown | Unknown; mode/stream control is adjacent in the lifecycle | Unknown | Unknown | Unknown | **NOT APPROVED FOR HARDWARE TESTING** |
| Keymap / Fn keymap | Unknown | Unknown | Unknown | Unknown | Unknown | **NOT APPROVED FOR HARDWARE TESTING** |
| Macro retrieval | Unknown | Unknown | Unknown | Unknown | Unknown | **NOT APPROVED FOR HARDWARE TESTING** |

The `watchSystemInfo` stream is the least command-like candidate because the renderer descriptor is server-streaming and carries status structures; however, the native source and any device-side polling/side effects were not resolved. It is not approved either.

## Prioritization

Ranking is for further static investigation only, not permission to communicate with hardware.

1. **Firmware revision** — strongest frontend call evidence (device insertion calls it), compact packet and parser (`0x80`, response LE16 at offset 1). Native ordinary RPC route remains unresolved and semantic side effects are unknown.
2. **Active profile** — compact packet and parser (`0x82`, response byte 1); several model-store call sites. Same native and safety gaps.
3. **Lighting mode** — request and response fields are substantially parsed (bytes 1–4), and it participates in `devToConfig`; opcode/model mapping varies and nearby lifecycle code controls lighting streams.
4. **Report rate / keyboard options** — explicit request fields and response parsing, but model-specific command numbers and uncertain device class.
5. **Battery** — simple response parser, but `0x83` collision between model classes and no confirmed direct caller. The status stream may be a better static target, but native provenance is missing.
6. **Keymap / Fn keymap** — richer multi-page model decoding exists, but request paging and model-specific details are incomplete.
7. **Macro retrieval** — response parser and chunking exist, but multiple opcodes/model paths and larger stateful data format increase uncertainty.
8. **Device information** — only an unused-looking `0x8f` constant was found; no operation to rank as a viable candidate.

No candidate scores sufficiently across static evidence, packet construction, response parsing, side-effect risk, and reversibility to justify a future controlled hardware test. A future test would first require independent firmware/protocol evidence that the request opcodes are side-effect-free and a proven TH80-specific class/route mapping.

## Windows updater and firmware artifacts

The V504 package contains `support_config.toml`, `selections_config.toml`, `upgrade_config.toml`, `Upgrade Guide.pdf`, the updater executable, and `X`. Existing static updater analysis identifies a Rust PE32+ firmware-updater application with HID support/bootloader configuration and firmware upload-related strings. The strings `Status24`, `battery`, `is_ready`, `is_online`, `YZWBoot`, `DJDeviceInfo`, and `upload_firmware` are not evidence of a safe query command. The updater's purpose and available evidence are update-centric; it was not executed.

`X` is 184,608 bytes of generic raw data. Readable strings include device/MCU source remnants, but no container, architecture, flash map, entry point, or command implementation was established. It is not treated as inspectable source code or as proof of any query semantics.

## Conclusion / stop condition

Phase 7B identifies the frontend candidate inventory and the SET-then-GET behavior common to its Feature-query helper. It does not establish a fully safe read-only command. All candidates remain **NOT APPROVED FOR HARDWARE TESTING**. No transport was enabled, no device handle was opened, no Feature GET/SET or other USB/HID transfer was performed, and no original app/updater/firmware was executed. Stop here; do not proceed to hardware testing or later phases under this task.

## Related evidence

- [Phase 5C Feature report correlation](PHASE5C_FEATURE_REPORT_CORRELATION.md)
- [Phase 5C RPC dispatch resolution](PHASE5C_RPC_DISPATCH_RESOLUTION.md)
- [Phase 5D Feature buffer layout](PHASE5D_FEATURE_BUFFER_LAYOUT.md)
- [Phase 5D RPC dispatch audit](PHASE5D_RPC_DISPATCH_AUDIT.md)
- [Phase 5D Windows updater buffer](PHASE5D_WINDOWS_UPDATER_BUFFER.md)
- [Firmware package analysis](FIRMWARE_ANALYSIS.md)
- [Updater reverse engineering](UPDATER_REVERSE_ENGINEERING.md)
