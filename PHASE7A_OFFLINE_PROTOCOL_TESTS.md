# Phase 7A — Offline protocol test harness

Date: 2026-11-01  
Status: complete; all validation is offline. No physical transport is enabled or invoked.

## Scope

Phase 7A audits the Phase 6 packet builders against the exact Phase 5D buffer reports and adds an offline regression suite. Packet builders and GET simulation operate only on in-memory byte slices. The tests do not open device nodes, enumerate USB devices, or call any transport backend.

## Phase 6 audit and correction

The Phase 6 byte offsets and most length rules matched the Phase 5D reports. One precondition needed to become explicit: the macOS checksum branches were recovered for the target device metadata selector `device+0x3a == 2`, but the Phase 6 builder accepted only `dangledevtype` and therefore appeared to generalize those branches to every metadata profile.

The API now requires `MacOsDeviceProfile::MetadataSelector2`. No other metadata profile is represented because no other selector's behavior is verified. The macOS builder also exposes both the logical buffer and its 64-byte transfer image so the 66-byte minimum allocation/resize behavior can be tested independently from the fixed transfer length.

## Fixture provenance

Fixtures are deliberately divided into two provenance classes:

1. **Captured evidence:** `crates/th80-probe/tests/fixtures/captured_interface2_descriptor.hex` is the exact 20-byte HID interface-2 report descriptor already published in `docs/hid-descriptors.md`. The test checks its captured SHA-256 (`350fea62e225e3d92e9db3bbbd408fcebe386c10405454be661d3d2c3229b0b5`), Feature report type, 64-byte report length and implicit Report ID 0. Phase 5B states that these are descriptor captures from Linux sysfs, not traffic captures.
2. **Synthetic protocol inputs:** Every packet payload in `tests/offline_protocol.rs` is generated in memory from simple byte patterns. Expected output is calculated from the independently stated Phase 5D offsets, length rules and checksum domains. The synthetic arrays are not presented as captured command or response packets.

The supplied Phase 5 evidence contains no complete captured Feature request/response packet. The harness does not fabricate a captured packet fixture. GET short-transfer cases are synthetic count/data combinations modeling the statically recovered buffer semantics.

## Verified facts covered by tests

### macOS Feature SET

- The recovered checksum branches apply to metadata selector `+0x3a == 2` only.
- Append the caller message without a driver-added prefix.
- Logical buffer length becomes `max(message_length, 66)` with zero fill; longer messages are not truncated by `BytesMut::resize`.
- `dangledevtype=0`: complemented wrapping 8-bit sum of offsets 1 through 7, stored at offset 7.
- `dangledevtype=1`: complemented wrapping 8-bit sum of offsets 1 through 8, stored at offset 8.
- `dangledevtype=2`: no checksum insertion.
- The pre-store destination byte participates in the checksum when present in the original message; otherwise it is zero from resize.
- The low-level transfer image is exactly the first 64 bytes. Report ID 0 is implicit; offset 0 is caller data, not an inserted ID.

### Windows Feature SET

- Prefix the complete dynamic payload with literal `0x01`; its Report ID meaning remains unproven.
- Logical Vec length is `max(65, payload_length + 1)`; longer payloads are retained.
- Selector 0 stores the complemented wrapping 8-bit sum of the complete source payload at vector offset 8.
- Selector 1 stores the same complete-payload checksum at vector offset 9.
- Checksum destinations overwrite the corresponding existing payload byte when that byte exists. Tests account for this rather than incorrectly expecting the entire appended payload to remain unchanged.
- The Windows vector length and checksum input domain are distinct from the macOS 64-byte transfer and fixed checksum windows. No equivalence is inferred.

### macOS Feature GET response

- The simulated receive buffer starts as 64 zero bytes.
- For a non-negative short transfer, only the returned prefix is overwritten; the logical response remains 64 bytes and the untouched suffix remains zero.
- Negative transfer results become an error. Impossible synthetic counts (larger than the buffer or supplied bytes) are rejected by the simulator.
- No command-specific response parser is run.

## Test coverage

The integration suite `crates/th80-probe/tests/offline_protocol.rs` covers:

| Category | Cases |
|---|---|
| Captured descriptor fixture | Exact captured bytes, SHA-256, Feature type, length, implicit ID |
| macOS packet construction | No prefix, 64-byte transfer image, 66-byte logical minimum, longer-message retention |
| macOS checksum | All three `dangledevtype` branches, exact windows/destinations, pre-store destination inclusion, zero-filled destination |
| macOS lengths | Payload sizes 0, 1, 6, 7, 8, 9, 63, 64, 65, 66, 67, 127 and 128 |
| Windows packet construction | Prefix, minimum 65-byte Vec, long payload retention, checksum overwrite positions |
| Windows lengths | Payload sizes 0, 1, 7, 8, 9, 63, 64, 65, 66 and 127 |
| Platform separation | Complete-payload Windows checksum differs from macOS fixed-window checksum; transfer lengths differ |
| GET short transfer | Return counts 0, 1, 7, 8, 31, 63 and 64; fixed length and zero suffix |
| Malformed inputs | Truncated descriptors, malformed long items, non-64-byte envelopes, negative/impossible GET counts |
| Unknown semantics | Arbitrary first bytes remain `CommandSemantics::Unknown`; no command enum or parser is inferred |
| Disabled transport | Enumeration, metadata, interfaces, Feature GET/SET, Input and Output all return `TransportDisabled` |

Existing Phase 5B parser unit tests and Phase 6 error-conversion tests continue to run. The suite contains 12 new integration tests.

## Remaining unknowns and assumptions

- No command IDs, command names, payload schema, status fields, sequence rules or response grammar are established. `command_semantics` therefore always returns `Unknown`.
- The Windows `0x01` prefix is literal recovered buffer data; its Report ID meaning is not established for the target descriptor.
- The Windows generic Feature caller's exact association with the V504 firmware-update flow remains unproven.
- Windows selector branches are represented only as recovered numeric values 0 and 1; their broader product meaning is not inferred.
- The macOS checksum logic is represented only for metadata selector 2. No behavior is inferred for other metadata selectors.
- The checksum algorithms are reproduced as observed, not endorsed as protocol validation or proof that any synthetic packet is accepted by firmware.
- macOS and Windows builders are intentionally separate. Their checksum input domains and transfer lengths differ.
- Short-transfer tests are simulations of the recovered buffer logic, not observations of a live device response.

## Transport isolation and safety

The integration suite imports only `DisabledTransport`. Each exposed operation is asserted to return `DriverError::TransportDisabled`. Packet construction and GET simulation have no transport parameter, device handle, file descriptor, USB/HID library call or operating-system I/O path.

No USB control transfer, HID Feature GET/SET, interrupt transfer, Output report, interface claim/detach, USB reset, configuration change, firmware operation or bootloader operation was performed. No device node was opened. The captured descriptor fixture is a static file copied from prior passive evidence.

## Validation results

Final verification for Phase 7A:

- `cargo fmt --all -- --check` — passed.
- `cargo test --locked` — 47 passed, 0 failed (26 library, 6 probe CLI, 12 offline integration, 3 workspace CLI tests).
- `cargo clippy --locked --all-targets -- -D warnings` — passed.
- `cargo build --locked --release -p th80-probe` — passed.
- Existing CLI smoke tests (`th80-probe list`, `th80-probe capabilities --all`) — passed using the existing passive sysfs diagnostic path; no HID/USB device node was opened.
- Markdown relative-link validation — 46 links checked, 0 broken.

**Phase 7A stops here.** No active protocol validation or hardware transport activation is authorized or performed.
