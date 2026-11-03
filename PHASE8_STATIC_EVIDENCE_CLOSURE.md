# Phase 8 — Static Evidence Closure & Command Semantics Audit

Date: 2026-11-01  
Scope: static analysis of the TH80 Pro project artifacts and existing offline driver workspace.  
Primary companion: [PHASE8_COMMAND_SEMANTICS_MATRIX.md](PHASE8_COMMAND_SEMANTICS_MATRIX.md)

## 1. Executive summary

Phase 8 closes several frontend evidence gaps and independently rechecks the original artifacts, but it does **not** establish a safe command path to the physical TH80. The final native RPC dispatch edge remains unresolved, the available firmware sidecar is not identified as a compatible firmware image, and device-side semantics for every candidate remain unknown.

Results:

- **0 of 12** Phase 7B candidates have a complete, proven path from the TH80 frontend through the concrete native RPC handler and transport to a device-side response parser/semantic implementation.
- **8 of 12** have identifiable response parsing in the TH80 frontend inheritance chain: C01 and C05–C10 and C12. This is frontend parsing, not proof of firmware-side semantics.
- C03's battery parser is from an unrelated model implementation. C04's `watchSystemInfo` stream is system telemetry, not battery status. C11's Fn-matrix builder is found in another model class, not the TH80 inheritance chain. C02 has a constant but no TH80 builder.
- The TH80 frontend's `commomFeature` helper submits a Feature SET command before issuing Feature GET. This is a real command transaction; it is not a passive GET.
- The native router's raw Feature URI-to-Tonic-wrapper mapping is confirmed. The concrete receiver/vtable target at the indirect call is not. The ordinary `/sendMsg` and `/readMsg` route-to-handler edges used by the 12 candidates are also not established.
- The Windows `X` sidecar is 184,608 bytes of generic data (SHA-256 below), entropy 3.6554 bits/byte, with embedded source-path strings consistent with possible embedded-controller code. Its format, architecture, target MCU, firmware version and compatibility are unknown. No command table or device-side parser was recovered.
- **No candidate qualifies as `STATICALLY ESTABLISHED READ-ONLY`.** Every candidate remains `UNKNOWN — NOT APPROVED FOR HARDWARE TESTING`.

This is a static evidence closure, not a hardware test authorization. No physical keyboard or receiver was opened, queried, written, updated, flashed, reset, paired or otherwise contacted.

## 2. Scope and safety restrictions

Only local file reads, hashing, string extraction, metadata inspection, disassembly and source-text analysis were performed. No original app, updater, firmware, extracted firmware code or device-facing executable was run. No HID/USB API was called. No transport was enabled. No packet builder or Rust source was changed.

Prohibited actions remained prohibited throughout:

- physical HID handle opening, Feature GET/SET, Output reports, interrupt transfers or USB control transfers;
- USB interface claim/detach or any indirect device communication;
- updater execution, firmware execution/flashing, bootloader/recovery, reset, pairing or persistent configuration changes;
- enabling the Rust transport or creating a test capable of reaching a physical device.

Evidence grades used here:

| Grade | Meaning |
|---|---|
| `CONFIRMED` | Direct, unambiguous static evidence establishes the stated fact. |
| `STRONGLY SUPPORTED` | Multiple independent facts support it, but at least one material edge remains unproven. |
| `HYPOTHESIS` | Plausible interpretation without sufficient direct evidence. |
| `UNKNOWN` | Available evidence cannot determine the fact. |

Safety labels are separate from evidence grades. An operation can have a confirmed frontend request layout and still be `UNKNOWN — NOT APPROVED FOR HARDWARE TESTING`.

## 3. Artifact inventory and fingerprints

Original paths are relative to the project root unless shown otherwise.

| Artifact | SHA-256 | Static identification / result |
|---|---|---|
| macOS `iot_driver` | `22c5dc67ba77e2373a5ffdbb0312ed8c8e73eb16e78037764eb676a030b2724b` | Mach-O native driver; independently rehashed. |
| macOS `dist/index.js` | `7d53737095fe5756fb0a18d5aecb7593555b9bb61a9d5d160ae0b988694c1e87` | Minified frontend bundle; independently rehashed. |
| Windows V504 updater EXE | `e9f3d8d2f3160be5e3817fb4eca8efe954cc854426351e3d0d21e73f39fce0fc` | PE executable; independently rehashed, not executed. |
| Windows sidecar `X` | `4b22336ec9ffaaac3372791a1154a2c2a833e3305a4cadbbdcab9184252c0520` | 184,608-byte generic data; independently rehashed. |

