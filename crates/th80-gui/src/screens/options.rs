use eframe::egui;
use std::collections::BTreeMap;

use crate::app::Th80App;

pub fn show(app: &mut Th80App, ui: &mut egui::Ui) {
    ui.heading("Keyboard options");
    ui.label(
        "Application-owned JSON values only. These options do not imply TH80 firmware settings.",
    );
    let Some(profile) = app.state.active.as_mut() else {
        ui.label("Open a profile to edit options.");
        return;
    };
    ui.add_space(12.0);
    ui.group(|ui| {
        ui.heading("Profile options");
        ui.label("Edit values as JSON. Existing values are preserved unless explicitly changed or removed.");
        let keys: Vec<String> = profile.configuration.options.values.keys().cloned().collect();
        for key in keys {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&key).strong());
                let current = profile.configuration.options.values.get(&key).cloned().unwrap_or(serde_json::Value::Null);
                let edit = app.state.option_edits.entry(key.clone()).or_insert_with(|| serde_json::to_string(&current).unwrap_or_else(|_| "null".into()));
                let response = ui.add(
                    egui::TextEdit::singleline(edit)
                        .desired_width(280.0)
                        .font(egui::TextStyle::Monospace),
                );
                super::set_accessible_label(ui, &response, format!("JSON value for option {key}"));
                super::set_accessible_description(
                    ui,
                    &response,
                    format!("Full JSON value: {edit}"),
                );
                response.on_hover_text(edit.clone());
                if ui.small_button("Update").clicked() {
                    match serde_json::from_str(edit) {
                        Ok(value) => { profile.configuration.options.values.insert(key.clone(), value); }
                        Err(error) => { app.state.message = Some((false, format!("Invalid JSON for '{key}': {error}"))); }
                    }
                }
                let remove = ui.small_button("×");
                super::set_accessible_label(ui, &remove, format!("Remove option {key}"));
                if remove.clicked() {
                    profile.configuration.options.values.remove(&key);
                    app.state.option_edits.remove(&key);
                }
            });
        }
        ui.separator();
        ui.label("New option key");
        let response = ui.text_edit_singleline(&mut app.state.option_key);
        super::set_accessible_label(ui, &response, "New option key");
        ui.label("JSON value");
        let response = ui.text_edit_singleline(&mut app.state.option_json);
        super::set_accessible_label(ui, &response, "New option JSON value");
        if ui.button("＋ Add option").clicked() {
            let key = app.state.option_key.trim().to_owned();
            match serde_json::from_str(&app.state.option_json) {
                Ok(value) if !key.is_empty() && !profile.configuration.options.values.contains_key(&key) => {
                    profile.configuration.options.values.insert(key, value);
                    app.state.option_key.clear();
                }
                Ok(_) => app.state.message = Some((false, "Option key must be non-empty and unique".into())),
                Err(error) => app.state.message = Some((false, format!("Invalid JSON value: {error}"))),
            }
        }
    });

    ui.add_space(12.0);
    ui.group(|ui| {
        ui.heading("Unknown profile extensions");
        ui.label("Unrecognized fields at supported extension boundaries are retained and shown read-only. Unknown action or layer schemas remain rejected until explicitly supported.");
        for (section, values) in [
            ("Profile", &profile.extensions),
            ("Metadata", &profile.metadata.extensions),
            ("Configuration", &profile.configuration.extensions),
            ("Keymap", &profile.configuration.keymap.extensions),
            ("Macros", &profile.configuration.macros.extensions),
            ("Lighting", &profile.configuration.lighting.extensions),
            ("Options", &profile.configuration.options.extensions),
        ] {
            show_extensions(ui, section, values);
        }
    });
}

fn show_extensions(ui: &mut egui::Ui, section: &str, values: &BTreeMap<String, serde_json::Value>) {
    ui.collapsing(section, |ui| {
        if values.is_empty() {
            ui.label("No retained extension fields.");
        } else {
            for (key, value) in values {
                ui.collapsing(key, |ui| {
                    let rendered =
                        serde_json::to_string_pretty(value).unwrap_or_else(|_| "null".into());
                    ui.add(egui::Label::new(egui::RichText::new(rendered).monospace()).wrap());
                });
            }
        }
    });
}
