# Phase 8 — Command Semantics Matrix

Date: 2026-11-01  
Companion closure: [PHASE8_STATIC_EVIDENCE_CLOSURE.md](PHASE8_STATIC_EVIDENCE_CLOSURE.md)  
Candidate identity source: [PHASE7B_COMMAND_EVIDENCE_MATRIX.md](PHASE7B_COMMAND_EVIDENCE_MATRIX.md)

## Matrix rules

- Preserve all Phase 7B candidate IDs and operation identities exactly.
- A frontend builder/parser is not proof of device-side semantics.
- `commomFeature` performs Feature SET followed by Feature GET. Its default checksum type is 2; it waits 10 ms before SET and 5 ms before GET. `dangledevtype` is computed from the frontend device list (2 mouse, 1 keyboard, 0 otherwise).
- All candidates use ordinary `/driver.DriverGrpc/sendMsg` and `/driver.DriverGrpc/readMsg` when they call `commomFeature`; these concrete ordinary route handlers are unresolved.
- `UNKNOWN` means the missing evidence is material. No candidate is approved for hardware testing.

## Candidate matrix

### C01 — Firmware revision

| Field | Finding |
|---|---|
| Candidate ID / operation | C01 — Firmware revision |
| Frontend event / method | Device insertion subscription (~3,747,000) → inherited `getFirmwareVersion`; TH80-chain implementation at ~2,359,981 in `YZWCommon` (the Phase 7B offset ~2,359,986 is a model copy). |
| Command construction | `Buffer.alloc(64)`; `[0]=0x80` (`FEA_CMD_GET_REV`); all remaining bytes initially zero. |
| RPC URI | `/driver.DriverGrpc/sendMsg` then `/driver.DriverGrpc/readMsg` through `commomFeature`. |
| Native handler | UNKNOWN. Candidate SendMsg/ReadMsg polls `0x1000aa0a0` / `0x1000a3ef0` are not proven route bindings. |
| Transport | Frontend intends Feature SET then Feature GET; generic macOS libusb methods are known, but candidate-to-native-handler edge is not proven. |
| Response format | Frontend expects a Buffer and reads LE16 at bytes 1–2. No captured device response. |
| Parser | `readUInt16LE(1)`; no explicit length/status/checksum validation in this method. |
| Side effects | Device-side effects UNKNOWN. The command is submitted in a SET before GET. |
| Evidence confidence | CONFIRMED frontend builder/parser; UNKNOWN native and device semantics. |
| Safety decision | `UNKNOWN — NOT APPROVED FOR HARDWARE TESTING`. |
| Missing evidence | Concrete ordinary RPC vtable target; device-side `0x80` dispatch; response length/status/checksum; side-effect proof. |

### C02 — Device information

| Field | Finding |
|---|---|
| Candidate ID / operation | C02 — Device information |
| Frontend event / method | `FEA_CMD_GET_INFOR=0x8f` exists in the inherited RY108 constructor (~2,404,639); no TH80 method or event was located. |
| Command construction | No TH80 builder found. Constant only; no verified 64-byte request contents. |
| RPC URI | None established. |
| Native handler | UNKNOWN / none linked. |
| Transport | UNKNOWN. |
| Response format | UNKNOWN. |
| Parser | None located for a TH80 operation. |
| Side effects | UNKNOWN. |
| Evidence confidence | CONFIRMED constant; UNKNOWN operation identity beyond the Phase 7B label. |
| Safety decision | `UNKNOWN — NOT APPROVED FOR HARDWARE TESTING`. |
| Missing evidence | TH80 call site, request builder, RPC route, native handler, response layout/parser and device-side semantics. |

### C03 — Battery query

| Field | Finding |
|---|---|
| Candidate ID / operation | C03 — Battery query |
| Frontend event / method | `getBattery` at ~2,380,229 is outside `YZW_CY_TH80 → YZW_3098 → YZW_K68 → YZWCommon → RY108 → K68`. No TH80 battery method was established. |
| Command construction | The unrelated model implementation allocates 64 bytes and sets `[0]=0x83`; the inherited RY108 constant also equals `0x83`, but that does not establish TH80 use. |
| RPC URI | The unrelated method uses `/sendMsg` then `/readMsg` through `commomFeature`; TH80 call site UNKNOWN. |
| Native handler | UNKNOWN; ordinary route mapping unresolved. |
| Transport | The unrelated frontend path performs SET then GET. TH80 transport path UNKNOWN. |
| Response format | Unrelated parser uses byte 1 as battery and byte 2 as charge-state code. |
| Parser | 0 → uncharging, 1 → charging, other → full (unrelated model only). |
| Side effects | UNKNOWN. No TH80 applicability or device-side semantics. |
| Evidence confidence | CONFIRMED unrelated-model method/parser; TH80 applicability UNKNOWN. |
| Safety decision | `UNKNOWN — NOT APPROVED FOR HARDWARE TESTING`. |
| Missing evidence | TH80-specific method/call site, model binding, native route, matching firmware dispatch and side-effect analysis. |

