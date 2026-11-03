# Phase 13.8 — Yichip SDK Provenance

**Scope:** static source provenance and firmware/source semantic comparison only. No device access or firmware execution was performed.

**Related reports:**

- [Timer register map reconciliation](PHASE13_8_TIMER_REGISTER_MAP_RECONCILIATION.md)
- [Clock and interrupt semantics](PHASE13_8_CLOCK_AND_INTERRUPT_SEMANTICS.md)
- [Findings and implementation boundary](PHASE13_8_FINDINGS_AND_IMPLEMENTATION_BOUNDARY.md)

## 1. Executive conclusion

**Status:** matching SDK family; exact release unknown.

The best-supported conclusion is **Yichip YC3121-family SDK lineage, with a strong semantic match to the Yichip-authored YC3121 POS BSP headers**. The exact SDK package/release used to build TH80 ANSI Firmware V504 cannot be identified from the material available.

The firmware contains source-path remnants for `yc_gpio.c`, `yc_timer.c`, `yc_uart.c`, `yc_systick.c`, and `yc_sysctrl.c`. Independently, the public YC3121 POS BSP contains Yichip-authored register headers whose timer register arithmetic, four-bit timer-control fields, interrupt enumeration, HCLK formula, and clock-gate definitions agree with the firmware's recovered instruction behavior.

However:

- The public Yichip `YC3121` repository contains only a README at the inspected commit, not the SDK implementation.
- The RT-Thread YC3121 POS BSP contains `yc_gpio.c`, `yc_uart.c`, `yc_systick.c`, etc., but **does not contain `yc_timer.c` or `yc_sysctrl.c`**.
- The timer helpers therefore cannot be compared to their original C function bodies or matched byte-for-byte against a known SDK compiler build.
- The public BSP is a YC3121 POS development-board package, not proof that the TH80 PCB carries that exact chip or silicon revision.

Classification: `STRONG_STATIC_INFERENCE` for Yichip YC3121-family SDK lineage and timer/register semantic compatibility; `UNKNOWN` for exact SDK release and exact PCB MCU.

## 2. Search and preservation method

Local-first search covered the project/artifact tree, `/tmp`, `/opt`, and `/usr/local/share` for:

- `yc_timer.c/.h`, `yc_sysctrl.c/.h`, `yc_gpio.c/.h`, `yc_uart.c/.h`, `yc_systick.c/.h`;
- YC3121/Yichip headers, SDK archives, ELF/map/object files, and source packages;
- existing disassembly and firmware metadata.

No complete Yichip SDK source package or local `yc_timer.c` was found. Existing original artifacts were not copied over, replaced, or modified. External material and derived text were kept under `/tmp/th80-phase13-8/`, outside the repository and original-artifact directories.

The firmware binary `X` is raw data, not an ELF file. `/tmp/th80-X-analysis.o` is a generated relocatable ARM object used for disassembly; it is not an original vendor object and supplies no original compiler/linker build identity or debug metadata.

The repository search (excluding `target/` and `.git/`) found no Ghidra/IDA database, firmware ELF/map, standalone `.dis` database, or dedicated Python/shell firmware-analysis scripts. The available disassembly is the generated text file `/tmp/th80-X-thumb.dis` (2,265,825 bytes); the generated `/tmp/th80-X-analysis.o` is a 32-bit little-endian ARM ELF relocatable object (185,132 bytes). The original `X` is 184,608 bytes and is identified by `file` as generic `data`. No build ID, linker map, or vendor compiler version was recovered from the original payload.

## 3. Public source provenance

### 3.1 Yichip vendor repository

- Repository: <https://github.com/Yichip-Microelectronics/YC3121>
- Inspected commit: `077cf0c74d590caeb654deec7fc08b5715e073d7`
- Commit date: `2021-12-30`
- Retrieved/inspected: `2026-11-01`
- Contents at inspected commit: `README.md` only (plus Git metadata); no source archive, timer driver, register headers, or release tags were available in that checkout.
- Classification: `CONFIRMED_STATIC` for the repository contents at that commit; it does not identify the firmware's SDK release.

### 3.2 Yichip-authored YC3121 POS BSP in RT-Thread

- Repository: <https://github.com/RT-Thread/rt-thread>
- Inspected commit: `cbc19778d0ae5ef2533a28c0f62c6a399858010a`
- Commit date: `2026-10-30T18:20:46+08:00`
- Retrieved/inspected: `2026-11-01`
- BSP path: `bsp/yichip/yc3121-pos/`
- Source tree URL: <https://github.com/RT-Thread/rt-thread/tree/cbc19778d0ae5ef2533a28c0f62c6a399858010a/bsp/yichip/yc3121-pos/libraries/core>
- BSP README identifies the board as Yichip YC3121, ARM 32-bit Cortex-M0, 96 MHz, 512 KB flash, 64 KB SRAM. This is board/package documentation, not PCB identification for the TH80.

