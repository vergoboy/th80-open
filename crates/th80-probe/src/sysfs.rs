//! Linux passive discovery backed only by sysfs metadata and readable HID
//! report-descriptor files. No `/dev/hidraw*` or USB device node is opened.

use crate::{
    capabilities,
    device::{
        DeviceIdentity, DeviceInstance, DeviceMetadata, EndpointInfo, InterfaceInfo, InterfaceRole,
        TransferType,
    },
    discovery::{DescriptorSource, DiscoveryProvider, ReportDescriptorObservation},
    error::DriverError,
};
use std::{
    fs,
    io::{ErrorKind, Read},
    path::{Path, PathBuf},
};

const USB_SYSFS_ROOT: &str = "/sys/bus/usb/devices";
const HIDRAW_SYSFS_ROOT: &str = "/sys/class/hidraw";

#[derive(Debug, Clone)]
pub struct SysfsDiscovery {
    usb_root: PathBuf,
    sysfs_boundary: PathBuf,
    descriptor_source: SysfsDescriptorSource,
}

#[derive(Debug, Clone)]
pub struct SysfsDescriptorSource {
    hidraw_root: PathBuf,
}

impl SysfsDiscovery {
    pub fn new(usb_root: impl Into<PathBuf>, hidraw_root: impl Into<PathBuf>) -> Self {
        let usb_root = usb_root.into();
        let system_root = Path::new(USB_SYSFS_ROOT);
        let sysfs_boundary = if usb_root == system_root {
            PathBuf::from("/sys")
        } else {
            usb_root.clone()
        };
        Self {
            usb_root,
            sysfs_boundary,
            descriptor_source: SysfsDescriptorSource::new(hidraw_root),
        }
    }

    /// Uses the platform's standard Linux sysfs locations.
    pub fn system() -> Self {
        Self::new(USB_SYSFS_ROOT, HIDRAW_SYSFS_ROOT)
    }

    /// Reads a small allow-listed set of optional USB metadata attributes
    /// through the same configured sysfs boundary used for enumeration.
    pub fn read_optional_device_attribute(
        &self,
        instance: &DeviceInstance,
        attribute: &str,
    ) -> Result<Option<String>, DriverError> {
        const ALLOWED: &[&str] = &["bcdDevice", "speed", "busnum", "devnum"];
        if !ALLOWED.contains(&attribute) {
            return Err(DriverError::UnsupportedOperation(
                "unapproved sysfs metadata attribute",
            ));
        }
        self.validate_instance_path(instance)?;
        verified_identity(
            &self.sysfs_boundary,
            &instance.sysfs_path,
            instance.identity,
        )?;
        read_optional_text_within(&self.sysfs_boundary, &instance.sysfs_path.join(attribute))
    }
}

impl SysfsDescriptorSource {
    pub fn new(hidraw_root: impl Into<PathBuf>) -> Self {
        Self {
            hidraw_root: hidraw_root.into(),
        }
    }
}

fn read_optional_text(path: &Path) -> Result<Option<String>, DriverError> {
    let mut value = String::new();
    let mut file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(DriverError::from(error)),
    };
    file.by_ref()
        .take(4097)
        .read_to_string(&mut value)
        .map_err(DriverError::from)?;
    if value.len() > 4096 {
        return Err(DriverError::MalformedReport(format!(
            "sysfs attribute exceeds the supported 4096-byte limit: {}",
            path.display()
        )));
    }
    let value = value.trim().to_owned();
    Ok((!value.is_empty()).then_some(value))
}

fn read_optional_text_within(boundary: &Path, path: &Path) -> Result<Option<String>, DriverError> {
    let canonical_boundary = boundary.canonicalize().map_err(DriverError::from)?;
    let canonical_path = match path.canonicalize() {
        Ok(path) => path,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(DriverError::from(error)),
    };
    if !canonical_path.starts_with(canonical_boundary) {
        return Err(DriverError::PermissionDenied(format!(
            "sysfs attribute escaped configured boundary: {}",
            path.display()
        )));
    }
    read_optional_text(&canonical_path)
}

fn parse_number<T>(boundary: &Path, path: &Path, radix: u32) -> Result<T, DriverError>
where
    T: TryFrom<u64>,
{
    let value = read_optional_text_within(boundary, path)?.ok_or_else(|| {
        DriverError::DescriptorUnavailable(format!("{} is missing", path.display()))
    })?;
    let parsed = u64::from_str_radix(value.trim_start_matches("0x"), radix).map_err(|error| {
        DriverError::MalformedReport(format!(
            "invalid sysfs value at {}: {error}",
            path.display()
        ))
    })?;
    T::try_from(parsed).map_err(|_| {
        DriverError::MalformedReport(format!("sysfs value at {} is out of range", path.display()))
    })
}

