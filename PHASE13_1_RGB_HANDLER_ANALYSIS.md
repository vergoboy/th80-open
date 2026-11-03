# Phase 13.1 — RGB Command Handlers, Runtime and Persistence Analysis

**Scope:** static firmware analysis only. No physical keyboard, receiver, HID interface or USB endpoint was accessed. No command was sent, no firmware was executed or modified, and no persistent configuration was changed.

## 1. Executive conclusion

The firmware contains a coherent full lighting-parameter write/read pair in the candidate command dispatcher:

- `0x08` → handler `0x19afe`: writes seven lighting fields into a profile-indexed state structure and invokes a downstream calculation/update routine.
- `0x87` → handler `0x195ca`: reads the corresponding fields from that same state structure and constructs a 64-byte response.

This is a strong firmware-side correlation with the vendor frontend's seven-byte lighting-parameter layout. It does **not** validate the frontend's `0x07` setter: firmware `0x07` maps to a different compact structure and does not consume RGB bytes `[5..7]`.

The `0x89` entry in the recovered upper dispatch table resolves to a return-only epilogue at `0x19cf8`; no command-specific handler is present at that entry in this dispatcher. The frontend's per-key picture and keymap operations therefore remain uncorrelated with this firmware path. A separate `0x8c` handler exists, but its semantic identity is not established.

The candidate routines `0x146f8`, `0x164a6`, `0x16c64`, `0x16d2c`, `0x164f6` and `0x16566` perform substantial record and buffer transformations. Only `0x146f8` is a proven direct callee of the full `0x08` writer; the other routines' call relationships are distinct or unresolved. None proves physical LED output, PWM/GPIO signalling, a controller IC, or persistence.

## 2. Full lighting parameter write — opcode `0x08`

### Dispatch evidence

- Dispatch table: file offset `0x19c82`.
- Opcode `0x08` table index: 8.
- Entry byte: `0x42`.
- Computed target: `0x19c82 + 2 * 0x42 = 0x19d06`.
- `0x19d06` calls handler `0x19afe`.

Classification: `CONFIRMED_STATIC` for the table entry and call target under the candidate Thumb decode.

### Handler behavior at `0x19afe`

The handler uses the shared command buffer pointer literal at `0x19b4c` (`0x00021240`) and a state-base pointer literal at `0x19b50` (`0x00020420`), subtracting `0x20` to obtain candidate state base `0x00020400`.

It reads request byte `[1]` as a profile/index selector, stores it at state-base offset `0x1a`, then writes:

| Request byte | Stored field | Candidate state address |
|---:|---|---|
| `[1]` | profile/index selector | `base + 0x1a` |
| `[2]` | first parameter (frontend mode position) | `base + 0x40 + index` |
| `[3]` | second parameter (frontend speed position) | `base + 0x60 + index` |
| `[4]` | third parameter (frontend value position) | `base + 0x80 + index` |
| `[5]` | first color byte | `base + 0xa0 + 3*index` |
| `[6]` | second color byte | `base + 0xa1 + 3*index` |
| `[7]` | third color byte | `base + 0xa2 + 3*index` |

After storing the final color byte, the handler updates status flags and calls `0x146f8` at `0x19bac`, then marks the update state and returns. The profile-indexed state and seven-byte field layout are directly visible in the stores.

Classification:

- `CONFIRMED_STATIC`: the handler reads bytes `[1..7]` and stores the seven fields at the indexed state offsets.
- `STRONG_STATIC_INFERENCE`: the fields represent the frontend's mode/speed/value/option/RGB structure. Exact semantic meanings and numeric ranges are not proven by the firmware stores alone.
- `UNKNOWN`: whether these values are saved to nonvolatile memory or immediately driven to physical LEDs.

## 3. Full lighting parameter read — opcode `0x87`

### Dispatch evidence

The dispatcher at `0x19c5e` compares the command byte against `0x87`, branches to `0x19dbe`, and calls handler `0x195ca`.

Classification: `CONFIRMED_STATIC` for the direct opcode comparison and call target.

### Handler behavior at `0x195ca`

The handler uses the same candidate state base `0x00020400` (literal at `0x19730` is `0x00020420`, then `-0x20`) and reads the active index from `[base + 0x1a]`.

It copies the indexed fields into the shared response buffer:

| Response byte | Source |
|---:|---|
| `[1]` | active index/profile selector |
| `[2]` | `base + 0x40 + index` |
| `[3]` | `base + 0x60 + index` |
| `[4]` | `base + 0x80 + index` |
| `[5]` | `base + 0xa0 + 3*index` |
| `[6]` | `base + 0xa1 + 3*index` |
| `[7]` | `base + 0xa2 + 3*index` |

The response-control structure at candidate address `0x000200d8` is updated with a zero status/offset field and length `0x40` (64 bytes).

This is a firmware-side read of the same structure written by `0x08`. It is not proof that the vendor frontend's complete `commomFeature` transaction is safe: the frontend submits a Feature SET before its Feature GET, and the transport/dispatch bridge remains unresolved.