Yichip-authored file headers and versions in the BSP:

| File | Embedded author/version/date | SHA-256 of inspected file |
|---|---|---|
| `libraries/core/yc3121.h` | Yichip, V1.0, 2018-03-27 | `c4f1a83a3db05458c70e77bbe8847a460a3ac48d2c31659d4174239e6a9f1d6c` |
| `libraries/core/system.h` | Yichip, V1.0, 2018-05-22 | `f90749c1e8e753053df1e63d85e193f90e3263d46ceb9860a41d03c600a65573` |
| `libraries/core/rom_api.h` | Yichip, V1.0, 2020-02-11 | `c8fc4ceed7acd63b4eee51380f9733616ef57c5b8bdfe9cfa9150edb5abe481b` |
| `libraries/core/misc.h` (its file banner says `board_config.h`) | Yichip, V1.0, 2020-07-17 | `6676004f95ece93c48384b109f9188fa7d1dc916e8774d304720634bb4c0ffa9` |

These are per-file version stamps, not a common SDK release identifier. The BSP's `libraries` tree contains GPIO, UART, SysTick, DMA and watchdog source/header pairs, but no timer source/header or `yc_sysctrl.c`. The BSP therefore provides authoritative-looking SoC definitions and neighboring SDK modules, but not the missing timer implementation.

Classification: `CONFIRMED_STATIC` for the inspected source contents and embedded file metadata; `STRONG_STATIC_INFERENCE` that these headers represent the relevant YC3121 SDK family; `UNKNOWN` for exact V504 SDK revision.

### 3.3 Datasheets

Two Yichip datasheets were retrieved and text-extracted on 2026-11-01. Both document the same nine-channel timer register model and the same internal address inconsistencies.

| Document | Source URL | Document metadata | SHA-256 |
|---|---|---|---|
| YC3121-D (BT), V1.2 | <https://img.eecart.com/dev/file/part/spec/YC3121-D-YICHIP.pdf> | 92 pages; PDF creation 2021-07-22 | `47be94eabde5391d48747b4f9159550236e9f76b196d31db076d1d1642198f09` |
| YC3121-L, V1.1 | <https://www.ycble.cn/apks/yc_file/YC3121-L芯片数据手册V1.1.pdf> | 82 pages; PDF creation 2021-06-08 | `e04c5b0c04c29f652331d14c617beb2969ad1f2bc99032f746573e07ff2aa174` |

The D datasheet timer chapter is section 12, pages 63–66; its clock description is section 3.8, pages 22–23, and `SYSCTRL_HCLK_CON`/`SYSCTRL_CLK_CLS` are section 3.10.7/3.10.9, pages 28–30. The L datasheet timer chapter is section 11, pages 52–56; its clock-control material is section 3.10.7/3.10.9, pages 25–28.

Both datasheets agree on:

- nine independent timer channels and nine independent timer interrupt sources;
- down-counting;
- HCLK as the timer clock;
- four-bit per-channel control fields;
- the PCNT, CNT, and control register tables.

Both also repeat the same apparent register-address typos in prose. This suggests a copied documentation error, not evidence of different register maps or silicon revisions.

## 4. Independent firmware fingerprints

### 4.1 Embedded source-path remnants

The firmware contains Yichip SDK source-path strings including `yc_gpio.c`, `yc_timer.c`, `yc_uart.c`, `yc_systick.c`, and `yc_sysctrl.c`. These establish SDK vocabulary and lineage, but are not by themselves a version fingerprint.

Classification: `CONFIRMED_STATIC` for the strings; `WEAK_INFERENCE` for any attempt to infer an exact release from them alone.

### 4.2 Timer helper versus SoC header

Firmware file offsets are shown below; candidate runtime addresses assume the application's established `0x01000000 + file_offset` mapping.

| Firmware function | File offset / candidate runtime | Recovered behavior | YC3121 header correlation |
|---|---|---|---|
| channel enable/disable | `0x1a9b2` / `0x0101a9b2` | `1 << (channel*4)`; RMW of control word at literal base `0x000f0c40 + 8` for channels 0–7, or `+0x0c` for channel 8 | `TIM_CTRL_ENABLE = 0`; four-bit channel field |
| configure timer | `0x1a9fa` / `0x0101a9fa` | disable channel; write PCNT at `0x000f0c40 - 0x40 + channel*8`; OR `0xC << (channel*4)` into control field | `TIM_PCNT(x) = TIMER_BASEADDR + x*8`; mode and auto-reload bits are field positions 2 and 3 |
| mode selector | `0x1aa50` / `0x0101aa50` | set/clear `0x4 << (channel*4)` | `TIM_CTRL_MODE = 2` |
| clock-gate helper | `0x1aa44` / `0x0101aa44` | OR bit `0x10` into `0x000f8540 + 0x2c = 0x000f856c` | `SYSCTRL_CLK_CLS`; bit 4 is timer-module clock gate; set means clock off |

