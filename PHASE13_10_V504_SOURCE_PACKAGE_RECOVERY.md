# Phase 13.10 — V504 Source Package Recovery

## 1. Executive result

**Overall source-provenance classification: `SAME_SDK_LINEAGE`.** The V504 image contains Yichip SDK source-path strings and has multiple independent register/ROM fingerprints matching the Yichip YC3121 source model. The exact V504 SDK archive, release, source revision and original build project were not recovered. The available RT-Thread YC3121 POS BSP is a Yichip-authored, same-family reference source snapshot; it is not proven to be the V504 build source.

The four priority files remain unavailable:

- `Librarier/sdk/yc_timer.c`
- `Librarier/sdk/yc_timer.h`
- `Librarier/sdk/yc_sysctrl.c`
- `Librarier/sdk/yc_sysctrl.h`

No byte-identical timer source, exact build project, V504 linker map, original ELF/AXF, compiler manifest or source revision identifier was found.

## 2. Search scope and methodology

All inspection was static. No vendor program, updater, firmware, downloaded SDK script or hardware operation was executed.

### 2.1 Local roots inspected

Exact-name and artifact searches were performed under:

- `/home/vergoboy/Projects/projects/th80/`
- `/home/vergoboy/Downloads/`
- `/home/vergoboy/Documents/`
- `/home/vergoboy/.cache/`
- `/home/vergoboy/.local/share/`
- `/tmp/th80-phase13-8/`
- `/tmp/th80-phase13-9/`

The exact-name search covered `yc_timer.c/.h`, `yc_sysctrl.c/.h`, `yc3121.h`, `misc.h`, Yichip/YC3121/V504 names, Keil `.uvproj/.uvprojx`, IAR `.ewp/.eww`, linker `.map/.sct/.icf/.lds`, `.axf/.elf/.out`, and startup assembly names. Cache search pruned `.git`, `node_modules`, `target` and `.rustup`. Two unrelated cache package directories returned permission-denied messages; they are not claimed as inspected contents.

Within the TH80 project root, archive inventory found only the V504 ZIP, Mac driver ZIP and an unrelated bundled SQLite source tarball. The Mac driver ZIP contains one DMG. The V504 ZIP was listed without extraction and contains eight entries: directory, `dev_fm/`, three TOML configuration files, updater EXE, upgrade guide PDF and firmware `X`.

### 2.2 Git repositories and object stores

The TH80 project contains Git roots at `th80-open/.git` and `entropy-audit/.git`. The target repository has zero commits, zero packed or loose objects and no remotes; it is not a hidden source history. The entropy-audit repository has one commit and 425 packed objects but is a separate project; no evidence ties it to V504 SDK source. The RT-Thread source checkout is shallow and its origin is `https://github.com/RT-Thread/rt-thread.git`.

No alternate TH80 worktree or local Yichip SDK object store was identified in the searched roots.

## 3. V504 firmware distribution archive

Artifact: `/home/vergoboy/Projects/projects/th80/TH80_ANSI_Firmware_V504.zip`

Archive inventory:

| Entry | Size | Archive timestamp |
|---|---:|---|
| `selections_config.toml` | 539 bytes | 2021-11-08 |
| `support_config.toml` | 1,234 bytes | 2022-09-08 |
| `upgrade_config.toml` | 78 bytes | 2021-09-07 |
| `TH80 Upgrade Firmware for v5xx.exe` | 10,274,816 bytes | 2021-10-23 |
| `Upgrade Guide.pdf` | 154,048 bytes | 2022-08-17 |
| `X` | 184,608 bytes | 2022-09-21 |

The firmware image's extracted original path is:

`/home/vergoboy/Projects/projects/th80/TH80_ANSI_Firmware_V504/TH80 ANSI Firmware V504(ID 742)/X`

Its SHA-256 is `4b22336ec9ffaaac3372791a1154a2c2a833e3305a4cadbbdcab9184252c0520`. The image is a raw `data` file, not an ELF/AXF container. `readelf` therefore cannot provide ELF headers, sections, symbols or build attributes from this artifact. The initial bytes are consistent with the existing Thumb firmware analysis; they do not encode a compiler release identifier.

The archive does not contain any of the four missing SDK source files, a source archive, project file, linker script, map file, ELF/AXF or compiler manifest. The `dev_fm/` directory is present in the archive listing; no claim is made that it contains SDK source.

