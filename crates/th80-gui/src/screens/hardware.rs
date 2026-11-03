use std::time::Duration;

use eframe::egui;

use crate::app::Th80App;

pub fn show(app: &mut Th80App, ui: &mut egui::Ui) {
    if app
        .state
        .hardware_last_scan
        .is_none_or(|last| last.elapsed() >= Duration::from_secs(2))
    {
        app.state.refresh_hardware();
    }
    ui.ctx().request_repaint_after(Duration::from_secs(2));

    ui.heading("Hardware test mode");
    ui.label("Offline profiles remain independent of the physical keyboard.");
    ui.label("This page performs read-only USB sysfs and HID descriptor discovery. It does not open hidraw or send reports.");
    ui.horizontal(|ui| {
        if ui.button("Refresh device detection").clicked() {
            app.state.refresh_hardware();
        }
        ui.label("Auto-refresh: every 2 seconds while this page is open");
    });

    if let Some(error) = &app.state.hardware_error {
        ui.colored_label(
            egui::Color32::LIGHT_RED,
            format!("Discovery error: {error}"),
        );
        return;
    }

    let Some(snapshot) = &app.state.hardware_snapshot else {
        ui.label("Waiting for the first read-only discovery pass…");
        return;
    };

    let Some(keyboard) = &snapshot.keyboard else {
        ui.strong("No wired TH80 Pro USB device detected");
        if snapshot.receiver_detected {
            ui.label("A separate ROYUAN 2.4 GHz receiver is present; it is not selected as the wired keyboard.");
        }
        ui.label(
            "Connect the keyboard by USB-C and refresh. No hardware operation has been attempted.",
        );
        return;
    };

    ui.separator();
    ui.heading("Selected device");
    ui.horizontal(|ui| {
        ui.label("USB enumeration:");
        ui.colored_label(egui::Color32::LIGHT_GREEN, "Detected");
        ui.label("· sysfs only; no live command connection claimed");
    });
    ui.label(format!(
        "{} · {} · {}",
        keyboard
            .manufacturer
            .as_deref()
            .unwrap_or("Unknown manufacturer"),
        keyboard.product.as_deref().unwrap_or("Unknown product"),
        keyboard.stable_id
    ));
    ui.label(format!(
        "USB release: {} · negotiated speed: {} Mbps",
        keyboard.usb_release.as_deref().unwrap_or("unavailable"),
        keyboard.speed_mbps.as_deref().unwrap_or("unknown")
    ));
    if keyboard.usb_identity_verified {
        ui.colored_label(
            egui::Color32::LIGHT_GREEN,
            "USB identity matches ROYUAN / EPOMAKER TH80 Pro (3151:4010)",
        );
    } else {
        ui.colored_label(
            egui::Color32::YELLOW,
            "USB identity is incomplete: VID/PID matched, but manufacturer/product strings did not",
        );
    }
    ui.label("V504 ANSI firmware compatibility: NOT VERIFIED. bcdDevice is a USB descriptor release, not a firmware read-back.");

    ui.separator();
    ui.heading("HID interfaces");
    for interface in &keyboard.interfaces {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.strong(format!("Interface {}", interface.number));
                ui.label(format!(
                    "class {:02x} · subclass {:02x} · protocol {:02x} · {}",
                    interface.class, interface.subclass, interface.protocol, interface.role
                ));
            });
            if let (Some(length), Some(hash)) = (
                interface.descriptor_length,
                interface.descriptor_sha256.as_deref(),
            ) {
                ui.label(format!(
                    "Report descriptor: {length} bytes · SHA-256 {hash}"
                ));
            } else {
                ui.label("Report descriptor: unavailable");
            }
            if interface.feature_report_count > 0 {
                ui.label(format!(
                    "Feature reports: {} · aggregate transfer bytes: {} · report IDs: {}",
                    interface.feature_report_count,
                    interface
                        .feature_report_bytes
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "unknown".into()),
                    match interface.uses_report_ids {
                        Some(true) => "explicit",
                        Some(false) => "none (implicit report ID 0)",
                        None => "unknown",
                    }
                ));
            }
            if !interface.endpoints.is_empty() {
                let endpoints = interface
                    .endpoints
                    .iter()
                    .map(|endpoint| {
                        format!(
                            "0x{:02x} {} {}B/{}ms",
                            endpoint.address,
                            endpoint.transfer_type,
                            endpoint.max_packet_size,
                            endpoint.interval
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(" · ");
                ui.label(format!("Endpoints: {endpoints}"));
            }
            if interface.has_expected_feature_report() {
                ui.colored_label(
                    egui::Color32::LIGHT_GREEN,
                    "Descriptor match: interface 2 exposes one unnumbered 64-byte Feature report",
                );
            }
        });
    }

    ui.separator();
    ui.heading("Operation readiness");
    ui.label("Device detected: yes · USB identity: TH80 Pro family matched · firmware/revision target: unknown");
    ui.label(
        "Read-only descriptor inspection: complete · live transport connection: not established",
    );
    ui.label("RGB GET / SET: blocked pending passive reference capture of the official configurator's exact request and response");
    ui.label("Per-key RGB, key mapping, layers, and macros: blocked pending their independent reference contracts");
    ui.add_enabled(
        false,
        egui::Button::new("Apply to keyboard — unavailable until protocol evidence is verified"),
    );
    ui.label("No report has been sent and no keyboard settings have been changed. A USB enumeration or OS-level write acceptance will never be shown as a successful physical change.");
}