Original locations:

- `../.phase3-work/mac/EPOMAKER Driver 1.0.0/EPOMAKER Driver.app/Contents/Resources/app/dist/index.js`
- `../.phase3-work/mac/EPOMAKER Driver 1.0.0/EPOMAKER Driver.app/Contents/Resources/app/iot_driver`
- `../TH80_ANSI_Firmware_V504/TH80 ANSI Firmware V504(ID 742)/TH80 Upgrade Firmware for v5xx.exe`
- `../TH80_ANSI_Firmware_V504/TH80 ANSI Firmware V504(ID 742)/X`

The V504 package also contains `Upgrade Guide.pdf`, `support_config.toml`, `selections_config.toml` and `upgrade_config.toml`. The support configuration lists HID identity/interface matching entries, including interface-2 entries for some product IDs and separate boot entries. Those are updater support metadata, not proof that a particular image is installed on the user's keyboard or receiver.

The updater's static strings include `upload_firmware`, `Status24`, `is_ready`, `battery`, `is_online`, `YZWBoot`, `YZWDev`, `DJDeviceInfo` and `device_info`. These strings establish that the updater contains firmware-upload and status-related code paths; they do not establish that any TH80 command candidate is read-only.

### 3.1 Sidecar `X`

`file` identifies `X` only as generic data. It has no recognized ELF, PE or Mach-O header. Its measured Shannon entropy is 3.6554 bits/byte. Strings include `yc_gpio.c`, `yc_uart.c` and `yc_timer.c`; the byte stream contains ARM/Thumb-like instruction patterns. These observations are consistent with (but do not prove) an embedded-controller code/resource blob. The first bytes do not establish a firmware vector table or a specific executable format.

No reliable architecture, MCU, load address, linker map, version, encryption/compression scheme, command table or compatibility relationship was recovered. The package does not provide a proven link between `X` and firmware currently installed on the physical keyboard or receiver. No extracted payload was executed.

`llvm-readobj --file-headers --sections --coff-resources` was used for static PE inspection. No conventional PE resource payload was identified in the returned resource listing. This does not prove that the updater contains no firmware data in non-resource sections or custom containers; no such embedded image was established in this phase.

## 4. TH80 frontend inheritance and command constants

The frontend class/export regions establish this inheritance chain:

`YZW_CY_TH80 → YZW_3098 → YZW_K68 → YZWCommon → RY108 → K68`

Approximate bundle character offsets:

| Class / region | Offset |
|---|---:|
| `RY108` | 2,404,222 |
| `YZWCommon` | 2,412,092 |
| `YZW_K68` | 2,526,624 |
| `YZW_3098` | 2,533,193 |
| `YZW_CY_TH80` export/class region | ~2,892,944 |

The RY108 constructor inherited by TH80 defines these GET constants:

| Constant | Value | TH80 relevance |
|---|---:|---|
| `FEA_CMD_GET_REV` | `0x80` | Used by inherited `getFirmwareVersion`. |
| `FEA_CMD_GET_BATTERY` | `0x83` | Constant exists; no TH80 `getBattery` method established. |
| `FEA_CMD_GET_REPORT` | `0x84` | Used by inherited `getReportRate`. |
| `FEA_CMD_GET_PROFILE` | `0x85` | Used by inherited `getCurrentProfile`. |
| `FEA_CMD_GET_KBOPTION` | `0x86` | Used by inherited `getKeyboardOption`. |
| `FEA_CMD_GET_LEDPARAM` | `0x87` | Used by TH80's inherited/overridden lighting parser. |
| `FEA_CMD_GET_SLEDPARAM` | `0x88` | Constant only for this candidate set. |
| `FEA_CMD_GET_KEYMATRIX` | `0x89` | Used by inherited `_getKeyMatrix`. |
| `FEA_CMD_GET_KEYENABLE` | `0x8a` | Not one of the 12 candidate builders. |
| `FEA_CMD_GET_MACRO` | `0x8b` | Used by inherited `getMacro`. |
| `FEA_CMD_GET_USERPIC` | `0x8c` | Exists in RY108; the TH80 `YZWCommon` `_getLightPic` override instead uses literal `0x89`. |
| `FEA_CMD_GET_INFOR` | `0x8f` | Constant exists; no TH80 builder/event found. |

