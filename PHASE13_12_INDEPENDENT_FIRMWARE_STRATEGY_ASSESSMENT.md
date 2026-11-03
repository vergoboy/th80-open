# Phase 13.12 — Independent Firmware Strategy Assessment

## 1. Decision summary

**Recommended near-term investment: preserve vendor firmware and continue host-application development.**

Do not begin replacement-firmware implementation or installation work yet. The static firmware analysis establishes a coherent Yichip/YC3121-family lineage, candidate ARM Thumb instruction decoding, two firmware-side boot/application stub banks, and useful software-level timer/RGB relationships. It does not establish the exact physical MCU, complete ABI, memory map, boot ROM behavior, peripheral acknowledgement rules, or update acceptance policy.

Original V504 SDK acquisition remains the highest-value route to close the silicon and timer/system-control uncertainties. Independent implementation should be reconsidered only after exact hardware and bootloader evidence is obtained. The host-only path is the only strategy that can make useful progress without first solving the physical MCU and flash-installation boundary.

## 2. Strategy A — Recover and build from the original V504 SDK

### Required evidence

1. Exact original production SDK archive used for TH80 ANSI V504.
2. Matching MCU variant and SoC-specific register headers.
3. Librarier/sdk/yc_timer.c and yc_timer.h.
4. Librarier/sdk/yc_sysctrl.c and yc_sysctrl.h.
5. Interrupt numbers, vector definitions and common IRQ wrappers.
6. Startup assembly and reset/boot handoff source.
7. Original linker script/scatter/IAR configuration and memory map.
8. Clock/oscillator/PLL initialization and peripheral clock gating.
9. ROM API declarations and the ROM function table/version.
10. GPIO and LED output implementation with board pin definitions.
11. V504 project configuration, exact compiler/toolchain version and build flags.
12. Linker map and preferably ELF/AXF/object/debug outputs.
13. Original image packaging instructions and a documented, authorized installation method.
14. Package checksum, release identifier and hardware compatibility matrix.

### Benefits

- Best chance of reproducing the vendor firmware's existing startup, interrupt, timer, clock and ROM dependencies.
- Can resolve the known timer register/address discrepancy and peripheral interrupt acknowledgement semantics.
- Preserves original device-specific abstractions and board configuration.
- Allows source-to-binary comparison against the protected V504 image.

### Risks and gaps

- The exact package is not in the V504 ZIP or updater.
- The public YC3121 POS BSP at commit cbc19778d0ae5ef2533a28c0f62c6a399858010a is same-lineage evidence, not the original V504 build source; it lacks the timer and system-control sources.
- The vendor may only distribute binary firmware to customers, or the production SDK may be confidential or restricted by NDA.
- A recovered SDK may still target a different YC3121 variant or board revision.
- A successful compile would not prove that the bootloader accepts the output.

### Assessment

**GO for acquisition request preparation. NO-GO for building or flashing until the exact target and installation constraints are verified.**

This is the preferred firmware route because it minimizes speculative reconstruction. The next action is the technical request dossier, not another generic YC3121 search.

## 3. Strategy B — Build independently against recovered hardware interfaces

### Required evidence

1. Confirmed instruction set, core profile, calling convention and ABI.
2. Exact MCU part, silicon revision and package.
3. Complete flash/RAM/peripheral map for that exact part.
4. Verified reset address, boot ROM behavior and vector-bank selection.
5. Startup code, stack placement, initialization order and clock tree.
6. ROM API mapping, calling conventions and compatibility requirements.
7. Interrupt-controller semantics, including peripheral flag acknowledgement and pending-state behavior.
8. USB controller, descriptors, endpoint configuration and firmware-side USB stack.
9. Wireless controller, host interface, radio stack and coexistence/power behavior.
10. GPIO pin map, LED driver identity, electrical signaling and timing constraints.
11. Battery, sleep, wake and clock-gating control map.
12. A verified, vendor-authorized way to install a test image and recover the device if it fails.

### Benefits

- Avoids dependence on unavailable vendor source if the hardware interface can be documented completely.
- Could support a maintainable independent implementation after the exact silicon and boot path are known.
- Allows architecture choices to be evaluated without importing unrelated vendor application code.

### Risks and gaps

- The exact MCU variant is unknown. YC3121-L is a lead, not a confirmed marking.
- Thumb-like disassembly does not by itself prove the exact ARM core/ABI.
- The candidate 0x01000200 + file-offset mapping is not a complete physical memory map or proof of the physical flash base.
- The image has custom stub banks rather than a conventional Cortex-M vector table; the hardware/ROM vector fetch is unresolved.
- The timer acknowledgement and clock source are unresolved.
- The USB/wireless controller, LED driver and power-management IC are not identified on the exact board.
- The bootloader's package format, address calculations, erase/program/verify sequence and authentication policy are unknown.
- Independent code could compile correctly yet fail to boot, fail to communicate, or permanently disable the device if installation behavior is misunderstood.

### Assessment

**NO-GO for implementation at present.** Reconsider only after exact MCU identification, primary documentation, and a verified recovery/install path exist. Do not start by writing guessed MMIO, startup code or a speculative image header.

## 4. Strategy C — Preserve vendor firmware and develop the host application

