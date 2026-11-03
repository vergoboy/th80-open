# Phase 13.11 — Updater Static Forensics

## 1. Executive result

The V504 updater is a vendor-distribution client built as a Rust Windows GUI executable with HID/USB enumeration support and Rongyuan API references. Its transport/UI foundations are generic HID/Windows components; its embedded firmware endpoints and data models are Rongyuan-specific. It is **not** a Yichip SDK installer, firmware compiler, BSP package or build tool.

No SDK source, SoC header, timer driver, system-control source, compiler configuration for the keyboard firmware, linker script, firmware symbols, or original V504 build metadata was recovered from the executable.

## 2. Identity and integrity

- Original archive member: `TH80 ANSI Firmware V504(ID 742)/TH80 Upgrade Firmware for v5xx.exe`
- Extracted Phase 13.9 copy: `/tmp/th80-phase13-9/vendor-metadata/TH80-Upgrade-Firmware-v5xx.exe`
- Analysis copy: same extracted file; never executed.
- Size: 10,274,816 bytes
- SHA-256: `e9f3d8d2f3160be5e3817fb4eca8efe954cc854426351e3d0d21e73f39fce0fc`
- Archive member and extracted copy hashes match exactly.
- `file`: PE32+ executable, Windows GUI, AMD64, five sections.
- Inspection date: 2026-11-01.

## 3. PE headers and sections

| Field | Value |
|---|---|
| Machine | AMD64 (`0x8664`) |
| Optional-header magic | PE32+ (`0x20b`) |
| COFF timestamp | 2021-10-23 03:00:25 UTC (`0x61737ac9`) |
| Linker version fields | 14.29 |
| Image base | `0x140000000` |
| Entry point RVA | `0x6163e4` |
| Subsystem | Windows GUI |
| Section count | 5 |
| Image size | `0x9d1000` |
| Resource directory | RVA/size zero |
| Security/certificate directory | RVA/size zero |
| Debug directory | RVA `0x84c010`, size `0x54` |
| CLR directory | zero |
| COFF symbol table | zero |

Sections:

| Section | RVA | Virtual size | Raw file offset | Raw size |
|---|---:|---:|---:|---:|
| `.text` | `0x1000` | `0x63366a` | `0x400` | `0x633800` |
| `.rdata` | `0x635000` | `0x33787e` | `0x633c00` | `0x337a00` |
| `.data` | `0x96d000` | `0x4438` | `0x96b600` | `0x3600` |
| `.pdata` | `0x972000` | `0x51894` | `0x96ec00` | `0x51800` |
| `.reloc` | `0x9c4000` | `0xc13c` | `0x9c0600` | `0xc200` |

The raw end of the final section is `0x9cc800`, equal to the file length. There is no appended overlay after the section data. The earlier virtual-size-only estimate of an overlay is not applicable; the raw `.reloc` section occupies the remaining bytes.

No PE resource directory means no embedded PE version-resource tree or resource-based SDK package. No security directory means no embedded Authenticode certificate table. This does not establish whether a detached signature ever accompanied the file.

## 4. Debug and build provenance

The PE debug directory contains:

- CodeView entry with PDB GUID `{C215AEE3-87E2-43DB-B791-B1A9DD29DCEB}`, PDB age `55`, and path `C:\Users\Windows\Documents\rust-hid\target\release\deps\rust_hid.pdb`.
- VCFeature debug entry.
- POGO debug entry with profile/section records.

The PDB itself is not included in the V504 archive. The embedded path is a build-machine path and does not recover the source tree.

Printable strings include:

- `C:\Users\Windows\Documents\rust-hid\target\release\build\rust-hid-a8b52b6b97a4a66d\out/holy_rob.rs`
- Rust standard-library source paths under `/rustc/c8dfcfe046a7680554bf4eb612bad840e7631c4b/`
- Cargo registry paths for `glium-0.30.1` and `winit-0.25.0`
- `rust-hid` build/dependency references

The Rust source-tree commit hash is a compiler standard-library source fingerprint, not a `rustc --version` string and not the Yichip keyboard-firmware compiler. Linker version 14.29 is PE toolchain metadata and does not identify the firmware compiler. The updater's PE timestamp and PDB path must not be attributed to the V504 firmware build.

## 5. Imports and exports

The import table references Windows libraries including:

