# Phase 13.2 — findings, closure status and next steps

## 1. Executive summary

Phase 13.2 recovered a concrete staged-data-to-dispatcher transition, the firmware's three report-descriptor slots and lengths, a GPIO-like serial output candidate, and the byte-level behavior of the `0x8c` handler.

The complete HID SET_REPORT-to-dispatcher callback registration remains one edge short of proof. The RGB subsystem now has a plausible matrix-to-serial-output sequence, but the exact producer linking the profile selector and six data bytes to the consumed output triples remains unresolved. The `0x8c` handler is a paged 64-byte response builder, but its data semantics remain unknown; it must not be conflated with the TH80-specific frontend's `0x89` picture path.

## 2. Important discoveries

### USB/HID

1. The class setup handler at `0x1cb48` routes `SET_REPORT` (`bRequest=0x09`) through `0x1cc24 → 0x1cdd4 → 0x1c4ac → 0x1c32a`.
2. The transfer state machine at `0x1c5b6` contains a branch requiring state byte `0x20118 == 3`. That branch copies a length from `0x2011a` bytes at `0x7b018` into the shared application command buffer `0x21240`, updates state through `0x1c274`, then calls dispatcher `0x19c5e`.
3. The exact callback/interrupt registration joining the SET_REPORT setup branch to the state-3 transfer callback has not been recovered. The full path is therefore `STRONG_STATIC_INFERENCE`, while the copy-to-dispatch transition itself is `CONFIRMED_STATIC`.
4. The firmware GET_DESCRIPTOR logic uses three RAM descriptor slots with exact report descriptor lengths 59, 177, and 20 bytes for interface indices 0, 1, and 2. These match the host-captured descriptors in `docs/hid-descriptors.md` exactly by length. The interface-2 host descriptor declares an unnumbered 64-byte Feature report.
5. The actual descriptor bytes were not found embedded in the raw firmware image. The firmware references a RAM-area base at `0x201ac`; its initialization source is unresolved.
6. The supplied native driver's generic libusb Feature SET/GET calls use `wIndex=0`, while the captured TH80 Feature report is on interface 2. This discrepancy remains unresolved. Native checksum construction also differs structurally from the firmware checksum validator.

### RGB

1. Opcode `0x08` at `0x19afe` stores the profile selector plus mode, speed, value, R, G, B in profile-indexed arrays based at `0x20400`. The firmware handler does not read a separate option byte; this differs from the frontend layout that places a packed option before RGB. The paired getter at `0x195ca` reads the same offsets.
2. After the write, `0x19afe` calls `0x146f8`. That routine iterates a 7×15 coordinate grid, calls `0x164a6`, then calls `0x16c64`.
3. `0x16c64` copies three bytes per entry for an 8×21 record grid from `0x20a78` to `0x20d18` using a row stride of `0x54`.
4. Runtime routine `0x135de` calls `0x13526`, which consumes `0x20d18` and builds 16 words at `0x20380`. It then calls `0x134f2` for those words and uses `0x133d2` as an additional control/latch-like operation.
5. `0x133d2` and `0x134f2` write bit-serial patterns to peripheral-like register addresses based at `0x000f8720`, with additional control at `0x000f8705`. This is strong static evidence for a GPIO-driven serial output stage. The pin identity, controller, electrical protocol and actual LED connection remain unknown.
6. The producer of source buffer `0x20a78` has not been tied directly to the profile selector and six command data bytes. `0x164a6` writes a different buffer at `0x2077c`; no direct data copy from that buffer to `0x20a78` was recovered. This prevents claiming a fully closed `0x08 → physical LED` data path.

### Opcode `0x8c`

1. The dispatch entry calls handler `0x1951e`.
2. It reads request bytes `[1]` and `[2]`, calculates `0x01029100 + ([1] << 8) + ([2] << 6)`, and builds a 64-byte response descriptor.
3. Its response-valid flag is set only when byte `[2] <= 4`.
4. A generic frontend constant labels `0x8c` as GET_USERPIC and one generic model loops through six 64-byte chunks. The TH80-specific `_getLightPic` path uses literal `0x89`, not `0x8c`. The `0x8c` handler's semantic identity remains unknown.

## 3. Completed call/data-flow paths

### Confirmed static paths

```text
0x08 dispatch -> 0x19afe -> profile state at 0x20400
                                  -> 0x146f8
                                      -> 0x164a6 (writes 0x2077c)
                                      -> 0x16c64 (copies 0x20a78 -> 0x20d18)

0x103b0 -> 0x135de -> 0x13526 (reads 0x20d18; writes 0x20380)
                   -> 16 x 0x134f2 (bit-serial GPIO-like writes)
                   -> 0x133d2 (additional GPIO-like operation)

0x8c dispatch -> 0x19dd6 -> 0x1951e
                            -> request-indexed source pointer
                            -> 64-byte response descriptor

transfer callback state 3:
0x7b018 -> 0x21240 -> 0x1c274 -> 0x19c5e -> 0x19c1e
```