These are frontend constants, not device-side dispatch proof. In particular, the same `0x89` value is used by both the TH80 per-key picture path and the TH80 keymap path, with different request layouts and response sequences.

### 4.1 Corrections to Phase 7B interpretations

Historical Phase 7B findings are preserved. Phase 8 records these corrections rather than rewriting the old report:

- C03's `getBattery` at bundle offset ~2,380,229 is outside the TH80 inheritance chain. The inherited `0x83` constant alone does not prove a TH80 battery method.
- C04's `watchSystemInfo` is a system telemetry stream. Its `SystemInfo` message has disk/network/CPU/memory fields, not battery/online fields. Battery/online fields belong to the separate `Status24` message nested by `DangleStatus`.
- C05's TH80 implementation uses `GET_PROFILE=0x85`, not the `0x82` value from another model dialect in the Phase 7B matrix.
- C06's TH80 implementation uses `GET_REPORT=0x84`, not the `0x81` value from another model dialect.
- C08's TH80 implementation uses `GET_LEDPARAM=0x87`, not the `0x84` value from another model dialect.
- C09's TH80 `YZWCommon` implementation uses literal `0x89`; other model implementations use different `GET_USERPIC` layouts/opcodes. Do not transfer those layouts to TH80.
- C10's TH80 `_getKeyMatrix` is inherited from `YZWCommon` and uses `0x89`; its request is distinct from C09 despite the opcode collision.
- C11's `_getFnKeyMatrix` implementation at ~2,584,845 is in `YZW_DK710B`, not the TH80 class chain. It uses `0x90` there; that is not a TH80 Fn-keymap command.
- C12's TH80 RY108 `getMacro` uses `0x8b`; other model copies use `0x88` and other layouts.

## 5. Shared frontend command construction

The original frontend's `USBDevice` implementation is at ~2,350,326 onward:

- `commomFeature(buffer, checksumType=2)` awaits `writeFeatureCmd(buffer, checksumType, 10, devicePath, false)` and, if the SET result is not `false`, awaits `readFeatureCmd(5)`.
- `writeFeatureCmd` waits 10 ms (the common helper passes 10), computes `dangledevtype` from the frontend device list, and calls ordinary `sendMsg(devicePath, buffer, checksumType, dangledevtype)`.
- `readFeatureCmd` waits 5 ms and calls ordinary `readMsg(devicePath)`.
- The common helper's default `checksumType` is 2. The `dangledevtype` value is selected as 2 for a matching mouse ID, 1 for a matching keyboard ID, otherwise 0. The exact selected value for the user's physical device is not inferred here.
- The common helper uses ordinary `/sendMsg` and `/readMsg`, not `/sendRawFeature` and `/readRawFeature`.

This is a sequential command transaction: Feature SET carrying the request buffer, followed by Feature GET for the returned bytes. The GET is not independent of the preceding SET. The helper does not establish whether the device interprets a request as a harmless query, changes volatile state, selects an internal page, or has other effects.

The model methods generally allocate a 64-byte zero-filled buffer and set command/argument bytes. Native code may further pad, checksum or route that data; the frontend's 64-byte buffer is not itself proof of the final USB transfer bytes.

## 6. RPC dispatch investigation

### 6.1 Raw Feature routes

Static router comparisons at `0x100048f80` onward compare the exact raw route strings. The branches select these route descriptors:

| URI | Router branch | Descriptor | Tonic wrapper poll |
|---|---:|---:|---:|
| `/driver.DriverGrpc/sendRawFeature` | `0x100049032` onward | `0x1003cd908` | `0x1000a0f50` |
| `/driver.DriverGrpc/readRawFeature` | `0x10004904e` onward | `0x1003cd948` | `0x100092ec0` |

The descriptor entries are four-qword async-future descriptors (drop glue, size, alignment, poll function). These wrapper poll addresses are confirmed as the raw URI's Tonic wrapper futures. They are not the final concrete handler implementation.

### 6.2 Unresolved indirect dispatch

At `0x1000ac5ed`, the disassembly of the independently extracted `__TEXT,__text` section is:

```asm
1000ac5d7: 48 8b 01             mov    (%rcx),%rax
1000ac5da: 48 8b 70 10          mov    0x10(%rax),%rsi
1000ac5de: 48 8b 40 18          mov    0x18(%rax),%rax
1000ac5e2: 48 8d bd 20 fe ff ff lea    -0x1e0(%rbp),%rdi
1000ac5e9: 48 8b 55 c8          mov    -0x38(%rbp),%rdx
1000ac5ed: ff 50 20             call   *0x20(%rax)
```

