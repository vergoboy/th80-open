# Phase 13.12 — Hardware Identity and Bootloader Feasibility

## 1. Executive decision

**Replacement-firmware installation: NO-GO on current evidence.**

The strongest static model is a raw V504 image containing two firmware-side executable stub banks: a boot-stage bank at file offset 0x00000 and an application bank at 0x10000. Internal reset pointers support the candidate mapping runtime address = 0x01000200 + file offset. The first bank has a reset/startup path and a mode-dependent dispatcher; the second has its own reset path and application interrupt dispatcher.

This does not establish immutable boot ROM contents, the complete bootloader implementation, the physical flash map, or the bootloader's acceptance rules. The updater's normal/boot VID:PID configuration and generic HID Feature sender are known, but the actual firmware-upgrade call chain, target addresses, erase/program sequence, image checksum policy, signature verification and authentication requirements remain unresolved.

The physical MCU is not identified conclusively. YC3121-family lineage is strongly supported by firmware source-path remnants and related public BSP/register/ROM fingerprints. YC3121-L is the best exact-part lead, not a confirmed marking on the target board. BK3632 is a third-party TH80-family wireless-controller lead, not a confirmed marking on this unit.

## 2. Product and release identity

| Field | Recovered value | Evidence / classification |
|---|---|---|
| Product name | EPOMAKER TH80 Pro | Embedded printable string in firmware X; confirmed static |
| Release family | TH80 ANSI Firmware V504 | V504 archive name and release directory |
| Updater subsystem label | YZW | Windows updater strings/source paths include YZW upgrade components; not proof of the firmware's SoC family |
| Archive release label | Product ID 742 | Directory name TH80 ANSI Firmware V504 (ID 742); confirmed as archive label only |
| Runtime frontend device ID | currentDev.deviceType.id | Frontend code reads this dynamically and submits it as dev_id; actual value unavailable |
| Does runtime ID equal 742? | Unknown | No local device database/API response proves equality |
| Firmware self-reported version | Not recovered | No V504 or 742 string found in raw image X |
| Hardware revision | Unknown | No board marking, BOM or revision manifest |
| Main MCU | YC3121 family candidate | Strong family-lineage inference; exact part unknown |
| Wireless controller | BK3632 candidate | Historical third-party TH80-family review only |
| LED controller | Unknown | No verified IC marking or driver identity |

The numeric archive label 742 is not a USB product ID. It must not be substituted for the frontend's runtime deviceType.id or for a USB PID.

### 2.1 Runtime and archive ID comparison

The protected frontend uses currentDev.deviceType.id in the active version request:

- POST to https://api2.rongyuan.tech:3816/api/v2/get_fw_version
- JSON property: dev_id
- Returned metadata includes version_str, lowest_app_version_str and file_path
- The returned path is downloaded, raw-DEFLATE inflated, and passed to a device-specific upgrade method.

The V504 archive directory contains (ID 742), but the frontend does not hard-code 742. No device/API lookup was made in this phase. The mapping is therefore UNKNOWN, not confirmed or disproven.

## 3. USB, receiver and boot-mode identifiers

The V504 archive's support_config.toml contains generic support and boot-mode HID records. The relevant normal keyboard and boot-mode pair in the Windows updater is:

| Role in updater configuration | VID:PID | Usage page | Usage | Interface | Classification |
|---|---:|---:|---:|---:|---|
| Normal keyboard | 3151:4010 | 0xffff | 0x02 | 2 | Confirmed updater support entry |
| Boot-mode HID device | 3151:4001 | 0xff01 | 0x01 | -1 (wildcard) | Confirmed updater support entry |

Other normal-mode pairs in the archive table:

| VID:PID | Usage page | Usage | Interface |
|---|---:|---:|---:|
| 0461:4002 | 0x0001 | 6 | 0 |
| 3151:4002 | 0x0001 | 6 | 0 |
| 05ac:024f | 0x0001 | 6 | 0 |
| 05a4:0800 | 0x0001 | 6 | 0 |
| 0461:4003 | 0x0001 | 6 | 0 |
| 3151:4003 | 0x0001 | 6 | 0 |
| 0461:4007 | 0xffff | 2 | 2 |
| 3151:4007 | 0xffff | 2 | 2 |
| 3151:4010 | 0xffff | 2 | 2 |

Boot-mode records:

| VID:PID | Usage page | Usage | Interface |
|---|---:|---:|---:|
| 0461:4001 | 0x0001 | 6 | -1 |
| 0461:4001 | 0xff01 | 1 | -1 |
| 3151:4001 | 0x0001 | 6 | -1 |
| 3151:4001 | 0xff01 | 1 | -1 |

