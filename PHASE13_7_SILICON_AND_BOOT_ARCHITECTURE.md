# Phase 13.7 — Silicon and Boot Architecture

## Scope and evidence rules

This report is static analysis of the supplied V504 firmware image `../TH80_ANSI_Firmware_V504/TH80 ANSI Firmware V504(ID 742)/X`, the existing disassembly `/tmp/th80-X-thumb.dis`, and the historical Phase 13 reports. No keyboard, USB/HID endpoint, bootloader, or firmware execution was used. Candidate runtime addresses below follow the Phase 13.6 mapping convention: application image offset `x` is discussed at candidate address `0x01010000 + x`; boot-stage offsets are discussed at candidate address `0x01000000 + x`. These are analysis mappings, not independently proven silicon aliases.

Evidence classifications used throughout this phase are exactly `CONFIRMED_STATIC`, `STRONG_STATIC_INFERENCE`, `WEAK_INFERENCE`, `FRONTEND_ONLY`, and `UNKNOWN`.

## Executive assessment

- The image contains a coherent ARM Thumb instruction stream, including `CPSID i`, `CPSIE i`, `MSR MSP,r0`, Thumb `BL`, and Thumb return idioms. This is much stronger than a superficial resemblance based only on `push`/`pop` mnemonics. **`STRONG_STATIC_INFERENCE`** for ARM Thumb code generation; the exact core profile remains unconfirmed.
- The firmware contains Yichip SDK source-path remnants including `yc_gpio.c`, `yc_uart.c`, `yc_timer.c`, `yc_systick.c`, `yc_sysctrl.c`, `yc_wdt.c`, and `yc_ssc.c`, plus `SYSCTRL_AHBPeriphClockCmd`, `SYSCTRL_EnterSleep`, and `SYSCTRL_HCLKConfig`. **`CONFIRMED_STATIC`** for those embedded strings; **`STRONG_STATIC_INFERENCE`** for a Yichip SDK lineage.
- YC3121-family silicon remains the best family candidate based on the historical TH80-family YC3121-L lead and the firmware's Yichip SDK remnants. The exact `YC3121`, `YC3121L`, or `YC3121H` variant is **`UNKNOWN`** for this board.
- A Cortex-M0 configuration clue was mentioned in the preceding investigation, but the exact BSP/config file was not recoverable in the current supplied workspace tree. It is therefore not used as primary evidence here. The candidate Cortex-M0/ARMv6-M interpretation remains **`WEAK_INFERENCE`**, not a confirmed core identity.
- The firmware's interrupt entry mechanism is a custom executable-stub/secondary-dispatch arrangement, not a conventional architectural Cortex-M vector table. The hardware/ROM initial-vector selection mechanism remains **`UNKNOWN`**.

## ISA and core evidence

### Instruction-level observations

| Firmware offset | Candidate runtime address | Observation | Classification |
|---:|---:|---|---|
| `0x1d332` | `0x0101d332` | `MSR MSP,r0` (`f380 8808`) | `STRONG_STATIC_INFERENCE` |
| `0x1d880`, `0x1d8a6` | `0x0101d880`, `0x0101d8a6` | `CPSID i` and `CPSIE i` around an interrupt-masked operation | `STRONG_STATIC_INFERENCE` |
| `0x103b0` | `0x010103b0` | Thumb `push`, `movs`, `bl`, `pop {r4,pc}` wrapper | `STRONG_STATIC_INFERENCE` |
| Boot stub region `0x00c–0x010` | `0x0100000c–0x01000010` | Register-indirect branches and secondary dispatch | `CONFIRMED_STATIC` for bytes/control flow |

The combined instruction set is consistent with ARM Thumb and includes system-register operations associated with an ARM microcontroller execution model. The disassembly alone does not identify Cortex-M0 versus another ARM core, nor prove that all recovered code is executed natively by the assumed core. The exact core profile is therefore not upgraded beyond a strong Thumb inference.

### SDK and device-family clues

`strings -a X` yields source-path remnants under `Librarier/sdk/` for `yc_gpio.c`, `yc_uart.c`, `yc_timer.c`, `yc_systick.c`, `yc_sysctrl.c`, `yc_wdt.c`, and `yc_ssc.c`. It also contains the SDK API names `SYSCTRL_AHBPeriphClockCmd`, `SYSCTRL_EnterSleep`, and `SYSCTRL_HCLKConfig`. This is direct binary evidence of a Yichip-oriented SDK naming lineage, but it is not a silicon ID register read or a PCB marking.

The historical report [PHASE13_FIRMWARE_RGB_ANALYSIS.md](PHASE13_FIRMWARE_RGB_ANALYSIS.md) records a third-party TH80-family review identifying YC3121-L and BK3632, while explicitly warning that the review does not prove the exact user's board revision. The candidate YC3121-L datasheet is cited in prior reports at pages 11, 52–55, and related system-control pages; it is conditional reference material, not a verified datasheet for the actual PCB.

The previous session mentioned a Yichip BSP configuration with `CONFIG_CPU_CORTEX_M0=y` and `CONFIG_SOC_YC3121`, `CONFIG_SOC_YC3121L`, and `CONFIG_SOC_YC3121H`. A bounded search of the currently available project tree did not reproduce that file: the extracted firmware package contains `X`, upgrade configuration TOMLs and an upgrade guide, but no SDK header/source tree or BSP config. Since the path/package identity cannot currently be cited, this lead is retained as a handoff clue only and is not treated as verified evidence.