Interpretation:

- `rcx` points to a slot containing a service-like object pointer.
- The loaded object supplies a pointer at `+0x10` passed in `rsi` (request-shaped argument) and a pointer at `+0x18` treated as a vtable.
- `rdi` points to an output/future storage area; `rdx` carries another context/service argument.
- The call dispatches through vtable slot `+0x20`.

This supports a trait-object/service dispatch interpretation, but the concrete receiver address, concrete vtable address, and slot target were not recovered from the constructor/route state. The call target must not be assigned to `send_msg_by_dev_path`, `read_msg_by_dev_path`, `send_raw_feature` or `read_raw_feature` solely by compatible signatures.

### 6.3 Async future/drop-glue evidence

The `__DATA_CONST,__const` area contains multiple four-qword future descriptors. Examples include:

| Descriptor VA | Poll VA | Drop glue VA | Size | Meaning established |
|---|---:|---:|---:|---|
| `0x1003cda40` | `0x1000a9e70` | `0x100040a60` | `0xc0` | One SET-shaped service-call future. |
| `0x1003cda60` | `0x1000b3ce0` | `0x100040990` | `0xa0` | One GET-shaped service-call future. |
| `0x1003cda80` | `0x1000b5300` | `0x100040a60` | `0xc0` | Another SET-shaped service-call future. |
| `0x1003cdaa0` | `0x1000b76a0` | `0x100040990` | `0xa0` | Another GET-shaped service-call future. |

Drop-glue symbols at `0x100040a60` and `0x100040990` are shared by generated service closure types for raw and ordinary send/read service paths. Sharing/folding means drop-glue identity cannot distinguish the four routes. The poll bodies also create nested futures, and the nested `GrpcServer` async future drop glue is visible in additional descriptors. However, the recovered data does not establish which concrete route's receiver/vtable is held at the unresolved `call *0x20(%rax)` site.

The Phase 7B candidate polls `0x1000aa0a0` (SendMsg-shaped, Feature SET slot candidate) and `0x1000a3ef0` (ReadMsg-shaped, Feature GET slot candidate) remain candidate bodies, not confirmed URI-to-handler edges. The generic device-path lookup and transport calls do not close the missing dispatch edge.

### 6.4 Device path, request and response boundaries

The frontend uses `deviceType.devAddr.toString()` as `devicePath`. Ordinary RPC request wrappers populate `devicepath`, `msg`, `checksumtype` and `dangledevtype` for send, and `devicepath` for read. The native code contains a device-path lookup candidate at `0x1000603a0` and generic SET/GET futures previously identified at `0x1000959e0` and `0x1000b4060`.

The ordinary `/sendMsg` and `/readMsg` URI-to-concrete-handler mapping is not proven. The raw Feature route mapping above does not substitute for the ordinary route used by `commomFeature`. The concrete path from `devicepath` equality to the correct `DJDevice` instance, then to a particular `HidSender` method and its response/error conversion, remains incomplete for each candidate.

### 6.5 Native transport facts retained from Phase 5D

These generic transport findings are retained as transport facts, not as command-specific semantics:

- `HidLibusb::send_feature` at `0x100048290` constructs a `BytesMut`, raises logical length to at least 66 with zero fill, does not truncate longer inputs, applies the selector-dependent checksum rules described in [PHASE5D_FEATURE_BUFFER_LAYOUT.md](PHASE5D_FEATURE_BUFFER_LAYOUT.md), and transfers the first 64 bytes. The app uses implicit Report ID 0 and explicit `wIndex=0`.
- `HidLibusb::get_feature` at `0x100048460` zeroes a 64-byte buffer and requests 64 bytes. A negative libusb result errors; a nonnegative short result can return a 64-byte buffer with a zero suffix. The method does not parse command-specific fields.
- `HIDAPI` is a separate backend. Its `HidBLE::send_feature` is an Output report (`_hid_write` / `IOHIDDeviceSetReport`, type Output=1), not the libusb Feature SET path. These paths must not be conflated.
- Windows V504 updater packet preparation is not byte-identical to the macOS libusb path. Its generic `HidD_SetFeature` wrapper prepends `0x01`, appends the dynamic payload and applies its own checksum/length behavior. This does not prove a relationship to a specific TH80 command.

