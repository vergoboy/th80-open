use eframe::egui;

use crate::app::Th80App;

pub fn show(app: &mut Th80App, ui: &mut egui::Ui) {
    ui.heading("Application settings");
    ui.label("TH80 Studio preferences affect this desktop application only.");
    ui.add_space(12.0);

    let mut preferences_changed = false;
    ui.horizontal(|ui| {
        ui.label("Appearance");
        ui.separator();
        preferences_changed |= ui
            .selectable_value(&mut app.state.dark_mode, true, "Dark")
            .clicked();
        preferences_changed |= ui
            .selectable_value(&mut app.state.dark_mode, false, "Light")
            .clicked();
    });
    ui.add_space(12.0);

    ui.horizontal(|ui| {
        ui.label("Interface scale");
        preferences_changed |= ui
            .add(
                egui::Slider::new(&mut app.state.ui_scale, 0.8..=1.4)
                    .step_by(0.05)
                    .suffix("×"),
            )
            .changed();
        if ui.small_button("Reset").clicked() {
            app.state.ui_scale = 1.0;
            preferences_changed = true;
        }
    });

    ui.horizontal(|ui| {
        ui.label("Accent color");
        let mut color = egui::Color32::from_rgb(
            app.state.accent_color[0],
            app.state.accent_color[1],
            app.state.accent_color[2],
        );
        if ui.color_edit_button_srgba(&mut color).changed() {
            app.state.accent_color = [color.r(), color.g(), color.b()];
            preferences_changed = true;
        }
        ui.label("Active navigation, selection and focus accents");
    });

    ui.add_space(16.0);
    ui.separator();
    ui.heading("Local storage");
    ui.label("Profiles are stored as local JSON files. No device connection, firmware write, account, or cloud service is used.");
    ui.monospace(app.state.service.root().display().to_string());

    ui.add_space(12.0);
    ui.heading("Import and export");
    ui.label("Profile import and export use the native file chooser. Export is available from the top bar when a local profile is open.");
    ui.label("Default file chooser folder (optional)");
    preferences_changed |= ui
        .text_edit_singleline(&mut app.state.file_dialog_start_dir)
        .changed();
    if !app.state.file_dialog_start_dir.trim().is_empty()
        && !std::path::Path::new(app.state.file_dialog_start_dir.trim()).is_dir()
    {
        ui.colored_label(egui::Color32::YELLOW, "The selected folder does not currently exist; the chooser will use its system default.");
    }

    ui.add_space(12.0);
    ui.heading("Application behavior");
    ui.label("Edits remain in the active in-memory draft until saved. Repository conflicts are reported rather than silently overwritten.");

    if preferences_changed {
        ui.ctx().set_zoom_factor(app.state.ui_scale);
        Th80App::apply_visual_theme(ui.ctx(), app.state.dark_mode, app.state.accent_color);
        ui.ctx().request_repaint();
        if let Err(error) = app.save_preferences() {
            app.state.message = Some((
                false,
                format!("Could not save application preferences: {error}"),
            ));
        }
    }
}
