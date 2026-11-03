# Phase 13.4 — RGB Runtime Gating

## Scope

This report correlates runtime state, initialization and output gating with the Phase 13.3 data path. It does not infer physical LED behavior from a software register write and does not label undocumented state bytes as USB, wireless, battery or sleep flags.

## 1. Shared state byte at `0x20348`

The main loop and output routine both read byte `0x20340 + 8` (`0x20348`).

### Main-loop use

At `0x1030c`, the loop reads `[r4+8]` where `r4` is loaded from literal `0x20340`. It dispatches values including `0..6`, `0xfd` and `0xfe` to different handlers. The `0xfd` branch calls `0x11328`; the `0xfe` branch calls `0x114f2`.

### Output use

At `0x135e0–0x135ec`, `0x135de` loads the same byte from `[r6+8]`, where `r6` is the pointer literal `0x20340`, and compares it with `0xfd`. If equal, it branches around the normal frame-preparation/transmission path. Otherwise it proceeds using the output index at `0x20076` and the frame words at `0x20380`.

This proves a shared software gate: while the state byte equals `0xfd`, the normal `0x135de` frame path is skipped. The main loop also dispatches state `0xfd` to a separate transition handler.

**Classification:** `CONFIRMED_STATIC` for both reads, the equality test and branch behavior; `STRONG_STATIC_INFERENCE` that `0xfd` is a subsystem transition/disabled state for this output path. The semantic name of the state is unknown.

## 2. State-`0xfd` transition path

The main loop calls `0x11328` when `0x20348 == 0xfd`.

Within `0x11328`:

- A pending flag at `[state+0x16]` causes the routine to clear the 672-byte candidate buffer through `0x1d45e`.
- It updates peripheral/timer-related configuration, including a call to `0x173cc` at `0x11360`.
- It modifies other state bytes and invokes several output/control helpers.
- A later conditional path calls `0x164a6` and `0x16c64` when a flag at `[state+0x1c]` is set.

This establishes that state `0xfd` can trigger buffer clearing and timer reconfiguration. It does not establish that the output wrapper `0x103b0` is called during or after this transition.

**Classification:** `CONFIRMED_STATIC` for the state branch, clear, timer call and conditional record/copy calls; `UNKNOWN` for whether the transition schedules a subsequent output frame.

## 3. Candidate application-init output initialization

Within the candidate application-init block at `0x1027e`, `0x13400` is called at `0x102ee`. That block falls through to the polling loop, but its entry from the reset stubs is unresolved. Independently, `0x13400` has additional direct callers in later transition paths. It clears the full 672-byte region based at the Phase 13.3 candidate `0x20a78`. It then writes several register-like values around `0x000f8700`, invokes `0x133d2` and other control helpers, and resets an output-related byte. It is also called from four additional transition callsites (`0x1096a`, `0x1167e`, `0x116f4`, `0x1173e`).

This is evidence that output-subsystem initialization/reinitialization code exists. The callsite in the candidate init block is not proof that the block executes on every boot. It is not proof that the external LED power rail is enabled or that a signal reaches the LED chain.

**Classification:** `CONFIRMED_STATIC` for the code operations; `STRONG_STATIC_INFERENCE` for candidate output-subsystem initialization; `UNKNOWN` for physical LED power and connection.

## 4. Timer and output activation are not yet joined

`0x173cc` is called inside the candidate application-init block and from later transition paths; the candidate block’s entry from reset is unresolved. It enables NVIC interrupt index 5 through `0x1d304`, configures a timer-like peripheral selected by `0x000f8b00`, and uses the scalar `500000` plus a `48,000,000` constant to derive a quotient of 96. The timer's exact interval and callback are unknown.

The interrupt-vector stubs at `0x1001c–0x10098` use a dynamic table at observed base `0x010102bc`. The index-5 stub supplies offset `0x14`; its runtime handler pointer is unavailable. No static pointer to `0x103b0` was found.

Thus timer setup and an interrupt-dispatch framework are confirmed, but the chain `timer IRQ 5 → 0x103b0 → 0x135de` remains `UNKNOWN`.

## 5. Buffer-zero versus missing-invocation distinction

The two failure modes are different:

1. **Renderer/output callback not invoked:** `0x103b0` has no recovered direct caller or static pointer registration. If it is not invoked, the `0x135de` frame path will not run through this wrapper even if `0x20a78` contains valid records.
2. **Buffer deliberately cleared or remains zero:** `0x13400` clears `0x20a78` in the candidate application-init block (whose reset reachability is unresolved), and `0x11328` can clear it during the `0xfd` transition. A clear is not itself proof of a failure; later renderer activity may repopulate the region.
3. **Output path invoked but gated:** `0x135de` skips its normal frame path when `0x20348 == 0xfd`, even if data exists in `0x20a78`/`0x20d18`.
4. **Output path invoked with stale/zero data:** the dataflow path can be present while the producer has not repopulated the candidate buffer; no runtime observation is available to distinguish this from a valid black frame.

**Classification:** `CONFIRMED_STATIC` for clear and gate instructions; `UNKNOWN` for actual runtime buffer contents and invocation cadence.

## 6. Profile and effect dependencies

Phase 13.3 recovered the configuration/rendering data path:

