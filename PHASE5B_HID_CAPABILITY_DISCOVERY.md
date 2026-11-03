# Phase 5B — HID Descriptor & Capability Discovery

## Status and scope

Phase 5B extends the independent `th80-probe` utility with a passive HID Report Descriptor parser and a versioned capability-profile JSON output. The parser operates only on descriptor bytes already exposed by Linux under `/sys/class/hidraw/*/device/report_descriptor`; it never opens a HID or USB device node. The previously captured descriptor bytes in [docs/hid-descriptors.md](docs/hid-descriptors.md) are used as regression fixtures.

Both the keyboard (`3151:4010`) and ROYUAN receiver (`3151:4011`) were present during final verification. The CLI read all six live descriptors from Linux sysfs, parsed them, and confirmed the hashes and report definitions against the existing captures. The keyboard and receiver descriptors remain byte-for-byte identical interface-by-interface.

## Parser design

The parser is isolated in `crates/th80-probe/src/hid.rs` and has no dependencies beyond the crate's existing `serde` support. It consumes a byte slice and returns either a structured `ParsedDescriptor` or a location-specific `ParseError`.

- Decodes HID short-item size/type/tag prefixes, including the 0/1/2/4-byte data sizes.
- Recognizes long-item framing (`0xfe`), validates its header and data bounds, records its tag/data/offset, and emits a warning because long-item semantics are not interpreted.
- Maintains a global state stack for Push/Pop.
- Maintains local Usage, Usage Minimum/Maximum, Designator Index/Minimum/Maximum and String Index/Minimum/Maximum state, resetting local state after each Main item.
- Tracks nested Collection/End Collection context and rejects unmatched or unclosed collections.
- Groups fields by `(report type, report ID)`, preserving each Main item's Report Size/Count and flags.
- Handles explicit Report IDs and implicit Report ID 0. It rejects invalid ID zero and a descriptor that mixes implicit ID 0 reports with numbered reports.
- Computes descriptor SHA-256 with a small dependency-free implementation.
- Emits warnings for reserved or recognized-but-not-expanded items instead of silently pretending to understand them. Truncated items, invalid report declarations, stack underflow, and unbalanced structures are errors.

### Supported items

| Item class | Supported tags / behavior |
|---|---|
| Main | Input, Output, Feature, Collection, End Collection |
| Global | Usage Page, Logical Minimum/Maximum, Physical Minimum/Maximum, Unit, Unit Exponent, Report Size, Report ID, Report Count, Push, Pop |
| Local | Usage, Usage Minimum/Maximum, Designator Index, String Index |
| Long | Bounds-checked and retained as raw tag/data metadata; semantic interpretation is not defined |
| Other / reserved | Explicit parser warnings; not silently discarded |

Designator and String range bounds are retained as metadata but are not expanded into individual indices. Delimiter alternate usage sets are warned about and not modeled. The parser stores logical/physical ranges, unit and unit exponent on each report field, but does not infer higher-level device behavior from them.

## Report length rules

For each Input, Output or Feature Main item:

`field_payload_bits = Report Size × Report Count`

Fields sharing the same report type and Report ID are accumulated. The report payload length is rounded up once after aggregation:

`payload_bytes = ceil(total_payload_bits / 8)`

For a report with an explicit Report ID, the serialized report-transfer representation includes one additional ID byte:

`transfer_bytes = payload_bytes + (explicit Report ID ? 1 : 0)`

