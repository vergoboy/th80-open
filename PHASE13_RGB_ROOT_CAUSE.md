# Phase 13 — RGB Root-Cause Assessment

**Status:** static root-cause assessment complete; no physical diagnosis can be made from current evidence. Hardware testing is **NOT APPROVED**.

## 1. Reported symptoms

The user reports:

- physical TH80 Pro RGB/backlighting is off;
- `Fn + Backspace` no longer restores lighting;
- other RGB shortcuts no longer work;
- most Fn shortcuts have stopped responding, while the USB-mode switching shortcut still works;
- the symptom persists over USB and 2.4 GHz.

These are user observations, not independently reproduced device states. No active device query or physical inspection was performed in Phase 13.

## 2. What the evidence establishes

1. The official TH80 quick guide documents `Fn + Backspace` as backlight on/off and `Fn + Backslash` as effect cycling. It also warns that using the wrong Windows/macOS keyboard mode after changing systems can make some shortcuts/multimedia keys fail. This guide is family-level documentation, not proof of the exact user's V504 firmware revision or current Fn-layer state.
2. EPOMAKER's TH80 Pro product material describes three editable layers, a programmable Fn layer and per-key RGB. It warns V504 is for ANSI V500/V500+ only, not V2xx or ISO units.
3. The V504 package's support configuration includes the keyboard's known normal-mode USB identity `3151:4010`, vendor interface 2, and bootloader PID `3151:4001`. This identifies a supported USB product tuple, not the user's exact PCB revision or installed firmware.
4. The supplied raw firmware contains Yichip SDK GPIO/timer/PWM material but no recovered Fn+Backspace handler, lighting state machine, LED driver, RGB controller mapping or persistent lighting structure.
5. The frontend has a lighting setting command (`0x07`) and a lighting-parameter request (`0x87`), but the `0x87` request uses Feature SET followed by Feature GET. Neither `0x87` nor either `0x89` path is firmware-correlated or proven read-only.
6. Phase 12.2 found that the Rust application is offline-only. It has no device write path and cannot have switched the physical keyboard's LEDs off. Its color-preview ambiguity was fixed; opening a local profile does not convert unspecified lighting into disabled lighting.
7. The user's cross-transport symptom makes a USB-only host transport failure less likely, but does not isolate the shared keyboard MCU, receiver, firmware, power rail or LED subsystem.

## 3. Ranked hypotheses

Ranks express investigation priority, not a diagnosis.

| Rank | Hypothesis | Supporting evidence | Contradicting/limiting evidence | Assessment |
|---:|---|---|---|---|
| 1 | Fn-layer/keymap, Fn dispatch or OS-mode state is abnormal | Multiple Fn shortcuts reportedly fail; TH80 Pro supports programmable layers and Fn mappings; the TH80 family guide warns that OS-mode mismatch can break some shortcuts; the `0x89` keymap path is frontend-observed but no actual device keymap is known | No current profile/keymap or OS-mode state was read; USB-mode switching still works, showing at least some mode/input handling remains functional; OS-mode mismatch alone may not explain RGB shortcuts | **Most useful first hypothesis; unconfirmed** |
| 2 | Persistent firmware lighting state is off, invalid or at minimum brightness | Manual documents a dedicated backlight toggle; frontend models off/mode/value/speed/color fields; RGB is off across transports | Other RGB shortcuts also reportedly fail, which is not explained by a simple off state alone; no current state values exist | **Plausible; unconfirmed** |
| 3 | Firmware configuration or firmware revision mismatch/corruption | Firmware/board revision and currently installed firmware are unknown; vendor explicitly restricts V504 to ANSI V500/V500+ and warns against V2xx/ISO cross-flashing | No evidence that the user recently flashed firmware or that the supplied V504 image is installed | **Plausible; no causal evidence** |
| 4 | LED initialization, LED power gating or controller state failure | A separate LED subsystem could fail while ordinary keyboard input remains functional; firmware-side power/sleep/wake logic is not recovered | No LED-controller identity, rail measurement, startup trace or physical inspection exists | **Possible; unknown** |
| 5 | Physical LED chain, connector, driver IC or power-path fault | All LEDs being dark can result from a shared supply/driver/connection failure independent of key scanning | No visual inspection or electrical observation; shortcut failures may indicate a separate or upstream issue | **Possible; no direct evidence** |
| 6 | USB-only transport/host issue | Vendor Feature transport has unresolved interface-index and route-mapping issues | Symptom reportedly persists over both USB and 2.4 GHz; those host transport differences do not explain a local keyboard Fn/LED failure by themselves | **Lower priority, not ruled out** |

### Interpretation of the working USB-mode shortcut

A working mode-switch action is evidence that the keyboard is not wholly unresponsive and that at least one mode-selection/input path remains operational. It does **not** prove that all Fn-layer dispatch, keymap lookup, lighting state handling or LED initialization paths work. Different shortcuts can be handled by different firmware paths or keymap entries.

### Why the symptom pattern does not isolate a cause

