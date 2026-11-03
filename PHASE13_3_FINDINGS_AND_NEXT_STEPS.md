# Phase 13.3 — Findings and Next Steps

## 1. Executive summary

Phase 13.3 closes the central static data-provenance gap from the profile-indexed lighting state to the candidate output data region. The decisive correction is that **`0x164a6` writes directly to `0x20a78`**, not `0x2077c`. `0x2077c` is a distinct intermediate matrix/state region. `0x153d4` consumes that intermediate state and calls `0x164a6` to populate `0x20a78`.

The recovered memory path is:

```text
0x20400 profile-indexed state
  -> 0x13b92 (active profile + mode lookup / per-cell matrix construction)
  -> 0x2077c (header + six-by-21 per-cell records)
  -> 0x153d4 (separate effect/update dispatch branch)
  -> 0x164a6 (scaled four-byte record writer)
  -> 0x20a78 (8 x 21 x 4-byte records)
  -> 0x16c64 (copy bytes +1..+3 only)
  -> 0x20d18
  -> 0x13526 (16-word frame in 0x20380)
  -> 0x135de / 16 x 0x134f2 / 0x133d2
  -> candidate register writes at 0x000f8720 and 0x000f8705
  -> physical LED connection UNKNOWN
```

The `0x13b92` and `0x153d4` routines are separate branches in state/effect dispatch, not a direct caller/callee pair. Their shared-state producer/consumer relationship is supported by the addresses and accesses. A complete execution schedule from configuration update through physical output is still not proven.

## 2. Strongest discoveries

1. **`0x20a78` layout — `CONFIRMED_STATIC`.** `0x164a6` uses literal `0x20a78` at `0x168c0`, with row stride `0x54`, column stride four, and four stores per record: byte zero control/scale plus three calculated bytes at `+1..+3`. The complete 8×21 region is 672 bytes.
2. **Writer inventory — `CONFIRMED_STATIC` within direct disassembly references.** There are 67 direct `BL 0x164a6` callsites. Five independent zero-fill callsites clear all 672 bytes via `0x1d45e`; `0x16ab2` performs a conditional per-record read/modify/rewrite. Indirect calls and all possible pointer-derived aliases remain a residual limitation.
3. **Profile provenance — `CONFIRMED_STATIC` for memory accesses.** `0x13b92` selects the active profile from `0x20400+0x1a`, reads mode/speed/value/RGB fields, and constructs a six-by-21 matrix in `0x2077c`. Modes `0..6` use a lookup table; mode `7` uses the selected profile's RGB triplet.
4. **Effect/update transformation — `STRONG_STATIC_INFERENCE`.** `0x153d4` consumes `0x2077c` and mode/effect state, calculates per-cell scaling, and calls `0x164a6` at `0x15398`.
5. **Copy — `CONFIRMED_STATIC`.** `0x16c64` copies bytes `+1..+3` from `0x20a78` to `0x20d18` for 8×21 records; it preserves destination byte zero and performs no channel transform.
6. **Output preparation and bit loop — `CONFIRMED_STATIC` for code behavior; `STRONG_STATIC_INFERENCE` for GPIO-like signalling.** `0x13526` builds 16 words in `0x20380`; `0x135de` sends them through 16 calls to the 32-bit loop at `0x134f2`, then calls `0x133d2` and timing/state helpers.

## 3. Fully recovered and unresolved edges

| Edge | Status | Remaining gap |
|---|---|---|
| `0x20400 -> 0x13b92` | `CONFIRMED_STATIC` | Host-side command compatibility/current state unknown |
| `0x13b92 -> 0x2077c` | `CONFIRMED_STATIC` | Full lifetime/ownership of all fields unknown |
| `0x2077c -> 0x153d4` | `STRONG_STATIC_INFERENCE` | Shared-state relation is clear; full scheduler ordering is not |
| `0x153d4 -> 0x164a6 -> 0x20a78` | `CONFIRMED_STATIC` for call and stores | Exact semantics of all effect branches not fully named |
| `0x20a78 -> 0x16c64 -> 0x20d18` | `CONFIRMED_STATIC` | Copy has five direct callers; caller-specific timing remains |
| `0x20d18 -> 0x13526 -> 0x20380` | `CONFIRMED_STATIC` | Exact hardware word encoding remains unknown |
| `0x20380 -> 0x134f2 / 0x133d2` | `CONFIRMED_STATIC` | Timing, pin roles and peripheral identity unknown |
| register stores -> physical LEDs | `UNKNOWN` | No board/pin/power/controller evidence; no physical test permitted |
| configuration update -> scheduled full render | `UNKNOWN` | Opcode `0x08` directly calls partial refresh `0x146f8`; full renderer scheduling not linked |

