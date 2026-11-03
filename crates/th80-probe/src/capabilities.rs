//! Typed profile conversion over the existing Phase 5B HID parser.

pub use crate::hid_parser::{
    parse, LongItem, ParseDiagnostic, ParseError, ParsedDescriptor, ReportDefinition, ReportField,
    ReportType, Usage,
};

use crate::device::{DescriptorFingerprint, InterfaceRole};

#[derive(Debug, Clone, serde::Serialize)]
pub struct CapabilityProfile {
    pub interface_role: InterfaceRole,
    pub descriptor: DescriptorFingerprint,
    pub uses_report_ids: bool,
    pub input_report_count: usize,
    pub output_report_count: usize,
    pub feature_report_count: usize,
    pub feature_report_bytes: Option<u64>,
    pub reports: Vec<ReportDefinition>,
    pub long_items: Vec<LongItem>,
    pub parser_warnings: Vec<ParseDiagnostic>,
}

impl CapabilityProfile {
    pub fn from_descriptor(interface_number: u8, parsed: ParsedDescriptor) -> Self {
        Self::from_descriptor_for_class(interface_number, 0x03, parsed)
    }

    pub fn from_descriptor_for_class(
        interface_number: u8,
        interface_class: u8,
        parsed: ParsedDescriptor,
    ) -> Self {
        let feature_report_bytes = parsed
            .reports
            .iter()
            .filter(|report| report.report_type == ReportType::Feature)
            .map(|report| report.transfer_bytes)
            .reduce(u64::saturating_add);
        let interface_role = InterfaceRole::classify(
            Some(interface_number),
            Some(interface_class),
            feature_report_bytes,
        );
        Self {
            interface_role,
            descriptor: DescriptorFingerprint {
                sha256: parsed.sha256,
                length: parsed.descriptor_length,
            },
            uses_report_ids: parsed.uses_report_ids,
            input_report_count: parsed.input_report_count,
            output_report_count: parsed.output_report_count,
            feature_report_count: parsed.feature_report_count,
            feature_report_bytes,
            reports: parsed.reports,
            long_items: parsed.long_items,
            parser_warnings: parsed.warnings,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_captured_vendor_feature_descriptor_to_typed_profile() {
        let bytes = [
            0x06, 0xff, 0xff, 0x09, 0x02, 0xa1, 0x01, 0x09, 0x02, 0x15, 0x80, 0x25, 0x7f, 0x95,
            0x40, 0x75, 0x08, 0xb1, 0x02, 0xc0,
        ];
        let profile = CapabilityProfile::from_descriptor(2, parse(&bytes).unwrap());
        assert_eq!(profile.interface_role, InterfaceRole::VendorFeature);
        assert_eq!(profile.feature_report_bytes, Some(64));
        assert_eq!(profile.feature_report_count, 1);
        assert_eq!(profile.descriptor.length, 20);
        assert!(!profile.uses_report_ids);
        assert_eq!(profile.reports[0].report_type, ReportType::Feature);
    }
}
