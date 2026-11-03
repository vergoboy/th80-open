//! Static packet layouts recovered in Phase 5D. Builders are pure memory
//! transformations and never submit reports to an operating-system API.

use crate::error::DriverError;

pub const FEATURE_REPORT_BYTES: usize = 64;
pub const MACOS_LOGICAL_MIN_BYTES: usize = 66;
pub const WINDOWS_LOGICAL_MIN_BYTES: usize = 65;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportId {
    ImplicitZero,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlTransferDirection {
    HostToDevice,
    DeviceToHost,
}

/// Recovered macOS libusb Feature-control setup. `index` is the literal USB
/// setup-packet wIndex; it is not the sysfs interface number retained by
/// passive discovery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacOsControlTransferSetup {
    pub direction: ControlTransferDirection,
    pub request_type: u8,
    pub request: u8,
    pub value: u16,
    pub index: u16,
    pub length: u16,
    pub timeout_ms: u32,
}

pub const MACOS_FEATURE_SET_SETUP: MacOsControlTransferSetup = MacOsControlTransferSetup {
    direction: ControlTransferDirection::HostToDevice,
    request_type: 0x21,
    request: 0x09,
    value: 0x0300,
    index: 0,
    length: FEATURE_REPORT_BYTES as u16,
    timeout_ms: 2_000,
};

pub const MACOS_FEATURE_GET_SETUP: MacOsControlTransferSetup = MacOsControlTransferSetup {
    direction: ControlTransferDirection::DeviceToHost,
    request_type: 0xa1,
    request: 0x01,
    value: 0x0300,
    index: 0,
    length: FEATURE_REPORT_BYTES as u16,
    timeout_ms: 2_000,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
/// Renderer protobuf `CheckSumType`. This is request metadata, not a
/// transformation performed by `build_macos_feature`; the ordinary
/// `sendMsg` native consumer is traced independently.
pub enum FrontendChecksumType {
    /// Protobuf `CheckSumType::BIT7` (numeric value 0).
    Bit7 = 0,
    /// Protobuf `CheckSumType::BIT8` (numeric value 1).
    Bit8 = 1,
    /// Protobuf `CheckSumType::NONE` (numeric value 2).
    None = 2,
}

/// Protobuf `DangleDevType`, a different field and enum from
/// `FrontendChecksumType`. The macOS `HidLibusb::send_feature` SET method's
/// effect is recovered independently: NONE inserts at offset 7, KEYBOARD at
/// offset 8, MOUSE skips insertion for metadata selector `+0x3A == 2`. This
/// does not establish how ordinary gRPC `sendMsg` reaches that method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DangleDeviceType {
    None = 0,
    Keyboard = 1,
    Mouse = 2,
}

/// Exact frontend `SendMsg` payload metadata. The message is the caller's
/// original 64-byte Buffer; checksum selection and Dangle classification are
/// independent protobuf fields and are not applied here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontendSendRequest {
    pub message: [u8; FEATURE_REPORT_BYTES],
    pub checksum_type: FrontendChecksumType,
    pub dangle_device_type: DangleDeviceType,
}

impl FrontendSendRequest {
    pub const fn new(
        message: [u8; FEATURE_REPORT_BYTES],
        checksum_type: FrontendChecksumType,
        dangle_device_type: DangleDeviceType,
    ) -> Self {
        Self {
            message,
            checksum_type,
            dangle_device_type,
        }
    }
}

/// The Phase 5D macOS checksum branch was recovered for metadata selector
/// `device+0x3a == 2` only. Other metadata selectors are intentionally absent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacOsDeviceProfile {
    MetadataSelector2,
}

