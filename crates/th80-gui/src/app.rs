use eframe::egui;
use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver, TryRecvError},
    time::Duration,
};
use th80_core::CoreError;

use crate::{
    profile_service::ProfileService,
    screens,
    state::{AppState, Page},
};

pub struct Th80App {
    pub state: AppState,
    pub(crate) file_dialog_pending: bool,
    file_dialog_result_rx: Option<Receiver<FileDialogResult>>,
    modal_previous_focus: Option<egui::Id>,
    modal_was_open: bool,
}

enum FileDialogResult {
    Import(Option<PathBuf>),
    Export {
        profile_id: th80_core::config::ProfileId,
        path: Option<PathBuf>,
    },
}

impl Th80App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self {
            state: AppState::new(ProfileService::new(profile_root())),
            file_dialog_pending: false,
            file_dialog_result_rx: None,
            modal_previous_focus: None,
            modal_was_open: false,
        };
        app.load_preferences();
        app
    }

    fn preferences_path() -> PathBuf {
        if let Some(path) = std::env::var_os("TH80_SETTINGS_PATH") {
            return path.into();
        }
        #[cfg(target_os = "windows")]
        if let Some(path) = std::env::var_os("APPDATA") {
            return PathBuf::from(path).join("TH80 Studio/settings.json");
        }
        #[cfg(target_os = "macos")]
        if let Some(path) = std::env::var_os("HOME") {
            return PathBuf::from(path)
                .join("Library/Application Support/TH80 Studio/settings.json");
        }
        if let Some(path) = std::env::var_os("XDG_CONFIG_HOME") {
            return PathBuf::from(path).join("th80-studio/settings.json");
        }
        if let Some(path) = std::env::var_os("HOME") {
            return PathBuf::from(path).join(".config/th80-studio/settings.json");
        }
        PathBuf::from("th80-studio-settings.json")
    }

    fn load_preferences(&mut self) {
        let path = Self::preferences_path();
        self.load_preferences_from(&path);
    }

    fn load_preferences_from(&mut self, path: &std::path::Path) {
        let Ok(contents) = std::fs::read_to_string(path) else {
            return;
        };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&contents) else {
            return;
        };
        if let Some(dark_mode) = value.get("dark_mode").and_then(serde_json::Value::as_bool) {
            self.state.dark_mode = dark_mode;
        }
        if let Some(scale) = value.get("ui_scale").and_then(serde_json::Value::as_f64) {
            if (0.8..=1.4).contains(&scale) {
                self.state.ui_scale = scale as f32;
            }
        }
        if let Some(color) = value
            .get("accent_color")
            .and_then(serde_json::Value::as_array)
        {
            if color.len() == 3 {
                let mut parsed = [0_u8; 3];
                let valid = color.iter().zip(parsed.iter_mut()).all(|(value, target)| {
                    value.as_u64().is_some_and(|channel| {
                        *target = channel.min(255) as u8;
                        true
                    })
                });
                if valid {
                    self.state.accent_color = parsed;
                }
            }
        }
        if let Some(directory) = value
            .get("file_dialog_start_dir")
            .and_then(serde_json::Value::as_str)
        {
            self.state.file_dialog_start_dir = directory.to_owned();
        }
    }

    pub(crate) fn save_preferences(&self) -> Result<(), String> {
        let path = Self::preferences_path();
        self.save_preferences_to(&path)
    }

    fn save_preferences_to(&self, path: &std::path::Path) -> Result<(), String> {
        let parent = path
            .parent()
            .ok_or_else(|| "Settings path has no parent directory".to_owned())?;
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        let value = serde_json::json!({
            "dark_mode": self.state.dark_mode,
            "ui_scale": self.state.ui_scale,
            "accent_color": self.state.accent_color,
            "file_dialog_start_dir": self.state.file_dialog_start_dir,
        });
        let temporary = path.with_extension("json.tmp");
        std::fs::write(
            &temporary,
            serde_json::to_vec_pretty(&value).map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
        std::fs::rename(&temporary, path).map_err(|error| error.to_string())
    }

    pub(crate) fn apply_visual_theme(ctx: &egui::Context, dark_mode: bool, accent_rgb: [u8; 3]) {
        let mut visuals = if dark_mode {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        let accent = egui::Color32::from_rgb(accent_rgb[0], accent_rgb[1], accent_rgb[2]);
        let hover = if dark_mode {
            egui::Color32::from_rgb(57, 52, 55)
        } else {
            egui::Color32::from_rgb(245, 235, 238)
        };
        let surface = if dark_mode {
            egui::Color32::from_rgb(39, 39, 41)
        } else {
            egui::Color32::WHITE
        };
        let panel = if dark_mode {
            egui::Color32::from_rgb(27, 27, 29)
        } else {
            egui::Color32::from_rgb(246, 246, 247)
        };
        let border = if dark_mode {
            egui::Color32::from_rgb(65, 65, 68)
        } else {
            egui::Color32::from_rgb(220, 220, 223)
        };
        visuals.panel_fill = panel;
        visuals.window_fill = panel;
        visuals.extreme_bg_color = if dark_mode {
            egui::Color32::from_rgb(19, 19, 21)
        } else {
            egui::Color32::WHITE
        };
        visuals.faint_bg_color = if dark_mode {
            egui::Color32::from_rgb(33, 33, 35)
        } else {
            egui::Color32::from_rgb(239, 239, 241)
        };
        visuals.selection.bg_fill = accent;
        visuals.widgets.noninteractive.bg_fill = panel;
        visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, border);
        visuals.widgets.inactive.bg_fill = surface;
        visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, border);
        visuals.widgets.hovered.bg_fill = hover;
        visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, accent);
        visuals.widgets.active.bg_fill = accent;
        visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, accent);
        visuals.hyperlink_color = accent;
        ctx.set_visuals(visuals);
        let theme = if dark_mode {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        };
        ctx.style_mut_of(theme, |style| {
            style.spacing.item_spacing = egui::vec2(8.0, 8.0);
            style.spacing.button_padding = egui::vec2(12.0, 7.0);
        });
    }

    pub(crate) fn import_dialog(&mut self) {
        if self.file_dialog_pending {
            return;
        }
        let dialog = self
            .file_dialog_builder()
            .add_filter("TH80 profile JSON", &["json"])
            .pick_file();
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let path = pollster::block_on(dialog).map(|file| file.path().to_path_buf());
            let _ = sender.send(FileDialogResult::Import(path));
        });
        self.file_dialog_result_rx = Some(receiver);
        self.file_dialog_pending = true;
    }

    fn export_dialog(&mut self) {
        if self.file_dialog_pending {
            return;
        }
        let Some(profile) = self.state.active.as_ref() else {
            return;
        };
        let id = profile.id.clone();
        let filename = format!("{}.json", id.0);
        let dialog = self
            .file_dialog_builder()
            .set_file_name(filename)
            .add_filter("TH80 profile JSON", &["json"])
            .save_file();
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let path = pollster::block_on(dialog).map(|file| file.path().to_path_buf());
            let _ = sender.send(FileDialogResult::Export {
                profile_id: id,
                path,
            });
        });
        self.file_dialog_result_rx = Some(receiver);
        self.file_dialog_pending = true;
    }

    fn file_dialog_builder(&self) -> rfd::AsyncFileDialog {
        let configured = (!self.state.file_dialog_start_dir.trim().is_empty())
            .then(|| PathBuf::from(self.state.file_dialog_start_dir.trim()));
        if let Some(directory) = configured
            .or_else(|| std::env::var_os("TH80_FILE_DIALOG_START_DIR").map(PathBuf::from))
            .filter(|path| path.is_dir())
        {
            rfd::AsyncFileDialog::new().set_directory(directory)
        } else {
            rfd::AsyncFileDialog::new()
        }
    }

    fn poll_file_dialog(&mut self, ctx: &egui::Context) {
        let result = self.file_dialog_result_rx.as_ref().map(Receiver::try_recv);
        match result {
            Some(Ok(result)) => {
                self.file_dialog_result_rx = None;
                self.file_dialog_pending = false;
                self.apply_file_dialog_result(result);
            }
            Some(Err(TryRecvError::Empty)) => {
                ctx.request_repaint_after(Duration::from_millis(100));
            }
            Some(Err(TryRecvError::Disconnected)) => {
                self.file_dialog_result_rx = None;
                self.file_dialog_pending = false;
                self.state.message = Some((
                    false,
                    "The file dialog closed without returning a result".into(),
                ));
            }
            None => {}
        }
    }

    fn apply_file_dialog_result(&mut self, result: FileDialogResult) {
        match result {
            FileDialogResult::Import(Some(path)) => self.state.import_file(path),
            FileDialogResult::Import(None) => {}
            FileDialogResult::Export {
                profile_id,
                path: Some(path),
            } => match self.state.service.export(&profile_id, path) {
                Ok(()) => self.state.message = Some((true, "Profile exported".into())),
                Err(error) => {
                    self.state.message = Some((false, crate::state::format_error(&error)))
                }
            },
            FileDialogResult::Export { path: None, .. } => {}
        }
    }

    fn validate_active(&mut self) {
        let Some(profile) = self.state.active.as_ref() else {
            return;
        };
        match self.state.service.validate(profile) {
            Ok(()) => {
                self.state.message = Some((
                    true,
                    "Profile passes local validation. Firmware compatibility is not verified."
                        .into(),
                ))
            }
            Err(CoreError::InvalidProfile(items)) => {
                let details = items
                    .iter()
                    .map(|item| {
                        format!(
                            "{} [{}] {}",
                            item.code,
                            item.path.as_deref().unwrap_or("profile"),
                            item.message
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                self.state.message = Some((
                    false,
                    format!("{} validation issue(s):\n{details}", items.len()),
                ));
            }
            Err(error) => self.state.message = Some((false, crate::state::format_error(&error))),
        }
    }

    fn navigation(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("PROFILE").small().weak());
            let profiles = self.state.profiles.clone();
            let mut selected = self
                .state
                .active
                .as_ref()
                .map(|profile| profile.id.0.clone())
                .unwrap_or_default();
            let selected_name = profiles
                .iter()
                .find(|profile| profile.id.0 == selected)
                .map(|profile| profile.name.clone())
                .unwrap_or_else(|| "Choose local profile".into());
            let response = egui::ComboBox::from_id_salt("local-profile-selector")
                .selected_text(compact_profile_selector_text(&selected_name))
                .width(150.0)
                .show_ui(ui, |ui| {
                    for profile in &profiles {
                        ui.selectable_value(&mut selected, profile.id.0.clone(), &profile.name);
                    }
                });
            screens::set_accessible_label(ui, &response.response, "Local profile selector");
            screens::set_accessible_description(
                ui,
                &response.response,
                format!("Selected local profile: {selected_name}"),
            );
            if let Some(profile) = profiles.iter().find(|profile| profile.id.0 == selected) {
                if self.state.active.as_ref().map(|active| &active.id) != Some(&profile.id) {
                    self.state.open(&profile.id);
                }
            }
            ui.separator();
            for (page, label) in [
                (Page::Keymap, "Layout"),
                (Page::Macros, "Advanced"),
                (Page::Settings, "Config"),
            ] {
                let active = match page {
                    Page::Keymap => matches!(self.state.page, Page::Keymap | Page::Layers),
                    Page::Macros => self.state.page == Page::Macros,
                    _ => matches!(
                        self.state.page,
                        Page::Lighting
                            | Page::Options
                            | Page::Dashboard
                            | Page::Profile
                            | Page::Settings
                            | Page::Hardware
                    ),
                };
                if ui
                    .add(
                        egui::Button::new(label)
                            .selected(active)
                            .min_size([82.0, 32.0].into()),
                    )
                    .clicked()
                {
                    self.state.page = page;
                }
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .small_button("+")
                    .on_hover_text("Increase interface scale")
                    .clicked()
                {
                    self.state.ui_scale = (self.state.ui_scale + 0.05).min(1.4);
                    ui.ctx().set_zoom_factor(self.state.ui_scale);
                    let _ = self.save_preferences();
                    ui.ctx().request_repaint();
                }
                if ui
                    .small_button(format!("{}%", (self.state.ui_scale * 100.0).round() as u32))
                    .on_hover_text("Reset interface scale")
                    .clicked()
                {
                    self.state.ui_scale = 1.0;
                    ui.ctx().set_zoom_factor(self.state.ui_scale);
                    let _ = self.save_preferences();
                    ui.ctx().request_repaint();
                }
                if ui
                    .small_button("−")
                    .on_hover_text("Decrease interface scale")
                    .clicked()
                {
                    self.state.ui_scale = (self.state.ui_scale - 0.05).max(0.8);
                    ui.ctx().set_zoom_factor(self.state.ui_scale);
                    let _ = self.save_preferences();
                    ui.ctx().request_repaint();
                }
                if ui
                    .add_enabled(
                        self.state.active.is_some() && self.state.dirty(),
                        egui::Button::new("Save"),
                    )
                    .clicked()
                {
                    self.state.save();
                }
                ui.menu_button("Files", |ui| {
                    if ui
                        .add_enabled(
                            !self.file_dialog_pending,
                            egui::Button::new("Import profile…"),
                        )
                        .clicked()
                    {
                        self.import_dialog();
                        ui.close();
                    }
                    if ui
                        .add_enabled(
                            !self.file_dialog_pending && self.state.active.is_some(),
                            egui::Button::new("Export profile…"),
                        )
                        .clicked()
                    {
                        self.export_dialog();
                        ui.close();
                    }
                    if ui
                        .add_enabled(
                            self.state.active.is_some(),
                            egui::Button::new("Save as new profile…"),
                        )
                        .clicked()
                    {
                        self.state.save_as();
                        ui.close();
                    }
                });
                if self.state.active.is_some() {
                    let status = if self.state.dirty() {
                        "● Unsaved"
                    } else {
                        "✓ Saved"
                    };
                    ui.label(
                        egui::RichText::new(status)
                            .small()
                            .color(if self.state.dirty() {
                                egui::Color32::from_rgb(205, 164, 85)
                            } else {
                                egui::Color32::from_rgb(115, 170, 145)
                            }),
                    );
                }
            });
        });
        ui.separator();
        ui.horizontal(|ui| {
            let entries: &[(Page, &str)] = match self.state.page {
                Page::Keymap | Page::Layers => {
                    &[(Page::Keymap, "Workspace"), (Page::Layers, "Layers")]
                }
                Page::Macros => &[(Page::Macros, "Macros")],
                Page::Hardware => &[(Page::Hardware, "Hardware test")],
                _ => &[
                    (Page::Lighting, "Lighting"),
                    (Page::Options, "Keyboard options"),
                    (Page::Dashboard, "Profile library"),
                    (Page::Profile, "Profile details"),
                    (Page::Settings, "App settings"),
                    (Page::Hardware, "Hardware test"),
                ],
            };
            for (page, label) in entries {
                let enabled = matches!(page, Page::Dashboard | Page::Settings | Page::Hardware)
                    || self.state.active.is_some();
                if ui
                    .add_enabled(
                        enabled,
                        egui::Button::new(*label)
                            .selected(self.state.page == *page)
                            .frame(false),
                    )
                    .clicked()
                {
                    self.state.page = *page;
                }
            }
            if self.state.page == Page::Keymap {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("Validate").clicked() {
                        self.validate_active();
                    }
                });
            }
        });
    }

    fn dialogs(&mut self, ctx: &egui::Context) {
        self.sync_modal_focus(ctx);
        if let Some(id) = self.state.pending_open.clone() {
            let response =
                egui::Modal::new(egui::Id::new("unsaved-profile-open-dialog")).show(ctx, |ui| {
                    ui.scope(|ui| {
                        ui.set_min_width(360.0);
                        ui.set_max_width(560.0);
                        ui.heading("Unsaved changes");
                        ui.label("This profile has unsaved changes. What would you like to do?");
                        ui.horizontal(|ui| {
                            if ui.button("Save and open").clicked() {
                                self.state.save();
                                if !self.state.dirty() {
                                    self.state.open_now(&id);
                                }
                            }
                            if ui.button("Discard and open").clicked() {
                                self.state.open_now(&id);
                            }
                            if ui.button("Cancel").clicked() {
                                self.state.pending_open = None;
                            }
                        });
                    })
                    .response
                });
            Self::mark_modal_dialog(ctx, &response.inner, "Unsaved changes");
            if response.should_close() {
                self.state.pending_open = None;
            }
        }
        if let Some(profile_id) = self.state.pending_delete_profile.clone() {
            let response = egui::Modal::new(egui::Id::new("delete-profile-dialog"))
                .show(ctx, |ui| {
                    ui.scope(|ui| {
                        ui.set_min_width(360.0);
                        ui.set_max_width(560.0);
                        ui.heading("Delete profile?");
                        let deleting_active = self
                            .state
                            .active
                            .as_ref()
                            .is_some_and(|profile| profile.id == profile_id);
                        ui.label(if deleting_active && self.state.dirty() {
                            "This permanently deletes the local profile file and discards its unsaved in-memory edits. This cannot be undone."
                        } else {
                            "This permanently deletes the local profile file. This cannot be undone."
                        });
                        ui.horizontal(|ui| {
                            if ui.button("Delete profile").clicked() {
                                self.state.delete_profile(&profile_id);
                            }
                            if ui.button("Cancel").clicked() {
                                self.state.pending_delete_profile = None;
                            }
                        });
                    })
                    .response
                });
            Self::mark_modal_dialog(ctx, &response.inner, "Delete profile?");
            if response.should_close() {
                self.state.pending_delete_profile = None;
            }
        }
        if let Some(macro_id) = self.state.pending_delete_macro.clone() {
            let macro_name = self
                .state
                .active
                .as_ref()
                .and_then(|profile| {
                    profile
                        .configuration
                        .macros
                        .items
                        .iter()
                        .find(|item| item.id == macro_id)
                })
                .map(|item| item.name.as_str())
                .unwrap_or("the selected macro")
                .to_owned();
            let macro_is_referenced = self.state.active.as_ref().is_some_and(|profile| {
                profile.configuration.keymap.layers.iter().any(|layer| {
                    layer.positions.values().any(|assignment| {
                        matches!(
                            &assignment.action,
                            th80_core::config::Action::Macro { macro_id: assigned_id }
                                if assigned_id == &macro_id
                        )
                    })
                })
            });
            let response = egui::Modal::new(egui::Id::new("delete-macro-dialog"))
                .show(ctx, |ui| {
                    ui.scope(|ui| {
                        ui.set_min_width(360.0);
                        ui.set_max_width(560.0);
                        ui.heading("Delete macro?");
                        ui.label(format!(
                            "Remove '{macro_name}' from this local profile draft? The change is not saved until you save the profile."
                        ));
                        if macro_is_referenced {
                            ui.colored_label(
                                egui::Color32::YELLOW,
                                "A key assignment references this macro. Reassign or clear those keys before the profile can pass validation.",
                            );
                        }
                        ui.horizontal(|ui| {
                            if ui.button("Delete macro").clicked() {
                                self.state.delete_macro(&macro_id);
                            }
                            if ui.button("Cancel").clicked() {
                                self.state.pending_delete_macro = None;
                            }
                        });
                    })
                    .response
                });
            Self::mark_modal_dialog(ctx, &response.inner, "Delete macro?");
            if response.should_close() {
                self.state.pending_delete_macro = None;
            }
        }
    }

    fn sync_modal_focus(&mut self, ctx: &egui::Context) {
        let modal_open = self.state.pending_open.is_some()
            || self.state.pending_delete_profile.is_some()
            || self.state.pending_delete_macro.is_some()
            || self.state.message.is_some()
            || self.state.picker_open;

        if modal_open && !self.modal_was_open {
            self.modal_previous_focus = ctx.memory(|memory| memory.focused());
        } else if !modal_open && self.modal_was_open {
            if let Some(previous_focus) = self.modal_previous_focus.take() {
                ctx.memory_mut(|memory| memory.request_focus(previous_focus));
            }
        }

        self.modal_was_open = modal_open;
    }

    fn mark_modal_dialog(ctx: &egui::Context, response: &egui::Response, label: &str) {
        ctx.accesskit_node_builder(response.id, |node| {
            node.set_role(egui::accesskit::Role::Dialog);
            node.set_modal();
            node.set_label(label.to_owned());
        });
    }
}