### C04 — Battery/online stream

| Field | Finding |
|---|---|
| Candidate ID / operation | C04 — Battery/online stream |
| Frontend event / method | Phase 7B points to `watchSystemInfo` (~2,339,661) and its descriptor (~2,335,267). This is a system telemetry stream, not a battery stream. |
| Command construction | Empty gRPC request for `watchSystemInfo`; no Feature command builder established. |
| RPC URI | `/driver.DriverGrpc/watchSystemInfo` (server streaming). |
| Native handler | UNKNOWN. Native manager symbols `handle_dangle_common_24_status` (`0x10012c940`) and `handle_24_status` (`0x10012c9b0`) exist, but no exact producer/stream edge was established. |
| Transport | gRPC server stream; no candidate-specific HID Feature transaction identified. |
| Response format | `watchSystemInfo` returns `SystemInfo`: disk totals/available, network totals, CPU temperature/usage and memory totals/used. Separately, `Status24` contains `battery:uint32` and `isOnline:bool`, wrapped by `DangleStatus`. |
| Parser | Generated protobuf conversion is identifiable for both message types; `Status24` producer and stream association UNKNOWN. |
| Side effects | UNKNOWN; no device-side producer path or command semantics established. |
| Evidence confidence | CONFIRMED protobuf schemas and `watchSystemInfo` wrapper; UNKNOWN association between `Status24`, native status handlers and a frontend battery stream. |
| Safety decision | `UNKNOWN — NOT APPROVED FOR HARDWARE TESTING`. |
| Missing evidence | Correct battery/online stream route, native producer, handler-to-stream mapping and source of `Status24` values. Do not use `watchSystemInfo` as a battery query. |

### C05 — Active profile

| Field | Finding |
|---|---|
| Candidate ID / operation | C05 — Active profile |
| Frontend event / method | Inherited `getCurrentProfile` at ~2,361,026 in `YZWCommon`; store/config call sites exist in the frontend. |
| Command construction | `Buffer.alloc(64)`; `[0]=FEA_CMD_GET_PROFILE=0x85`; remaining bytes zero. Phase 7B's `0x82` is another model dialect, not the TH80 chain. |
| RPC URI | `/sendMsg` then `/readMsg` through `commomFeature`. |
| Native handler | UNKNOWN; generic candidate handlers are not route-confirmed. |
| Transport | Feature SET then Feature GET. |
| Response format | Byte 1 is read as an unsigned byte. |
| Parser | `readUInt8(1)`; returned value is assigned to local `this.currentProfile`. No explicit response length/status/checksum validation. |
| Side effects | Local frontend cache mutation is CONFIRMED; device-side profile selection or other effects UNKNOWN. |
| Evidence confidence | CONFIRMED TH80 frontend builder/parser/local update; UNKNOWN native/device semantics. |
| Safety decision | `UNKNOWN — NOT APPROVED FOR HARDWARE TESTING`. |
| Missing evidence | Ordinary RPC handler, device-side `0x85` case, meaning of SET-before-GET, profile side effects and response validation. |

### C06 — Report rate

| Field | Finding |
|---|---|
| Candidate ID / operation | C06 — Report rate |
| Frontend event / method | Inherited `getReportRate` at ~2,360,481 in `YZWCommon`; configuration workflow call sites exist. |
| Command construction | 64-byte zero buffer; `[0]=FEA_CMD_GET_REPORT=0x84`; `[1]=currentProfile || 0`. Phase 7B's `0x81` belongs to another dialect. |
| RPC URI | `/sendMsg` then `/readMsg` through `commomFeature`. |
| Native handler | UNKNOWN; ordinary route binding unresolved. |
| Transport | Feature SET then Feature GET. |
| Response format | Parser reads response byte 2. |
| Parser | 1 → 1000 Hz; 2 → 500 Hz; 4 → 250 Hz; 8 → 125 Hz; otherwise `"err"`. No explicit length/status/checksum validation. |
| Side effects | UNKNOWN. No proof that the preceding SET is side-effect-free. |
| Evidence confidence | CONFIRMED TH80 frontend request and mapping; UNKNOWN device-side meaning. |
| Safety decision | `UNKNOWN — NOT APPROVED FOR HARDWARE TESTING`. |
| Missing evidence | Native handler, firmware dispatch for `0x84`, profile preconditions, response status/length and side-effect proof. |

### C07 — Keyboard options

