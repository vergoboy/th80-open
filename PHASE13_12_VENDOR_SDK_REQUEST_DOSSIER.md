# Phase 13.12 — Original TH80 ANSI V504 SDK Request Dossier

## 1. Purpose

This dossier prepares a precise request for the **original EPOMAKER TH80 ANSI / TH80 Pro V504 production development package**, associated with archive release directory ID 742.

This is not a request for a generic YC3121 datasheet, an unrelated public BSP, or a current TH80 Pro V2 firmware package. The required material is the exact SDK/build configuration and board target used for the original V504 production image.

No email, support ticket, upload or vendor contact has been made.

## 2. Product and firmware identification

| Field | Verified value |
|---|---|
| Product | EPOMAKER TH80 / TH80 Pro |
| Layout/release | TH80 ANSI Firmware V504 |
| Archive release directory | TH80 ANSI Firmware V504 (ID 742) |
| Archive product label | 742; mapping to frontend runtime deviceType.id is unverified |
| Firmware image member | X |
| Image size | 184,608 bytes |
| Firmware image SHA-256 | 4b22336ec9ffaaac3372791a1154a2c2a833e3305a4cadbbdcab9184252c0520 |
| Original ZIP | TH80_ANSI_Firmware_V504.zip |
| ZIP SHA-256 | 7a3c128cfa385d458ca1850b9e5d297285c79f7aaca981022334f7140db538e4 |
| Windows updater | TH80 Upgrade Firmware for v5xx.exe |
| Updater SHA-256 | e9f3d8d2f3160be5e3817fb4eca8efe954cc854426351e3d0d21e73f39fce0fc |
| Native Mac driver SHA-256 | 22c5dc67ba77e2373a5ffdbb0312ed8c8e73eb16e78037764eb676a030b2724b |
| Frontend bundle SHA-256 | 7d53737095fe5756fb0a18d5aecb7593555b9bb61a9d5d160ae0b988694c1e87 |
| Updater support VID:PID | Normal keyboard 3151:4010; boot-mode entry 3151:4001 |

The normal/boot VID:PID values are entries in the V504 updater's support configuration. They are not proof of the exact current USB descriptors of a particular physical unit. Product ID 742 is an archive release-directory label, not a USB PID.

## 3. SDK lineage and provenance

The V504 image embeds source-path remnants including:

- Librarier/sdk/yc_gpio.c
- Librarier/sdk/yc_uart.c
- Librarier/sdk/yc_timer.c
- Librarier/sdk/yc_systick.c
- Librarier/sdk/yc_sysctrl.c
- Librarier/sdk/yc_wdt.c
- Librarier/sdk/yc_ssc.c

Prior static analysis also found Yichip-style register, interrupt and ROM API fingerprints consistent with the YC3121 family. The current provenance classification is **SAME_SDK_LINEAGE**. It is not an exact source-to-binary match and does not establish the exact MCU variant.

The closest verifiable public reference is the Yichip-authored YC3121 POS BSP in RT-Thread at commit:

**cbc19778d0ae5ef2533a28c0f62c6a399858010a**

That public BSP provides related YC3121 core, register, startup, ROM API and selected peripheral material. It is not proven to be the V504 build source and lacks the four timer/system-control files listed below. The public Yichip YC3121 repository is also not the missing production package.

Public references:

- Yichip YC3121 repository: https://github.com/Yichip-Microelectronics/YC3121
- RT-Thread repository: https://github.com/RT-Thread/rt-thread
- RT-Thread YC3121 POS BSP path: bsp/yichip/yc3121-pos
- EPOMAKER official contact page: https://epomaker.com/pages/contact-us

## 4. Exact files and technical reasons requested

### Priority 0 — timer and system-control implementation

| Requested file | Why it is required |
|---|---|
| Librarier/sdk/yc_timer.c | Resolve actual timer initialization, channel semantics, enable/stop/restart behavior, interrupt flag handling and timer acknowledgement |
| Librarier/sdk/yc_timer.h | Establish timer API contracts, register definitions, channel layout, control bits, reload semantics and interrupt identifiers |
| Librarier/sdk/yc_sysctrl.c | Recover oscillator/PLL startup, HCLK selection, peripheral clock enable/gating, reset and sleep behavior |
| Librarier/sdk/yc_sysctrl.h | Recover system-control API, clock-source selectors, clock-tree constants and control-register definitions |