fn compact_profile_selector_text(name: &str) -> String {
    const MAX_VISIBLE_CHARS: usize = 22;
    let mut chars = name.chars();
    let prefix = chars.by_ref().take(MAX_VISIBLE_CHARS).collect::<String>();
    if chars.next().is_some() {
        format!("{prefix}…")
    } else {
        prefix
    }
}

impl eframe::App for Th80App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.poll_file_dialog(&ctx);
        ctx.accesskit_node_builder(egui::accesskit_root_id(), |node| {
            node.set_label("TH80 Studio");
            node.set_description("Offline local profile editor. No physical keyboard connection or firmware writes are performed.");
        });
        Self::apply_visual_theme(&ctx, self.state.dark_mode, self.state.accent_color);
        ctx.set_zoom_factor(self.state.ui_scale);
        egui::Panel::top("topbar").show(ui, |ui| {
            self.navigation(ui);
        });

        let mut preferences_changed = false;
        egui::Panel::bottom("statusbar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("TH80 STUDIO").small().weak());
                ui.separator();
                let local_save_state = if self.state.dirty() {
                    "● Unsaved changes"
                } else {
                    "✓ Saved locally"
                };
                ui.label(egui::RichText::new(local_save_state).small().weak());
                ui.separator();
                if self.state.picker_open {
                    ui.label(
                        egui::RichText::new("Choose an action · Escape cancels")
                            .small()
                            .weak(),
                    );
                } else if self.file_dialog_pending {
                    ui.label(
                        egui::RichText::new("Local file chooser open…")
                            .small()
                            .weak(),
                    );
                } else if self.state.page == Page::Keymap {
                    ui.label(
                        egui::RichText::new(
                            "Select a logical position to inspect and edit its assignment",
                        )
                        .small()
                        .weak(),
                    );
                } else {
                    ui.label(
                        egui::RichText::new("Changes remain in the local draft until saved")
                            .small()
                            .weak(),
                    );
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .small_button(if self.state.dark_mode {
                            "☼ Light"
                        } else {
                            "☾ Dark"
                        })
                        .clicked()
                    {
                        self.state.dark_mode = !self.state.dark_mode;
                        Self::apply_visual_theme(
                            ui.ctx(),
                            self.state.dark_mode,
                            self.state.accent_color,
                        );
                        preferences_changed = true;
                    }
                    if let Some(profile) = &self.state.active {
                        ui.label(egui::RichText::new(profile.name.clone()).small().weak());
                    }
                });
            });
        });
        if preferences_changed {
            ctx.request_repaint();
            if let Err(error) = self.save_preferences() {
                self.state.message = Some((
                    false,
                    format!("Could not save application preferences: {error}"),
                ));
            }
        }

        egui::CentralPanel::default().show(ui, |ui| {
            ui.add_space(8.0);
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| match self.state.page {
                    Page::Dashboard => screens::dashboard::show(self, ui),
                    Page::Keymap => screens::keymap::show(self, ui),
                    Page::Layers => screens::layers::show(self, ui),
                    Page::Macros => screens::macros::show(self, ui),
                    Page::Lighting => screens::lighting::show(self, ui),
                    Page::Options => screens::options::show(self, ui),
                    Page::Profile => screens::profile::show(self, ui),
                    Page::Settings => screens::settings::show(self, ui),
                    Page::Hardware => screens::hardware::show(self, ui),
                });
        });

        self.sync_modal_focus(&ctx);
        if let Some((success, message)) = self.state.message.clone() {
            let response = egui::Modal::new(egui::Id::new("profile-status-message-dialog"))
                .show(&ctx, |ui| {
                    ui.scope(|ui| {
                        ui.set_min_width(360.0);
                        ui.set_max_width(560.0);
                        ui.heading(if success { "Information" } else { "Action failed" });
                        ui.label(message);
                        if self.state.save_conflict {
                            ui.separator();
                            ui.label("The saved profile changed elsewhere. Reloading discards this in-memory draft; Save as can preserve it under a new profile ID.");
                            ui.horizontal(|ui| {
                                if ui.button("Reload saved version").clicked() {
                                    self.state.message = None;
                                    self.state.reload_active();
                                }
                                if ui.button("Save draft as new profile").clicked() {
                                    self.state.message = None;
                                    self.state.save_as();
                                }
                            });
                        }
                        if ui.button("Dismiss").clicked() {
                            self.state.message = None;
                            self.state.save_conflict = false;
                        }
                    })
                    .response
                });
            Self::mark_modal_dialog(
                &ctx,
                &response.inner,
                if success {
                    "Information"
                } else {
                    "Action failed"
                },
            );
            if response.should_close() {
                self.state.message = None;
                self.state.save_conflict = false;
            }
        }
        self.dialogs(&ctx);
        screens::picker::show(self, &ctx);
    }
}