### Strong but not fully closed

```text
HID SET_REPORT setup:
0x1cb48 -> 0x1cc24 -> 0x1cdd4 -> 0x1c4ac -> 0x1c32a

candidate completion callback:
0x1bd54 -> 0x1c5b6
  state 3 -> 0x7b018 to 0x21240 -> 0x19c5e
```

The missing edge is the exact callback registration/interrupt dispatch from the SET_REPORT data stage to wrapper `0x1bd54`/state value 3.

## 4. Exact unresolved transitions

1. **USB callback registration:** identify the registration site and callback table entry that routes the EP0 SET_REPORT data stage into the `0x1c5b6` state machine. `0x1bd54` has no recovered direct call site; it may be invoked indirectly by the USB stack.
2. **Descriptor initialization:** locate the startup routine or encoded source that populates RAM descriptor area `0x201ac` and its adjacent report/class descriptor slots.
3. **RGB source producer:** identify all writers of `0x20a78` and prove how mode/speed/value/RGB state at `0x20400` affects those bytes.
4. **RGB scheduler:** identify the caller/registration that periodically invokes `0x103b0` and establish whether it runs in USB, wireless, or common application contexts.
5. **GPIO mapping:** identify the MCU peripheral and pin mapping for raw addresses `0x000f8720` and `0x000f8705` only if supported by a board/SDK register map or further static references. Do not infer an LED controller from the waveform alone.
6. **Opcode `0x8c` data region:** identify the memory map/owner of `0x01029100` and whether it is flash, mapped RAM, or another memory space. Do not infer picture/keymap semantics from the generic frontend constant.
7. **USB/wireless comparison:** locate independent receiver/radio initialization and compare the exact RGB state and output callbacks. Existing identical host HID descriptor captures do not prove identical firmware runtime paths.

## 5. What the evidence says about the physical RGB failure

The binary contains a concrete software-controlled, bit-serial GPIO-like output sequence. That rules out the earlier assumption that no candidate physical output stage could be identified statically. It does not establish that the output routine is currently scheduled, that its source buffer is valid, that the pin/peripheral is enabled, or that the LED chain/controller receives a valid waveform.

The user-reported failure on both USB and 2.4 GHz makes a purely USB-only failure less likely, but is not enough to conclude that the MCU, GPIO, LED controller, LED power rail, or LED chain has failed. The current evidence cannot distinguish firmware state/scheduler failure from electrical or peripheral failure.

## 6. Rust application boundary and future hardware-test safety

Useful offline additions are supportable:

- Decode and display the confirmed profile selector plus mode/speed/value/RGB structure.
- Parse the captured host HID descriptors and report IDs/lengths.
- Document the generic 64-byte page-response layout without assigning unsupported semantics.
- Visualize the known 8×21 triple-record and 16-word output structures as offline data models, clearly marking the unknown source semantics.

Do not implement or enable Feature transport, report writes, opcode execution, profile changes, reset, pairing, calibration, bootloader, or firmware update based on these findings. The native transport/interface mismatch and unresolved callback edge make active commands especially inappropriate. A future hardware test must be designed and reviewed separately, with a safe read-only method established from independent evidence; no hardware test is authorized in Phase 13.2.

## 7. Single most valuable next investigation

Trace all writers of `0x20a78` and follow their inputs back to the profile selector plus mode/speed/value/RGB state at `0x20400`. This is the missing link between the confirmed lighting configuration writer and the newly identified GPIO-like serial output path. In parallel, preserve the USB callback-registration gap as a separate blocker rather than conflating the two investigations.

## 8. Deliverable cross-references

- `PHASE13_2_USB_FEATURE_BRIDGE.md` — setup request, staged buffer, dispatcher and descriptor evidence.
- `PHASE13_2_RGB_OUTPUT_PATH.md` — profile state, matrix records and GPIO-like output candidate.
- `PHASE13_2_OPCODE_8C_ANALYSIS.md` — byte-level paged response handler and frontend non-match.

## 9. Integrity and safety statement

The original firmware and native-driver hashes were verified against the Phase 13.1 documented values before analysis. The final verification records the same hashes, the frontend hash, the four Phase 13.2 report inventory and final Git status. No source code, Cargo manifest, firmware, native driver, frontend bundle, historical report, Git index, Git history, remote, or physical keyboard was modified or operated. No files were staged or committed.
