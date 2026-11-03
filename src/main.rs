use hidapi::{DeviceInfo, HidApi};
use serde::Serialize;
use std::collections::BTreeMap;
use std::env;
use std::error::Error;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const VENDOR_ID: u16 = 0x3151;
const TH80_PRO_PID: u16 = 0x4010;
const RECEIVER_PID: u16 = 0x4011;

#[derive(Default, Clone)]
struct GlobalState {
    usage_page: u32,
    logical_min: i32,
    logical_max: i32,
    physical_min: i32,
    physical_max: i32,
    has_physical_min: bool,
    has_physical_max: bool,
    report_size: u32,
    report_count: u32,
    report_id: u8,
}

#[derive(Default, Serialize)]
struct ReportSummary {
    report_id: u8,
    input_bits: u64,
    output_bits: u64,
    feature_bits: u64,
}

#[derive(Default, Serialize)]
struct DescriptorSummary {
    usages: Vec<String>,
    collections: Vec<String>,
    reports: Vec<ReportSummary>,
    logical_ranges: Vec<String>,
    physical_ranges: Vec<String>,
    errors: Vec<String>,
}

#[derive(Serialize)]
struct DeviceRecord {
    vendor_id: u16,
    product_id: u16,
    release_number: u16,
    manufacturer: String,
    product: String,
    serial: String,
    usage_page: u16,
    usage: u16,
    interface_number: i32,
    hidapi_path: String,
    report_descriptor_path: Option<String>,
    report_descriptor_hex: Option<String>,
    descriptor: Option<DescriptorSummary>,
}

fn role(pid: u16) -> &'static str {
    match pid {
        TH80_PRO_PID => "physical TH80 Pro",
        RECEIVER_PID => "2.4 GHz receiver",
        _ => "other 0x3151 device",
    }
}

fn read_descriptor(path: &str) -> Option<(PathBuf, Vec<u8>)> {
    let name = Path::new(path).file_name()?.to_str()?;
    let descriptor_path =
        PathBuf::from(format!("/sys/class/hidraw/{name}/device/report_descriptor"));
    fs::read(&descriptor_path)
        .ok()
        .map(|bytes| (descriptor_path, bytes))
}

fn unsigned(bytes: &[u8]) -> u32 {
    bytes.iter().enumerate().fold(0u32, |value, (index, byte)| {
        value | ((*byte as u32) << (index * 8))
    })
}

fn signed(bytes: &[u8]) -> i32 {
    if bytes.is_empty() {
        return 0;
    }
    let value = unsigned(bytes);
    let bits = (bytes.len() * 8) as u32;
    ((value << (32 - bits)) as i32) >> (32 - bits)
}

fn usage_name(page: u32, usage: u32) -> String {
    let page_name = match page {
        0x01 => "Generic Desktop",
        0x07 => "Keyboard/Keypad",
        0x08 => "LED",
        0x09 => "Button",
        0x0c => "Consumer",
        0xff55 => "Vendor 0xff55",
        0xff66 => "Vendor 0xff66",
        0xffff => "Vendor 0xffff",
        _ => "Usage Page",
    };
    format!("{page_name} (page 0x{page:04x}), usage 0x{usage:04x}")
}

