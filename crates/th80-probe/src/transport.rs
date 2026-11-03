//! Active device I/O only. Passive discovery lives in `discovery` and has no
//! transport dependency.

use crate::device::DeviceIdentity;
use crate::error::DriverError;
use crate::protocol::MacOsControlTransferSetup;

/// Active report operations remain disabled until command semantics and a
/// separate hardware authorization gate exist.
/// Low-level control-transfer boundary for an evidence-backed native USB
/// envelope. This trait has no production implementation in this workspace;
/// physical transport remains explicitly disabled.
pub trait ControlTransferBackend {
    fn control_transfer(
        &self,
        _identity: DeviceIdentity,
        _setup: MacOsControlTransferSetup,
        _data: &mut [u8],
    ) -> Result<usize, DriverError> {
        Err(DriverError::TransportDisabled("USB control transfer"))
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DisabledControlTransferBackend;

impl ControlTransferBackend for DisabledControlTransferBackend {}

pub trait Transport {
    fn feature_get(
        &self,
        _identity: DeviceIdentity,
        _interface: u8,
        _length: usize,
    ) -> Result<Vec<u8>, DriverError> {
        Err(DriverError::TransportDisabled("HID Feature GET"))
    }
    fn feature_set(
        &self,
        _identity: DeviceIdentity,
        _interface: u8,
        _report: &[u8],
    ) -> Result<(), DriverError> {
        Err(DriverError::TransportDisabled("HID Feature SET"))
    }
    fn input_read(
        &self,
        _identity: DeviceIdentity,
        _interface: u8,
        _length: usize,
    ) -> Result<Vec<u8>, DriverError> {
        Err(DriverError::TransportDisabled("HID Input read"))
    }
    fn output_write(
        &self,
        _identity: DeviceIdentity,
        _interface: u8,
        _report: &[u8],
    ) -> Result<(), DriverError> {
        Err(DriverError::TransportDisabled("HID Output write"))
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DisabledTransport;

impl Transport for DisabledTransport {}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct RecordingControlTransfer {
        call: std::cell::RefCell<Option<(DeviceIdentity, MacOsControlTransferSetup, Vec<u8>)>>,
        result: Result<usize, &'static str>,
        response_prefix: Vec<u8>,
    }

    impl ControlTransferBackend for RecordingControlTransfer {
        fn control_transfer(
            &self,
            identity: DeviceIdentity,
            setup: MacOsControlTransferSetup,
            data: &mut [u8],
        ) -> Result<usize, DriverError> {
            *self.call.borrow_mut() = Some((identity, setup, data.to_vec()));
            let copied = self.response_prefix.len().min(data.len());
            data[..copied].copy_from_slice(&self.response_prefix[..copied]);
            self.result
                .map_err(|message| DriverError::DeviceUnavailable(message.into()))
        }
    }

    #[test]
    fn mock_control_transfer_receives_exact_macos_set_setup_and_wire_bytes() {
        use crate::protocol::{
            build_macos_feature, DangleDeviceType, MacOsDeviceProfile, MACOS_FEATURE_SET_SETUP,
        };

        let mut packet = build_macos_feature(
            &[0x10, 1, 2, 3],
            MacOsDeviceProfile::MetadataSelector2,
            DangleDeviceType::None,
        );
        let expected_bytes = packet.transfer.bytes.to_vec();
        let mock = RecordingControlTransfer {
            call: std::cell::RefCell::new(None),
            result: Ok(64),
            response_prefix: Vec::new(),
        };
        assert_eq!(
            mock.control_transfer(
                DeviceIdentity::TH80_PRO,
                packet.setup,
                &mut packet.transfer.bytes,
            ),
            Ok(64)
        );
        let (identity, setup, data) = mock.call.into_inner().unwrap();
        assert_eq!(identity, DeviceIdentity::TH80_PRO);
        assert_eq!(setup, MACOS_FEATURE_SET_SETUP);
        assert_eq!(data, expected_bytes);
        assert_eq!(data.len(), 64);
    }

    #[test]
    fn mock_control_transfer_preserves_zero_filled_short_get_buffer() {
        use crate::protocol::MACOS_FEATURE_GET_SETUP;

        let mock = RecordingControlTransfer {
            call: std::cell::RefCell::new(None),
            result: Ok(5),
            response_prefix: vec![0xa0, 0xa1, 0xa2, 0xa3, 0xa4],
        };
        let mut buffer = [0_u8; 64];
        assert_eq!(
            mock.control_transfer(
                DeviceIdentity::TH80_PRO,
                MACOS_FEATURE_GET_SETUP,
                &mut buffer
            ),
            Ok(5)
        );
        assert_eq!(&buffer[..5], &[0xa0, 0xa1, 0xa2, 0xa3, 0xa4]);
        assert!(buffer[5..].iter().all(|byte| *byte == 0));
        let (_, setup, captured) = mock.call.into_inner().unwrap();
        assert_eq!(setup, MACOS_FEATURE_GET_SETUP);
        assert_eq!(captured.len(), 64);
    }

    #[test]
    fn mock_control_transfer_propagates_native_failures_and_default_stays_disabled() {
        use crate::protocol::MACOS_FEATURE_SET_SETUP;

        let failing = RecordingControlTransfer {
            call: std::cell::RefCell::new(None),
            result: Err("mock libusb failure"),
            response_prefix: Vec::new(),
        };
        let mut buffer = [0_u8; 64];
        assert!(matches!(
            failing.control_transfer(
                DeviceIdentity::TH80_PRO,
                MACOS_FEATURE_SET_SETUP,
                &mut buffer
            ),
            Err(DriverError::DeviceUnavailable(_))
        ));

        let disabled = DisabledControlTransferBackend;
        assert!(matches!(
            disabled.control_transfer(
                DeviceIdentity::TH80_PRO,
                MACOS_FEATURE_SET_SETUP,
                &mut buffer
            ),
            Err(DriverError::TransportDisabled(_))
        ));
    }

    #[test]
    fn every_active_operation_is_explicitly_disabled() {
        let transport = DisabledTransport;
        let identity = DeviceIdentity::TH80_PRO;
        assert!(matches!(
            transport.feature_get(identity, 2, 64),
            Err(DriverError::TransportDisabled(_))
        ));
        assert!(matches!(
            transport.feature_set(identity, 2, &[0; 64]),
            Err(DriverError::TransportDisabled(_))
        ));
        assert!(matches!(
            transport.input_read(identity, 0, 8),
            Err(DriverError::TransportDisabled(_))
        ));
        assert!(matches!(
            transport.output_write(identity, 0, &[0]),
            Err(DriverError::TransportDisabled(_))
        ));
    }
}
