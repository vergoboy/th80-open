# Phase 13.11 — V504 Distribution and Archive Analysis

## 1. Executive result

The exact V504 distribution archive is a firmware/updater release bundle, not a development SDK bundle. Its `dev_fm/` directory is explicitly present but empty. The only payloads are the raw firmware image, three updater TOML configuration files, one updater executable and a one-page upgrade guide. No source archive, SDK library, header, build project, linker script, map, ELF/AXF, compiler manifest or SoC development package is present.

**Overall SDK provenance remains `SAME_SDK_LINEAGE`.** This phase recovered no original V504 SDK source. Product ID 742 is present in the archive's directory name, but the TOML configurations do not contain a `product_id = 742` field, and the frontend does not hard-code 742.

## 2. Exact archive identity

- Local artifact: `/home/vergoboy/Projects/projects/th80/TH80_ANSI_Firmware_V504.zip`
- Vendor download URL recorded for this release: `https://orders.epomaker.com/firmware/TH80_ANSI_Firmware_V504.zip`
- Inspection date: 2026-11-01
- Size: 4,059,612 bytes
- SHA-256: `7a3c128cfa385d458ca1850b9e5d297285c79f7aaca981022334f7140db538e4`
- ZIP comment: empty
- Entry count: 8 including two directory entries; 6 files
- Extraction for analysis: `/tmp/th80-phase13-11/archive/` only. No archive member was extracted into the repository or original firmware tree.

The archive was inspected by ZIP central-directory listing and Python `zipfile`; every file member was hashed after extraction. The ZIP itself was not modified. The archive entry timestamps are DOS ZIP metadata, not independently verified source-build timestamps.

## 3. Complete file inventory

| Relative path | Size | SHA-256 | Detected type / metadata |
|---|---:|---|---|
| `TH80 ANSI Firmware V504(ID 742)/selections_config.toml` | 539 | `91adfbcfaa8b227ddf2984d40ffc5ffb760c34d5e42d4f9ab1439d8ced8e157b` | TOML text; ZIP timestamp 2021-11-08 18:19:50 |
| `TH80 ANSI Firmware V504(ID 742)/support_config.toml` | 1,234 | `95fd1124b020ceec89b987b5f4fa954f831b014e86b1c540f31d93263fca2593` | TOML text; ZIP timestamp 2022-09-08 10:46:28 |
| `TH80 ANSI Firmware V504(ID 742)/TH80 Upgrade Firmware for v5xx.exe` | 10,274,816 | `e9f3d8d2f3160be5e3817fb4eca8efe954cc854426351e3d0d21e73f39fce0fc` | PE32+ AMD64 GUI executable; ZIP timestamp 2021-10-23 11:01:44 |
| `TH80 ANSI Firmware V504(ID 742)/Upgrade Guide.pdf` | 154,048 | `4d7f8c5acf3e3f2bb5f86f409882d758c66b90627437a30b3d8e6d91ae301df4` | PDF 1.7, one A4 page; author NING MEI; creator WPS 文字; created/modified 2022-08-17 12:30:17 +0430 |
| `TH80 ANSI Firmware V504(ID 742)/upgrade_config.toml` | 78 | `4968768bec983b0ad442f21bdecf83e95821465a96b33ff408b4e16664e012c6` | TOML text; ZIP timestamp 2021-09-07 12:19:58 |
| `TH80 ANSI Firmware V504(ID 742)/X` | 184,608 | `4b22336ec9ffaaac3372791a1154a2c2a833e3305a4cadbbdcab9184252c0520` | Raw `data`, not ELF/AXF; ZIP timestamp 2022-09-21 16:32:30 |

Directory entries:

- `TH80 ANSI Firmware V504(ID 742)/`
- `TH80 ANSI Firmware V504(ID 742)/dev_fm/` — empty; no child entries.

The updater member's hash matches the separately extracted Phase 13.9 updater exactly. Firmware `X` matches the protected original firmware hash.

## 4. Configuration TOML analysis

All three TOML files were parsed with Python's standard `tomllib`; parsing succeeded.

### 4.1 `selections_config.toml`

Top-level key: `selections` (array of six records). Fields per record: `selection_id`, `selection_need_show`, `selection_name`.

| ID | Visible | Name |
|---:|---|---|
| 0 | true | `yzw 升级` |
| 1 | false | `fw upload` |
| 2 | false | `cmd test` |
| 3 | false | `24 new test` |
| 4 | false | `holy rob` |
| 5 | false | `set into test` |

The names indicate updater UI modes/test labels, including hidden upload/test choices. They do not contain compiler commands, SDK package names or build targets. The spelling is preserved exactly from the file.

### 4.2 `support_config.toml`

Top-level arrays: `support` (9 records) and `boot` (4 records). Fields: `vid`, `pid`, `usage`, `usage_page`, `interface_number`. Numeric TOML values are hexadecimal where written with `0x`.

`support` records:

| VID | PID | Usage | Usage page | Interface |
|---:|---:|---:|---:|---:|
| `0x0461` | `0x4002` | 6 | `0x0001` | 0 |
| `0x3151` | `0x4002` | 6 | `0x0001` | 0 |
| `0x05ac` | `0x024f` | 6 | `0x0001` | 0 |
| `0x05a4` | `0x0800` | 6 | `0x0001` | 0 |
| `0x0461` | `0x4003` | 6 | `0x0001` | 0 |
| `0x3151` | `0x4003` | 6 | `0x0001` | 0 |
| `0x0461` | `0x4007` | 2 | `0xffff` | 2 |
| `0x3151` | `0x4007` | 2 | `0xffff` | 2 |
| `0x3151` | `0x4010` | 2 | `0xffff` | 2 |