fn parse_descriptor(bytes: &[u8]) -> DescriptorSummary {
    let mut result = DescriptorSummary::default();
    let mut global = GlobalState::default();
    let mut stack: Vec<GlobalState> = Vec::new();
    let mut local_usages: Vec<u32> = Vec::new();
    let mut report_map: BTreeMap<u8, ReportSummary> = BTreeMap::new();
    let mut offset = 0usize;

    while offset < bytes.len() {
        let prefix = bytes[offset];
        offset += 1;
        if prefix == 0xfe {
            if offset + 2 > bytes.len() {
                result
                    .errors
                    .push(format!("truncated long item at {offset}"));
                break;
            }
            let length = bytes[offset] as usize;
            offset += 2;
            if offset + length > bytes.len() {
                result
                    .errors
                    .push(format!("truncated long item payload at {offset}"));
                break;
            }
            offset += length;
            continue;
        }

        let size_code = (prefix & 0x03) as usize;
        let size = match size_code {
            3 => 4,
            value => value,
        };
        if offset + size > bytes.len() {
            result
                .errors
                .push(format!("truncated short item at {}", offset - 1));
            break;
        }
        let data = &bytes[offset..offset + size];
        offset += size;
        let item_type = (prefix >> 2) & 0x03;
        let tag = (prefix >> 4) & 0x0f;
        let value = unsigned(data);

        match item_type {
            1 => match tag {
                0 => global.usage_page = value,
                1 => global.logical_min = signed(data),
                2 => {
                    global.logical_max = if global.logical_min < 0 {
                        signed(data)
                    } else {
                        value as i32
                    }
                }
                3 => {
                    global.physical_min = signed(data);
                    global.has_physical_min = true;
                }
                4 => {
                    global.physical_max = if global.physical_min < 0 {
                        signed(data)
                    } else {
                        value as i32
                    };
                    global.has_physical_max = true;
                }
                7 => global.report_size = value,
                8 => global.report_id = value as u8,
                9 => global.report_count = value,
                10 => stack.push(global.clone()),
                11 => {
                    if let Some(previous) = stack.pop() {
                        global = previous;
                    } else {
                        result.errors.push("global POP without PUSH".into());
                    }
                }
                _ => {}
            },
            2 => {
                if (0..=2).contains(&tag) {
                    let (page, usage) = if size == 4 {
                        (value >> 16, value & 0xffff)
                    } else {
                        (global.usage_page, value)
                    };
                    local_usages.push(usage);
                    result.usages.push(usage_name(page, usage));
                }
            }
            0 => match tag {
                10 => {
                    let usage = local_usages.first().copied().unwrap_or(0);
                    result
                        .collections
                        .push(usage_name(global.usage_page, usage));
                    local_usages.clear();
                }
                8..=11 => {
                    let bits = (global.report_size as u64) * (global.report_count as u64);
                    let entry =
                        report_map
                            .entry(global.report_id)
                            .or_insert_with(|| ReportSummary {
                                report_id: global.report_id,
                                ..ReportSummary::default()
                            });
                    match tag {
                        8 => entry.input_bits += bits,
                        9 => entry.output_bits += bits,
                        11 => entry.feature_bits += bits,
                        _ => {}
                    }
                    if global.report_size > 0 && global.report_count > 0 {
                        result.logical_ranges.push(format!(
                            "{}: {}..{} (size {} x count {})",
                            usage_name(
                                global.usage_page,
                                local_usages.first().copied().unwrap_or(0)
                            ),
                            global.logical_min,
                            global.logical_max,
                            global.report_size,
                            global.report_count
                        ));
                        if global.has_physical_min || global.has_physical_max {
                            result.physical_ranges.push(format!(
                                "{}: {}..{}",
                                usage_name(
                                    global.usage_page,
                                    local_usages.first().copied().unwrap_or(0)
                                ),
                                global.physical_min,
                                global.physical_max
                            ));
                        }
                    }
                    local_usages.clear();
                }
                12 => local_usages.clear(),
                _ => local_usages.clear(),
            },
            _ => {}
        }
    }

    for report in report_map.into_values() {
        result.reports.push(report);
    }
    if result.physical_ranges.is_empty() {
        result
            .physical_ranges
            .push("Not declared in this report descriptor".into());
    }
    result
}

