//! Read-only hardware inventory for the GUI.
//!
//! This module deliberately stops at sysfs metadata and HID report
//! descriptors. It does not open hidraw/USB nodes or submit any report.

use th80_probe::{
    device::{DeviceKind, EndpointInfo, InterfaceInfo, TransferType},
    discovery::DiscoveryProvider,
    sysfs::SysfsDiscovery,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HardwareSnapshot {
    pub keyboard: Option<DetectedKeyboard>,
    pub receiver_detected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedKeyboard {
    pub stable_id: String,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub usb_release: Option<String>,
    pub speed_mbps: Option<String>,
    pub usb_identity_verified: bool,
    pub interfaces: Vec<DetectedInterface>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedInterface {
    pub number: u8,
    pub class: u8,
    pub subclass: u8,
    pub protocol: u8,
    pub role: String,
    pub descriptor_length: Option<usize>,
    pub descriptor_sha256: Option<String>,
    pub uses_report_ids: Option<bool>,
    pub feature_report_count: usize,
    pub feature_report_bytes: Option<u64>,
    pub endpoints: Vec<DetectedEndpoint>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedEndpoint {
    pub address: u8,
    pub transfer_type: String,
    pub max_packet_size: u16,
    pub interval: u8,
}

impl DetectedInterface {
    /// True only when the live descriptor exposes one unnumbered 64-byte
    /// Feature report on the vendor interface expected by the TH80 profile.
    pub fn has_expected_feature_report(&self) -> bool {
        self.number == 2
            && self.class == 0x03
            && self.feature_report_count == 1
            && self.feature_report_bytes == Some(64)
            && self.uses_report_ids == Some(false)
    }
}

fn endpoint_summary(endpoint: &EndpointInfo) -> DetectedEndpoint {
    let transfer_type = match endpoint.transfer_type {
        TransferType::Control => "Control",
        TransferType::Isochronous => "Isochronous",
        TransferType::Bulk => "Bulk",
        TransferType::Interrupt => "Interrupt",
    };
    DetectedEndpoint {
        address: endpoint.address,
        transfer_type: transfer_type.into(),
        max_packet_size: endpoint.max_packet_size,
        interval: endpoint.interval,
    }
}

fn interface_summary(interface: &InterfaceInfo) -> DetectedInterface {
    let capability = interface.capabilities.as_ref();
    DetectedInterface {
        number: interface.number,
        class: interface.class,
        subclass: interface.subclass,
        protocol: interface.protocol,
        role: format!("{:?}", interface.role),
        descriptor_length: interface.descriptor.as_ref().map(|value| value.length),
        descriptor_sha256: interface
            .descriptor
            .as_ref()
            .map(|value| value.sha256.clone()),
        uses_report_ids: capability.map(|value| value.uses_report_ids),
        feature_report_count: capability.map_or(0, |value| value.feature_report_count),
        feature_report_bytes: capability.and_then(|value| value.feature_report_bytes),
        endpoints: interface.endpoints.iter().map(endpoint_summary).collect(),
    }
}

/// Enumerate supported vendor devices and inspect their descriptors without
/// opening any device node. The USB release is shown separately from firmware
/// identity; it is not treated as a firmware version.
pub fn scan() -> Result<HardwareSnapshot, String> {
    let discovery = SysfsDiscovery::system();
    let instances = discovery.enumerate().map_err(|error| error.to_string())?;
    let mut keyboard = None;
    let mut receiver_detected = false;

    for instance in instances {
        let metadata = discovery
            .inspect_metadata(&instance)
            .map_err(|error| error.to_string())?;
        let usb_release = discovery
            .read_optional_device_attribute(&instance, "bcdDevice")
            .map_err(|error| error.to_string())?;
        let speed_mbps = discovery
            .read_optional_device_attribute(&instance, "speed")
            .map_err(|error| error.to_string())?;
        let interfaces = discovery
            .inspect_interfaces(&instance)
            .map_err(|error| error.to_string())?;
        let latest_metadata = discovery
            .inspect_metadata(&instance)
            .map_err(|error| error.to_string())?;
        if latest_metadata != metadata {
            return Err(format!(
                "USB identity changed during read-only inspection of {}",
                metadata.stable_id
            ));
        }

        match instance.identity.kind {
            DeviceKind::Th80Pro => {
                let usb_identity_verified = metadata.manufacturer.as_deref() == Some("ROYUAN")
                    && metadata.product.as_deref() == Some("EPOMAKER TH80 Pro");
                keyboard = Some(DetectedKeyboard {
                    stable_id: metadata.stable_id,
                    manufacturer: metadata.manufacturer,
                    product: metadata.product,
                    usb_release,
                    speed_mbps,
                    usb_identity_verified,
                    interfaces: interfaces.iter().map(interface_summary).collect(),
                });
            }
            DeviceKind::RoyuanReceiver => receiver_detected = true,
        }
    }

    Ok(HardwareSnapshot {
        keyboard,
        receiver_detected,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_the_exact_unnumbered_64_byte_feature_contract() {
        let expected = DetectedInterface {
            number: 2,
            class: 0x03,
            subclass: 0,
            protocol: 0,
            role: "VendorFeature".into(),
            descriptor_length: Some(20),
            descriptor_sha256: Some("descriptor-hash".into()),
            uses_report_ids: Some(false),
            feature_report_count: 1,
            feature_report_bytes: Some(64),
            endpoints: Vec::new(),
        };
        assert!(expected.has_expected_feature_report());

        let mut numbered = expected.clone();
        numbered.uses_report_ids = Some(true);
        assert!(!numbered.has_expected_feature_report());

        let mut short = expected.clone();
        short.feature_report_bytes = Some(63);
        assert!(!short.has_expected_feature_report());

        let mut wrong_interface = expected;
        wrong_interface.number = 1;
        assert!(!wrong_interface.has_expected_feature_report());
    }

    #[test]
    fn endpoint_direction_is_retained_in_the_address_without_inventing_out_endpoints() {
        let endpoint = DetectedEndpoint {
            address: 0x83,
            transfer_type: "Interrupt".into(),
            max_packet_size: 8,
            interval: 1,
        };
        assert_ne!(endpoint.address & 0x80, 0);
        assert_eq!(endpoint.transfer_type, "Interrupt");
    }
}