The Windows support table does not list receiver PID 3151:4011. The native Mac driver has a separate, broader embedded multi-device configuration. Static string recovery found these unique VID:PID pairs:

3151:4011, 3151:4014, 3151:4017, 3151:4013, 3151:4012, 0461:4002, 3151:4003, 3151:4007, 3151:4008, 3151:4010, 3151:4015, 25a7:2422, 3151:4002, 0461:4003, 25a7:2420, 006f:fd10, 05ac:024f, 25a7:2301, 25a7:2302, 25a7:2303, 0738:bc11, 0738:0a12, 3151:400d, 3151:4001, 3151:4016, 0461:4001, 25a7:2300, 0c45:7040.

The Mac driver strings mark some 3151:4011/4014/4017 records as dongle_common and 3151:4013/4012 records as BLE. These are source-level labels in a generic support table, not proof that each pair belongs to the TH80 Pro. Neither the broad Mac table nor the Windows updater list proves what the exact physical keyboard or dongle currently enumerates as.

No USB serial number, physical descriptor capture, MCU USB-controller part number or exact board USB routing was obtained. No physical device was enumerated.

## 4. Firmware image identity and layout

### 4.1 Artifact

| Property | Value |
|---|---|
| Original archive | /home/vergoboy/Projects/projects/th80/TH80_ANSI_Firmware_V504.zip |
| Raw image | /home/vergoboy/Projects/projects/th80/TH80_ANSI_Firmware_V504/TH80 ANSI Firmware V504(ID 742)/X |
| Size | 184,608 bytes (0x2d120) |
| SHA-256 | 4b22336ec9ffaaac3372791a1154a2c2a833e3305a4cadbbdcab9184252c0520 |
| File identification | Generic raw data; not ELF, AXF, PE, Intel HEX or UF2 |
| Archive composition | Six files; empty dev_fm directory; no SDK or build project |

The first bytes decode as executable stub code under the existing Thumb hypothesis. The raw file has no recognizable standard executable/container magic. That does not rule out a vendor-specific header or metadata convention.

### 4.2 Annotated image regions

| File offset | Length / boundary | Observed contents | Assessment |
|---|---:|---|---|
| 0x00000–0x03fff | 0x4000 bytes | Boot-side executable stubs, reset/startup code, dispatch logic and SDK-related strings/data | Boot-stage firmware component strongly supported |
| 0x04000–0x0ffff | 0xc000 bytes | Zero-filled region | Confirmed padding/gap in this file snapshot; physical flash meaning unknown |
| 0x10000–0x1ffff | 0x10000 bytes | Application reset/vector stubs, secondary interrupt dispatcher, application code and embedded strings | Application firmware component strongly supported |
| 0x20000–0x23fff | 0x4000 bytes | Zero-filled region | Confirmed zero-filled region; physical flash meaning unknown |
| 0x24000–0x2d11f | 0x9120 bytes | Nonzero constants/data and sparse nonzero regions | Contents not assigned a complete section map |

The image's nonzero tail is not a recovered package header or verified signature block. No complete linker map or section table exists, so the last region is intentionally not labelled as a particular firmware section.

### 4.3 Candidate load mapping and reset entries

Internal absolute pointers support the candidate relation:

    candidate runtime address = 0x01000200 + file offset

| Item | File offset | Raw pointer / candidate address | Interpretation |
|---|---:|---:|---|
| Boot vector-bank reset stub | 0x00000 | Candidate bank 0x01000200 | Boot-stage stub bank |
| Boot reset/startup target | 0x00198 | Raw pointer 0x01000401 → normalized 0x01000400 | Candidate boot reset/startup code |
| Application vector-bank reset stub | 0x10000 | Candidate bank 0x01010200 | Application stub bank |
| Application reset/startup target | 0x10128 | Raw pointer 0x01010401 → normalized 0x01010400 | Candidate application reset/startup code |
| Application IRQ dispatcher | 0x1009c | Candidate 0x0101029c | Secondary application interrupt dispatcher |
| Application handler table | 0x100bc | Candidate 0x010102bc | 32-word handler table |

The reset-pointer arithmetic independently supports this offset relation: the boot reset pointer at file offset 0x00198 resolves to runtime 0x01000400, corresponding to file offset 0x00200; the application reset pointer at 0x10128 resolves to runtime 0x01010400, corresponding to file offset 0x10200. This yields the same 0x01000200 offset base for both banks. A Phase 13.7 introductory shorthand used 0x01000000 + file offset for boot-side addresses, which is inconsistent by 0x200 with these pointer correlations. The Phase 13.6 vector-table derivation and the literal pointers support the 0x01000200 relation adopted here. Historical reports are not modified.

