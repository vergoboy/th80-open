# Phase 12.1 — TH80 Pro RGB Diagnostics

**Status: static investigation complete; hardware testing NOT APPROVED.** This document is an evidence assessment and safe offline workflow. No physical HID handle was opened and no report, USB transfer, reset, firmware or bootloader operation was performed.

## 1. Reported symptom

The user reports that the physical TH80 Pro's RGB/backlighting is off, `Fn + Backspace` no longer turns it on, and the lighting is no longer in its normal state. The symptom is user-reported; no device state was read and no independent reproduction was attempted.

### Ranked possible causes

| Rank | Candidate | Evidence and confidence |
|---:|---|---|
| 1 | Keyboard is in a lighting-off state, or the shortcut's toggle state is not what the user expects | The TH80 Pro quick-start/manual material documents `Fn + Backspace` as backlight on/off. This is a plausible explanation, not a diagnosis; current state is unknown. **Moderate confidence that shortcut is documented; low confidence as cause.** |
| 2 | Lighting mode/brightness/color state is abnormal or brightness is at its minimum | Vendor manual documents separate effect, color, brightness and speed shortcuts. Static vendor frontend has a TH80 lighting-parameter query/parser. No actual device values are available. **Plausible; device state unverified.** |
| 3 | Active layer/profile or keymap remapping changes what the Fn chord does | Core model supports multiple layers and logical key assignments; vendor frontend has profile-specific keymap/configuration workflows. No actual keyboard keymap or layer was retrieved. **Plausible but unconfirmed.** |
| 4 | Keyboard mode/connection or firmware state affects shortcut handling | Vendor guide distinguishes wired/2.4G/BT modes and OS modes. Existing reverse-engineering work does not establish how those states gate RGB shortcut handling. **Weak hypothesis.** |
| 5 | Firmware/controller/RGB hardware fault | MCU, PCB revision, separate RGB controller and firmware-side dispatch are unknown in `HARDWARE.md`/`RESEARCH.md`. No physical inspection or controlled comparison exists. **Possible, no supporting diagnosis.** |

Do not infer that the GUI profile is the cause. The Phase 12 GUI has never applied a profile to the keyboard and cannot report the physical lighting state.

## 2. Documented lighting shortcuts

The TH80 Pro quick-start guide material documents these combinations:

| Shortcut | Documented function | Evidence grade |
|---|---|---|
| `Fn + Backspace` | Turn backlights on/off | **Vendor-manual documented** |
| `Fn + \\` / `Fn + \|` | Toggle RGB/backlight effects | **Vendor-manual documented; symbol rendering varies between text extractions** |
| `Fn + P` | Toggle RGB colors | **Vendor-manual documented; wording is ambiguous in some copies** |
| `Fn + ↓` / `Fn + ↑` | Decrease/increase backlight brightness | **Vendor-manual documented** |
| `Fn + ←` / `Fn + →` | Decrease/increase backlight speed | **Vendor-manual documented** |

The official EPOMAKER-hosted manual PDF (`TH80-manual update-v2.pdf`) lists `Fn + Backspace` for backlight on/off and `Fn + \\` for light-mode toggle. Its extracted text does not prove the exact installed keyboard hardware/firmware revision matches the user's unit. See [EPOMAKER-hosted TH80 manual](https://orders.epomaker.com/manuals/TH80-manual%20update-v2.pdf). An additional TH80 Pro quick-start transcription lists the complete brightness/speed combinations: [TH80 Pro quick-start guide](https://www.manualslib.com/guide/3387042/epomaker-th80-pro-rgb-bluetooth-wired-mechanical-gaming-keyboard-manual.html).

No Phase 5C–8 static reverse-engineering report identifies the firmware's physical Fn+Backspace handler, key matrix entry, debounce behavior, or the implementation of these shortcuts. The key combination is therefore documented by vendor material, not reverse-engineered firmware evidence. No additional undocumented RGB shortcut was recovered from the reports reviewed.

## 3. Static protocol findings

### C08 — TH80 lighting parameters (`0x87`)

**Confirmed frontend facts** (Phase 8 C08; `PHASE8_COMMAND_SEMANTICS_MATRIX.md`, section C08; `PHASE8_STATIC_EVIDENCE_CLOSURE.md`, sections 4–5):

