use eframe::egui;
use th80_core::config::{Action, KeyAssignment};

use crate::{app::Th80App, state::Page};

pub fn show(app: &mut Th80App, ui: &mut egui::Ui) {
    let Some(profile) = app.state.active.as_mut() else {
        empty_layout(
            ui,
            "Choose a local profile",
            "Select a profile in the header or create one under Config → Profile library.",
        );
        return;
    };
    if profile.configuration.keymap.layers.is_empty() {
        empty_layout(
            ui,
            "No layers in this profile",
            "Create a logical layer to begin assigning positions.",
        );
        if ui.button("Create layer →").clicked() {
            app.state.page = Page::Layers;
        }
        return;
    }

    let layers = profile.configuration.keymap.layers.clone();
    let mut selected = app
        .state
        .selected_layer
        .clone()
        .unwrap_or_else(|| layers[0].id.clone());
    let current_index = layers
        .iter()
        .position(|layer| layer.id == selected)
        .unwrap_or(0);
    let focused_widget = ui.ctx().memory(|memory| memory.focused());
    let text_input_focused =
        app.state.layer_rename || focused_widget == Some(egui::Id::new("new-logical-position"));
    let can_navigate_layer = !text_input_focused && !app.state.picker_open;
    let (left, right, wheel) = ui.input(|input| {
        (
            input.key_pressed(egui::Key::ArrowLeft),
            input.key_pressed(egui::Key::ArrowRight),
            input
                .raw
                .events
                .iter()
                .filter_map(|event| match event {
                    egui::Event::MouseWheel { delta, .. } => Some(delta.y),
                    _ => None,
                })
                .sum::<f32>(),
        )
    });
    let mut next_index = current_index;
    if can_navigate_layer {
        if left {
            next_index = next_index.saturating_sub(1);
        }
        if right {
            next_index = (next_index + 1).min(layers.len() - 1);
        }
    }
    if next_index != current_index {
        selected = layers[next_index].id.clone();
    }
    if app.state.selected_layer.as_deref() != Some(selected.as_str()) {
        app.state.selected_position = None;
        app.state.layer_rename = false;
    }
    app.state.selected_layer = Some(selected.clone());
    let layer_index = profile
        .configuration
        .keymap
        .layers
        .iter()
        .position(|layer| layer.id == selected)
        .unwrap_or(0);

    ui.horizontal(|ui| {
        ui.add_space(4.0);
        let previous = ui.add_enabled(
            layer_index > 0,
            egui::Button::new("‹").min_size([34.0, 34.0].into()),
        );
        super::set_accessible_label(ui, &previous, "Previous logical layer");
        if previous.clicked() && layer_index > 0 {
            app.state.selected_layer = Some(layers[layer_index - 1].id.clone());
            app.state.selected_position = None;
        }
        if app.state.layer_rename {
            let response = ui.add(
                egui::TextEdit::singleline(
                    &mut profile.configuration.keymap.layers[layer_index].name,
                )
                .desired_width(220.0),
            );
            super::set_accessible_label(ui, &response, "Rename active layer");
            if response.lost_focus()
                || ui.input(|input| {
                    input.key_pressed(egui::Key::Enter) || input.key_pressed(egui::Key::Escape)
                })
            {
                app.state.layer_rename = false;
            }
        } else {
            let name = &profile.configuration.keymap.layers[layer_index].name;
            let title =
                ui.add(egui::Button::new(egui::RichText::new(name).size(23.0)).frame(false));
            super::set_accessible_label(
                ui,
                &title,
                format!("Active layer: {name}. Activate to rename."),
            );
            if title.clicked() {
                app.state.layer_rename = true;
            }
        }
        let next = ui.add_enabled(
            layer_index + 1 < layers.len(),
            egui::Button::new("›").min_size([34.0, 34.0].into()),
        );
        super::set_accessible_label(ui, &next, "Next logical layer");
        if next.clicked() && layer_index + 1 < layers.len() {
            app.state.selected_layer = Some(layers[layer_index + 1].id.clone());
            app.state.selected_position = None;
        }
        ui.label(
            egui::RichText::new(format!("{} / {}", layer_index + 1, layers.len()))
                .small()
                .weak(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Layer operations").clicked() {
                app.state.page = Page::Layers;
            }
        });
    });
    ui.add_space(2.0);
    ui.label(
        egui::RichText::new(
            "Logical positions · profile-defined slots, not verified physical switch geometry",
        )
        .small()
        .weak(),
    );
    ui.add_space(12.0);

    let positions: Vec<(String, Action)> = profile.configuration.keymap.layers[layer_index]
        .positions
        .iter()
        .map(|(id, assignment)| (id.clone(), assignment.action.clone()))
        .collect();
    let selected_position = app.state.selected_position.clone();
    let mut picker_request: Option<(String, String, Action)> = None;
    let content_size = ui.available_size();
    let board_width = (content_size.x * 0.68)
        .min(content_size.x - 280.0)
        .max(360.0);
    let inspector_width = (content_size.x - board_width - ui.spacing().item_spacing.x).max(240.0);
    ui.horizontal_top(|ui| {
        ui.allocate_ui_with_layout(
            egui::vec2(board_width, content_size.y),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
            ui.horizontal(|ui| {
                ui.heading("Layout");
                ui.label(egui::RichText::new(format!("{} positions", positions.len())).small().weak());
            });
            ui.add_space(8.0);
            ui.set_width(ui.available_width());
            let board = egui::Frame::group(ui.style()).inner_margin(egui::Margin::same(18)).show(ui, |ui| {
                egui::ScrollArea::vertical().id_salt("logical-position-board").max_height(500.0).show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        for (position, action) in &positions {
                            let is_selected = selected_position.as_deref() == Some(position.as_str());
                            let legend = action_legend(action);
                            let response = ui.add_sized([88.0, 56.0], egui::Button::new(
                                egui::RichText::new(format!("{position}\n{}", legend)).size(13.0)
                            ).selected(is_selected));
                            super::set_accessible_label(ui, &response, format!("Select logical position {position}: {legend}"));
                            super::set_accessible_description(ui, &response, format!("Layer {}. Activate to inspect this logical position; double activate to edit its action.", layers[layer_index].name));
                            if response.clicked() { app.state.selected_position = Some(position.clone()); }
                            if response.double_clicked() {
                                app.state.selected_position = Some(position.clone());
                                picker_request = Some((selected.clone(), position.clone(), action.clone()));
                            }
                        }
                        if positions.is_empty() { ui.label("No logical positions yet. Add a slot below."); }
                    });
                });
            });
            let pointer_over_board = ui.input(|input| {
                input.pointer.hover_pos().is_some_and(|position| {
                    position.x >= board.response.rect.left()
                        && position.x <= board.response.rect.left() + board_width
                        && position.y >= board.response.rect.top()
                        && position.y <= board.response.rect.bottom()
                })
            });
            if can_navigate_layer && pointer_over_board && wheel.abs() > 0.5 {
                let next = if wheel < 0.0 { (layer_index + 1).min(layers.len() - 1) } else { layer_index.saturating_sub(1) };
                if next != layer_index {
                    app.state.selected_layer = Some(layers[next].id.clone());
                    app.state.selected_position = None;
                }
            }
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                let response = ui.add_sized([230.0, 30.0], egui::TextEdit::singleline(&mut app.state.new_position_id).id(egui::Id::new("new-logical-position")).hint_text("New logical position ID"));
                super::set_accessible_label(ui, &response, "New logical position ID");
                if ui.button("＋ Add position").clicked() {
                    let id = app.state.new_position_id.trim().to_owned();
                    if !id.is_empty() && !profile.configuration.keymap.layers[layer_index].positions.contains_key(&id) {
                        profile.configuration.keymap.layers[layer_index].positions.insert(id.clone(), KeyAssignment { action: Action::Unassigned });
                        app.state.selected_position = Some(id);
                        app.state.new_position_id.clear();
                    } else { app.state.message = Some((false, "Position ID must be non-empty and unique within this layer".into())); }
                }
            });
            },
        );

        ui.allocate_ui_with_layout(
            egui::vec2(inspector_width, content_size.y),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
            ui.heading("Selected position");
            ui.add_space(8.0);
            let Some(position) = selected_position else {
                ui.label("Select a logical position to inspect its assignment.");
                return;
            };
            let Some(action) = profile.configuration.keymap.layers[layer_index].positions.get(&position).map(|assignment| assignment.action.clone()) else {
                ui.label("The selected position is no longer present.");
                return;
            };
            let legend = action_legend(&action);
            let details = action_details(&action);
            egui::Frame::group(ui.style()).inner_margin(egui::Margin::same(16)).show(ui, |ui| {
                ui.label(egui::RichText::new(&position).size(20.0).strong());
                ui.label(egui::RichText::new("LOGICAL POSITION").small().weak());
                ui.separator();
                ui.label("Current action");
                ui.label(egui::RichText::new(legend).size(18.0).strong());
                ui.add_space(6.0);
                ui.label(details);
                ui.add_space(14.0);
                if ui.button("Edit action…").clicked() { picker_request = Some((selected.clone(), position.clone(), action.clone())); }
                if ui.button("Clear assignment").clicked() {
                    if let Some(value) = profile.configuration.keymap.layers[layer_index].positions.get_mut(&position) { value.action = Action::Unassigned; }
                }
                if ui.button("Remove logical position").clicked() {
                    profile.configuration.keymap.layers[layer_index].positions.remove(&position);
                    app.state.selected_position = None;
                }
            });
            },
        );
    });
    if let Some((layer, position, action)) = picker_request {
        begin_picker(app, &layer, &position, &action);
    }
    ui.add_space(10.0);
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Hints").small().strong());
        ui.label(egui::RichText::new("Click a position to inspect · Double-click or Edit action to assign · ← / → or wheel changes layer · Click layer name to rename").small().weak());
    });
}