### Priority 1 — exact SoC, interrupts, reset and memory

| Requested material | Why it is required |
|---|---|
| Matching SoC-specific register headers | Resolve the exact timer, GPIO, USB, clock, power and interrupt register map for the actual V504 target |
| MCU variant/stepping selection headers and configuration | Distinguish YC3121, YC3121-L, YC3121-H or another exact device/revision |
| Interrupt number definitions | Confirm IRQ14/IRQ15 mappings and peripheral source routing |
| Vector definitions and shared IRQ wrappers | Recover hardware-to-software entry, acknowledgement and handler calling convention |
| Startup assembly | Establish reset entry, stack initialization, data/BSS initialization, vector selection and boot/application handoff |
| Original linker script/scatter/IAR configuration | Establish code/data placement, flash origin/size, RAM origin/size, ROM regions and reserved boot areas |
| Original memory map and linker map output | Tie image offsets and absolute pointers to physical flash/RAM regions |
| ROM API headers and function table | Resolve ROM call addresses, ABI, required arguments and ROM-version compatibility |

### Priority 2 — clocks, GPIO, LED and board-specific support

| Requested material | Why it is required |
|---|---|
| Clock initialization source and board clock configuration | Determine the actual oscillator, PLL, HCLK and peripheral clock rates |
| GPIO driver source and matching headers | Recover pin configuration, alternate functions, pull state and output behavior |
| LED/RGB output source | Identify output pin(s), timing, serialization/PWM method and any external driver dependency |
| Board pin map and schematic/BOM excerpt | Correlate SDK GPIO names with the actual PCB and LED/power components |
| Wireless controller interface documentation/configuration | Establish how the keyboard MCU communicates with the radio controller or receiver |
| USB peripheral/stack source and descriptors | Determine whether USB is integrated, externally bridged or handled by another controller |

### Priority 3 — reproducible build and release metadata

| Requested material | Why it is required |
|---|---|
| Original V504 project files | Identify build targets, source membership, preprocessor definitions and post-build steps |
| Compiler/toolchain name and exact version | Reproduce code generation, ABI and linker behavior |
| Compiler/linker flags and configuration | Reproduce optimization, CPU target, section placement and runtime libraries |
| SDK release identifier and package manifest | Identify the exact production SDK package |
| SDK archive SHA-256 or vendor release checksum | Verify package identity and detect incomplete or altered transfers |
| Build log and linker map | Reconcile the production image's code/data placement and toolchain |
| ELF/AXF/object/debug artifacts, if retained | Enable symbol and source-to-binary comparison |
| Firmware packaging instructions and format specification | Understand the vendor image/container boundary without reverse-engineering acceptance rules by trial |
| Hardware compatibility matrix | Establish which PCB revision, MCU variant, layout and wireless configuration V504 targets |

## 5. Specific technical questions for the vendor

1. Which exact MCU manufacturer, part number, variant, silicon revision and package were used on the TH80 ANSI V504 production board?
2. Which PCB revision(s) does the V504 image support?
3. Is the V504 image a composite boot-stage/application image, an application-only image, or another vendor-specific flash representation?
4. Which code region is factory bootloader/ROM, which region is the replaceable application, and which regions must remain unchanged?
5. What are the authoritative flash and RAM base addresses and sizes for this board/MCU revision?
6. Does the production bootloader accept only vendor-signed images, or is there another documented integrity/authentication requirement? Please provide the official policy and package format; no bypass is requested.
7. Is an image checksum or signature manifest generated during the V504 build? If so, which algorithm and which metadata/header fields are covered?
8. Are image encryption, per-device keys, secure boot or ROM authentication used?
9. Which update transport and boot-mode VID:PID are correct for this exact ANSI revision? The local updater configuration contains normal 3151:4010 and boot-mode 3151:4001 entries; please confirm whether these are the intended V504 pair.
10. Is there an official, documented recovery path for a failed application image?
11. Which compiler/toolchain and exact version generated V504?
12. Are the original SDK and project maintained by EPOMAKER, Rongyuan, Yichip or another ODM/firmware contractor?
13. Can the original SDK/build package be shared under an open-source-compatible license that permits modification and redistribution?
14. If the SDK itself cannot be released, is a source license or written permission for independent development and publication of clean-room implementation code available?
15. Is the production SDK confidential or available only under NDA? If so, what is the appropriate NDA process?
16. If source cannot be shared, can the vendor provide a non-confidential register/peripheral programming reference, startup/memory map, interrupt documentation and LED interface specification?
17. Can the vendor provide a written statement of the exact MCU/PCB compatibility and the safe installation/recovery constraints without disclosing proprietary source?

