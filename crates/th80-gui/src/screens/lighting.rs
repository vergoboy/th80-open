use eframe::egui;
use th80_core::{
    config::{LightingMode, PerKeyColor, RgbColor},
    validation::validate_profile,
};

use super::set_accessible_label;
use crate::app::Th80App;

pub fn show(app: &mut Th80App, ui: &mut egui::Ui) {
    ui.heading("Lighting");
    ui.label("These settings are stored locally in the profile. They have not been applied to the physical keyboard.");
    ui.label("Generic local profile values only. TH80 firmware lighting support and value ranges are not verified.");
    let Some(profile) = app.state.active.as_mut() else {
        ui.label("Open a profile to edit lighting settings.");
        return;
    };
    let lighting = &mut profile.configuration.lighting;
    let enabled_summary = match lighting.enabled {
        Some(true) => "Enabled in local profile",
        Some(false) => "Disabled in local profile",
        None => "Not specified in local profile",
    };
    ui.strong(enabled_summary);
    ui.add_space(12.0);
    ui.group(|ui| {
        ui.heading("General");
        let mut enabled_state = match lighting.enabled {
            None => 0,
            Some(true) => 1,
            Some(false) => 2,
        };
        let enabled_response = egui::ComboBox::from_label("Enabled state")
            .selected_text(match enabled_state {
                0 => "Unspecified",
                1 => "Enabled",
                _ => "Disabled",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut enabled_state, 0, "Unspecified");
                ui.selectable_value(&mut enabled_state, 1, "Enabled");
                ui.selectable_value(&mut enabled_state, 2, "Disabled");
            });
        set_accessible_label(
            ui,
            &enabled_response.response,
            "Local lighting enabled state",
        );
        lighting.enabled = match enabled_state {
            0 => None,
            1 => Some(true),
            _ => Some(false),
        };

        let mut mode = match &lighting.mode {
            None => "Unspecified".to_owned(),
            Some(LightingMode::Static) => "Static".into(),
            Some(LightingMode::Breathing) => "Breathing".into(),
            Some(LightingMode::Wave) => "Wave".into(),
            Some(LightingMode::Reactive) => "Reactive".into(),
            Some(LightingMode::Custom(value)) => format!("Custom: {value}"),
        };
        let response = egui::ComboBox::from_label("Mode")
            .selected_text(&mode)
            .show_ui(ui, |ui| {
                for value in [
                    "Unspecified",
                    "Static",
                    "Breathing",
                    "Wave",
                    "Reactive",
                    "Custom",
                ] {
                    ui.selectable_value(&mut mode, value.to_owned(), value);
                }
            });
        super::set_accessible_label(ui, &response.response, "Local lighting mode");
        if mode == "Unspecified" {
            lighting.mode = None;
        } else if mode == "Static" {
            lighting.mode = Some(LightingMode::Static);
        } else if mode == "Breathing" {
            lighting.mode = Some(LightingMode::Breathing);
        } else if mode == "Wave" {
            lighting.mode = Some(LightingMode::Wave);
        } else if mode == "Reactive" {
            lighting.mode = Some(LightingMode::Reactive);
        } else {
            let mut custom = match &lighting.mode {
                Some(LightingMode::Custom(value)) => value.clone(),
                _ => String::new(),
            };
            ui.horizontal(|ui| {
                ui.label("Custom mode name");
                let response = ui.text_edit_singleline(&mut custom);
                super::set_accessible_label(ui, &response, "Custom lighting mode name");
                if response.changed() {
                    lighting.mode = Some(LightingMode::Custom(custom.clone()));
                }
            });
            if custom.is_empty() {
                lighting.mode = Some(LightingMode::Custom(String::new()));
            }
        }

        let mut brightness = lighting.brightness.unwrap_or(50);
        let mut has_brightness = lighting.brightness.is_some();
        if ui.checkbox(&mut has_brightness, "Set brightness").changed() {
            lighting.brightness = has_brightness.then_some(brightness);
        }
        if lighting.brightness.is_some() {
            ui.add(
                egui::Slider::new(&mut brightness, 0..=100).text("Brightness · normalized 0–100"),
            );
            lighting.brightness = Some(brightness);
        }
        let mut speed = lighting.speed.unwrap_or(50);
        let mut has_speed = lighting.speed.is_some();
        if ui.checkbox(&mut has_speed, "Set speed").changed() {
            lighting.speed = has_speed.then_some(speed);
        }
        if lighting.speed.is_some() {
            ui.add(egui::Slider::new(&mut speed, 0..=100).text("Speed · normalized 0–100"));
            lighting.speed = Some(speed);
        }

        let mut has_color = lighting.color.is_some();
        if ui.checkbox(&mut has_color, "Set primary color").changed() {
            lighting.color = has_color.then_some(RgbColor {
                red: 255,
                green: 255,
                blue: 255,
            });
        }
        if let Some(color) = lighting.color.as_mut() {
            let mut egui_color = egui::Color32::from_rgb(color.red, color.green, color.blue);
            ui.horizontal(|ui| {
                ui.label("Primary color");
                let response = ui.color_edit_button_srgba(&mut egui_color);
                super::set_accessible_label(ui, &response, "Primary RGB color");
                if response.changed() {
                    *color = RgbColor {
                        red: egui_color.r(),
                        green: egui_color.g(),
                        blue: egui_color.b(),
                    };
                }
            });
        } else {
            ui.label("Primary color: Not specified in local profile");
        }
    });

    ui.add_space(10.0);
    ui.group(|ui| {
        ui.heading("Per-position colors");
        let mut remove_color = None;
        for (index, entry) in lighting.per_key.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.label(format!("{}.", index + 1));
                let response = ui.text_edit_singleline(&mut entry.position);
                super::set_accessible_label(
                    ui,
                    &response,
                    format!("Logical position for color entry {}", index + 1),
                );
                let mut color =
                    egui::Color32::from_rgb(entry.color.red, entry.color.green, entry.color.blue);
                let color_response = ui.color_edit_button_srgba(&mut color);
                super::set_accessible_label(
                    ui,
                    &color_response,
                    format!("RGB color for logical position {}", entry.position),
                );
                if color_response.changed() {
                    entry.color = RgbColor {
                        red: color.r(),
                        green: color.g(),
                        blue: color.b(),
                    };
                }
                let remove = ui.small_button("×");
                super::set_accessible_label(
                    ui,
                    &remove,
                    format!("Remove color for logical position {}", entry.position),
                );
                if remove.clicked() {
                    remove_color = Some(index);
                }
            });
        }
        if let Some(index) = remove_color {
            lighting.per_key.remove(index);
        }
        ui.horizontal(|ui| {
            let response = ui.text_edit_singleline(&mut app.state.lighting_position);
            super::set_accessible_label(ui, &response, "New logical position for color");
            if ui.button("Add logical position color").clicked() {
                let position = app.state.lighting_position.trim().to_owned();
                if !position.is_empty()
                    && !lighting
                        .per_key
                        .iter()
                        .any(|entry| entry.position == position)
                {
                    lighting.per_key.push(PerKeyColor {
                        position,
                        color: RgbColor {
                            red: 255,
                            green: 255,
                            blue: 255,
                        },
                    });
                    app.state.lighting_position.clear();
                } else {
                    app.state.message =
                        Some((false, "Position must be non-empty and unique".into()));
                }
            }
        });
    });

    ui.add_space(10.0);
    ui.group(|ui| {
        ui.heading("Effect parameters");
        ui.label("Opaque JSON parameters are retained without assigning device-specific meaning.");
        let entries: Vec<(String, String)> = lighting
            .effect_parameters
            .iter()
            .map(|(key, value)| {
                (
                    key.clone(),
                    serde_json::to_string_pretty(value).unwrap_or_else(|_| "null".into()),
                )
            })
            .collect();
        for (key, value) in entries {
            ui.horizontal(|ui| {
                ui.label(&key);
                ui.label(egui::RichText::new(value).small().monospace());
                let remove = ui.small_button("×");
                super::set_accessible_label(ui, &remove, format!("Remove effect parameter {key}"));
                if remove.clicked() {
                    lighting.effect_parameters.remove(&key);
                }
            });
        }
        ui.horizontal(|ui| {
            let response = ui.text_edit_singleline(&mut app.state.lighting_parameter_key);
            super::set_accessible_label(ui, &response, "Lighting effect parameter name");
            let response = ui.add(
                egui::TextEdit::singleline(&mut app.state.lighting_parameter_json)
                    .desired_width(190.0),
            );
            super::set_accessible_label(ui, &response, "Lighting effect parameter JSON value");
            if ui.button("Add / update JSON value").clicked() {
                let key = app.state.lighting_parameter_key.trim().to_owned();
                match serde_json::from_str(&app.state.lighting_parameter_json) {
                    Ok(value) if !key.is_empty() => {
                        lighting.effect_parameters.insert(key, value);
                    }
                    _ => {
                        app.state.message = Some((
                            false,
                            "Provide a non-empty parameter name and valid JSON value".into(),
                        ))
                    }
                }
            }
        });
    });

    let diagnostics = validate_profile(profile);
    if !diagnostics.is_empty() {
        ui.add_space(8.0);
        ui.colored_label(
            egui::Color32::YELLOW,
            format!("{} profile validation issue(s)", diagnostics.len()),
        );
        for item in diagnostics {
            ui.label(format!(
                "{} · {} · {}",
                item.code,
                item.path.as_deref().unwrap_or("profile"),
                item.message
            ));
        }
    }
}