fn profile_root() -> std::path::PathBuf {
    if let Some(path) = std::env::var_os("TH80_PROFILE_DIR") {
        return path.into();
    }
    #[cfg(target_os = "windows")]
    if let Some(path) = std::env::var_os("APPDATA") {
        return std::path::PathBuf::from(path).join("th80-profiles");
    }
    #[cfg(target_os = "macos")]
    if let Some(path) = std::env::var_os("HOME") {
        return std::path::PathBuf::from(path).join("Library/Application Support/th80-profiles");
    }
    if let Some(path) = std::env::var_os("XDG_DATA_HOME") {
        return std::path::PathBuf::from(path).join("th80-profiles");
    }
    if let Some(path) = std::env::var_os("HOME") {
        return std::path::PathBuf::from(path).join(".local/share/th80-profiles");
    }
    std::path::PathBuf::from("th80-profiles")
}

#[cfg(test)]
mod ui_tests {
    use super::{mpsc, FileDialogResult, Th80App};
    use crate::{profile_service::ProfileService, screens, state::AppState};
    use eframe::egui::{self, Event, Key, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};
    use std::collections::BTreeMap;
    use th80_core::{
        config::{Action, KeyAssignment, Layer, MacroDefinition, MacroStep},
        Profile,
    };

    type ScreenRenderer = fn(&egui::Context, &mut Th80App, Vec<Event>) -> egui::FullOutput;

    fn test_app(root: &std::path::Path) -> Th80App {
        Th80App {
            state: AppState::new(ProfileService::new(root)),
            file_dialog_pending: false,
            file_dialog_result_rx: None,
            modal_previous_focus: None,
            modal_was_open: false,
        }
    }