## 6. Licensing and source ownership questions

The request should explicitly ask:

- Who owns or maintains the original V504 firmware source and SDK: EPOMAKER, Rongyuan, Yichip or another ODM?
- Is the production SDK proprietary, confidential or restricted by a customer/vendor agreement?
- Can the SDK be released under an open-source-compatible license?
- If the SDK itself cannot be released, can selected headers, register definitions, programming references and interface documentation be shared under a permissive documentation license?
- Can the vendor grant written permission to publish independently written implementation code based on non-confidential hardware documentation?
- Are there attribution, redistribution, patent, trademark or binary redistribution restrictions that should be observed?

No proprietary source should be published or redistributed unless the vendor grants an appropriate license or permission. Receiving an archive does not itself authorize publication.

## 7. Public contact routes — verified scope

### EPOMAKER

EPOMAKER's official Contact Us page lists:

- support@epomaker.com for product and technical support.
- business@epomaker.com for business, OEM/ODM and partnership inquiries.
- Official contact page: https://epomaker.com/pages/contact-us

The most direct first route is support, with a request to forward the dossier to firmware engineering or the responsible ODM. The business address is a verified alternate escalation route for OEM/ODM matters.

### Rongyuan

The local frontend/updater artifacts establish Rongyuan service hostnames and firmware-distribution API generations, but no verified public Rongyuan technical contact channel was identified in this phase. Do not guess an email address or contact form. Ask EPOMAKER to identify the responsible Rongyuan engineering contact or forward the request internally.

### Yichip

The public Yichip YC3121 repository is available at https://github.com/Yichip-Microelectronics/YC3121. A direct first-party technical contact route suitable for this production SDK request was not verified in this phase. Ask EPOMAKER to confirm the actual silicon supplier and provide an official Yichip field-application-engineering contact if Yichip owns the SDK.

No vendor was contacted, and no support form or email was submitted.

## 8. Evidence package and attachment hashes

The following verified local artifacts identify the exact release under discussion. The hashes can be included in the request body; attaching binaries is not necessary for an initial inquiry.

| Artifact | SHA-256 |
|---|---|
| V504 ZIP | 7a3c128cfa385d458ca1850b9e5d297285c79f7aaca981022334f7140db538e4 |
| Raw firmware X | 4b22336ec9ffaaac3372791a1154a2c2a833e3305a4cadbbdcab9184252c0520 |
| Windows updater EXE | e9f3d8d2f3160be5e3817fb4eca8efe954cc854426351e3d0d21e73f39fce0fc |
| Native Mac iot_driver | 22c5dc67ba77e2373a5ffdbb0312ed8c8e73eb16e78037764eb676a030b2724b |
| Mac frontend dist/index.js | 7d53737095fe5756fb0a18d5aecb7593555b9bb61a9d5d160ae0b988694c1e87 |

The V504 ZIP itself has six file members and an empty dev_fm directory. It contains no SDK source or build project. The updater is a Rust Windows HID/Rongyuan distribution client, not a Yichip compiler or SDK installer.

## 9. Ready-to-review email draft

**To:** support@epomaker.com  
**Subject:** Technical SDK/build-source request — TH80 ANSI V504 (archive ID 742)

Dear EPOMAKER Support / Firmware Engineering Team,

I am working on an independent, open-source host application and a documentation-led reverse-engineering effort for the original EPOMAKER TH80 / TH80 Pro ANSI firmware V504. I am trying to identify the exact production SDK and build configuration used for this release so that any future low-level work can preserve the original hardware behavior and remain compatible with the correct board revision.

The local release archive is labelled **TH80 ANSI Firmware V504 (ID 742)**. Its SHA-256 is:

7a3c128cfa385d458ca1850b9e5d297285c79f7aaca981022334f7140db538e4