impl TryFrom<u8> for DangleDeviceType {
    type Error = DriverError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Keyboard),
            2 => Ok(Self::Mouse),
            _ => Err(DriverError::UnsupportedOperation(
                "unverified dangledevtype value",
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureEnvelope {
    pub report_id: ReportId,
    pub bytes: [u8; FEATURE_REPORT_BYTES],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacOsFeaturePacket {
    /// Logical BytesMut after append, resize and checksum; may exceed 66 bytes.
    pub logical_buffer: Vec<u8>,
    /// The exact first 64 bytes passed to the recovered libusb SET method.
    pub transfer: FeatureEnvelope,
    /// Literal setup fields used by the recovered libusb SET method.
    pub setup: MacOsControlTransferSetup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandSemantics {
    Unknown,
}

impl FeatureEnvelope {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DriverError> {
        if bytes.len() != FEATURE_REPORT_BYTES {
            return Err(DriverError::InvalidPacketLength {
                expected: "exactly 64 bytes".into(),
                actual: bytes.len(),
            });
        }
        let mut packet = [0; FEATURE_REPORT_BYTES];
        packet.copy_from_slice(bytes);
        Ok(Self {
            report_id: ReportId::ImplicitZero,
            bytes: packet,
        })
    }
}

/// Reconstructs the recovered macOS `HidLibusb::send_feature` logical
/// `BytesMut`: append caller bytes, resize to at least 66 bytes with zero fill,
/// apply the DangleDeviceType branch, then expose its first 64 bytes. Do not
/// assume ordinary gRPC `sendMsg` uses this path; its vtable target is unknown.
pub fn build_macos_feature(
    message: &[u8],
    _profile: MacOsDeviceProfile,
    selector: DangleDeviceType,
) -> MacOsFeaturePacket {
    let mut logical = message.to_vec();
    logical.resize(logical.len().max(MACOS_LOGICAL_MIN_BYTES), 0);
    match selector {
        DangleDeviceType::None => logical[7] = complemented_sum(&logical[1..=7]),
        DangleDeviceType::Keyboard => logical[8] = complemented_sum(&logical[1..=8]),
        DangleDeviceType::Mouse => {}
    }
    let transfer = FeatureEnvelope::from_bytes(&logical[..FEATURE_REPORT_BYTES])
        .expect("macOS logical buffer is always at least 66 bytes");
    MacOsFeaturePacket {
        logical_buffer: logical,
        transfer,
        setup: MACOS_FEATURE_SET_SETUP,
    }
}

/// Mirrors the macOS GET response behavior without invoking libusb: the
/// original 64-byte zero-filled allocation is retained after a short read.
pub fn simulate_macos_get_response(
    received: &[u8],
    transfer_count: i32,
) -> Result<FeatureEnvelope, DriverError> {
    if transfer_count < 0 {
        return Err(DriverError::DeviceUnavailable(format!(
            "simulated libusb GET error {transfer_count}"
        )));
    }
    let count = transfer_count as usize;
    if count > FEATURE_REPORT_BYTES || count > received.len() {
        return Err(DriverError::MalformedReport(format!(
            "simulated GET count {count} exceeds supplied bytes ({}) or 64-byte buffer",
            received.len()
        )));
    }
    let mut buffer = [0_u8; FEATURE_REPORT_BYTES];
    buffer[..count].copy_from_slice(&received[..count]);
    FeatureEnvelope::from_bytes(&buffer)
}

/// Deliberately does not decode command IDs or response fields. Phase 5D did
/// not establish any command grammar for the 64-byte payload.
pub fn command_semantics(_packet: &FeatureEnvelope) -> CommandSemantics {
    CommandSemantics::Unknown
}

/// Reconstructs the separate Windows HIDAPI caller Vec. The leading 0x01 is
/// preserved as a literal prefix; its Report ID meaning is not asserted.
pub fn build_windows_feature(payload: &[u8], selector: u8) -> Result<Vec<u8>, DriverError> {
    if selector > 1 {
        return Err(DriverError::UnsupportedOperation(
            "unverified Windows selector",
        ));
    }
    let mut packet = Vec::with_capacity((payload.len() + 1).max(WINDOWS_LOGICAL_MIN_BYTES));
    packet.push(0x01);
    packet.extend_from_slice(payload);
    packet.resize(packet.len().max(WINDOWS_LOGICAL_MIN_BYTES), 0);
    let checksum = complemented_sum(payload);
    packet[if selector == 0 { 8 } else { 9 }] = checksum;
    Ok(packet)
}

pub(crate) fn complemented_sum(bytes: &[u8]) -> u8 {
    !bytes.iter().fold(0_u8, |sum, byte| sum.wrapping_add(*byte))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontend_send_request_preserves_the_original_buffer_and_separate_fields() {
        let mut message = [0_u8; FEATURE_REPORT_BYTES];
        message[0] = 0x07;
        message[5..8].copy_from_slice(&[0x11, 0x22, 0x33]);
        let request =
            FrontendSendRequest::new(message, FrontendChecksumType::Bit8, DangleDeviceType::None);
        assert_eq!(request.message, message);
        assert_eq!(request.checksum_type, FrontendChecksumType::Bit8);
        assert_eq!(request.dangle_device_type, DangleDeviceType::None);
    }

    #[test]
    fn raw_macos_feature_builder_uses_dangle_type_not_frontend_checksum_metadata() {
        let mut message = [0_u8; FEATURE_REPORT_BYTES];
        message[0] = 0x07;
        message[5..8].copy_from_slice(&[0x11, 0x22, 0x33]);
        let bit7 =
            FrontendSendRequest::new(message, FrontendChecksumType::Bit7, DangleDeviceType::None);
        let bit8 =
            FrontendSendRequest::new(message, FrontendChecksumType::Bit8, DangleDeviceType::None);
        let native_bit7 = build_macos_feature(
            &bit7.message,
            MacOsDeviceProfile::MetadataSelector2,
            bit7.dangle_device_type,
        );
        let native_bit8 = build_macos_feature(
            &bit8.message,
            MacOsDeviceProfile::MetadataSelector2,
            bit8.dangle_device_type,
        );
        assert_eq!(native_bit7.transfer.bytes, native_bit8.transfer.bytes);
        assert_eq!(
            native_bit7.transfer.bytes[7],
            complemented_sum(&message[1..=7])
        );
    }

    #[test]
    fn macos_envelope_is_exactly_64_bytes_with_implicit_report_zero() {
        let packet = build_macos_feature(
            &[0x10, 1, 2],
            MacOsDeviceProfile::MetadataSelector2,
            DangleDeviceType::Mouse,
        );
        assert_eq!(packet.logical_buffer.len(), 66);
        assert_eq!(packet.transfer.bytes.len(), 64);
        assert_eq!(packet.transfer.report_id, ReportId::ImplicitZero);
        assert_eq!(packet.setup, MACOS_FEATURE_SET_SETUP);
        assert_eq!(&packet.transfer.bytes[..3], &[0x10, 1, 2]);
        assert!(packet.transfer.bytes[3..].iter().all(|byte| *byte == 0));
    }

    #[test]
    fn macos_checksum_branches_match_recovered_pre_store_windows() {
        let input: Vec<u8> = (0..12).collect();
        let type0 = build_macos_feature(
            &input,
            MacOsDeviceProfile::MetadataSelector2,
            DangleDeviceType::None,
        );
        let type1 = build_macos_feature(
            &input,
            MacOsDeviceProfile::MetadataSelector2,
            DangleDeviceType::Keyboard,
        );
        assert_eq!(type0.transfer.bytes[7], !(1_u8 + 2 + 3 + 4 + 5 + 6 + 7));
        assert_eq!(type1.transfer.bytes[8], !(1_u8 + 2 + 3 + 4 + 5 + 6 + 7 + 8));
        assert_eq!(
            build_macos_feature(
                &input,
                MacOsDeviceProfile::MetadataSelector2,
                DangleDeviceType::Mouse
            )
            .transfer
            .bytes[7],
            7
        );
    }

    #[test]
    fn macos_checksum_includes_zero_filled_destination_when_message_is_short() {
        let packet = build_macos_feature(
            &[0xaa],
            MacOsDeviceProfile::MetadataSelector2,
            DangleDeviceType::None,
        );
        assert_eq!(packet.transfer.bytes[7], 0xff);
    }

    #[test]
    fn macos_feature_control_setup_preserves_direction_index_length_and_timeout() {
        assert_eq!(
            MACOS_FEATURE_SET_SETUP.direction,
            ControlTransferDirection::HostToDevice
        );
        assert_eq!(MACOS_FEATURE_SET_SETUP.request_type, 0x21);
        assert_eq!(MACOS_FEATURE_SET_SETUP.request, 0x09);
        assert_eq!(MACOS_FEATURE_SET_SETUP.value, 0x0300);
        assert_eq!(MACOS_FEATURE_SET_SETUP.index, 0);
        assert_eq!(MACOS_FEATURE_SET_SETUP.length, 64);
        assert_eq!(MACOS_FEATURE_SET_SETUP.timeout_ms, 2_000);

        assert_eq!(
            MACOS_FEATURE_GET_SETUP.direction,
            ControlTransferDirection::DeviceToHost
        );
        assert_eq!(MACOS_FEATURE_GET_SETUP.request_type, 0xa1);
        assert_eq!(MACOS_FEATURE_GET_SETUP.request, 0x01);
        assert_eq!(MACOS_FEATURE_GET_SETUP.value, 0x0300);
        assert_eq!(MACOS_FEATURE_GET_SETUP.index, 0);
        assert_eq!(MACOS_FEATURE_GET_SETUP.length, 64);
        assert_eq!(MACOS_FEATURE_GET_SETUP.timeout_ms, 2_000);
    }

    #[test]
    fn frontend_checksum_and_dangle_device_enums_are_distinct_protocol_fields() {
        assert_eq!(FrontendChecksumType::Bit7 as u8, 0);
        assert_eq!(FrontendChecksumType::Bit8 as u8, 1);
        assert_eq!(FrontendChecksumType::None as u8, 2);

        assert_eq!(DangleDeviceType::None as u8, 0);
        assert_eq!(DangleDeviceType::Keyboard as u8, 1);
        assert_eq!(DangleDeviceType::Mouse as u8, 2);

        assert_eq!(
            DangleDeviceType::try_from(0).unwrap(),
            DangleDeviceType::None
        );
        assert_eq!(
            DangleDeviceType::try_from(1).unwrap(),
            DangleDeviceType::Keyboard
        );
        assert_eq!(
            DangleDeviceType::try_from(2).unwrap(),
            DangleDeviceType::Mouse
        );
        assert!(DangleDeviceType::try_from(3).is_err());
    }

    #[test]
    fn windows_builder_keeps_prefix_length_and_checksum_domains_separate() {
        let payload: Vec<u8> = (0..12).collect();
        let type0 = build_windows_feature(&payload, 0).unwrap();
        let type1 = build_windows_feature(&payload, 1).unwrap();
        assert_eq!(type0.len(), 65);
        assert_eq!(type0[0], 1);
        assert_eq!(type0[8], complemented_sum(&payload));
        assert_eq!(type1[9], complemented_sum(&payload));
        assert_eq!(build_windows_feature(&[1], 0).unwrap().len(), 65);
        assert_eq!(build_windows_feature(&[0; 70], 0).unwrap().len(), 71);
        assert!(build_windows_feature(&[], 2).is_err());
    }

    #[test]
    fn rejects_non_64_byte_feature_envelopes() {
        assert!(matches!(
            FeatureEnvelope::from_bytes(&[0; 63]),
            Err(DriverError::InvalidPacketLength { .. })
        ));
        assert!(FeatureEnvelope::from_bytes(&[0; 64]).is_ok());
    }
}
