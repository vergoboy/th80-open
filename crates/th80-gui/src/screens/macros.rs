use eframe::egui;
use th80_core::{config::MacroStep, validation::validate_profile};

use crate::app::Th80App;

pub fn show(app: &mut Th80App, ui: &mut egui::Ui) {
    ui.heading("Macros");
    ui.label("Edit local macro definitions. These steps are application data, not a firmware packet format.");
    let Some(profile) = app.state.active.as_mut() else {
        ui.label("Open a profile to edit macros.");
        return;
    };
    ui.add_space(10.0);
    ui.columns(2, |columns| {
        columns[0].group(|ui| {
            ui.heading("Macro library");
            let items: Vec<(String, String)> = profile
                .configuration
                .macros
                .items
                .iter()
                .map(|m| (m.id.clone(), m.name.clone()))
                .collect();
            if items.is_empty() {
                ui.label("No macros defined.");
            }
            for (id, name) in &items {
                if ui
                    .selectable_label(
                        app.state.selected_macro.as_deref() == Some(id),
                        format!("{name}\n{id}"),
                    )
                    .clicked()
                {
                    app.state.selected_macro = Some(id.clone());
                }
            }
            ui.separator();
            ui.label("Macro ID");
            let response = ui.text_edit_singleline(&mut app.state.new_macro_id);
            super::set_accessible_label(ui, &response, "New macro ID");
            ui.label("Name");
            let response = ui.text_edit_singleline(&mut app.state.new_macro_name);
            super::set_accessible_label(ui, &response, "New macro name");
            if ui.button("＋ Create macro").clicked() {
                let id = app.state.new_macro_id.trim().to_owned();
                let name = app.state.new_macro_name.trim().to_owned();
                if id.is_empty()
                    || name.is_empty()
                    || profile
                        .configuration
                        .macros
                        .items
                        .iter()
                        .any(|m| m.id == id)
                {
                    app.state.message = Some((
                        false,
                        "Macro ID and name must be non-empty; IDs must be unique".into(),
                    ));
                } else {
                    profile
                        .configuration
                        .macros
                        .items
                        .push(th80_core::config::MacroDefinition {
                            id: id.clone(),
                            name,
                            steps: Vec::new(),
                        });
                    app.state.selected_macro = Some(id);
                    app.state.new_macro_id.clear();
                }
            }
            if let Some(selected_id) = app.state.selected_macro.clone() {
                if ui.button("Duplicate selected macro").clicked() {
                    let source = profile
                        .configuration
                        .macros
                        .items
                        .iter()
                        .find(|item| item.id == selected_id)
                        .cloned();
                    if let Some(source) = source {
                        let id = app.state.new_macro_id.trim().to_owned();
                        if id.is_empty()
                            || profile
                                .configuration
                                .macros
                                .items
                                .iter()
                                .any(|item| item.id == id)
                        {
                            app.state.message = Some((
                                false,
                                "Enter a new, unique macro ID before duplicating".into(),
                            ));
                        } else {
                            profile.configuration.macros.items.push(
                                th80_core::config::MacroDefinition {
                                    id: id.clone(),
                                    name: format!("{} copy", source.name),
                                    steps: source.steps,
                                },
                            );
                            app.state.selected_macro = Some(id.clone());
                            app.state.new_macro_id.clear();
                        }
                    }
                }
            }
        });
        columns[1].group(|ui| {
            ui.heading("Macro steps");
            let selected = app.state.selected_macro.clone();
            let Some(index) = selected.as_ref().and_then(|id| {
                profile
                    .configuration
                    .macros
                    .items
                    .iter()
                    .position(|item| &item.id == id)
            }) else {
                ui.label("Select or create a macro.");
                return;
            };
            let macro_item = &mut profile.configuration.macros.items[index];
            ui.label("Name");
            let response = ui.text_edit_singleline(&mut macro_item.name);
            super::set_accessible_label(ui, &response, "Selected macro name");
            ui.label(format!("{} step(s)", macro_item.steps.len()));
            if macro_item.steps.is_empty() {
                ui.label("This macro has no steps yet.");
            }
            egui::ScrollArea::vertical()
                .max_height(250.0)
                .show(ui, |ui| {
                    let mut remove = None;
                    let mut move_step = None;
                    let step_count = macro_item.steps.len();
                    for (step_index, step) in macro_item.steps.iter_mut().enumerate() {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(format!("{:02}", step_index + 1));
                            match step {
                                MacroStep::KeyDown { key } => {
                                    ui.label("Key down");
                                    let response = ui.add_sized(
                                        [
                                            ui.available_width().min(140.0),
                                            ui.spacing().interact_size.y,
                                        ],
                                        egui::TextEdit::singleline(key),
                                    );
                                    super::set_accessible_label(
                                        ui,
                                        &response,
                                        "Key down action name",
                                    );
                                }
                                MacroStep::KeyUp { key } => {
                                    ui.label("Key up");
                                    let response = ui.add_sized(
                                        [
                                            ui.available_width().min(140.0),
                                            ui.spacing().interact_size.y,
                                        ],
                                        egui::TextEdit::singleline(key),
                                    );
                                    super::set_accessible_label(
                                        ui,
                                        &response,
                                        "Key up action name",
                                    );
                                }
                                MacroStep::Delay { milliseconds } => {
                                    ui.label("Delay ms");
                                    let response = ui.add(egui::DragValue::new(milliseconds));
                                    super::set_accessible_label(
                                        ui,
                                        &response,
                                        "Macro delay in milliseconds",
                                    );
                                }
                                MacroStep::Text { value } => {
                                    ui.label("Text");
                                    let response = ui.add_sized(
                                        [
                                            ui.available_width().min(140.0),
                                            ui.spacing().interact_size.y,
                                        ],
                                        egui::TextEdit::singleline(value),
                                    );
                                    super::set_accessible_label(ui, &response, "Macro text step");
                                }
                            }
                            let remove_response = ui.small_button("×");
                            super::set_accessible_label(
                                ui,
                                &remove_response,
                                format!("Remove macro step {}", step_index + 1),
                            );
                            if remove_response.clicked() {
                                remove = Some(step_index);
                            }
                            if ui
                                .add_enabled(step_index > 0, egui::Button::new("Move up"))
                                .clicked()
                            {
                                move_step = Some((step_index, step_index - 1));
                            }
                            if ui
                                .add_enabled(
                                    step_index + 1 < step_count,
                                    egui::Button::new("Move down"),
                                )
                                .clicked()
                            {
                                move_step = Some((step_index, step_index + 1));
                            }
                        });
                    }
                    if let Some(remove) = remove {
                        macro_item.steps.remove(remove);
                    } else if let Some((from, to)) = move_step {
                        move_macro_step(&mut macro_item.steps, from, to);
                    }
                });
            ui.separator();
            ui.label("Key name (opaque local value)");
            let response = ui.text_edit_singleline(&mut app.state.macro_key);
            super::set_accessible_label(ui, &response, "Macro key name");
            ui.horizontal(|ui| {
                if ui.button("＋ Key down").clicked() {
                    macro_item.steps.push(MacroStep::KeyDown {
                        key: app.state.macro_key.clone(),
                    });
                }
                if ui.button("＋ Key up").clicked() {
                    macro_item.steps.push(MacroStep::KeyUp {
                        key: app.state.macro_key.clone(),
                    });
                }
            });
            ui.label("Delay in milliseconds");
            let response = ui.text_edit_singleline(&mut app.state.macro_delay);
            super::set_accessible_label(ui, &response, "New macro delay in milliseconds");
            if ui.button("＋ Delay").clicked() {
                match app.state.macro_delay.parse::<u32>() {
                    Ok(milliseconds) => macro_item.steps.push(MacroStep::Delay { milliseconds }),
                    Err(_) => {
                        app.state.message =
                            Some((false, "Delay must be an unsigned integer".into()))
                    }
                }
            }
            ui.label("Text step");
            let response = ui.text_edit_multiline(&mut app.state.macro_text);
            super::set_accessible_label(ui, &response, "New macro text step");
            if ui.button("＋ Text").clicked() && !app.state.macro_text.is_empty() {
                macro_item.steps.push(MacroStep::Text {
                    value: app.state.macro_text.clone(),
                });
            }
            if ui.button("Delete macro").clicked() {
                app.state.pending_delete_macro = selected.clone();
            }
        });
    });
    ui.add_space(10.0);
    let diagnostics = validate_profile(profile);
    if diagnostics.is_empty() {
        ui.colored_label(
            egui::Color32::from_rgb(115, 190, 164),
            "Macro/profile validation: no issues",
        );
    } else {
        ui.colored_label(
            egui::Color32::YELLOW,
            format!("{} validation issue(s)", diagnostics.len()),
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

fn move_macro_step(steps: &mut [MacroStep], from: usize, to: usize) -> bool {
    if from >= steps.len() || to >= steps.len() {
        return false;
    }
    steps.swap(from, to);
    true
}

#[cfg(test)]
mod tests {
    use super::move_macro_step;
    use th80_core::config::MacroStep;

    #[test]
    fn macro_step_reordering_preserves_order_and_rejects_invalid_indexes() {
        let mut steps = vec![
            MacroStep::KeyDown { key: "A".into() },
            MacroStep::Delay { milliseconds: 10 },
            MacroStep::KeyUp { key: "A".into() },
        ];
        assert!(move_macro_step(&mut steps, 2, 1));
        assert_eq!(
            steps,
            vec![
                MacroStep::KeyDown { key: "A".into() },
                MacroStep::KeyUp { key: "A".into() },
                MacroStep::Delay { milliseconds: 10 },
            ]
        );
        assert!(!move_macro_step(&mut steps, 3, 0));
        assert!(!move_macro_step(&mut steps, 0, 3));
    }
}