The raw firmware member X is 184,608 bytes and has SHA-256:

4b22336ec9ffaaac3372791a1154a2c2a833e3305a4cadbbdcab9184252c0520

The firmware contains Yichip SDK source-path references, including Librarier/sdk/yc_timer.c and Librarier/sdk/yc_sysctrl.c. The closest public source reference I have found is the Yichip-authored YC3121 POS BSP in RT-Thread, commit cbc19778d0ae5ef2533a28c0f62c6a399858010a. That BSP is related-family material, not a confirmed source revision for V504, and it does not contain the timer and system-control implementations needed to resolve the firmware's behavior.

Could you please provide, or direct me to the owner of, the **original TH80 ANSI V504 production SDK/build package**? The highest-priority files are:

- Librarier/sdk/yc_timer.c and yc_timer.h
- Librarier/sdk/yc_sysctrl.c and yc_sysctrl.h
- Matching SoC-specific register, timer, clock and interrupt headers
- Interrupt/vector definitions and shared IRQ wrappers
- Startup assembly, linker script and authoritative flash/RAM memory map
- Clock initialization and ROM API definitions
- GPIO and LED/RGB output source and board pin map
- Original V504 project files, compiler/toolchain version, build flags and linker map
- SDK release identifier, archive checksum and MCU/PCB compatibility documentation

I would also appreciate confirmation of the exact MCU part/variant and PCB revision used for V504, whether the image contains both boot-stage and application regions, and the documented image format and installation/recovery requirements. The local updater configuration lists normal-mode VID:PID 3151:4010 and boot-mode VID:PID 3151:4001; please confirm whether those entries correspond to the intended TH80 ANSI V504 hardware.

If the source package cannot be shared, could you let me know:

1. Whether it is confidential or available under NDA.
2. Whether an open-source-compatible license or written permission for independent development is available.
3. Whether non-confidential register, peripheral, interrupt, startup, memory-map and LED-interface documentation can be provided instead.
4. Whether the SDK is maintained by EPOMAKER, Rongyuan, Yichip or another ODM, and whether you can forward this request to the responsible engineering team.

The purpose is to reproduce and document the existing firmware behavior and support an independently maintained open-source host application. I am not requesting proprietary source for redistribution without permission, and I will not publish any confidential material. I have not contacted the update API, attempted to enter bootloader mode, modified an update package or performed a firmware update as part of this investigation.

Thank you for your help. Even a confirmation of the exact MCU/board revision and the correct technical contact would be very useful.

Best regards,

TH80 Open-Source Host Application Project

## 10. Request stopping conditions

Do not repeat broad public searches for generic YC3121 SDKs. Resume source acquisition only if one of these concrete leads appears:

1. EPOMAKER identifies the exact source owner or supplies a technical contact.
2. Rongyuan or Yichip confirms an exact V504 production SDK archive/revision.
3. A verifiable original SDK archive, build project, map or ELF/AXF appears.
4. A vendor reply confirms that only confidential source exists and offers a documented non-confidential reference or NDA route.
5. A new independently preserved source archive has exact V504/TH80 ANSI compatibility evidence.

If the vendor confirms that the production SDK is unavailable for release and no non-confidential technical reference can be supplied, stop generic SDK recovery. Continue host-only work and retain replacement firmware as blocked.

## 11. Cross-references

- [Hardware identity and bootloader feasibility](PHASE13_12_HARDWARE_IDENTITY_AND_BOOTLOADER_FEASIBILITY.md)
- [Independent firmware strategy assessment](PHASE13_12_INDEPENDENT_FIRMWARE_STRATEGY_ASSESSMENT.md)
- [Findings and next steps](PHASE13_12_FINDINGS_AND_NEXT_STEPS.md)
- [Phase 13.11 archive analysis](PHASE13_11_V504_DISTRIBUTION_AND_ARCHIVE_ANALYSIS.md)
- [Phase 13.11 updater forensics](PHASE13_11_UPDATER_STATIC_FORENSICS.md)
- [Phase 13.11 Rongyuan frontend flow](PHASE13_11_RONGYUAN_API_AND_FRONTEND_FLOW.md)
- [Phase 13.10 source package recovery](PHASE13_10_V504_SOURCE_PACKAGE_RECOVERY.md)
