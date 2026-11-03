# Phase 13 — Vendor Protocol RGB Correlation

**Status:** frontend request construction and generic native transport are statically correlated. No device-side firmware handler has been recovered. No Feature GET/SET, HID/USB transfer or hardware test was performed.

## 1. Executive conclusion

The inspected TH80 frontend constructs a 64-byte lighting-parameter request with opcode `0x87` and parses bytes 1–7 as mode, inverted speed, value, option and RGB. Its lighting setter uses a separate command byte `0x07` and serializes the same fields. The apparent `GET` operation is implemented by the frontend helper as Feature SET followed by Feature GET; therefore `0x87` is not a proven read-only transaction.

Opcode `0x89` is overloaded in two TH80 frontend contexts:

1. per-key RGB/user-picture retrieval, request `[0x89, 0]`, followed by additional Feature GET reads interpreted as RGB triples;
2. keymap retrieval, request `[0x89, profile, page]`, with eight pages (0–7), each submitted through SET-then-GET.

The two frontend operations share a numeric opcode but have different request layouts and response parsers. Whether firmware dispatches by request bytes, subcommand, state or a common handler is unknown. No recovered firmware code ties either operation to an implementation.

## 2. Artifacts and inheritance path

- Frontend bundle: `../.phase3-work/mac/EPOMAKER Driver 1.0.0/EPOMAKER Driver.app/Contents/Resources/app/dist/index.js`
  - 4,370,837 bytes; SHA-256 `7d53737095fe5756fb0a18d5aecb7593555b9bb61a9d5d160ae0b988694c1e87`.
- Native driver: `../.phase3-work/mac/EPOMAKER Driver 1.0.0/EPOMAKER Driver.app/Contents/Resources/app/iot_driver`
  - 6,075,584 bytes; SHA-256 `22c5dc67ba77e2373a5ffdbb0312ed8c8e73eb16e78037764eb676a030b2724b`.

The TH80 model class is `YZW_CY_TH80`, whose constructor extends `YZW_3098` (bundle module around byte offset `2,892,952`). The relevant `YZW_3098` module is around byte offset `2,533,193`; its base command vocabulary is inherited from the `YZW_K68`/`RY108` chain. This is the TH80 class path, not a generic model-dialect inference.

## 3. Common request/response transport in the frontend

The bundle's common helper around byte offset `2,350,331` implements:

```text
commomFeature(request, checksumtype = 2):
    result = writeFeatureCmd(request, checksumtype, delay=10ms, devicePath, no-extra-Bluetooth-delay)
    if result is not false:
        return readFeatureCmd(delay=5ms, devicePath)
```

`writeFeatureCmd` waits while the frontend's `block` flag is set, delays, determines a separate `dangledevtype` value from paired receiver relationships, then calls the renderer RPC `sendMsg(devicePath, request, checksumtype, dangledevtype)`. The frontend helper argument is `checksumtype`; it is not the native `dangledevtype`. `readFeatureCmd` delays and calls `readMsg`. The exact native ordinary `/sendMsg` and `/readMsg` route-to-handler binding remains unresolved in the earlier Phase 5C–8 evidence.

Thus `commomFeature` is a **Feature SET request followed by Feature GET**, not a direct GET. The GET response parser does not establish that the preceding SET had no side effects.

### Generic native Feature framing

Earlier Phase 5C/5D disassembly established generic macOS `HidLibusb` wrappers:

| Method | Setup fields | Length | Timeout |
|---|---|---:|---:|
| `send_feature` at VA `0x100048290` | SET `0x21/0x09`, `wValue=0x0300`, `wIndex=0` | 64 bytes | 2,000 ms |
| `get_feature` at VA `0x100048460` | GET `0xa1/0x01`, `wValue=0x0300`, `wIndex=0` | 64 bytes | 2,000 ms |

