//! Read-only device discovery and descriptor-source contracts.

use crate::{
    device::{DeviceInstance, DeviceMetadata, InterfaceInfo},
    error::DriverError,
};
use std::path::{Path, PathBuf};

/// A report descriptor read from a host-exposed, read-only source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportDescriptorObservation {
    pub source: PathBuf,
    pub bytes: Vec<u8>,
}

/// Reads descriptor bytes without opening a HID/USB device node or submitting reports.
pub trait DescriptorSource {
    fn read_report_descriptor(
        &self,
        interface_path: &Path,
    ) -> Result<Option<ReportDescriptorObservation>, DriverError>;
}

/// Read-only host discovery. Implementations may inspect OS metadata and
/// descriptor files, but must not open HID/USB device nodes or submit reports.
pub trait DiscoveryProvider {
    fn enumerate(&self) -> Result<Vec<DeviceInstance>, DriverError>;
    fn inspect_metadata(&self, instance: &DeviceInstance) -> Result<DeviceMetadata, DriverError>;
    fn inspect_interfaces(
        &self,
        instance: &DeviceInstance,
    ) -> Result<Vec<InterfaceInfo>, DriverError>;
}