Classification: `CONFIRMED_STATIC` for reading the indexed structure and preparing a 64-byte response; `UNKNOWN` for the complete transport transaction's side effects and exact host/firmware compatibility.

## 4. Separate compact parameter pair — `0x07` and `0x86`

### Opcode `0x07` write handler at `0x19bb6`

Dispatch:

- Table base `0x19c82`.
- Entry index 7 = `0x3f`.
- Computed target `0x19d00`.
- `0x19d00` calls `0x19bb6`.

The handler reads request byte `[1]` as an index, combines bytes `[2]` and `[3]` into a 16-bit value, stores the low/high bytes at `base + 0x0a + 2*index` and `base + 0x0b + 2*index`, and stores byte `[4]` at `base + 0x1c`. It does not read request bytes `[5]`, `[6]` or `[7]` in this handler.

### Opcode `0x86` read handler at `0x19614`

The direct `0x86` branch calls `0x19614`. It reads the corresponding compact state and constructs response bytes `[2]`, `[3]` and `[4]`, then sets response length to 64 bytes.

### Consequence for frontend correlation

The TH80 frontend's `setLightSetting` builder uses opcode `0x07` and places mode, speed, value, option and optional RGB in bytes `[1..7]`. The firmware `0x07` handler instead treats byte `[1]` as an index and consumes only bytes `[2..4]` into a different compact state layout. This is a real structural discrepancy in the available static evidence.

Do not relabel firmware `0x07` as the full RGB writer, and do not infer that the frontend's setter reaches the `0x08` handler. Possible explanations include a firmware/frontend revision mismatch, a model-dialect mismatch, or another role for the compact structure; none is resolved.

Classification: `CONFIRMED_STATIC` for the compact write/read behavior; `FRONTEND_ONLY` for the frontend's seven-field `0x07` request; `UNKNOWN` for compatibility and intended semantics.

## 5. Opcode `0x89` and adjacent `0x8a–0x8d`

The upper dispatch table at `0x19cda` uses the command index `opcode - 0x89`:

| Opcode | Target | Handler entry |
|---:|---:|---:|
| `0x89` | `0x19cf8` | Return-only epilogue; no command-specific call |
| `0x8a` | `0x19dca` | `0x1956e` |
| `0x8b` | `0x19dd0` | `0x1954e` |
| `0x8c` | `0x19dd6` | `0x1951e` |
| `0x8d` | `0x19ddc` | `0x194e0` |

The `0x8c` handler at `0x1951e` reads request bytes `[1]` and `[2]`, calculates a data pointer using a table literal at `0x1973c` (`0x01029100` under the unconfirmed base hypothesis), sets a 64-byte response length and updates response fields. This is consistent with a data/page response but does not prove a user-picture or keymap meaning.

The frontend uses `0x89` in two different TH80 contexts:

1. Per-key RGB/user-picture retrieval: request `[0x89, 0]`, followed by additional Feature GETs parsed as RGB triples.
2. Keymap retrieval: request `[0x89, profile, page]`, repeated for pages 0–7.

Those host-side request and parser details are `FRONTEND_ONLY`. The recovered firmware dispatch entry for `0x89` is return-only, so neither frontend context has a firmware handler established here. The existence of a separate `0x8c` page-like handler is a research lead, not permission to substitute opcodes or infer that it implements the frontend picture operation.

Classification: `STRONG_STATIC_INFERENCE` that `0x89` has no handler in this dispatcher; `UNKNOWN` whether another firmware path or build supports either frontend operation.

## 6. RGB runtime and buffer-processing candidates

The following functions are connected to the full lighting write handler or have code references consistent with color/matrix processing. Their exact role is constrained by the observed instructions, not by invented semantic names.

### `0x16d2c` — separate 12-entry calculation candidate

The routine iterates 12 entries, reads paired values from a table near `0x170d8`, and calls `0x164f6` for each entry. Direct callsites occur at `0x17352` and `0x19af2`. The latter is immediately before the separate handler entry at `0x19afe` and cannot be attributed to the `0x08` handler from the current function-boundary evidence. Do not describe `0x16d2c` as a proven callee of the full lighting writer.

The full lighting writer's proven downstream call is instead `0x19bac → 0x146f8`.

### `0x164f6` — indexed component calculation/store

The routine:

- rejects coordinate/index values of `0xff`;
- performs integer multiplication and scaling using a divisor/scale argument of `0x64` (100);
- writes three computed byte values and a fourth input byte into a record addressed using a stride of `0x54` and a four-byte per-index step.

This is a color/component or graphical-record transformation candidate. It does not directly identify the LED output mechanism.

### `0x16566` — compact record transformation

The routine reads a record selector byte and handles cases `0..3`, modifying adjacent byte fields with bounded arithmetic. It is called from many locations across the image. No direct connection to a Fn shortcut or a named LED controller was found.

### `0x146f8` and `0x164a6` — grid/record rendering candidate

