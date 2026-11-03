# Phase 13.12 — Findings and Next Steps

## 1. Final decision

**Replacement firmware: NO-GO. Host-only application development: GO for offline work. Original SDK acquisition: prepare one targeted vendor request.**

The V504 image contains coherent boot-stage and application-side code structures, but the physical MCU variant and bootloader acceptance rules are unknown. A replacement image cannot responsibly be built, booted or installed from the present evidence.

The best-supported near-term path is to leave the vendor firmware unchanged and continue the host application using already established protocol semantics and offline validation. In parallel, the prepared original V504 SDK request can be sent manually if and when the user chooses. No vendor contact has occurred in this phase.

## 2. Strongest findings

1. **Product and release identity:** The raw image contains the product string EPOMAKER TH80 Pro. The V504 archive path is TH80 ANSI Firmware V504 (ID 742). ID 742 is a release-directory label, not a USB PID.
2. **Runtime ID remains unresolved:** The frontend submits currentDev.deviceType.id as dev_id. No static evidence proves that this runtime ID equals 742.
3. **Updater USB identities:** The V504 updater configuration includes normal keyboard VID:PID 3151:4010 and boot-mode VID:PID 3151:4001. These are updater support entries, not proof of current physical enumeration.
4. **Receiver and broad driver IDs:** The native Mac driver contains a wider multi-device VID/PID table, including receiver/dongle-class entries such as 3151:4011. These are not all TH80-specific identities.
5. **Composite firmware-side structure:** X contains a boot-stage stub bank at offset 0x00000 and an application stub bank at 0x10000, with reset/startup pointers and a mode-dependent software handoff. Reset-pointer arithmetic supports candidate runtime address = 0x01000200 + file offset; this is not a confirmed physical flash base.
6. **Image is not a conventional executable container:** X is 184,608 bytes of raw data, not ELF/AXF/PE/HEX/UF2. It has zero-filled gaps and a nonzero data tail, but no complete linker/flash section map.
7. **Bootloader policy not recovered:** No complete update-specific call chain, target address calculation, erase/program/verify sequence, image size rule, package header rule, signature verification or bootloader authentication behavior is known.
8. **Generic HID checksum is not proven to be image acceptance:** The Windows updater's generic Feature sender has a recoverable prefixed/padded/checksummed buffer, but its call graph is not proven to belong to the YZW V504 update flow.
9. **Integrity/authentication are unresolved:** A generic checksum diagnostic and firmware-side small checksum helper exist, but neither is tied to whole-image verification. No obvious firmware signature/decryption mechanism was recovered; this is not evidence that the image is unsigned or unauthenticated.
10. **Silicon identity remains incomplete:** Yichip YC3121-family SDK lineage is strongly supported. YC3121-L and BK3632 remain external TH80-family leads, not confirmed markings on this board.

## 3. Is independent firmware development technically plausible?

Yes, in principle: the firmware contains coherent executable structures, a candidate memory mapping, and substantial Yichip SDK lineage evidence. The updater also recognizes a boot-mode HID identity. Those facts establish a plausible embedded firmware architecture.

No, not as a current implementation or installation plan: the exact MCU/ABI, physical flash map, reset/vector-fetch mechanism, ROM dependency contract, bootloader image-acceptance rules and recovery procedure are missing. The presence of boot-side code in X does not prove that the full bootloader is present or that an arbitrary replacement image will be accepted.

The correct decision is therefore **technical plausibility without installation feasibility**.

## 4. Exact blockers

### MCU and board

- Exact MCU part/variant/stepping/package unknown.
- YC3121-L is a TH80-family lead, not a confirmed physical marking.
- Exact PCB revision and BOM unknown.
- Wireless controller marking unknown; BK3632 is an external review lead.
- LED driver and power-management IC identities unknown.

### Build and runtime

- Original V504 SDK release/build project/compiler configuration unavailable.
- Matching timer and system-control source unavailable.
- Exact instruction-set/core profile and ABI not conclusively established.
- Complete memory map and linker placement unavailable.
- ROM API version/compatibility and startup requirements incomplete.
- Peripheral interrupt acknowledgement and clock-source semantics remain unresolved.

### Installation

- Immutable boot ROM and field bootloader behavior unknown.
- Exact firmware image/container contract unknown.
- No update-specific erase/program/verify sequence recovered.
- No authoritative image size, target address or bank-selection rules.
- Signature, authentication, encryption and anti-rollback policy unknown.
- No vendor-documented recovery path for a rejected or failed replacement image.

## 5. Development strategy recommendation

| Strategy | Decision | Next action |
|---|---|---|
| Original V504 SDK/build recovery | Pursue once, specifically | Use the prepared request dossier; ask EPOMAKER to route to Rongyuan/Yichip if needed |
| Independent firmware implementation | Blocked | Reopen only after exact MCU, complete hardware docs and authorized installation/recovery rules are obtained |
| Vendor firmware + host application | Recommended | Continue offline application, protocol documentation and compatibility work; gate live hardware operations behind separate authorization |

Do not spend more time on broad public YC3121 searches or speculative timer/register work. The highest-value missing evidence is the original production package or an exact board BOM plus primary MCU/bootloader documentation.

## 6. Minimum additional physical evidence