- `SETUPAPI.dll` (`SetupDiGetClassDevsA`, interface enumeration/detail and device registry functions)
- `OPENGL32.dll` (WGL context functions)
- `bcrypt.dll`, `ntdll.dll`, `ADVAPI32.dll`, `CRYPT32.dll`
- `KERNEL32.dll`, `USER32.dll`, `GDI32.dll`, `COMCTL32.dll`, `SHELL32.dll`, `WS2_32.dll`
- `VCRUNTIME140.dll` and Universal CRT API-set DLLs

The executable exports 21 HID-style functions, including `hid_enumerate`, `hid_open`, `hid_open_path`, `hid_read`, `hid_write`, `hid_get_feature_report`, `hid_send_feature_report`, `hid_init` and `hid_exit`. The export-table image name is `rust_hid.exe`.

Together, SetupAPI imports and HID-style exports support a generic Windows HID transport/UI role. They do not identify a Yichip-only bootloader protocol. The presence of `hid_send_feature_report` as an exported symbol does not prove that this analysis executed or invoked it; no HID operation was performed.

## 6. Embedded strings and Rongyuan protocol references

The updater contains these backend URLs:

- `http://api.rongyuan.tech:3000/v1/downloadDevUpgradeFile`
- `http://api.rongyuan.tech:3000/v1/admin_upload_firmware`
- `http://localhost:80`

It also contains `/getVersion` and data-model strings:

- `DevVersionList`
- `DevVersionInfo`
- `DeviceList`
- `DeviceUser`
- `User`
- `devId`, `version`, `lowestAppVersion`, `devices`, `deviceId`, `devIdStr`, `upload_firmware`, `orderid`

These are `CONFIRMED_STATIC` embedded strings. They show a Rongyuan firmware distribution/version data model in an older API generation. The `admin_upload_firmware` string is not evidence that this updater performs an upload during ordinary update operation; no request was sent and no control flow was executed.

Targeted string inspection did not recover `TH80`, `V504`, `742`, `YC3121`, `Yichip`, `Librarier`, `yc_timer.c` or `yc_sysctrl.c` identifiers from the updater. The firmware/product identity instead appears in the ZIP path, firmware payload and associated frontend/configuration artifacts.

## 7. Embedded package and SDK search

Static inspection covered PE headers, sections, data directories, resources, imports, exports, debug directory, strings, archive recognition and trailing file bytes.

Findings:

- No PE resource directory.
- No certificate table.
- No appended archive overlay.
- No embedded PDB file; only its path/GUID reference.
- No SDK C source, header, startup assembly, linker script, SoC register header or ROM API definition recovered.
- No original firmware ELF/AXF, map, symbol table or DWARF data recovered.
- No compiler command line or Yichip SDK release identifier recovered.

The updater may contain internal compiled application code and generic Rust/HID dependencies, but it does not contain an identifiable development package. Its PDB reference is for `rust_hid`, not for the ARM keyboard firmware.

## 8. Classification

| Question | Finding | Classification |
|---|---|---|
| PE identity, hash, timestamp, sections and data directories | Direct PE parse | `CONFIRMED_STATIC` |
| Rust/HID application build provenance | PDB path, Rust source paths, HID exports and SetupAPI imports | `STRONG_STATIC_INFERENCE` |
| Rongyuan backend association | Embedded Rongyuan URLs and API model strings | `CONFIRMED_STATIC` |
| Yichip-specific updater protocol | No MCU/SDK/SoC identity; generic HID support | `UNKNOWN` |
| Original V504 SDK embedded in updater | No such files/resources found | Not recovered |
| Exact Rust compiler release | No release string or PDB | `UNKNOWN` |
| Exact V504 firmware compiler | Updater metadata is unrelated | `UNKNOWN` |

The updater is best described as a **Rongyuan firmware distribution client with generic HID/Windows transport components**, not as the V504 development environment.

## 9. Cross-references

- [V504 archive and configuration inventory](PHASE13_11_V504_DISTRIBUTION_AND_ARCHIVE_ANALYSIS.md)
- [Rongyuan API and frontend flow](PHASE13_11_RONGYUAN_API_AND_FRONTEND_FLOW.md)
- [Findings and next steps](PHASE13_11_FINDINGS_AND_NEXT_STEPS.md)
- [Phase 13.10 timer/source comparison](PHASE13_10_TIMER_SDK_SOURCE_COMPARISON.md)
