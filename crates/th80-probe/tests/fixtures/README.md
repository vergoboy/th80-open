# Offline protocol fixture provenance

- `captured_interface2_descriptor.hex` is an exact copy of the 20-byte interface-2 HID report descriptor captured passively from Linux sysfs and published in `docs/hid-descriptors.md`. Its SHA-256 is `350fea62e225e3d92e9db3bbbd408fcebe386c10405454be661d3d2c3229b0b5`. This is a captured **descriptor**, not a Feature packet or USB traffic capture.
- Packet payloads in `offline_protocol.rs` are synthetic byte patterns. Their expected offsets, transfer lengths, checksum windows and prefix behavior are derived from `PHASE5D_FEATURE_BUFFER_LAYOUT.md` and `PHASE5D_WINDOWS_UPDATER_BUFFER.md`. They are not captured packets and do not imply command semantics.
- The GET short-transfer cases are synthetic return-count/data combinations that model the statically recovered zero-initialized 64-byte buffer and non-negative short-transfer handling. No transfer is issued.

No complete captured Feature request/response packet fixture is present in the supplied Phase 5 evidence. The harness intentionally does not fabricate one.
