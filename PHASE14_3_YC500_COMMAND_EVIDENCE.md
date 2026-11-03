# Phase 14.3 — TH80 yc500 command contracts and evidence boundary

Date: 2026-11-02  
Scope: static evidence correlation and offline packet codecs only. No USB/HID request was sent; no firmware or persistent device state was changed.

## Device-family correlation

An independently maintained ROYUAN device registry contains an `EPOMAKER TH80` entry (`epomaker_th80`) with VID `0x3151`, PID `0x4010`, four profiles, and family `yc500` (registry record ID 742). The user's local V504 vendor application bundle independently contains the matching device record and class dispatch. This is direct evidence of the vendor frontend's selected implementation, not a direct capture from the physical unit. The bundle SHA-256 is `7d53737095fe5756fb0a18d5aecb7593555b9bb61a9d5d160ae0b988694c1e87`.

The public protocol analysis documents 64-byte Feature reports, report ID 0, vendor usage page `0xffff` / usage 2, command-family collisions, echoed scalar replies, and raw un-echoed page replies. It explicitly warns that product IDs alone cannot select a command family. See `https://github.com/dniminenn/sharkfin/blob/master/docs/PROTOCOL.md`.

The actual V504 application bundle in the user's existing evidence tree provides stronger target-specific inheritance evidence. Its `dist/index.js` device record at byte offset `2,259,844` identifies ID 742, VID 12625 (`0x3151`), PID 16400 (`0x4010`), name `yc500_th80`, display name `TH80`, and `fnLayer:1`. `findYC500Dev` at offset `3,765,000` routes `yc500_th80` to `YC500_TH80` (case at `3,772,629`); `YC500_TH80` at `3,568,206` extends `YC500Common`. `YC500Common` at `3,459,303` extends `YC200Common`; `YC200Common` at `2,812,661` extends `YZW_3098`. This is direct evidence of the V504 frontend's selected class chain, not physical-device execution.

The existing TH80 evidence still contains an unresolved native transport mismatch: the TH80/receiver vendor Feature report is declared on interface 2, while the recovered generic macOS libusb routines use `wIndex=0`. The frontend `sendMsg`/`readMsg` caller-to-native-handler edge is also not completely proven. Therefore, this phase does not enable active transport.

## C01–C12 matrix

Confidence labels describe applicability to the TH80 yc500 profile, not merely whether an opcode string exists:

- **Strongly correlated:** exact TH80 registry family match plus an independently documented yc500 packet contract; not a direct capture from this unit.
- **Family-confirmed / target-correlated:** packet shape is documented for yc500 but the exact TH80 frontend/device path remains incomplete.
- **Unknown / unsupported:** evidence does not establish an applicable TH80 yc500 operation.

