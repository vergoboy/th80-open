# Phase 14 — Hardware Integration Readiness

## Current device-support boundary

TH80 Studio is an offline configurator. Its normal dependency graph is:

    th80-gui -> th80-core

It does not depend on th80-probe, the root th80-hid-research package, HIDAPI, Vial, QMK, RMK or a USB/HID transport. th80-core contains profile/domain logic, local validation and file persistence only.

No code in this phase enumerates a device, opens a USB/HID handle, sends a report, reads live firmware state, changes keyboard settings or applies lighting.

## Evidence still missing from Phase 13

The Phase 13 investigation established:

- Product family: EPOMAKER TH80 / TH80 Pro, with the V504 distribution archive labelled TH80 ANSI Firmware V504 (ID 742).
- Normal updater support entry 3151:4010 and boot-mode entry 3151:4001; these are static updater configuration entries, not a live device observation.
- A receiver/dongle identity 3151:4011 exists in the broader native driver evidence, but its exact association to a particular keyboard/receiver revision remains distinct from the keyboard identity.
- Runtime frontend device type ID is obtained dynamically and has not been proven equal to archive directory ID 742.
- Yichip YC3121-family SDK lineage is strongly supported, but the exact physical MCU part/variant and board revision are not established.
- V504 raw image contains candidate boot-stage and application-side structures, but bootloader image acceptance, physical memory map, authentication/integrity policy and recovery procedure remain unknown.
- Candidate timer/RGB control flow exists in static firmware analysis, but exact MCU interrupt acknowledgement and electrical LED output semantics are not closed.
- The exact production V504 SDK/build package has not been recovered.

These facts are not sufficient to implement or authorize a hardware transport.

## Required evidence before device support

### 1. Identity and compatibility

- Exact product SKU/layout and PCB revision.
- Exact MCU manufacturer, part number, variant and stepping.
- Keyboard versus receiver identity and interface mapping.
- Runtime vendor device-type ID and its authoritative product mapping.
- Firmware version read semantics and board/firmware compatibility matrix.

### 2. Firmware and boot path

- Original production SDK or complete register/startup/ROM references.
- Exact instruction set, ABI, clock and memory map.
- Immutable boot ROM and field bootloader responsibilities.
- Update image/container layout, target address, bank selection and size limits.
- Checksum/integrity/signature/encryption/authentication rules.
- Anti-rollback behavior and a documented recovery path.

Phase 14 does not implement replacement firmware or flash operations.

### 3. Transport and protocol

- Exact interface and report descriptors for each supported device mode.
- Verified command framing, selector/report ID, lengths, checksums and response semantics.
- Evidence that each operation is valid for the exact model/firmware revision.
- Explicit timeout, retry, disconnect, partial-transfer and malformed-response behavior.
- Read/write acknowledgement semantics and read-back verification.
- Idempotency and persistence behavior for each write.
- Safe behavior when a profile contains unsupported or unknown fields.

VID/PID recognition alone is not protocol compatibility.

## Future integration architecture

These are conceptual responsibility boundaries, not interfaces added prematurely in Phase 14.

1. **Profile domain** — schema, logical layers/actions/macros/lighting/options, validation and serialization. Remains hardware-independent.
2. **Application state** — selected profile, dirty state, conflict resolution, explicit user intent and authorization prompts.
3. **Device compatibility model** — verified identity, firmware/board revision, negotiated capabilities and evidence-backed constraints. Unknown values must remain unknown.
4. **Transport interface** — device discovery/connection and bounded byte/report I/O only after a real consumer and exact transport contract exist. It must not be linked into the normal GUI profile workflow by default.
5. **Hardware-specific protocol implementation** — device-family command encoding, decoding, validation, state synchronization and operation lifecycle. Must be separately gated by evidence and tests.

No unused transport abstraction was added because there is no implemented hardware operation that currently needs one.

## Future operation lifecycle requirements

Before any future write operation:

1. Discover a candidate device without claiming support from VID/PID alone.
2. Establish exact identity, board revision and firmware version.
3. Negotiate only documented capabilities; do not infer them from UI controls.
4. Check profile compatibility against evidence-backed constraints.
5. Read and preserve the current device state where supported.
6. Present a user-visible operation plan and require explicit authorization.
7. Send only documented operations with bounded timeouts and precise response checks.
8. Read back/verify resulting state where the protocol supports it.
9. Keep local profile state, device-reported state and device-applied state distinct.
10. Report partial failure and recovery options without silently marking the profile as applied.

Transport errors, disconnects, short reads/writes, invalid acknowledgements, unsupported firmware, mismatched hardware and partial application must be represented explicitly.

## Features that can continue independently

The following work does not require hardware integration:

- Offline profile CRUD, duplication, import/export and validation.
- Logical keymap/layer editing and portability.
- Macro editing and ordering as local profile data.
- Generic lighting profile preferences and explicit optional values.
- Application-owned JSON options and top-level extension preservation.
- CLI and GUI consistency, accessibility, packaging and documentation.
- Schema migration design and synthetic fixture compatibility testing.

## Readiness conclusion

**Offline configurator: ready for continued product development.**

**Physical-device integration: not ready for implementation.**

The exact MCU/board identity, firmware compatibility contract, transport protocol and operation safety/recovery model remain unresolved. The next hardware-related investment should be evidence acquisition and documentation, not a generic HID abstraction or guessed command implementation.

See [Phase 13.12 hardware/bootloader assessment](PHASE13_12_HARDWARE_IDENTITY_AND_BOOTLOADER_FEASIBILITY.md), [Phase 13.12 strategy decision](PHASE13_12_INDEPENDENT_FIRMWARE_STRATEGY_ASSESSMENT.md) and [Phase 13.12 findings](PHASE13_12_FINDINGS_AND_NEXT_STEPS.md).

See also the [Phase 14 application audit](PHASE14_APPLICATION_AUDIT.md), [implementation report](PHASE14_IMPLEMENTATION_REPORT.md) and [test/platform report](PHASE14_TEST_AND_PLATFORM_REPORT.md).