### Requirements

1. Continue using the existing vendor firmware unchanged.
2. Use only commands whose semantics are supported by the existing protocol evidence.
3. Keep the transport implementation separate from profile/configuration logic.
4. Validate packet layouts offline before any separately authorized physical-device work.
5. Obtain explicit authorization for any future live USB/HID/BLE interaction.
6. Preserve compatibility with the currently installed vendor firmware and distinguish known commands from unresolved commands.

### Benefits

- Does not require the MCU, linker map or bootloader to be known.
- Existing host-side protocol and device metadata analysis can support useful application work.
- Offline core/profile logic can be implemented and tested without device access.
- Avoids the risk of replacing firmware while RGB, USB, wireless and power interfaces remain uncertain.
- Allows the project to deliver user-facing value while source acquisition is pending.

### Risks and gaps

- Some command semantics and device-family distinctions remain unresolved.
- The native vendor driver supports multiple device families; its broad VID/PID table must not be treated as an exact TH80 descriptor map.
- The generic Feature checksum path and the V504 firmware-upgrade path are not proven to be the same call path.
- Live transport compatibility still requires a separately authorized, controlled validation phase.
- Vendor firmware bugs and limitations remain outside host-side control.

### Assessment

**GO for offline host-application development. DESIGN WITH GAPS for live device integration.**

Continue the native application while keeping hardware transport gated behind verified command semantics and explicit authorization. Do not change the vendor firmware.

## 5. Comparative assessment

| Criterion | Strategy A: original SDK | Strategy B: independent | Strategy C: host-only |
|---|---|---|---|
| Requires exact MCU | Yes | Yes | No for offline work |
| Requires complete memory map | Yes, from SDK/docs | Yes | No |
| Requires bootloader acceptance rules | Before installation | Before installation | No, if firmware unchanged |
| Can resolve original timer semantics | Best route | Possible with complete manual | Not required for host-only |
| Can implement replacement firmware | Potentially, after recovery | Potentially, after full hardware recovery | No |
| Risk of bricking keyboard | High if installation attempted without proof | Very high with current gaps | Low if no firmware/device mutation |
| Current feasibility | Acquisition request ready; source unavailable | Blocked | Offline work ready; live work gated |
| Recommendation | Pursue once, specifically | Defer | Continue now |

## 6. Project milestone readiness

Statuses describe whether the milestone can be responsibly designed from current evidence, not whether a test has succeeded.

| Milestone | Status | Reason |
|---|---|---|
| Build a replacement firmware | BLOCKED | Exact MCU/ABI, toolchain, complete memory map and startup/linker model are not confirmed |
| Boot replacement firmware | BLOCKED | ROM reset/vector selection and bootloader/application entry contract are unknown |
| Preserve USB functionality | BLOCKED | Host-side VID/PID and protocol evidence exists, but MCU USB controller, firmware USB stack and exact board mapping are unknown |
| Preserve wireless functionality | BLOCKED | BK3632 is only a TH80-family review lead; exact wireless silicon and firmware/radio interface are unknown |
| Implement RGB | DESIGN_WITH_GAPS | Software timer/IRQ/output path is mapped, but exact MCU, interrupt acknowledgement, pin map and electrical LED protocol are missing |
| Improve battery life | BLOCKED | Sleep, clock gating, power rails and PMIC behavior are not established for the exact board |
| Safely install updates | BLOCKED | Bootloader acceptance, image format, address/write sequence, integrity and authentication rules are unknown |

No milestone is classified READY_FOR_DESIGN for replacement firmware. The host-only offline application remains a separate feasible workstream.

## 7. Evidence gates before reconsidering replacement firmware

All of the following should be obtained before implementation planning is reopened:

1. A board marking or vendor BOM confirming the exact MCU and board revision.
2. Matching primary SoC documentation and the exact V504 SDK or equivalent complete register/startup reference.
3. Exact toolchain, ABI, startup, linker and memory configuration.
4. Boot ROM and bootloader behavior documentation, including image acceptance and recovery behavior.
5. A vendor-authorized installation method and a documented recovery path.
6. Verified USB, wireless, RGB and power hardware interfaces.
7. A source-to-binary comparison or other reliable proof that the recovered source package matches the V504 lineage.

An image that parses or disassembles coherently is not evidence that it is bootable or installable.

## 8. Cross-references

- [Hardware identity and bootloader feasibility](PHASE13_12_HARDWARE_IDENTITY_AND_BOOTLOADER_FEASIBILITY.md)
- [Vendor SDK request dossier](PHASE13_12_VENDOR_SDK_REQUEST_DOSSIER.md)
- [Findings and next steps](PHASE13_12_FINDINGS_AND_NEXT_STEPS.md)
- [Phase 13.7 silicon and boot architecture](PHASE13_7_SILICON_AND_BOOT_ARCHITECTURE.md)
- [Phase 13.9 timer acknowledgement](PHASE13_9_TIMER_INTERRUPT_ACKNOWLEDGEMENT.md)
- [Phase 13.10 V504 source recovery](PHASE13_10_V504_SOURCE_PACKAGE_RECOVERY.md)
- [Phase 13.11 findings](PHASE13_11_FINDINGS_AND_NEXT_STEPS.md)