fn begin_picker(app: &mut Th80App, layer: &str, position: &str, action: &Action) {
    app.state.picker_target_layer = Some(layer.to_owned());
    app.state.picker_target_position = Some(position.to_owned());
    app.state.picker_action_draft = Some(action.clone());
    app.state.picker_kind = action_kind(action).to_owned();
    app.state.picker_category = category_for_kind(&app.state.picker_kind).to_owned();
    app.state.picker_query.clear();
    app.state.picker_open = true;
}

fn empty_layout(ui: &mut egui::Ui, title: &str, detail: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(80.0);
        ui.heading(title);
        ui.label(detail);
    });
}

pub fn action_kind(action: &Action) -> &'static str {
    match action {
        Action::Key { .. } => "Key",
        Action::Modifier { .. } => "Modifier",
        Action::Combination { .. } => "Combination",
        Action::LayerSwitch { .. } => "Layer switch",
        Action::LayerMomentary { .. } => "Momentary layer",
        Action::Macro { .. } => "Macro",
        Action::Transparent => "Transparent",
        Action::Unassigned => "Unassigned",
    }
}

pub fn category_for_kind(kind: &str) -> &'static str {
    match kind {
        "Key" => "Keys",
        "Modifier" | "Combination" => "Modifiers",
        "Layer switch" | "Momentary layer" => "Layers",
        "Macro" => "Macros",
        _ => "Other",
    }
}