**Important instruction-level correction:** the disassembly at `0x1a9c8` is `lsls r1, r4, #2`, followed by `lsls r0, r1`. The shift is `channel*4`, not `channel*2`. The same `#2` then register-controlled shift is present at `0x1aa32–0x1aa36` and `0x1aa68–0x1aa6e`. Thus the user-supplied `channel × 2` shorthand is not supported by the binary; the four-bit stride is independently corroborated by the header and datasheet.

This is a **semantic source match**, not an exact binary match. No `yc_timer.c` implementation was found for direct source-to-machine-code comparison.

Classification: `CONFIRMED_STATIC` for the instructions and literal values; `STRONG_STATIC_INFERENCE` for the semantic mapping to the YC3121 timer header.

### 4.3 Timer interrupt enumeration

The Yichip `misc.h` enum assigns `TIM0_IRQn=13`, `TIM1_IRQn=14`, `TIM2_IRQn=15`, and subsequent timer channels sequentially. Firmware Timer1 initialization at `0x10240` enables NVIC index 14; Timer2 initialization at `0x10214` enables index 15. This is an independent match between binary behavior and SoC interrupt definitions.

Classification: `CONFIRMED_STATIC` for the header enum and firmware helper arguments; `STRONG_STATIC_INFERENCE` for the SoC identity association.

### 4.4 Clock and clock-gate fingerprints

`yc3121.h` defines `CLKGEN_BASEADDR=0xf8560`, `SYSCTRL_HCLK_CON` at that base, and `SYSCTRL_CLK_CLS` at base `+0x0c` (`0xf856c`). It defines `CPU_MHZ = 192 MHz / ((SYSCTRL_HCLK_CON & 0x0f) + 2)` for POS_CHIP.

Firmware function `0x1b362` reads `0x000f8540 + 0x20` (`SYSCTRL_HCLK_CON`), preserves bits `[31:4]`, and writes the requested selector into bits `[3:0]`. Startup calls it with argument zero at file offset `0x1028c`, before Timer1/Timer2 initialization. Other callsites select 14 or 0 during operating-mode transitions.

Firmware helper `0x1aa44` accesses `0x000f8540 + 0x2c` (`SYSCTRL_CLK_CLS`) and sets bit 4. The SDK header's `disable_clock(id)` macro ORs `1 << id`; the datasheet defines bit 4 as the TIM module clock gate, where 1 means clock disabled. No direct call to `0x1aa44` or pointer reference to its Thumb entry was found, so this is a compiled helper's semantics, not evidence that the active timer path gates the module clock.

Classification: `CONFIRMED_STATIC` for firmware accesses and header macros; `STRONG_STATIC_INFERENCE` for semantic correspondence.

## 5. Version fingerprint assessment

| Fingerprint | Result | Weight |
|---|---|---|
| Yichip SDK source path names | Present in firmware | Lineage only |
| YC3121 POS source header and register map | Strong agreement with timer helpers, IRQ indices and clock registers | Strong family-level match |
| Yichip header version dates | 2018/2020 components | Package-era clue, not a unified release |
| YC3121-D and YC3121-L datasheets | Same timer architecture and register map; same prose typos | Strong hardware-family corroboration |
| Exact `yc_timer.c` source | Not found in public BSP or local files | Missing decisive source |
| Exact binary/source compilation match | Not possible without timer C source and matching compiler/build flags | Missing decisive fingerprint |
| Original ELF, map, debug symbols or build manifest | Not present in original `X` artifact | Missing decisive fingerprint |

**Conclusion classification:** `STRONG_STATIC_INFERENCE` — matching Yichip YC3121 SDK family / SoC definition lineage; exact SDK release unknown. No claim of an exact SDK version is made.

## 6. What would close SDK provenance

The minimum additional source evidence is an authenticated original V504 SDK package or repository revision containing:

1. `Librarier/sdk/yc_timer.c` and its matching timer header;
2. `yc_sysctrl.c` / matching SoC register header;
3. package manifest, version string, tag, commit, or dated archive metadata;
4. ideally the original compiler/toolchain and link map.

A source implementation matching the four helper algorithms and the same literal base/register calculations would raise confidence. A binary-identical match additionally needs compiler, optimization, target flags, and link/layout equivalence.

## 7. Reproducibility paths

The external source checkout and extracted datasheets used for this report are under `/tmp/th80-phase13-8/`:

- `/tmp/th80-phase13-8/rt-thread/bsp/yichip/yc3121-pos/`
- `/tmp/th80-phase13-8/sdk/official-YC3121/`
- `/tmp/th80-phase13-8/docs/YC3121-D-V1.2.pdf`
- `/tmp/th80-phase13-8/docs/YC3121-D-V1.2.txt`
- `/tmp/th80-phase13-8/docs/YC3121-L-V1.1.pdf`
- `/tmp/th80-phase13-8/docs/YC3121-L-V1.1.txt`

These are research inputs only; no downloaded source was added to the project repository.