**Candidate ranking:**

1. **Yichip YC3121 family** — strongest family-level candidate (`STRONG_STATIC_INFERENCE`) from SDK source names and the historical TH80-family review lead.
2. **YC3121-L** — plausible exact part (`WEAK_INFERENCE`); prior review/datasheet evidence is not tied to the exact PCB.
3. **YC3121 / YC3121H** — cannot be excluded; the firmware strings and available family-level documentation do not distinguish them (`UNKNOWN`).
4. **Cortex-M0 / ARMv6-M** — plausible core profile, but not proven by a reproducible matching BSP configuration or silicon documentation for the exact part (`WEAK_INFERENCE`).

## Boot/application image structure

Phase 13.6 identified two executable stub banks:

| Image region | File offset | Candidate runtime address | Role |
|---|---:|---:|---|
| Boot-stage stubs | `0x00000` | `0x01000200` | Boot-side interrupt stubs and dispatcher |
| Application stubs | `0x10000` | `0x01010200` | Application interrupt stubs and dispatcher |

The boot dispatcher at file offset `0x0ce` reads a pointer from the boot data slot at `0x1b4`, dereferences it to obtain the mode-state address, compares the state with `2`, and selects either the boot-mode local handler table (base pointer literal at `0x19c`, candidate table `0x0100032c`) or the application secondary dispatcher at candidate `0x0101029c`. The boot-mode table's slot 14 is a default trap. The application table at candidate `0x010102bc` has slot 14 raw pointer `0x010105b1`, resolving to handler `0x103b0` after clearing the Thumb low bit. See [PHASE13_6_VECTOR_TABLE_AND_IRQ_ENTRY.md](PHASE13_6_VECTOR_TABLE_AND_IRQ_ENTRY.md).

The structures are executable stubs plus a secondary dispatch table. They do not have the standard Cortex-M vector-table shape of initial MSP, core exception vectors, and external IRQ vectors. No conventional VTOR write was identified in the examined image. The supplied image therefore does not prove what address the hardware or boot ROM fetches on reset, or how it selects the initial stub bank.

## `0x000f8564` boot-mode state

The boot code uses a pointer indirection rather than embedding `0x000f8564` at every access:

- File offset `0x0ce` (candidate `0x010000ce`) loads a pointer from the boot data slot at `0x1b4`, dereferences it, reads the resulting word, and compares it with `2`.
- File offset `0x0f8` (candidate `0x010000f8`) loads the same pointer chain, writes `1` to the pointed-to word, then branches indirectly through the boot vector-bank pointer at `0x1b8`; the historical analysis resolves this to the application stub bank at candidate `0x01010201`.
- File offset `0x108` (candidate `0x01000108`) writes `2` through the same pointer chain and returns. No direct caller was identified in the Phase 13.6 cross-reference work.

The literal `0x000f8564` appears in the boot region's pointer/data material; the code's use is consistent with a software-visible boot-versus-application dispatch state. `1` selects the application forwarding path; `2` selects the local boot-mode table. This establishes the variable's software role, not its hardware role.

**Classification:** `CONFIRMED_STATIC` for the reads/writes and comparisons described above; `STRONG_STATIC_INFERENCE` for the software boot/application dispatch-state interpretation; `UNKNOWN` for whether this address is a ROM service mailbox, remap register, vector-bank selector, or ordinary peripheral-backed state. No evidence justifies calling it VTOR or claiming it changes the hardware's initial vector fetch.

## Datasheet and register-map boundary

The candidate YC3121-L documentation describes a 32-bit RISC core, an NVIC-like block, SysTick/NVIC base addresses, timer channels, and HCLK timer clocking. Its architecture wording does not by itself establish ARM Cortex-M. The firmware's Thumb/system-register instruction stream and NVIC-compatible addresses support an ARM-compatible interpretation, but the mismatch between its custom vector stubs and conventional Cortex-M vector format remains material.

There are also timer-address conflicts between the prior datasheet reading and the current firmware literal-pool analysis. Those are documented in [PHASE13_7_TIMER1_REGISTER_AND_CLOCK_ANALYSIS.md](PHASE13_7_TIMER1_REGISTER_AND_CLOCK_ANALYSIS.md); they must not be resolved by forcing datasheet names onto the binary.

## What would establish the silicon definitively

1. A clear, safely obtained photograph of the exact PCB's main MCU top marking and board revision, or an authorized board BOM.
2. The exact matching Yichip SDK/BSP release including `yc_timer` and `yc_sysctrl` headers/source, SOC selection config, compiler target flags, and linker script.
3. Primary boot-ROM documentation specifying reset address, vector-bank fetch, and interrupt forwarding for the exact part.
4. A vendor silicon ID / ROM identification method documented for the exact MCU. No such read was performed in this static-only phase.

## Cross-references

- [Timer1 register and clock analysis](PHASE13_7_TIMER1_REGISTER_AND_CLOCK_ANALYSIS.md)
- [IRQ14 event and acknowledgement](PHASE13_7_IRQ14_EVENT_AND_ACKNOWLEDGEMENT.md)
- [Findings and hardware boundary](PHASE13_7_FINDINGS_AND_HARDWARE_BOUNDARY.md)
- Historical baseline: [Phase 13.6 vector table and IRQ entry](PHASE13_6_VECTOR_TABLE_AND_IRQ_ENTRY.md)
