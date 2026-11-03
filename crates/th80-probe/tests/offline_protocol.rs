//! Offline-only regression harness. No test imports or constructs an active
//! USB/HID backend; all protocol inputs are in-memory synthetic byte slices.

use th80_probe::capabilities::{parse, ReportType};
use th80_probe::error::DriverError;
use th80_probe::protocol::{
    build_macos_feature, build_windows_feature, command_semantics, simulate_macos_get_response,
    CommandSemantics, DangleDeviceType, FeatureEnvelope, MacOsDeviceProfile, ReportId,
    FEATURE_REPORT_BYTES, MACOS_LOGICAL_MIN_BYTES, WINDOWS_LOGICAL_MIN_BYTES,
};
use th80_probe::transport::{DisabledTransport, Transport};
use th80_probe::DeviceIdentity;

fn hex_bytes(input: &str) -> Vec<u8> {
    input
        .split_whitespace()
        .map(|byte| u8::from_str_radix(byte, 16).expect("valid fixture hex"))
        .collect()
}

fn checksum(bytes: &[u8]) -> u8 {
    !bytes.iter().fold(0_u8, |sum, byte| sum.wrapping_add(*byte))
}

fn macos(message: &[u8], kind: DangleDeviceType) -> Vec<u8> {
    build_macos_feature(message, MacOsDeviceProfile::MetadataSelector2, kind).logical_buffer
}

#[test]
fn captured_descriptor_fixture_is_not_mistaken_for_a_packet_capture() {
    let descriptor = hex_bytes(include_str!("fixtures/captured_interface2_descriptor.hex"));
    let parsed = parse(&descriptor).unwrap();
    assert_eq!(parsed.descriptor_length, 20);
    assert_eq!(
        parsed.sha256,
        "350fea62e225e3d92e9db3bbbd408fcebe386c10405454be661d3d2c3229b0b5"
    );
    assert_eq!(parsed.reports.len(), 1);
    assert_eq!(parsed.reports[0].report_type, ReportType::Feature);
    assert_eq!(parsed.reports[0].transfer_bytes, 64);
    assert!(!parsed.uses_report_ids);
}