    #[test]
    fn application_theme_accent_scale_and_file_dialog_preferences_persist() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        let mut source = test_app(dir.path());
        source.state.dark_mode = false;
        source.state.ui_scale = 1.25;
        source.state.accent_color = [12, 123, 234];
        source.state.file_dialog_start_dir = "/tmp/th80-picker".into();
        source.save_preferences_to(&path).unwrap();

        let mut restored = test_app(dir.path());
        restored.state.dark_mode = true;
        restored.load_preferences_from(&path);

        assert!(!restored.state.dark_mode);
        assert_eq!(restored.state.ui_scale, 1.25);
        assert_eq!(restored.state.accent_color, [12, 123, 234]);
        assert_eq!(restored.state.file_dialog_start_dir, "/tmp/th80-picker");
    }

    fn raw_input(events: Vec<Event>) -> RawInput {
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 820.0))),
            events,
            ..Default::default()
        }
    }

    fn key_event(key: Key, pressed: bool, modifiers: Modifiers) -> Event {
        Event::Key {
            key,
            physical_key: Some(key),
            pressed,
            repeat: false,
            modifiers,
        }
    }

    fn finish_output(mut output: egui::FullOutput) -> egui::FullOutput {
        // Unit tests inspect the accessibility tree, not renderer texture uploads.
        // Newer epaint versions assert that unapplied texture deltas are handled.
        output.textures_delta.clear();
        output
    }

    fn render_macros(
        ctx: &egui::Context,
        app: &mut Th80App,
        events: Vec<Event>,
    ) -> egui::FullOutput {
        let dialog_ctx = ctx.clone();
        finish_output(ctx.run_ui(raw_input(events), |ui| {
            screens::macros::show(app, ui);
            app.dialogs(&dialog_ctx);
        }))
    }

    fn render_lighting(
        ctx: &egui::Context,
        app: &mut Th80App,
        events: Vec<Event>,
    ) -> egui::FullOutput {
        finish_output(ctx.run_ui(raw_input(events), |ui| {
            screens::lighting::show(app, ui);
        }))
    }

    fn render_profile(
        ctx: &egui::Context,
        app: &mut Th80App,
        events: Vec<Event>,
    ) -> egui::FullOutput {
        finish_output(ctx.run_ui(raw_input(events), |ui| screens::profile::show(app, ui)))
    }

    fn render_dashboard(
        ctx: &egui::Context,
        app: &mut Th80App,
        events: Vec<Event>,
    ) -> egui::FullOutput {
        finish_output(ctx.run_ui(raw_input(events), |ui| screens::dashboard::show(app, ui)))
    }

    fn render_keymap(
        ctx: &egui::Context,
        app: &mut Th80App,
        events: Vec<Event>,
    ) -> egui::FullOutput {
        let dialog_ctx = ctx.clone();
        finish_output(ctx.run_ui(raw_input(events), |ui| {
            screens::keymap::show(app, ui);
            screens::picker::show(app, &dialog_ctx);
        }))
    }

    fn render_layers(
        ctx: &egui::Context,
        app: &mut Th80App,
        events: Vec<Event>,
    ) -> egui::FullOutput {
        finish_output(ctx.run_ui(raw_input(events), |ui| screens::layers::show(app, ui)))
    }

    fn render_options(
        ctx: &egui::Context,
        app: &mut Th80App,
        events: Vec<Event>,
    ) -> egui::FullOutput {
        finish_output(ctx.run_ui(raw_input(events), |ui| screens::options::show(app, ui)))
    }

    fn render_navigation(
        ctx: &egui::Context,
        app: &mut Th80App,
        events: Vec<Event>,
    ) -> egui::FullOutput {
        finish_output(ctx.run_ui(raw_input(events), |ui| {
            app.navigation(ui);
        }))
    }

    fn render_navigation_with_macro_dialog(
        ctx: &egui::Context,
        app: &mut Th80App,
        events: Vec<Event>,
    ) -> egui::FullOutput {
        let dialog_ctx = ctx.clone();
        finish_output(ctx.run_ui(raw_input(events), |ui| {
            app.navigation(ui);
            screens::macros::show(app, ui);
            app.dialogs(&dialog_ctx);
        }))
    }

    fn widget_position(output: &egui::FullOutput, label_fragment: &str) -> Pos2 {
        let update = output
            .platform_output
            .accesskit_update
            .as_ref()
            .expect("AccessKit update should be generated when enabled");
        let (_, node) = update
            .nodes
            .iter()
            .find(|(_, node)| {
                node.label()
                    .is_some_and(|label| label.contains(label_fragment))
            })
            .unwrap_or_else(|| {
                panic!(
                    "no accessible node contains {label_fragment:?}; nodes: {:?}",
                    update
                        .nodes
                        .iter()
                        .map(|(_, node)| (node.role(), node.label(), node.value()))
                        .collect::<Vec<_>>()
                )
            });
        let bounds = node
            .bounds()
            .expect("interactive widget should have bounds");
        Pos2::new(
            ((bounds.x0 + bounds.x1) / 2.0) as f32,
            ((bounds.y0 + bounds.y1) / 2.0) as f32,
        )
    }

    fn widget_node_id(output: &egui::FullOutput, label_fragment: &str) -> egui::accesskit::NodeId {
        output
            .platform_output
            .accesskit_update
            .as_ref()
            .expect("AccessKit update should be generated when enabled")
            .nodes
            .iter()
            .find(|(_, node)| {
                node.label()
                    .is_some_and(|label| label.contains(label_fragment))
            })
            .map(|(id, _)| *id)
            .unwrap_or_else(|| panic!("missing accessible widget {label_fragment:?}"))
    }

    fn dialog_widget_node_id(
        output: &egui::FullOutput,
        label_fragment: &str,
    ) -> egui::accesskit::NodeId {
        let update = output
            .platform_output
            .accesskit_update
            .as_ref()
            .expect("AccessKit update should be generated when enabled");
        let dialog_id = update
            .nodes
            .iter()
            .find(|(_, node)| node.role() == egui::accesskit::Role::Dialog)
            .map(|(id, _)| *id)
            .expect("modal dialog node should exist");
        let mut descendants = Vec::new();
        collect_accessible_descendants(&update.nodes, dialog_id, &mut descendants);
        update
            .nodes
            .iter()
            .find(|(id, node)| {
                descendants.contains(id)
                    && node
                        .label()
                        .is_some_and(|label| label.contains(label_fragment))
            })
            .map(|(id, _)| *id)
            .unwrap_or_else(|| panic!("missing dialog widget {label_fragment:?}"))
    }

    fn collect_accessible_descendants(
        nodes: &[(egui::accesskit::NodeId, egui::accesskit::Node)],
        parent_id: egui::accesskit::NodeId,
        descendants: &mut Vec<egui::accesskit::NodeId>,
    ) {
        if let Some((_, parent)) = nodes.iter().find(|(id, _)| *id == parent_id) {
            for child_id in parent.children() {
                descendants.push(*child_id);
                collect_accessible_descendants(nodes, *child_id, descendants);
            }
        }
    }

    fn activate_accessibility_node(
        ctx: &egui::Context,
        app: &mut Th80App,
        node_id: egui::accesskit::NodeId,
    ) {
        render_macros(
            ctx,
            app,
            vec![Event::AccessKitActionRequest(
                egui::accesskit::ActionRequest {
                    action: egui::accesskit::Action::Click,
                    target_tree: egui::accesskit::TreeId::ROOT,
                    target_node: node_id,
                    data: None,
                },
            )],
        );
    }

    fn activate_keymap_accessibility_node(
        ctx: &egui::Context,
        app: &mut Th80App,
        node_id: egui::accesskit::NodeId,
    ) {
        render_keymap(
            ctx,
            app,
            vec![Event::AccessKitActionRequest(
                egui::accesskit::ActionRequest {
                    action: egui::accesskit::Action::Click,
                    target_tree: egui::accesskit::TreeId::ROOT,
                    target_node: node_id,
                    data: None,
                },
            )],
        );
    }

    fn click(
        ctx: &egui::Context,
        app: &mut Th80App,
        pos: Pos2,
        render: fn(&egui::Context, &mut Th80App, Vec<Event>) -> egui::FullOutput,
    ) {
        render(
            ctx,
            app,
            vec![
                Event::PointerMoved(pos),
                Event::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed: true,
                    modifiers: Modifiers::NONE,
                },
                Event::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed: false,
                    modifiers: Modifiers::NONE,
                },
            ],
        );
    }

    fn profile_with_two_macros() -> Profile {
        let mut profile = Profile::new("Synthetic UI test");
        profile.configuration.macros.items = vec![
            MacroDefinition {
                id: "macro-a".into(),
                name: "Macro A".into(),
                steps: vec![MacroStep::KeyDown {
                    key: "KEY_A".into(),
                }],
            },
            MacroDefinition {
                id: "macro-b".into(),
                name: "Macro B".into(),
                steps: Vec::new(),
            },
        ];
        profile.configuration.keymap.layers.push(Layer {
            id: "base".into(),
            name: "Base".into(),
            positions: BTreeMap::from([(
                "logical-01".into(),
                KeyAssignment {
                    action: Action::Macro {
                        macro_id: "macro-a".into(),
                    },
                },
            )]),
        });
        profile
    }

    #[test]
    fn macro_confirmation_deletes_original_target_after_selection_changes() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = test_app(dir.path());
        let profile = profile_with_two_macros();
        app.state.active = Some(profile.clone());
        app.state.baseline = Some(profile);
        app.state.selected_macro = Some("macro-a".into());
        app.state.pending_delete_macro = Some("macro-a".into());

        // The underlying editor selection changes while the confirmation is open.
        app.state.selected_macro = Some("macro-b".into());
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let output = render_macros(&ctx, &mut app, Vec::new());
        let update = output.platform_output.accesskit_update.as_ref().unwrap();
        assert!(update.nodes.iter().any(|(_, node)| {
            node.role() == egui::accesskit::Role::Dialog
                && node.is_modal()
                && node.label() == Some("Delete macro?")
                && !node.children().is_empty()
        }));
        assert!(update.nodes.iter().any(|(_, node)| {
            node.value()
                .is_some_and(|value| value.contains("A key assignment references this macro"))
        }));
        assert!(update.nodes.iter().any(|(_, node)| {
            node.value()
                .is_some_and(|value| value.contains("Remove 'Macro A'"))
        }));
        let delete = dialog_widget_node_id(&output, "Delete macro");
        activate_accessibility_node(&ctx, &mut app, delete);

        let active = app.state.active.as_ref().unwrap();
        assert_eq!(
            active
                .configuration
                .macros
                .items
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
            vec!["macro-b"]
        );
        assert_eq!(app.state.selected_macro.as_deref(), Some("macro-b"));
        assert!(app.state.pending_delete_macro.is_none());
        assert!(app.state.dirty());
        assert!(!th80_core::validation::validate_profile(active).is_empty());
    }

    #[test]
    fn macro_confirmation_cancel_preserves_both_macros_and_reference() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = test_app(dir.path());
        let profile = profile_with_two_macros();
        app.state.active = Some(profile.clone());
        app.state.baseline = Some(profile);
        app.state.selected_macro = Some("macro-b".into());
        app.state.pending_delete_macro = Some("macro-a".into());
        let ctx = egui::Context::default();
        ctx.enable_accesskit();

        let output = render_macros(&ctx, &mut app, Vec::new());
        assert!(output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| { node.role() == egui::accesskit::Role::Dialog && node.is_modal() }));
        let cancel = widget_node_id(&output, "Cancel");
        activate_accessibility_node(&ctx, &mut app, cancel);

        let active = app.state.active.as_ref().unwrap();
        assert_eq!(active.configuration.macros.items.len(), 2);
        assert_eq!(app.state.selected_macro.as_deref(), Some("macro-b"));
        assert!(app.state.pending_delete_macro.is_none());
        assert!(!app.state.dirty());
    }

    #[test]
    fn simulated_egui_pointer_creates_macro_and_appends_ordered_steps() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = test_app(dir.path());
        let profile = Profile::new("Macro UI");
        app.state.active = Some(profile.clone());
        app.state.baseline = Some(profile);
        app.state.new_macro_id = "macro-ui".into();
        app.state.new_macro_name = "UI macro".into();
        let ctx = egui::Context::default();
        ctx.enable_accesskit();

        let output = render_macros(&ctx, &mut app, Vec::new());
        let create = widget_position(&output, "Create macro");
        click(&ctx, &mut app, create, render_macros);
        assert_eq!(app.state.selected_macro.as_deref(), Some("macro-ui"));
        assert_eq!(
            app.state
                .active
                .as_ref()
                .unwrap()
                .configuration
                .macros
                .items
                .len(),
            1
        );

        let output = render_macros(&ctx, &mut app, Vec::new());
        let key_down = widget_position(&output, "Key down");
        click(&ctx, &mut app, key_down, render_macros);
        let active = app.state.active.as_ref().unwrap();
        assert_eq!(
            active.configuration.macros.items[0].steps,
            vec![MacroStep::KeyDown {
                key: "KEY_A".into()
            }]
        );
    }

    #[test]
    fn lighting_pointer_toggles_preserve_unspecified_and_explicit_values() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = test_app(dir.path());
        let profile = Profile::new("Lighting UI");
        app.state.active = Some(profile.clone());
        app.state.baseline = Some(profile);
        let ctx = egui::Context::default();
        ctx.enable_accesskit();

        let output = render_lighting(&ctx, &mut app, Vec::new());
        let brightness = widget_position(&output, "Set brightness");
        click(&ctx, &mut app, brightness, render_lighting);
        assert_eq!(
            app.state
                .active
                .as_ref()
                .unwrap()
                .configuration
                .lighting
                .brightness,
            Some(50)
        );

        let output = render_lighting(&ctx, &mut app, Vec::new());
        let primary_color = widget_position(&output, "Set primary color");
        click(&ctx, &mut app, primary_color, render_lighting);
        assert_eq!(
            app.state
                .active
                .as_ref()
                .unwrap()
                .configuration
                .lighting
                .color,
            Some(th80_core::config::RgbColor {
                red: 255,
                green: 255,
                blue: 255
            })
        );

        let output = render_lighting(&ctx, &mut app, Vec::new());
        let brightness = widget_position(&output, "Set brightness");
        click(&ctx, &mut app, brightness, render_lighting);
        assert_eq!(
            app.state
                .active
                .as_ref()
                .unwrap()
                .configuration
                .lighting
                .brightness,
            None
        );
    }

    #[test]
    fn lighting_enabled_state_can_switch_between_unspecified_enabled_and_disabled() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = test_app(dir.path());
        let profile = Profile::new("Lighting enabled state");
        app.state.active = Some(profile.clone());
        app.state.baseline = Some(profile);
        let ctx = egui::Context::default();
        ctx.enable_accesskit();

        assert_eq!(
            app.state
                .active
                .as_ref()
                .unwrap()
                .configuration
                .lighting
                .enabled,
            None
        );

        let output = render_lighting(&ctx, &mut app, Vec::new());
        let enabled_state = widget_position(&output, "Local lighting enabled state");
        click(&ctx, &mut app, enabled_state, render_lighting);
        let output = render_lighting(&ctx, &mut app, Vec::new());
        let disabled = widget_position(&output, "Disabled");
        click(&ctx, &mut app, disabled, render_lighting);
        assert_eq!(
            app.state
                .active
                .as_ref()
                .unwrap()
                .configuration
                .lighting
                .enabled,
            Some(false)
        );

        let output = render_lighting(&ctx, &mut app, Vec::new());
        let enabled_state = widget_position(&output, "Local lighting enabled state");
        click(&ctx, &mut app, enabled_state, render_lighting);
        let output = render_lighting(&ctx, &mut app, Vec::new());
        let unspecified = widget_position(&output, "Unspecified");
        click(&ctx, &mut app, unspecified, render_lighting);
        assert_eq!(
            app.state
                .active
                .as_ref()
                .unwrap()
                .configuration
                .lighting
                .enabled,
            None
        );

        let output = render_lighting(&ctx, &mut app, Vec::new());
        let enabled_state = widget_position(&output, "Local lighting enabled state");
        click(&ctx, &mut app, enabled_state, render_lighting);
        let output = render_lighting(&ctx, &mut app, Vec::new());
        let enabled = widget_position(&output, "Enabled");
        click(&ctx, &mut app, enabled, render_lighting);
        assert_eq!(
            app.state
                .active
                .as_ref()
                .unwrap()
                .configuration
                .lighting
                .enabled,
            Some(true)
        );
    }

    #[test]
    fn tab_and_shift_tab_move_focus_between_navigation_controls() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = test_app(dir.path());
        let profile = Profile::new("Navigation UI");
        app.state.active = Some(profile.clone());
        app.state.baseline = Some(profile);
        let ctx = egui::Context::default();

        render_navigation(&ctx, &mut app, Vec::new());
        render_navigation(
            &ctx,
            &mut app,
            vec![
                key_event(Key::Tab, true, Modifiers::NONE),
                key_event(Key::Tab, false, Modifiers::NONE),
            ],
        );
        let first = ctx.memory(|memory| memory.focused());
        render_navigation(
            &ctx,
            &mut app,
            vec![
                key_event(Key::Tab, true, Modifiers::NONE),
                key_event(Key::Tab, false, Modifiers::NONE),
            ],
        );
        let second = ctx.memory(|memory| memory.focused());
        assert!(first.is_some(), "Tab should focus a navigation control");
        assert_ne!(first, second, "Tab should advance focus");

        render_navigation(
            &ctx,
            &mut app,
            vec![
                key_event(Key::Tab, true, Modifiers::SHIFT),
                key_event(Key::Tab, false, Modifiers::SHIFT),
            ],
        );
        render_navigation(&ctx, &mut app, Vec::new());
        assert_eq!(ctx.memory(|memory| memory.focused()), first);
    }

    #[test]
    fn enter_and_space_activate_focused_navigation_buttons() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = test_app(dir.path());
        let profile = Profile::new("Keyboard activation");
        app.state.active = Some(profile.clone());
        app.state.baseline = Some(profile);
        let ctx = egui::Context::default();

        for _ in 0..2 {
            render_navigation(
                &ctx,
                &mut app,
                vec![
                    key_event(Key::Tab, true, Modifiers::NONE),
                    key_event(Key::Tab, false, Modifiers::NONE),
                ],
            );
        }
        render_navigation(
            &ctx,
            &mut app,
            vec![
                key_event(Key::Enter, true, Modifiers::NONE),
                key_event(Key::Enter, false, Modifiers::NONE),
            ],
        );
        assert_eq!(app.state.page, crate::state::Page::Keymap);

        let ctx = egui::Context::default();
        app.state.page = crate::state::Page::Dashboard;
        for _ in 0..3 {
            render_navigation(
                &ctx,
                &mut app,
                vec![
                    key_event(Key::Tab, true, Modifiers::NONE),
                    key_event(Key::Tab, false, Modifiers::NONE),
                ],
            );
        }
        render_navigation(
            &ctx,
            &mut app,
            vec![
                key_event(Key::Space, true, Modifiers::NONE),
                key_event(Key::Space, false, Modifiers::NONE),
            ],
        );
        assert_eq!(app.state.page, crate::state::Page::Macros);
    }

    #[test]
    fn escape_closes_macro_confirmation_and_preserves_macro_data() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = test_app(dir.path());
        let profile = profile_with_two_macros();
        app.state.active = Some(profile.clone());
        app.state.baseline = Some(profile);
        app.state.selected_macro = Some("macro-a".into());
        app.state.pending_delete_macro = Some("macro-a".into());
        let ctx = egui::Context::default();
        ctx.enable_accesskit();

        render_macros(&ctx, &mut app, Vec::new());
        render_macros(
            &ctx,
            &mut app,
            vec![
                key_event(Key::Escape, true, Modifiers::NONE),
                key_event(Key::Escape, false, Modifiers::NONE),
            ],
        );
        render_macros(&ctx, &mut app, Vec::new());

        assert!(app.state.pending_delete_macro.is_none());
        assert_eq!(
            app.state
                .active
                .as_ref()
                .unwrap()
                .configuration
                .macros
                .items
                .len(),
            2
        );
        assert!(!app.state.dirty());
    }

    #[test]
    fn modal_closure_restores_the_previous_keyboard_focus() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = test_app(dir.path());
        let profile = profile_with_two_macros();
        app.state.active = Some(profile.clone());
        app.state.baseline = Some(profile);
        app.state.selected_macro = Some("macro-a".into());
        let ctx = egui::Context::default();

        render_navigation_with_macro_dialog(
            &ctx,
            &mut app,
            vec![
                key_event(Key::Tab, true, Modifiers::NONE),
                key_event(Key::Tab, false, Modifiers::NONE),
            ],
        );
        let focus_before = ctx.memory(|memory| memory.focused());
        assert!(focus_before.is_some());

        app.state.pending_delete_macro = Some("macro-a".into());
        render_navigation_with_macro_dialog(&ctx, &mut app, Vec::new());
        render_navigation_with_macro_dialog(
            &ctx,
            &mut app,
            vec![
                key_event(Key::Escape, true, Modifiers::NONE),
                key_event(Key::Escape, false, Modifiers::NONE),
            ],
        );
        render_navigation_with_macro_dialog(&ctx, &mut app, Vec::new());

        assert!(app.state.pending_delete_macro.is_none());
        assert_eq!(ctx.memory(|memory| memory.focused()), focus_before);
    }

    #[test]
    fn profile_accessibility_tree_reports_text_input_names() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = test_app(dir.path());
        let profile = Profile::new("Accessibility UI");
        app.state.active = Some(profile.clone());
        app.state.baseline = Some(profile);
        let ctx = egui::Context::default();
        ctx.enable_accesskit();

        let output = render_profile(&ctx, &mut app, Vec::new());
        let update = output.platform_output.accesskit_update.unwrap();
        let text_input_labels = update
            .nodes
            .iter()
            .filter(|(_, node)| {
                matches!(
                    node.role(),
                    egui::accesskit::Role::TextInput | egui::accesskit::Role::MultilineTextInput
                )
            })
            .map(|(_, node)| node.label().map(str::to_owned))
            .collect::<Vec<_>>();
        assert!(text_input_labels.contains(&Some("Profile name".into())));
        assert!(text_input_labels.contains(&Some("Profile description".into())));
        assert!(text_input_labels.contains(&Some("Profile tags, comma separated".into())));
    }

    #[test]
    fn keyboard_focus_and_text_input_editing_work_without_pointer_events() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = test_app(dir.path());
        let mut profile = Profile::new("Keyboard profile");
        profile.name = "Keyboard profile".into();
        app.state.active = Some(profile.clone());
        app.state.baseline = Some(profile);
        let ctx = egui::Context::default();
        ctx.enable_accesskit();

        let output = render_profile(&ctx, &mut app, Vec::new());
        let name_node = widget_node_id(&output, "Profile name");
        render_profile(
            &ctx,
            &mut app,
            vec![
                key_event(Key::Tab, true, Modifiers::NONE),
                key_event(Key::Tab, false, Modifiers::NONE),
            ],
        );
        let focused = ctx.memory(|memory| memory.focused());
        assert!(focused.is_some());
        assert_eq!(focused.unwrap().accesskit_id(), name_node);

        render_profile(&ctx, &mut app, vec![Event::Text(" edited".into())]);
        assert!(app.state.active.as_ref().unwrap().name.contains("edited"));
    }

    #[test]
    fn keymap_positions_are_named_and_selectable_with_tab_and_enter() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = test_app(dir.path());
        let mut profile = Profile::new("Keyboard keymap");
        profile.configuration.keymap.layers = vec![Layer {
            id: "base".into(),
            name: "Base".into(),
            positions: BTreeMap::from([
                (
                    "logical-a".into(),
                    KeyAssignment {
                        action: Action::Unassigned,
                    },
                ),
                (
                    "logical-b".into(),
                    KeyAssignment {
                        action: Action::Unassigned,
                    },
                ),
            ]),
        }];
        app.state.active = Some(profile.clone());
        app.state.baseline = Some(profile);
        let ctx = egui::Context::default();
        ctx.enable_accesskit();

        let output = render_keymap(&ctx, &mut app, Vec::new());
        let target = widget_node_id(&output, "Select logical position logical-a");
        let update = output.platform_output.accesskit_update.unwrap();
        let labels = update
            .nodes
            .iter()
            .filter_map(|(_, node)| node.label())
            .collect::<Vec<_>>();
        assert!(labels.contains(&"Previous logical layer"));
        assert!(labels.contains(&"Next logical layer"));

        let mut reached = false;
        for _ in 0..32 {
            render_keymap(
                &ctx,
                &mut app,
                vec![
                    key_event(Key::Tab, true, Modifiers::NONE),
                    key_event(Key::Tab, false, Modifiers::NONE),
                ],
            );
            if ctx.memory(|memory| memory.focused().map(|focused| focused.accesskit_id()))
                == Some(target)
            {
                reached = true;
                break;
            }
        }
        assert!(reached, "Tab should reach the logical position button");
        render_keymap(
            &ctx,
            &mut app,
            vec![
                key_event(Key::Enter, true, Modifiers::NONE),
                key_event(Key::Enter, false, Modifiers::NONE),
            ],
        );
        assert_eq!(app.state.selected_position.as_deref(), Some("logical-a"));
    }

    #[test]
    fn long_profile_name_does_not_push_navigation_out_of_a_narrow_window() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = test_app(dir.path());
        let long_name = format!("{}wide-content-", "long-profile-").repeat(24);
        let profile = Profile::new(long_name.clone());
        app.state.active = Some(profile.clone());
        app.state.baseline = Some(profile.clone());
        app.state.profiles.push(profile);

        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let output = finish_output(ctx.run_ui(
            RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(926.0, 492.0))),
                ..Default::default()
            },
            |ui| app.navigation(ui),
        ));
        let update = output
            .platform_output
            .accesskit_update
            .expect("navigation should expose an AccessKit tree");

        let (_, selector) = update
            .nodes
            .iter()
            .find(|(_, node)| {
                node.label()
                    .is_some_and(|label| label == "Local profile selector")
            })
            .expect("local profile selector should be accessible");
        assert!(selector
            .description()
            .is_some_and(|description| description.contains(&long_name)));

        for label in ["Layout", "Advanced", "Config", "Validate"] {
            let (_, node) = update
                .nodes
                .iter()
                .find(|(_, node)| node.label().is_some_and(|name| name == label))
                .unwrap_or_else(|| panic!("{label} navigation control is missing"));
            let bounds = node
                .bounds()
                .expect("navigation control should have bounds");
            assert!(
                bounds.x0 >= 0.0 && bounds.x1 <= 926.0,
                "{label} is outside the narrow viewport: {bounds:?}"
            );
        }
    }

    #[test]
    fn interactive_controls_on_all_screens_have_accessible_names() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = test_app(dir.path());
        let mut profile = Profile::new("All screens");
        profile.configuration.keymap.layers.push(Layer {
            id: "base".into(),
            name: "Base".into(),
            positions: BTreeMap::from([(
                "logical-01".into(),
                KeyAssignment {
                    action: Action::Key {
                        code: "KEY_A".into(),
                    },
                },
            )]),
        });
        profile.configuration.macros.items.push(MacroDefinition {
            id: "macro-1".into(),
            name: "Macro one".into(),
            steps: vec![
                MacroStep::KeyDown {
                    key: "KEY_A".into(),
                },
                MacroStep::Delay { milliseconds: 50 },
                MacroStep::Text {
                    value: "sample".into(),
                },
            ],
        });
        profile.configuration.lighting.mode = Some(th80_core::config::LightingMode::Custom(
            "vendor-mode".into(),
        ));
        profile
            .configuration
            .lighting
            .per_key
            .push(th80_core::config::PerKeyColor {
                position: "logical-01".into(),
                color: th80_core::config::RgbColor {
                    red: 12,
                    green: 34,
                    blue: 56,
                },
            });
        profile
            .configuration
            .lighting
            .effect_parameters
            .insert("opaque".into(), serde_json::json!({"value": 1}));
        profile
            .configuration
            .options
            .values
            .insert("vendor.option".into(), serde_json::json!(true));
        app.state.active = Some(profile.clone());
        app.state.baseline = Some(profile);
        app.state.selected_layer = Some("base".into());
        app.state.selected_position = Some("logical-01".into());
        app.state.selected_macro = Some("macro-1".into());
        app.state
            .option_edits
            .insert("vendor.option".into(), "true".into());

        let pages: [(&str, ScreenRenderer); 7] = [
            ("Dashboard", render_dashboard),
            ("Keymap", render_keymap),
            ("Layers", render_layers),
            ("Macros", render_macros),
            ("Lighting", render_lighting),
            ("Options", render_options),
            ("Profile", render_profile),
        ];
        let interactive_roles = [
            egui::accesskit::Role::Button,
            egui::accesskit::Role::TextInput,
            egui::accesskit::Role::MultilineTextInput,
            egui::accesskit::Role::ComboBox,
            egui::accesskit::Role::CheckBox,
            egui::accesskit::Role::Slider,
            egui::accesskit::Role::SpinButton,
            egui::accesskit::Role::ColorWell,
        ];

        for (page, render) in pages {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let output = render(&ctx, &mut app, Vec::new());
            let update = output
                .platform_output
                .accesskit_update
                .expect("AccessKit tree update should be generated");
            let unlabeled = update
                .nodes
                .iter()
                .filter(|(_, node)| interactive_roles.contains(&node.role()))
                .filter(|(_, node)| node.label().is_none_or(|label| label.trim().is_empty()))
                .map(|(_, node)| (node.role(), node.value(), node.placeholder()))
                .collect::<Vec<_>>();
            assert!(
                unlabeled.is_empty(),
                "{page} has unlabeled controls: {unlabeled:?}"
            );
        }
    }

    #[test]
    fn key_action_picker_suggests_only_codes_already_in_the_local_profile() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = test_app(dir.path());
        let mut profile = Profile::new("Key picker suggestions");
        profile.configuration.keymap.layers.push(Layer {
            id: "base".into(),
            name: "Base".into(),
            positions: BTreeMap::from([
                (
                    "logical-01".into(),
                    KeyAssignment {
                        action: Action::Key {
                            code: "KEY_A".into(),
                        },
                    },
                ),
                (
                    "logical-02".into(),
                    KeyAssignment {
                        action: Action::Key {
                            code: "KEY_B".into(),
                        },
                    },
                ),
            ]),
        });
        app.state.active = Some(profile.clone());
        app.state.baseline = Some(profile);
        app.state.selected_layer = Some("base".into());
        app.state.selected_position = Some("logical-01".into());
        app.state.picker_open = true;
        app.state.picker_category = "Keys".into();
        app.state.picker_kind = "Key".into();
        app.state.picker_action_draft = Some(Action::Key {
            code: "KEY_A".into(),
        });

        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let output = render_keymap(&ctx, &mut app, Vec::new());
        let update = output.platform_output.accesskit_update.unwrap();
        let has_button = |label: &str| {
            update.nodes.iter().any(|(_, node)| {
                node.role() == egui::accesskit::Role::Button && node.label() == Some(label)
            })
        };
        assert!(has_button("KEY_A"));
        assert!(has_button("KEY_B"));
        assert!(!has_button("KEY_C"));
    }

    #[test]
    fn action_picker_cancel_preserves_the_original_assignment() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = test_app(dir.path());
        let mut profile = Profile::new("Picker cancellation");
        profile.configuration.keymap.layers.push(Layer {
            id: "base".into(),
            name: "Base".into(),
            positions: BTreeMap::from([(
                "logical-a".into(),
                KeyAssignment {
                    action: Action::Key {
                        code: "KEY_A".into(),
                    },
                },
            )]),
        });
        app.state.active = Some(profile.clone());
        app.state.baseline = Some(profile);
        app.state.selected_layer = Some("base".into());
        app.state.selected_position = Some("logical-a".into());
        app.state.picker_open = true;
        app.state.picker_target_layer = Some("base".into());
        app.state.picker_target_position = Some("logical-a".into());
        app.state.picker_action_draft = Some(Action::Key {
            code: "KEY_B".into(),
        });
        let ctx = egui::Context::default();
        ctx.enable_accesskit();

        let output = render_keymap(&ctx, &mut app, Vec::new());
        let update = output.platform_output.accesskit_update.unwrap();
        assert!(update
            .nodes
            .iter()
            .any(|(_, node)| node.role() == egui::accesskit::Role::Dialog
                && node.label() == Some("Action picker")));
        render_keymap(
            &ctx,
            &mut app,
            vec![
                key_event(Key::Escape, true, Modifiers::NONE),
                key_event(Key::Escape, false, Modifiers::NONE),
            ],
        );

        let assignment = &app
            .state
            .active
            .as_ref()
            .unwrap()
            .configuration
            .keymap
            .layers[0]
            .positions["logical-a"];
        assert_eq!(
            assignment.action,
            Action::Key {
                code: "KEY_A".into()
            }
        );
        assert!(!app.state.picker_open);
        assert!(!app.state.dirty());
    }

    #[test]
    fn action_picker_apply_updates_only_the_target_logical_position() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = test_app(dir.path());
        let mut profile = Profile::new("Picker apply");
        profile.configuration.keymap.layers.push(Layer {
            id: "base".into(),
            name: "Base".into(),
            positions: BTreeMap::from([
                (
                    "logical-a".into(),
                    KeyAssignment {
                        action: Action::Key {
                            code: "KEY_A".into(),
                        },
                    },
                ),
                (
                    "logical-b".into(),
                    KeyAssignment {
                        action: Action::Key {
                            code: "KEY_B".into(),
                        },
                    },
                ),
            ]),
        });
        app.state.active = Some(profile.clone());
        app.state.baseline = Some(profile);
        app.state.selected_layer = Some("base".into());
        app.state.selected_position = Some("logical-a".into());
        app.state.picker_open = true;
        app.state.picker_target_layer = Some("base".into());
        app.state.picker_target_position = Some("logical-a".into());
        app.state.picker_action_draft = Some(Action::Key {
            code: "KEY_C".into(),
        });
        app.state.picker_category = "Keys".into();
        app.state.picker_kind = "Key".into();
        let ctx = egui::Context::default();
        ctx.enable_accesskit();

        let output = render_keymap(&ctx, &mut app, Vec::new());
        let apply = widget_node_id(&output, "Apply action");
        activate_keymap_accessibility_node(&ctx, &mut app, apply);

        let active = app.state.active.as_ref().unwrap();
        assert_eq!(
            active.configuration.keymap.layers[0].positions["logical-a"].action,
            Action::Key {
                code: "KEY_C".into()
            }
        );
        assert_eq!(
            active.configuration.keymap.layers[0].positions["logical-b"].action,
            Action::Key {
                code: "KEY_B".into()
            }
        );
        assert!(!app.state.picker_open);
        assert!(app.state.dirty());
    }

    #[test]
    fn left_and_right_keys_navigate_logical_layers_without_text_focus() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = test_app(dir.path());
        let mut profile = Profile::new("Layer navigation");
        profile.configuration.keymap.layers = vec![
            Layer {
                id: "base".into(),
                name: "Base".into(),
                positions: BTreeMap::from([(
                    "logical-a".into(),
                    KeyAssignment {
                        action: Action::Unassigned,
                    },
                )]),
            },
            Layer {
                id: "nav".into(),
                name: "Navigation".into(),
                positions: BTreeMap::from([(
                    "logical-b".into(),
                    KeyAssignment {
                        action: Action::Unassigned,
                    },
                )]),
            },
        ];
        app.state.active = Some(profile.clone());
        app.state.baseline = Some(profile);
        app.state.selected_layer = Some("base".into());
        let ctx = egui::Context::default();

        render_keymap(
            &ctx,
            &mut app,
            vec![
                key_event(Key::ArrowRight, true, Modifiers::NONE),
                key_event(Key::ArrowRight, false, Modifiers::NONE),
            ],
        );
        assert_eq!(app.state.selected_layer.as_deref(), Some("nav"));
        render_keymap(
            &ctx,
            &mut app,
            vec![
                key_event(Key::ArrowLeft, true, Modifiers::NONE),
                key_event(Key::ArrowLeft, false, Modifiers::NONE),
            ],
        );
        assert_eq!(app.state.selected_layer.as_deref(), Some("base"));
    }

    #[test]
    fn layer_arrow_navigation_remains_available_with_position_button_focus() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = test_app(dir.path());
        let mut profile = Profile::new("Focused layer navigation");
        profile.configuration.keymap.layers = vec![
            Layer {
                id: "base".into(),
                name: "Base".into(),
                positions: BTreeMap::from([(
                    "logical-a".into(),
                    KeyAssignment {
                        action: Action::Unassigned,
                    },
                )]),
            },
            Layer {
                id: "nav".into(),
                name: "Navigation".into(),
                positions: BTreeMap::from([(
                    "logical-b".into(),
                    KeyAssignment {
                        action: Action::Unassigned,
                    },
                )]),
            },
        ];
        app.state.active = Some(profile.clone());
        app.state.baseline = Some(profile);
        app.state.selected_layer = Some("base".into());
        let ctx = egui::Context::default();
        let tabs = (0..5)
            .flat_map(|_| {
                [
                    key_event(Key::Tab, true, Modifiers::NONE),
                    key_event(Key::Tab, false, Modifiers::NONE),
                ]
            })
            .collect();
        render_keymap(&ctx, &mut app, tabs);
        render_keymap(
            &ctx,
            &mut app,
            vec![
                key_event(Key::ArrowRight, true, Modifiers::NONE),
                key_event(Key::ArrowRight, false, Modifiers::NONE),
            ],
        );
        assert_eq!(app.state.selected_layer.as_deref(), Some("nav"));
    }

    #[test]
    fn mouse_wheel_over_logical_board_navigates_layers() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = test_app(dir.path());
        let mut profile = Profile::new("Wheel layer navigation");
        profile.configuration.keymap.layers = vec![
            Layer {
                id: "base".into(),
                name: "Base".into(),
                positions: BTreeMap::from([(
                    "logical-a".into(),
                    KeyAssignment {
                        action: Action::Unassigned,
                    },
                )]),
            },
            Layer {
                id: "nav".into(),
                name: "Navigation".into(),
                positions: BTreeMap::from([(
                    "logical-b".into(),
                    KeyAssignment {
                        action: Action::Unassigned,
                    },
                )]),
            },
        ];
        app.state.active = Some(profile.clone());
        app.state.baseline = Some(profile);
        app.state.selected_layer = Some("base".into());
        let ctx = egui::Context::default();

        render_keymap(
            &ctx,
            &mut app,
            vec![
                Event::PointerMoved(Pos2::new(200.0, 150.0)),
                Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Line,
                    delta: Vec2::new(0.0, -1.0),
                    phase: egui::TouchPhase::Start,
                    modifiers: Modifiers::NONE,
                },
            ],
        );

        assert_eq!(app.state.selected_layer.as_deref(), Some("nav"));
    }

    #[test]
    fn long_option_values_remain_available_to_accessibility_and_inspection() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = test_app(dir.path());
        let mut profile = Profile::new("Long option value");
        let full_value = "long-json-value-".repeat(80);
        profile.configuration.options.values.insert(
            "long.option".into(),
            serde_json::Value::String(full_value.clone()),
        );
        app.state.active = Some(profile.clone());
        app.state.baseline = Some(profile);
        let expected_editor_value =
            serde_json::to_string(&serde_json::Value::String(full_value)).unwrap();
        app.state
            .option_edits
            .insert("long.option".into(), expected_editor_value.clone());

        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let output = render_options(&ctx, &mut app, Vec::new());
        let update = output
            .platform_output
            .accesskit_update
            .expect("options page should expose an AccessKit update");
        let (_, node) = update
            .nodes
            .iter()
            .find(|(_, node)| {
                node.label()
                    .is_some_and(|label| label == "JSON value for option long.option")
            })
            .expect("long option editor should have an accessible name");
        assert_eq!(node.value(), Some(expected_editor_value.as_str()));
        assert!(node
            .description()
            .is_some_and(|description| description.contains(&expected_editor_value)));
    }

    #[test]
    fn macro_step_reordering_controls_remain_inside_narrow_window_bounds() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = test_app(dir.path());
        let mut profile = Profile::new("Narrow macro editor");
        profile.configuration.macros.items.push(MacroDefinition {
            id: "macro-1".into(),
            name: "A macro with several steps".into(),
            steps: vec![
                MacroStep::KeyDown {
                    key: "KEY_A".into(),
                },
                MacroStep::KeyUp {
                    key: "KEY_A".into(),
                },
                MacroStep::Delay { milliseconds: 250 },
            ],
        });
        app.state.active = Some(profile.clone());
        app.state.baseline = Some(profile);
        app.state.selected_macro = Some("macro-1".into());

        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let output = finish_output(ctx.run_ui(
            RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1_000.0, 660.0))),
                ..Default::default()
            },
            |ui| screens::macros::show(&mut app, ui),
        ));
        let update = output
            .platform_output
            .accesskit_update
            .expect("macro editor should expose an AccessKit tree");
        for label in ["Move up", "Move down"] {
            let (_, node) = update
                .nodes
                .iter()
                .find(|(_, node)| node.label().is_some_and(|name| name == label))
                .unwrap_or_else(|| {
                    panic!("{label} control is missing from the narrow macro editor")
                });
            let bounds = node
                .bounds()
                .expect("reordering control should have bounds");
            assert!(
                bounds.x0 >= 0.0 && bounds.x1 <= 1_000.0,
                "{label} is horizontally clipped: {bounds:?}"
            );
            assert!(
                bounds.y0 >= 0.0 && bounds.y1 <= 660.0,
                "{label} is vertically clipped: {bounds:?}"
            );
        }
    }

    #[test]
    fn async_import_dialog_result_is_applied_on_the_ui_thread() {
        let dir = tempfile::tempdir().unwrap();
        let profile_root = dir.path().join("profiles");
        std::fs::create_dir_all(&profile_root).unwrap();
        let import_path = dir.path().join("synthetic-import.json");
        let source = Profile::new("Imported through native chooser");
        std::fs::write(&import_path, serde_json::to_vec(&source).unwrap()).unwrap();

        let mut app = test_app(&profile_root);
        let (sender, receiver) = mpsc::channel();
        sender
            .send(FileDialogResult::Import(Some(import_path)))
            .unwrap();
        app.file_dialog_result_rx = Some(receiver);
        app.file_dialog_pending = true;

        app.poll_file_dialog(&egui::Context::default());

        assert!(!app.file_dialog_pending);
        assert!(app.file_dialog_result_rx.is_none());
        assert_eq!(app.state.profiles.len(), 1);
        assert_eq!(
            app.state.active.as_ref().unwrap().name,
            "Imported through native chooser"
        );
    }

    #[test]
    fn async_export_dialog_result_writes_profile_and_clears_pending_state() {
        let dir = tempfile::tempdir().unwrap();
        let profile_root = dir.path().join("profiles");
        std::fs::create_dir_all(&profile_root).unwrap();
        let export_path = dir.path().join("synthetic-export.json");
        let mut app = test_app(&profile_root);
        let profile = Profile::new("Exported through native chooser");
        app.state.service.create(&profile).unwrap();
        app.state.active = Some(profile.clone());
        app.state.baseline = Some(profile.clone());

        let (sender, receiver) = mpsc::channel();
        sender
            .send(FileDialogResult::Export {
                profile_id: profile.id.clone(),
                path: Some(export_path.clone()),
            })
            .unwrap();
        app.file_dialog_result_rx = Some(receiver);
        app.file_dialog_pending = true;

        app.poll_file_dialog(&egui::Context::default());

        assert!(!app.file_dialog_pending);
        assert!(app.file_dialog_result_rx.is_none());
        let exported: Profile =
            serde_json::from_slice(&std::fs::read(export_path).unwrap()).unwrap();
        assert_eq!(exported.name, profile.name);
        assert_eq!(exported.id, profile.id);
    }

    #[test]
    fn async_file_dialog_waiting_state_does_not_clear_before_result_arrives() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = test_app(dir.path());
        let (sender, receiver) = mpsc::channel();
        app.file_dialog_result_rx = Some(receiver);
        app.file_dialog_pending = true;

        app.poll_file_dialog(&egui::Context::default());
        assert!(app.file_dialog_pending);

        sender.send(FileDialogResult::Import(None)).unwrap();
        app.poll_file_dialog(&egui::Context::default());
        assert!(!app.file_dialog_pending);
        assert!(app.file_dialog_result_rx.is_none());
        assert!(app.state.profiles.is_empty());
    }
}