pub fn action_legend(action: &Action) -> String {
    match action {
        Action::Key { code } => {
            if code.is_empty() {
                "Key · unset".into()
            } else {
                code.clone()
            }
        }
        Action::Modifier { modifier } => {
            if modifier.is_empty() {
                "Modifier · unset".into()
            } else {
                modifier.clone()
            }
        }
        Action::Combination { keys } => {
            if keys.is_empty() {
                "Combination".into()
            } else {
                keys.join(" + ")
            }
        }
        Action::LayerSwitch { layer_id } => format!("Switch · {layer_id}"),
        Action::LayerMomentary { layer_id } => format!("Hold · {layer_id}"),
        Action::Macro { macro_id } => format!("Macro · {macro_id}"),
        Action::Transparent => "Transparent".into(),
        Action::Unassigned => "Unassigned".into(),
    }
}

pub fn action_details(action: &Action) -> String {
    match action {
        Action::Key { code } => {
            format!("Opaque local key code: {code}. No TH80 firmware key catalog is asserted.")
        }
        Action::Modifier { modifier } => format!("Opaque local modifier value: {modifier}."),
        Action::Combination { keys } => {
            format!("{} key value(s): {}", keys.len(), keys.join(" + "))
        }
        Action::LayerSwitch { layer_id } => format!("Switches to logical layer ID {layer_id}."),
        Action::LayerMomentary { layer_id } => {
            format!("Activates logical layer ID {layer_id} while held.")
        }
        Action::Macro { macro_id } => format!("References local macro ID {macro_id}."),
        Action::Transparent => "Transparent assignment; no additional parameters.".into(),
        Action::Unassigned => "No action is assigned to this logical position.".into(),
    }
}
