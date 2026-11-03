# Phase 5D — macOS HIDAPI / IOKit Feature transport audit

Date: 2026-11-01  
Scope: exhaustive static audit of HIDAPI and IOKit report paths in the exact bundled `iot_driver` Mach-O. No application execution or device I/O.

## Artifact identity

- Path: `../.phase3-work/mac/EPOMAKER Driver 1.0.0/EPOMAKER Driver.app/Contents/Resources/app/iot_driver`
- SHA-256: `22c5dc67ba77e2373a5ffdbb0312ed8c8e73eb16e78037764eb676a030b2724b`
- Format: Mach-O x86-64; image base `0x100000000`.

## Executive result

- **CONFIRMED:** The binary contains one HIDAPI Output-report write implementation, `_hid_write` → `IOHIDDeviceSetReport` with report type `1`.
- **CONFIRMED:** HIDAPI input is received through `IOHIDDeviceRegisterInputReportCallback`, queued by `_hid_report_callback`, and returned through `_hid_read_timeout` / `HidDevice::read_timeout`.
- **CONFIRMED:** No HIDAPI Feature GET or Feature SET implementation was identified. `IOHIDDeviceGetReport` is not imported; the only `IOHIDDeviceSetReport` call site passes report type `1`, not Feature type `2`.
- **CONFIRMED:** `HidBLE::send_feature` is an application-trait method name, not a wire-level Feature operation: it prepends `0x06` and calls HIDAPI Output write. `HidBLE::get_feature` reads from the HIDAPI input queue.
- **CONFIRMED:** The target keyboard and receiver select `HidLibusb`, not `HidBLE`, through the already recovered metadata selector branch (`DJDevice::new`, `+0x3A == 2`). Their native Feature GET/SET path is the separate libusb implementation.

## HIDAPI wrapper and C-backend map

| Function | VA | Call / behavior | Grade |
|---|---:|---|---|
| `HidApi::new` | `0x1001be470` | Calls `_hid_init`, enumerates device information | **CONFIRMED** |
| `HidApi::refresh_devices` | `0x1001be660` | Refreshes enumeration | **CONFIRMED** |
| `HidApi::device_list` | `0x1001beb70` | Exposes enumerated device list | **CONFIRMED** |
| `HidApi::open_path` | `0x1001beb90` | Calls `_hid_open_path` at `0x1001beba4` → `0x1001c1b70` | **CONFIRMED** |
| `HidDevice::write` | `0x1001bf190` | Calls `_hid_write` at `0x1001bf1af` → `0x1001c2290` | **CONFIRMED** |
| `HidDevice::read_timeout` | `0x1001bf2a0` | Calls `_hid_read_timeout` at `0x1001bf2bd` → `0x1001c22f0` | **CONFIRMED** |
| `_hid_report_callback` | `0x1001c1f20` | Queues callback-provided input report bytes | **CONFIRMED** |
| `_hid_write` | `0x1001c2290` | Calls IOKit `IOHIDDeviceSetReport` at `0x1001c22d4` | **CONFIRMED** |
| `_hid_read_timeout` | `0x1001c22f0` | Drains queued input reports; waits/returns according to timeout | **CONFIRMED** |

The HIDAPI Rust wrappers are not Feature-report wrappers: this binary contains only `write` and `read_timeout` on `HidDevice`; no Rust `send_feature_report` / `get_feature_report` method was found in the bundled crate.

## IOKit imports and call-site audit

The Mach-O imports these report-related IOKit APIs:

- `IOHIDDeviceRegisterInputReportCallback`
- `IOHIDDeviceSetReport`

It does not import `IOHIDDeviceGetReport`. It also imports device lifecycle/enumeration APIs (`IOHIDDeviceCreate`, `IOHIDDeviceOpen`, `IOHIDDeviceClose`, manager matching/scheduling, removal callback, property/service accessors).

The sole call site to the IOKit `IOHIDDeviceSetReport` stub is `_hid_write` at `0x1001c22d4`. Immediately before the call, `0x1001c22cf` loads `esi = 1`; this is the IOKit report type argument (Output). The function normalizes a leading zero report ID in its input buffer by advancing the data pointer and reducing the length; it does not change the report type to Feature.

`_hid_read_timeout` drains the linked queue populated by `_hid_report_callback`; the callback is registered through `IOHIDDeviceRegisterInputReportCallback`. This is input-report delivery, not `IOHIDDeviceGetReport` Feature polling.

**CONFIRMED:** The inspected binary has no IOKit Feature GET call and no Feature-type IOKit SET call. No indirect function-pointer route to `IOHIDDeviceGetReport` is present because that symbol is absent from the import table; the one `SetReport` call site is statically typed Output.

## Application-level `HidBLE` methods

- `HidBLE::new`: `0x100035fc0`; opens a HIDAPI path via `HidApi::open_path` at `0x100036056`.
- `HidBLE::send_feature`: `0x100036ab0`; builds a `BytesMut`, appends byte `0x06` at `0x100036aea`, appends caller bytes at `0x100036b0a`, then calls `HidDevice::write` at `0x100036b6c`.
- `HidBLE::get_feature`: `0x100036d30`; repeatedly checks/drains the shared input-report queue, with a 10 ms sleep (`0x989680` ns) on the polling loop; it does not call a Feature GET API.

**CONFIRMED:** `send_feature` is a trait/API label only. Its concrete HIDAPI report type is Output (1). `get_feature` is an input-queue read. These methods must not be counted as USB/HID Feature GET/SET operations.

## Device branch and call sites

`DJDevice::new` at `0x1000c8ea0` checks metadata byte `+0x3A`:

- equal to `2`: calls `HidLibusb::new` at `0x100047d30`;
- otherwise: calls `HidBLE::new` at `0x100035fc0` (`0x1000c8f6b`).

The direct `HidBLE::new` call site is this non-`2` branch. Existing support-record analysis identifies `3151:4010` and `3151:4011` with selector value `2`; the corresponding device wrappers are `BleHid` and `YZWDangleCommon`, respectively, but both wrap the `DJDevice` whose underlying sender is `HidLibusb`.

**CONFIRMED:** For the two target physical identities, the HIDAPI `HidBLE` implementation is not the selected Feature transport. Its presence does not provide an alternate path around libusb's `wIndex` value.

## Exhaustiveness boundary

The audited binary contains no HIDAPI Feature wrapper symbol or call, no `IOHIDDeviceGetReport` import, and exactly one IOKit `SetReport` call site with report type Output. This supports the conclusion that **no HIDAPI/IOKit Feature transport exists in this supplied binary**. It does not prove that every vendor frontend operation reaches the libusb methods; the unresolved raw RPC route mapping is documented separately in `PHASE5D_RPC_DISPATCH_AUDIT.md`.

## Evidence classification summary

- **CONFIRMED:** HIDAPI write is Output; HIDAPI read is input callback queue; no IOKit Feature GET/SET call was found; target device branch selects libusb.
- **UNKNOWN:** Whether any external helper process or a different vendor-driver build uses another transport. Such artifacts are outside this binary audit.

No HIDAPI device I/O, IOKit report call, USB transfer, or hardware access was performed.