The physical TH80 Pro vendor Feature report is on interface 2, while this generic path hardcodes `wIndex=0`. The reason/validity is unknown. The vendor native support config identifies `3151:4010` interface 2 as vendor HID and `3151:4011` interface 2 as `dongle_common`.

Phase 5D also recovered generic macOS SET buffer handling: append caller bytes, resize logical buffer to at least 66 bytes, apply selector-dependent complemented wrapping-sum checksum for native `dangledevtype=0` (offsets 1–7, stored at 7) or `1` (offsets 1–8, stored at 8), skip checksum for `2`, then transfer the first 64 bytes. The native checksum branch consumes `dangledevtype`, not the frontend protobuf `checksumtype` argument. These are generic transport rules, not RGB-specific firmware semantics.

The frontend's ordinary `sendMsg`/`readMsg` calls are not the same thing as the raw Feature RPC endpoints. Phase 5D proves raw URI-to-Tonic-wrapper edges but not the final indirect wrapper-to-handler call; ordinary route-to-handler mapping also remains unresolved. Do not claim an end-to-end RGB request path.

## 4. Opcode `0x87` — lighting parameters

### Frontend-observed request

The TH80 `getLightSetting` builder creates a zero-filled 64-byte buffer and sets byte `[0] = 0x87` (`FEA_CMD_GET_LEDPARAM`). It invokes `commomFeature(request, 0)` (explicit frontend `checksumtype=0`) and parses the returned buffer:

| Byte(s) | Frontend interpretation |
|---|---|
| `[0]` | `0x87` command byte |
| `[1]` | lighting mode/type |
| `[2]` | speed encoding; the TH80 `YZW_3098` class uses `MAXSPEED = 4` and converts as `4 - byte[2]` |
| `[3]` | value/intensity field, interpreted by mode |
| `[4]` | packed option/style: generally low nibble is `NORMAL`/`DAZZLE` (8/7), high nibble is direction/submode where used |
| `[5..7]` | RGB channels |

The `YZW_3098` parser sets `MAXSPEED = 4`; therefore its setter writes `byte[2] = 4 - UI_speed`, and its getter returns `4 - byte[2]`. This is frontend logic; it does not establish firmware bounds or physical LED semantics.

### Related setter

The frontend's `setLightSetting` uses `FEA_CMD_SET_LEDPARAM = 0x07`, not `0x87`. It serializes mode, `4 - speed`, value, packed option/style and optional RGB into bytes `[1..7]`, then calls `writeFeatureCmd(request, 1, 500)` (frontend `checksumtype=1`, 500 ms delay) and waits an additional `BIGCMDDELAY` of 500 ms after success. This `checksumtype` argument is distinct from native `dangledevtype`; the actual native checksum branch is not determined by this argument alone. The lower nibble of byte 4 carries the `DAZZLE`/`NORMAL` style value (7/8) for most modes; the upper nibble carries mode-specific direction/option values. The setter can map these named frontend variants to mode values:

| Mode byte `[1]` | Frontend effect |
|---:|---|
| 0 | LightOff |
| 1 | LightAlwaysOn |
| 2 | LightBreath |
| 3 | LightNeon |
| 4 | LightWave |
| 5 | LightRipple |
| 6 | LightRaindrop |
| 7 | LightSnake |
| 8 | LightPressAction |
| 9 | LightConverage |
| 10 | LightSineWave |
| 11 | LightKaleidoscope |
| 12 | LightLineWave |
| 13 | LightUserPicture |
| 14 | LightLaser |
| 15 | LightCircleWave |
| 16 | LightDazzing |
| 17 | LightRainDown |
| 18 | LightMeteor |
| 19 | LightPressActionOff |
| 20 | LightMusicFollow |
| 21 | LightScreenColor |
| 22 | LightMusicFollow2 |
| 23 | LightTrain |
| 24 | LightFireWorks |

When a color is supplied, RGB is serialized as R/G/B in bytes `[5]`, `[6]`, `[7]`. The frontend maps `0xFFFFFF` to its special stored value `16449530`; `LightUserPicture` instead writes `[5..7] = [0, 200, 200]`. Byte `[3]` is the mode's value/intensity field. These are frontend conventions, not confirmed firmware semantics.

