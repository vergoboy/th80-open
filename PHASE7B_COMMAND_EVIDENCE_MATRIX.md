# Phase 7B — Command evidence matrix

Date: 2026-11-01  
Primary report: [PHASE7B_READ_ONLY_COMMAND_DISCOVERY.md](PHASE7B_READ_ONLY_COMMAND_DISCOVERY.md)  
Method: static analysis only. Offsets are character offsets in the SHA-256-pinned minified renderer bundle. Native addresses are Mach-O VM addresses based at `0x100000000`.

## Evidence matrix

| ID | Operation | Frontend event / model method | RPC URI | Native handler / address | Packet builder and offsets | Report / direction | Response handling | Evidence grade | Safety |
|---|---|---|---|---|---|---|---|---|---|
| C01 | Firmware revision | Device insertion subscription (~`3,747,000`) → `getFirmwareVersion` (`2,359,986`; other class copies) | `/driver.DriverGrpc/sendMsg` then `/driver.DriverGrpc/readMsg` via `commomFeature` | Ordinary route's concrete handler unresolved. Candidate SendMsg poll `0x1000aa0a0` and ReadMsg poll `0x1000a3ef0`; downstream `DJDevice` SET `0x1000959e0`, GET `0x1000b4060`, libusb SET `0x100048290`, GET `0x100048460` are generic candidate chain only | 64-byte zero buffer; `[0]=0x80`; response `[1..2]` LE16 | Feature SET command + Feature GET response; 64-byte transfer | `readUInt16LE(1)` | **CONFIRMED** frontend; **UNKNOWN** exact ordinary native route binding / semantic command | **NOT APPROVED** |
| C02 | Device information | `FEA_CMD_GET_INFOR=0x8f` constant (`2,357,830`); no method/event found | None established | None | No builder found; byte 0 constant only | Unknown | None | **UNKNOWN** | **NOT APPROVED** |
| C03 | Battery query | `getBattery` (`2,380,229`); direct caller not established | `/sendMsg` then `/readMsg` via `commomFeature` | Same unresolved ordinary route; generic candidate addresses as C01, not a proven per-command mapping | 64-byte buffer; `[0]=0x83`; response `[1]=battery`, `[2]=charge state` | Feature SET then Feature GET | Converts status byte 2: 0 uncharging, 1 charging, other full | **CONFIRMED** frontend; model/opcode applicability **UNKNOWN** | **NOT APPROVED** |
| C04 | Battery/online stream | `watchSystemInfo` wrapper (`2,339,661`); descriptor at `2,335,267` | `/driver.DriverGrpc/watchSystemInfo` (server streaming) | Native stream handler/source unresolved in this analysis | Empty request; protobuf `SystemInfo` contains status messages; `Status24` fields battery and isOnline (`2,311,718`) | gRPC stream; no HID report identified | Protobuf object conversion | **CONFIRMED** frontend schema/wrapper; native producer **UNKNOWN** | **NOT APPROVED** |
| C05 | Active profile | `getCurrentProfile` (`2,361,031`; copies around `2,416,000`); store/config call sites (`3,759,163` onward) | `/sendMsg` then `/readMsg` | Same unresolved ordinary route; generic candidate chain only | 64-byte buffer; `[0]=0x82`; response `[1]` | Feature SET then Feature GET | Reads response byte 1; stores `currentProfile` | **CONFIRMED** frontend; native semantic behavior **UNKNOWN** | **NOT APPROVED** |
| C06 | Report rate | `getReportRate` (~`2,360,000`, `2,416,000`); `devToConfig` workflow (~`3,750,000`) | `/sendMsg` then `/readMsg` | Same unresolved ordinary route | 64-byte buffer; GET_REPORT is `0x81` in one dialect; `[1]=profile`; response `[2]` | Feature SET then Feature GET | Maps response 1/2/4/8 to 1000/500/250/125 Hz | **CONFIRMED** frontend; dialect-specific | **NOT APPROVED** |
| C07 | Keyboard options | `getKeyboardOption` (~`2,361,400`); store call sites (~`3,759,525`) | `/sendMsg` then `/readMsg` | Same unresolved ordinary route | `[0]=GET_KBOPTION`, `[1]=profile`; parses response `[2..4]`; opcode differs among model classes | Feature SET then Feature GET | Bitfields decoded to option booleans | **CONFIRMED** frontend; model-specific | **NOT APPROVED** |
| C08 | RGB/lighting mode | `getLightSetting` (~`2,362,000` and model copies); `devToConfig` | `/sendMsg` then `/readMsg` | Same unresolved ordinary route | `[0]=GET_LEDPARAM` (`0x84` in one dialect); response `[1]=mode`, `[2]=speed encoding`, `[3]=value`, `[4]=option` | Feature SET then Feature GET | Converts fields into lighting mode object | **CONFIRMED** frontend; command meaning/model binding **UNKNOWN** | **NOT APPROVED** |
| C09 | Per-key RGB/user picture | `getLightPic` (`2,374,026`); `_getLightPic` (`2,394,234`) | `/sendMsg` then `/readMsg` | Same unresolved ordinary route | One dialect `[0]=0x89`, `[1]=0`; another uses GET_USERPIC and page `[2]`, repeated pages | Feature SET then one/multiple Feature GET responses | Converts bitmap/RGB bytes into key-index/color objects | **CONFIRMED** frontend; model-specific | **NOT APPROVED** |
| C10 | Keymap | `getKeyConfig` (`2,370,865` / `3,708,082`) → `_getKeyMatrix` (`3,708,763`) | `/sendMsg` then `/readMsg` | Same unresolved ordinary route | `[0]=GET_KEYMATRIX` (`0x89` in this dialect); `[1]=profile`; some variants page at `[2]` | Feature SET then Feature GET, repeated per page | Matrix bytes decoded in 4-byte entries to configuration values | **CONFIRMED** frontend; native command semantics **UNKNOWN** | **NOT APPROVED** |
| C11 | Fn keymap | `getFnKeyConfig` (`2,371,285` / `3,708,502`) → model-specific `_getFnKeyMatrix` | `/sendMsg` then `/readMsg` where common helper is used | Same unresolved ordinary route | Per-model Fn command/layout not completely recovered | Feature SET then Feature GET | Matrix converted to Fn configuration values | **STRONGLY SUPPORTED** frontend; exact builder incomplete | **NOT APPROVED** |
| C12 | Macro data | `getMacro` (`2,374,508`, `2,403,061`, `3,711,192`) | `/sendMsg` then `/readMsg` | Same unresolved ordinary route | Model-specific GET opcodes include `0x88` and `0x8b`; macro index `[1]`, page `[2]`; multi-chunk reads | Feature SET then repeated Feature GET | Concatenates chunks and parses macro events | **CONFIRMED** frontend; dialect/native semantics **UNKNOWN** | **NOT APPROVED** |