#[test]
fn macos_logical_and_wire_lengths_hold_at_all_boundaries() {
    for length in [0, 1, 6, 7, 8, 9, 63, 64, 65, 66, 67, 127, 128] {
        let input: Vec<u8> = (0..length).map(|n| n as u8).collect();
        let mut pre_checksum = input.clone();
        pre_checksum.resize(length.max(MACOS_LOGICAL_MIN_BYTES), 0);
        for kind in [
            DangleDeviceType::None,
            DangleDeviceType::Keyboard,
            DangleDeviceType::Mouse,
        ] {
            let packet = build_macos_feature(&input, MacOsDeviceProfile::MetadataSelector2, kind);
            assert_eq!(
                packet.logical_buffer.len(),
                length.max(MACOS_LOGICAL_MIN_BYTES)
            );
            assert_eq!(packet.transfer.bytes.len(), FEATURE_REPORT_BYTES);
            assert_eq!(packet.transfer.report_id, ReportId::ImplicitZero);
            assert_eq!(
                &packet.transfer.bytes[..],
                &packet.logical_buffer[..FEATURE_REPORT_BYTES]
            );
            let checksum_offset = match kind {
                DangleDeviceType::None => {
                    assert_eq!(packet.logical_buffer[7], checksum(&pre_checksum[1..=7]));
                    Some(7)
                }
                DangleDeviceType::Keyboard => {
                    assert_eq!(packet.logical_buffer[8], checksum(&pre_checksum[1..=8]));
                    Some(8)
                }
                DangleDeviceType::Mouse => {
                    assert_eq!(packet.logical_buffer, pre_checksum);
                    None
                }
            };
            if length < MACOS_LOGICAL_MIN_BYTES {
                for (offset, byte) in packet.logical_buffer.iter().enumerate().skip(length) {
                    if Some(offset) != checksum_offset {
                        assert_eq!(
                            *byte, 0,
                            "kind={kind:?}, payload_len={length}, offset={offset}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn macos_checksum_branches_have_exact_recovered_windows_and_offsets() {
    let payload: Vec<u8> = (0..20).map(|n| (n * 7) as u8).collect();
    let type0 = macos(&payload, DangleDeviceType::None);
    let type1 = macos(&payload, DangleDeviceType::Keyboard);
    let type2 = macos(&payload, DangleDeviceType::Mouse);

    assert_eq!(type0[7], checksum(&payload[1..=7]));
    assert_eq!(type1[8], checksum(&payload[1..=8]));
    assert_eq!(type2, {
        let mut expected = payload.clone();
        expected.resize(expected.len().max(MACOS_LOGICAL_MIN_BYTES), 0);
        expected
    });
    assert_eq!(type0[8], payload[8]);
    assert_eq!(type1[7], payload[7]);
}

#[test]
fn macos_checksum_includes_pre_store_destination_and_zero_fill() {
    let mut with_destination = vec![0x55; 12];
    with_destination[7] = 0x23;
    with_destination[8] = 0x45;
    let type0 = macos(&with_destination, DangleDeviceType::None);
    let type1 = macos(&with_destination, DangleDeviceType::Keyboard);
    assert_eq!(type0[7], checksum(&with_destination[1..=7]));
    assert_eq!(type1[8], checksum(&with_destination[1..=8]));

    let short = macos(&[0xaa], DangleDeviceType::None);
    assert_eq!(short[7], checksum(&[0, 0, 0, 0, 0, 0, 0]));
    assert_eq!(short[0], 0xaa);
}

#[test]
fn windows_prefix_length_and_payload_checksum_are_independent() {
    for length in [0, 1, 7, 8, 9, 63, 64, 65, 66, 127] {
        let payload: Vec<u8> = (0..length).map(|n| (n * 11) as u8).collect();
        for selector in [0, 1] {
            let packet = build_windows_feature(&payload, selector).unwrap();
            assert_eq!(packet.len(), (length + 1).max(WINDOWS_LOGICAL_MIN_BYTES));
            assert_eq!(packet[0], 0x01);
            let checksum_offset = if selector == 0 { 8 } else { 9 };
            for (payload_index, expected) in payload.iter().enumerate() {
                let packet_offset = payload_index + 1;
                if packet_offset != checksum_offset {
                    assert_eq!(
                        packet[packet_offset], *expected,
                        "selector={selector}, payload_len={length}, offset={packet_offset}"
                    );
                }
            }
            assert_eq!(packet[checksum_offset], checksum(&payload));
            if length + 1 < WINDOWS_LOGICAL_MIN_BYTES {
                for (index, byte) in packet.iter().enumerate().skip(length + 1) {
                    if index != checksum_offset {
                        assert_eq!(
                            *byte, 0,
                            "selector={selector}, payload_len={length}, offset={index}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn windows_checksum_uses_complete_payload_not_macos_fixed_window() {
    let payload: Vec<u8> = (0..40).map(|n| n as u8).collect();
    let win = build_windows_feature(&payload, 0).unwrap();
    let mac = build_macos_feature(
        &payload,
        MacOsDeviceProfile::MetadataSelector2,
        DangleDeviceType::None,
    );
    assert_eq!(win[8], checksum(&payload));
    assert_eq!(mac.transfer.bytes[7], checksum(&payload[1..=7]));
    assert_ne!(win[8], mac.transfer.bytes[7]);
    assert_ne!(win.len(), mac.transfer.bytes.len());
}

#[test]
fn macos_get_short_transfer_preserves_zero_filled_64_byte_buffer() {
    for count in [0_usize, 1, 7, 8, 31, 63, 64] {
        let received: Vec<u8> = (0..count).map(|n| (0xa0 + n) as u8).collect();
        let packet = simulate_macos_get_response(&received, count as i32).unwrap();
        assert_eq!(packet.bytes.len(), 64);
        assert_eq!(&packet.bytes[..count], received.as_slice());
        assert!(packet.bytes[count..].iter().all(|byte| *byte == 0));
        assert_eq!(packet.report_id, ReportId::ImplicitZero);
    }
}

#[test]
fn macos_get_simulation_rejects_negative_or_impossible_transfer_counts() {
    assert!(matches!(
        simulate_macos_get_response(&[], -9),
        Err(DriverError::DeviceUnavailable(_))
    ));
    assert!(matches!(
        simulate_macos_get_response(&[1, 2], 3),
        Err(DriverError::MalformedReport(_))
    ));
    assert!(matches!(
        simulate_macos_get_response(&[0; 65], 65),
        Err(DriverError::MalformedReport(_))
    ));
}

#[test]
fn malformed_and_truncated_feature_envelopes_are_rejected() {
    for length in [0, 1, 62, 63, 65, 66, 128] {
        let input = vec![0; length];
        assert!(matches!(
            FeatureEnvelope::from_bytes(&input),
            Err(DriverError::InvalidPacketLength { .. })
        ));
    }
    assert!(FeatureEnvelope::from_bytes(&[0; 64]).is_ok());
    assert!(parse(&[0x75]).is_err());
    assert!(parse(&[0xfe, 0x02, 0x99, 0xaa]).is_err());
}

#[test]
fn unverified_selector_values_are_not_interpreted() {
    assert!(DangleDeviceType::try_from(3).is_err());
    assert!(build_windows_feature(&[1, 2, 3], 2).is_err());
}

#[test]
fn arbitrary_feature_bytes_never_become_known_command_semantics() {
    for first_byte in [0x00, 0x01, 0x02, 0x7f, 0x80, 0xff] {
        let mut bytes = [0_u8; 64];
        bytes[0] = first_byte;
        let envelope = FeatureEnvelope::from_bytes(&bytes).unwrap();
        assert_eq!(command_semantics(&envelope), CommandSemantics::Unknown);
    }
}

#[test]
fn offline_harness_uses_disabled_transport_only() {
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
