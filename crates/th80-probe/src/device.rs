use serde::Serialize;
use std::path::PathBuf;
pub use th80_core::{DeviceIdentity, DeviceKind};

pub const VENDOR_ID: u16 = 0x3151;
pub const TH80_PRO_PID: u16 = 0x4010;
pub const ROYUAN_RECEIVER_PID: u16 = 0x4011;

/// One observed sysfs instance. Its path is ephemeral and must not be persisted as identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInstance {
    pub identity: DeviceIdentity,
    pub sysfs_path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InterfaceRole {
    BootKeyboard,
    ExtendedInput,
    VendorFeature,
    Other,
}

impl InterfaceRole {
    pub fn classify(
        number: Option<u8>,
        class: Option<u8>,
        feature_report_bytes: Option<u64>,
    ) -> Self {
        if class != Some(0x03) {
            return Self::Other;
        }
        match number {
            Some(0) => Self::BootKeyboard,
            Some(1) => Self::ExtendedInput,
            Some(2) if feature_report_bytes == Some(64) => Self::VendorFeature,
            _ => Self::Other,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EndpointInfo {
    pub sysfs_path: PathBuf,
    pub address: u8,
    pub transfer_type: TransferType,
    pub max_packet_size: u16,
    pub interval: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferType {
    Control,
    Isochronous,
    Bulk,
    Interrupt,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DescriptorFingerprint {
    pub sha256: String,
    pub length: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct InterfaceInfo {
    pub sysfs_path: PathBuf,
    pub number: u8,
    pub class: u8,
    pub subclass: u8,
    pub protocol: u8,
    pub role: InterfaceRole,
    pub endpoints: Vec<EndpointInfo>,
    pub descriptor: Option<DescriptorFingerprint>,
    pub capabilities: Option<crate::capabilities::CapabilityProfile>,
    #[serde(skip)]
    pub report_descriptor_bytes: Option<Vec<u8>>,
    #[serde(skip)]
    pub report_descriptor_source: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeviceMetadata {
    pub identity: DeviceIdentity,
    pub stable_id: String,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub serial_number: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifies_only_the_two_verified_usb_ids() {
        assert_eq!(
            DeviceIdentity::from_vid_pid(0x3151, 0x4010),
            Some(DeviceIdentity::TH80_PRO)
        );
        assert_eq!(
            DeviceIdentity::from_vid_pid(0x3151, 0x4011),
            Some(DeviceIdentity::ROYUAN_RECEIVER)
        );
        assert_eq!(DeviceIdentity::from_vid_pid(0x3151, 0x4001), None);
    }

    #[test]
    fn classifies_captured_hid_interfaces_without_guessing_feature_semantics() {
        assert_eq!(
            InterfaceRole::classify(Some(0), Some(3), None),
            InterfaceRole::BootKeyboard
        );
        assert_eq!(
            InterfaceRole::classify(Some(1), Some(3), None),
            InterfaceRole::ExtendedInput
        );
        assert_eq!(
            InterfaceRole::classify(Some(2), Some(3), Some(64)),
            InterfaceRole::VendorFeature
        );
        assert_eq!(
            InterfaceRole::classify(Some(2), Some(3), None),
            InterfaceRole::Other
        );
        assert_eq!(
            InterfaceRole::classify(Some(2), Some(8), Some(64)),
            InterfaceRole::Other
        );
    }
}