| Field | Finding |
|---|---|
| Candidate ID / operation | C07 — Keyboard options |
| Frontend event / method | Inherited `getKeyboardOption` at ~2,361,744 in `YZWCommon`. |
| Command construction | 64-byte zero buffer; `[0]=FEA_CMD_GET_KBOPTION=0x86`; `[1]=currentProfile || 0`. |
| RPC URI | `/sendMsg` then `/readMsg` through `commomFeature`. |
| Native handler | UNKNOWN; ordinary route binding unresolved. |
| Transport | Feature SET then Feature GET. |
| Response format | Bytes 2, 3 and 4 are consumed. |
| Parser | Byte 2 bits 0–7 → Win-key lock, system, WASD/arrow exchange, LED off, secondary LED off, keyboard mode, keyboard lock. Byte 3 bit 0 → `keyboardFnKeyMatrix`. Byte 4 → Boolean `powerSaveMode`. |
| Side effects | UNKNOWN. The parser describes frontend options, not the effect of the command on the device. |
| Evidence confidence | CONFIRMED TH80 frontend layout/parser; UNKNOWN native/device semantics. |
| Safety decision | `UNKNOWN — NOT APPROVED FOR HARDWARE TESTING`. |
| Missing evidence | Device-side flag meanings, native dispatch, profile requirements, response status/length/checksum and side-effect proof. |

### C08 — RGB/lighting mode

| Field | Finding |
|---|---|
| Candidate ID / operation | C08 — RGB/lighting mode |
| Frontend event / method | TH80's `YZW_3098` `getLightSetting` implementation at ~2,536,096. |
| Command construction | 64-byte zero buffer; `[0]=FEA_CMD_GET_LEDPARAM=0x87`. Phase 7B's `0x84` belongs to another model dialect. |
| RPC URI | `/sendMsg` then `/readMsg` through `commomFeature`. |
| Native handler | UNKNOWN; ordinary route binding unresolved. |
| Transport | Feature SET then Feature GET. |
| Response format | `[1]=mode`, `[2]=speed encoding`, `[3]=value`, `[4]=option`, `[5..7]=RGB` bytes. `YZW_3098` uses its own `MAXSPEED` value (4). |
| Parser | Converts mode and option fields into the frontend lighting object; RGB bytes are converted to a color value where applicable. No device-side enum or length validation established. |
| Side effects | UNKNOWN. Lighting mode can describe configuration, but the query's SET semantics are not proven. |
| Evidence confidence | CONFIRMED TH80 frontend builder/parser; UNKNOWN device-side enum and side effects. |
| Safety decision | `UNKNOWN — NOT APPROVED FOR HARDWARE TESTING`. |
| Missing evidence | Firmware `0x87` dispatch, mode enum, profile/mode preconditions, native handler and side-effect proof. |

### C09 — Per-key RGB/user picture

| Field | Finding |
|---|---|
| Candidate ID / operation | C09 — Per-key RGB/user picture |
| Frontend event / method | TH80 `YZWCommon._getLightPic` at ~2,467,585 and `getLightPic` at ~2,467,869. Phase 7B's earlier offsets include other model copies. |
| Command construction | 64-byte buffer; `[0]=0x89`, `[1]=0`. `commomFeature` sends this once; then the method performs `picsCount` additional `readFeatureCmd` calls. `picsCount=floor(defaultMatrix.length/4*3/64)` in the inherited base constructor. |
| RPC URI | First transaction `/sendMsg` then `/readMsg`; subsequent page reads use `/readMsg`. |
| Native handler | UNKNOWN; ordinary route binding unresolved. |
| Transport | One Feature SET + Feature GET, followed by repeated Feature GETs without another SET in this method. |
| Response format | Concatenated bytes interpreted as RGB triples. Exact device response length/page meaning UNKNOWN. |
| Parser | Converts each RGB triple to a color value and maps the triple index through the model's default matrix/HID-index helper; updates local `lightPic`. |
| Side effects | UNKNOWN. The first command is sent through Feature SET; subsequent reads do not prove the first SET harmless. |
| Evidence confidence | CONFIRMED TH80 frontend request/sequence/parser. |
| Safety decision | `UNKNOWN — NOT APPROVED FOR HARDWARE TESTING`. |
| Missing evidence | Firmware dispatch for `0x89` with `[1]=0`, page protocol, response length, RGB encoding and side-effect proof. |

### C10 — Keymap