## 7. Candidate command and response findings

The exact candidate IDs and operation identities C01–C12 are preserved in [PHASE8_COMMAND_SEMANTICS_MATRIX.md](PHASE8_COMMAND_SEMANTICS_MATRIX.md). The most important per-command results are:

- C01 firmware revision: TH80 inherited builder `0x80`, response LE16 at bytes 1–2.
- C02 device information: inherited constant `0x8f`, but no TH80 builder or call site.
- C03 battery query: `0x83` method and parser exist in another model region, not in TH80's inheritance chain.
- C04 battery/online stream: `Status24` protobuf schema is separate from the `SystemInfo` stream; native producer-to-stream link is unknown.
- C05 active profile: TH80 inherited builder `0x85`; response byte 1 is stored in local `currentProfile`.
- C06 report rate: TH80 inherited builder `0x84`; response byte 2 maps 1/2/4/8 to 1000/500/250/125 Hz.
- C07 keyboard options: TH80 inherited builder `0x86`; response bytes 2–4 become option flags/values.
- C08 RGB/lighting mode: TH80 `YZW_3098` implementation uses `0x87`; it decodes mode, speed, value, option and RGB fields.
- C09 per-key RGB/user picture: TH80 `YZWCommon` override uses `0x89` with `[1]=0`, then reads `picsCount` further Feature responses.
- C10 keymap: TH80 `YZWCommon` `_getKeyMatrix` also uses `0x89`, but `[1]=profile`, `[2]=page`; it performs eight page transactions. The inherited matrix parser interprets four-byte entries.
- C11 Fn keymap: no TH80 `_getFnKeyMatrix` implementation found. The `0x90`/nine-page implementation belongs to `YZW_DK710B`.
- C12 macro data: TH80 RY108 `getMacro` uses `0x8b`, macro index `[1]`, page `[2]` 0–3, and four command/response transactions; the inherited parser reads a repeat count and macro event records.

No complete device-side response parser, firmware dispatch case, or command-specific firmware side-effect analysis was found for any of the 12 candidates.

## 8. Response parser and state semantics

### 8.1 Frontend parser behavior

The eight identifiable TH80-chain parsers are frontend parsers only. The frontend generally assumes the returned Buffer has sufficient length and does not establish a device-side packet length/checksum contract:

- C01 calls `readUInt16LE(1)`; no explicit response-length guard is present in the method.
- C05 calls `readUInt8(1)` and updates the frontend's `currentProfile` cache. This is a local model-state mutation; it is not proof that the device profile changes.
- C06 reads byte 2 and maps 1, 2, 4 and 8 to report rates; other values become `"err"`.
- C07 reads bytes 2, 3 and 4. Byte 2 is decoded as eight flags; byte 3 bit 0 becomes `keyboardFnKeyMatrix`; byte 4 becomes a Boolean `powerSaveMode`. The method does not validate a device-side checksum or status field.
- C08 interprets bytes 1–7 as lighting mode, encoded speed, value, option and RGB. `YZW_3098` has its own `MAXSPEED` value and mode dialect; other model copies differ.
- C09 concatenates RGB triples and maps each triple to a key index using the model's default matrix. The initial command response is followed by repeated `readFeatureCmd` calls. No captured response length or firmware-side meaning is established.
- C10 concatenates eight page responses and passes the resulting bytes through inherited `matrixToConfigValues`, which compares four-byte entries with the default matrix and maps them to key/function/mouse/combo/forbidden configuration objects. It is a frontend interpretation, not a recovered firmware parser.
- C12's `buffToMacroEvents` reads a little-endian 16-bit repeat count at offset 0, starts records at offset 2, and interprets 4-byte event records. It recognizes mouse movement (`0xf9`), keyboard codes, mouse-button mappings and delay encodings. It filters zero-delay events in the returned `macroEvents`. This is a frontend parser; no actual captured macro packet is available.

C03's unrelated-model battery parser interprets byte 1 as battery and byte 2 as charging state (0 uncharging, 1 charging, other full). It must not be presented as TH80 behavior. C04's generated protobuf schema defines `Status24.battery` as uint32 and `Status24.isOnline` as bool, but no command-specific response parser or producer route was established. C02 and C11 have no TH80 parser identified.

### 8.2 Evidence categories