| ID | Operation | Candidate opcode | Direction / transport contract | Request and response contract available offline | Confidence and implementation |
|---|---|---:|---|---|---|
| C01 | Firmware revision | `0x80` | 64-byte Feature SET query then Feature GET; Bit7 checksum (`CheckSumType::BIT7 = 0`) | Response parser consumes LE16 at bytes 1–2. Vendor parser does not validate opcode echo. | Direct selected `YC200Common.getFirmwareVersion` method at offset `2,817,232`. Offline request/response codec implemented. |
| C02 | Device information / identify | `0x8f` | 64-byte Feature SET query then Feature GET; exact checksum mode not established in the target frontend | Generic family response `[0x8f, id:u32 LE, ...]`; scalar response length 64. | Correlated static command constant plus independent family reference; no direct V504 getter found. Offline parser is retained as candidate; do not dispatch until vendor checksum/request path is confirmed. Returned ID is not assumed to equal archive ID 742. |
| C03 | Battery query | `0x83` candidate only | No applicable TH80 Feature SET/GET transport contract established | The only located `getBattery` implementation is inside the unrelated `Dk2017` class at offset `2,463,583`; it consumes bytes 1 and 2 but is not inherited by `YC500_TH80`. | Unknown / unsupported for TH80. `unsupported_candidate("C03")` returns a structured error. Do not borrow the Dk2017 battery contract. |
| C04 | Battery/online stream | no HID opcode established | `watchSystemInfo` is a gRPC system telemetry stream | Its documented fields are host disk/network/CPU/memory metrics; `Status24` battery fields have no proven producer/stream association to TH80. | Unknown / unsupported as a TH80 HID command. |
| C05 | Active profile | SET `0x05`, GET `0x85` | 64-byte Feature SET query/SET; explicit `CheckSumType::BIT7 = 0` | GET parser consumes byte 1 and does not require an opcode echo; profile setter accepts zero-based index. TH80 record has four profiles. | Inherited V504 methods in `YZWCommon` (`setCurrentProfile` at `2,500,112`; `getCurrentProfile` at `2,500,312`), on the selected `YC500_TH80` inheritance chain. Offline GET/SET builders/parser implemented. |
| C06 | Report rate | SET `0x04`, GET `0x84` | 64-byte Feature SET query/SET; explicit `CheckSumType::BIT7 = 0` | GET parser consumes byte 2: `1→1000`, `2→500`, `4→250`, `8→125` Hz. SET bytes 1/2 are profile/rate code. | Direct V504 frontend methods (`setReportRate` at `2,817,426`; `getReportRate` at `2,817,778`). Offline GET/SET codecs implemented. Conflicts with newer generic yc500 analysis that omits report rate; do not merge dialects. |
| C07 | Keyboard options | SET `0x06`, GET `0x86` | 64-byte Feature SET query/SET; Bit7 checksum | GET consumes flags byte 2, Fn-matrix byte 3, power-save byte 4. SET `[0x06, profile, flags, fn, ...]`; setter does not expose power-save. Source has a Mac/system flag discrepancy. | Direct `YC200Common.setKeyboardOption` / `getKeyboardOption` at offsets `2,813,421` / `2,813,927`. Raw options GET/SET codecs implemented; no guessed bit normalization. |
| C08 | Lighting parameters | SET `0x07`, GET `0x87` | GET: 64-byte Feature SET query with Bit7 checksum. SET: 64-byte Feature SET with Bit8 checksum at byte 8. | TH80 frontend consumes mode/speed/brightness/option/RGB bytes 1–7. Mode/range/flag interpretation is model-specific. | Direct V504 frontend methods (`setLightSetting` at `2,820,215`; `getLightSetting` at `2,822,457`). Raw-field GET/SET codecs implemented; no guessed mode/range mapping. |
| C09 | Per-key RGB / user picture | SET `0x0c`, GET `0x8c` | SET: seven 64-byte Feature SET reports; helper uses Bit7 checksum mode. GET: six page queries | SET has 8-byte header (`0x0c`, profile 0, total length 384 LE, page index) and up to 56 RGB bytes at offsets 8–63. GET page read-back is reported by independent hardware analysis as non-reflective; the inherited V504 frontend loops six pages. | SET builder implemented from inherited V504 frontend `_setLightPic` at offset `2,614,626`. GET read-back remains unsupported/untrusted. Persistent writes are never sent. |
| C10 | Keymap | GET `0x89`; single-slot SET `0x13` | GET: eight 64-byte page queries with Bit7 mode; SET: one 64-byte Feature SET with Bit7 mode | GET `[0x89, profile, page]`, pages 0–7, raw page replies. SET `[0x13, profile, slot, ...]`, four encoded value bytes at 8–11. TH80 default matrix has 512 bytes / 128 four-byte slots. | Inherited `YZWCommon._getKeyMatrix` at offset `2,503,072` (eight pages) and `YC500Common.setKeyConfigSimple` at `3,460,385`. Page GET and single-slot SET codecs implemented. Bulk keymap write remains unsupported because the inherited 9×56-byte transfer and 512-byte matrix sizing do not reconcile cleanly. |
| C11 | Fn keymap | simple SET `0x15`; GET candidate `0x95` | SET: 64-byte Feature SET with Bit7 mode | SET `[0x15, layer, slot, ...]`, four encoded value bytes at 8–11. `GET_FN_SIMPLE=0x95` is declared but no read method uses it in the selected `YC500Common` class. | Direct V504 `setFnKeyConfigSimple` method at offset `3,460,975`. Single-slot SET builder implemented; GET explicitly unsupported. |
| C12 | Macro data | GET `0x88`; SET `0x16` | GET: four 64-byte Feature SET page queries followed by four Feature GET responses; SET: up to five 64-byte Feature SET pages with Bit7 mode | GET sends `[0x88, slot, page]` for pages 0–3, each followed by one raw 64-byte response (256-byte aggregate). SET uses a 256-byte encoded macro buffer in 56-byte chunks. Vendor counts nonzero chunks, then sends the first N chunks with the final marker on N−1; this odd count/iteration behavior is reproduced. Generic yc500 `0x8b` is not selected for this TH80 V504 class. | Inherited `RY108.getMacro` at offset `2,493,596` and `YC500Common._setMacro` at `3,461,522`. Model-specific page-query/collector and `0x16` upload framing codecs implemented. Macro event grammar remains a separate parser concern. |