`boot` records:

| VID | PID | Usage | Usage page | Interface |
|---:|---:|---:|---:|---:|
| `0x0461` | `0x4001` | 6 | `0x0001` | -1 |
| `0x0461` | `0x4001` | 1 | `0xff01` | -1 |
| `0x3151` | `0x4001` | 6 | `0x0001` | -1 |
| `0x3151` | `0x4001` | 1 | `0xff01` | -1 |

These are updater-supported HID interface descriptors. The configuration does not label a particular VID/PID as product 742 or as a YC3121 silicon identifier. Do not infer manufacturer ownership or SoC identity from these numbers alone.

### 4.3 `upgrade_config.toml`

Four integer fields:

```toml
before_start_delay = 1000
start_delay = 1000
per_delay = 20
end_delay = 100
```

These are updater timing configuration values. The file does not declare their units or relate them to the MCU's firmware timer clock. They are not firmware build parameters.

## 5. Upgrade guide and firmware payload

`Upgrade Guide.pdf` is one page. Its procedure says to exit the driver, unplug the 2.4 GHz dongle and use wired mode. It warns against opening multiple upgrade tools simultaneously and describes reconnecting by cable after a failed upgrade. It is operational guidance, not a development manual.

PDF metadata: author `NING MEI`; creator `WPS 文字`; creation/modification 2022-08-17 12:30:17 +0430. No SDK or source link is present in the extracted text.

Firmware `X` is 184,608 bytes and identified as generic `data`, not an ELF/AXF image. It contains the string `EPOMAKER TH80 Pro` and Yichip SDK source-path remnants including `Librarier/sdk/yc_timer.c` and `Librarier/sdk/yc_sysctrl.c`. Those remnants establish source naming/lineage, not source contents or a release identity. The protected image SHA-256 is unchanged.

## 6. Product ID 742 and release relationship

`742` appears in the ZIP directory path `TH80 ANSI Firmware V504(ID 742)/`. This is `CONFIRMED_STATIC` evidence that the vendor package labels this release with ID 742. It is not separately declared inside any TOML file, the PDF text, or the raw firmware strings found in this phase.

The frontend obtains a device type ID dynamically and sends it as `dev_id` to the firmware-version endpoint. It does not hard-code 742. No API request was made, so this phase does not prove that the runtime `deviceType.id` for the target keyboard equals 742. The API flow and this distinction are detailed in [the Rongyuan API/frontend report](PHASE13_11_RONGYUAN_API_AND_FRONTEND_FLOW.md).

## 7. Vendor distribution and public development resources

The local archive is associated with the vendor URL `https://orders.epomaker.com/firmware/TH80_ANSI_Firmware_V504.zip`. This phase did not refetch the ZIP over HTTP; therefore no HTTP response headers, content type or current server availability are claimed. The local archive's ZIP format and SHA-256 are directly verified.

Public checks on 2026-11-01:

- Yichip's public [YC3121 repository](https://github.com/Yichip-Microelectronics/YC3121) displays one commit and a README as its only repository file; no public release or package is listed in the inspected view.
- EPOMAKER's [Driver support page](https://epomaker.com/blogs/software/epomaker-driver) lists TH80 and TH80 Pro among supported keyboard families and offers end-user Windows/macOS driver downloads, not a development SDK.
- EPOMAKER's [firmware catalogue](https://epomaker.com/blogs/firmware) is an end-user firmware/upgrade-tool catalogue. Its currently surfaced TH80 Pro V2 entry is a different product generation and is not evidence of the V504 source package.
- Public searches for `TH80 V504`, `productId 742`, `downloadDevUpgradeFile`, `Librarier/sdk/yc_timer.c`, `Librarier/sdk/yc_sysctrl.c`, Rongyuan TH80 SDK and Yichip keyboard SDK did not reveal a verifiable V504 development archive, SDK index or public API documentation.

The public Yichip source and the RT-Thread BSP remain related-family references only. See [Phase 13.10 source recovery](PHASE13_10_V504_SOURCE_PACKAGE_RECOVERY.md).

## 8. Evidence summary

| Claim | Evidence | Classification | Remaining uncertainty |
|---|---|---|---|
| The exact local V504 ZIP has the recorded six files | ZIP path, ZIP SHA-256, central directory and per-member hashes | `CONFIRMED_STATIC` | Original HTTP response headers were not fetched |
| `dev_fm/` is empty | Complete central-directory inventory | `CONFIRMED_STATIC` | None for this archive snapshot |
| Product ID 742 labels the release directory | ZIP member paths | `CONFIRMED_STATIC` | Not tied to runtime `dev_id` without API/device metadata |
| TOMLs describe updater selections, HID support and delays | Parsed TOML content | `CONFIRMED_STATIC` | VID/PID ownership not established |
| Archive includes original SDK/build project | No such member; six-file inventory complete | `UNKNOWN` / not present in this archive | A separate private vendor delivery could exist |
| Public V504 SDK package is downloadable | No verifiable public package found | `UNKNOWN` | Private/customer-only package not ruled out |

## 9. Cross-references

- [Updater static forensics](PHASE13_11_UPDATER_STATIC_FORENSICS.md)
- [Rongyuan API and frontend flow](PHASE13_11_RONGYUAN_API_AND_FRONTEND_FLOW.md)
- [Findings and next steps](PHASE13_11_FINDINGS_AND_NEXT_STEPS.md)
- [Phase 13.10 V504 source package recovery](PHASE13_10_V504_SOURCE_PACKAGE_RECOVERY.md)