| Field | Finding |
|---|---|
| Candidate ID / operation | C10 — Keymap |
| Frontend event / method | Generic `getKeyConfig` callers; TH80 inherited `YZWCommon._getKeyMatrix` at ~2,419,545. |
| Command construction | For each page 0–7: 64-byte buffer; `[0]=FEA_CMD_GET_KEYMATRIX=0x89`; `[1]=requested/current profile`; `[2]=page`. Each page is sent via `commomFeature`. |
| RPC URI | Eight `/sendMsg` + `/readMsg` transactions. |
| Native handler | UNKNOWN; ordinary route binding unresolved. |
| Transport | Eight Feature SET + Feature GET transactions. |
| Response format | Eight response chunks concatenated into a matrix buffer. No captured page data or verified device-side page length. |
| Parser | Inherited K68 `matrixToConfigValues` (~2,375,575) compares four-byte entries against `defaultMatrix`, then maps entries to key, combo, mouse, function, macro or forbidden configuration values. |
| Side effects | UNKNOWN. Profile/page selection may be interpreted by firmware; no side-effect proof exists. |
| Evidence confidence | CONFIRMED TH80 frontend builder and inherited parser; UNKNOWN firmware semantics. |
| Safety decision | `UNKNOWN — NOT APPROVED FOR HARDWARE TESTING`. |
| Missing evidence | Firmware distinction between C09 and C10 despite shared `0x89`; page bounds/lengths, profile preconditions, native route and side-effect proof. |

### C11 — Fn keymap

| Field | Finding |
|---|---|
| Candidate ID / operation | C11 — Fn keymap |
| Frontend event / method | Phase 7B generic `getFnKeyConfig` references remain candidate evidence. No `_getFnKeyMatrix` implementation was found in the TH80 inheritance chain. |
| Command construction | UNKNOWN for TH80. An unrelated `YZW_DK710B` implementation (~2,584,845) uses `[0]=0x90`, `[1]=Fn index`, `[2]=page`, nine pages; this is explicitly not transferred to TH80. |
| RPC URI | Phase 7B's generic path names `/sendMsg` and `/readMsg`; no TH80-specific caller/builder edge confirmed. |
| Native handler | UNKNOWN. |
| Transport | UNKNOWN for TH80. The other-model implementation uses `commomFeature`, but it is not evidence for this device. |
| Response format | UNKNOWN for TH80. |
| Parser | No TH80 Fn response parser located. |
| Side effects | UNKNOWN. |
| Evidence confidence | STRONGLY SUPPORTED that the bundle has Fn-keymap functionality in other models; UNKNOWN for TH80 implementation. |
| Safety decision | `UNKNOWN — NOT APPROVED FOR HARDWARE TESTING`. |
| Missing evidence | TH80-specific model method, command ID/layout, native handler, response parser and device-side semantics. |

### C12 — Macro data

| Field | Finding |
|---|---|
| Candidate ID / operation | C12 — Macro data |
| Frontend event / method | Inherited RY108 `getMacro` at ~2,410,211; generic call sites also exist in other model copies. |
| Command construction | For each macro index: 64-byte buffer; `[0]=FEA_CMD_GET_MACRO=0x8b`; `[1]=macro index`; `[2]=page` 0–3. Four `commomFeature` transactions are performed. Result is cached separately in `reComData` or `reFnData` according to local `isFn`. |
| RPC URI | Four `/sendMsg` + `/readMsg` transactions. |
| Native handler | UNKNOWN; ordinary route binding unresolved. |
| Transport | Four Feature SET + Feature GET transactions. |
| Response format | Four response chunks concatenated; exact firmware payload size/status/checksum unknown. |
| Parser | Inherited `buffToMacroEvents` (~2,377,258): LE16 repeat count at offset 0; 4-byte event records from offset 2; recognizes keyboard up/down, mouse button, mouse movement and delay records. |
| Side effects | Local macro cache update is CONFIRMED; device-side effects UNKNOWN. The request is still submitted via Feature SET. |
| Evidence confidence | CONFIRMED TH80 frontend builder/parser/cache behavior; UNKNOWN native/device semantics. |
| Safety decision | `UNKNOWN — NOT APPROVED FOR HARDWARE TESTING`. |
| Missing evidence | Firmware `0x8b` dispatch, macro index/page bounds, response framing, malformed/partial handling, native route and side-effect proof. |

## Cross-candidate conclusions

| Question | Result |
|---|---|
| Complete frontend → RPC → native handler → transport → device parser path | **0 / 12** |
| TH80-chain frontend response parsers identifiable | **8 / 12**: C01, C05, C06, C07, C08, C09, C10, C12 |
| Other-model parser evidence only | C03 battery; C11 Fn matrix |
| Schema only / route mismatch | C04 `Status24`; `watchSystemInfo` is unrelated `SystemInfo` telemetry |
| Constant only / no operation | C02 |
| Confirmed device-side state-changing effects | None established |
| Confirmed frontend-local state/cache updates | C05 current-profile cache; C09 picture cache; C12 macro cache |
| Candidates statically established read-only | **0** |
| Hardware testing approval | **None — all 12 NOT APPROVED** |

**Safety rule:** a command-bearing Feature SET followed by Feature GET is not approved as read-only unless the complete device-side dispatch and all meaningful side effects are established. No such evidence exists here.