- TH80 inheritance chain: `YZW_CY_TH80 → YZW_3098 → YZW_K68 → YZWCommon → RY108 → K68`.
- `YZW_3098.getLightSetting` builds a 64-byte zero-initialized request with byte `[0] = 0x87` (`FEA_CMD_GET_LEDPARAM`). Phase 7B's earlier `0x84` lighting opcode belongs to another model dialect; Phase 8 explicitly corrected it for TH80.
- Frontend response interpretation: `[1]` mode, `[2]` speed encoding, `[3]` value, `[4]` option, `[5..7]` RGB channels. `YZW_3098` has a model-specific `MAXSPEED = 4` in the frontend parser.
- This establishes what the inspected frontend constructs and parses. It does **not** establish firmware enum values, exact field semantics, valid ranges, response validity, persistence, or that the physical keyboard accepts this request.
- The model method calls `commomFeature`, which sends a Feature SET request and then issues a Feature GET. The command's apparent `GET_` name does not make the transaction read-only.

**Confidence:** high for the TH80 frontend builder/parser; unknown for firmware semantics and side effects. **Safety:** not approved for hardware testing.

### C09 — per-key RGB/user picture (`0x89`)

**Confirmed frontend facts** (Phase 8 C09):

- TH80 `YZWCommon._getLightPic` builds a 64-byte buffer with `[0] = 0x89` and `[1] = 0`.
- It uses `commomFeature` for the initial transaction, then performs `picsCount` additional `readFeatureCmd` calls. The frontend concatenates returned data, interprets it as RGB triples, maps triples through its model's default matrix/HID-index helper, and stores local `lightPic` data.
- Exact response length, page semantics, RGB encoding as implemented by firmware, and side effects are unknown.

**Confidence:** high for the frontend request/sequence/parser; unknown for device-side meaning. **Safety:** not approved for hardware testing.

### C10 — keymap (`0x89`): critical opcode collision

The same TH80 frontend uses `0x89` for `_getKeyMatrix`, but with a different request layout: `[0] = 0x89`, `[1] = profile`, `[2] = page`, for pages 0–7. It issues eight SET-then-GET transactions and parses concatenated four-byte entries as keymap records.

Therefore `0x89` must **not** be described as an RGB opcode. The `0x89` per-key picture path and `0x89` keymap path are separate frontend operations sharing a byte value. Firmware dispatch distinguishing them has not been recovered. The existence of `FEA_CMD_GET_USERPIC = 0x8c` in the inherited constant table does not change the fact that this TH80 `_getLightPic` override uses literal `0x89`.

**Confidence:** high for both frontend call constructions; unknown firmware dispatch and side effects. **Safety:** not approved for hardware testing.

### Generic Feature transport boundary

Relevant evidence:

- `PHASE5C_FEATURE_REPORT_CORRELATION.md`: generic macOS `HidLibusb::send_feature` and `get_feature` are 64-byte Feature class control-transfer wrappers. `wValue=0x0300` selects Feature Report ID 0; `wIndex=0` is hardcoded even though the physical vendor Feature descriptor is on interface 2. The reason and operational validity of this mismatch remain unknown.
- `PHASE5C_RPC_DISPATCH_RESOLUTION.md`: raw Feature RPC route descriptors and candidate handler bodies are correlated, but final route-to-handler mapping is not proven. The ordinary `sendMsg`/`readMsg` mapping used by the frontend model methods remains unresolved.
- `PHASE5D_FEATURE_BUFFER_LAYOUT.md`: the higher SET future appends caller bytes, resizes to at least 66 bytes, applies selector-dependent checksum insertion for `dangledevtype=0` or `1`, and skips that checksum for `2`; the low-level transfer sends only the first 64 bytes. This is generic transport framing, not RGB-specific command semantics.
- `PHASE5D_HIDAPI_FEATURE_AUDIT.md`: HIDAPI's recovered macOS write is an Output report, not an alternate Feature GET/SET path. The target keyboard/receiver construction selects the separate libusb Feature path in the analyzed binary.
- `PHASE5D_RPC_DISPATCH_AUDIT.md`: the ordinary and raw RPC dispatch/transport limitations remain; do not treat the generic candidate chain as a fully proven RGB request path.

No complete captured Feature transaction or device-side command implementation is present in the reviewed Phase 5–8 evidence. No firmware-side parser, mode enum, RGB controller mapping, nonvolatile storage location or default lighting table was recovered.

### Other adjacent operation: `setLightType(2)`

`PHASE7B_READ_ONLY_COMMAND_DISCOVERY.md` records that device insertion may call `setLightType(..., 2, ...)` to stop a lighting stream before querying firmware version. This is a separate stream-control operation, not the `0x87` lighting-parameter request and not proof of the Fn+Backspace toggle. Its exact device-side effects and relation to the user's symptom are unknown. It must not be tried as a diagnostic command.

### Firmware defaults and persistence