The boot-stage dispatcher uses a software-visible mode state through a pointer chain associated with 0x000f8564. Value 2 selects a local boot-mode handler table; the handoff path writes 1 and branches to the application vector bank. This is a software boot/application dispatch interpretation. The hardware meaning of the state address and the silicon's initial reset-vector fetch mechanism remain unknown.

The two stub banks are not a conventional Cortex-M initial-MSP/vector-word table. The exact core profile, hardware vector-fetch mechanism and physical flash mapping remain unconfirmed.

### 4.4 Does X contain a complete image?

The best-supported interpretation is that X contains at least a boot-stage software component and an application software component, plus data/constants. It is not possible to establish whether it is a complete field image, a composite region image, or a vendor-specific partial image without the flash map, boot-ROM specification and packaging documentation.

The image's boot-stage code is not proof that immutable boot ROM is included. Conversely, the existence of a boot-stage code bank does not prove that it is safe or permissible to replace that bank.

## 5. Static updater and package model

### 5.1 V504 archive and frontend processing

The ZIP contains X, three TOML files, the Windows updater and a one-page end-user upgrade guide. It contains no SDK, source archive, compiler project, linker map, flash layout manifest or image-signing manifest. Its dev_fm directory is empty.

The active Mac frontend update flow is:

1. Obtain version metadata using runtime currentDev.deviceType.id.
2. Compare returned version_str and lowest_app_version_str with current device/application version state.
3. Obtain the server-provided file_path.
4. GET the path as an array buffer.
5. Apply raw-DEFLATE decompression.
6. Pass the resulting bytes to a device-specific upgrade method.

This proves frontend-side transport/decompression and version metadata handling only. It does not prove that local X is byte-identical to the server response after inflation, or reveal the device-specific writer's internal protocol.

The Windows updater is a Rust PE32+ AMD64 HID/Rongyuan client. It contains a YZW upgrade subsystem, a normal/boot HID support table, and a dynamically resolved HidD_SetFeature call path. Earlier static disassembly recovered a generic Feature sender that prepends byte 0x01, appends a dynamic payload of N bytes, resizes the buffer to max(65, N + 1), inserts the complemented 8-bit additive sum of the source payload at offset 8 or 9 depending on a selector, and checks the returned write length.

The generic Feature sender's direct call graph was not proven to be the YZW firmware-upgrade path. Its buffer layout and checksum therefore cannot be asserted as the firmware image acceptance format.

### 5.2 Addressing, erase, program and verification

| Question | Static result |
|---|---|
| Is the target flash base known? | Candidate file-offset mapping base 0x01000200 is supported by reset-pointer arithmetic; physical flash mapping is not confirmed |
| Are write target addresses recovered from updater code? | No |
| Is image origin passed as a fixed address or derived from metadata? | Unknown |
| Is a proprietary package header required? | Unknown |
| Does updater write X directly or transform an intermediate container? | Unknown for Windows V504; active Mac frontend performs raw-DEFLATE inflation before device-specific upgrade |
| Is an erase sequence recovered? | No |
| Is a program/chunk sequence recovered? | No complete V504 OTA call chain or packet/address sequence recovered |
| Is read-back verification recovered? | No |
| Is image-size validation recovered? | No authoritative limit or rejection condition recovered |
| Are target device compatibility checks recovered? | HID support configuration is known; exact YZW update-path compatibility branch is not recovered |
| Is a minimum application version check present? | Frontend consumes lowest_app_version_str; exact bootloader-side policy is unknown |

The updater configuration contains timing values: before_start_delay 1000, start_delay 1000, per_delay 20 and end_delay 100. These are updater delays, not MCU clock or flash-programming parameters.

### 5.3 Version, checksum, signature and encryption evidence

**Version handling**

- The active frontend reads firmware version and minimum compatible application version metadata.
- The older Windows updater contains version, lowestAppVersion, DevVersionList, DevVersionInfo and /getVersion strings.
- No exact device-side version rejection rule or downgrade policy was reconstructed.

**Ordinary integrity checks**

