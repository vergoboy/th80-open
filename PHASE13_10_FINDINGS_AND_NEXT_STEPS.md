# Phase 13.10 — Findings and Next Steps

## 1. Executive conclusion

**Classification: `SAME_SDK_LINEAGE`**

The exact original V504 Yichip SDK package and build project were not recovered. The strongest defensible overall classification is **`SAME_SDK_LINEAGE`**: the V504 firmware contains Yichip SDK source-path strings, and its timer register arithmetic, IRQ numbering/NVIC operations and ROM delay entry correspond to public Yichip YC3121 source definitions. The best available source snapshot is the Yichip-authored YC3121 POS BSP in RT-Thread, commit `cbc19778d0ae5ef2533a28c0f62c6a399858010a`. It is a reference source lineage, not the exact V504 build source.

The four missing source files remain unavailable:

- `Librarier/sdk/yc_timer.c`
- `Librarier/sdk/yc_timer.h`
- `Librarier/sdk/yc_sysctrl.c`
- `Librarier/sdk/yc_sysctrl.h`

The original startup/vector source, linker/project configuration, compiler version, map/ELF/AXF, ROM API revision and complete build provenance are also unavailable.

## 2. Strongest discoveries

1. **V504 SDK path provenance — `CONFIRMED_STATIC`.** The raw firmware image contains relative paths to `yc_gpio.c`, `yc_uart.c`, `yc_timer.c`, `yc_systick.c`, `yc_sysctrl.c`, `yc_wdt.c` and `yc_ssc.c` under `Librarier/sdk/`. These strings identify the source names and Yichip SDK lineage, not the original source content or revision.
2. **YC3121 register model — `STRONG_STATIC_INFERENCE`.** Public Yichip-authored `yc3121.h` timer register formulas and control field positions agree with the recovered firmware helpers and timer addresses.
3. **ROM delay entry — `STRONG_STATIC_INFERENCE`.** Firmware thunk literal `0x4239` exactly matches public `FUNC_DELAY_US_ADDR` (`0x4238 + 1`); adjacent delay-ms entry is `0x425d`.
4. **Public BSP build artifacts recovered.** The RT-Thread YC3121 POS snapshot includes Keil/IAR project files, GCC/ARMCC/IAR build configuration, startup assembly, linker scripts, SoC header and ROM API definitions. Its Keil project names ARMCC 5.06 update 5 build 528, but this is only the BSP project's compiler setting and cannot be attributed to V504.
5. **Vendor distribution mechanism identified — `FRONTEND_ONLY`.** Protected frontend source contains `/downloadDevUpgradeFile` with a device ID request and `api.rongyuan.tech` service configuration. The V504 package itself contains firmware, updater and TOML update configuration, not SDK source.
6. **Exact V504 build provenance unavailable.** No compiler version, linker map, object files, debug information, project file, source manifest or SDK archive was recovered.

## 3. Overall provenance classification

**`SAME_SDK_LINEAGE`**

Why this classification is appropriate:

- More than a generic same-vendor association is present: the V504 firmware contains Yichip SDK source paths, matching timer register arithmetic and a ROM API address fingerprint.
- The public YC3121 POS BSP is authored within the Yichip/YC3121 source lineage and supplies corresponding register/ROM/NVIC definitions.
- The missing timer and system-control implementations prevent exact source-to-binary correlation.
- No SDK archive release ID, source hash, build project relationship, compiler record or build timestamp tie establishes that the BSP snapshot was used to build V504.

`EXACT_BUILD_SOURCE_RECOVERED` and `STRONG_SOURCE_MATCH` would overstate the evidence. `RELATED_SDK_ONLY` would understate the direct source-path/register/ROM correspondences.

## 4. Timer and interrupt conclusion

Timer1 and Timer2 behavior is documented in [the timer SDK source comparison](PHASE13_10_TIMER_SDK_SOURCE_COMPARISON.md). The key boundary is:

- Timer1 init configures channel 1, PCNT 8,814, TIMER mode, auto-reload and NVIC index 14.
- IRQ14 wrapper disables Timer1, invokes the full output routine, then re-enables Timer1.
- Timer2 configures channel 2, a multiplier-derived one-second count and NVIC index 15; its handler writes an application state byte and returns.
- The public timer datasheet says CNT loads when enabled, supporting a fresh Timer1 countdown after re-enable as `STRONG_STATIC_INFERENCE`.
- Neither application handler visibly acknowledges a peripheral status flag or NVIC pending state.
- The missing SDK wrapper/driver and incomplete timer interrupt documentation prevent a definitive answer.

**Peripheral interrupt acknowledgement: `UNKNOWN`. NVIC pending acknowledgement: `UNKNOWN`.** Re-enabling Timer1 must not be described as acknowledgement. Timer2's behavior does not prove an edge-triggered or auto-clear mechanism.

## 5. Clock and RGB implications

The firmware calls the HCLK selector helper with zero before timer initialization. The YC3121-D formula maps selector zero to 96 MHz and selector 14 to 12 MHz, conditional on the expected oscillator/PLL source being initialized and stable. The exact V504 clock startup implementation is unavailable.

The ROM delay-us API fingerprint is strong, but the 7/15/4 microsecond arguments are programmed software delays, not measured output pulse widths. Timer1's nominal 91.8 microsecond countdown is not the RGB callback interval because Timer1 is disabled during the output routine. The actual callback interval includes output execution and interrupt overhead.

The current static evidence is sufficient to describe the firmware reference sequence and its software timing requests. It is not sufficient to implement physical timer/RGB control safely. Exact silicon, peripheral acknowledgement, GPIO mapping/polarity, output electrical behavior, and sleep/wake dependencies remain missing. No hardware testing or transport is authorized by this phase.

## 6. Search coverage and negative-result boundary