fn verified_identity(
    boundary: &Path,
    path: &Path,
    expected: DeviceIdentity,
) -> Result<DeviceIdentity, DriverError> {
    let vendor_id = parse_number::<u16>(boundary, &path.join("idVendor"), 16)?;
    let product_id = parse_number::<u16>(boundary, &path.join("idProduct"), 16)?;
    if (vendor_id, product_id) != (expected.vendor_id, expected.product_id) {
        return Err(DriverError::DeviceUnavailable(format!(
            "sysfs instance {} changed identity from {:04x}:{:04x} to {vendor_id:04x}:{product_id:04x}",
            path.display(), expected.vendor_id, expected.product_id
        )));
    }
    Ok(expected)
}

impl DescriptorSource for SysfsDescriptorSource {
    fn read_report_descriptor(
        &self,
        interface_path: &Path,
    ) -> Result<Option<ReportDescriptorObservation>, DriverError> {
        let canonical_interface = interface_path.canonicalize()?;
        let entries = match fs::read_dir(&self.hidraw_root) {
            Ok(entries) => entries,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(DriverError::from(error)),
        };
        for entry in entries {
            let entry = entry?;
            let hid_device = entry.path().join("device");
            let Ok(canonical_hid_device) = hid_device.canonicalize() else {
                continue;
            };
            if canonical_hid_device.parent() != Some(canonical_interface.as_path()) {
                continue;
            }
            let descriptor_path = hid_device.join("report_descriptor");
            let canonical_descriptor = match descriptor_path.canonicalize() {
                Ok(path) => path,
                Err(error) if error.kind() == ErrorKind::NotFound => continue,
                Err(error) => return Err(DriverError::from(error)),
            };
            if !canonical_descriptor.starts_with(&canonical_interface) {
                return Err(DriverError::PermissionDenied(format!(
                    "report descriptor escaped its USB interface: {}",
                    descriptor_path.display()
                )));
            }
            let mut bytes = Vec::with_capacity(4096);
            fs::File::open(&canonical_descriptor)
                .map_err(DriverError::from)?
                .take(4097)
                .read_to_end(&mut bytes)
                .map_err(DriverError::from)?;
            if bytes.len() > 4096 {
                return Err(DriverError::MalformedReport(
                    "HID report descriptor exceeds the supported 4096-byte limit".into(),
                ));
            }
            return Ok(Some(ReportDescriptorObservation {
                source: descriptor_path,
                bytes,
            }));
        }
        Ok(None)
    }
}

impl DiscoveryProvider for SysfsDiscovery {
    fn enumerate(&self) -> Result<Vec<DeviceInstance>, DriverError> {
        let entries = fs::read_dir(&self.usb_root).map_err(DriverError::from)?;
        let mut devices = Vec::new();
        for entry in entries {
            let entry = entry.map_err(DriverError::from)?;
            let path = entry.path();
            let canonical_path = path.canonicalize().map_err(DriverError::from)?;
            let canonical_boundary = self
                .sysfs_boundary
                .canonicalize()
                .map_err(DriverError::from)?;
            if !canonical_path.starts_with(&canonical_boundary) {
                return Err(DriverError::PermissionDenied(format!(
                    "USB sysfs entry escaped configured boundary: {}",
                    path.display()
                )));
            }
            let Some(vendor) =
                read_optional_text_within(&self.sysfs_boundary, &canonical_path.join("idVendor"))?
            else {
                continue;
            };
            let Some(product) =
                read_optional_text_within(&self.sysfs_boundary, &canonical_path.join("idProduct"))?
            else {
                continue;
            };
            let vendor_id =
                u16::from_str_radix(vendor.trim_start_matches("0x"), 16).map_err(|error| {
                    DriverError::MalformedReport(format!(
                        "invalid USB idVendor at {}: {error}",
                        canonical_path.display()
                    ))
                })?;
            let product_id =
                u16::from_str_radix(product.trim_start_matches("0x"), 16).map_err(|error| {
                    DriverError::MalformedReport(format!(
                        "invalid USB idProduct at {}: {error}",
                        canonical_path.display()
                    ))
                })?;
            let Some(identity) = DeviceIdentity::from_vid_pid(vendor_id, product_id) else {
                continue;
            };
            devices.push(DeviceInstance {
                identity,
                sysfs_path: canonical_path,
            });
        }
        devices.sort_by(|left, right| left.sysfs_path.cmp(&right.sysfs_path));
        Ok(devices)
    }

    fn inspect_metadata(&self, instance: &DeviceInstance) -> Result<DeviceMetadata, DriverError> {
        self.validate_instance_path(instance)?;
        let current = verified_identity(
            &self.sysfs_boundary,
            &instance.sysfs_path,
            instance.identity,
        )?;
        let topology = instance
            .sysfs_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown-topology");
        Ok(DeviceMetadata {
            identity: current,
            stable_id: format!(
                "{:04x}:{:04x}@{topology}",
                current.vendor_id, current.product_id
            ),
            manufacturer: read_optional_text_within(
                &self.sysfs_boundary,
                &instance.sysfs_path.join("manufacturer"),
            )?,
            product: read_optional_text_within(
                &self.sysfs_boundary,
                &instance.sysfs_path.join("product"),
            )?,
            serial_number: read_optional_text_within(
                &self.sysfs_boundary,
                &instance.sysfs_path.join("serial"),
            )?,
        })
    }