For an unnumbered descriptor, Report ID 0 is implicit and no ID byte is added to the HID payload. API-specific conventions (for example HIDAPI's caller buffer convention) are outside this descriptor-level calculation. A report's `report_size_bits` is `null` when different Main items contributing to the same report use different Report Size values; the exact per-field sizes and counts remain available under `fields`.

USB endpoint maximum packet size is separate metadata. It is displayed and serialized as `endpoints[].max_packet_size`; it is never used as a substitute for a HID report length. A HID report may be larger than one interrupt packet, and Feature reports are not inferred from interrupt endpoints.

## CLI

```sh
cargo run --locked -p th80-probe -- capabilities --device keyboard
cargo run --locked -p th80-probe -- capabilities --device receiver
cargo run --locked -p th80-probe -- capabilities --all
cargo run --locked -p th80-probe -- capabilities --json
cargo run --locked -p th80-probe -- capabilities --device keyboard --json
```

All Phase 5A `list` and `inspect` commands remain available. `capabilities --json` emits a versioned `passive_hid_capabilities` snapshot with device identity, interface/HID metadata, descriptor hash, parsed reports, counts, warnings/errors, long-item metadata and endpoint descriptors. When both devices are present, text mode prints a keyboard-versus-receiver comparison.

If a device or descriptor is unavailable, the CLI reports it as unavailable; it does not attempt a control transfer or an alternate active retrieval path.

## Captured TH80 and receiver findings

The following definitions are derived from the captured descriptor bytes in `docs/hid-descriptors.md`. The two products' corresponding interface descriptors have matching SHA-256 hashes on the captured host.

| Device | Interface | SHA-256 | Input reports | Output reports | Feature reports |
|---|---:|---|---:|---:|---:|
| Keyboard and receiver | 0 | `a5395cb7e86bebce11aecb4cc6d73d5d2b23d1c63f28df3c7c90069490213dd8` | 1 | 1 | 0 |
| Keyboard and receiver | 1 | `3cc697106805cc7be08443be8dae9f4eda2069d7b5c8352e03abdea7e5a98c74` | 5 | 0 | 0 |
| Keyboard and receiver | 2 | `350fea62e225e3d92e9db3bbbd408fcebe386c10405454be661d3d2c3229b0b5` | 0 | 0 | 1 |

### Interface 0 — boot keyboard

- Implicit Report ID 0.
- Input: 64 bits / 8 payload bytes (modifier bitmap and six-key array).
- Output: 8 bits / 1 payload byte (LED state and padding).
- The interface's interrupt-IN endpoint packet size is 8 bytes. That endpoint fact is independent of the descriptor-derived report lengths.

### Interface 1 — extended input collections

Explicit Report IDs are used. No Report ID 4 is defined.

| Report ID | Type | Payload bits | Payload bytes | Transfer bytes |
|---:|---|---:|---:|---:|
| 1 | Input | 120 | 15 | 16 |
| 2 | Input | 8 | 1 | 2 |
| 3 | Input | 16 | 2 | 3 |
| 5 | Input | 24 | 3 | 4 |
| 6 | Input | 56 | 7 | 8 |

The descriptor identifies keyboard, system-control, consumer-control, mouse and vendor-defined input usages/collections. It does not define Output or Feature reports on this interface. Its USB interrupt-IN endpoint packet size is 16 bytes; the individual report transfer sizes above are calculated separately.

### Interface 2 — vendor Feature report

- Implicit Report ID 0.
- One Feature report: 512 bits / 64 payload bytes.
- Usage Page `0xffff`, Usage `0x02`.
- No Input or Output Main items are defined in this report descriptor.
- The USB interface also exposes an 8-byte interrupt-IN endpoint, but that endpoint does not define or size the 64-byte Feature report.

The descriptor establishes the report's type, usage and length only. It does **not** establish command opcodes, field meanings, checksums, sequencing, request/response framing, or whether a particular vendor-driver code path targets this interface correctly. The 64-byte Feature payload size is descriptor-derived; protocol semantics remain unverified. No Feature GET or SET was performed.

## Keyboard versus receiver

The live Phase 5B enumeration found three HID interfaces on each product, with matching corresponding descriptor hashes and the same report definitions. Their USB VID/PID and device identity differ (`3151:4010` versus `3151:4011`). The capability JSON smoke test confirmed both devices and all six descriptor lengths/hashes.

## Tests and validation

Hardware-independent tests cover a simple keyboard, vendor Feature report, multiple IDs, Input/Output, nested collections, Push/Pop, long items, malformed/truncated items, report-ID conventions, payload/transfer calculations, SHA-256, and all three captured TH80 descriptor fixtures. Captured descriptor hashes are asserted against the existing research record.

## Safety guarantees

The Phase 5B capability path only reads the existing Linux USB/HID sysfs metadata and kernel-exposed report descriptor files. It does not:

- issue HID Feature GET/SET or any USB control transfer;
- send Output or interrupt reports;
- open `/dev/hidraw*` or `/dev/bus/usb/*`;
- claim or detach an interface;
- reset or reconfigure a device;
- enter a bootloader, invoke an updater, or read/write firmware.

The parser operates entirely on memory buffers. Test fixtures are local copies of already-captured descriptor bytes. No active device communication is part of Phase 5B.

## Remaining unknowns and boundary

- The semantics and framing of the 64-byte vendor Feature report remain unknown.
- The vendor driver's statically recovered EP0 request path and its interface-index mismatch remain unresolved; this phase does not test that path.
- The report parser does not expand Delimiter alternate usage sets or Designator/String ranges into individual entries, and does not assign vendor-specific meanings to usages.
- Linux sysfs remains the only live descriptor source supported by this utility.
- HID descriptor fields not exposed by the existing sysfs collection (such as the full HID class descriptor metadata) are not fabricated.

Recommended next work is a static-only correlation of the parsed report fields/usages with the existing macOS driver and updater call-chain evidence. Do not infer protocol semantics or begin active protocol testing from the descriptor alone. Phase 5B ends here; Phase 5C is not started.
