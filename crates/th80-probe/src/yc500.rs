//! Offline-only command contracts for the EPOMAKER TH80 (`3151:4010`)
//! yc500 protocol family. This module only constructs and validates bytes;
//! it has no transport and cannot communicate with a device.
//!
//! The family assignment is corroborated by the public ROYUAN registry's
//! EPOMAKER TH80 record (record 742, VID:PID 3151:4010). The command layouts
//! are family-level evidence, not a claim that every firmware revision has
//! been exercised. Active transport remains disabled in `transport`.

use crate::error::DriverError;
use crate::protocol::{DangleDeviceType, FrontendChecksumType, FrontendSendRequest};

pub const FEATURE_REPORT_BYTES: usize = 64;
pub const TH80_PROFILE_COUNT: u8 = 4;
pub const MACRO_SLOT_COUNT: u8 = 50;
pub const KEYMAP_PAGE_COUNT: u8 = 8;
pub const MACRO_PAGE_COUNT: u8 = 4;
pub const USER_PICTURE_BYTES: usize = 384;
pub const USER_PICTURE_PAGE_BYTES: usize = 56;
pub const MACRO_BYTES: usize = 256;
pub const MACRO_WRITE_PAGE_BYTES: usize = 56;
pub const MACRO_READ_PAGE_COUNT: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Generic yc500 family commands. Variants such as Fn page GET are not
/// automatically supported by the exact TH80 V504 frontend.
pub enum Yc500Command {
    FirmwareRevision,
    ActiveProfile,
    KeyboardOptions { profile: u8 },
    LightingParameters,
    KeymapPage { profile: u8, page: u8 },
    FnLayerPage { profile: u8, page: u8 },
    MacroPage { slot: u8, page: u8 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Yc500Response {
    FirmwareRevision(u16),
    ActiveProfile(u8),
    KeyboardOptions {
        flags: u8,
        fn_matrix: u8,
        power_save: u8,
    },
    LightingParameters {
        mode: u8,
        speed: u8,
        brightness: u8,
        option_flags: u8,
        red: u8,
        green: u8,
        blue: u8,
    },
    RawPage([u8; FEATURE_REPORT_BYTES]),
}

/// Commands exposed by the exact EPOMAKER V504 `yc500_th80` frontend class.
/// Kept separate from the generic yc500 family because opcode 0x83 and the
/// report-rate command differ across public protocol dialects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Th80V504Command {
    ReportRate { profile: u8 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Th80V504Response {
    ReportRateHz(u16),
}

impl Th80V504Command {
    /// Encode the exact 64-byte caller Buffer and its separate SendMsg metadata.
    /// CheckSumType::BIT7 is forwarded as protobuf metadata; no checksum byte
    /// is inserted into the caller Buffer here.
    pub fn encode(self) -> Result<FrontendSendRequest, DriverError> {
        let mut packet = [0_u8; FEATURE_REPORT_BYTES];
        match self {
            Self::ReportRate { profile } => {
                validate_profile(profile)?;
                packet[0] = 0x84;
                packet[1] = profile;
            }
        }
        Ok(FrontendSendRequest::new(
            packet,
            FrontendChecksumType::Bit7,
            DangleDeviceType::None,
        ))
    }

    /// Decode only the fields consumed by the V504 frontend. Its parser does
    /// not validate an echoed opcode, so this codec does not invent one.
    pub fn decode(self, bytes: &[u8]) -> Result<Th80V504Response, DriverError> {
        let _ = self.encode()?;
        if bytes.len() != FEATURE_REPORT_BYTES {
            return Err(DriverError::InvalidPacketLength {
                expected: "exactly 64-byte TH80 V504 Feature response".into(),
                actual: bytes.len(),
            });
        }
        match self {
            Self::ReportRate { .. } => {
                let rate = match bytes[2] {
                    1 => 1000,
                    2 => 500,
                    4 => 250,
                    8 => 125,
                    value => {
                        return Err(DriverError::MalformedReport(format!(
                            "TH80 V504 report-rate code {value} is not one of 1, 2, 4, 8"
                        )))
                    }
                };
                Ok(Th80V504Response::ReportRateHz(rate))
            }
        }
    }
}

/// SET_REPORT (`0x04`) rate mapping; CheckSumType::BIT7 is separate metadata.
pub fn encode_th80_v504_set_report_rate(
    profile: u8,
    rate_hz: u16,
) -> Result<FrontendSendRequest, DriverError> {
    validate_profile(profile)?;
    let code = match rate_hz {
        1000 => 1,
        500 => 2,
        250 => 4,
        125 => 8,
        _ => {
            return Err(DriverError::UnsupportedOperation(
                "TH80 V504 report rate must be 125, 250, 500, or 1000 Hz",
            ))
        }
    };
    let mut packet = [0_u8; FEATURE_REPORT_BYTES];
    packet[0] = 0x04;
    packet[1] = profile;
    packet[2] = code;
    Ok(FrontendSendRequest::new(
        packet,
        FrontendChecksumType::Bit7,
        DangleDeviceType::None,
    ))
}

/// YC500Common's single-slot key setter (`0x13`); the four-byte value is
/// already in the vendor matrix representation and is not reinterpreted.
pub fn encode_th80_key_slot_write(
    profile: u8,
    slot: u8,
    value: [u8; 4],
) -> Result<FrontendSendRequest, DriverError> {
    validate_profile(profile)?;
    validate_slot(slot)?;
    let mut packet = [0_u8; FEATURE_REPORT_BYTES];
    packet[0] = 0x13;
    packet[1] = profile;
    packet[2] = slot;
    packet[8..12].copy_from_slice(&value);
    Ok(FrontendSendRequest::new(
        packet,
        FrontendChecksumType::Bit7,
        DangleDeviceType::None,
    ))
}

/// YC500Common's single-slot Fn setter (`0x15`). The TH80 record declares
/// one Fn layer, indexed zero; there is no matching frontend GET method.
pub fn encode_th80_fn_slot_write(
    layer: u8,
    slot: u8,
    value: [u8; 4],
) -> Result<FrontendSendRequest, DriverError> {
    if layer != 0 {
        return Err(DriverError::UnsupportedOperation(
            "TH80 V504 exposes one Fn layer at index 0",
        ));
    }
    validate_slot(slot)?;
    let mut packet = [0_u8; FEATURE_REPORT_BYTES];
    packet[0] = 0x15;
    packet[1] = layer;
    packet[2] = slot;
    packet[8..12].copy_from_slice(&value);
    Ok(FrontendSendRequest::new(
        packet,
        FrontendChecksumType::Bit7,
        DangleDeviceType::None,
    ))
}

/// Encode YC500Common's macro write (`0x16`). The caller supplies the already
/// encoded 256-byte macro event buffer; this function reproduces the vendor's
/// nonzero-page count and first-N-page framing. No event grammar is inferred.
pub fn encode_th80_macro_upload(
    slot: u8,
    macro_data: &[u8],
) -> Result<Vec<FrontendSendRequest>, DriverError> {
    if slot >= MACRO_SLOT_COUNT {
        return Err(DriverError::UnsupportedOperation(
            "TH80 V504 macro slot must be in 0..50",
        ));
    }
    if macro_data.len() != MACRO_BYTES {
        return Err(DriverError::InvalidPacketLength {
            expected: format!("exactly {MACRO_BYTES} encoded macro bytes"),
            actual: macro_data.len(),
        });
    }
    let page_count = (0..5_usize)
        .filter(|page| {
            let start = page * MACRO_WRITE_PAGE_BYTES;
            let end = (start + MACRO_WRITE_PAGE_BYTES).min(macro_data.len());
            macro_data[start..end].iter().any(|byte| *byte != 0)
        })
        .count();
    let mut packets = Vec::with_capacity(page_count);
    for page in 0..page_count {
        let mut packet = [0_u8; FEATURE_REPORT_BYTES];
        packet[0] = 0x16;
        packet[1] = slot;
        packet[2] = page as u8;
        packet[3] = MACRO_WRITE_PAGE_BYTES as u8;
        packet[4] = u8::from(page + 1 == page_count);
        let start = page * MACRO_WRITE_PAGE_BYTES;
        let end = (start + MACRO_WRITE_PAGE_BYTES).min(macro_data.len());
        packet[8..8 + end - start].copy_from_slice(&macro_data[start..end]);
        packets.push(FrontendSendRequest::new(
            packet,
            FrontendChecksumType::Bit7,
            DangleDeviceType::None,
        ));
    }
    Ok(packets)
}

/// The TH80 V504 inherited `getMacro` path sends four page-specific `0x88`
/// Feature SET queries and reads one raw 64-byte Feature response after each.
/// It forwards CheckSumType::BIT7 separately and differs from the generic
/// yc500 `0x8b` per-page command.
pub fn encode_th80_macro_read(slot: u8, page: u8) -> Result<FrontendSendRequest, DriverError> {
    if slot >= MACRO_SLOT_COUNT {
        return Err(DriverError::UnsupportedOperation(
            "TH80 V504 macro slot must be in 0..50",
        ));
    }
    validate_page(page, MACRO_PAGE_COUNT, "macro")?;
    let mut packet = [0_u8; FEATURE_REPORT_BYTES];
    packet[0] = 0x88;
    packet[1] = slot;
    packet[2] = page;
    Ok(FrontendSendRequest::new(
        packet,
        FrontendChecksumType::Bit7,
        DangleDeviceType::None,
    ))
}

/// Build all four page-specific TH80 V504 macro-read query buffers. The
/// frontend performs a separate Feature SET + Feature GET transaction for
/// each page; it does not issue one query followed by four GETs.
pub fn encode_th80_macro_read_pages(slot: u8) -> Result<Vec<FrontendSendRequest>, DriverError> {
    (0..MACRO_PAGE_COUNT)
        .map(|page| encode_th80_macro_read(slot, page))
        .collect()
}

/// Collect the four raw Feature replies produced after the TH80 V504 `0x88`
/// macro-read command. Macro event decoding is intentionally separate.
pub fn decode_th80_macro_pages(pages: &[Vec<u8>]) -> Result<[u8; MACRO_BYTES], DriverError> {
    if pages.len() != MACRO_READ_PAGE_COUNT {
        return Err(DriverError::InvalidPacketLength {
            expected: format!("exactly {MACRO_READ_PAGE_COUNT} macro response pages"),
            actual: pages.len(),
        });
    }
    let mut macro_data = [0_u8; MACRO_BYTES];
    for (index, page) in pages.iter().enumerate() {
        if page.len() != FEATURE_REPORT_BYTES {
            return Err(DriverError::InvalidPacketLength {
                expected: "exactly 64 bytes per TH80 V504 macro page".into(),
                actual: page.len(),
            });
        }
        let start = index * FEATURE_REPORT_BYTES;
        macro_data[start..start + FEATURE_REPORT_BYTES].copy_from_slice(page);
    }
    Ok(macro_data)
}

impl Yc500Command {
    /// Build the exact caller Buffer plus its independent CheckSumType field.
    /// This does not calculate or insert a checksum byte into the message.
    pub fn encode(self) -> Result<FrontendSendRequest, DriverError> {
        let mut packet = [0_u8; FEATURE_REPORT_BYTES];
        match self {
            Self::FirmwareRevision => packet[0] = 0x80,
            Self::ActiveProfile => packet[0] = 0x85,
            Self::KeyboardOptions { profile } => {
                validate_profile(profile)?;
                packet[0] = 0x86;
                packet[1] = profile;
            }
            Self::LightingParameters => packet[0] = 0x87,
            Self::KeymapPage { profile, page } => {
                validate_profile(profile)?;
                validate_page(page, KEYMAP_PAGE_COUNT, "keymap")?;
                packet[0] = 0x89;
                packet[1] = profile;
                packet[2] = page;
            }
            Self::FnLayerPage { profile, page } => {
                validate_profile(profile)?;
                validate_page(page, KEYMAP_PAGE_COUNT, "Fn layer")?;
                packet[0] = 0x90;
                packet[1] = profile;
                packet[2] = page;
            }
            Self::MacroPage { slot, page } => {
                if slot >= MACRO_SLOT_COUNT {
                    return Err(DriverError::UnsupportedOperation(
                        "yc500 macro slot must be in 0..50",
                    ));
                }
                validate_page(page, MACRO_PAGE_COUNT, "macro")?;
                packet[0] = 0x8b;
                packet[1] = slot;
                packet[2] = page;
            }
        }
        let checksum_type = if matches!(self, Self::MacroPage { .. }) {
            FrontendChecksumType::None
        } else {
            FrontendChecksumType::Bit7
        };
        Ok(FrontendSendRequest::new(
            packet,
            checksum_type,
            DangleDeviceType::None,
        ))
    }

    /// Validate the fixed response envelope and decode only fields consumed
    /// by the vendor frontend. Scalar parsers do not require an opcode echo;
    /// keymap, Fn and macro page replies are raw 64-byte pages.
    pub fn decode(self, bytes: &[u8]) -> Result<Yc500Response, DriverError> {
        // Validate request parameters as well as the response envelope.
        let _ = self.encode()?;
        if bytes.len() != FEATURE_REPORT_BYTES {
            return Err(DriverError::InvalidPacketLength {
                expected: "exactly 64-byte yc500 Feature response".into(),
                actual: bytes.len(),
            });
        }
        match self {
            Self::FirmwareRevision => Ok(Yc500Response::FirmwareRevision(u16::from_le_bytes([
                bytes[1], bytes[2],
            ]))),
            Self::ActiveProfile => {
                if bytes[1] >= TH80_PROFILE_COUNT {
                    return Err(DriverError::MalformedReport(format!(
                        "yc500 active profile {} is outside the TH80 profile range 0..4",
                        bytes[1]
                    )));
                }
                Ok(Yc500Response::ActiveProfile(bytes[1]))
            }
            Self::KeyboardOptions { .. } => Ok(Yc500Response::KeyboardOptions {
                flags: bytes[2],
                fn_matrix: bytes[3],
                power_save: bytes[4],
            }),
            Self::LightingParameters => Ok(Yc500Response::LightingParameters {
                mode: bytes[1],
                speed: bytes[2],
                brightness: bytes[3],
                option_flags: bytes[4],
                red: bytes[5],
                green: bytes[6],
                blue: bytes[7],
            }),
            Self::KeymapPage { .. } | Self::FnLayerPage { .. } | Self::MacroPage { .. } => {
                let mut page = [0_u8; FEATURE_REPORT_BYTES];
                page.copy_from_slice(bytes);
                Ok(Yc500Response::RawPage(page))
            }
        }
    }
}

/// Decode the candidate GET_INFOR response (`0x8f`) without constructing a
/// request. The V504 bundle declares the opcode but exposes no target getter
/// method or checksum selection, so request dispatch remains unsupported.
pub fn decode_candidate_identify_response(bytes: &[u8]) -> Result<u32, DriverError> {
    if bytes.len() != FEATURE_REPORT_BYTES {
        return Err(DriverError::InvalidPacketLength {
            expected: "exactly 64-byte identify response".into(),
            actual: bytes.len(),
        });
    }
    if bytes[0] != 0x8f {
        return Err(DriverError::MalformedReport(format!(
            "identify response opcode mismatch: expected 0x8f, got 0x{:02x}",
            bytes[0]
        )));
    }
    Ok(u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]))
}

/// Build the seven-page TH80 V504 user-picture upload (`0x0c`). The V504
/// frontend sends seven 64-byte reports containing an 8-byte header and up to
/// 56 RGB bytes. This is a persistent-write packet encoder only; it never
/// submits packets. GET_USERPIC read-back remains unreliable.
pub fn encode_user_picture_upload(rgb: &[u8]) -> Result<Vec<FrontendSendRequest>, DriverError> {
    if rgb.len() != USER_PICTURE_BYTES {
        return Err(DriverError::InvalidPacketLength {
            expected: format!("exactly {USER_PICTURE_BYTES} RGB bytes for 128 keys"),
            actual: rgb.len(),
        });
    }
    let mut packets = Vec::with_capacity(7);
    for page in 0..7_usize {
        let mut packet = [0_u8; FEATURE_REPORT_BYTES];
        packet[0] = 0x0c;
        packet[1] = 0;
        packet[2..4].copy_from_slice(&(USER_PICTURE_BYTES as u16).to_le_bytes());
        packet[4] = page as u8;
        let start = page * USER_PICTURE_PAGE_BYTES;
        let end = (start + USER_PICTURE_PAGE_BYTES).min(rgb.len());
        packet[8..8 + end - start].copy_from_slice(&rgb[start..end]);
        packets.push(FrontendSendRequest::new(
            packet,
            FrontendChecksumType::Bit7,
            DangleDeviceType::None,
        ));
    }
    Ok(packets)
}

/// SET_PROFILE (`0x05`), one-byte zero-based profile index. The selected V504
/// frontend passes `CheckSumType::BIT7` (enum value 0); byte 7 remains zero in
/// the caller Buffer and native checksum handling is a separate layer.
pub fn encode_set_active_profile(profile: u8) -> Result<FrontendSendRequest, DriverError> {
    validate_profile(profile)?;
    let mut packet = [0_u8; FEATURE_REPORT_BYTES];
    packet[0] = 0x05;
    packet[1] = profile;
    Ok(FrontendSendRequest::new(
        packet,
        FrontendChecksumType::Bit7,
        DangleDeviceType::None,
    ))
}

/// SET_KBOPTION (`0x06`). The flag byte is deliberately opaque: vendor JS
/// has a Mac-mode getter/setter bit discrepancy, so this API does not
/// reinterpret or normalize caller-supplied option bits.
pub fn encode_keyboard_options_write(
    profile: u8,
    flags: u8,
    fn_matrix: u8,
) -> Result<FrontendSendRequest, DriverError> {
    validate_profile(profile)?;
    let mut packet = [0_u8; FEATURE_REPORT_BYTES];
    packet[0] = 0x06;
    packet[1] = profile;
    packet[2] = flags;
    packet[3] = fn_matrix;
    Ok(FrontendSendRequest::new(
        packet,
        FrontendChecksumType::Bit7,
        DangleDeviceType::None,
    ))
}

/// SET_LEDPARAM (`0x07`). `option_flags` is the complete byte 4; mode-specific
/// interpretation and ranges are intentionally not inferred. The TH80 V504
/// frontend forwards CheckSumType::BIT8 separately; byte 7 remains RGB blue
/// and byte 8 remains zero in the original caller Buffer.
pub fn encode_lighting_write(
    mode: u8,
    speed: u8,
    brightness: u8,
    option_flags: u8,
    rgb: [u8; 3],
) -> FrontendSendRequest {
    let mut packet = [0_u8; FEATURE_REPORT_BYTES];
    packet[..8].copy_from_slice(&[
        0x07,
        mode,
        speed,
        brightness,
        option_flags,
        rgb[0],
        rgb[1],
        rgb[2],
    ]);
    FrontendSendRequest::new(packet, FrontendChecksumType::Bit8, DangleDeviceType::None)
}

/// These candidates do not have an applicable yc500 command contract.
/// Keeping them explicit prevents borrowing opcodes from the incompatible
/// gen2 family or unrelated frontend model classes.
pub fn unsupported_candidate(candidate: &str) -> DriverError {
    match candidate {
        "C02" => DriverError::UnsupportedOperation(
            "GET_INFOR request/checksum method is not established for TH80 V504",
        ),
        "C03" => DriverError::UnsupportedOperation(
            "the only located getBattery method belongs to the unrelated Dk2017 class, not the TH80 inheritance chain",
        ),
        "C04" => {
            DriverError::UnsupportedOperation("C04 system telemetry is not a TH80 HID command")
        }
        "C06" => DriverError::UnsupportedOperation(
            "use the TH80 V504 report-rate codec; generic yc500 report-rate semantics conflict",
        ),
        "C09-read" => DriverError::UnsupportedOperation(
            "yc500 GET_USERPIC does not provide reliable read-back",
        ),
        "C10-bulk-write" => DriverError::UnsupportedOperation(
            "TH80 512-byte matrix does not reconcile with the inherited nine 56-byte bulk transfer pages",
        ),
        "C11-write" => DriverError::UnsupportedOperation(
            "use encode_th80_fn_slot_write for the TH80 V504 single-slot write contract",
        ),
        "C11-read" => DriverError::UnsupportedOperation(
            "TH80 V504 defines GET_FN_SIMPLE but no frontend read method",
        ),
        "C12-events" => DriverError::UnsupportedOperation(
            "macro event encoding grammar is not established; only raw 256-byte page framing is supported",
        ),
        _ => DriverError::UnsupportedOperation("command candidate has no verified yc500 contract"),
    }
}

fn validate_profile(profile: u8) -> Result<(), DriverError> {
    if profile >= TH80_PROFILE_COUNT {
        return Err(DriverError::UnsupportedOperation(
            "TH80 profile index must be in 0..4",
        ));
    }
    Ok(())
}

fn validate_slot(slot: u8) -> Result<(), DriverError> {
    if slot >= 128 {
        return Err(DriverError::UnsupportedOperation(
            "TH80 V504 key slot must be in 0..128",
        ));
    }
    Ok(())
}

fn validate_page(page: u8, count: u8, label: &'static str) -> Result<(), DriverError> {
    if page >= count {
        return Err(DriverError::UnsupportedOperation(match label {
            "keymap" => "yc500 keymap page must be in 0..8",
            "Fn layer" => "yc500 Fn page must be in 0..8",
            "macro" => "yc500 macro page must be in 0..4",
            _ => "yc500 page index out of range",
        }));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_requests_preserve_frontend_checksum_metadata_without_mutating_buffers() {
        for (command, opcode, checksum_type) in [
            (
                Yc500Command::FirmwareRevision,
                0x80,
                FrontendChecksumType::Bit7,
            ),
            (
                Yc500Command::ActiveProfile,
                0x85,
                FrontendChecksumType::Bit7,
            ),
            (
                Yc500Command::KeyboardOptions { profile: 0 },
                0x86,
                FrontendChecksumType::Bit7,
            ),
            (
                Yc500Command::LightingParameters,
                0x87,
                FrontendChecksumType::Bit7,
            ),
            (
                Yc500Command::KeymapPage {
                    profile: 3,
                    page: 7,
                },
                0x89,
                FrontendChecksumType::Bit7,
            ),
            (
                Yc500Command::FnLayerPage {
                    profile: 0,
                    page: 0,
                },
                0x90,
                FrontendChecksumType::Bit7,
            ),
            (
                Yc500Command::MacroPage { slot: 49, page: 3 },
                0x8b,
                FrontendChecksumType::None,
            ),
        ] {
            let request = command.encode().unwrap();
            assert_eq!(request.message.len(), FEATURE_REPORT_BYTES);
            assert_eq!(request.message[0], opcode);
            assert_eq!(request.checksum_type, checksum_type);
            assert_eq!(request.dangle_device_type, DangleDeviceType::None);
            assert_eq!(request.message[7], 0);
            assert!(request.message[8..].iter().all(|byte| *byte == 0));
        }
    }
    #[test]
    fn paged_commands_encode_the_documented_profile_slot_and_page_offsets() {
        let keymap = Yc500Command::KeymapPage {
            profile: 2,
            page: 6,
        }
        .encode()
        .unwrap();
        assert_eq!(&keymap.message[..3], &[0x89, 2, 6]);
        let function = Yc500Command::FnLayerPage {
            profile: 1,
            page: 5,
        }
        .encode()
        .unwrap();
        assert_eq!(&function.message[..3], &[0x90, 1, 5]);
        let macro_page = Yc500Command::MacroPage { slot: 48, page: 2 }
            .encode()
            .unwrap();
        assert_eq!(&macro_page.message[..3], &[0x8b, 48, 2]);
    }
    #[test]
    fn page_and_profile_boundaries_are_rejected() {
        for command in [
            Yc500Command::KeymapPage {
                profile: 4,
                page: 0,
            },
            Yc500Command::KeymapPage {
                profile: 0,
                page: 8,
            },
            Yc500Command::FnLayerPage {
                profile: 4,
                page: 0,
            },
            Yc500Command::FnLayerPage {
                profile: 0,
                page: 8,
            },
            Yc500Command::MacroPage { slot: 50, page: 0 },
            Yc500Command::MacroPage { slot: 0, page: 4 },
        ] {
            assert!(matches!(
                command.encode(),
                Err(DriverError::UnsupportedOperation(_))
            ));
        }
    }

    #[test]
    fn scalar_responses_require_exact_length_without_inventing_opcode_echo() {
        let mut revision = [0_u8; FEATURE_REPORT_BYTES];
        revision[0] = 0x80;
        revision[1..3].copy_from_slice(&0x0504_u16.to_le_bytes());
        assert_eq!(
            Yc500Command::FirmwareRevision.decode(&revision).unwrap(),
            Yc500Response::FirmwareRevision(0x0504)
        );
        assert!(matches!(
            Yc500Command::FirmwareRevision.decode(&revision[..63]),
            Err(DriverError::InvalidPacketLength { .. })
        ));
        revision[0] = 0x81;
        assert_eq!(
            Yc500Command::FirmwareRevision.decode(&revision).unwrap(),
            Yc500Response::FirmwareRevision(0x0504)
        );
    }

    #[test]
    fn identify_profile_options_and_lighting_responses_are_decoded_without_guessing_enums() {
        let mut identify = [0_u8; FEATURE_REPORT_BYTES];
        identify[0] = 0x8f;
        identify[1..5].copy_from_slice(&0x1234_5678_u32.to_le_bytes());
        assert_eq!(
            decode_candidate_identify_response(&identify).unwrap(),
            0x1234_5678
        );
        identify[0] = 0x90;
        assert!(matches!(
            decode_candidate_identify_response(&identify),
            Err(DriverError::MalformedReport(_))
        ));

        let mut profile = [0_u8; FEATURE_REPORT_BYTES];
        profile[0] = 0x00;
        profile[1] = 3;
        assert_eq!(
            Yc500Command::ActiveProfile.decode(&profile).unwrap(),
            Yc500Response::ActiveProfile(3)
        );
        profile[1] = 4;
        assert!(Yc500Command::ActiveProfile.decode(&profile).is_err());

        let mut options = [0_u8; FEATURE_REPORT_BYTES];
        options[0] = 0x01;
        options[2..5].copy_from_slice(&[0x82, 1, 1]);
        assert_eq!(
            Yc500Command::KeyboardOptions { profile: 0 }
                .decode(&options)
                .unwrap(),
            Yc500Response::KeyboardOptions {
                flags: 0x82,
                fn_matrix: 1,
                power_save: 1,
            }
        );

        let mut led = [0_u8; FEATURE_REPORT_BYTES];
        led[..5].copy_from_slice(&[0x87, 5, 2, 4, 0x87]);
        led[0] = 0x02;
        assert_eq!(
            Yc500Command::LightingParameters.decode(&led).unwrap(),
            Yc500Response::LightingParameters {
                mode: 5,
                speed: 2,
                brightness: 4,
                option_flags: 0x87,
                red: 0,
                green: 0,
                blue: 0,
            }
        );
    }

    #[test]
    fn page_responses_are_raw_and_do_not_require_an_opcode_echo() {
        let bytes = [0xa5_u8; FEATURE_REPORT_BYTES];
        assert_eq!(
            Yc500Command::KeymapPage {
                profile: 0,
                page: 0
            }
            .decode(&bytes)
            .unwrap(),
            Yc500Response::RawPage(bytes)
        );
    }

    #[test]
    fn user_picture_upload_is_seven_bounded_pages_with_exact_total_length() {
        let rgb: Vec<u8> = (0..USER_PICTURE_BYTES).map(|n| n as u8).collect();
        let packets = encode_user_picture_upload(&rgb).unwrap();
        assert_eq!(packets.len(), 7);
        for (page, request) in packets.iter().enumerate() {
            assert_eq!(&request.message[..5], &[0x0c, 0, 0x80, 0x01, page as u8]);
            assert_eq!(request.checksum_type, FrontendChecksumType::Bit7);
            assert_eq!(request.dangle_device_type, DangleDeviceType::None);
            assert_eq!(request.message[7], 0);
            let start = page * USER_PICTURE_PAGE_BYTES;
            let end = (start + USER_PICTURE_PAGE_BYTES).min(USER_PICTURE_BYTES);
            assert_eq!(&request.message[8..8 + end - start], &rgb[start..end]);
            assert!(request.message[8 + end - start..]
                .iter()
                .all(|byte| *byte == 0));
        }
        assert!(matches!(
            encode_user_picture_upload(&rgb[..383]),
            Err(DriverError::InvalidPacketLength { .. })
        ));
    }
    #[test]
    fn simple_setters_preserve_payload_and_frontend_checksum_metadata_separately() {
        let profile = encode_set_active_profile(2).unwrap();
        assert_eq!(&profile.message[..2], &[0x05, 2]);
        assert_eq!(profile.checksum_type, FrontendChecksumType::Bit7);
        assert_eq!(profile.dangle_device_type, DangleDeviceType::None);
        assert_eq!(profile.message[7], 0);
        assert!(encode_set_active_profile(4).is_err());

        let options = encode_keyboard_options_write(2, 0xa5, 1).unwrap();
        assert_eq!(&options.message[..5], &[0x06, 2, 0xa5, 1, 0]);
        assert_eq!(options.checksum_type, FrontendChecksumType::Bit7);
        assert_eq!(options.message[7], 0);

        let lighting = encode_lighting_write(5, 2, 4, 0x87, [255, 128, 77]);
        assert_eq!(&lighting.message[..8], &[0x07, 5, 2, 4, 0x87, 255, 128, 77]);
        assert_eq!(lighting.checksum_type, FrontendChecksumType::Bit8);
        assert_eq!(lighting.dangle_device_type, DangleDeviceType::None);
        assert_eq!(lighting.message[8], 0);
    }
    #[test]
    fn unsupported_candidates_fail_explicitly() {
        for candidate in [
            "C02",
            "C03",
            "C04",
            "C09-read",
            "C10-bulk-write",
            "C11-read",
            "C12-events",
        ] {
            assert!(matches!(
                unsupported_candidate(candidate),
                DriverError::UnsupportedOperation(_)
            ));
        }
    }

    #[test]
    fn th80_v504_report_rate_uses_the_frontend_dialect() {
        let query = Th80V504Command::ReportRate { profile: 2 }.encode().unwrap();
        assert_eq!(&query.message[..2], &[0x84, 2]);
        assert_eq!(query.checksum_type, FrontendChecksumType::Bit7);
        assert_eq!(query.message[7], 0);
        let mut response = [0_u8; FEATURE_REPORT_BYTES];
        response[2] = 4;
        assert_eq!(
            Th80V504Command::ReportRate { profile: 2 }
                .decode(&response)
                .unwrap(),
            Th80V504Response::ReportRateHz(250)
        );
        response[2] = 3;
        assert!(matches!(
            Th80V504Command::ReportRate { profile: 2 }.decode(&response),
            Err(DriverError::MalformedReport(_))
        ));
    }
    #[test]
    fn th80_v504_report_rate_setter_maps_only_documented_rates() {
        for (rate, code) in [(1000, 1), (500, 2), (250, 4), (125, 8)] {
            let request = encode_th80_v504_set_report_rate(3, rate).unwrap();
            assert_eq!(&request.message[..3], &[0x04, 3, code]);
            assert_eq!(request.checksum_type, FrontendChecksumType::Bit7);
            assert_eq!(request.message[7], 0);
        }
        assert!(encode_th80_v504_set_report_rate(4, 1000).is_err());
        assert!(encode_th80_v504_set_report_rate(0, 333).is_err());
    }
    #[test]
    fn th80_v504_single_slot_key_and_fn_writes_preserve_four_byte_values() {
        let key = encode_th80_key_slot_write(2, 127, [1, 2, 3, 4]).unwrap();
        assert_eq!(&key.message[..3], &[0x13, 2, 127]);
        assert_eq!(&key.message[8..12], &[1, 2, 3, 4]);
        assert_eq!(key.checksum_type, FrontendChecksumType::Bit7);
        assert_eq!(key.message[7], 0);
        let function = encode_th80_fn_slot_write(0, 126, [4, 3, 2, 1]).unwrap();
        assert_eq!(&function.message[..3], &[0x15, 0, 126]);
        assert_eq!(&function.message[8..12], &[4, 3, 2, 1]);
        assert_eq!(function.checksum_type, FrontendChecksumType::Bit7);
        assert_eq!(function.message[7], 0);
        assert!(encode_th80_key_slot_write(0, 128, [0; 4]).is_err());
        assert!(encode_th80_fn_slot_write(1, 0, [0; 4]).is_err());
    }
    #[test]
    fn th80_v504_macro_upload_uses_vendor_nonzero_page_count_and_final_marker() {
        let data: Vec<u8> = (0..MACRO_BYTES).map(|value| value as u8).collect();
        let packets = encode_th80_macro_upload(49, &data).unwrap();
        assert_eq!(packets.len(), 5);
        for (page, request) in packets.iter().enumerate() {
            assert_eq!(
                &request.message[..5],
                &[0x16, 49, page as u8, 56, u8::from(page == 4)]
            );
            assert_eq!(request.checksum_type, FrontendChecksumType::Bit7);
            assert_eq!(request.message[7], 0);
            let start = page * MACRO_WRITE_PAGE_BYTES;
            let end = (start + MACRO_WRITE_PAGE_BYTES).min(MACRO_BYTES);
            assert_eq!(&request.message[8..8 + end - start], &data[start..end]);
            assert!(request.message[8 + end - start..]
                .iter()
                .all(|byte| *byte == 0));
        }
        assert!(encode_th80_macro_upload(49, &[0; MACRO_BYTES])
            .unwrap()
            .is_empty());
        let mut short = [0; MACRO_BYTES];
        short[0] = 1;
        assert_eq!(encode_th80_macro_upload(49, &short).unwrap().len(), 1);
        assert!(encode_th80_macro_upload(50, &data).is_err());
        assert!(encode_th80_macro_upload(0, &data[..255]).is_err());
    }
    #[test]
    fn th80_v504_macro_read_uses_four_page_specific_set_get_queries() {
        let requests = encode_th80_macro_read_pages(49).unwrap();
        assert_eq!(requests.len(), 4);
        for (page, request) in requests.iter().enumerate() {
            assert_eq!(&request.message[..3], &[0x88, 49, page as u8]);
            assert_eq!(request.checksum_type, FrontendChecksumType::Bit7);
            assert_eq!(request.message[7], 0);
            assert!(request.message[8..].iter().all(|byte| *byte == 0));
        }
        let pages: Vec<Vec<u8>> = (0..MACRO_READ_PAGE_COUNT)
            .map(|page| vec![page as u8; FEATURE_REPORT_BYTES])
            .collect();
        let data = decode_th80_macro_pages(&pages).unwrap();
        assert_eq!(&data[..64], &[0; 64]);
        assert_eq!(&data[64..128], &[1; 64]);
        assert_eq!(&data[192..], &[3; 64]);
        assert!(decode_th80_macro_pages(&pages[..3]).is_err());
        let mut malformed = pages;
        malformed[2].pop();
        assert!(decode_th80_macro_pages(&malformed).is_err());
        assert!(encode_th80_macro_read(50, 0).is_err());
        assert!(encode_th80_macro_read(0, 4).is_err());
    }
}
