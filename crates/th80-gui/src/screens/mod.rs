use eframe::egui;

pub mod dashboard;
pub mod hardware;
pub mod keymap;
pub mod layers;
pub mod lighting;
pub mod macros;
pub mod options;
pub mod picker;
pub mod profile;
pub mod settings;

/// Add an explicit accessible name to a widget while keeping its visible label
/// and layout unchanged. egui does not automatically associate adjacent labels
/// with text-edit widgets.
pub fn set_accessible_label(ui: &egui::Ui, response: &egui::Response, label: impl Into<String>) {
    let label = label.into();
    ui.ctx()
        .accesskit_node_builder(response.id, |node| node.set_label(label));
}

/// Expose additional context for controls whose visible text may be clipped.
pub fn set_accessible_description(
    ui: &egui::Ui,
    response: &egui::Response,
    description: impl Into<String>,
) {
    let description = description.into();
    ui.ctx()
        .accesskit_node_builder(response.id, |node| node.set_description(description));
}
