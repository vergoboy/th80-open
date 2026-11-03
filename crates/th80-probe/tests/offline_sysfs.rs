#![cfg(target_os = "linux")]

use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};
use th80_probe::{
    device::InterfaceRole,
    discovery::{DescriptorSource, DiscoveryProvider},
    sysfs::{SysfsDescriptorSource, SysfsDiscovery},
    DeviceIdentity,
};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    root: PathBuf,
    usb: PathBuf,
    hidraw: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "th80-sysfs-fixture-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let usb = root.join("usb");
        let hidraw = root.join("hidraw");
        fs::create_dir_all(&usb).unwrap();
        fs::create_dir_all(&hidraw).unwrap();
        Self { root, usb, hidraw }
    }

    fn add_keyboard(&self, descriptor: &[u8]) {
        self.add_device(
            "1-2",
            "hidraw0",
            "3151",
            "4010",
            "Synthetic TH80 Pro",
            descriptor,
        );
    }

    fn add_receiver(&self, descriptor: &[u8]) {
        self.add_device(
            "1-3",
            "hidraw1",
            "3151",
            "4011",
            "Synthetic ROYUAN receiver",
            descriptor,
        );
    }

    fn add_unknown(&self, descriptor: &[u8]) {
        self.add_device(
            "1-4",
            "hidraw2",
            "1234",
            "9999",
            "Unrelated synthetic device",
            descriptor,
        );
    }

    fn add_device(
        &self,
        topology: &str,
        hidraw_name: &str,
        vendor_id: &str,
        product_id: &str,
        product_name: &str,
        descriptor: &[u8],
    ) {
        let device = self.usb.join(topology);
        let interface = device.join(format!("{topology}:1.2"));
        let endpoint = interface.join("ep_83");
        fs::create_dir_all(&endpoint).unwrap();
        write(&device.join("idVendor"), &format!("{vendor_id}\n"));
        write(&device.join("idProduct"), &format!("{product_id}\n"));
        write(&device.join("manufacturer"), "Synthetic Vendor\n");
        write(&device.join("product"), &format!("{product_name}\n"));
        write(&device.join("serial"), &format!("SYNTHETIC-{topology}\n"));
        write(&device.join("busnum"), "001\n");
        write(&device.join("devnum"), "001\n");
        write(&interface.join("bInterfaceNumber"), "02\n");
        write(&interface.join("bInterfaceClass"), "03\n");
        write(&interface.join("bInterfaceSubClass"), "00\n");
        write(&interface.join("bInterfaceProtocol"), "00\n");
        write(&endpoint.join("bEndpointAddress"), "83\n");
        write(&endpoint.join("bmAttributes"), "03\n");
        write(&endpoint.join("wMaxPacketSize"), "0008\n");
        write(&endpoint.join("bInterval"), "01\n");
        let hid_device = interface.join(format!("0003:{vendor_id}:{product_id}.0001"));
        fs::create_dir_all(&hid_device).unwrap();
        fs::write(hid_device.join("report_descriptor"), descriptor).unwrap();
        let hid = self.hidraw.join(hidraw_name);
        fs::create_dir_all(&hid).unwrap();
        std::os::unix::fs::symlink(&hid_device, hid.join("device")).unwrap();
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_th80-probe"))
            .args(args)
            .env("TH80_USB_SYSFS_ROOT", &self.usb)
            .env("TH80_HIDRAW_SYSFS_ROOT", &self.hidraw)
            .output()
            .unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn write(path: &Path, value: &str) {
    fs::write(path, value).unwrap();
}

const FEATURE_DESCRIPTOR: &[u8] = &[
    0x06, 0xff, 0xff, 0x09, 0x02, 0xa1, 0x01, 0x09, 0x02, 0x15, 0x80, 0x25, 0x7f, 0x95, 0x40, 0x75,
    0x08, 0xb1, 0x02, 0xc0,
];

#[test]
fn synthetic_sysfs_inventory_flows_through_list_and_capability_cli() {
    let fixture = Fixture::new();
    fixture.add_keyboard(FEATURE_DESCRIPTOR);
    fixture.add_receiver(FEATURE_DESCRIPTOR);
    fixture.add_unknown(FEATURE_DESCRIPTOR);

    let listed = fixture.run(&["list", "--json"]);
    assert!(
        listed.status.success(),
        "{}",
        String::from_utf8_lossy(&listed.stderr)
    );
    let snapshot: Value = serde_json::from_slice(&listed.stdout).unwrap();
    assert_eq!(snapshot["devices"].as_array().unwrap().len(), 2);
    assert_eq!(snapshot["devices"][0]["kind"], "keyboard");
    assert_eq!(snapshot["devices"][1]["kind"], "receiver");
    assert_eq!(snapshot["devices"][0]["vendor_id"], 0x3151);
    assert_eq!(snapshot["devices"][0]["product_id"], 0x4010);
    assert_eq!(snapshot["devices"][0]["manufacturer"], "Synthetic Vendor");
    assert!(snapshot["devices"][0]["usb_device_path"].is_null());
    assert_eq!(snapshot["devices"][0]["interfaces"][0]["number"], 2);
    assert_eq!(
        snapshot["devices"][0]["interfaces"][0]["endpoints"][0]["address"],
        0x83
    );
    assert_eq!(snapshot["devices"][1]["kind"], "receiver");

    let capabilities = fixture.run(&["capabilities", "--all", "--json"]);
    assert!(
        capabilities.status.success(),
        "{}",
        String::from_utf8_lossy(&capabilities.stderr)
    );
    let snapshot: Value = serde_json::from_slice(&capabilities.stdout).unwrap();
    assert_eq!(snapshot["devices"].as_array().unwrap().len(), 2);
    let interface = &snapshot["devices"][0]["interfaces"][0];
    assert_eq!(interface["descriptor_length"], FEATURE_DESCRIPTOR.len());
    assert_eq!(interface["feature_report_count"], 1);
    assert_eq!(interface["reports"][0]["report_type"], "feature");
    assert_eq!(interface["reports"][0]["transfer_bytes"], 64);
    assert!(interface["parser_errors"].as_array().unwrap().is_empty());
}

#[test]
fn protocol_cli_is_offline_and_does_not_enumerate_configured_sysfs_roots() {
    let root = std::env::temp_dir().join(format!(
        "th80-protocol-cli-empty-roots-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let output = Command::new(env!("CARGO_BIN_EXE_th80-probe"))
        .args(["protocol", "--json"])
        .env("TH80_USB_SYSFS_ROOT", root.join("missing-usb"))
        .env("TH80_HIDRAW_SYSFS_ROOT", root.join("missing-hidraw"))
        .output()
        .expect("probe protocol command should start");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let snapshot: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(snapshot["mode"], "offline_protocol_contracts");
    assert_eq!(snapshot["family"], "yc500");
    assert_eq!(snapshot["feature_report_bytes"], 64);
    assert_eq!(snapshot["commands"][0]["candidate"], "C01");
}

#[test]
fn reusable_discovery_provider_returns_typed_metadata_and_descriptor_capabilities() {
    let fixture = Fixture::new();
    fixture.add_keyboard(FEATURE_DESCRIPTOR);
    fixture.add_receiver(FEATURE_DESCRIPTOR);
    fixture.add_unknown(FEATURE_DESCRIPTOR);
    let provider = SysfsDiscovery::new(&fixture.usb, &fixture.hidraw);

    let instances = provider.enumerate().unwrap();
    assert_eq!(instances.len(), 2);
    assert_eq!(instances[0].identity, DeviceIdentity::TH80_PRO);
    assert_eq!(instances[1].identity, DeviceIdentity::ROYUAN_RECEIVER);

    let metadata = provider.inspect_metadata(&instances[0]).unwrap();
    assert_eq!(metadata.stable_id, "3151:4010@1-2");
    assert_eq!(metadata.manufacturer.as_deref(), Some("Synthetic Vendor"));
    assert_eq!(metadata.product.as_deref(), Some("Synthetic TH80 Pro"));
    assert_eq!(metadata.serial_number.as_deref(), Some("SYNTHETIC-1-2"));

    let interfaces = provider.inspect_interfaces(&instances[0]).unwrap();
    assert_eq!(interfaces.len(), 1);
    assert_eq!(interfaces[0].number, 2);
    assert_eq!(interfaces[0].role, InterfaceRole::VendorFeature);
    assert_eq!(interfaces[0].descriptor.as_ref().unwrap().length, 20);
    assert_eq!(interfaces[0].endpoints[0].address, 0x83);

    let descriptor_source = SysfsDescriptorSource::new(&fixture.hidraw);
    let descriptor = descriptor_source
        .read_report_descriptor(&instance_path(&instances[0]))
        .unwrap()
        .unwrap();
    assert_eq!(descriptor.bytes, FEATURE_DESCRIPTOR);
    assert!(descriptor.source.ends_with("report_descriptor"));

    let receiver_metadata = provider.inspect_metadata(&instances[1]).unwrap();
    assert_eq!(receiver_metadata.identity, DeviceIdentity::ROYUAN_RECEIVER);
    assert_eq!(
        receiver_metadata.product.as_deref(),
        Some("Synthetic ROYUAN receiver")
    );
}

fn instance_path(instance: &th80_probe::DeviceInstance) -> PathBuf {
    instance.sysfs_path.join(format!(
        "{}:1.2",
        instance.sysfs_path.file_name().unwrap().to_string_lossy()
    ))
}

#[test]
fn discovery_rejects_a_sysfs_path_reused_for_a_different_device_identity() {
    let fixture = Fixture::new();
    fixture.add_keyboard(FEATURE_DESCRIPTOR);
    let provider = SysfsDiscovery::new(&fixture.usb, &fixture.hidraw);
    let instance = provider.enumerate().unwrap().remove(0);

    write(&instance.sysfs_path.join("idProduct"), "4011\n");
    assert!(matches!(
        provider.inspect_metadata(&instance),
        Err(th80_probe::DriverError::DeviceUnavailable(_))
    ));
}

#[test]
fn malformed_synthetic_usb_identity_is_reported_instead_of_silently_skipped() {
    let fixture = Fixture::new();
    fixture.add_keyboard(FEATURE_DESCRIPTOR);
    write(&fixture.usb.join("1-2/idVendor"), "not-hex\n");

    let provider = SysfsDiscovery::new(&fixture.usb, &fixture.hidraw);
    assert!(matches!(
        provider.enumerate(),
        Err(th80_probe::DriverError::MalformedReport(message))
            if message.contains("idVendor")
    ));
}

#[test]
fn stale_device_identity_is_rejected_even_when_replacement_is_unsupported() {
    let fixture = Fixture::new();
    fixture.add_keyboard(FEATURE_DESCRIPTOR);
    let provider = SysfsDiscovery::new(&fixture.usb, &fixture.hidraw);
    let instance = provider.enumerate().unwrap().remove(0);

    write(&instance.sysfs_path.join("idVendor"), "1234\n");
    write(&instance.sysfs_path.join("idProduct"), "9999\n");
    assert!(matches!(
        provider.inspect_metadata(&instance),
        Err(th80_probe::DriverError::DeviceUnavailable(_))
    ));
    assert!(matches!(
        provider.read_optional_device_attribute(&instance, "speed"),
        Err(th80_probe::DriverError::DeviceUnavailable(_))
    ));
}

#[test]
fn discovery_rejects_usb_symlinks_that_escape_the_configured_synthetic_root() {
    let fixture = Fixture::new();
    let outside = fixture.root.join("outside-device");
    fs::create_dir_all(&outside).unwrap();
    write(&outside.join("idVendor"), "3151\n");
    write(&outside.join("idProduct"), "4010\n");
    std::os::unix::fs::symlink(&outside, fixture.usb.join("1-9")).unwrap();

    let provider = SysfsDiscovery::new(&fixture.usb, &fixture.hidraw);
    assert!(matches!(
        provider.enumerate(),
        Err(th80_probe::DriverError::PermissionDenied(_))
    ));
}

#[test]
fn descriptor_symlink_cannot_escape_its_interface_directory() {
    let fixture = Fixture::new();
    fixture.add_keyboard(FEATURE_DESCRIPTOR);
    let interface = fixture.usb.join("1-2/1-2:1.2");
    let hid_device = interface.join("0003:3151:4010.0001");
    fs::remove_file(hid_device.join("report_descriptor")).unwrap();
    let outside = fixture.root.join("outside-descriptor");
    fs::write(&outside, FEATURE_DESCRIPTOR).unwrap();
    std::os::unix::fs::symlink(&outside, hid_device.join("report_descriptor")).unwrap();

    let provider = SysfsDiscovery::new(&fixture.usb, &fixture.hidraw);
    let instance = provider.enumerate().unwrap().remove(0);
    assert!(matches!(
        provider.inspect_interfaces(&instance),
        Err(th80_probe::DriverError::PermissionDenied(_))
    ));
}

#[test]
fn attribute_symlink_cannot_escape_the_configured_synthetic_root() {
    let fixture = Fixture::new();
    fixture.add_keyboard(FEATURE_DESCRIPTOR);
    let outside = fixture.root.join("outside-product");
    fs::write(&outside, "Host data must not be read\n").unwrap();
    fs::remove_file(fixture.usb.join("1-2/product")).unwrap();
    std::os::unix::fs::symlink(&outside, fixture.usb.join("1-2/product")).unwrap();

    let provider = SysfsDiscovery::new(&fixture.usb, &fixture.hidraw);
    let instance = provider.enumerate().unwrap().remove(0);
    assert!(matches!(
        provider.inspect_metadata(&instance),
        Err(th80_probe::DriverError::PermissionDenied(_))
    ));
}

#[test]
fn oversized_synthetic_sysfs_attribute_is_bounded_and_rejected() {
    let fixture = Fixture::new();
    fixture.add_keyboard(FEATURE_DESCRIPTOR);
    fs::write(fixture.usb.join("1-2/product"), vec![b'x'; 4097]).unwrap();

    let provider = SysfsDiscovery::new(&fixture.usb, &fixture.hidraw);
    let instance = provider.enumerate().unwrap().remove(0);
    assert!(matches!(
        provider.inspect_metadata(&instance),
        Err(th80_probe::DriverError::MalformedReport(message))
            if message.contains("4096-byte limit")
    ));
}

#[test]
fn malformed_synthetic_descriptor_is_reported_without_panicking() {
    let fixture = Fixture::new();
    fixture.add_keyboard(&[0xfe, 0x04, 0x99, 0xaa]);

    let output = fixture.run(&["capabilities", "--all", "--json"]);
    assert_eq!(output.status.code(), Some(2));
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(error.contains("malformed report"), "{error}");
    assert!(error.contains("truncated long item"), "{error}");
}

#[test]
fn oversized_synthetic_descriptor_is_rejected_before_unbounded_read() {
    let fixture = Fixture::new();
    fixture.add_keyboard(&vec![0x00; 4097]);
    let provider = SysfsDiscovery::new(&fixture.usb, &fixture.hidraw);
    let instance = provider.enumerate().unwrap().remove(0);
    assert!(matches!(
        provider.inspect_interfaces(&instance),
        Err(th80_probe::DriverError::MalformedReport(message))
            if message.contains("4096-byte limit")
    ));
}