fn record_device(info: &DeviceInfo) -> DeviceRecord {
    let path = info.path().to_string_lossy().into_owned();
    let descriptor = read_descriptor(&path);
    let (descriptor_path, descriptor_bytes) = match descriptor {
        Some((path, bytes)) => (Some(path), Some(bytes)),
        None => (None, None),
    };
    DeviceRecord {
        vendor_id: info.vendor_id(),
        product_id: info.product_id(),
        release_number: info.release_number(),
        manufacturer: info
            .manufacturer_string()
            .unwrap_or("(unavailable)")
            .to_string(),
        product: info.product_string().unwrap_or("(unavailable)").to_string(),
        serial: info.serial_number().unwrap_or("(unavailable)").to_string(),
        usage_page: info.usage_page(),
        usage: info.usage(),
        interface_number: info.interface_number(),
        hidapi_path: path,
        report_descriptor_path: descriptor_path.map(|value| value.display().to_string()),
        report_descriptor_hex: descriptor_bytes.as_ref().map(|value| hex(value)),
        descriptor: descriptor_bytes
            .as_ref()
            .map(|value| parse_descriptor(value)),
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn devices(api: &HidApi) -> Vec<DeviceRecord> {
    let mut seen = std::collections::BTreeSet::new();
    api.device_list()
        .filter(|info| {
            info.vendor_id() == VENDOR_ID
                && matches!(info.product_id(), TH80_PRO_PID | RECEIVER_PID)
                && seen.insert(info.path().to_bytes().to_vec())
        })
        .map(record_device)
        .collect()
}

fn print_devices(records: &[DeviceRecord]) {
    if records.is_empty() {
        println!("No TH80 Pro or receiver HID interfaces found.");
        return;
    }
    for item in records {
        println!(
            "{} {:04x}:{:04x} interface {}",
            role(item.product_id),
            item.vendor_id,
            item.product_id,
            item.interface_number
        );
        println!("  {} / {}", item.manufacturer, item.product);
        println!(
            "  release 0x{:04x}, usage page 0x{:04x}, usage 0x{:04x}",
            item.release_number, item.usage_page, item.usage
        );
        println!("  path {}", item.hidapi_path);
        if let Some(summary) = &item.descriptor {
            println!(
                "  report descriptor: {}",
                item.report_descriptor_path
                    .as_deref()
                    .unwrap_or("unknown path")
            );
            for collection in &summary.collections {
                println!("    collection: {collection}");
            }
            for usage in &summary.usages {
                println!("    usage: {usage}");
            }
            for report in &summary.reports {
                println!(
                    "    report ID {}: input {} bits, output {} bits, feature {} bits",
                    report.report_id, report.input_bits, report.output_bits, report.feature_bits
                );
            }
            for range in &summary.logical_ranges {
                println!("    logical: {range}");
            }
            for range in &summary.physical_ranges {
                println!("    physical: {range}");
            }
            for error in &summary.errors {
                println!("    parser: {error}");
            }
        } else {
            println!("  report descriptor unavailable (check sysfs permissions)");
        }
        println!();
    }
}

fn record_input(api: &HidApi, pid: u16, seconds: u64, output: &Path) -> Result<(), Box<dyn Error>> {
    let info = api
        .device_list()
        .find(|item| {
            item.vendor_id() == VENDOR_ID
                && item.product_id() == pid
                && item.usage_page() == 0x01
                && item.usage() == 0x06
        })
        .ok_or("requested boot-keyboard interface not found")?;
    let path = info.path().to_owned();
    let device = api.open_path(&path)?;
    device.set_blocking_mode(false)?;
    let file = fs::File::create(output)?;
    let mut writer = BufWriter::new(file);
    let start = std::time::Instant::now();
    let deadline = start + Duration::from_secs(seconds);
    let mut buffer = [0u8; 128];
    let mut count = 0u64;
    eprintln!("Recording raw keyboard input for {seconds}s from {} ({:04x}:{pid:04x}). This can contain typed text.", role(pid), VENDOR_ID);
    while std::time::Instant::now() < deadline {
        match device.read_timeout(&mut buffer, 100) {
            Ok(0) => {}
            Ok(length) => {
                let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
                let bytes = buffer[..length]
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<Vec<_>>();
                writeln!(writer, "{{\"timestamp_unix_ms\":{timestamp},\"interface\":{},\"length\":{length},\"data_hex\":\"{}\"}}", info.interface_number(), bytes.join(" "))?;
                count += 1;
            }
            Err(error) => return Err(Box::new(error)),
        }
    }
    writer.flush()?;
    println!("Saved {count} input reports to {}", output.display());
    Ok(())
}

fn usage() {
    eprintln!("TH80 read-only diagnostic CLI\n\nUSAGE:\n  th80ctl [list|descriptors|diagnostics [FILE]]\n  th80ctl record-input --pid 4010|4011 --seconds N --out FILE\n\nNo Feature Report reads/writes, output reports, control transfers, or flashing are implemented.\nInput recording is opt-in and captures raw key data.");
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    let api = HidApi::new()?;
    let command = args.first().map(String::as_str).unwrap_or("list");
    match command {
        "list" | "descriptors" => {
            let records = devices(&api);
            print_devices(&records);
            if command == "descriptors" {
                for item in &records {
                    println!(
                        "{} interface {} descriptor: {}",
                        role(item.product_id),
                        item.interface_number,
                        item.report_descriptor_hex
                            .as_deref()
                            .unwrap_or("unavailable")
                    );
                }
            }
        }
        "diagnostics" => {
            let records = devices(&api);
            let json = serde_json::to_string_pretty(&records)?;
            if let Some(path) = args.get(1) {
                fs::write(path, &json)?;
                println!("Wrote read-only diagnostic snapshot to {path}");
            } else {
                println!("{json}");
            }
        }
        "record-input" => {
            let get = |name: &str| {
                args.windows(2)
                    .find(|pair| pair[0] == name)
                    .map(|pair| pair[1].as_str())
            };
            let pid = match get("--pid").ok_or("missing --pid")? {
                "4010" => TH80_PRO_PID,
                "4011" => RECEIVER_PID,
                _ => return Err("--pid must be 4010 or 4011".into()),
            };
            let seconds = get("--seconds")
                .ok_or("missing --seconds")?
                .parse::<u64>()?;
            if seconds == 0 || seconds > 300 {
                return Err("--seconds must be between 1 and 300".into());
            }
            let output = PathBuf::from(get("--out").ok_or("missing --out")?);
            record_input(&api, pid, seconds, &output)?;
        }
        _ => usage(),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_unnumbered_vendor_feature_report() {
        let descriptor = [
            0x06, 0xff, 0xff, 0x09, 0x02, 0xa1, 0x01, 0x09, 0x02, 0x15, 0x80, 0x25, 0x7f, 0x95,
            0x40, 0x75, 0x08, 0xb1, 0x02, 0xc0,
        ];
        let parsed = parse_descriptor(&descriptor);
        assert_eq!(parsed.reports.len(), 1);
        assert_eq!(parsed.reports[0].report_id, 0);
        assert_eq!(parsed.reports[0].feature_bits, 512);
        assert!(parsed
            .logical_ranges
            .iter()
            .any(|range| range.contains("-128..127")));
        assert_eq!(
            parsed.physical_ranges,
            ["Not declared in this report descriptor"]
        );
    }

    #[test]
    fn parses_extended_report_ids_and_sizes() {
        let descriptor = [
            0x05, 0x0c, 0x09, 0x01, 0xa1, 0x01, 0x85, 0x03, 0x19, 0x00, 0x2a, 0x3c, 0x03, 0x15,
            0x00, 0x26, 0x3c, 0x03, 0x95, 0x01, 0x75, 0x10, 0x81, 0x00, 0xc0,
        ];
        let parsed = parse_descriptor(&descriptor);
        assert_eq!(parsed.reports.len(), 1);
        assert_eq!(parsed.reports[0].report_id, 3);
        assert_eq!(parsed.reports[0].input_bits, 16);
        assert!(parsed.collections[0].contains("Consumer"));
    }

    #[test]
    fn reports_truncated_items_without_panicking() {
        let parsed = parse_descriptor(&[0x06, 0xff]);
        assert!(!parsed.errors.is_empty());
    }
}
