use serde::{Serialize, Serializer};
use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;

use th80_probe::capabilities as hid;
use th80_probe::device::{DeviceInstance, DeviceKind, DeviceMetadata};
use th80_probe::yc500::{
    encode_keyboard_options_write, encode_lighting_write, encode_set_active_profile,
    encode_th80_fn_slot_write, encode_th80_key_slot_write, encode_th80_macro_read_pages,
    encode_th80_macro_upload, encode_th80_v504_set_report_rate, encode_user_picture_upload,
    Th80V504Command, Yc500Command, FEATURE_REPORT_BYTES as YC500_REPORT_BYTES,
};
use th80_probe::{discovery::DiscoveryProvider, sysfs::SysfsDiscovery};

const USB_SYSFS_ROOT: &str = "/sys/bus/usb/devices";
const HIDRAW_SYSFS_ROOT: &str = "/sys/class/hidraw";

fn device_label(kind: DeviceKind) -> &'static str {
    match kind {
        DeviceKind::Th80Pro => "EPOMAKER TH80 Pro",
        DeviceKind::RoyuanReceiver => "ROYUAN 2.4 GHz receiver",
    }
}

fn serialize_cli_device_kind<S>(kind: &DeviceKind, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(match kind {
        DeviceKind::Th80Pro => "keyboard",
        DeviceKind::RoyuanReceiver => "receiver",
    })
}