No complete captured Feature packet exists in the Phase 5–7 evidence. The 20-byte interface-2 descriptor fixture is a captured HID descriptor, not Feature traffic. The offline GET-response simulation is synthetic and must not be represented as captured device data. This phase did not create or relabel synthetic packets as captures.

There is no device-side evidence for report ID defaults, response status values, optional fields, checksum verification, partial-response handling or command-specific error codes beyond the generic transport behavior described above. No new firmware-side `Option<bool>` or report-ID semantics are asserted in this phase.

## 9. Side-effect and safety assessment

The frontend's SET-before-GET pattern means that the device receives a command-bearing Feature SET for C01, C05–C10 and C12. This is confirmed as a transport action, but its device-side semantics are unknown. A command name beginning with `GET_` does not prove the SET is side-effect-free.

Confirmed local frontend effects include updating `currentProfile` after C05 and caching retrieved macro/matrix/picture data. No persistent or volatile **device-side** effect was conclusively established from available static evidence. Conversely, absence of an identified write path is not evidence that no device-side side effect exists.

| Candidate | Persistent configuration | Volatile state / profile / mode | Pairing / calibration / bootloader / reset | Overall safety |
|---|---|---|---|---|
| C01 | UNKNOWN | UNKNOWN | UNKNOWN | `UNKNOWN — NOT APPROVED FOR HARDWARE TESTING` |
| C02 | UNKNOWN | UNKNOWN | UNKNOWN | `UNKNOWN — NOT APPROVED FOR HARDWARE TESTING` |
| C03 | UNKNOWN | UNKNOWN | UNKNOWN | `UNKNOWN — NOT APPROVED FOR HARDWARE TESTING` |
| C04 | UNKNOWN | UNKNOWN | UNKNOWN | `UNKNOWN — NOT APPROVED FOR HARDWARE TESTING` |
| C05 | UNKNOWN | UNKNOWN | UNKNOWN | `UNKNOWN — NOT APPROVED FOR HARDWARE TESTING` |
| C06 | UNKNOWN | UNKNOWN | UNKNOWN | `UNKNOWN — NOT APPROVED FOR HARDWARE TESTING` |
| C07 | UNKNOWN | UNKNOWN | UNKNOWN | `UNKNOWN — NOT APPROVED FOR HARDWARE TESTING` |
| C08 | UNKNOWN | UNKNOWN | UNKNOWN | `UNKNOWN — NOT APPROVED FOR HARDWARE TESTING` |
| C09 | UNKNOWN | UNKNOWN | UNKNOWN | `UNKNOWN — NOT APPROVED FOR HARDWARE TESTING` |
| C10 | UNKNOWN | UNKNOWN | UNKNOWN | `UNKNOWN — NOT APPROVED FOR HARDWARE TESTING` |
| C11 | UNKNOWN | UNKNOWN | UNKNOWN | `UNKNOWN — NOT APPROVED FOR HARDWARE TESTING` |
| C12 | UNKNOWN | UNKNOWN | UNKNOWN | `UNKNOWN — NOT APPROVED FOR HARDWARE TESTING` |

There is no confirmed device-side state-changing command among the 12 candidates, but there is also no proven read-only command. `STATE-CHANGING` is not assigned without direct evidence; `STATICALLY ESTABLISHED READ-ONLY` is not assigned without a complete command path and side-effect analysis.

## 10. Confirmed facts versus hypotheses

### Confirmed

- Original artifact hashes listed in Section 3.
- TH80 frontend inheritance chain and inherited command constants listed in Section 4.
- `commomFeature` sends a Feature SET command before Feature GET, with frontend defaults and delays described in Section 5.
- TH80's C09 and C10 frontend methods both use `0x89` with distinct request layouts/sequences.
- C03 battery method and C11 Fn-matrix implementation located in the bundle are outside the TH80 inheritance chain.
- `SystemInfo` and `Status24` are distinct protobuf messages with different fields.
- Raw Feature URI path comparisons select descriptors `0x1003cd908` and `0x1003cd948`, whose wrapper polls are `0x1000a0f50` and `0x100092ec0`.
- Indirect dispatch at `0x1000ac5ed` calls vtable slot `+0x20`.
- Generic macOS libusb and Windows updater packet preparation differ; HIDAPI Output is distinct from libusb Feature SET.
- No suitable firmware image or command table was identified in `X` or the inspected updater resource listing.

### Hypotheses / not established

