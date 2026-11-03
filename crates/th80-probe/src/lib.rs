//! Hardware-independent TH80 driver foundation.
//!
//! This crate deliberately contains no USB/HID handle implementation. Every
//! operation that could communicate with a device is disabled by contract.

pub mod capabilities;
pub mod device;
pub mod discovery;
pub mod error;
#[path = "hid.rs"]
mod hid_parser;
pub mod protocol;
pub mod sysfs;
pub mod transport;
pub mod yc500;

pub use device::{DeviceIdentity, DeviceInstance, DeviceKind, EndpointInfo, InterfaceInfo};
pub use error::DriverError;