## Shared frontend transport facts

| Evidence | Location | Result | Grade |
|---|---|---|---|
| Ordinary unary RPC descriptor | Renderer offset `2,334,500` onward | `/sendMsg`: `SendMsg → ResSend`; `/readMsg`: `ReadMsg → ResRead` | **CONFIRMED** |
| Ordinary RPC client wrappers | Renderer offset `2,337,609` onward | SET wrapper populates `devicepath`, `msg`, `checksumtype`, `dangledevtype`; GET wrapper populates `devicepath` and returns `msg` bytes | **CONFIRMED** |
| Model common query helper | Renderer offset `2,350,331` onward | `commomFeature` awaits `writeFeatureCmd` and only on success awaits `readFeatureCmd` | **CONFIRMED** |
| Ordinary send/read helper methods | Renderer offset `2,350,493` onward | `writeFeatureCmd` calls `sendMsg`; `readFeatureCmd` calls `readMsg` | **CONFIRMED** |
| Raw Feature helpers | Renderer offset `2,351,000` onward | `writeRawFeatureCmd` / `readRawFeatureCmd` call raw RPC wrappers; not the helper used by all candidates | **CONFIRMED** |
| Raw route descriptor/native mapping | `PHASE5D_RPC_DISPATCH_AUDIT.md` | Raw URIs select Tonic wrapper polls `0x1000a0f50` / `0x100092ec0`; final indirect handler target unresolved | **CONFIRMED** URI→wrapper; **UNKNOWN** wrapper→handler |

## Generic native packet facts (not command-specific)

| Layer | Address | Recovered fact | Grade |
|---|---:|---|---|
| Tonic router | `0x100048d50` | Routes gRPC paths; ordinary `sendMsg`/`readMsg` concrete poll binding not uniquely established in this report | **CONFIRMED** router; **UNKNOWN** concrete ordinary handler mapping |
| SendMsg-shaped handler candidate | `0x1000aa0a0` | Device path lookup and Base Feature SET slot `+0x30` | **CONFIRMED** candidate body; route assignment not proven |
| ReadMsg-shaped handler candidate | `0x1000a3ef0` | Device path lookup and Base Feature GET slot `+0x38` | **CONFIRMED** candidate body; route assignment not proven |
| Device path lookup | `0x1000603a0` | Manager device-list lookup using request path | **CONFIRMED** |
| DJDevice SET future | `0x1000959e0` | Appends request bytes, minimum logical length 66, selector-specific checksum insertion; passes to HidSender | **CONFIRMED** |
| DJDevice GET future | `0x1000b4060` | Dispatches Feature GET through HidSender | **CONFIRMED** |
| HidLibusb SET | `0x100048290` | Feature SET `0x21/0x09`, `wValue=0x0300`, `wIndex=0`, length 64 | **CONFIRMED** |
| HidLibusb GET | `0x100048460` | Feature GET `0xa1/0x01`, `wValue=0x0300`, `wIndex=0`, length 64; opaque bytes returned | **CONFIRMED** |