No verified TH80 firmware default for lighting enabled state, mode, brightness, speed or color was found. `RESEARCH.md` and `HARDWARE.md` explicitly leave the MCU, PCB revision, separate RGB controller and ownership/persistence of settings unknown. No static evidence establishes whether lighting state is stored in keyboard flash, controller NVM, receiver, host profile or volatile RAM.

## 4. Safe offline diagnostic sequence

Perform these checks without opening a device handle or issuing any protocol request:

1. **Inspect local profiles only if the user supplies/identifies an exported profile file.** Read `configuration.lighting`, `configuration.keymap.layers`, layer references, and `device_model`. Check whether `enabled` is explicitly `false`, unspecified, or `true`; whether a mode/color/brightness/speed is present; and whether any profile maps the physical Fn/Backspace logical positions differently. Do not access the default profile directory on the user's behalf.
2. **Check the profile provenance and history.** Establish whether a recent profile was edited/imported and whether its values are only local drafts. Phase 12 GUI never applies them to the keyboard, so a profile edit alone cannot explain a physical state change through this application.
3. **Confirm keyboard identity without active protocol traffic.** Use the existing label/packaging/physical markings and the user's known USB/Bluetooth operating mode. The repository's earlier passive inventory observed VID:PID `3151:4010` for the keyboard and `3151:4011` for the receiver, but do not re-enumerate or open devices for this phase.
4. **Compare against the correct vendor manual.** Confirm this is the original TH80 Pro rather than TH80 Pro V2 or another revision. The manual's Fn+Backspace behavior is documented, but manual/firmware revision matching is not established.
5. **Inspect the current keymap/layer hypothesis.** If the user has an existing exported vendor configuration or profile, compare the Fn and Backspace logical assignments across active layers. Do not assume the Phase 12 generic logical position IDs map to physical keys without verified geometry.
6. **Gather non-invasive observations.** Ask the user to note whether the keyboard is wired/2.4G/BT, whether ordinary typing and other Fn shortcuts work, whether the keyboard was power-cycled since the symptom began, and whether the lighting stopped after a specific event. These are observations only; do not ask the user to issue reset/pairing/firmware commands.

The quickest useful next input is an exported configuration/profile from before and after the symptom (if available), plus confirmation of the exact TH80 Pro revision and connection mode. Redact serials or other identifiers if present.

## 5. Evidence required before any hardware-level diagnosis

A future proposal would first need all of the following, independently reviewed:

1. Exact physical model, PCB revision, MCU and RGB-controller identity, and exact firmware build/version for the user's unit.
2. Vendor manual/software compatibility tied to that same hardware/firmware revision.
3. A verified firmware-side dispatch/parser for the proposed opcode and request fields, including whether its SET-before-GET request is read-only, volatile, persistent, or mode-changing.
4. Proven native route-to-handler mapping for the exact operation; resolve the ordinary RPC route ambiguity rather than relying on shared signatures or raw-route candidates.
5. Exact request/response lengths, report ID/interface routing, checksums, response status handling, profile preconditions and bounds from trusted source or matching firmware—not guessed from another model.
6. A written risk/reversibility analysis, backup/recovery path and explicit user review/authorization for one narrowly scoped test. A read-looking command is not exempt from this requirement.

## 6. Hardware test approval boundary

**NOT APPROVED — DO NOT EXECUTE.** This phase authorizes no HID Feature GET/SET, Output report, USB control transfer, unknown HID report, RGB command, `setLightType`, reset, pairing, calibration, firmware update, bootloader operation, device opening or device enumeration. Do not test `0x87` or `0x89` on the keyboard. Do not infer a safe request from the command names.

A future test specification may be prepared only after the evidence gaps above are closed. The user must review that specification and explicitly authorize the specific operation before any hardware test is considered. No such test has been prepared or executed here.

## 7. References

- `PHASE5C_FEATURE_REPORT_CORRELATION.md`
- `PHASE5C_RPC_DISPATCH_RESOLUTION.md`
- `PHASE5D_FEATURE_BUFFER_LAYOUT.md`
- `PHASE5D_HIDAPI_FEATURE_AUDIT.md`
- `PHASE5D_RPC_DISPATCH_AUDIT.md`
- `PHASE7B_COMMAND_EVIDENCE_MATRIX.md`
- `PHASE7B_READ_ONLY_COMMAND_DISCOVERY.md`
- `PHASE8_COMMAND_SEMANTICS_MATRIX.md` (C08–C10)
- `PHASE8_STATIC_EVIDENCE_CLOSURE.md` (sections 4–5, 7–10, 12)
- `EP0_PROTOCOL.md`, `USB_PROTOCOL.md`, `HARDWARE.md`, `RESEARCH.md`