## 4. Failure hypotheses

| Hypothesis | Supporting evidence | Contradicting / missing evidence | Confidence |
|---|---|---|---|
| Runtime renderer/output callback is not scheduled | `0x103b0` calls `0x135de`, but no direct caller/registration recovered | Indirect callback/timer dispatch may exist | `UNKNOWN` |
| Lighting enable or output state disables frame generation | Output routine branches on state at `0x20340+8`; zero-word branches exist | Meaning of state and active value not established | `WEAK_INFERENCE` |
| Buffer cleared or not repopulated | Five 672-byte zero fills and partial zeroing path exist | Effect routines also populate records; no runtime memory snapshot | `STRONG_STATIC_INFERENCE` as a possible failure point, root cause `UNKNOWN` |
| Active profile/mode state mismatch | `0x13b92` uses active profile and mode-indexed paths | No current device state or proven host-to-opcode-08 call | `WEAK_INFERENCE` |
| Peripheral/GPIO output not initialized or pin mux disabled | Candidate register writes are conditional and initialization is not recovered | Register identity and init path unknown | `UNKNOWN` |
| LED power rail/controller/physical chain fault | Cross-transport symptom can be compatible with shared downstream hardware fault | No board-level/electrical evidence; static binary cannot assess physical integrity | `UNKNOWN` |
| USB-only command transport failure | User reports issue over USB and 2.4 GHz, lowering likelihood of USB-only cause | Shared firmware/receiver path and frontend mismatch remain unresolved | `WEAK_INFERENCE` |

No physical root cause is established. Do not reset, flash, issue lighting commands, change settings, or run vendor software against the keyboard.

## 5. Can a safe RGB implementation specification now be written?

A **static, offline data-model specification** can be written for:

- the profile-indexed state fields and active-profile selector;
- the intermediate `0x2077c` header and six-by-21 four-byte cell records, with unassigned field names where semantics are unclear;
- the `0x20a78` eight-by-21 four-byte records;
- the `0x16c64` copy behavior into `0x20d18`;
- the `0x13526` 16-word frame preparation as a byte-level transform.

A **live hardware-control specification cannot yet be written safely**. Missing evidence includes the exact register/peripheral identity, pin mapping, electrical polarity, controller/protocol identity, frame timing/latch requirements, output-enable and power-control paths, full scheduling path, host command compatibility and current firmware state. No hardware transport or command execution is authorized by these static findings.

## 6. Single most valuable next investigation

Recover the full initialization and callback/scheduler path for `0x103b0` / `0x135de`, including indirect call tables and state gates, then correlate it with the dispatch path that runs `0x13b92` and `0x153d4`. This would establish whether profile/matrix rendering and serial output are actually scheduled together and whether a disabled state can suppress the output path. The next investigation should remain static and must not interact with the keyboard.

A secondary high-value task is to locate a trustworthy MCU register map/board pinout for the raw `0x000f8720` and `0x000f8705` constants; do not infer identity from the numeric address alone.

## 7. Related reports

- `PHASE13_3_RGB_BUFFER_WRITER_ANALYSIS.md` — all identified writers, clears, RMW path and memory layout.
- `PHASE13_3_RGB_RENDERER_DATAFLOW.md` — profile-to-buffer and copy dataflow.
- `PHASE13_3_LED_OUTPUT_PROTOCOL_ANALYSIS.md` — bit-level output evidence and protocol limitations.
- `PHASE13_2_RGB_OUTPUT_PATH.md` — prior candidate output consumer and copy findings; note its old `0x164a6 -> 0x2077c` attribution is superseded by Phase 13.3.
- `PHASE13_1_RGB_HANDLER_ANALYSIS.md` — opcode `0x08`/`0x87` and earlier runtime candidate analysis.

## 8. Preservation and safety

Analysis was performed against the existing raw firmware and temporary Thumb disassembly. The original firmware, native driver, frontend, Rust source, Cargo files and historical reports were not modified. No files were staged or committed; no remotes or Git history were created. No physical keyboard/HID/USB communication, firmware execution, patching, reset, pairing, calibration, bootloader entry or firmware update was performed.