The bounded search covered the TH80 project root, Downloads, Documents, `.cache`, `.local/share`, Phase 13.8 and Phase 13.9 directories; exact SDK/build filenames; project archives in the TH80 root; V504 and Mac driver ZIP indexes; local TH80 Git roots/object stores; protected vendor frontend/native artifacts; firmware `X`; and the previously extracted updater. It found no original V504 SDK source archive, timer/system-control source, exact project, map file or compiler manifest.

Two unrelated `.cache/paru` package directories were permission-denied. The search did not exhaust arbitrary browser cache blobs, every user directory, external backups, vendor-private portals or private Yichip delivery channels. Therefore the negative result is bounded to the listed roots and inspected artifacts, not a claim that no such package exists anywhere.

## 7. Required reports

- [V504 source package recovery](PHASE13_10_V504_SOURCE_PACKAGE_RECOVERY.md)
- [Timer SDK source comparison](PHASE13_10_TIMER_SDK_SOURCE_COMPARISON.md)
- [Clock, ROM and interrupt model](PHASE13_10_CLOCK_ROM_AND_INTERRUPT_MODEL.md)

## 8. Single most valuable next investigation

Obtain the original V504 vendor SDK/build archive or a verifiable preserved YC3121 SDK package containing `yc_timer.c/.h`, `yc_sysctrl.c/.h`, matching interrupt headers and the associated project/startup/linker configuration. Ask the vendor for the SDK package associated with TH80 firmware V504 / product ID 742, ideally with release ID, archive checksum and toolchain/build manifest. A random YC3121 package without provenance or function-level correspondence is not enough.

If no package can be obtained, the most useful next source-level comparison is a vendor-provided timer driver/header pair checked against firmware helper offsets `0x1a9b2`, `0x1a9fa`, `0x1aa50` and the IRQ14/IRQ15 wrappers. Do not begin another speculative register analysis or implement hardware behavior from the related BSP.

## 9. Evidence classification matrix

Use only the specified vocabulary:

| Claim | Classification | Boundary |
|---|---|---|
| V504 contains Yichip `Librarier/sdk` source paths | `CONFIRMED_STATIC` | Names only; no source revision/hash |
| Public Yichip YC3121 POS BSP files and build configs exist at inspected commit | `CONFIRMED_STATIC` | Public BSP snapshot, not V504 source |
| Firmware timer helpers agree with public YC3121 register model | `STRONG_STATIC_INFERENCE` | Missing original timer implementation |
| Firmware ROM delay thunk target matches public ROM API address | `STRONG_STATIC_INFERENCE` | ROM implementation/revision not recovered |
| Frontend has device firmware download endpoint | `FRONTEND_ONLY` | Static host-side bundle evidence only |
| Exact SDK release and source revision used by V504 | `UNKNOWN` | No manifest or source/build provenance |
| Timer peripheral pending/acknowledgement | `UNKNOWN` | No exact driver/wrapper or substantive datasheet description |
| NVIC pending clear behavior for Timer1/Timer2 | `UNKNOWN` | No firmware ICPR write; wrapper/source semantics unavailable |
| Exact HCLK source/PLL startup and runtime frequency | `UNKNOWN` | Selector mapping conditional on source setup |
| Actual RGB waveform and physical refresh frequency | `UNKNOWN` | No physical measurements; software requests are not measurements |

## 10. Integrity and repository hygiene

Protected original paths and expected/current hashes:

| Artifact | SHA-256 | Result |
|---|---|---|
| Firmware `X` | `4b22336ec9ffaaac3372791a1154a2c2a833e3305a4cadbbdcab9184252c0520` | MATCH |
| Native `iot_driver` | `22c5dc67ba77e2373a5ffdbb0312ed8c8e73eb16e78037764eb676a030b2724b` | MATCH |
| Frontend `dist/index.js` | `7d53737095fe5756fb0a18d5aecb7593555b9bb61a9d5d160ae0b988694c1e87` | MATCH |

Phase 13.1–13.5 report baseline is available at `/tmp/th80-phase13-6/phase13-reports-before.sha256`; Phase 13.6–13.8 comparison baseline is `/tmp/th80-phase13-9/phase13-6-8-after.sha256` where applicable. Phase 13.9 reports were read only during this phase. The repository had no commits, no remotes, and 96 pre-existing untracked entries at the start. The four Phase 13.10 reports are the only intended new repository files. No file was staged or committed. The final verification pass confirmed the protected hashes, exactly four nonempty reports, UTF-8/LF formatting, balanced Markdown fences, local links and cross-references. The Phase 13.1–13.5 and Phase 13.6–13.8 saved manifests both pass `sha256sum -c`. Phase 13.9 reports were read only; no pre-Phase-13.10 hash manifest for them was available, so no cryptographic before/after comparison is claimed for those four reports.

## 11. Final verification

- Exactly four Phase 13.10 reports exist; all are nonempty UTF-8 files with LF final newlines and no CR characters.
- All four reports cross-reference the other three; all local Markdown links resolve. All code-fence counts are balanced (zero fences in these reports).
- Protected firmware, native driver and frontend hashes match the expected values shown above.
- Saved Phase 13.1–13.5 and Phase 13.6–13.8 report manifests validate successfully. Phase 13.9 historical reports were not modified; no pre-phase hash manifest exists for a cryptographic comparison.
- Git reports zero commits, zero remotes, no tracked diff and no staged diff. The initial 96 untracked status entries remain; the only four additional untracked status entries are the requested Phase 13.10 reports.
- No firmware, native driver, frontend, Rust application, dependency, historical report or original vendor artifact was modified. No SDK scripts, vendor executable, firmware or hardware operation was run.

Phase 13.10 is complete. Stop here; no implementation or further timer speculation is authorized by this phase.