```text
0x20400 profile-indexed lighting state
  -> 0x13b92 active-profile processing
  -> 0x2077c intermediate matrix
  -> 0x153d4 effect/update processing
  -> 0x164a6 record generation
  -> 0x20a78
  -> 0x16c64
  -> 0x20d18
```

Phase 13.4 did not recover a scheduler edge from the configuration handler or profile/effect routines to `0x103b0`. The full lighting opcode handler at `0x19afe` calls `0x146f8`, but no direct call from it to `0x103b0` was found. The data-production chain and output-consumption wrapper must therefore remain separate in the runtime model.

Mode, profile and brightness fields can affect generated records as documented in Phase 13.3. This phase does not prove that each field is checked before scheduling or that a configuration change immediately triggers a frame.

**Classification:** `CONFIRMED_STATIC` for the previously recovered dataflow and direct calls; `UNKNOWN` for the configuration-change-to-output scheduling edge.

## 7. USB, wireless and power-management gates

No direct condition in `0x103b0` or `0x135de` was resolved to a USB-connected flag, 2.4 GHz mode flag, battery level, charging state, low-battery state, sleep flag or wake flag.

The function `0x1094c` has several direct callers and invokes `0x173cc` as part of a state-dependent path; those callers do not establish that the timer reconfiguration is specifically USB- or wireless-owned. The `0x11328` state transition also reconfigures the timer, but its state value must not be renamed as sleep or wake without independent evidence.

The timer-status helpers used by `0x103d0` and `0x10418` inspect timer-like registers at `0x000f8c00` / `0x000f8b00`. They do not establish a keyboard power-management state.

The host-side observation that RGB shortcuts fail over both USB and 2.4 GHz lowers the likelihood of a strictly USB-host transport fault, but does not identify a shared MCU, receiver, power or LED-chain fault.

**Classification:** `UNKNOWN` for USB/wireless/power gating of RGB; `WEAK_INFERENCE` that the reported cross-mode symptom points toward a shared path rather than an exclusively wired-host issue.

## 8. Failure hypotheses

| Hypothesis | Supporting static evidence | Contradicting evidence | Missing evidence | Confidence |
|---|---|---|---|---|
| Output wrapper `0x103b0` is never invoked | Zero direct callers; no static absolute pointer to `0x103b0`; indirect vector target unresolved | A dynamic interrupt/callback table exists and could invoke it | Runtime table contents and registration writer | `UNKNOWN` |
| IRQ 5 is the RGB refresh interrupt | Timer setup enables NVIC IRQ 5; vector stub has slot `+0x14`; `0x103b0` is a plausible output wrapper | No slot target resolved; timer purpose and period unknown | Runtime vector entry, timer mapping and callback semantics | `WEAK_INFERENCE` |
| State `0xfd` suppresses normal frame output | `0x135de` explicitly branches around normal output when `0x20348 == 0xfd`; main loop dispatches same state to `0x11328` | No evidence the state remains `0xfd` during the reported failure | State transitions and actual runtime value | `STRONG_STATIC_INFERENCE` for suppression behavior; `UNKNOWN` as root cause |
| Candidate output buffer remains zero | `0x13400` and `0x11328` clear all 672 bytes | Renderer paths can write the region later | Whether renderers run after each clear and actual data | `UNKNOWN` |
| Output peripheral initialization is skipped | `0x13400` performs candidate register setup and is called in the candidate init block and from transition paths | The code path exists in the image | Whether the init block is reached, whether the path completes, and whether register meanings/pin mux/power are correct | `UNKNOWN` |
| Timer is not initialized after a state transition | `0x173cc` is called in the candidate init block and multiple later transition paths | Timer reconfiguration code is present | Whether the init block is reached, whether all relevant transitions reach it, and whether the timer starts successfully | `WEAK_INFERENCE` |
| USB/wireless transition disables shared RGB | Several state handlers reconfigure timer/output state | No direct USB/wireless flag-to-RGB gate found | Radio/USB state mapping and shared subsystem ownership | `UNKNOWN` |
| Sleep/wake restoration fails | No proven sleep/wake-to-output restoration chain | Startup and transition reinitialization paths exist | Sleep/wake state fields, callback path and wake handler | `UNKNOWN` |

## 9. Interpretation of user-reported failure

The firmware contains:

- Candidate output initialization (`0x13400`).
- Timer configuration and NVIC interrupt enable (`0x173cc`, `0x1d304`).
- A vector-dispatch mechanism with a runtime handler table.
- An output wrapper (`0x103b0`) and frame transmitter (`0x135de`).
- An explicit state value (`0xfd`) that suppresses normal output.

But it does not reveal whether the actual runtime handler table invokes `0x103b0`, whether state `0xfd` is active on the user's device, whether the buffer is populated, or whether the output register writes reach the physical LEDs. The physical root cause remains unknown.

## Related reports

- [Runtime entry and call graph](PHASE13_4_RGB_RUNTIME_ENTRY_AND_CALLGRAPH.md)
- [Scheduler, timer and initialization](PHASE13_4_SCHEDULER_TIMER_AND_INITIALIZATION.md)
- [Findings and next steps](PHASE13_4_FINDINGS_AND_NEXT_STEPS.md)
- [Phase 13.3 RGB buffer writer analysis](PHASE13_3_RGB_BUFFER_WRITER_ANALYSIS.md)
