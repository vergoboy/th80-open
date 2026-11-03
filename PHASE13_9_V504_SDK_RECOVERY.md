# Phase 13.9 — V504 SDK Recovery and Provenance

## 1. Executive result

**Conclusion: the original V504 SDK package and exact SDK release remain unidentified.** The closest verifiable Yichip-authored source is the YC3121 POS BSP embedded in RT-Thread. It provides a matching YC3121 register model, IRQ numbering, Cortex-M0/NVIC support code, ROM API addresses and several SDK drivers. It does not contain the timer or system-control driver sources needed to fingerprint the V504 timer helper implementations.

The evidence supports the following distinctions:

| Question | Result | Classification |
|---|---|---|
| Yichip SDK lineage in V504 | Embedded source-path strings name yc_gpio.c, yc_timer.c, yc_uart.c, yc_systick.c and yc_sysctrl.c | CONFIRMED_STATIC |
| Public Yichip YC3121 source lineage | Yichip-authored headers and drivers are present in the RT-Thread YC3121 POS BSP | CONFIRMED_STATIC |
| Same YC3121 register model | Timer address macros, field positions and IRQ identifiers agree with firmware behavior | STRONG_STATIC_INFERENCE |
| Same ROM API map | V504 delay thunk target matches the public Yichip FUNC_DELAY_US_ADDR constant | STRONG_STATIC_INFERENCE |
| Exact source revision / SDK release | No release manifest, tag, build metadata or timer source implementation found | UNKNOWN |
| Exact source-to-binary match for timer helpers | Not possible without yc_timer.c / yc_timer.h | UNKNOWN |

Do not label the RT-Thread BSP as the exact V504 SDK. Its commit is an independently identifiable repository snapshot, not a vendor SDK release identifier.

## 2. Search methodology and preservation

The investigation was static. It searched:

- The original TH80 project tree and existing Phase 13 analysis material.
- The local Phase 13.8 research directory and downloaded YC3121 datasheets.
- Local Yichip-named source files, SDK headers, archives, Git repositories and source caches.
- The V504 firmware package listing, updater executable strings and firmware source-path strings.
- Public Yichip GitHub repositories and the public RT-Thread YC3121 POS BSP.
- Public searches for exact filenames and the literal directory spelling Librarier/sdk.

No downloaded script, project file, executable, firmware or vendor binary was executed. The updater executable was extracted to /tmp/th80-phase13-9/vendor-metadata for static file/strings inspection only. Downloaded or pre-existing SDK material remains outside the repository and outside the original firmware directory.

The protected firmware X, native iot_driver and frontend dist/index.js were checked before and after the investigation. Their expected hashes are listed in the final findings report.

## 3. Local V504 package and metadata findings

The firmware image contains source-path strings consistent with a Yichip SDK build, including:

- yc_gpio.c
- yc_timer.c
- yc_uart.c
- yc_systick.c
- yc_sysctrl.c

These strings establish source naming/lineage, not the contents or revision of the source files.

The V504 firmware distribution archive contains firmware/update material and configuration metadata, not the original SDK source tree. No yc_timer.c, yc_timer.h, yc_sysctrl.c, yc_sysctrl.h, compiler project, linker map, ELF, source archive, SDK manifest or exact SDK release label was recovered from the package.

The updater executable extracted for inspection is a PE32+ Rust application. Its strings include a rust-hid build path and application/API identifiers; no Yichip timer/system-control source path, SDK version or compiler project fingerprint was found. It was not executed.

No complete local Yichip SDK repository, alternate worktree or Git object store containing the missing timer/system-control implementation was found in the searched project/cache locations.

## 4. Public sources and provenance

### 4.1 Official Yichip YC3121 repository

URL: https://github.com/Yichip-Microelectronics/YC3121

The public repository contains one commit and README.md only in the inspected repository view. It has no public release or package history and no timer/system-control source files. The organization page identifies YC3121 as one of the two public repositories; the YC3121 repository page shows the single README file and one commit.

Classification: CONFIRMED_STATIC for the inspected public repository state; UNKNOWN for any private or unpublished vendor download.

### 4.2 RT-Thread YC3121 POS BSP

Repository: https://github.com/RT-Thread/rt-thread

Path: bsp/yichip/yc3121-pos

Local snapshot path: /tmp/th80-phase13-8/rt-thread

Inspected commit: cbc19778d0ae5ef2533a28c0f62c6a399858010a

Commit date: 2026-10-30T18:20:46+08:00

