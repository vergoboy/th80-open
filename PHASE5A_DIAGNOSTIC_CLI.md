# Phase 5A — TH80 Diagnostic CLI v0.1

## Status and scope

`th80-probe` is the first standalone diagnostic executable in this workspace. It passively discovers these USB devices:

| Role | VID:PID |
|---|---|
| EPOMAKER TH80 Pro | `3151:4010` |
| ROYUAN 2.4 GHz receiver | `3151:4011` |

Version 0.1 reports USB identity, optional manufacturer/product/serial metadata, the current USB device node when available, HID interfaces, interrupt endpoints and HID report descriptors where the Linux sysfs `hidraw` descriptor file is readable.

**Platform:** Linux with the standard USB sysfs and HID sysfs interfaces mounted. This implementation deliberately does not use libusb, HIDAPI, or open a USB/HID device handle. It reads text attributes under `/sys/bus/usb/devices` and report descriptor files under `/sys/class/hidraw` only.

## Architecture

The standalone crate is `crates/th80-probe/` and has no dependency on the existing `th80ctl` research executable.

| Responsibility | Implementation |
|---|---|
| Device discovery | Scan `/sys/bus/usb/devices` for the exact target VID/PID pairs |
| Device identification | `DeviceKind::from_vid_pid`, with keyboard/receiver classification |
| Stable identity | VID:PID plus the USB topology name (for example `3151:4010@1-9`) |
| USB path | Derive `/dev/bus/usb/BBB/DDD` from sysfs bus/device numbers when that node exists |
| Interface inspection | Read interface number, class, subclass, protocol and bound driver from interface sysfs entries |
| Endpoint inspection | Read endpoint address, direction, transfer type, maximum packet size and interval |
| Report descriptor | Match the interface's canonical sysfs path to `/sys/class/hidraw/*/device`, then read its `report_descriptor` file |
| JSON | Serialize a versioned `DiagnosticSnapshot` using the workspace's existing `serde` and `serde_json` dependencies |
| CLI | Small standard-library argument parser; no additional CLI framework |

USB descriptor attributes such as `idVendor`, `idProduct`, endpoint address and maximum packet size are hexadecimal strings in sysfs. The implementation parses those as hexadecimal; decimal-valued attributes such as endpoint interval are parsed as decimal.

Optional metadata is represented as JSON `null` and displayed as unavailable rather than causing device inspection to fail. Per-interface warnings record unavailable descriptors or attributes. Failure to enumerate the sysfs root is a command error.

## Build and installation

From the workspace root:

```sh
cargo build --locked -p th80-probe --release
```

Run directly from the workspace:

```sh
cargo run --locked -p th80-probe -- list
cargo run --locked -p th80-probe -- inspect --device keyboard
cargo run --locked -p th80-probe -- inspect --device receiver
cargo run --locked -p th80-probe -- inspect --all
cargo run --locked -p th80-probe -- inspect --json
cargo run --locked -p th80-probe -- inspect --device keyboard --json
```

Install the binary into Cargo's configured install directory:

```sh
cargo install --path crates/th80-probe --locked
```

Then invoke `th80-probe` directly. Access to sysfs descriptor files is normally read-only; access to USB metadata is subject to the host's sysfs mount and permissions. No elevated privileges should be needed on a normally configured Linux system.

## CLI usage

```text
th80-probe list [--json]
th80-probe inspect --device keyboard|receiver [--json]
th80-probe inspect --all [--json]
th80-probe inspect --json
```

- `list` shows the connected target devices, identity and interface/endpoint summary.
- `inspect --device keyboard` inspects all currently connected TH80 Pro devices.
- `inspect --device receiver` inspects all currently connected receiver devices.
- `inspect --all` prints full details for both target classes.
- `inspect --json` prints one JSON snapshot containing all connected targets. Add `--device` to filter it.
- A disconnected target is reported as not connected; it is not treated as a fatal error.

The JSON top-level object contains `schema_version`, `tool` and `devices`. Each device has a topology-based `stable_id`, role, VID/PID, optional USB metadata, interfaces, endpoints, report descriptor bytes in hex when readable, and warnings.

## Illustrative output

The following is an **illustrative format example**, not a claim about a currently attached device. Bus/device numbers, USB topology and metadata vary by host and connection:

```text
EPOMAKER TH80 Pro [3151:4010@1-9]
  USB 3151:4010
  Manufacturer: ROYUAN
  Product: EPOMAKER TH80 Pro
  Serial: (not exposed)
  USB release: 0505
  Speed: 12 Mbps
  USB device path: /dev/bus/usb/001/018
  Interfaces: 3
    Interface 0: class=0x03 subclass=0x01 protocol=0x01 driver=usbhid
      Endpoint 0x81 in interrupt max-packet=8 interval=1
    Interface 1: class=0x03 subclass=0x00 protocol=0x00 driver=usbhid
      Endpoint 0x82 in interrupt max-packet=16 interval=1
    Interface 2: class=0x03 subclass=0x00 protocol=0x00 driver=usbhid
      Endpoint 0x83 in interrupt max-packet=8 interval=1
```

## Safety guarantees

The program only performs filesystem reads of USB/HID sysfs metadata and the kernel-exposed HID report descriptor files. It does not:

- open `/dev/bus/usb/*` or `/dev/hidraw*`;
- issue USB control transfers, including HID Feature GET/SET;
- send HID Output or interrupt reports;
- claim or detach kernel interfaces;
- reset, reconfigure, suspend, or otherwise alter USB devices;
- send vendor or undocumented commands;
- enter bootloader mode, invoke an updater, or write firmware.

The vendor Feature report's 64-byte EP0 payload is not inferred from the vendor interrupt endpoint. The tool reports the interface-2 interrupt-IN endpoint as an endpoint descriptor only; it does not read from it. Report descriptor retrieval is a read of the kernel's cached/exposed descriptor file, not a HID Feature request.

## Tests

Hardware-independent unit tests cover:

- target VID/PID identification and keyboard/receiver classification;
- interface and endpoint model representation;
- endpoint transfer-type representation;
- JSON snapshot serialization;
- missing optional USB metadata;
- hexadecimal and decimal sysfs numeric parsing.

The tests do not require a connected keyboard or receiver.

## Known limitations

- Linux sysfs implementation only; macOS and Windows enumeration are not implemented.
- USB device path `/dev/bus/usb/BBB/DDD` is ephemeral and may change after reconnect. `stable_id` uses the USB topology path and can change when moved to another port or hub.
- Manufacturer/product strings and serial numbers are optional USB descriptor strings. A device may omit them or the kernel may not expose them.
- HID report descriptors are included only when a matching `hidraw` sysfs descriptor file is readable. Permission restrictions, unusual driver layouts, or races during disconnect can leave them unavailable.
- Endpoint metadata is the kernel's sysfs view of the active configuration; this CLI does not independently parse raw configuration descriptor bytes.
- No HID report parser, Feature protocol implementation, input capture, configuration operation or firmware operation is included in this crate.
- The tool does not claim that the original vendor driver's generic Feature transfer path is correct for either device. In particular, the previously reverse-engineered `wIndex=0` behavior is not reproduced here.

## Research boundary

This crate is an independent diagnostic utility, not an implementation of the experimental command protocol. It does not modify the existing `th80ctl` research source or any firmware/configuration artifact. Further protocol work must remain separate and must be reviewed against the repository's safety policy before adding any operation that communicates with a device.