#[derive(Debug, Clone, Serialize)]
struct EndpointInfo {
    address: Option<u8>,
    direction: Option<String>,
    transfer_type: Option<String>,
    max_packet_size: Option<u16>,
    interval: Option<u8>,
    sysfs_path: String,
    warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct InterfaceInfo {
    number: Option<u8>,
    class: Option<u8>,
    subclass: Option<u8>,
    protocol: Option<u8>,
    driver: Option<String>,
    endpoints: Vec<EndpointInfo>,
    report_descriptor: Option<ReportDescriptor>,
    warnings: Vec<String>,
    sysfs_path: String,
    #[serde(skip)]
    capabilities: Option<hid::CapabilityProfile>,
}

#[derive(Debug, Clone, Serialize)]
struct ReportDescriptor {
    length: usize,
    hex: String,
    source: String,
}

#[derive(Debug, Clone, Serialize)]
struct UsbDevice {
    stable_id: String,
    #[serde(serialize_with = "serialize_cli_device_kind")]
    kind: DeviceKind,
    label: String,
    vendor_id: u16,
    product_id: u16,
    manufacturer: Option<String>,
    product: Option<String>,
    serial_number: Option<String>,
    usb_release: Option<String>,
    speed_mbps: Option<String>,
    bus_number: Option<u8>,
    device_number: Option<u8>,
    usb_device_path: Option<String>,
    sysfs_path: String,
    interfaces: Vec<InterfaceInfo>,
    warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct DiagnosticSnapshot {
    schema_version: u8,
    tool: &'static str,
    devices: Vec<UsbDevice>,
}

#[derive(Debug, Clone, Serialize)]
struct CapabilityInterface {
    interface_number: Option<u8>,
    interface_class: Option<u8>,
    interface_subclass: Option<u8>,
    interface_protocol: Option<u8>,
    descriptor_source: Option<String>,
    descriptor_length: Option<usize>,
    report_descriptor_sha256: Option<String>,
    uses_report_ids: Option<bool>,
    reports: Vec<hid::ReportDefinition>,
    long_items: Vec<hid::LongItem>,
    input_report_count: usize,
    output_report_count: usize,
    feature_report_count: usize,
    parser_warnings: Vec<hid::ParseDiagnostic>,
    parser_errors: Vec<String>,
    endpoints: Vec<EndpointInfo>,
    warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct DeviceCapabilities {
    stable_id: String,
    #[serde(serialize_with = "serialize_cli_device_kind")]
    kind: DeviceKind,
    label: String,
    vendor_id: u16,
    product_id: u16,
    manufacturer: Option<String>,
    product: Option<String>,
    serial_number: Option<String>,
    interfaces: Vec<CapabilityInterface>,
    warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct CapabilitySnapshot {
    schema_version: u8,
    tool: &'static str,
    mode: &'static str,
    devices: Vec<DeviceCapabilities>,
}

#[derive(Debug, Clone, Serialize)]
struct ProtocolCommandInfo {
    candidate: &'static str,
    operation: &'static str,
    opcode: String,
    request: &'static str,
    response: &'static str,
    evidence: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct UnsupportedProtocolCommand {
    candidate: &'static str,
    reason: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct ProtocolSnapshot {
    schema_version: u8,
    tool: &'static str,
    mode: &'static str,
    vendor_id: u16,
    product_id: u16,
    family: &'static str,
    feature_report_bytes: usize,
    report_id: u8,
    commands: Vec<ProtocolCommandInfo>,
    unsupported: Vec<UnsupportedProtocolCommand>,
}

fn protocol_snapshot() -> Result<ProtocolSnapshot, Box<dyn Error>> {
    let command = |candidate, operation, opcode, request, response, evidence| ProtocolCommandInfo {
        candidate,
        operation,
        opcode: format!("0x{opcode:02x}"),
        request,
        response,
        evidence,
    };
    let commands = vec![
        command(
            "C01",
            "firmware revision",
            Yc500Command::FirmwareRevision.encode()?.message[0],
            "64-byte Feature SET query; CheckSumType=BIT7; DangleDevType=NONE",
            "64-byte response; bytes 1..2 are LE16; no opcode echo required by frontend parser",
            "strongly correlated yc500 family contract",
        ),
        command(
            "C05",
            "active profile GET",
            Yc500Command::ActiveProfile.encode()?.message[0],
            "64-byte Feature SET query; CheckSumType=BIT7; DangleDevType=NONE",
            "64-byte response; byte 1 is zero-based profile; no opcode echo required by frontend parser",
            "direct V504 frontend method; four profiles in TH80 registry record",
        ),
        command(
            "C05",
            "active profile SET",
            encode_set_active_profile(0)?.message[0],
            "64-byte Feature SET request; CheckSumType=BIT7; DangleDevType=NONE",
            "acknowledgement format not established",
            "direct V504 frontend method; no acknowledgement layout established",
        ),
        command(
            "C06",
            "report rate GET",
            Th80V504Command::ReportRate { profile: 0 }.encode()?.message[0],
            "64-byte Feature query; profile byte 1; CheckSumType=BIT7; DangleDevType=NONE",
            "64-byte frontend response; byte 2 code 1/2/4/8 maps to 1000/500/250/125 Hz",
            "direct vendor frontend method; conflicts with generic family analysis",
        ),
        command(
            "C06",
            "report rate SET",
            encode_th80_v504_set_report_rate(0, 1000)?.message[0],
            "64-byte Feature SET; bytes 1/2 are profile/rate code; CheckSumType=BIT7; DangleDevType=NONE",
            "no acknowledgement contract exposed by frontend",
            "direct vendor frontend method; no hardware confirmation",
        ),
        command(
            "C07",
            "keyboard options GET",
            Yc500Command::KeyboardOptions { profile: 0 }.encode()?.message[0],
            "64-byte Feature SET query; profile byte 1; CheckSumType=BIT7; DangleDevType=NONE",
            "64-byte response; raw fields at bytes 2..4; no opcode echo required by frontend parser",
            "strongly correlated; flag byte kept opaque",
        ),
        command(
            "C07",
            "keyboard options SET",
            encode_keyboard_options_write(0, 0, 0)?.message[0],
            "64-byte Feature SET request; profile/flags/Fn bytes 1..3; CheckSumType=BIT7; DangleDevType=NONE",
            "acknowledgement format not established",
            "payload shape correlated; no flag reinterpretation",
        ),
        command(
            "C08",
            "lighting parameters GET",
            Yc500Command::LightingParameters.encode()?.message[0],
            "64-byte Feature SET query; CheckSumType=BIT7; DangleDevType=NONE",
            "64-byte response; mode/speed/brightness/packed flags at bytes 1..4; no opcode echo required by frontend parser",
            "strongly correlated; device-specific enums not inferred",
        ),
        command(
            "C08",
            "lighting parameters SET",
            encode_lighting_write(0, 0, 0, 0, [0; 3]).message[0],
            "64-byte Feature SET request; RGB bytes 5..7; CheckSumType=BIT8; DangleDevType=NONE",
            "acknowledgement format not established",
            "packet layout correlated; ranges not inferred",
        ),
        command(
            "C09",
            "user-picture upload",
            encode_user_picture_upload(&vec![0; 384])?[0].message[0],
            "seven 64-byte Feature SET pages; CheckSumType=BIT7; DangleDevType=NONE; 384 RGB bytes total",
            "GET_USERPIC read-back is not reliable",
            "direct TH80 V504 inherited frontend upload framing; persistent write not dispatched",
        ),
        command(
            "C10",
            "keymap page GET",
            Yc500Command::KeymapPage {
                profile: 0,
                page: 0,
            }
            .encode()?.message[0],
            "Feature SET page query, then Feature GET; CheckSumType=BIT7; DangleDevType=NONE",
            "raw 64-byte page; no opcode echo",
            "TH80 V504 keymap page layout; CheckSumType=BIT7",
        ),
        command(
            "C10",
            "single key-slot SET",
            encode_th80_key_slot_write(0, 0, [0; 4])?.message[0],
            "64-byte Feature SET; profile/slot at bytes 1/2, value at bytes 8..11; CheckSumType=BIT7; DangleDevType=NONE",
            "write result only; no response layout established",
            "direct YC500Common frontend method; CheckSumType=BIT7; no hardware confirmation",
        ),
        command(
            "C11",
            "single Fn-slot SET",
            encode_th80_fn_slot_write(0, 0, [0; 4])?.message[0],
            "64-byte Feature SET; layer/slot at bytes 1/2, value at bytes 8..11; CheckSumType=BIT7; DangleDevType=NONE",
            "write result only; no response layout established",
            "direct YC500Common frontend method; CheckSumType=BIT7; no Fn GET method found",
        ),
        command(
            "C12",
            "macro page GET",
            encode_th80_macro_read_pages(0)?[0].message[0],
            "four 64-byte Feature SET queries [0x88, slot, page], each followed by Feature GET; CheckSumType=BIT7; DangleDevType=NONE",
            "four raw 64-byte Feature responses; 256-byte aggregate",
            "direct inherited V504 getMacro path (0x88), not generic yc500 0x8b",
        ),
        command(
            "C12",
            "macro upload SET",
            encode_th80_macro_upload(0, &[1; 256])?[0].message[0],
            "up to five 64-byte Feature SET pages; CheckSumType=BIT7; DangleDevType=NONE; vendor nonzero-page count over 56-byte chunks",
            "no acknowledgement layout; final-page marker on last transmitted page",
            "direct YC500Common frontend method; CheckSumType=BIT7; no event grammar inferred",
        ),
    ];
    Ok(ProtocolSnapshot {
        schema_version: 1,
        tool: "th80-probe",
        mode: "offline_protocol_contracts",
        vendor_id: 0x3151,
        product_id: 0x4010,
        family: "yc500",
        feature_report_bytes: YC500_REPORT_BYTES,
        report_id: 0,
        commands,
        unsupported: vec![
            UnsupportedProtocolCommand {
                candidate: "C02",
                reason: "GET_INFOR opcode is declared and response parser is available, but no TH80 V504 request/checksum method was found",
            },
            UnsupportedProtocolCommand {
                candidate: "C03",
                reason: "the only located getBattery implementation belongs to Dk2017, outside the selected YC500_TH80 inheritance chain",
            },
            UnsupportedProtocolCommand {
                candidate: "C04",
                reason: "Status24/watchSystemInfo telemetry has no proven association with the TH80 HID command path",
            },
            UnsupportedProtocolCommand {
                candidate: "C09-read",
                reason: "GET_USERPIC read helper defaults to CheckSumType::NONE, but vendor read-back is not reliable",
            },
            UnsupportedProtocolCommand {
                candidate: "C10-bulk-write",
                reason: "bulk setter defaults to CheckSumType::NONE; 512-byte TH80 matrix does not reconcile with inherited nine 56-byte transfer pages",
            },
            UnsupportedProtocolCommand {
                candidate: "C11-read",
                reason: "TH80 V504 defines GET_FN_SIMPLE but no frontend read method",
            },
            UnsupportedProtocolCommand {
                candidate: "C12-events",
                reason: "macro event encoding grammar is unknown; only raw 256-byte framing is supported",
            },
        ],
    })
}

fn print_protocol(snapshot: &ProtocolSnapshot) {
    println!(
        "TH80 {:04x}:{:04x} family={} Feature={} bytes report-id={}",
        snapshot.vendor_id,
        snapshot.product_id,
        snapshot.family,
        snapshot.feature_report_bytes,
        snapshot.report_id
    );
    println!("Offline packet codecs only; no device requests are sent.");
    for item in &snapshot.commands {
        println!(
            "{} {:<24} opcode={} | {} | {}",
            item.candidate, item.operation, item.opcode, item.request, item.response
        );
    }
    for item in &snapshot.unsupported {
        println!("{} unsupported: {}", item.candidate, item.reason);
    }
}

fn parse_num<T>(value: &str) -> Option<T>
where
    T: TryFrom<u64>,
{
    let parsed = if let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        u64::from_str_radix(hex, 16).ok()?
    } else {
        value.parse::<u64>().ok()?
    };
    T::try_from(parsed).ok()
}

fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn read_device(
    instance: &DeviceInstance,
    discovery: &SysfsDiscovery,
    metadata: &DeviceMetadata,
    allow_system_device_node_check: bool,
) -> Result<UsbDevice, Box<dyn Error>> {
    let path = &instance.sysfs_path;
    let vendor_id = metadata.identity.vendor_id;
    let product_id = metadata.identity.product_id;
    let kind = metadata.identity.kind;
    let bus_number = discovery
        .read_optional_device_attribute(instance, "busnum")?
        .as_deref()
        .and_then(parse_num::<u8>);
    let device_number = discovery
        .read_optional_device_attribute(instance, "devnum")?
        .as_deref()
        .and_then(parse_num::<u8>);
    let usb_device_path = match (bus_number, device_number) {
        (Some(bus), Some(device)) if allow_system_device_node_check => {
            let candidate = format!("/dev/bus/usb/{bus:03}/{device:03}");
            Path::new(&candidate).exists().then_some(candidate)
        }
        _ => None,
    };
    let mut interfaces = Vec::new();
    let mut warnings = Vec::new();
    for discovered in discovery.inspect_interfaces(instance)? {
        let interface_path = &discovered.sysfs_path;
        let driver = fs::read_link(interface_path.join("driver"))
            .ok()
            .and_then(|value| {
                value
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
            });
        let report_descriptor =
            discovered
                .report_descriptor_bytes
                .as_ref()
                .map(|bytes| ReportDescriptor {
                    length: bytes.len(),
                    hex: hex(bytes),
                    source: discovered.report_descriptor_source.as_ref().map_or_else(
                        || interface_path.display().to_string(),
                        |path| path.display().to_string(),
                    ),
                });
        let interface_warnings = if discovered.class == 0x03 && report_descriptor.is_none() {
            vec!["HID report descriptor unavailable through readable hidraw sysfs entry".into()]
        } else {
            Vec::new()
        };
        let endpoints = discovered
            .endpoints
            .into_iter()
            .map(|endpoint| EndpointInfo {
                address: Some(endpoint.address),
                direction: Some(
                    if endpoint.address & 0x80 != 0 {
                        "in"
                    } else {
                        "out"
                    }
                    .into(),
                ),
                transfer_type: Some(
                    match endpoint.transfer_type {
                        th80_probe::device::TransferType::Control => "control",
                        th80_probe::device::TransferType::Isochronous => "isochronous",
                        th80_probe::device::TransferType::Bulk => "bulk",
                        th80_probe::device::TransferType::Interrupt => "interrupt",
                    }
                    .into(),
                ),
                max_packet_size: Some(endpoint.max_packet_size),
                interval: Some(endpoint.interval),
                sysfs_path: endpoint.sysfs_path.display().to_string(),
                warnings: Vec::new(),
            })
            .collect();
        interfaces.push(InterfaceInfo {
            number: Some(discovered.number),
            class: Some(discovered.class),
            subclass: Some(discovered.subclass),
            protocol: Some(discovered.protocol),
            driver,
            endpoints,
            report_descriptor,
            warnings: interface_warnings,
            sysfs_path: interface_path.display().to_string(),
            capabilities: discovered.capabilities,
        });
    }
    interfaces.sort_by_key(|interface| interface.number.unwrap_or(u8::MAX));
    if interfaces.is_empty() {
        warnings.push("no USB interface entries were readable".into());
    }

    Ok(UsbDevice {
        stable_id: metadata.stable_id.clone(),
        kind,
        label: device_label(kind).into(),
        vendor_id,
        product_id,
        manufacturer: metadata.manufacturer.clone(),
        product: metadata.product.clone(),
        serial_number: metadata.serial_number.clone(),
        usb_release: discovery.read_optional_device_attribute(instance, "bcdDevice")?,
        speed_mbps: discovery.read_optional_device_attribute(instance, "speed")?,
        bus_number,
        device_number,
        usb_device_path,
        sysfs_path: path.display().to_string(),
        interfaces,
        warnings,
    })
}

fn enumerate_from_roots(
    usb_root: &Path,
    hidraw_root: &Path,
) -> Result<Vec<UsbDevice>, Box<dyn Error>> {
    // Synthetic/custom roots must remain hermetic: do not even stat a host
    // USB device node while exercising the offline discovery path.
    let allow_system_device_node_check =
        usb_root == Path::new(USB_SYSFS_ROOT) && hidraw_root == Path::new(HIDRAW_SYSFS_ROOT);
    let discovery = SysfsDiscovery::new(usb_root, hidraw_root);
    let instances = discovery.enumerate()?;
    let mut devices = Vec::new();
    for instance in instances {
        let metadata = discovery.inspect_metadata(&instance)?;
        let device = read_device(
            &instance,
            &discovery,
            &metadata,
            allow_system_device_node_check,
        )?;
        let latest = discovery.inspect_metadata(&instance)?;
        if latest != metadata {
            return Err(format!(
                "device instance {} changed identity during diagnostic snapshot",
                instance.sysfs_path.display()
            )
            .into());
        }
        devices.push(device);
    }
    devices.sort_by(|left, right| left.stable_id.cmp(&right.stable_id));
    Ok(devices)
}

fn enumerate() -> Result<Vec<UsbDevice>, Box<dyn Error>> {
    let usb_root = env::var_os("TH80_USB_SYSFS_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| USB_SYSFS_ROOT.into());
    let hidraw_root = env::var_os("TH80_HIDRAW_SYSFS_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| HIDRAW_SYSFS_ROOT.into());
    enumerate_from_roots(&usb_root, &hidraw_root)
}

fn print_device(device: &UsbDevice, verbose: bool) {
    println!("{} [{}]", device.label, device.stable_id);
    println!("  USB {:04x}:{:04x}", device.vendor_id, device.product_id);
    println!(
        "  Manufacturer: {}",
        device.manufacturer.as_deref().unwrap_or("(unavailable)")
    );
    println!(
        "  Product: {}",
        device.product.as_deref().unwrap_or("(unavailable)")
    );
    println!(
        "  Serial: {}",
        device.serial_number.as_deref().unwrap_or("(not exposed)")
    );
    println!(
        "  USB release: {}",
        device.usb_release.as_deref().unwrap_or("(unavailable)")
    );
    println!(
        "  Speed: {} Mbps",
        device.speed_mbps.as_deref().unwrap_or("(unavailable)")
    );
    println!(
        "  USB device path: {}",
        device.usb_device_path.as_deref().unwrap_or("(unavailable)")
    );
    println!("  Interfaces: {}", device.interfaces.len());
    for interface in &device.interfaces {
        println!(
            "    Interface {}: class={} subclass={} protocol={} driver={}",
            interface
                .number
                .map(|value| format!("{value}"))
                .unwrap_or_else(|| "?".into()),
            interface
                .class
                .map(|value| format!("0x{value:02x}"))
                .unwrap_or_else(|| "?".into()),
            interface
                .subclass
                .map(|value| format!("0x{value:02x}"))
                .unwrap_or_else(|| "?".into()),
            interface
                .protocol
                .map(|value| format!("0x{value:02x}"))
                .unwrap_or_else(|| "?".into()),
            interface.driver.as_deref().unwrap_or("(unbound)")
        );
        for endpoint in &interface.endpoints {
            println!(
                "      Endpoint {} {} {} max-packet={} interval={}",
                endpoint
                    .address
                    .map(|value| format!("0x{value:02x}"))
                    .unwrap_or_else(|| "?".into()),
                endpoint.direction.as_deref().unwrap_or("?"),
                endpoint.transfer_type.as_deref().unwrap_or("?"),
                endpoint
                    .max_packet_size
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "?".into()),
                endpoint
                    .interval
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "?".into())
            );
        }
        if verbose {
            if let Some(descriptor) = &interface.report_descriptor {
                println!(
                    "      Report descriptor: {} bytes ({})",
                    descriptor.length, descriptor.source
                );
                println!("      {}", descriptor.hex);
            } else if interface.class == Some(0x03) {
                println!("      Report descriptor: unavailable");
            }
        }
        for warning in &interface.warnings {
            println!("      Warning: {warning}");
        }
    }
    for warning in &device.warnings {
        println!("  Warning: {warning}");
    }
}

fn capability_device(device: &UsbDevice) -> DeviceCapabilities {
    let interfaces = device
        .interfaces
        .iter()
        .map(|interface| {
            let capability = interface.capabilities.as_ref();
            let descriptor_length = interface.report_descriptor.as_ref().map(|d| d.length);
            CapabilityInterface {
                interface_number: interface.number,
                interface_class: interface.class,
                interface_subclass: interface.subclass,
                interface_protocol: interface.protocol,
                descriptor_source: interface
                    .report_descriptor
                    .as_ref()
                    .map(|d| d.source.clone()),
                descriptor_length,
                report_descriptor_sha256: capability.map(|value| value.descriptor.sha256.clone()),
                uses_report_ids: capability.map(|value| value.uses_report_ids),
                reports: capability.map_or_else(Vec::new, |value| value.reports.clone()),
                long_items: capability.map_or_else(Vec::new, |value| value.long_items.clone()),
                input_report_count: capability.map_or(0, |value| value.input_report_count),
                output_report_count: capability.map_or(0, |value| value.output_report_count),
                feature_report_count: capability.map_or(0, |value| value.feature_report_count),
                parser_warnings: capability
                    .map_or_else(Vec::new, |value| value.parser_warnings.clone()),
                parser_errors: Vec::new(),
                endpoints: interface.endpoints.clone(),
                warnings: interface.warnings.clone(),
            }
        })
        .collect();
    DeviceCapabilities {
        stable_id: device.stable_id.clone(),
        kind: device.kind,
        label: device.label.clone(),
        vendor_id: device.vendor_id,
        product_id: device.product_id,
        manufacturer: device.manufacturer.clone(),
        product: device.product.clone(),
        serial_number: device.serial_number.clone(),
        interfaces,
        warnings: device.warnings.clone(),
    }
}

fn print_capabilities(device: &DeviceCapabilities) {
    println!("{} [{}]", device.label, device.stable_id);
    println!(
        "  USB {:04x}:{:04x} manufacturer={} product={}",
        device.vendor_id,
        device.product_id,
        device.manufacturer.as_deref().unwrap_or("unavailable"),
        device.product.as_deref().unwrap_or("unavailable")
    );
    for interface in &device.interfaces {
        println!(
            "  Interface {} class={}",
            interface
                .interface_number
                .map(|v| v.to_string())
                .unwrap_or_else(|| "?".into()),
            interface
                .interface_class
                .map(|v| format!("0x{v:02x}"))
                .unwrap_or_else(|| "?".into())
        );
        println!(
            "    Descriptor: {} bytes SHA-256={}",
            interface
                .descriptor_length
                .map(|v| v.to_string())
                .unwrap_or_else(|| "unavailable".into()),
            interface
                .report_descriptor_sha256
                .as_deref()
                .unwrap_or("unavailable")
        );
        println!(
            "    Report IDs: {}",
            interface
                .uses_report_ids
                .map(|v| if v { "explicit IDs" } else { "implicit ID 0" })
                .unwrap_or("unknown")
        );
        for endpoint in &interface.endpoints {
            println!(
                "    USB endpoint {} {} {} packet-size={} bytes (not a HID report length)",
                endpoint
                    .address
                    .map(|v| format!("0x{v:02x}"))
                    .unwrap_or_else(|| "?".into()),
                endpoint.direction.as_deref().unwrap_or("?"),
                endpoint.transfer_type.as_deref().unwrap_or("?"),
                endpoint
                    .max_packet_size
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "unknown".into())
            );
        }
        println!(
            "    Reports: input={} output={} feature={}",
            interface.input_report_count,
            interface.output_report_count,
            interface.feature_report_count
        );
        for report in &interface.reports {
            println!("    {:?} ID={}{}: {} bits, {} payload bytes, {} transfer bytes (uniform-size={} bits; aggregate-count={})",
                report.report_type, report.report_id, if report.report_id_explicit { " explicit" } else { " implicit" },
                report.payload_bits, report.payload_bytes, report.transfer_bytes, report.report_size_bits.map(|v| v.to_string()).unwrap_or_else(|| "mixed".into()), report.report_count);
            for field in &report.fields {
                for usage in &field.usages {
                    println!("      Usage {:#06x}:{:#06x}", usage.page, usage.id);
                }
                if let (Some(min), Some(max)) = (&field.usage_minimum, &field.usage_maximum) {
                    println!(
                        "      Usage range {:#06x}:{:#06x}..{:#06x}:{:#06x}",
                        min.page, min.id, max.page, max.id
                    );
                }
                if !field.collection_path.is_empty() {
                    println!(
                        "      Collection path: {}",
                        field.collection_path.join(" / ")
                    );
                }
            }
        }
        for warning in &interface.parser_warnings {
            println!(
                "    Parser warning at 0x{:x}: {}",
                warning.offset, warning.message
            );
        }
        for error in &interface.parser_errors {
            println!("    Parser error: {error}");
        }
        for warning in &interface.warnings {
            println!("    Warning: {warning}");
        }
    }
}

fn print_comparison(devices: &[DeviceCapabilities]) {
    let keyboard = devices.iter().find(|d| d.kind == DeviceKind::Th80Pro);
    let receiver = devices
        .iter()
        .find(|d| d.kind == DeviceKind::RoyuanReceiver);
    if let (Some(keyboard), Some(receiver)) = (keyboard, receiver) {
        println!("Keyboard / receiver comparison:");
        println!(
            "  Identity: {:04x}:{:04x} vs {:04x}:{:04x}",
            keyboard.vendor_id, keyboard.product_id, receiver.vendor_id, receiver.product_id
        );
        println!(
            "  HID interfaces: {} vs {}",
            keyboard.interfaces.len(),
            receiver.interfaces.len()
        );
        for (left, right) in keyboard.interfaces.iter().zip(&receiver.interfaces) {
            println!(
                "  Interface {}: descriptor SHA-256 {} | {} ; report definitions {} | {}",
                left.interface_number
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "?".into()),
                left.report_descriptor_sha256
                    .as_deref()
                    .unwrap_or("unavailable"),
                right
                    .report_descriptor_sha256
                    .as_deref()
                    .unwrap_or("unavailable"),
                left.reports.len(),
                right.reports.len()
            );
        }
    }
}

fn usage() {
    eprintln!("TH80 passive USB diagnostic CLI\n\nUSAGE:\n  th80-probe list [--json]\n  th80-probe inspect --device keyboard|receiver [--json]\n  th80-probe inspect --all [--json]\n  th80-probe inspect --json\n  th80-probe capabilities --device keyboard|receiver [--json]\n  th80-probe capabilities --all [--json]\n  th80-probe capabilities --json\n  th80-probe protocol [--json]\n\nThe protocol command prints offline yc500 packet contracts only. Other commands inspect USB sysfs metadata and readable HID report-descriptor files. No device is opened.");
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    let command = args.first().map(String::as_str).unwrap_or("list");
    if !matches!(
        command,
        "list" | "inspect" | "capabilities" | "protocol" | "--help" | "-h"
    ) {
        usage();
        return Err(format!("unknown command: {command}").into());
    }
    if matches!(command, "--help" | "-h") {
        usage();
        return Ok(());
    }
    let json = args.iter().any(|arg| arg == "--json");
    if command == "protocol" {
        let snapshot = protocol_snapshot()?;
        if json {
            println!("{}", serde_json::to_string_pretty(&snapshot)?);
        } else {
            print_protocol(&snapshot);
        }
        return Ok(());
    }
    let all = args.iter().any(|arg| arg == "--all");
    let selected = args
        .windows(2)
        .find(|pair| pair[0] == "--device")
        .map(|pair| pair[1].as_str());
    if matches!(command, "inspect" | "capabilities") && !all && selected.is_none() && !json {
        usage();
        return Err(
            format!("{command} requires --device keyboard|receiver, --all, or --json").into(),
        );
    }
    let mut devices = enumerate()?;
    if let Some(kind) = selected {
        let wanted = match kind {
            "keyboard" => DeviceKind::Th80Pro,
            "receiver" => DeviceKind::RoyuanReceiver,
            _ => return Err("--device must be keyboard or receiver".into()),
        };
        devices.retain(|device| device.kind == wanted);
    }
    if command == "capabilities" {
        let capabilities: Vec<_> = devices.iter().map(capability_device).collect();
        if json {
            let snapshot = CapabilitySnapshot {
                schema_version: 1,
                tool: "th80-probe",
                mode: "passive_hid_capabilities",
                devices: capabilities,
            };
            println!("{}", serde_json::to_string_pretty(&snapshot)?);
        } else if capabilities.is_empty() {
            match selected {
                Some("keyboard") => println!("EPOMAKER TH80 Pro (3151:4010) is not currently connected; no descriptor fixture is configured."),
                Some("receiver") => println!("ROYUAN receiver (3151:4011) is not currently connected; no descriptor fixture is configured."),
                _ => println!("No matching TH80 Pro or receiver currently detected."),
            }
        } else {
            for device in &capabilities {
                print_capabilities(device);
            }
            print_comparison(&capabilities);
        }
    } else if json {
        let snapshot = DiagnosticSnapshot {
            schema_version: 1,
            tool: "th80-probe",
            devices,
        };
        println!("{}", serde_json::to_string_pretty(&snapshot)?);
    } else if devices.is_empty() {
        match selected {
            Some("keyboard") => {
                println!("EPOMAKER TH80 Pro (3151:4010) is not currently connected.")
            }
            Some("receiver") => println!("ROYUAN receiver (3151:4011) is not currently connected."),
            _ => println!("No matching TH80 Pro or receiver currently detected."),
        }
    } else {
        let verbose = command == "inspect";
        for device in &devices {
            print_device(device, verbose);
        }
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use th80_probe::device::DeviceIdentity;

    #[test]
    fn identifies_target_vid_pid_pairs() {
        assert_eq!(
            DeviceIdentity::from_vid_pid(0x3151, 0x4010),
            Some(DeviceIdentity::TH80_PRO)
        );
        assert_eq!(
            DeviceIdentity::from_vid_pid(0x3151, 0x4011),
            Some(DeviceIdentity::ROYUAN_RECEIVER)
        );
        assert_eq!(DeviceIdentity::from_vid_pid(0x3151, 0x4001), None);
        assert_eq!(DeviceIdentity::from_vid_pid(0x1234, 0x4010), None);
    }

    #[test]
    fn represents_interfaces_and_endpoints() {
        let endpoint = EndpointInfo {
            address: Some(0x83),
            direction: Some("in".into()),
            transfer_type: Some("interrupt".into()),
            max_packet_size: Some(8),
            interval: Some(1),
            sysfs_path: "/sys/test/ep_83".into(),
            warnings: vec![],
        };
        let interface = InterfaceInfo {
            number: Some(2),
            class: Some(3),
            subclass: Some(0),
            protocol: Some(0),
            driver: Some("usbhid".into()),
            endpoints: vec![endpoint],
            report_descriptor: None,
            warnings: vec![],
            sysfs_path: "/sys/test/1:2".into(),
            capabilities: None,
        };
        assert_eq!(interface.number, Some(2));
        assert_eq!(interface.endpoints[0].address, Some(0x83));
        assert_eq!(interface.endpoints[0].max_packet_size, Some(8));
    }

    #[test]
    fn serializes_json_snapshot() {
        let snapshot = DiagnosticSnapshot {
            schema_version: 1,
            tool: "th80-probe",
            devices: vec![],
        };
        let json = serde_json::to_string(&snapshot).unwrap();
        assert!(json.contains("\"schema_version\":1"));
        assert!(json.contains("\"devices\":[]"));
    }

    #[test]
    fn tolerates_missing_optional_usb_metadata() {
        let device = UsbDevice {
            stable_id: "3151:4010@1-9".into(),
            kind: DeviceKind::Th80Pro,
            label: "EPOMAKER TH80 Pro".into(),
            vendor_id: 0x3151,
            product_id: 0x4010,
            manufacturer: None,
            product: None,
            serial_number: None,
            usb_release: None,
            speed_mbps: None,
            bus_number: None,
            device_number: None,
            usb_device_path: None,
            sysfs_path: "/sys/test".into(),
            interfaces: vec![],
            warnings: vec![],
        };
        let json = serde_json::to_string(&device).unwrap();
        assert!(json.contains("\"manufacturer\":null"));
        assert!(json.contains("\"usb_device_path\":null"));
        assert!(json.contains("\"kind\":\"keyboard\""));
    }

    #[test]
    fn parses_decimal_and_hex_sysfs_numbers() {
        assert_eq!(parse_num::<u8>("129"), Some(129));
        assert_eq!(parse_num::<u8>("0x81"), Some(129));
        assert_eq!(parse_num::<u16>("512"), Some(512));
        assert_eq!(
            u16::try_from(u64::from_str_radix("3151", 16).unwrap()).ok(),
            Some(0x3151)
        );
    }

    #[test]
    fn protocol_snapshot_is_offline_and_uses_the_yc500_codec() {
        let snapshot = protocol_snapshot().expect("offline protocol contract should build");
        assert_eq!((snapshot.vendor_id, snapshot.product_id), (0x3151, 0x4010));
        assert_eq!(snapshot.family, "yc500");
        assert_eq!(snapshot.feature_report_bytes, 64);
        assert!(snapshot
            .commands
            .iter()
            .any(|item| { item.candidate == "C10" && item.opcode == "0x89" }));
        assert!(snapshot
            .unsupported
            .iter()
            .any(|item| item.candidate == "C04"));
        assert!(snapshot
            .unsupported
            .iter()
            .any(|item| item.candidate == "C03"));
    }
}