The configuration files describe update selection, supported USB/HID identifiers and update delays. They are updater packaging metadata, not compiler or firmware build settings.

## 4. Updater and host-side vendor package

The updater extracted during Phase 13.9 is:

`/tmp/th80-phase13-9/vendor-metadata/TH80-Upgrade-Firmware-v5xx.exe`

It is byte-identical to the updater entry in the V504 ZIP: SHA-256 `e9f3d8d2f3160be5e3817fb4eca8efe954cc854426351e3d0d21e73f39fce0fc`. Static PE metadata identifies PE32+, AMD64, five sections and COFF timestamp `2021-10-23 03:00:25 UTC`. Printable strings include Rust build paths, `rust-hid`, `holy_rob.rs`, and API/model identifiers such as `DevVersionList`, `DevVersionInfo`, `devId`, `version`, `lowestAppVersion`, `/getVersion`, `/downloadDevUpgradeFile` and `/v1/admin_upload_firmware`. It was not executed.

The protected Mac frontend at `.../Resources/app/dist/index.js` contains a download path `/downloadDevUpgradeFile`, a request payload keyed by `deviceId`, and configured service base `https://api.rongyuan.tech:3814/v1`. It inflates the returned data and passes it to the device-upgrade path. This is `FRONTEND_ONLY` evidence of the vendor firmware distribution workflow; it does not establish that the endpoint is active today or offers source archives.

The native Mac `iot_driver` is a Mach-O x86_64 binary. Its strings contain macOS framework references and Rust build/source paths but no useful Yichip SDK, timer-driver or compiler provenance. The frontend/native artifacts are host software, not V504 firmware build artifacts. The Mac driver ZIP contains only `EPOMAKER_Driver(MAC2023021).dmg`.

Protected frontend and native hashes are recorded in `PHASE13_10_FINDINGS_AND_NEXT_STEPS.md`.

## 5. Firmware source and build metadata

Static strings in firmware `X` include relative source paths:

- `..\\..\\..\\..\\..\\Librarier\\sdk\\yc_gpio.c`
- `..\\..\\..\\..\\..\\Librarier\\sdk\\yc_uart.c`
- `..\\..\\..\\..\\..\\Librarier\\sdk\\yc_timer.c`
- `..\\..\\..\\..\\..\\Librarier\\sdk\\yc_systick.c`
- `..\\..\\..\\..\\..\\Librarier\\sdk\\yc_sysctrl.c`
- `..\\..\\..\\..\\..\\Librarier\\sdk\\yc_wdt.c`
- `..\\..\\..\\..\\..\\Librarier\\sdk\\yc_ssc.c`

These strings confirm the original source naming convention and Yichip SDK lineage (`CONFIRMED_STATIC`). They do not disclose file contents, hashes, revision, compiler, optimization or build project. The spelling `Librarier` is retained exactly as present in the image.

No embedded ELF/AXF container, symbol table, DWARF source mapping, map file, build timestamp record or compiler identification string was recovered from `X`. The raw image has no direct basis for asserting an exact ARMCC, GCC or IAR release. The Thumb code and earlier vector/dispatch analysis support the ARM Cortex-M0-style target model; this is not a compiler-version fingerprint.

## 6. Closest recoverable Yichip source and provenance

### 6.1 Official Yichip YC3121 repository

URL: `https://github.com/Yichip-Microelectronics/YC3121`

The inspected public repository view contains one commit and README only, with no public releases or packages listed. It does not expose the missing timer/system-control source. The Yichip organization page identifies YC3121 as its public repository for that family. This is evidence about the inspected public state only; it says nothing conclusive about private customer SDK deliveries.

### 6.2 Yichip-authored YC3121 POS BSP in RT-Thread

Repository: `https://github.com/RT-Thread/rt-thread`

Path: `bsp/yichip/yc3121-pos`

Local snapshot: `/tmp/th80-phase13-8/rt-thread/bsp/yichip/yc3121-pos`

Inspected commit: `cbc19778d0ae5ef2533a28c0f62c6a399858010a` (2026-10-30T18:20:46+08:00, `[clang-format] Pin version 22.1.8 and format kernel sources (#11936)`). This is the RT-Thread checkout's HEAD, not a Yichip SDK release or proven source revision for V504. The checkout is shallow, so it does not establish complete file-level history.

The BSP contains core register definitions, system support, ROM API definitions, NVIC helpers, GPIO/UART/SysTick/DMA/WDT drivers, startup variants and build project/linker files. It does not contain `yc_timer.c`, `yc_timer.h`, `yc_sysctrl.c` or `yc_sysctrl.h`.