This proves the vendor frontend's intended UI-to-request mapping only. It does not prove that the exact TH80 firmware implements every frontend mode or accepts every value.

### What `0x87` actually does

**Device-side answer: unknown.** The frontend labels it `GET_LEDPARAM` and parses a lighting record, but it sends a Feature SET containing `0x87` before issuing Feature GET. No firmware dispatch, response construction, validation, state mutation, persistence or acknowledgement semantics have been recovered.

A firmware occurrence of immediate `0x87` at raw file offset `0x0c5c` was investigated and is unrelated: it is an error identifier passed by a `yc_gpio.c` validation path to the SDK logger. It is not a firmware match for the HID opcode.

## 5. Opcode `0x89` — two independent frontend contexts

### 5.1 Per-key RGB/user-picture retrieval

The TH80 inherits `_getLightPic` from the shared `K68` base (bundle offset around `2,394,233`, reached through `RY108` → `YZWCommon` → `YZW_K68` → `YZW_3098` → `YZW_CY_TH80`). It builds a 64-byte request with:

- `[0] = 0x89`
- `[1] = 0`

It submits that request through `commomFeature` with its default frontend `checksumtype=2`, then performs `picsCount` additional `readFeatureCmd` calls. The frontend concatenates returned bytes, interprets them as RGB triples, maps each triple through the model's default matrix/HID-index helper, and stores the parsed result in the frontend object's in-memory `this.lightPic` property. That is a host-side cache, not evidence of where the keyboard persists per-key RGB. The read loop uses inherited `this.picsCount`. The RY108 constructor initializes this as `Math.floor(this.defaultMatrix.length / 4 * 3 / 64)`; later subclasses replace `defaultMatrix`, and no recomputation is visible in the TH80 subclass chain. Treat this as a frontend-internal read count, not a proven firmware picture-size specification.

Frontend evidence establishes this request and parser, not the firmware's RGB encoding, exact number of bytes returned, page semantics, or side effects. Because the initial call uses `commomFeature`, it includes Feature SET before Feature GET; the subsequent reads do not retroactively prove the first operation was safe.

### 5.2 Keymap retrieval

The same TH80 inheritance chain's `_getKeyMatrix` method uses:

- `[0] = 0x89`
- `[1] = profile` (explicit argument or current profile/default 0)
- `[2] = page`, iterated from 0 through 7.

It performs eight `commomFeature(request, 0)` transactions (explicit frontend `checksumtype=0`) and concatenates the returned page data into a keymap buffer. This is distinct from the per-key picture path: different byte-1 meaning, page byte, call count and response parser.

### 5.3 Collision and firmware dispatch

`0x89` must not be documented as exclusively RGB or exclusively keymap. The frontend uses it for both. The inherited constant table also contains `FEA_CMD_GET_USERPIC = 0x8c`, but this TH80 `_getLightPic` override uses literal `0x89`; do not replace the observed TH80 byte with the generic constant.

No firmware code has been tied to either path. It is unknown whether firmware distinguishes the contexts using byte 1, byte 2, internal state, transport path or separate dispatch tables.

## 6. Adjacent vendor lighting-stream control

The frontend exports `setLightType` with values:

- `0`: start music-follow data stream;
- `1`: start screen-color data stream;
- `2`: stop sending that stream.

A device-insertion lifecycle calls `setLightType(..., 2, ...)` before reading firmware version. This is host-driven ambient/screen/music lighting-stream control, not the per-key `0x07` setting command and not proof of the physical `Fn + Backspace` toggle. Its device-side effect remains unknown. It must not be tested as a diagnostic.

## 7. Native driver findings

Static inspection of the Mach-O native driver confirms:

