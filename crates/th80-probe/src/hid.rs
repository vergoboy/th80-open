use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportType {
    Input,
    Output,
    Feature,
}

impl ReportType {
    fn from_main_tag(tag: u8) -> Option<Self> {
        match tag {
            8 => Some(Self::Input),
            9 => Some(Self::Output),
            11 => Some(Self::Feature),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Usage {
    pub page: u32,
    pub id: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReportField {
    pub report_size_bits: u32,
    pub report_count: u32,
    pub payload_bits: u64,
    pub flags: u32,
    pub constant: bool,
    pub variable: bool,
    pub relative: bool,
    pub logical_minimum: i64,
    pub logical_maximum: i64,
    pub physical_minimum: i64,
    pub physical_maximum: i64,
    pub unit: u32,
    pub unit_exponent: i8,
    pub usages: Vec<Usage>,
    pub usage_minimum: Option<Usage>,
    pub usage_maximum: Option<Usage>,
    pub collection_path: Vec<String>,
    pub designator_indices: Vec<u32>,
    pub designator_minimum: Option<u32>,
    pub designator_maximum: Option<u32>,
    pub string_indices: Vec<u32>,
    pub string_minimum: Option<u32>,
    pub string_maximum: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReportDefinition {
    pub report_type: ReportType,
    pub report_id: u8,
    pub report_id_explicit: bool,
    pub report_size_bits: Option<u32>,
    pub report_count: u32,
    pub payload_bits: u64,
    pub payload_bytes: u64,
    pub transfer_bytes: u64,
    pub fields: Vec<ReportField>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ParseDiagnostic {
    pub offset: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LongItem {
    pub offset: usize,
    pub tag: u8,
    pub data_hex: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ParsedDescriptor {
    pub descriptor_length: usize,
    pub sha256: String,
    pub uses_report_ids: bool,
    pub reports: Vec<ReportDefinition>,
    pub long_items: Vec<LongItem>,
    pub input_report_count: usize,
    pub output_report_count: usize,
    pub feature_report_count: usize,
    pub warnings: Vec<ParseDiagnostic>,
    pub errors: Vec<ParseDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub offset: usize,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "descriptor offset 0x{:x}: {}", self.offset, self.message)
    }
}
impl std::error::Error for ParseError {}

#[derive(Debug, Clone, Default)]
struct Globals {
    usage_page: u32,
    logical_minimum: i64,
    logical_maximum: i64,
    physical_minimum: i64,
    physical_maximum: i64,
    unit: u32,
    unit_exponent: i8,
    report_size: u32,
    report_id: u8,
    report_id_explicit: bool,
    report_count: u32,
}

#[derive(Debug, Clone, Default)]
struct Locals {
    usages: Vec<Usage>,
    usage_minimum: Option<Usage>,
    usage_maximum: Option<Usage>,
    designator_indices: Vec<u32>,
    designator_minimum: Option<u32>,
    designator_maximum: Option<u32>,
    string_indices: Vec<u32>,
    string_minimum: Option<u32>,
    string_maximum: Option<u32>,
}

#[derive(Debug, Clone)]
struct Collection {
    kind: u8,
    usage: Option<Usage>,
}

fn unsigned(bytes: &[u8]) -> u32 {
    bytes.iter().enumerate().fold(0, |value, (shift, byte)| {
        value | ((*byte as u32) << (shift * 8))
    })
}
fn signed(bytes: &[u8]) -> i64 {
    if bytes.is_empty() {
        return 0;
    }
    let value = unsigned(bytes) as i64;
    let bits = bytes.len() * 8;
    if value & (1_i64 << (bits - 1)) != 0 {
        value - (1_i64 << bits)
    } else {
        value
    }
}
fn usage(value: u32, size: usize, page: u32) -> Usage {
    if size == 4 && value > 0xffff {
        Usage {
            page: value >> 16,
            id: value & 0xffff,
        }
    } else {
        Usage { page, id: value }
    }
}
fn usage_label(value: &Usage) -> String {
    format!("{:#06x}:{:#06x}", value.page, value.id)
}

fn valid_numeric_item_size(size: usize) -> bool {
    matches!(size, 1 | 2 | 4)
}

pub fn parse(bytes: &[u8]) -> Result<ParsedDescriptor, ParseError> {
    const MAX_DESCRIPTOR_BYTES: usize = 4096;
    if bytes.is_empty() {
        return Err(ParseError {
            offset: 0,
            message: "empty HID report descriptor".into(),
        });
    }
    if bytes.len() > MAX_DESCRIPTOR_BYTES {
        return Err(ParseError {
            offset: MAX_DESCRIPTOR_BYTES,
            message: format!("descriptor exceeds the supported {MAX_DESCRIPTOR_BYTES}-byte limit"),
        });
    }
    let mut globals = Globals::default();
    let mut global_stack: Vec<Globals> = Vec::new();
    let mut locals = Locals::default();
    let mut collections: Vec<Collection> = Vec::new();
    let mut reports: BTreeMap<(ReportType, u8), ReportDefinition> = BTreeMap::new();
    let mut warnings = Vec::new();
    let mut long_items = Vec::new();
    let mut offset = 0;
    let mut uses_report_ids = false;
    let mut explicit_zero_report_id = false;

    while offset < bytes.len() {
        let item_offset = offset;
        let prefix = bytes[offset];
        offset += 1;
        if prefix == 0xfe {
            if offset + 2 > bytes.len() {
                return Err(ParseError {
                    offset: item_offset,
                    message: "truncated long-item header".into(),
                });
            }
            let size = bytes[offset] as usize;
            let tag = bytes[offset + 1];
            offset += 2;
            if offset + size > bytes.len() {
                return Err(ParseError {
                    offset: item_offset,
                    message: format!(
                        "truncated long item (tag 0x{tag:02x}, expected {size} data bytes)"
                    ),
                });
            }
            let data = &bytes[offset..offset + size];
            long_items.push(LongItem {
                offset: item_offset,
                tag,
                data_hex: data
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<Vec<_>>()
                    .join(" "),
            });
            warnings.push(ParseDiagnostic { offset: item_offset, message: format!("long item tag 0x{tag:02x} with {size} data bytes recorded; semantics are not interpreted") });
            offset += size;
            continue;
        }
        let size_code = (prefix & 0x03) as usize;
        let size = match size_code {
            0 => 0,
            1 => 1,
            2 => 2,
            _ => 4,
        };
        let item_type = (prefix >> 2) & 0x03;
        let tag = prefix >> 4;
        if offset + size > bytes.len() {
            return Err(ParseError {
                offset: item_offset,
                message: format!("truncated short item: needs {size} data bytes"),
            });
        }
        let data = &bytes[offset..offset + size];
        offset += size;
        let value = unsigned(data);

        match item_type {
            0 => {
                if let Some(report_type) = ReportType::from_main_tag(tag) {
                    if !valid_numeric_item_size(size) {
                        return Err(ParseError {
                            offset: item_offset,
                            message: format!(
                                "{:?} main item must contain 1, 2, or 4 data bytes",
                                report_type
                            ),
                        });
                    }
                    if globals.report_size == 0 || globals.report_count == 0 {
                        return Err(ParseError {
                            offset: item_offset,
                            message: format!(
                                "{:?} item requires non-zero Report Size and Report Count",
                                report_type
                            ),
                        });
                    }
                    let bits = u64::from(globals.report_size)
                        .checked_mul(u64::from(globals.report_count))
                        .ok_or_else(|| ParseError {
                            offset: item_offset,
                            message: "report bit length overflow".into(),
                        })?;
                    let key = (report_type, globals.report_id);
                    let entry = reports.entry(key).or_insert_with(|| ReportDefinition {
                        report_type,
                        report_id: globals.report_id,
                        report_id_explicit: globals.report_id_explicit,
                        report_size_bits: Some(globals.report_size),
                        report_count: 0,
                        payload_bits: 0,
                        payload_bytes: 0,
                        transfer_bytes: 0,
                        fields: Vec::new(),
                    });
                    entry.report_count = entry
                        .report_count
                        .checked_add(globals.report_count)
                        .ok_or_else(|| ParseError {
                            offset: item_offset,
                            message: "report count overflow".into(),
                        })?;
                    entry.payload_bits =
                        entry
                            .payload_bits
                            .checked_add(bits)
                            .ok_or_else(|| ParseError {
                                offset: item_offset,
                                message: "report payload length overflow".into(),
                            })?;
                    if entry.report_size_bits != Some(globals.report_size) {
                        entry.report_size_bits = None;
                    }
                    entry.fields.push(ReportField {
                        report_size_bits: globals.report_size,
                        report_count: globals.report_count,
                        payload_bits: bits,
                        flags: value,
                        constant: value & 1 != 0,
                        variable: value & 2 != 0,
                        relative: value & 4 != 0,
                        logical_minimum: globals.logical_minimum,
                        logical_maximum: globals.logical_maximum,
                        physical_minimum: globals.physical_minimum,
                        physical_maximum: globals.physical_maximum,
                        unit: globals.unit,
                        unit_exponent: globals.unit_exponent,
                        usages: locals.usages.clone(),
                        usage_minimum: locals.usage_minimum.clone(),
                        usage_maximum: locals.usage_maximum.clone(),
                        collection_path: collections
                            .iter()
                            .map(|c| {
                                format!(
                                    "type={};usage={}",
                                    c.kind,
                                    c.usage
                                        .as_ref()
                                        .map(usage_label)
                                        .unwrap_or_else(|| "unspecified".into())
                                )
                            })
                            .collect(),
                        designator_indices: locals.designator_indices.clone(),
                        designator_minimum: locals.designator_minimum,
                        designator_maximum: locals.designator_maximum,
                        string_indices: locals.string_indices.clone(),
                        string_minimum: locals.string_minimum,
                        string_maximum: locals.string_maximum,
                    });
                } else if tag == 10 {
                    if size != 1 {
                        return Err(ParseError {
                            offset: item_offset,
                            message: "Collection item must contain exactly one data byte".into(),
                        });
                    }
                    let current_usage = locals
                        .usages
                        .first()
                        .cloned()
                        .or_else(|| locals.usage_minimum.clone());
                    if value > 6 {
                        warnings.push(ParseDiagnostic {
                            offset: item_offset,
                            message: format!("reserved Collection type {value}"),
                        });
                    }
                    collections.push(Collection {
                        kind: value as u8,
                        usage: current_usage,
                    });
                } else if tag == 12 {
                    if size != 0 {
                        return Err(ParseError {
                            offset: item_offset,
                            message: "End Collection item must not contain data".into(),
                        });
                    }
                    if collections.pop().is_none() {
                        return Err(ParseError {
                            offset: item_offset,
                            message: "End Collection without matching Collection".into(),
                        });
                    }
                } else {
                    warnings.push(ParseDiagnostic {
                        offset: item_offset,
                        message: format!("unsupported/reserved main item tag {tag}"),
                    });
                }
                locals = Locals::default();
            }
            1 => match tag {
                0 => {
                    if !valid_numeric_item_size(size) {
                        return Err(ParseError {
                            offset: item_offset,
                            message: "Usage Page item must contain 1, 2, or 4 data bytes".into(),
                        });
                    }
                    globals.usage_page = value
                }
                1 => {
                    if !valid_numeric_item_size(size) {
                        return Err(ParseError {
                            offset: item_offset,
                            message: "Logical Minimum item must contain 1, 2, or 4 data bytes"
                                .into(),
                        });
                    }
                    globals.logical_minimum = signed(data)
                }
                2 => {
                    if !valid_numeric_item_size(size) {
                        return Err(ParseError {
                            offset: item_offset,
                            message: "Logical Maximum item must contain 1, 2, or 4 data bytes"
                                .into(),
                        });
                    }
                    globals.logical_maximum = if globals.logical_minimum < 0 {
                        signed(data)
                    } else {
                        i64::from(value)
                    }
                }
                3 => {
                    if !valid_numeric_item_size(size) {
                        return Err(ParseError {
                            offset: item_offset,
                            message: "Physical Minimum item must contain 1, 2, or 4 data bytes"
                                .into(),
                        });
                    }
                    globals.physical_minimum = signed(data)
                }
                4 => {
                    if !valid_numeric_item_size(size) {
                        return Err(ParseError {
                            offset: item_offset,
                            message: "Physical Maximum item must contain 1, 2, or 4 data bytes"
                                .into(),
                        });
                    }
                    globals.physical_maximum = if globals.physical_minimum < 0 {
                        signed(data)
                    } else {
                        i64::from(value)
                    }
                }
                5 => {
                    if size != 1 {
                        return Err(ParseError {
                            offset: item_offset,
                            message: "Unit Exponent item must contain exactly one data byte".into(),
                        });
                    }
                    globals.unit_exponent = (((value as u8) & 0x0f) as i8) << 4 >> 4
                }
                6 => {
                    if !valid_numeric_item_size(size) {
                        return Err(ParseError {
                            offset: item_offset,
                            message: "Unit item must contain 1, 2, or 4 data bytes".into(),
                        });
                    }
                    globals.unit = value
                }
                7 => {
                    if !valid_numeric_item_size(size) {
                        return Err(ParseError {
                            offset: item_offset,
                            message: "Report Size item must contain 1, 2, or 4 data bytes".into(),
                        });
                    }
                    globals.report_size = value
                }
                8 => {
                    if size != 1 || value == 0 || value > u8::MAX as u32 {
                        return Err(ParseError {
                            offset: item_offset,
                            message: format!("invalid Report ID {value}"),
                        });
                    }
                    globals.report_id = value as u8;
                    globals.report_id_explicit = true;
                    uses_report_ids = true;
                }
                9 => {
                    if !valid_numeric_item_size(size) {
                        return Err(ParseError {
                            offset: item_offset,
                            message: "Report Count item must contain 1, 2, or 4 data bytes".into(),
                        });
                    }
                    globals.report_count = value
                }
                10 => {
                    if size != 0 {
                        return Err(ParseError {
                            offset: item_offset,
                            message: "Global Push item must not contain data".into(),
                        });
                    }
                    global_stack.push(globals.clone())
                }
                11 => {
                    if size != 0 {
                        return Err(ParseError {
                            offset: item_offset,
                            message: "Global Pop item must not contain data".into(),
                        });
                    }
                    globals = global_stack.pop().ok_or_else(|| ParseError {
                        offset: item_offset,
                        message: "Global Pop without matching Push".into(),
                    })?
                }
                _ => warnings.push(ParseDiagnostic {
                    offset: item_offset,
                    message: format!("unsupported/reserved global item tag {tag}"),
                }),
            },
            2 => match tag {
                0 => locals.usages.push(usage(value, size, globals.usage_page)),
                1 => locals.usage_minimum = Some(usage(value, size, globals.usage_page)),
                2 => locals.usage_maximum = Some(usage(value, size, globals.usage_page)),
                3 => locals.designator_indices.push(value),
                4 => locals.designator_minimum = Some(value),
                5 => locals.designator_maximum = Some(value),
                7 => locals.string_indices.push(value),
                8 => locals.string_minimum = Some(value),
                9 => locals.string_maximum = Some(value),
                10 => warnings.push(ParseDiagnostic {
                    offset: item_offset,
                    message:
                        "Delimiter local item is not expanded; alternate usage sets are not modeled"
                            .into(),
                }),
                _ => warnings.push(ParseDiagnostic {
                    offset: item_offset,
                    message: format!("unsupported/reserved local item tag {tag}"),
                }),
            },
            _ => warnings.push(ParseDiagnostic {
                offset: item_offset,
                message: format!("reserved item type 3, tag {tag}"),
            }),
        }
    }
    if !collections.is_empty() {
        return Err(ParseError {
            offset: bytes.len(),
            message: format!("{} unclosed Collection item(s)", collections.len()),
        });
    }
    if !global_stack.is_empty() {
        return Err(ParseError {
            offset: bytes.len(),
            message: format!("{} unmatched Global Push item(s)", global_stack.len()),
        });
    }
    if uses_report_ids && reports.keys().any(|(_, id)| *id == 0) {
        explicit_zero_report_id = true;
    }
    if explicit_zero_report_id {
        return Err(ParseError {
            offset: bytes.len(),
            message: "Report ID 0 cannot be mixed with explicitly numbered reports".into(),
        });
    }

    for report in reports.values_mut() {
        report.payload_bytes = report.payload_bits.div_ceil(8);
        report.transfer_bytes = report.payload_bytes + u64::from(report.report_id_explicit);
    }
    let reports: Vec<_> = reports.into_values().collect();
    let input_report_count = reports
        .iter()
        .filter(|r| r.report_type == ReportType::Input)
        .count();
    let output_report_count = reports
        .iter()
        .filter(|r| r.report_type == ReportType::Output)
        .count();
    let feature_report_count = reports
        .iter()
        .filter(|r| r.report_type == ReportType::Feature)
        .count();
    Ok(ParsedDescriptor {
        descriptor_length: bytes.len(),
        sha256: sha256_hex(bytes),
        uses_report_ids,
        reports,
        long_items,
        input_report_count,
        output_report_count,
        feature_report_count,
        warnings,
        errors: Vec::new(),
    })
}

// Small dependency-free SHA-256 implementation for descriptor fingerprints.
fn sha256_hex(input: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut data = input.to_vec();
    let bit_len = (data.len() as u64).wrapping_mul(8);
    data.push(0x80);
    while data.len() % 64 != 56 {
        data.push(0);
    }
    data.extend_from_slice(&bit_len.to_be_bytes());
    let mut h = [
        0x6a09e667u32,
        0xbb67ae85,
        0x3c6ef372,
        0xa54ff53a,
        0x510e527f,
        0x9b05688c,
        0x1f83d9ab,
        0x5be0cd19,
    ];
    for chunk in data.as_chunks::<64>().0 {
        let mut w = [0u32; 64];
        for (i, word) in chunk.as_chunks::<4>().0.iter().enumerate() {
            w[i] = u32::from_be_bytes(*word);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (slot, value) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *slot = slot.wrapping_add(value);
        }
    }
    h.iter().map(|v| format!("{v:08x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse_hex(value: &str) -> Vec<u8> {
        value
            .split_whitespace()
            .map(|part| u8::from_str_radix(part, 16).unwrap())
            .collect()
    }
    const KEYBOARD: &str = "05 01 09 06 a1 01 05 08 15 00 25 01 19 01 29 05 75 01 95 05 91 02 95 03 91 01 05 07 19 e0 29 e7 75 01 95 08 81 02 95 08 81 01 15 00 26 ff 00 19 00 2a ff 00 75 08 95 06 81 00 c0";
    const VENDOR_FEATURE: &str = "06 ff ff 09 02 a1 01 09 02 15 80 25 7f 95 40 75 08 b1 02 c0";
    const MULTI: &str = "05 01 09 06 a1 01 85 01 75 08 95 02 81 02 85 02 75 04 95 03 91 02 c0";
    const IFACE1: &str = "05 0c 09 01 a1 01 85 03 19 00 2a 3c 03 15 00 26 3c 03 95 01 75 10 81 00 c0 05 01 09 80 a1 01 85 02 05 01 19 81 29 83 15 00 25 01 95 03 75 01 81 02 95 01 75 05 81 01 c0 05 01 09 06 a1 01 85 01 05 07 15 00 25 01 19 00 29 77 95 78 75 01 81 02 c0 05 01 09 02 a1 01 85 06 09 01 a1 00 05 09 15 00 25 01 19 01 29 05 75 01 95 05 81 02 95 03 81 01 05 01 16 01 80 26 ff 7f 09 30 09 31 75 10 95 02 81 06 15 81 25 7f 09 38 75 08 95 01 81 06 05 0c 0a 38 02 95 01 81 06 c0 c0 06 ff ff 09 01 a1 01 85 05 09 01 15 00 26 ff 00 75 08 95 03 81 02 c0";

    #[test]
    fn keyboard_has_eight_byte_input_and_one_byte_output() {
        let parsed = parse(&parse_hex(KEYBOARD)).unwrap();
        assert_eq!(parsed.input_report_count, 1);
        assert_eq!(parsed.output_report_count, 1);
        assert_eq!(parsed.reports[0].payload_bytes, 8);
        assert_eq!(parsed.reports[0].transfer_bytes, 8);
        assert!(!parsed.uses_report_ids);
    }
    #[test]
    fn vendor_feature_length_and_usage_are_derived() {
        let parsed = parse(&parse_hex(VENDOR_FEATURE)).unwrap();
        let r = &parsed.reports[0];
        assert_eq!(r.report_type, ReportType::Feature);
        assert_eq!(r.payload_bits, 512);
        assert_eq!(r.payload_bytes, 64);
        assert_eq!(r.transfer_bytes, 64);
        assert_eq!(r.fields[0].usages[0].page, 0xffff);
    }
    #[test]
    fn multiple_report_ids_include_id_byte_only_in_transfer_length() {
        let parsed = parse(&parse_hex(MULTI)).unwrap();
        assert!(parsed.uses_report_ids);
        assert_eq!(parsed.reports[0].transfer_bytes, 3);
        assert_eq!(parsed.reports[1].transfer_bytes, 3);
        assert!(parsed.reports.iter().all(|r| r.report_id_explicit));
    }
    #[test]
    fn push_pop_restores_global_report_state() {
        let d = parse_hex("05 01 75 08 95 01 a4 75 10 95 02 81 02 b4 81 02");
        let p = parse(&d).unwrap();
        assert_eq!(p.reports[0].payload_bits, 40);
    }
    #[test]
    fn nested_collections_are_retained_in_context() {
        let p = parse(&parse_hex(
            "05 01 09 06 a1 01 09 02 a1 00 75 08 95 01 81 02 c0 c0",
        ))
        .unwrap();
        assert_eq!(p.reports[0].fields[0].collection_path.len(), 2);
    }
    #[test]
    fn long_item_is_reported_as_warning() {
        let p = parse(&parse_hex("fe 02 99 aa bb 05 01 75 08 95 01 81 02")).unwrap();
        assert_eq!(p.warnings.len(), 1);
        assert_eq!(p.long_items[0].tag, 0x99);
        assert_eq!(p.long_items[0].data_hex, "aa bb");
    }
    #[test]
    fn truncated_items_and_unbalanced_structures_are_errors() {
        assert!(parse(&[0x75]).unwrap_err().message.contains("truncated"));
        assert!(parse(&[0xa4]).unwrap_err().message.contains("Push"));
        assert!(parse(&[0xc0])
            .unwrap_err()
            .message
            .contains("End Collection"));
        assert!(parse(&[0xa1, 0x01])
            .unwrap_err()
            .message
            .contains("unclosed"));
        assert!(parse(&[0xfe, 0x02])
            .unwrap_err()
            .message
            .contains("long-item header"));
        assert!(parse(&[0xfe, 0x02, 0x99, 0xaa])
            .unwrap_err()
            .message
            .contains("truncated long item"));
    }

    #[test]
    fn rejects_empty_and_oversized_descriptors_but_retains_reportless_data() {
        assert!(parse(&[]).unwrap_err().message.contains("empty"));
        let oversized = vec![0x00; 4097];
        assert!(parse(&oversized)
            .unwrap_err()
            .message
            .contains("4096-byte limit"));
        assert!(parse(&[0xa1, 0x01, 0xc0]).is_ok());
    }

    #[test]
    fn accepts_descriptor_at_exact_size_limit() {
        let mut descriptor = vec![0x00; 4096];
        descriptor[0] = 0xa1;
        descriptor[1] = 0x01;
        descriptor[2] = 0xc0;
        assert_eq!(parse(&descriptor).unwrap().descriptor_length, 4096);
    }
    #[test]
    fn malformed_main_and_global_item_sizes_are_rejected() {
        assert!(parse(&parse_hex("75 08 95 01 80"))
            .unwrap_err()
            .message
            .contains("must contain 1, 2, or 4"));
        assert!(parse(&parse_hex("54"))
            .unwrap_err()
            .message
            .contains("Unit Exponent"));
    }
    #[test]
    fn reserved_items_are_reported_as_warnings() {
        let parsed = parse(&parse_hex("05 01 75 08 95 01 81 02 fd 01")).unwrap();
        assert_eq!(parsed.warnings.len(), 1);
        assert!(parsed.warnings[0].message.contains("reserved item type"));
    }
    #[test]
    fn report_id_zero_cannot_be_mixed_with_numbered_reports() {
        let d = parse_hex("75 08 95 01 81 02 85 01 81 02");
        assert!(parse(&d).unwrap_err().message.contains("Report ID 0"));
    }
    #[test]
    fn sha256_matches_known_vector() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn unit_exponent_designator_and_string_indices_are_retained() {
        let p = parse(&parse_hex(
            "05 01 09 06 a1 01 09 01 39 03 49 04 79 05 89 06 15 00 25 01 55 0e 65 11 75 08 95 01 81 02 c0",
        )).unwrap();
        let field = &p.reports[0].fields[0];
        assert_eq!(field.unit_exponent, -2);
        assert_eq!(field.unit, 0x11);
        assert_eq!(field.designator_indices, vec![3]);
        assert_eq!(field.string_indices, vec![5]);
    }

    #[test]
    fn designator_and_string_range_bounds_are_retained() {
        let p = parse(&parse_hex(
            "05 01 09 06 a1 01 49 03 59 04 89 05 99 06 15 00 25 01 75 08 95 01 81 02 c0",
        ))
        .unwrap();
        let field = &p.reports[0].fields[0];
        assert_eq!(field.designator_minimum, Some(3));
        assert_eq!(field.designator_maximum, Some(4));
        assert_eq!(field.string_minimum, Some(5));
        assert_eq!(field.string_maximum, Some(6));
    }

    #[test]
    fn captured_th80_keyboard_receiver_descriptors_parse_to_expected_reports() {
        let interface0 = parse(&parse_hex(KEYBOARD)).unwrap();
        let interface1 = parse(&parse_hex(IFACE1)).unwrap();
        let interface2 = parse(&parse_hex(VENDOR_FEATURE)).unwrap();
        assert_eq!(
            (
                interface0.input_report_count,
                interface0.output_report_count,
                interface0.feature_report_count
            ),
            (1, 1, 0)
        );
        assert_eq!(
            (
                interface1.input_report_count,
                interface1.output_report_count,
                interface1.feature_report_count
            ),
            (5, 0, 0)
        );
        assert_eq!(
            (
                interface2.input_report_count,
                interface2.output_report_count,
                interface2.feature_report_count
            ),
            (0, 0, 1)
        );
        let id_lengths: Vec<_> = interface1
            .reports
            .iter()
            .map(|r| (r.report_id, r.payload_bytes))
            .collect();
        assert_eq!(id_lengths, vec![(1, 15), (2, 1), (3, 2), (5, 3), (6, 7)]);
        assert_eq!(
            interface1
                .reports
                .iter()
                .find(|r| r.report_id == 2)
                .unwrap()
                .report_size_bits,
            None
        );
        assert_eq!(interface2.reports[0].payload_bytes, 64);
        assert_eq!(interface2.reports[0].transfer_bytes, 64);
        assert_eq!(
            interface0.sha256,
            "a5395cb7e86bebce11aecb4cc6d73d5d2b23d1c63f28df3c7c90069490213dd8"
        );
        assert_eq!(
            interface1.sha256,
            "3cc697106805cc7be08443be8dae9f4eda2069d7b5c8352e03abdea7e5a98c74"
        );
        assert_eq!(
            interface2.sha256,
            "350fea62e225e3d92e9db3bbbd408fcebe386c10405454be661d3d2c3229b0b5"
        );
    }
}