Commit subject: [clang-format] Pin version 22.1.8 and format kernel sources (#11936)

The local Git checkout is shallow. Its current commit is recorded, but the shallow checkout cannot establish the full history of the BSP's individual source files or the original date at which each Yichip source file entered RT-Thread. This commit must not be presented as the SDK's source revision or vendor release.

The BSP includes:

- libraries/core/yc3121.h
- libraries/core/system.h and system.c
- libraries/core/misc.h and misc.c
- libraries/core/rom_api.h
- libraries/sdk/yc_gpio.c/.h
- libraries/sdk/yc_uart.c/.h
- libraries/sdk/yc_systick.c/.h
- libraries/sdk/yc_dma.c/.h
- libraries/sdk/yc_wdt.c/.h

It does not include yc_timer.c, yc_timer.h, yc_sysctrl.c or yc_sysctrl.h. There is no timer driver implementation in the inspected BSP directory.

### 4.3 Yichip 16xx repository — excluded negative control

URL: https://github.com/Yichip-Microelectronics/16xx

The public README describes the YC16xx family as a 32-bit RISC-V BLE SoC with a separate YC16xx SDK package. It is not the YC3121 family and is not a valid source for V504 timer behavior. Its existence does not close the YC3121 source gap.

### 4.4 Manufacturer datasheets

- YC3121-D V1.2, revision history through 2021-07-22.
- YC3121-L V1.1, revision history through 2021-05-11.

The documents provide a nine-channel timer model, register addresses, counter reload description and HCLK relationship. Both timer interrupt-handling subsections contain a heading but no substantive acknowledgement description. They do not identify a timer interrupt status or clear register.

YC3121-D PDF: https://img.eecart.com/dev/file/part/spec/YC3121-D-YICHIP.pdf

YC3121-L PDF: https://ycble.cn/apks/yc_file/YC3121-L%E8%8A%AF%E7%89%87%E6%95%B0%E6%8D%AE%E6%89%8B%E5%86%8CV1.1.pdf

The exact keyboard PCB MCU remains unconfirmed. YC3121-D/L documentation is therefore a matching family candidate, not proof of the physical silicon fitted to the keyboard.

## 5. Candidate SDK version stamps and file hashes

The public BSP files carry component-level version banners. They do not form a single SDK release identifier.

| Source file | Embedded banner | SHA-256 |
|---|---|---|
| core/yc3121.h | V1.0, 2018-03-27 | c4f1a83a3db05458c70e77bbe8847a460a3ac48d2c31659d4174239e6a9f1d6c |
| core/system.h | V1.0, 2018-05-22 | f90749c1e8e753053df1e63d85e193f90e3263d46ceb9860a41d03c600a65573 |
| core/misc.h | V1.0, 2020-07-17 | 6676004f95ece93c48384b109f9188fa7d1dc916e8774d304720634bb4c0ffa9 |
| core/misc.c | V1.0, 2020-07-17 | 3426c11bce84a7172151f80ef69bdf7f8c44e0814896d67ca77e0d9303838826 |
| core/rom_api.h | V1.0, 2020-02-11 | c8fc4ceed7acd63b4eee51380f9733616ef57c5b8bdfe9cfa9150edb5abe481b |
| sdk/yc_gpio.c | V1.0, 2019-12-03 | 104f740e836492ea29d0fdfc1afabd325639ee678a152da66966392e1c875bc3 |
| sdk/yc_uart.c | V1.0, 2019-12-04 | 7139bbe8954e0b5d2ba57b4b2df518ac5493e23b921958705ff2a260c0ecf3f6 |
| sdk/yc_systick.c | V1.0, 2018-03-27 | 45a78da58635e873e19474357b527bf776f58ea2998f7df247d5b3fe333cfe1e |
| sdk/yc_dma.c | V1.0, 2018-03-27 | c3bfb588125129f21fa86a88734241a4b620a584e40a3ae28463830fe80437f1 |
| sdk/yc_wdt.c | No comparable banner in inspected file | 1d312a49abc7b6ad9bda71684648e250192062f37e6448e8585bfaf302c83a62 |

The misc.h and misc.c banner's File Name field says board_config.h. This is retained as a source-file metadata quirk, not interpreted as a different source identity.

## 6. Function- and register-level fingerprints

### 6.1 Timer register model

In core/yc3121.h:

- TIMER_BASEADDR = 0xf0c00
- TIM_PCNT(x) = TIMER_BASEADDR + x*8
- TIM_NCNT(x) = TIMER_BASEADDR + 4 + x*8
- TIM_CTRL = TIMER_BASEADDR + PWM_TOTAL*8, with PWM_TOTAL=9
- TIM_CTRL1 = TIM_CTRL + 4
- TIM_CNT(x) = the counter array beginning at 0xf0c50, with four-byte channel stride
- TIM_CTRL_ENABLE, TIM_CTRL_START_LEVEL, TIM_CTRL_MODE and TIM_CTRL_AUTO_RELOAD are bit positions 0, 1, 2 and 3.

The firmware helper arithmetic at file offsets 0x1a9b2, 0x1a9fa and 0x1aa50 matches the same four-bit channel packing and the same control fields. This is a strong register-model match. The source implementation of those helper functions is absent, so it is not an exact source-to-binary match.

### 6.2 IRQ numbering and NVIC helpers

In core/misc.h, TIM1_IRQn is 14 and TIM2_IRQn is 15. The firmware's Timer1 and Timer2 init functions at 0x10240 and 0x10214 enable NVIC indices 14 and 15 respectively.

In core/yc3121.h:

- enable_intr(intid) ORs bit intid into NVIC_ISER at 0xe000e100.
- disable_intr(intid) writes bit intid to NVIC_ICER at 0xe000e180.

The firmware functions at 0x1d304 and 0x1d312 perform the corresponding ISER and ICER operations. The firmware priority helper at 0x1d2b8 is consistent with the source NVIC priority register update.

core/misc.c explicitly describes a Cortex-M0 hardware mapping, defines __NVIC_PRIO_BITS as 2, and declares NVIC ICPR at offset 0x180 from NVIC_BASE. This is a meaningful public BSP architecture fingerprint, but does not prove that the keyboard PCB contains the same exact MCU variant.

### 6.3 ROM delay API — additional source-to-binary fingerprint

core/rom_api.h defines:

- FUNC_DELAY_US_ADDR = 0x4238 + 1
- FUNC_DELAY_MS_ADDR = 0x425c + 1

The firmware literal at file offset 0x1ac0c is 0x00004239. The thunk at file offset 0x1a9a8 loads that literal and branches through it. This is an exact address match to the public SDK's delay-us ROM entry. The adjacent thunk at 0x1a9ac adds 0x24, producing the delay-ms entry 0x425d.

V504 calls the 0x1a9a8 thunk from the RGB output routine with branch-selected arguments 7 or 15 and with argument 4. This is a strong independent ROM API fingerprint and is analyzed in PHASE13_9_RGB_CALLBACK_TIMING_MODEL.md.

### 6.4 Other drivers

The BSP's GPIO, UART, SysTick, DMA and watchdog sources confirm a Yichip-authored SDK component family and provide dated component banners. No reliable function-level binary match for these drivers was established in this phase. In particular, the RGB output path performs direct register writes and software bit loops; it must not be assumed to be a call to the public GPIO driver.

The watchdog and DMA drivers have explicit pending-clear functions. This is a useful SDK coding-pattern comparison, but it cannot substitute for the missing timer source.

## 7. Why the exact release cannot be identified

The available fingerprints distinguish the likely SoC register model and ROM API, but not a unique source revision:

1. The original V504 source archive and project metadata were not recovered.
2. The official YC3121 repository contains only README.md in its inspected public state.
3. The RT-Thread BSP snapshot lacks the timer and system-control source files.
4. Its Git checkout is shallow, and its current commit is not a Yichip SDK tag.
5. Component banners range across 2018–2020 and are not a common package version.
6. The firmware's embedded source-path strings do not encode source hashes or release tags.
7. No byte-identical timer driver source exists for a source-to-binary comparison.

The best-supported conclusion is therefore: **same Yichip/YC3121-family register and ROM API lineage is plausible; exact V504 SDK release and exact source revision remain UNKNOWN.**

## 8. Highest-value recovery lead

The single most valuable missing artifact is the original vendor SDK archive or the exact V504 build project containing:

- Librarier/sdk/yc_timer.c and yc_timer.h
- Librarier/sdk/yc_sysctrl.c and yc_sysctrl.h
- The SoC-specific timer/NVIC headers
- The linker/scatter file and build configuration
- The ROM API header and version manifest used for V504

If that package cannot be obtained, a source revision that contains these exact files and can be tied to the V504 build by multiple independent function fingerprints is the next-best target. Do not substitute the YC16xx SDK or infer timer behavior from another Yichip family.

## 9. Cross-references

- [Timer register and acknowledgement reconciliation](PHASE13_9_TIMER_INTERRUPT_ACKNOWLEDGEMENT.md)
- [RGB callback timing model](PHASE13_9_RGB_CALLBACK_TIMING_MODEL.md)
- [Findings and implementation boundary](PHASE13_9_FINDINGS_AND_IMPLEMENTATION_BOUNDARY.md)
- [Phase 13.8 SDK provenance baseline](PHASE13_8_YICHIP_SDK_PROVENANCE.md)