- `X` may be embedded-controller firmware or a firmware-related blob; its format and target are unknown.
- `Status24` may be produced by the native manager status handlers, but no exact path to a frontend stream was proven.
- The candidate native SendMsg/ReadMsg poll bodies may correspond to ordinary RPC handlers, but route-to-concrete-vtable dispatch was not established.
- Any candidate's device-side operation may be read-only, but none has sufficient evidence to claim this.

## 11. Unresolved evidence gaps and priority

| Priority | Gap | Safety / correctness impact | Can static analysis resolve it? | Next useful evidence |
|---|---|---|---|---|
| P0 | Concrete receiver/vtable at `0x1000ac5ed` and exact ordinary `/sendMsg`/`/readMsg` dispatch | Determines whether frontend command bytes reach the assumed native SET/GET handlers | Possibly | Recover constructor/state object, concrete service type, vtable address and slot target; trace route branch through returned future and nested handler future. |
| P0 | Device-side semantics for all 12 IDs | Determines persistent/volatile side effects and hardware safety | Only if compatible firmware/source is available | Verified matching keyboard/receiver firmware, symbols/map or source; no third-party or guessed image. |
| P0 | Identity/format/target of `X` | Prevents false firmware attribution | Possibly | Identify container/MCU from validated header, linker metadata or exact matching package manifest; currently absent. |
| P1 | C04 `Status24` producer and stream mapping | Prevents confusing battery status with host telemetry | Possibly | Trace `DJDeviceManger` handlers into the concrete `watchDevList`/status stream implementation and protobuf producer. |
| P1 | C02 `0x8f` request construction and response parser | No operation currently established | Possibly | Locate a TH80-specific call site and correlate request/response types. |
| P1 | C03 battery method target-model applicability | Existing method is from unrelated model | Possibly | Locate a TH80-specific implementation or model dispatch binding. |
| P1 | C11 Fn-keymap implementation | Existing `0x90` path belongs to another model | Possibly | Find a TH80-specific method/command table, not another model's class. |
| P1 | Command-specific checksums, status fields, length and short-read handling | Current frontend parser assumptions may not match device responses | Possibly | Firmware parser or trusted captures from the exact hardware revision; no test is authorized now. |
| P2 | Model/connection/profile preconditions for multi-page commands | May affect semantics and device state | Possibly | Static call-site/state-flow analysis and compatible firmware control flow. |
| P2 | Windows updater-to-TH80 installed firmware relationship | Prevents assuming V504 payload matches installed device | Possibly | Package manifest/build ID and verified hardware revision compatibility metadata. |

## 12. Final hardware-testing decision

<strong>NOT APPROVED FOR HARDWARE TESTING — all 12 candidates.</strong>

No candidate has a complete frontend → ordinary RPC → concrete native handler → transport → device response → device-side semantic path. The SET-before-GET behavior is confirmed, but the effect of the command-bearing SET is unknown. The firmware sidecar does not close the device-side evidence gap. The physical keyboard and receiver are to remain untouched.

No future controlled hardware test should be proposed until the P0 evidence gaps are closed and each proposed command has a proven request format, device-side dispatch case, response parser, bounded side-effect analysis and a separately reviewed safety case. This report does not authorize such a test.

## 13. Verification and repository discipline

No Rust source, packet builder, Cargo manifest or historical Phase 5–7B report was intentionally modified during Phase 8. The only intended new repository files are:

- `PHASE8_STATIC_EVIDENCE_CLOSURE.md`
- `PHASE8_COMMAND_SEMANTICS_MATRIX.md`

The required Cargo checks, repository Markdown-link validation, final file-diff checks and confirmation of exactly two new reports are recorded in the Phase 8 completion response after execution. Historical Phase 7A's 47 passing tests are not counted as Phase 8 verification.

## 14. Recommended next steps

1. Continue static recovery of the service constructor and concrete vtable at `0x1000ac5ed`; do not infer the route edge from signatures or shared drop glue.
2. Locate a verifiably compatible keyboard/receiver firmware artifact before attempting device-side command dispatch recovery.
3. Resolve C04 through the native manager and actual status-stream producer; keep `watchSystemInfo` separate.
4. Close the missing TH80-specific C02, C03 and C11 frontend paths or leave them unsupported.
5. Preserve the offline-only transport boundary. Any later hardware-testing proposal requires a separate explicit safety review and authorization.

**Phase 8 ends here. No hardware test or Phase 9 work is started.**