`0x146f8` loops over six rows and fifteen columns, calls `0x164a6` for each cell and then calls `0x16c64`. `0x164a6` performs scaled arithmetic and writes three calculated byte components plus an index into a record addressed using a `0x54` stride. Numerous functions call `0x146f8`, indicating a shared transformation routine rather than a dedicated command handler.

### `0x16c64` — bulk three-byte record copy

The routine copies three bytes for 21 entries across eight outer iterations: `8 × 21 × 3` bytes in total, using a per-row stride of `0x54`. This may represent a matrix/frame or another structured color buffer. No physical key ordering or TH80 geometry is established.

### Output-stage boundary

The inspected candidate routines perform memory calculations, record writes and bulk copies. They do not, in the recovered call paths, directly demonstrate:

- a GPIO pin assignment for LED data;
- a timer/PWM channel used by RGB;
- a bit-banged addressable-LED waveform;
- an SPI/I2C/DMA transfer to a controller;
- a distinct LED driver IC;
- an LED power-enable or rail-gating signal.

The presence of Yichip SDK strings such as `yc_gpio.c`, `yc_timer.c`, `GPIO_Init`, `TIM_PWMInit` or `TIM_SetPWMPeriod` establishes SDK material/capability, not RGB use. The exact LED output stage remains `UNKNOWN`.

## 7. Persistent configuration

The full configuration writer and reader operate on a shared state structure based at the literal-derived candidate address `0x00020400`. The writer updates indexed fields in this structure; the reader returns those fields. This is evidence of a profile-indexed in-memory representation.

No direct flash/EEPROM/NVM call is visible in the `0x08` writer or `0x87` reader. The firmware string scan found no clear `flash`, `EEPROM`, `NVRAM`, `NVM`, `erase`, `save` or `load` API names. Strings such as `SSC_LPMKeyRead` and `SSC_LPMKeyWrite` exist but their semantics and connection to lighting are not established.

Therefore:

- volatile/profile-indexed state: `CONFIRMED_STATIC`;
- a persistent lighting record or flash save path: `UNKNOWN`;
- restoration across USB reconnect, wireless reconnect, power cycle or profile switch: `UNKNOWN`;
- whether the candidate RAM state is initialized from persistent defaults elsewhere: `UNKNOWN`.

Do not describe the `0x08` handler as persistent merely because it updates a profile-indexed state structure.

## 8. Firmware/frontend correlation matrix

| Host operation | Host evidence | Firmware evidence | Conclusion |
|---|---|---|---|
| Full parameter read `0x87` | TH80 frontend parses bytes `[1..7]` as mode/speed/value/option/RGB | `0x87 → 0x195ca` reads the same seven positions from profile-indexed state | `STRONG_STATIC_INFERENCE` of field-layout correlation; full transport and side-effect safety unresolved |
| Full parameter write `0x07` | TH80 frontend sends seven fields in bytes `[1..7]` | Firmware `0x07 → 0x19bb6` reads compact fields `[1..4]` into a different structure | `FRONTEND_ONLY` compatibility claim; actual firmware mismatch is `CONFIRMED_STATIC` at the field-layout level |
| Full parameter write `0x08` | No matching TH80 frontend setter established in the reviewed reports | Firmware `0x08 → 0x19afe` writes the full seven fields and calls `0x146f8` | `CONFIRMED_STATIC` firmware behavior; host invocation unknown |
| Per-key picture `0x89` | `[0x89,0]` plus repeated GETs; RGB-triple parser | Dispatch table target is return-only | `FRONTEND_ONLY`; firmware support unknown/unproven |
| Keymap `0x89` | `[0x89,profile,page]`, pages 0–7 | Same return-only dispatch entry | `FRONTEND_ONLY`; firmware support unknown/unproven |
| Adjacent `0x8c` | Generic frontend constant exists, but TH80 picture path uses literal `0x89` | Page-like 64-byte response handler at `0x1951e` | `STRONG_STATIC_INFERENCE` of data-response behavior; semantic identity unknown |

## 9. Next static targets

1. Trace the `0x16d2c` callers, including `0x17352`, and continue toward any hardware-facing output function. Keep GPIO/PWM/SPI/I2C hypotheses separate until an actual call/data path proves them.
2. Trace the pointer/table source used by `0x1951e` (`0x8c`) and identify whether its data area is a matrix, picture, keymap or unrelated object.
3. Locate flash/NVM APIs and all references to the candidate state base `0x00020400`; determine whether any load/save path exists without modifying storage.
4. Continue the USB Feature SET bridge investigation separately; the RGB command handler evidence does not close the HID transport gap.
5. Obtain exact board/firmware identity only from already available static documentation/artifacts. Do not flash or communicate with the keyboard.

## 10. Safety boundary

No `0x07`, `0x08`, `0x86`, `0x87`, `0x89` or `0x8c` operation was tested. No report was sent or received. No hardware reset, profile change, pairing, calibration, bootloader operation, firmware update or persistent write was performed. Hardware testing remains unauthorized.