The native candidate handler bodies and generic Feature methods are recovered, but the ordinary RPC URI → concrete handler edge is not. Do not label a candidate's native path `CONFIRMED` end-to-end. The exact command packet bytes beyond the frontend-generated request buffers are not all known because the model dialect, `dangledevtype`, selector metadata, and ordinary-route mapping are not proven for the target device.

## Five-point safety gate

`Unknown` means the candidate fails the approval gate. A Feature GET by itself is not treated as proof of read-only behavior; the frontend helper first submits a Feature SET command buffer.

| ID | Persistent configuration unchanged | No reset/mode transition | No bootloader/update transition | No pairing/calibration change | No undocumented side effects | Approval |
|---|---|---|---|---|---|---|
| C01 firmware revision | Unknown | Unknown | Unknown | Unknown | Unknown | **NOT APPROVED FOR HARDWARE TESTING** |
| C02 device information | Unknown | Unknown | Unknown | Unknown | Unknown | **NOT APPROVED FOR HARDWARE TESTING** |
| C03 battery query | Unknown | Unknown | Unknown | Unknown | Unknown | **NOT APPROVED FOR HARDWARE TESTING** |
| C04 status stream | Unknown | Unknown | Unknown | Unknown | Unknown | **NOT APPROVED FOR HARDWARE TESTING** |
| C05 active profile | Unknown | Unknown | Unknown | Unknown | Unknown | **NOT APPROVED FOR HARDWARE TESTING** |
| C06 report rate | Unknown | Unknown | Unknown | Unknown | Unknown | **NOT APPROVED FOR HARDWARE TESTING** |
| C07 keyboard options | Unknown | Unknown | Unknown | Unknown | Unknown | **NOT APPROVED FOR HARDWARE TESTING** |
| C08 lighting mode | Unknown | Unknown | Unknown | Unknown | Unknown | **NOT APPROVED FOR HARDWARE TESTING** |
| C09 per-key RGB | Unknown | Unknown | Unknown | Unknown | Unknown | **NOT APPROVED FOR HARDWARE TESTING** |
| C10 keymap | Unknown | Unknown | Unknown | Unknown | Unknown | **NOT APPROVED FOR HARDWARE TESTING** |
| C11 Fn keymap | Unknown | Unknown | Unknown | Unknown | Unknown | **NOT APPROVED FOR HARDWARE TESTING** |
| C12 macro data | Unknown | Unknown | Unknown | Unknown | Unknown | **NOT APPROVED FOR HARDWARE TESTING** |

## Confidence-ranked static follow-up order

1. C01 firmware revision — concrete lifecycle caller, short parser, compact request.
2. C05 active profile — compact request and one-byte response parser.
3. C08 lighting mode — response field parsing is relatively complete; mode-specific side effects remain unknown.
4. C06 report rate and C07 keyboard options — explicit field parsing but dialect-specific.
5. C03 battery query / C04 status stream — simple fields but caller/native producer evidence is incomplete.
6. C10 keymap and C11 Fn keymap — complex and model-specific page layout.
7. C12 macro data — multi-page model-specific payload and parser.
8. C02 device information — no concrete operation found.

This order does not authorize hardware testing. No operation currently has sufficient semantic safety evidence, route certainty, and reversibility for a controlled test.

## Windows and firmware evidence boundary

- Windows updater SHA-256: `e9f3d8d2f3160be5e3817fb4eca8efe954cc854426351e3d0d21e73f39fce0fc`.
- `X` SHA-256: `4b22336ec9ffaaac3372791a1154a2c2a833e3305a4cadbbdcab9184252c0520`.
- Existing package inventory records `support_config.toml`, `selections_config.toml`, `upgrade_config.toml`, `Upgrade Guide.pdf`, updater executable and `X`.
- Updater strings include `Status24`, `battery`, `is_ready`, `is_online`, `YZWBoot`, `DJDeviceInfo`, and `upload_firmware`; these do not identify a read-only command or prove its effects.
- `X` is generic data with no established executable/firmware format or command semantics.

## Evidence references

- [Phase 5C Feature report correlation](PHASE5C_FEATURE_REPORT_CORRELATION.md)
- [Phase 5C RPC dispatch resolution](PHASE5C_RPC_DISPATCH_RESOLUTION.md)
- [Phase 5D Feature buffer layout](PHASE5D_FEATURE_BUFFER_LAYOUT.md)
- [Phase 5D RPC dispatch audit](PHASE5D_RPC_DISPATCH_AUDIT.md)
- [Phase 5D Windows updater buffer](PHASE5D_WINDOWS_UPDATER_BUFFER.md)
- [Firmware package analysis](FIRMWARE_ANALYSIS.md)
- [Updater reverse engineering](UPDATER_REVERSE_ENGINEERING.md)
