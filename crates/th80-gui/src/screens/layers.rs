use eframe::egui;
use th80_core::config::{Action, Layer};

use crate::{app::Th80App, state::Page};

pub fn show(app: &mut Th80App, ui: &mut egui::Ui) {
    ui.heading("Layers");
    ui.label("Layers are named sets of logical key assignments. No physical matrix is assumed.");
    let Some(profile) = app.state.active.as_mut() else {
        ui.label("Open a profile to manage its layers.");
        return;
    };
    ui.add_space(12.0);
    ui.columns(2, |columns| {
        columns[0].group(|ui| {
            ui.heading("Layer list");
            let layers: Vec<(String, String)> = profile.configuration.keymap.layers.iter().map(|l| (l.id.clone(), l.name.clone())).collect();
            if layers.is_empty() { ui.label("No layers defined yet."); }
            for (id, name) in &layers {
                if ui.selectable_label(app.state.selected_layer.as_deref() == Some(id), format!("{name}\n{id}")).clicked() {
                    app.state.selected_layer = Some(id.clone());
                    app.state.selected_position = None;
                }
            }
            ui.separator();
            ui.label("New layer ID");
            let response = ui.text_edit_singleline(&mut app.state.new_layer_id);
            super::set_accessible_label(ui, &response, "New layer ID");
            ui.label("Display name");
            let response = ui.text_edit_singleline(&mut app.state.new_layer_name);
            super::set_accessible_label(ui, &response, "New layer display name");
            if ui.button("＋ Add layer").clicked() {
                let id = app.state.new_layer_id.trim().to_owned();
                let name = app.state.new_layer_name.trim().to_owned();
                if id.is_empty() || name.is_empty() || profile.configuration.keymap.layers.iter().any(|l| l.id == id) {
                    app.state.message = Some((false, "Layer ID and name must be non-empty; IDs must be unique".into()));
                } else {
                    profile.configuration.keymap.layers.push(Layer { id: id.clone(), name, positions: Default::default() });
                    app.state.selected_layer = Some(id);
                    app.state.new_layer_id.clear();
                    app.state.new_layer_name.clear();
                }
            }
        });
        columns[1].group(|ui| {
            ui.heading("Selected layer");
            let selected = app.state.selected_layer.clone();
            let Some(index) = selected.as_ref().and_then(|id| profile.configuration.keymap.layers.iter().position(|layer| &layer.id == id)) else {
                ui.label("Select a layer or create one.");
                return;
            };
            let layer = &mut profile.configuration.keymap.layers[index];
            ui.label(format!("Layer ID: {}", layer.id));
            ui.label("Name");
            let response = ui.text_edit_singleline(&mut layer.name);
            super::set_accessible_label(ui, &response, "Selected layer name");
            ui.label(format!("{} logical position(s)", layer.positions.len()));
            ui.separator();
            let layer_id = layer.id.clone();
            let referenced = profile.configuration.keymap.layers.iter().any(|other| other.positions.values().any(|assignment| matches!(&assignment.action, Action::LayerSwitch { layer_id: target } | Action::LayerMomentary { layer_id: target } if target == &layer_id)));
            if referenced {
                ui.colored_label(egui::Color32::YELLOW, "This layer is referenced by another assignment. Remove those references before deleting it.");
            }
            if ui.add_enabled(!referenced, egui::Button::new("Remove layer")).clicked() {
                profile.configuration.keymap.layers.remove(index);
                app.state.selected_layer = profile.configuration.keymap.layers.first().map(|l| l.id.clone());
                app.state.selected_position = None;
            }
            if ui.button("Edit assignments →").clicked() { app.state.page = Page::Keymap; }
        });
    });
}