- x86-64 Mach-O executable; exact supplied SHA-256 verified;
- embedded YZW command vocabulary including SET/GET LED parameter, sleep parameter, key matrix, user picture, macro and profile commands;
- embedded support configuration for keyboard `3151:4010` interface 2 and receiver `3151:4011` interface 2;
- generic libusb Feature SET/GET implementation and the unresolved `wIndex=0` mismatch described above;
- generic Rust/Tonic transport, with no firmware source or device-side RGB implementation.

`otool` was unavailable. GNU `nm` did not accept Mach-O's `-m` option; `llvm-objdump --macho` and `strings` were available. The earlier Phase 5C/5D reports retain the focused native disassembly and call-site analysis; this phase did not execute the native driver.

## 8. Safety classification

| Operation/evidence | Classification | Reason |
|---|---|---|
| Static frontend bundle extraction and native disassembly | `STATIC_ONLY` | Offline file inspection only |
| `getLightSetting` request builder (`0x87`) | `FRONTEND_OBSERVED` | Frontend request/parser recovered; firmware behavior unknown |
| Full `0x87` SET-then-GET transaction | `UNKNOWN` | Device-side side effects and response contract unknown |
| `setLightSetting` (`0x07`) | `WRITE_OPERATION` | Frontend constructs and submits a lighting-setting command; device acceptance/persistence unknown |
| Per-key picture `0x89` path | `FRONTEND_OBSERVED` + device operation `UNKNOWN` | Request/sequence/parser visible; firmware handler absent |
| Keymap `0x89` path | `FRONTEND_OBSERVED` + device operation `UNKNOWN` | Eight request pages visible; firmware handler absent |
| Native generic Feature SET/GET wrappers | `STATIC_ONLY` | Static transfer implementation; no command-specific read-only proof |
| `setLightType(2)` | `FRONTEND_OBSERVED` + device effect `UNKNOWN` | Stream-stop call observed; device side effects not correlated |
| Firmware handler for `0x87` or either `0x89` path | `UNKNOWN` | No dispatch/handler correlation |

No operation is `READ_ONLY_PROVEN` or `FIRMWARE_CORRELATED` in this report. No hardware test is authorized.

## 9. Reproducibility

Key static commands (run from `th80-open/`):

```bash
JS='../.phase3-work/mac/EPOMAKER Driver 1.0.0/EPOMAKER Driver.app/Contents/Resources/app/dist/index.js'
APP='../.phase3-work/mac/EPOMAKER Driver 1.0.0/EPOMAKER Driver.app/Contents/Resources/app/iot_driver'
sha256sum "$JS" "$APP"
file "$APP"
strings -a "$APP" | rg 'FEA_CMD|YZW_CMD_LIST|HidLibusb|sendRawFeature|readRawFeature'
rg -o -n '.{0,80}(commomFeature|getLightSetting|_getLightPic|_getKeyMatrix|setLightType).{0,180}' "$JS"
```

Important bundle offsets: common helper `2,350,331`; RY108 constructor `2,398,438`; inherited K68 `_getLightPic` `2,394,233`; YZWCommon `_getKeyMatrix` `2,419,545`; YZW_3098 class `2,533,193`; YZW_CY_TH80 class `2,892,952`. Offsets refer to the exact hashed bundle above and may change in another vendor-app version.

## 10. References

- `PHASE5C_FEATURE_REPORT_CORRELATION.md`
- `PHASE5C_RPC_DISPATCH_RESOLUTION.md`
- `PHASE5D_FEATURE_BUFFER_LAYOUT.md`
- `PHASE5D_RPC_DISPATCH_AUDIT.md`
- `PHASE5D_WINDEX_RESOLUTION.md`
- `PHASE7B_COMMAND_EVIDENCE_MATRIX.md`
- `PHASE8_COMMAND_SEMANTICS_MATRIX.md`, `PHASE8_STATIC_EVIDENCE_CLOSURE.md`
- `PHASE12_1_RGB_DIAGNOSTICS.md`, `PHASE12_1_CLOSURE.md`