- The firmware contains a small complemented wrapping-byte-sum helper, but no recovered call graph connects it to whole-image acceptance or OTA.
- The native Mac driver contains check_sum_type and a generic checksum-error diagnostic in its BLE/OTA-related code. The exact algorithm and its relationship to the Windows V504 updater are not established.
- The Windows updater's generic HID Feature path checks whether the HID write returned the requested byte count. That is transport-write handling, not proof of image integrity validation.
- No V504 image-level checksum algorithm, checksum field or expected checksum value was recovered.

**Authentication and signatures**

- The Windows PE has no embedded Authenticode certificate table. This describes the updater executable, not the keyboard firmware's trust policy.
- No firmware signature manifest, public key, certificate chain, bootloader authentication transcript or signature-verification routine was identified in the inspected artifacts.
- Absence of an obvious RSA string or PE certificate is not evidence that firmware is unsigned. Signature verification could be implemented in boot ROM, undocumented bootloader code, a generic cryptographic routine, or a server/package path not represented by the local artifacts.
- Bootloader authentication requirements remain UNKNOWN.

**Encryption**

- The firmware image contains readable code/data and is not wholly opaque ciphertext.
- No firmware decryption stage, key material or encryption envelope was identified in the inspected update path.
- This does not rule out encrypted subregions, obfuscation, protected ROM operations or a device-side encrypted transport.

**Secure boot**

- No complete secure-boot chain can be reconstructed.
- No evidence establishes whether ROM authenticates the boot-stage image, whether boot-stage code authenticates the application, or whether the updater must authenticate to the bootloader.

## 6. MCU and silicon provenance

| Identifier | Evidence | Classification |
|---|---|---|
| Yichip SDK lineage | Embedded yc_timer.c, yc_sysctrl.c, yc_gpio.c, yc_uart.c, yc_systick.c, yc_wdt.c and yc_ssc.c paths and Yichip API names | Confirmed static lineage strings |
| YC3121 family | Register/IRQ/ROM fingerprints correlated with Yichip-authored YC3121 POS BSP | Strong family inference |
| YC3121-L | Historical TH80-family review and candidate datasheet | Weak exact-part lead; not a board marking |
| YC3121-D / YC3121-H | Not distinguished by image strings | Unknown |
| Cortex-M0 | Public BSP has Cortex-M0 configuration; firmware Thumb-like decode is coherent | Not proven for exact PCB |
| BK3632 wireless controller | Third-party TH80-family review | External lead only |
| PCB revision | No marking or BOM | Unknown |
| LED driver IC | No marking or BOM | Unknown |
| Boot ROM identifier/version | No ROM dump or documented silicon ID | Unknown |

The exact MCU variant, stepping, package, flash size, ROM version and board-specific pin map remain missing. Do not infer any of these from the firmware filename or archive ID.

## 7. Feasibility verdict

Independent firmware installation is technically plausible in principle for a programmable embedded controller, but not demonstrated and not currently safe to attempt. The main blockers are the exact MCU/ABI, memory map, startup/ROM dependencies, and the bootloader's package acceptance and authentication rules.

The image's internal boot/application dispatch structures do not prove that a replacement image can be installed. The updater's VID/PID list does not prove that the bootloader accepts arbitrary payloads. No package modification, update request, hardware operation or flashing attempt was made.

## 8. Cross-references

- [Independent firmware strategy assessment](PHASE13_12_INDEPENDENT_FIRMWARE_STRATEGY_ASSESSMENT.md)
- [Vendor SDK request dossier](PHASE13_12_VENDOR_SDK_REQUEST_DOSSIER.md)
- [Findings and next steps](PHASE13_12_FINDINGS_AND_NEXT_STEPS.md)
- [Phase 13.11 updater static forensics](PHASE13_11_UPDATER_STATIC_FORENSICS.md)
- [Phase 13.11 archive analysis](PHASE13_11_V504_DISTRIBUTION_AND_ARCHIVE_ANALYSIS.md)
- [Phase 13.11 frontend update flow](PHASE13_11_RONGYUAN_API_AND_FRONTEND_FLOW.md)
- [Phase 5C updater correlation](PHASE5C_UPDATER_CORRELATION.md)
- [Phase 5D Windows updater Feature buffer](PHASE5D_WINDOWS_UPDATER_BUFFER.md)
- [Phase 13.6 vector table and IRQ entry](PHASE13_6_VECTOR_TABLE_AND_IRQ_ENTRY.md)
- [Phase 13.7 silicon and boot architecture](PHASE13_7_SILICON_AND_BOOT_ARCHITECTURE.md)
- [Phase 13.10 source package recovery](PHASE13_10_V504_SOURCE_PACKAGE_RECOVERY.md)
