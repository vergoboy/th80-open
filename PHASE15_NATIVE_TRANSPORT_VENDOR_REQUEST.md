# Phase 15 — Minimal native host-driver information request

**Status: prepared for review only. Do not send without explicit user authorization.**

The existing `PHASE13_12_VENDOR_SDK_REQUEST_DOSSIER.md` requests the original TH80 ANSI V504 firmware SDK/build package. This separate request asks only for the missing host-driver material needed to close the normal configuration transport path.

## Requested material

For the exact V504 driver release represented by these local artifacts:

- macOS `iot_driver` SHA-256: `22c5dc67ba77e2373a5ffdbb0312ed8c8e73eb16e78037764eb676a030b2724b`
- macOS `iot_driver` Mach-O UUID: `E66C5739-F4BF-3DB7-AA10-01600946BF11`
- macOS binary: x86-64 Mach-O, `LC_BUILD_VERSION` platform `macos`, SDK 13.0, minimum OS 12.0; `LC_SOURCE_VERSION` 0.0; linker `ld` 820.1
- macOS bundle: `com.rongyuan.driverElectron`, EPOMAKER Driver `1.0.0`
- frontend `dist/index.js` SHA-256: `7d53737095fe5756fb0a18d5aecb7593555b9bb61a9d5d160ae0b988694c1e87`
- Windows updater PE SHA-256: `e9f3d8d2f3160be5e3817fb4eca8efe954cc854426351e3d0d21e73f39fce0fc`

The macOS Mach-O has an `LC_SYMTAB` (22,512 entries; 20,333 local, 1,874 externally defined, 305 undefined) but no DWARF/debug sections or companion dSYM in the supplied package. Its embedded build paths identify the source workspace as `/Users/jiandong/Documents/dj_hid_sdk_rs` and include `src/grpc_server/mod.rs`, `src/dj_dev_api/mod.rs`, `src/dj_dev_api/cmd_list.rs`, `src/dj_hid_device/mod.rs`, `src/dj_hid_device/dj_hid/mod.rs`, `src/dj_hid_device/dj_hid/hid_ble.rs`, and `src/dj_hid_device/dj_hid/hid_libusb.rs`. The generated protobuf Rust file was built under `target/release/build/iot_driver-bf78ff4bc4bf14df/out/driver.rs`.

The local Windows PE is the separate firmware-upgrade application, not a verified matching Windows `iot_driver.exe`. Its CodeView record points to `C:\Users\Windows\Documents\rust-hid\target\release\deps\rust_hid.pdb`, GUID `C215AEE3-87E2-43DB-B791-B1A9DD29DCEB`, age `55`, PE timestamp `2021-10-23 03:00:25`. That PDB identity is not the macOS `iot_driver` identity and must not be used as a substitute for its symbols.

Please provide, in order of preference:

1. The complete matching Rust crate at the build represented by the macOS UUID/hash above, or its exact dSYM/DWARF companion. In particular, include the handler and concrete device API implementations in `src/grpc_server/`, `src/dj_dev_api/`, and `src/dj_hid_device/`, plus `Cargo.toml`, `Cargo.lock`, the Rust toolchain/build metadata, and the generated `out/driver.rs` or exact source `driver.proto`. If only a symbol-rich binary can be supplied, include its matching UUID/build ID and the exact companion debug file identifier.
2. The implementation or API contract for ordinary `sendMsg` / `readMsg`, including which device methods they invoke and how `check_sum_type` and `dangle_dev_type` are forwarded or transformed.
3. The documented relationship between the physical HID interface number 2 and the Feature control-transfer `wIndex=0` used by the macOS libusb backend.
4. The exact Windows `iot_driver.exe` shipped with this V504 host-driver release, together with its matching PDB/CodeView identity or source. Also provide the Windows normal-configuration source/call path and identify whether the generic `HidD_SetFeature` future found in the separate firmware-upgrade executable is used for configuration, firmware update, or both.
5. The expected `ResSend.err` / `ResRead.msg` semantics, including acknowledgement, response-length, short-transfer and timeout behavior for the TH80 keyboard PID `3151:4010` and receiver PID `3151:4011`.

If source cannot be shared, a short non-confidential protocol/transport specification answering items 2–5 is sufficient. No firmware source, signing secrets, bootloader material, or security bypass is requested by this host-driver inquiry.

## Why these exact items are needed

Static analysis confirms the frontend's unary protobuf contract and identifies the native Tonic handler futures, but the final ordinary handler-to-device call is indirect through stripped Rust trait-object vtables. Separately, the macOS low-level `HidLibusb` Feature method is recovered and hardcodes `wIndex=0`, despite the vendor support metadata retaining interface 2. The Windows updater PE exposes a generic HID Feature path but does not establish its relationship to normal configuration.

These items are the minimum evidence needed to implement an interoperable native adapter without guessing. No device communication will be performed to infer missing behavior. The supplied updater's `rust_hid.pdb` and public reverse-engineering notes for other Rongyuan driver releases are explicitly not accepted as matching debug/source material for this binary.