Representative file hashes:

| BSP artifact | SHA-256 |
|---|---|
| `project.uvprojx` | `69d6c5eeb5e1c46d076d8b00784b0b0703af5443c465f3842fef77a332634264` |
| `project.ewp` | `0baa5981b2617316ed615a6350d61de99e96432ccd15e37ae499406e096bc452` |
| `rtconfig.py` | `562b43018ea95d385954dddf71377103aa95a640c5e7768eb2526e8c2405e24e` |
| `linker_scripts/link.lds` | `0d3c7fad00163cd70ea2c192c939f42df9618bf8f4c5fcaddd0acad3389e9fba` |
| `libraries/startup/startup.s` | `dc877988bee22583eddc1c6b4b79dd88ea5cb8caaeddca0f67bb2c177ae5e704` |
| `libraries/core/yc3121.h` | `c4f1a83a3db05458c70e77bbe8847a460a3ac48d2c31659d4174239e6a9f1d6c` |
| `libraries/core/rom_api.h` | `c8fc4ceed7acd63b4eee51380f9733616ef57c5b8bdfe9cfa9150edb5abe481b` |

The Keil project declares ARM-ADS / ARMCC `V5.06 update 5 (build 528)`, generic `ARMCM0`, Cortex-M0 CPU, and memory descriptors. The IAR project is configured for a Synwit SWM320xE device. `rtconfig.py` offers GCC, ARMCC and IAR configurations; its GCC path names `arm-none-eabi`, Cortex-M0/thumb flags, section splitting and `-Map=rt-thread.map`; its ARMCC path selects `armcc`, `armasm`, `armlink`, `--cpu Cortex-M0`, a scatter file and map output; its IAR path selects `iccarm`/`ilinkarm` and `link.icf`. The GCC linker script specifies flash origin `0x01000000`, length `0x80000`, RAM origin `0x00020000`, length `0x10000`, and `Reset_Handler` entry. The startup assembly defines a vector table with external IRQ0–IRQ31 symbols and branches from reset to `Reset_Handler`.

These artifacts establish the BSP's supported build configurations and memory/startup model, not the V504 compiler, linker or memory layout. The mixed generic ARMCM0/Synwit labels and multiple toolchain alternatives are further reasons not to transplant its project metadata into the V504 provenance claim.

### 6.3 Excluded YC16xx package

The public `Yichip-Microelectronics/16xx` repository describes a separate 32-bit RISC-V YC16xx family and its own SDK structure. It is not a YC3121 ARM/Cortex-M0 source package and is not a valid substitute for the TH80 firmware.

## 7. Provenance decision

Classification: **`SAME_SDK_LINEAGE`**.

Rationale: V504 embeds Yichip SDK source-path strings, and prior phases established matching YC3121 timer register formulas, IRQ numbering/NVIC behavior and a ROM delay entry fingerprint against Yichip-authored public source. The closest source package is nevertheless a separate RT-Thread BSP snapshot with missing timer and system-control implementations. No exact release, package identity, project relationship or source-to-binary timer match is available. `EXACT_BUILD_SOURCE_RECOVERED` and `STRONG_SOURCE_MATCH` would overstate the evidence.

## 8. Remaining source/dependency list

Still required to close exact provenance and timer semantics:

- Original `Librarier/sdk/yc_timer.c/.h` and `yc_sysctrl.c/.h`.
- Matching SoC-specific timer, clock and interrupt headers.
- Original startup assembly/vector table and linker/scatter/IAR configuration.
- Original Keil/IAR/GCC project and compiler version/configuration.
- Linker map, ELF/AXF, object files or debug symbols.
- ROM API header and any ROM function table used by V504.
- SDK release manifest, archive checksum or vendor delivery record.
- Matching BSP examples and board/SoC configuration.

The most valuable next step is obtaining the vendor's original V504 SDK/build archive or an independently preserved YC3121 package containing the missing timer/system-control sources and a verifiable relationship to this firmware. Timer/clock interpretation is in [the timer source comparison](PHASE13_10_TIMER_SDK_SOURCE_COMPARISON.md) and [the clock/ROM/interrupt model](PHASE13_10_CLOCK_ROM_AND_INTERRUPT_MODEL.md); final status is in [findings and next steps](PHASE13_10_FINDINGS_AND_NEXT_STEPS.md).