No disassembly or hardware interaction is required for this phase. If information is already available from packaging, existing photographs or prior authorized documentation, the most useful checklist is:

1. Product carton/label showing exact model, ANSI/ISO layout and any hardware revision/SKU.
2. Main MCU top marking and PCB silkscreen revision.
3. Wireless-controller marking and receiver/dongle marking.
4. LED-driver or GPIO-related IC markings, if a separate device exists.
5. Flash-memory marking, if an external flash chip is present.
6. Battery connector/cell label and power-management IC marking.
7. Existing vendor BOM, schematic excerpt or authorized board-revision compatibility note.

If board markings cannot be seen without opening the keyboard, that is an optional future evidence-gathering step, not a requirement or instruction. Do not probe, remove components, enter bootloader mode, enumerate the device or perform electrical measurements under this phase.

## 7. Milestone readiness

| Milestone | Status | Evidence gap |
|---|---|---|
| Build a replacement firmware | BLOCKED | Exact MCU/ABI/toolchain/memory map |
| Boot replacement firmware | BLOCKED | ROM reset/vector selection and bootloader/application entry contract |
| Preserve USB functionality | BLOCKED | Exact USB controller, firmware stack and board mapping |
| Preserve wireless functionality | BLOCKED | Exact wireless silicon and MCU/radio interface |
| Implement RGB | DESIGN_WITH_GAPS | Software path known; exact GPIO/LED electrical interface and timer acknowledgement missing |
| Improve battery life | BLOCKED | Sleep, clock, PMIC and power-rail semantics unknown |
| Safely install updates | BLOCKED | Image acceptance, address/write, integrity/authentication and recovery rules unknown |

No replacement-firmware milestone is READY_FOR_DESIGN. Offline host-only application development is a separate workstream and can continue without changing firmware.

## 8. Vendor request and contact status

The complete source request and a ready-to-review email draft are in [the SDK request dossier](PHASE13_12_VENDOR_SDK_REQUEST_DOSSIER.md).

Verified public EPOMAKER contact routes are listed there: support@epomaker.com and business@epomaker.com, with the official contact page. No verified direct Rongyuan technical contact or first-party Yichip SDK-request contact was established in this phase; the dossier asks EPOMAKER to route or identify the responsible engineering owner.

No email was sent, no support ticket was submitted, no vendor API was called and no file was uploaded.

## 9. SDK search stopping conditions

Do not repeat generic YC3121 SDK searches. Resume only when a concrete lead appears:

- An EPOMAKER/Rongyuan/Yichip response identifies the owner and availability of the exact V504 production package.
- An original SDK archive/build project/map/ELF/AXF is obtained with a verifiable release identity.
- An exact board BOM or MCU marking resolves the physical silicon variant.
- A vendor supplies non-confidential register, startup, bootloader, LED or memory-map documentation.

If the vendor confirms the production SDK cannot be released and offers no usable non-confidential technical reference, close source-recovery work and continue the host-only path. A generic family BSP is not a substitute for V504 provenance.

## 10. Preservation and scope

Protected artifact hashes were checked before report creation and matched the Phase 13.11 manifest:

| Artifact | SHA-256 |
|---|---|
| Firmware X | 4b22336ec9ffaaac3372791a1154a2c2a833e3305a4cadbbdcab9184252c0520 |
| Native iot_driver | 22c5dc67ba77e2373a5ffdbb0312ed8c8e73eb16e78037764eb676a030b2724b |
| Frontend dist/index.js | 7d53737095fe5756fb0a18d5aecb7593555b9bb61a9d5d160ae0b988694c1e87 |
| Windows updater | e9f3d8d2f3160be5e3817fb4eca8efe954cc854426351e3d0d21e73f39fce0fc |
| V504 ZIP | 7a3c128cfa385d458ca1850b9e5d297285c79f7aaca981022334f7140db538e4 |

The saved Phase 13.11 default Git status snapshot matches the pre-write Phase 13.12 status exactly: 104 entries. The expanded --untracked-files=all view reports 147 paths because existing untracked directories are expanded. All 12 historical Phase 13.8–13.10 report hashes pass.

The four Phase 13.12 reports are intended to be the only new repository files. No firmware, updater, native driver, frontend, Rust source, Cargo dependency, GUI or historical report was modified. No files were staged, no tracked changes were made, no commits or remotes were added.

All analysis was static. No updater or firmware execution, SDK script execution, physical USB/HID/BLE access, device enumeration, settings change, reset, pairing, calibration, bootloader entry, package modification, flashing or vendor contact occurred.

## 11. Cross-references

- [Hardware identity and bootloader feasibility](PHASE13_12_HARDWARE_IDENTITY_AND_BOOTLOADER_FEASIBILITY.md)
- [Independent firmware strategy assessment](PHASE13_12_INDEPENDENT_FIRMWARE_STRATEGY_ASSESSMENT.md)
- [Vendor SDK request dossier](PHASE13_12_VENDOR_SDK_REQUEST_DOSSIER.md)
- [Phase 13.11 findings](PHASE13_11_FINDINGS_AND_NEXT_STEPS.md)
- [Phase 13.10 source package recovery](PHASE13_10_V504_SOURCE_PACKAGE_RECOVERY.md)
- [Phase 13.7 hardware boundary](PHASE13_7_FINDINGS_AND_HARDWARE_BOUNDARY.md)
