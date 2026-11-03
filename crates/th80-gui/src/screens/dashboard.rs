use eframe::egui;

use crate::app::Th80App;

pub fn show(app: &mut Th80App, ui: &mut egui::Ui) {
    ui.heading("Your profiles");
    ui.label(
        "Create, import and edit local TH80 configuration profiles. Nothing is sent to a keyboard.",
    );
    ui.add_space(18.0);

    ui.columns(2, |columns| {
        columns[0].group(|ui| {
            ui.heading("Create profile");
            ui.label("Start with an empty offline profile.");
            ui.add_space(8.0);
            let response = ui.text_edit_singleline(&mut app.state.new_profile_name);
            super::set_accessible_label(ui, &response, "New profile name");
            if ui.button("＋  Create profile").clicked() {
                app.state.create_profile();
            }
        });
        columns[1].group(|ui| {
            ui.heading("Import profile");
            ui.label("Import a previously exported TH80 profile JSON file.");
            ui.add_space(8.0);
            if ui
                .add_enabled(
                    !app.file_dialog_pending,
                    egui::Button::new("⇧  Import JSON…"),
                )
                .clicked()
            {
                app.import_dialog();
            }
        });
    });

    ui.add_space(18.0);
    ui.horizontal(|ui| {
        ui.heading("Recent profiles");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("↻ Refresh").clicked() {
                app.state.refresh_profiles();
            }
        });
    });
    ui.label("Most recently modified profiles appear first.");
    ui.separator();

    let profiles = app.state.profiles.clone();
    if profiles.is_empty() {
        ui.add_space(28.0);
        ui.vertical_centered(|ui| {
            ui.heading("No profiles yet");
            ui.label("Create a profile or import a JSON backup to get started.");
        });
    } else {
        egui::ScrollArea::vertical().show(ui, |ui| {
            for profile in profiles {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.strong(&profile.name);
                            ui.label(
                                egui::RichText::new(format!(
                                    "{} · {} layer(s) · {} macro(s)",
                                    profile.id.0,
                                    profile.configuration.keymap.layers.len(),
                                    profile.configuration.macros.items.len()
                                ))
                                .small()
                                .color(egui::Color32::GRAY),
                            );
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Open profile").clicked() {
                                app.state.open(&profile.id);
                            }
                            if ui.button("Duplicate").clicked() {
                                app.state.duplicate_profile(&profile.id);
                            }
                        });
                    });
                    if !profile.description.is_empty() {
                        ui.label(&profile.description);
                    }
                });
            }
        });
    }
}
