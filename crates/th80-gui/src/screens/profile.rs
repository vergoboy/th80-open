use eframe::egui;
use th80_core::{
    config::ConfigurationProvenance,
    validation::{assess_device_compatibility, DeviceCompatibility},
    DeviceKind,
};

use crate::app::Th80App;

pub fn show(app: &mut Th80App, ui: &mut egui::Ui) {
    ui.heading("Profile details");
    let dirty = app.state.dirty();
    let service = app.state.service.clone();
    let Some(profile) = app.state.active.as_mut() else {
        ui.label("Open a profile to view its details.");
        return;
    };
    let profile_id = profile.id.clone();
    ui.add_space(10.0);
    ui.group(|ui| {
        ui.heading("Identity");
        ui.label(format!("Profile ID: {}", profile.id.0));
        ui.label(format!("Schema version: {}", profile.schema_version));
        ui.label("Name");
        let response = ui.text_edit_singleline(&mut profile.name);
        super::set_accessible_label(ui, &response, "Profile name");
        ui.label("Description");
        let response = ui.add(egui::TextEdit::multiline(&mut profile.description).desired_rows(3));
        super::set_accessible_label(ui, &response, "Profile description");
        let mut tags = profile.metadata.tags.join(", ");
        ui.label("Tags (comma separated)");
        let response = ui.text_edit_singleline(&mut tags);
        super::set_accessible_label(ui, &response, "Profile tags, comma separated");
        if response.changed() {
            profile.metadata.tags = tags
                .split(',')
                .map(str::trim)
                .filter(|tag| !tag.is_empty())
                .map(str::to_owned)
                .collect();
        }
    });
    ui.add_space(10.0);
    ui.group(|ui| {
        ui.heading("Target model");
        let mut selected = match profile.device_model {
            None => "Unspecified",
            Some(DeviceKind::Th80Pro) => "EPOMAKER TH80 Pro",
            Some(DeviceKind::RoyuanReceiver) => "ROYUAN receiver",
        };
        let response = egui::ComboBox::from_label("Associated model")
            .selected_text(selected)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut selected, "Unspecified", "Unspecified");
                ui.selectable_value(&mut selected, "EPOMAKER TH80 Pro", "EPOMAKER TH80 Pro");
                ui.selectable_value(&mut selected, "ROYUAN receiver", "ROYUAN receiver");
            });
        super::set_accessible_label(ui, &response.response, "Associated model");
        profile.device_model = match selected {
            "EPOMAKER TH80 Pro" => Some(DeviceKind::Th80Pro),
            "ROYUAN receiver" => Some(DeviceKind::RoyuanReceiver),
            _ => None,
        };
        let compatibility = assess_device_compatibility(profile, DeviceKind::Th80Pro);
        match compatibility.status {
            DeviceCompatibility::NotVerified => {
                ui.colored_label(egui::Color32::YELLOW, "Compatibility: NOT VERIFIED")
            }
            DeviceCompatibility::ModelMismatch => {
                ui.colored_label(egui::Color32::YELLOW, "Compatibility: MODEL MISMATCH")
            }
        };
        ui.label(compatibility.explanation);
    });
    ui.add_space(10.0);
    ui.group(|ui| {
        ui.heading("Local state");
        ui.label(format!("Provenance: {:?}", profile.provenance));
        ui.label(format!(
            "Created: {}",
            format_timestamp(profile.metadata.created_at_unix_ms)
        ));
        ui.label(format!(
            "Last modified: {}",
            format_timestamp(profile.metadata.modified_at_unix_ms)
        ));
        ui.label(if dirty {
            "Unsaved in-memory edits"
        } else {
            "Matches the last saved copy"
        });
        if profile.provenance == ConfigurationProvenance::DeviceReported
            || profile.provenance == ConfigurationProvenance::DeviceApplied
        {
            ui.colored_label(
                egui::Color32::YELLOW,
                "This provenance is not supported by the offline application validator.",
            );
        }
    });
    ui.add_space(12.0);
    ui.horizontal(|ui| {
        if ui.button("Validate profile").clicked() {
            match service.validate(profile) {
                Ok(()) => app.state.message = Some((true, "Profile passes local application validation only. Firmware compatibility remains unverified.".into())),
                Err(error) => app.state.message = Some((false, crate::state::format_error(&error))),
            }
        }
        if ui.button("Delete profile…").clicked() {
            app.state.pending_delete_profile = Some(profile_id.clone());
        }
    });
}

fn format_timestamp(unix_ms: u64) -> String {
    let seconds = (unix_ms / 1_000).min(i64::MAX as u64) as i64;
    let days = seconds.div_euclid(86_400);
    let day_seconds = seconds.rem_euclid(86_400);

    // Gregorian civil date conversion, using UTC and no additional runtime dependency.
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 }.div_euclid(146_097);
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    let hour = day_seconds / 3_600;
    let minute = (day_seconds % 3_600) / 60;
    let second = day_seconds % 60;

    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02} UTC")
}

#[cfg(test)]
mod tests {
    use super::format_timestamp;

    #[test]
    fn formats_profile_timestamps_as_readable_utc_dates() {
        assert_eq!(format_timestamp(0), "1970-01-01 00:00:00 UTC");
        assert_eq!(
            format_timestamp(1_700_000_000_000),
            "2023-11-14 22:13:20 UTC"
        );
    }
}
