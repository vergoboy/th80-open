# Phase 6 — Rust driver foundation

Date: 2026-11-01  
Status: architecture and hardware-independent implementation only. No device was opened and no USB/HID request was issued.

## Scope and crate layout

Phase 6 adds a reusable library to `crates/th80-probe` while preserving the existing `th80-probe` executable and its Phase 5A/5B commands. The library is deliberately independent of USB/HID operating-system APIs.

| Module | Responsibility |
|---|---|
| `device` | Typed TH80 Pro / ROYUAN receiver identity, endpoint and interface metadata, descriptor fingerprints, interface-role classification |
| `discovery` | Passive `DiscoveryProvider` and `DescriptorSource` contracts, independent of active device I/O |
| `sysfs` | Linux read-only USB/HID sysfs discovery and kernel-exposed report-descriptor source |
| `transport` | Active Feature/Input/Output operation contract only; `DisabledTransport` rejects every operation |
| `protocol` | Pure in-memory representations/builders for the recovered Feature envelope; macOS and Windows layouts remain separate |
| `capabilities` | Reuses the Phase 5B HID parser and converts parsed descriptors into a typed capability profile |
| `error` | Shared typed errors and conversion from HID descriptor parser errors |

The CLI retains its established diagnostic JSON shape and optional-field/raw-descriptor presentation wrappers. Its enumeration now uses the library `SysfsDiscovery`; device identity and descriptor capability conversion use the shared library models. The CLI does not open device nodes or invoke active transport operations.

## Device identities and captured descriptor profile

| Device | VID:PID | Identity |
|---|---|---|
| EPOMAKER TH80 Pro | `3151:4010` | `DeviceIdentity::TH80_PRO` |
| ROYUAN 2.4 GHz receiver | `3151:4011` | `DeviceIdentity::ROYUAN_RECEIVER` |

Only these exact VID/PID pairs are accepted by `DeviceIdentity::from_vid_pid`. Bootloader PID `3151:4001` is not treated as a supported normal-mode identity.

`InterfaceRole` classifies the captured HID interface layout:

- Interface 0: boot keyboard.
- Interface 1: extended input collections.
- Interface 2: vendor Feature interface only when a 64-byte Feature report capability is present.
- Other/incomplete combinations: `Other` rather than inferred behavior.

Phase 5B captured descriptor fingerprints are preserved in `docs/hid-descriptors.md`: interface 0 `a5395cb7e86bebce11aecb4cc6d73d5d2b23d1c63f28df3c7c90069490213dd8`, interface 1 `3cc697106805cc7be08443be8dae9f4eda2069d7b5c8352e03abdea7e5a98c74`, and interface 2 `350fea62e225e3d92e9db3bbbd408fcebe386c10405454be661d3d2c3229b0b5`. The keyboard and receiver corresponding descriptors were byte-for-byte identical in the captured host inventory. The capability parser continues to consume descriptor bytes only; it does not query a device.

## Feature envelope facts encoded

The descriptor-backed vendor Feature report is 512 bits / 64 bytes, unnumbered, with implicit Report ID 0. `FeatureEnvelope` therefore stores exactly 64 payload bytes and does not insert a synthetic report-ID byte. A non-64-byte envelope is rejected.

### macOS builder

`build_macos_feature` is a pure-memory reconstruction of the Phase 5D path and explicitly requires `MacOsDeviceProfile::MetadataSelector2`, because that is the only metadata selector for which the checksum branches were recovered:

1. Start with the caller message bytes (no prefix).
2. Extend the logical buffer to a minimum of 66 bytes using zero fill; do not truncate a longer message.
3. For `dangledevtype=0`, compute the complemented 8-bit additive sum over logical offsets 1 through 7 inclusive and overwrite offset 7.
4. For `dangledevtype=1`, compute the complemented 8-bit additive sum over offsets 1 through 8 inclusive and overwrite offset 8.
5. For `dangledevtype=2`, do not insert a checksum.
6. Expose the first 64 bytes as the Feature envelope.

The checksum sum includes the pre-overwrite destination byte when it exists in the supplied message; otherwise that position is zero from resize. This models the observed code, not an assertion that these checksum rules are correct for all commands.

### Windows builder

`build_windows_feature` is intentionally separate and returns a variable-length `Vec<u8>`:

