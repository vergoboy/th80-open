# Phase 13.11 — Findings and Next Steps

## 1. Final decision

**Overall SDK provenance classification: `SAME_SDK_LINEAGE`.**

The Phase 13.11 investigation did not recover the exact V504 SDK, build project, SoC configuration package or timer/system-control source. The evidence continues to support Yichip YC3121-family SDK lineage, but does not establish the exact private SDK revision or prove that the public YC3121 POS BSP was used to build V504.

The vendor distribution path is now better understood: the V504 ZIP is an end-user firmware/updater bundle; the Windows updater is a Rust/HID Rongyuan client; and the active Mac frontend update path obtains firmware version metadata and downloads a compressed firmware payload. None of these artifacts exposes the original development SDK.

## 2. Strongest new discoveries

1. **Complete V504 archive inventory:** exact archive SHA-256 `7a3c128cfa385d458ca1850b9e5d297285c79f7aaca981022334f7140db538e4`; six file members, two directory entries; `dev_fm/` is empty. No SDK or project/build file is present.
2. **Updater provenance:** updater hash `e9f3d8d2f3160be5e3817fb4eca8efe954cc854426351e3d0d21e73f39fce0fc`; PE32+ AMD64 Rust GUI application, PDB path under `rust-hid`, HID/SetupAPI functionality and Rongyuan endpoint strings. No resource directory, certificate table, PDB file or SDK package is embedded.
3. **Three distinct API surfaces:** updater strings reference `api.rongyuan.tech:3000`; a separate frontend module exposes `api.rongyuan.tech:3814/v1/downloadDevUpgradeFile`; the active `upgradeDev()` path uses `api2.rongyuan.tech:3816/api/v2/get_fw_version` plus GET `/download/{file_path}`. The similarly named functions are not interchangeable.
4. **Product ID distinction:** 742 is confirmed in the archive directory name. The frontend dynamically uses `currentDev.deviceType.id` as `dev_id`; no static evidence equates that runtime field with 742.
5. **No API experiment was necessary:** the actual firmware `file_path` is server-supplied and absent from local static artifacts; no exact API response was retrieved and no endpoint was called.

Detailed evidence is in [archive analysis](PHASE13_11_V504_DISTRIBUTION_AND_ARCHIVE_ANALYSIS.md), [updater forensics](PHASE13_11_UPDATER_STATIC_FORENSICS.md) and [API/frontend flow](PHASE13_11_RONGYUAN_API_AND_FRONTEND_FLOW.md).

## 3. Original source recovery status

No new original source was recovered. The following remain unavailable:

- `Librarier/sdk/yc_timer.c`
- `Librarier/sdk/yc_timer.h`
- `Librarier/sdk/yc_sysctrl.c`
- `Librarier/sdk/yc_sysctrl.h`
- Matching SoC timer/clock/interrupt headers and common IRQ wrappers
- Startup assembly/vector table
- Original V504 Keil/IAR/GCC project and compiler configuration
- Original linker/scatter/IAR script and linker map
- Original ELF/AXF/object/debug/PDB artifacts
- ROM API definitions and board/SoC configuration manifest

The updater's `rust_hid.pdb` reference belongs to the Windows updater application, not the ARM keyboard firmware. The updater's Rust compiler source hash and PE linker version are not V504 firmware compiler fingerprints.