    fn inspect_interfaces(
        &self,
        instance: &DeviceInstance,
    ) -> Result<Vec<InterfaceInfo>, DriverError> {
        self.validate_instance_path(instance)?;
        verified_identity(
            &self.sysfs_boundary,
            &instance.sysfs_path,
            instance.identity,
        )?;
        let entries = fs::read_dir(&instance.sysfs_path).map_err(DriverError::from)?;
        let mut interfaces = Vec::new();
        for entry in entries {
            let entry = entry.map_err(DriverError::from)?;
            let path = entry.path();
            if !entry.file_type().map_err(DriverError::from)?.is_dir()
                || !entry.file_name().to_string_lossy().contains(':')
            {
                continue;
            }
            let number =
                parse_number::<u8>(&self.sysfs_boundary, &path.join("bInterfaceNumber"), 16)?;
            let class =
                parse_number::<u8>(&self.sysfs_boundary, &path.join("bInterfaceClass"), 16)?;
            let subclass =
                parse_number::<u8>(&self.sysfs_boundary, &path.join("bInterfaceSubClass"), 16)?;
            let protocol =
                parse_number::<u8>(&self.sysfs_boundary, &path.join("bInterfaceProtocol"), 16)?;
            let mut endpoints = Vec::new();
            for endpoint_entry in fs::read_dir(&path).map_err(DriverError::from)? {
                let endpoint_entry = endpoint_entry.map_err(DriverError::from)?;
                let endpoint_path = endpoint_entry.path();
                if !endpoint_entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("ep_")
                    || !endpoint_entry
                        .file_type()
                        .map_err(DriverError::from)?
                        .is_dir()
                {
                    continue;
                }
                let address = parse_number::<u8>(
                    &self.sysfs_boundary,
                    &endpoint_path.join("bEndpointAddress"),
                    16,
                )?;
                let attributes = parse_number::<u8>(
                    &self.sysfs_boundary,
                    &endpoint_path.join("bmAttributes"),
                    16,
                )?;
                let max_packet_size = parse_number::<u16>(
                    &self.sysfs_boundary,
                    &endpoint_path.join("wMaxPacketSize"),
                    16,
                )?;
                let interval =
                    parse_number::<u8>(&self.sysfs_boundary, &endpoint_path.join("bInterval"), 10)?;
                let transfer_type = match attributes & 0x03 {
                    0 => TransferType::Control,
                    1 => TransferType::Isochronous,
                    2 => TransferType::Bulk,
                    _ => TransferType::Interrupt,
                };
                endpoints.push(EndpointInfo {
                    sysfs_path: endpoint_path,
                    address,
                    transfer_type,
                    max_packet_size,
                    interval,
                });
            }
            endpoints.sort_by_key(|endpoint| endpoint.address);
            let observation = self.descriptor_source.read_report_descriptor(&path)?;
            let (capability, report_descriptor_bytes, report_descriptor_source) =
                if let Some(observation) = observation {
                    let parsed = capabilities::parse(&observation.bytes)?;
                    (
                        Some(capabilities::CapabilityProfile::from_descriptor_for_class(
                            number, class, parsed,
                        )),
                        Some(observation.bytes),
                        Some(observation.source),
                    )
                } else {
                    (None, None, None)
                };
            interfaces.push(InterfaceInfo {
                sysfs_path: path,
                number,
                class,
                subclass,
                protocol,
                role: capability.as_ref().map_or_else(
                    || InterfaceRole::classify(Some(number), Some(class), None),
                    |value| value.interface_role,
                ),
                endpoints,
                descriptor: capability.as_ref().map(|value| value.descriptor.clone()),
                capabilities: capability,
                report_descriptor_bytes,
                report_descriptor_source,
            });
        }
        interfaces.sort_by_key(|interface| interface.number);
        Ok(interfaces)
    }
}

impl SysfsDiscovery {
    fn validate_instance_path(&self, instance: &DeviceInstance) -> Result<(), DriverError> {
        let canonical_boundary = self
            .sysfs_boundary
            .canonicalize()
            .map_err(DriverError::from)?;
        let canonical_instance = instance
            .sysfs_path
            .canonicalize()
            .map_err(DriverError::from)?;
        if !canonical_instance.starts_with(canonical_boundary) {
            return Err(DriverError::PermissionDenied(format!(
                "device instance escaped configured sysfs boundary: {}",
                instance.sysfs_path.display()
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sysfs_provider_uses_only_the_configured_roots() {
        let provider = SysfsDiscovery::new("/definitely/missing/th80-usb", "/missing/hidraw");
        assert!(matches!(provider.enumerate(), Err(DriverError::Io(_))));
    }

    #[test]
    fn optional_metadata_reader_rejects_unapproved_attribute_names() {
        let provider = SysfsDiscovery::new("/definitely/missing/th80-usb", "/missing/hidraw");
        let instance = DeviceInstance {
            identity: DeviceIdentity::TH80_PRO,
            sysfs_path: "/definitely/missing/th80-usb/1-2".into(),
        };
        assert!(matches!(
            provider.read_optional_device_attribute(&instance, "../../etc/passwd"),
            Err(DriverError::UnsupportedOperation(_))
        ));
    }
}
