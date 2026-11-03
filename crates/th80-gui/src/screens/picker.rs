use eframe::egui;
use th80_core::config::Action;

use crate::{app::Th80App, screens::keymap};

const CATEGORIES: [&str; 5] = ["Keys", "Modifiers", "Layers", "Macros", "Other"];
const KINDS: [&str; 8] = [
    "Key",
    "Modifier",
    "Combination",
    "Layer switch",
    "Momentary layer",
    "Macro",
    "Transparent",
    "Unassigned",
];

pub fn show(app: &mut Th80App, ctx: &egui::Context) {
    if !app.state.picker_open {
        return;
    }
    let layer_ids: Vec<(String, String)> = app
        .state
        .active
        .as_ref()
        .map(|profile| {
            profile
                .configuration
                .keymap
                .layers
                .iter()
                .map(|layer| (layer.id.clone(), layer.name.clone()))
                .collect()
        })
        .unwrap_or_default();
    let macro_ids: Vec<(String, String)> = app
        .state
        .active
        .as_ref()
        .map(|profile| {
            profile
                .configuration
                .macros
                .items
                .iter()
                .map(|item| (item.id.clone(), item.name.clone()))
                .collect()
        })
        .unwrap_or_default();
    let mut known_keys: Vec<String> = app
        .state
        .active
        .as_ref()
        .map(|profile| {
            profile
                .configuration
                .keymap
                .layers
                .iter()
                .flat_map(|layer| layer.positions.values())
                .filter_map(|assignment| match &assignment.action {
                    Action::Key { code } if !code.trim().is_empty() => Some(code.clone()),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default();
    known_keys.sort();
    known_keys.dedup();
    let mut known_modifiers: Vec<String> = app
        .state
        .active
        .as_ref()
        .map(|profile| {
            profile
                .configuration
                .keymap
                .layers
                .iter()
                .flat_map(|layer| layer.positions.values())
                .filter_map(|assignment| match &assignment.action {
                    Action::Modifier { modifier } if !modifier.trim().is_empty() => {
                        Some(modifier.clone())
                    }
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default();
    known_modifiers.sort();
    known_modifiers.dedup();

    let mut query = app.state.picker_query.clone();
    let mut category = app.state.picker_category.clone();
    let mut kind = app.state.picker_kind.clone();
    let mut draft = app
        .state
        .picker_action_draft
        .clone()
        .unwrap_or(Action::Unassigned);
    let mut apply = false;
    let mut cancel = false;
    let viewport = ctx.content_rect().size();
    let dialog_width = (viewport.x * 0.9).clamp(680.0, 1120.0);
    let dialog_height = (viewport.y * 0.82).clamp(360.0, 700.0);
    let response = egui::Modal::new(egui::Id::new("th80-action-picker")).show(ctx, |ui| {
        ui.set_min_width(dialog_width);
        ui.set_max_width(dialog_width);
        ui.set_min_height(dialog_height);
        ui.set_max_height(dialog_height);
        ui.horizontal(|ui| {
            ui.heading("Choose an action");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("×").on_hover_text("Cancel and close").clicked() { cancel = true; }
            });
        });
        ui.label(egui::RichText::new("Select a supported local profile action. No firmware key catalog is implied.").small().weak());
        ui.add_space(10.0);
        let search = ui.add(egui::TextEdit::singleline(&mut query).desired_width(f32::INFINITY).hint_text("Search actions and values…"));
        super::set_accessible_label(ui, &search, "Search actions and profile values");
        ui.add_space(8.0);

        let category_row_width = CATEGORIES.len() as f32 * 142.0
            + CATEGORIES.len().saturating_sub(1) as f32 * ui.spacing().item_spacing.x;
        let category_left_space = ((ui.available_width() - category_row_width) / 2.0).max(0.0);
        ui.horizontal(|ui| {
            ui.add_space(category_left_space);
            ui.horizontal_wrapped(|ui| {
            for item in CATEGORIES {
                if !query.trim().is_empty()
                    && !item.to_lowercase().contains(&query.to_lowercase())
                    && !KINDS.iter().any(|candidate| {
                        keymap::category_for_kind(candidate) == item
                            && candidate.to_lowercase().contains(&query.to_lowercase())
                    })
                {
                    continue;
                }
                let count = KINDS.iter().filter(|candidate| keymap::category_for_kind(candidate) == item).count();
                let response = ui.add_sized([142.0, 34.0], egui::Button::new(format!("{item}  ·  {count}")).selected(category == item));
                super::set_accessible_label(ui, &response, format!("{item} action category"));
                if response.clicked() {
                    category = item.to_owned();
                    if !KINDS.contains(&kind.as_str()) || keymap::category_for_kind(&kind) != item {
                        kind = KINDS.iter().find(|candidate| keymap::category_for_kind(candidate) == item).unwrap_or(&"Unassigned").to_string();
                        draft = create_action(&kind, &draft, &layer_ids, &macro_ids);
                    }
                }
            }
            });
        });
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("CURRENT").small().weak());
            ui.label(egui::RichText::new(keymap::action_legend(&draft)).strong());
        });
        ui.separator();
        egui::ScrollArea::vertical().max_height((dialog_height - 180.0).max(160.0)).show(ui, |ui| {
            ui.columns(2, |columns| {
                columns[0].vertical(|ui| {
                    ui.label(egui::RichText::new("ACTIONS").small().weak());
                    ui.add_space(6.0);
                    ui.horizontal_wrapped(|ui| {
                        for candidate in KINDS.iter().filter(|candidate| keymap::category_for_kind(candidate) == category) {
                            if !query.trim().is_empty() && !candidate.to_lowercase().contains(&query.to_lowercase()) { continue; }
                            let selected = kind == *candidate;
                            if ui.add_sized([132.0, 48.0], egui::Button::new(*candidate).selected(selected)).clicked() {
                                kind = (*candidate).to_string();
                                draft = create_action(&kind, &draft, &layer_ids, &macro_ids);
                            }
                        }
                    });
                    if category == "Keys" {
                        ui.add_space(12.0);
                        ui.label(egui::RichText::new("Values already used in this profile").small().weak());
                        ui.horizontal_wrapped(|ui| {
                            for value in &known_keys {
                                if value.to_lowercase().contains(&query.to_lowercase()) && ui.add_sized([104.0, 40.0], egui::Button::new(value).selected(matches!(&draft, Action::Key { code } if code == value))).clicked() {
                                    kind = "Key".into(); draft = Action::Key { code: value.clone() };
                                }
                            }
                        });
                    }
                    if category == "Modifiers" && !known_modifiers.is_empty() {
                        ui.add_space(12.0);
                        ui.label(egui::RichText::new("Modifiers already used in this profile").small().weak());
                        ui.horizontal_wrapped(|ui| {
                            for value in &known_modifiers {
                                if value.to_lowercase().contains(&query.to_lowercase()) && ui.add_sized([104.0, 40.0], egui::Button::new(value).selected(matches!(&draft, Action::Modifier { modifier } if modifier == value))).clicked() {
                                    kind = "Modifier".into(); draft = Action::Modifier { modifier: value.clone() };
                                }
                            }
                        });
                    }
                    if category == "Layers" {
                        ui.add_space(12.0);
                        ui.label(egui::RichText::new("Layers in this profile").small().weak());
                        ui.horizontal_wrapped(|ui| {
                            for (id, name) in &layer_ids {
                                let label = format!("{name} · {id}");
                                if label.to_lowercase().contains(&query.to_lowercase())
                                    && ui.add_sized([150.0, 40.0], egui::Button::new(&label).selected(matches!(&draft, Action::LayerSwitch { layer_id } if layer_id == id))).clicked()
                                {
                                    match draft {
                                        Action::LayerMomentary { .. } => {
                                            kind = "Momentary layer".into();
                                            draft = Action::LayerMomentary { layer_id: id.clone() };
                                        }
                                        _ => {
                                            kind = "Layer switch".into();
                                            draft = Action::LayerSwitch { layer_id: id.clone() };
                                        }
                                    }
                                }
                            }
                        });
                    }
                    if category == "Macros" {
                        ui.add_space(12.0);
                        ui.label(egui::RichText::new("Macros in this profile").small().weak());
                        ui.horizontal_wrapped(|ui| {
                            for (id, name) in &macro_ids {
                                let label = format!("{name} · {id}");
                                if label.to_lowercase().contains(&query.to_lowercase())
                                    && ui.add_sized([150.0, 40.0], egui::Button::new(&label).selected(matches!(&draft, Action::Macro { macro_id } if macro_id == id))).clicked()
                                {
                                    kind = "Macro".into();
                                    draft = Action::Macro { macro_id: id.clone() };
                                }
                            }
                        });
                    }
                });
                columns[1].vertical(|ui| {
                    ui.label(egui::RichText::new("ACTION DETAILS").small().weak());
                    ui.separator();
                    ui.label(egui::RichText::new(kind.as_str()).size(18.0).strong());
                    ui.label(keymap::action_details(&draft));
                    ui.add_space(10.0);
                    edit_draft(ui, &mut draft, &kind, &layer_ids, &macro_ids);
                    ui.add_space(12.0);
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Apply action").clicked() { apply = true; }
                        if ui.button("Cancel").clicked() { cancel = true; }
                    });
                    ui.label(egui::RichText::new("Enter applies when a detail control is not being edited · Esc cancels").small().weak());
                });
            });
        });
    });
    ctx.accesskit_node_builder(response.response.id, |node| {
        node.set_role(egui::accesskit::Role::Dialog);
        node.set_modal();
        node.set_label("Action picker");
        node.set_description("Choose a supported local profile action. Apply updates the active draft; Escape or Cancel preserves the original assignment.");
    });
    let closed = response.should_close() || cancel;
    app.state.picker_query = query;
    app.state.picker_category = category;
    app.state.picker_kind = kind;
    app.state.picker_action_draft = Some(draft.clone());
    if apply {
        if let (Some(layer_id), Some(position), Some(profile)) = (
            app.state.picker_target_layer.as_deref(),
            app.state.picker_target_position.as_deref(),
            app.state.active.as_mut(),
        ) {
            if let Some(layer) = profile
                .configuration
                .keymap
                .layers
                .iter_mut()
                .find(|layer| layer.id == layer_id)
            {
                if let Some(assignment) = layer.positions.get_mut(position) {
                    assignment.action = draft;
                }
            }
        }
        close_picker(app);
    } else if closed {
        close_picker(app);
    }
}

fn create_action(
    kind: &str,
    current: &Action,
    layers: &[(String, String)],
    macros: &[(String, String)],
) -> Action {
    match (kind, current) {
        ("Key", Action::Key { .. })
        | ("Modifier", Action::Modifier { .. })
        | ("Combination", Action::Combination { .. })
        | ("Layer switch", Action::LayerSwitch { .. })
        | ("Momentary layer", Action::LayerMomentary { .. })
        | ("Macro", Action::Macro { .. }) => current.clone(),
        ("Key", _) => Action::Key {
            code: String::new(),
        },
        ("Modifier", _) => Action::Modifier {
            modifier: String::new(),
        },
        ("Combination", _) => Action::Combination { keys: Vec::new() },
        ("Layer switch", _) => Action::LayerSwitch {
            layer_id: layers
                .first()
                .map(|value| value.0.clone())
                .unwrap_or_default(),
        },
        ("Momentary layer", _) => Action::LayerMomentary {
            layer_id: layers
                .first()
                .map(|value| value.0.clone())
                .unwrap_or_default(),
        },
        ("Macro", _) => Action::Macro {
            macro_id: macros
                .first()
                .map(|value| value.0.clone())
                .unwrap_or_default(),
        },
        ("Transparent", _) => Action::Transparent,
        _ => Action::Unassigned,
    }
}

fn edit_draft(
    ui: &mut egui::Ui,
    draft: &mut Action,
    kind: &str,
    layers: &[(String, String)],
    macros: &[(String, String)],
) {
    match draft {
        Action::Key { code } => {
            ui.label("Opaque local key code");
            let response = ui.text_edit_singleline(code);
            super::set_accessible_label(ui, &response, "Key action code");
        }
        Action::Modifier { modifier } => {
            ui.label("Opaque modifier value");
            let response = ui.text_edit_singleline(modifier);
            super::set_accessible_label(ui, &response, "Modifier action value");
        }
        Action::Combination { keys } => {
            let mut value = keys.join(", ");
            ui.label("Key values, separated by commas");
            let response = ui.text_edit_singleline(&mut value);
            super::set_accessible_label(ui, &response, "Combination key values");
            if response.changed() {
                *keys = value
                    .split(',')
                    .map(str::trim)
                    .filter(|part| !part.is_empty())
                    .map(str::to_owned)
                    .collect();
            }
        }
        Action::LayerSwitch { layer_id } | Action::LayerMomentary { layer_id } => {
            ui.label("Target logical layer");
            let selected = layers
                .iter()
                .find(|value| &value.0 == layer_id)
                .map(|value| value.1.clone())
                .unwrap_or_else(|| layer_id.clone());
            egui::ComboBox::from_id_salt(("picker-layer", kind))
                .selected_text(selected)
                .show_ui(ui, |ui| {
                    for (id, name) in layers {
                        ui.selectable_value(layer_id, id.clone(), format!("{name} · {id}"));
                    }
                });
            if layers.is_empty() {
                ui.label("Create a layer before assigning this action.");
            }
        }
        Action::Macro { macro_id } => {
            ui.label("Local macro");
            let selected = macros
                .iter()
                .find(|value| &value.0 == macro_id)
                .map(|value| value.1.clone())
                .unwrap_or_else(|| macro_id.clone());
            egui::ComboBox::from_id_salt("picker-macro")
                .selected_text(selected)
                .show_ui(ui, |ui| {
                    for (id, name) in macros {
                        ui.selectable_value(macro_id, id.clone(), format!("{name} · {id}"));
                    }
                });
            if macros.is_empty() {
                ui.label("Create a macro under Advanced before assigning one.");
            }
        }
        Action::Transparent | Action::Unassigned => {
            ui.label("This action has no additional parameters.");
        }
    };
}

fn close_picker(app: &mut Th80App) {
    app.state.picker_open = false;
    app.state.picker_target_layer = None;
    app.state.picker_target_position = None;
    app.state.picker_action_draft = None;
    app.state.picker_query.clear();
}