The best public source remains the Yichip-authored YC3121 POS BSP at RT-Thread commit `cbc19778d0ae5ef2533a28c0f62c6a399858010a`, which lacks `yc_timer.c/.h` and `yc_sysctrl.c/.h`. The official [Yichip YC3121 repository](https://github.com/Yichip-Microelectronics/YC3121) currently exposes only a README in the inspected public view. It is not the V504 build source.

## 4. Does Rongyuan distribute SDK source through these paths?

**No SDK distribution is evidenced by the inspected local release or frontend code.**

- The V504 ZIP contains firmware, updater configuration and upgrade instructions only.
- The active frontend version endpoint returns firmware metadata used to locate a downloadable firmware file.
- The active file endpoint returns a compressed payload that the frontend inflates before passing it to the device upgrade method.
- The older `/downloadDevUpgradeFile` implementation and the Windows updater's port-3000 URL are firmware-update API references, not documented SDK repositories.
- No public API documentation, SDK catalogue, source archive listing or development-package response was found.

This is a bounded negative result. It establishes that the inspected public/local distribution artifacts do not expose the SDK; it does not prove that EPOMAKER, Rongyuan or Yichip never supplied a private production SDK to a factory/customer.

## 5. Timer acknowledgement and clock implications

Phase 13.11 does not change the Phase 13.10 timer conclusions:

- Original timer driver source remains unavailable.
- Timer1 IRQ14 application wrapper still visibly disables Timer1, runs the RGB output routine, then re-enables Timer1.
- Timer2 IRQ15 handler still has no visible acknowledgement operation in the application handler body.
- Peripheral pending-state acknowledgement remains `UNKNOWN`.
- NVIC pending-clear behavior remains `UNKNOWN` for the V504 runtime path.
- HCLK selector-zero interpretation remains conditional on oscillator/PLL initialization.
- No new evidence establishes Timer1's pending-state behavior across disable/re-enable.

The updater and distribution API do not contain the missing peripheral semantics. Do not infer acknowledgement from updater protocol or API behavior.

## 6. Minimum source needed before a safe RGB driver

Priority recovery remains:

1. Original `yc_timer.c/.h` and matching SoC timer/status/interrupt definitions.
2. Original `yc_sysctrl.c/.h`, oscillator/PLL setup and clock-gating definitions.
3. Shared IRQ registration/wrapper code, vector table and startup assembly.
4. Exact MCU variant/revision configuration and ROM API declarations.
5. Original compiler/project/linker configuration and map/ELF/AXF where available.
6. GPIO driver and exact board pin mapping.

The source must be tied to V504 by a verifiable archive/release/commit and preferably source-to-binary function comparison. Another related SDK with the same family name is not sufficient.

## 7. Evidence classifications

Only the specified labels are used below.

| Finding | Classification | Basis / remaining uncertainty |
|---|---|---|
| Local V504 ZIP member inventory and hashes | `CONFIRMED_STATIC` | Exact archive and extracted members; does not rule out private external package |
| `dev_fm/` empty | `CONFIRMED_STATIC` | Complete ZIP central directory |
| Product ID 742 release-directory label | `CONFIRMED_STATIC` | Directory path only; runtime API ID not equated |
| Updater PE identity, PDB path and embedded endpoint strings | `CONFIRMED_STATIC` | Static PE parsing and strings; no execution |
| Updater is a Rongyuan distribution client using generic HID components | `STRONG_STATIC_INFERENCE` | API URLs, data model, SetupAPI/HID exports; no runtime trace |
| Active Mac frontend version/download flow | `FRONTEND_ONLY` | Static bundle call sites; no API response/request |
| Active API delivers only firmware, not SDK | `STRONG_STATIC_INFERENCE` within inspected flow | Code consumes `file_path` and inflates firmware payload; server behavior outside this client is unknown |
| Exact V504 SDK release/source revision | `UNKNOWN` | No manifest/source package/build artifact |
| Timer1 peripheral acknowledgement | `UNKNOWN` | No original timer driver or authoritative peripheral status/clear semantics |
| Clock source actually active at selector zero | `UNKNOWN` | No matching original clock/startup source recovered |

## 8. Single most valuable next investigation

Request the **original TH80 ANSI V504 production SDK/build package for release directory ID 742** directly from EPOMAKER/Rongyuan (and, if necessary, Yichip). Provide the verified firmware archive and updater hashes and ask specifically for:

- `Librarier/sdk/yc_timer.c/.h`
- `Librarier/sdk/yc_sysctrl.c/.h`
- the exact SoC interrupt/vector and timer-status headers
- startup assembly and clock initialization
- the project/compiler configuration and linker map
- the SDK archive version/checksum and the MCU part/revision used for V504

This is more valuable than another generic YC3121 SDK search or another pass over timer disassembly. If the vendor confirms that only binary firmware is distributed to customers and the production SDK is not releasable, document that response and stop source-recovery work until a separately preserved original package appears.

## 9. Integrity and repository hygiene

Pre-analysis verification matched all four protected hashes:

| Artifact | SHA-256 |
|---|---|
| Firmware `X` | `4b22336ec9ffaaac3372791a1154a2c2a833e3305a4cadbbdcab9184252c0520` |
| Native `iot_driver` | `22c5dc67ba77e2373a5ffdbb0312ed8c8e73eb16e78037764eb676a030b2724b` |
| Frontend `dist/index.js` | `7d53737095fe5756fb0a18d5aecb7593555b9bb61a9d5d160ae0b988694c1e87` |
| Updater EXE | `e9f3d8d2f3160be5e3817fb4eca8efe954cc854426351e3d0d21e73f39fce0fc` |

A pre-write SHA-256 manifest was saved outside the repository at `/tmp/th80-phase13-11/historical-reports-before.sha256` for all Phase 13.8, 13.9 and 13.10 reports. The initial Git status had 100 untracked entries, representing the 96 pre-existing entries plus Phase 13.10's four reports. This phase adds only the four requested reports. No Git history or remotes are created; no files are staged.

Final post-write verification:

- Exactly four Phase 13.11 reports exist and are nonempty; their final sizes and SHA-256 hashes are recorded in the repository status summary delivered with this phase.
- All four reports decode as UTF-8, use LF newlines, end with a newline, have balanced Markdown fences, and cross-reference the other three reports. Every local Markdown link resolves.
- The protected firmware `X`, native `iot_driver`, frontend `dist/index.js` and updater hashes match their expected values after analysis. The exact V504 ZIP hash also remains `7a3c128cfa385d458ca1850b9e5d297285c79f7aaca981022334f7140db538e4`.
- All saved historical-report manifests available for Phase 13.1–13.10 pass. A pre-write manifest covering the Phase 13.8, 13.9 and 13.10 reports was saved outside the repository and passes after report creation.
- Git status increased from 100 to 104 untracked entries. The four additions are exactly the requested Phase 13.11 reports; all 100 baseline entries remain present. Repository root regular files increased from 96 to 100.
- No tracked diff, no staged diff, zero commits and zero remotes.
- No firmware, updater, vendor executable, SDK script, Rust application, native driver, frontend, dependency, GUI or historical report was modified.

The analysis was static: no updater/vendor binary, firmware, SDK script, HID device, USB endpoint or API endpoint was executed or accessed. The final status is closed; no implementation or hardware validation is authorized by this phase.

## 10. Cross-references

- [V504 distribution and archive analysis](PHASE13_11_V504_DISTRIBUTION_AND_ARCHIVE_ANALYSIS.md)
- [Updater static forensics](PHASE13_11_UPDATER_STATIC_FORENSICS.md)
- [Rongyuan API and frontend flow](PHASE13_11_RONGYUAN_API_AND_FRONTEND_FLOW.md)
- [Phase 13.10 source package recovery](PHASE13_10_V504_SOURCE_PACKAGE_RECOVERY.md)
- [Phase 13.10 timer/source comparison](PHASE13_10_TIMER_SDK_SOURCE_COMPARISON.md)