## Implemented codec boundary

`crates/th80-probe/src/yc500.rs` contains pure in-memory builders and parsers for C01, C05–C08, C09 upload, C10 keymap pages/single-slot writes, C11 single-slot writes, and C12 macro read/write framing. It enforces 64-byte report sizes, scalar response validation where established, per-command checksum modes, profile/page/slot bounds, raw-page response semantics, and exact 384-byte user-picture / 256-byte macro input sizes. C02, C03, C04, C09 read-back, C10 bulk write, C11 read-back, and C12 event encoding return structured `DriverError::UnsupportedOperation`.

The codec does not contain USB/HID handles, does not choose a physical interface, and does not dispatch requests. `DisabledTransport` remains the only active transport implementation and fails closed for Feature GET/SET and Input/Output operations.

## External technical questions still needed

1. Confirm that the production TH80 V504 firmware image (`X`, SHA-256 `4b22336ec9ffaaac3372791a1154a2c2a833e3305a4cadbbdcab9184252c0520`) matches the exact `yc500_th80` frontend class in the V504 bundle.
2. Provide the native/firmware call path for interface-2 Feature SET/GET and explain why recovered generic libusb routines use interface index 0.
3. Confirm which commands are supported on wired keyboard PID `3151:4010` versus receiver PID `3151:4011`, including receiver relay/status sequencing.
4. Confirm the V504 helper's per-command checksum handling at the wire boundary, especially the renderer `CheckSumType` values versus native `dangledevtype`-based insertion, and the GET_USERPIC read-back behavior.
5. Confirm whether the simple Fn GET opcode `0x95` is intentionally unused and whether any Fn-layer read method exists.
6. Confirm the exact TH80 bulk keymap write page count and the 512-byte matrix versus 9×56-byte inherited transfer mismatch.
7. Confirm the 256-byte macro event encoding and the semantics of the final-page marker; no event parser is inferred here.
8. Confirm the safe user-picture update cadence and recovery behavior; no persistent write is enabled here.
9. Provide TH80-specific lighting mode IDs, speed/brightness ranges, option-flag mapping, and mode-specific payload requirements.

The existing `PHASE13_12_VENDOR_SDK_REQUEST_DOSSIER.md` remains the source for the original V504 SDK/build package request, hashes, exact requested SDK files, and vendor questions about MCU/PCB/build provenance.

## External references inspected

- `dniminenn/sharkfin`, `docs/PROTOCOL.md` — transport/family warnings; identify; keymap/Fn pages; LED parameters; profile/options; revision/report-rate distinctions; macro and user-picture framing.
- `dniminenn/sharkfin`, `app/src-tauri/data/devices.json` — independent `epomaker_th80` record (742), `3151:4010`, family `yc500`, four profiles.
- `dniminenn/sharkfin`, `app/src-tauri/src/registry.rs` — typed registry record and family-specific command routing.
- `EP0_PROTOCOL.md`, `PHASE7B_COMMAND_EVIDENCE_MATRIX.md`, `PHASE8_COMMAND_SEMANTICS_MATRIX.md`, `PHASE13_11_RONGYUAN_API_AND_FRONTEND_FLOW.md`, and `PHASE13_12_VENDOR_SDK_REQUEST_DOSSIER.md` — existing target-specific evidence and unresolved native path.

The external repository is GPL-3.0-or-later. No source code was copied into this MIT project; the new module is an independent implementation of protocol facts, with attribution and the evidence boundary recorded here.