1. Prepend the literal byte `0x01`.
2. Append the complete source payload.
3. Resize to a minimum of 65 bytes with zero fill, retaining longer payloads.
4. For selector 0, store the complemented additive sum of the complete source payload at vector offset 8; for selector 1, store it at offset 9.

The `0x01` prefix is represented as a literal prefix only. Its Report ID meaning is unproven for the target descriptor. The Windows checksum input is the complete source payload, unlike the macOS fixed offset windows. No byte-for-byte packet equivalence is claimed.

No command IDs, opcode meanings, response structures, sequence fields, or command-specific semantics are encoded.

## Transport boundary and safety

`DiscoveryProvider` and `DescriptorSource` own passive host observations. `SysfsDiscovery` uses USB sysfs metadata and the `SysfsDescriptorSource` reads kernel-exposed HID report-descriptor files. Neither opens a HID/USB device node or submits reports. `Transport` contains only Feature GET, Feature SET, Input read and Output write; `DisabledTransport` and its default methods return `DriverError::TransportDisabled` for every active operation.

There is no libusb, HIDAPI, hidraw, USB control-transfer, interface-claim, reset, configuration, bootloader or firmware implementation in the library. No transport method contains a fallback to another API. Packet builders cannot access an operating-system device handle; they only manipulate supplied byte slices in memory.

The existing `th80-probe` CLI is preserved as a passive diagnostic path and now consumes the reusable discovery provider. It remains independent of `DisabledTransport`; the latter is not a discovery provider and cannot perform any active operation.

## Error model

`DriverError` includes unsupported device, unavailable device, permission denied, unavailable descriptor, malformed report, invalid packet length, disabled transport and unsupported operation variants, plus a generic I/O diagnostic. The existing HID parser's `ParseError` converts to `MalformedReport`, preserving its offset-bearing diagnostic string. `std::io::Error` with `PermissionDenied` maps to the typed permission error; other I/O errors retain their message in `Io`.

## Unresolved protocol questions

- The vendor Feature report's command and response semantics remain unknown.
- macOS `wIndex=0` remains a hardcoded vendor implementation choice despite the captured Feature descriptor being on interface 2; its intent/validity is unresolved. Phase 6 does not copy that USB setup packet behavior into a transport.
- The Windows `0x01` prefix is not proven to be the target report ID, and the recovered generic Windows Feature caller is not proven to be the V504 firmware-update path.
- The macOS and Windows checksum domains differ; their packet builders must not be merged without new evidence.
- `checksumtype` native consumption, full raw RPC handler mapping and response/error conversion remain unresolved in Phase 5D.
- Captured descriptors establish report types and lengths, not the meaning of report bytes or applicability across unobserved hardware revisions.

## Future implementation gate

Any later transport implementation requires a separately authorized phase, an explicit device/interface selection policy, platform-specific permission/error handling, descriptor revalidation, and dedicated safety review. It must not silently enable operations through this foundation. Active protocol validation, Feature GET/SET, Output/interrupt transfers, interface claiming, resets, configuration changes, firmware and bootloader operations are outside Phase 6.

## Validation

Phase 6 adds synthetic unit tests for exact VID/PID identification, interface classification, captured Feature capability conversion, macOS 64-byte envelope invariants, both checksum branches and the no-checksum branch, Windows prefix/length/checksum behavior, invalid packet lengths, disabled transport methods, parser-error conversion and permission-error conversion. Existing Phase 5B captured-descriptor parser tests remain in place.

Validation results: `cargo fmt --all -- --check` passed; `cargo test --locked` passed (26 library tests, 6 probe tests, 3 workspace tests); `cargo clippy --locked --all-targets -- -D warnings` passed; `cargo build --locked --release -p th80-probe` passed. The existing `th80-probe list` and `capabilities --all` smoke tests both exited successfully and reported the two target devices plus all six captured descriptor fingerprints. Markdown relative-link validation checked 45 links with zero broken links.

The requested Cargo formatting, tests, Clippy and release build checks are recorded in the Phase 6 completion report. The existing `th80-probe` CLI remains the same executable and command surface.

**Safety confirmation:** no device handle was opened; no USB control transfer, HID Feature GET/SET, interrupt transfer, Output report, interface claim/detach, USB reset, configuration change, firmware operation or bootloader operation occurred during Phase 6.