RGB output and Fn shortcut processing are related at the product level but need not share one firmware function. The RGB renderer may continue to operate while the Fn shortcut that toggles it is broken; conversely, Fn input may be processed correctly while the LED power/driver path is dead. A shared MCU can host both functions without making their failure modes identical.

## 4. What can be ruled out or deprioritized

- **The Rust GUI as the direct cause:** ruled out by architecture audit. It edits local profiles only and has no device communication path.
- **A local profile edit from this application as a physical write:** ruled out. Phase 12.2 confirms no device-apply behavior.
- **A purely USB-only fault as the complete explanation:** deprioritized by the reported USB and 2.4 GHz symptom, although host transport and receiver behavior are not fully excluded.
- **A completely unresponsive keyboard:** inconsistent with the reported working mode-switch action, but partial firmware or peripheral failures remain possible.
- **A safe `0x87`/`0x89` recovery operation:** not established. The frontend calls are SET-before-GET and no firmware semantics or side effects are known.
- **A safe reset, hidden Fn chord or V504 reflash:** not established. No such action is recommended.

Nothing in the available evidence rules out a failed LED driver, LED power rail, LED chain, corrupted persistent setting, or selective firmware/keymap defect.

## 5. Restoration feasibility

There is currently **no safe, evidence-backed restoration operation**.

Do not:

- send `0x87`, `0x89`, `0x07` or `setLightType(2)`;
- run the vendor application to query or change the device;
- issue unknown Fn combinations, reset or pairing sequences; the family guide documents `Fn + Esc` as a 3-second default-restore shortcut, but it is not a safe diagnostic and could overwrite configuration;
- use hidden updater selections or activate the bootloader;
- flash V504 or any other image;
- infer safety from a `GET` name or a frontend response parser.

The V504 package may be relevant to the `3151:4010` ANSI V500+ product family, but no exact board/firmware match or complete backup/recovery path is established. Firmware update is not a diagnostic shortcut.

## 6. Safest next diagnostic step

The highest-value next evidence item is a **clear, non-invasive photograph of the exact keyboard PCB's main controller area and silkscreen/revision markings**, if the PCB is already accessible without risky disassembly. The photo should show the top markings on the main MCU and any nearby LED-driver/controller ICs, plus the board revision. If the board is not already accessible, do not open it solely for this investigation; the bottom label/model/revision and packaging information are a safer first substitute.

Why this is the best single next item:

- tests whether the external YC3121-L/BK3632 TH80-family lead applies to this actual unit;
- can reveal a separate LED driver or controller that the firmware artifact cannot identify;
- establishes the hardware revision needed to judge whether the V504 package and vendor frontend model are relevant;
- narrows the next static research target without sending a command or changing device state.

It will not by itself prove the root cause. The installed firmware version and exact keymap remain unknown. If a version is already visible in an existing screenshot or previously exported report, it can be supplied as supporting context; do not launch a device utility just to obtain it.

## 7. Remaining unknowns

- exact PCB revision and MCU marking in the user's unit;
- exact installed keyboard firmware and receiver firmware versions;
- whether the supplied V504 image matches the user's board and whether it is currently installed;
- actual Fn keymap/layer and Fn+Backspace dispatch;
- stored lighting enable/mode/brightness/speed/color state and its persistence owner;
- firmware command handlers for `0x07`, `0x87` and both `0x89` contexts;
- LED driver IC, bus, GPIO/PWM assignments, enable rail and initialization sequence;
- sleep/wake and USB/2.4G transition effects on LED power/state;
- physical condition of LED power, connectors, LED chain and driver.

## 8. Safety boundary

This phase authorized static local-file analysis only. No HID enumeration/open, Feature GET/SET, Output report, USB transfer, reset, pairing, calibration, updater execution, bootloader activation, firmware execution or flash operation was performed. Hardware testing remains **NOT APPROVED** until a separate narrowly scoped test proposal is backed by exact board/firmware identity, firmware-handler evidence, transport details, side-effect analysis, backup/recovery strategy and explicit user authorization.

## 9. References

- `HARDWARE.md`, `RESEARCH.md`, `FIRMWARE_ANALYSIS.md`, `UPDATER_REVERSE_ENGINEERING.md`
- `PHASE8_STATIC_EVIDENCE_CLOSURE.md`, `PHASE8_COMMAND_SEMANTICS_MATRIX.md`
- `PHASE12_1_RGB_DIAGNOSTICS.md`, `PHASE12_1_CLOSURE.md`, `PHASE12_2_LIGHTING_APP_AUDIT.md`
- EPOMAKER TH80 Pro product/revision warning: https://epomaker.kr/products/epomaker-th80-kit
- Official TH80 family quick guide (Fn shortcuts and OS-mode caveat): https://orders.epomaker.com/manuals/TH80-manual%20update-v2.pdf
- TH80-family hardware review: https://next.lab501.ro/featured/review-epomaker-th80-budgerigar-switch-monet-mda-v2/2
