use std::path::PathBuf;
use std::time::Instant;

use th80_core::{config::Action, Profile, ProfileId};

use crate::profile_service::ProfileService;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Page {
    #[default]
    Dashboard,
    Keymap,
    Layers,
    Macros,
    Lighting,
    Options,
    Profile,
    Settings,
    Hardware,
}

pub struct AppState {
    pub service: ProfileService,
    pub profiles: Vec<Profile>,
    pub active: Option<Profile>,
    pub baseline: Option<Profile>,
    pub page: Page,
    pub selected_layer: Option<String>,
    pub selected_position: Option<String>,
    pub selected_macro: Option<String>,
    pub message: Option<(bool, String)>,
    pub save_conflict: bool,
    pub pending_open: Option<ProfileId>,
    pub pending_delete_profile: Option<ProfileId>,
    pub pending_delete_macro: Option<String>,
    pub new_profile_name: String,
    pub new_layer_id: String,
    pub new_layer_name: String,
    pub new_position_id: String,
    pub new_macro_id: String,
    pub new_macro_name: String,
    pub macro_key: String,
    pub macro_text: String,
    pub macro_delay: String,
    pub option_key: String,
    pub option_json: String,
    pub option_edits: std::collections::BTreeMap<String, String>,
    pub lighting_position: String,
    pub lighting_parameter_key: String,
    pub lighting_parameter_json: String,
    pub dark_mode: bool,
    pub ui_scale: f32,
    pub file_dialog_start_dir: String,
    pub picker_open: bool,
    pub picker_target_layer: Option<String>,
    pub picker_target_position: Option<String>,
    pub picker_action_draft: Option<Action>,
    pub picker_query: String,
    pub picker_category: String,
    pub picker_kind: String,
    pub accent_color: [u8; 3],
    pub layer_rename: bool,
    pub hardware_snapshot: Option<crate::hardware::HardwareSnapshot>,
    pub hardware_error: Option<String>,
    pub hardware_last_scan: Option<Instant>,
}

impl AppState {
    pub fn new(service: ProfileService) -> Self {
        let mut state = Self {
            service,
            profiles: Vec::new(),
            active: None,
            baseline: None,
            page: Page::Keymap,
            selected_layer: None,
            selected_position: None,
            selected_macro: None,
            message: None,
            save_conflict: false,
            pending_open: None,
            pending_delete_profile: None,
            pending_delete_macro: None,
            new_profile_name: "My TH80 profile".into(),
            new_layer_id: "layer-1".into(),
            new_layer_name: "Layer 1".into(),
            new_position_id: String::new(),
            new_macro_id: "macro-1".into(),
            new_macro_name: "New macro".into(),
            macro_key: "KEY_A".into(),
            macro_text: String::new(),
            macro_delay: "100".into(),
            option_key: String::new(),
            option_json: "true".into(),
            option_edits: Default::default(),
            lighting_position: String::new(),
            lighting_parameter_key: String::new(),
            lighting_parameter_json: "null".into(),
            dark_mode: true,
            ui_scale: 1.0,
            file_dialog_start_dir: String::new(),
            picker_open: false,
            picker_target_layer: None,
            picker_target_position: None,
            picker_action_draft: None,
            picker_query: String::new(),
            picker_category: "Keys".into(),
            picker_kind: "Key".into(),
            accent_color: [190, 126, 145],
            layer_rename: false,
            hardware_snapshot: None,
            hardware_error: None,
            hardware_last_scan: None,
        };
        state.refresh_profiles();
        // A single local profile is an unambiguous startup target. Opening it
        // keeps the primary Layout workspace useful without changing the
        // explicit-save model or silently selecting among multiple profiles.
        if state.profiles.len() == 1 {
            if let Some(id) = state.profiles.first().map(|profile| profile.id.clone()) {
                state.open_now(&id);
                state.message = None;
            }
        }
        state
    }

    pub fn dirty(&self) -> bool {
        matches!((&self.active, &self.baseline), (Some(current), Some(saved)) if current != saved)
    }

    /// Refreshes passive sysfs/descriptor discovery only. This never opens a
    /// HID node, submits a report, or changes keyboard state.
    pub fn refresh_hardware(&mut self) {
        self.hardware_last_scan = Some(Instant::now());
        match crate::hardware::scan() {
            Ok(snapshot) => {
                self.hardware_snapshot = Some(snapshot);
                self.hardware_error = None;
            }
            Err(error) => {
                self.hardware_snapshot = None;
                self.hardware_error = Some(error);
            }
        }
    }

    pub fn refresh_profiles(&mut self) {
        match self.service.list() {
            Ok(mut profiles) => {
                profiles
                    .sort_by_key(|profile| std::cmp::Reverse(profile.metadata.modified_at_unix_ms));
                self.profiles = profiles;
            }
            Err(error) => self.message = Some((false, format_error(&error))),
        }
    }

    pub fn open(&mut self, id: &ProfileId) {
        if self.dirty() {
            self.pending_open = Some(id.clone());
            return;
        }
        self.open_now(id);
    }

    pub fn open_now(&mut self, id: &ProfileId) {
        match self.service.load(id) {
            Ok(profile) => {
                self.selected_layer = profile
                    .configuration
                    .keymap
                    .layers
                    .first()
                    .map(|l| l.id.clone());
                self.selected_position = None;
                self.selected_macro = profile
                    .configuration
                    .macros
                    .items
                    .first()
                    .map(|m| m.id.clone());
                self.option_edits.clear();
                self.lighting_position.clear();
                self.lighting_parameter_key.clear();
                self.lighting_parameter_json = "null".into();
                self.baseline = Some(profile.clone());
                self.active = Some(profile);
                self.page = Page::Keymap;
                self.pending_open = None;
                self.pending_delete_profile = None;
                self.pending_delete_macro = None;
                self.save_conflict = false;
                self.message = Some((true, "Profile opened offline".into()));
            }
            Err(error) => self.message = Some((false, format_error(&error))),
        }
    }

    pub fn create_profile(&mut self) {
        let name = self.new_profile_name.trim();
        if name.is_empty() {
            self.message = Some((false, "Enter a profile name".into()));
            return;
        }
        let profile = Profile::new(name.to_owned());
        match self.service.create(&profile) {
            Ok(()) => {
                self.refresh_profiles();
                if self.dirty() {
                    self.pending_open = Some(profile.id.clone());
                    self.message = Some((
                        true,
                        "Profile created. Save or discard the current draft to open it.".into(),
                    ));
                } else {
                    self.open_now(&profile.id);
                    self.message = Some((true, "Profile created".into()));
                }
            }
            Err(error) => self.message = Some((false, format_error(&error))),
        }
    }

    pub fn save(&mut self) {
        let Some(profile) = self.active.as_ref() else {
            return;
        };
        match self
            .service
            .validate(profile)
            .and_then(|()| self.service.save(profile))
        {
            Ok(()) => {
                let id = profile.id.clone();
                match self.service.load(&id) {
                    Ok(saved) => {
                        self.baseline = Some(saved.clone());
                        self.active = Some(saved);
                        self.save_conflict = false;
                        self.refresh_profiles();
                        self.message = Some((true, "Profile saved".into()));
                    }
                    Err(error) => self.message = Some((false, format_error(&error))),
                }
            }
            Err(error) => {
                self.save_conflict = matches!(&error, th80_core::CoreError::Conflict(_));
                self.message = Some((false, format_error(&error)));
            }
        }
    }

    /// Replaces the in-memory draft with the latest repository version after explicit user choice.
    pub fn reload_active(&mut self) {
        let Some(id) = self.active.as_ref().map(|profile| profile.id.clone()) else {
            return;
        };
        match self.service.load(&id) {
            Ok(profile) => {
                self.selected_layer = profile
                    .configuration
                    .keymap
                    .layers
                    .first()
                    .map(|layer| layer.id.clone());
                self.selected_position = None;
                self.selected_macro = profile
                    .configuration
                    .macros
                    .items
                    .first()
                    .map(|item| item.id.clone());
                self.option_edits.clear();
                self.lighting_position.clear();
                self.lighting_parameter_key.clear();
                self.lighting_parameter_json = "null".into();
                self.baseline = Some(profile.clone());
                self.active = Some(profile);
                self.save_conflict = false;
                self.refresh_profiles();
                self.message = Some((
                    true,
                    "Reloaded the latest saved profile. Previous in-memory edits were discarded."
                        .into(),
                ));
            }
            Err(error) => self.message = Some((false, format_error(&error))),
        }
    }

    pub fn save_as(&mut self) {
        let Some(source) = self.active.as_ref() else {
            return;
        };
        let mut copy = Profile::new(format!("{} copy", source.name));
        copy.name = format!("{} copy", source.name);
        copy.description = source.description.clone();
        copy.device_model = source.device_model;
        copy.configuration = source.configuration.clone();
        copy.metadata.tags = source.metadata.tags.clone();
        copy.metadata.extensions = source.metadata.extensions.clone();
        copy.extensions = source.extensions.clone();
        match self.service.create(&copy) {
            Ok(()) => {
                self.refresh_profiles();
                self.open_now(&copy.id);
                self.message = Some((true, "Saved as a new profile".into()));
            }
            Err(error) => self.message = Some((false, format_error(&error))),
        }
    }

    /// Duplicates a saved profile using a fresh identity and fresh local timestamps.
    /// If another profile has unsaved edits, opening the duplicate is deferred.
    pub fn duplicate_profile(&mut self, id: &ProfileId) {
        let source = match self.service.load(id) {
            Ok(profile) => profile,
            Err(error) => {
                self.message = Some((false, format_error(&error)));
                return;
            }
        };
        let mut duplicate = Profile::new(format!("{} copy", source.name));
        duplicate.description = source.description;
        duplicate.device_model = source.device_model;
        duplicate.configuration = source.configuration;
        duplicate.metadata.tags = source.metadata.tags;
        duplicate.metadata.extensions = source.metadata.extensions;
        duplicate.extensions = source.extensions;
        match self.service.create(&duplicate) {
            Ok(()) => {
                self.refresh_profiles();
                if self.dirty() {
                    self.pending_open = Some(duplicate.id);
                    self.message = Some((
                        true,
                        "Profile duplicated. Save or discard the current edits to open the copy."
                            .into(),
                    ));
                } else {
                    self.open_now(&duplicate.id);
                    self.message = Some((true, "Profile duplicated".into()));
                }
            }
            Err(error) => self.message = Some((false, format_error(&error))),
        }
    }

    pub fn import_file(&mut self, path: PathBuf) {
        match self.service.import(path) {
            Ok(profile) => {
                self.refresh_profiles();
                if self.dirty() {
                    self.pending_open = Some(profile.id.clone());
                    self.message = Some((
                        true,
                        "Profile imported. Save or discard the current draft to open it.".into(),
                    ));
                } else {
                    self.open_now(&profile.id);
                    self.message = Some((true, "Profile imported".into()));
                }
            }
            Err(error) => self.message = Some((false, format_error(&error))),
        }
    }

    /// Deletes the profile identity captured when the confirmation dialog opened.
    pub fn delete_profile(&mut self, id: &ProfileId) {
        match self.service.delete(id) {
            Ok(()) => {
                let deleting_active = self
                    .active
                    .as_ref()
                    .is_some_and(|profile| profile.id == *id);
                if deleting_active {
                    self.active = None;
                    self.baseline = None;
                    self.option_edits.clear();
                    self.lighting_position.clear();
                    self.lighting_parameter_key.clear();
                    self.selected_layer = None;
                    self.selected_position = None;
                    self.page = Page::Dashboard;
                }
                self.pending_delete_profile = None;
                self.pending_delete_macro = None;
                self.save_conflict = false;
                self.refresh_profiles();
                self.message = Some((true, "Profile deleted".into()));
            }
            Err(error) => self.message = Some((false, format_error(&error))),
        }
    }

    /// Removes the selected macro from the active local draft after explicit confirmation.
    pub fn delete_macro(&mut self, macro_id: &str) {
        let Some(profile) = self.active.as_mut() else {
            self.pending_delete_macro = None;
            return;
        };
        let Some(index) = profile
            .configuration
            .macros
            .items
            .iter()
            .position(|item| item.id == macro_id)
        else {
            self.pending_delete_macro = None;
            return;
        };

        profile.configuration.macros.items.remove(index);
        if self.selected_macro.as_deref() == Some(macro_id) {
            self.selected_macro = profile
                .configuration
                .macros
                .items
                .first()
                .map(|item| item.id.clone());
        }
        self.pending_delete_macro = None;
        self.message = Some((true, "Macro removed from the local draft".into()));
    }
}

pub fn format_error(error: &th80_core::CoreError) -> String {
    format!("{} — {error}", error.code())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile_service::ProfileService;

    #[test]
    fn dirty_state_tracks_model_changes_and_returns_clean_after_restore() {
        let dir = tempfile::tempdir().unwrap();
        let mut state = AppState::new(ProfileService::new(dir.path()));
        let profile = Profile::new("Draft");
        state.baseline = Some(profile.clone());
        state.active = Some(profile.clone());
        assert!(!state.dirty());
        state.active.as_mut().unwrap().description = "edited".into();
        assert!(state.dirty());
        state.active = Some(profile);
        assert!(!state.dirty());
    }

    #[test]
    fn single_local_profile_opens_in_the_layout_workspace_at_startup() {
        let dir = tempfile::tempdir().unwrap();
        let service = ProfileService::new(dir.path());
        let profile = Profile::new("Startup profile");
        service.create(&profile).unwrap();

        let state = AppState::new(ProfileService::new(dir.path()));

        assert_eq!(state.page, Page::Keymap);
        assert_eq!(
            state.active.as_ref().map(|value| &value.id),
            Some(&profile.id)
        );
        assert!(!state.dirty());
        assert!(state.message.is_none());
    }

    #[test]
    fn opening_profile_with_unspecified_lighting_does_not_mutate_it() {
        let dir = tempfile::tempdir().unwrap();
        let service = ProfileService::new(dir.path());
        let profile = Profile::new("Unspecified lighting");
        service.create(&profile).unwrap();

        let mut state = AppState::new(service);
        state.open(&profile.id);

        let active = state.active.as_ref().unwrap();
        assert_eq!(active.configuration.lighting.enabled, None);
        assert_eq!(active.configuration.lighting.color, None);
        assert_eq!(active.configuration.lighting.brightness, None);
        assert_eq!(active.configuration.lighting.speed, None);
        assert!(!state.dirty());
    }

    #[test]
    fn dashboard_profiles_are_sorted_by_most_recent_modification() {
        let dir = tempfile::tempdir().unwrap();
        let service = ProfileService::new(dir.path());
        let mut older = Profile::new("Older");
        older.metadata.created_at_unix_ms = 100;
        older.metadata.modified_at_unix_ms = 100;
        let mut newer = Profile::new("Newer");
        newer.metadata.created_at_unix_ms = 200;
        newer.metadata.modified_at_unix_ms = 200;
        service.create(&older).unwrap();
        service.create(&newer).unwrap();

        let mut state = AppState::new(service);
        state.refresh_profiles();
        assert_eq!(state.profiles[0].name, "Newer");
        assert_eq!(state.profiles[1].name, "Older");
    }

    #[test]
    fn duplicate_profile_gets_new_identity_and_preserves_configuration_and_all_extensions() {
        let dir = tempfile::tempdir().unwrap();
        let service = ProfileService::new(dir.path());
        let mut source = Profile::new("Source");
        source.description = "A portable profile".into();
        source.metadata.tags = vec!["work".into(), "quiet".into()];
        source
            .metadata
            .extensions
            .insert("future_metadata".into(), serde_json::json!({"revision": 4}));
        source.configuration.extensions.insert(
            "future_section".into(),
            serde_json::json!({"opaque": [1, 2, 3]}),
        );
        source.extensions.insert(
            "future_extension".into(),
            serde_json::json!({"nested": [1, 2]}),
        );
        service.create(&source).unwrap();

        let mut state = AppState::new(service.clone());
        state.duplicate_profile(&source.id);

        let duplicate = state.active.as_ref().unwrap();
        assert_ne!(duplicate.id, source.id);
        assert_eq!(duplicate.name, "Source copy");
        assert_eq!(duplicate.description, source.description);
        assert_eq!(duplicate.metadata.tags, source.metadata.tags);
        assert_eq!(
            duplicate.metadata.extensions["future_metadata"],
            serde_json::json!({"revision": 4})
        );
        assert_eq!(
            duplicate.configuration.extensions["future_section"],
            serde_json::json!({"opaque": [1, 2, 3]})
        );
        assert_eq!(
            duplicate.extensions["future_extension"],
            serde_json::json!({"nested": [1, 2]})
        );
        assert_eq!(service.list().unwrap().len(), 2);
        assert!(!state.dirty());
    }

    #[test]
    fn switching_profiles_clears_profile_specific_option_editor_buffers() {
        let dir = tempfile::tempdir().unwrap();
        let service = ProfileService::new(dir.path());
        let first = Profile::new("First");
        let second = Profile::new("Second");
        service.create(&first).unwrap();
        service.create(&second).unwrap();
        let mut state = AppState::new(service);
        state.option_edits.insert("same.key".into(), "true".into());
        state.open(&first.id);
        assert!(state.option_edits.is_empty());
        state.option_edits.insert("same.key".into(), "false".into());
        state.open(&second.id);
        assert!(state.option_edits.is_empty());
    }

    #[test]
    fn confirmed_discard_opens_target_without_persisting_the_original_draft() {
        let dir = tempfile::tempdir().unwrap();
        let service = ProfileService::new(dir.path());
        let mut original = Profile::new("Saved original name");
        original.description = "Saved original description".into();
        original.metadata.tags = vec!["saved-tag".into()];
        original
            .configuration
            .options
            .values
            .insert("saved.option".into(), serde_json::json!("saved value"));
        let target = Profile::new("Target profile");
        service.create(&original).unwrap();
        service.create(&target).unwrap();

        let mut state = AppState::new(service.clone());
        state.open(&original.id);
        state.active.as_mut().unwrap().name = "Discarded draft name".into();
        state.active.as_mut().unwrap().description = "Discarded draft description".into();
        state.active.as_mut().unwrap().metadata.tags = vec!["discarded-tag".into()];
        state
            .active
            .as_mut()
            .unwrap()
            .configuration
            .options
            .values
            .insert("saved.option".into(), serde_json::json!("discarded value"));
        assert!(state.dirty());

        state.open(&target.id);
        assert_eq!(state.pending_open.as_ref(), Some(&target.id));
        assert!(state.dirty());

        // The UI's explicit Discard and open action calls open_now for this ID.
        state.open_now(&target.id);
        assert_eq!(
            state.active.as_ref().map(|profile| &profile.id),
            Some(&target.id)
        );
        assert!(!state.dirty());
        assert!(state.pending_open.is_none());

        let persisted = service.load(&original.id).unwrap();
        assert_eq!(persisted.name, "Saved original name");
        assert_eq!(persisted.description, "Saved original description");
        assert_eq!(persisted.metadata.tags, vec!["saved-tag"]);
        assert_eq!(
            persisted.configuration.options.values["saved.option"],
            serde_json::json!("saved value")
        );

        state.open(&original.id);
        let reopened = state.active.as_ref().unwrap();
        assert_eq!(reopened.name, "Saved original name");
        assert_eq!(reopened.description, "Saved original description");
        assert_eq!(reopened.metadata.tags, vec!["saved-tag"]);
        assert_eq!(
            reopened.configuration.options.values["saved.option"],
            serde_json::json!("saved value")
        );
        assert!(!state.dirty());
    }

    #[test]
    fn stale_save_offers_explicit_reload_and_replaces_the_local_draft() {
        let dir = tempfile::tempdir().unwrap();
        let service = ProfileService::new(dir.path());
        let source = Profile::new("Conflict");
        service.create(&source).unwrap();
        let mut state = AppState::new(service.clone());
        state.open(&source.id);
        state.active.as_mut().unwrap().description = "local draft".into();

        let mut external = service.load(&source.id).unwrap();
        external.description = "saved by another process".into();
        service.save(&external).unwrap();

        state.save();
        assert!(state.save_conflict);
        assert!(state.dirty());
        state.reload_active();

        assert!(!state.save_conflict);
        assert!(!state.dirty());
        assert_eq!(
            state.active.as_ref().unwrap().description,
            "saved by another process"
        );
    }

    #[test]
    fn confirmed_macro_deletion_updates_draft_and_selects_remaining_macro() {
        use th80_core::config::MacroDefinition;

        let dir = tempfile::tempdir().unwrap();
        let mut profile = Profile::new("Macro deletion");
        profile.configuration.macros.items = vec![
            MacroDefinition {
                id: "first".into(),
                name: "First".into(),
                steps: Vec::new(),
            },
            MacroDefinition {
                id: "second".into(),
                name: "Second".into(),
                steps: Vec::new(),
            },
        ];
        let mut state = AppState::new(ProfileService::new(dir.path()));
        state.baseline = Some(profile.clone());
        state.active = Some(profile.clone());
        state.selected_macro = Some("first".into());
        state.pending_delete_macro = Some("first".into());

        state.delete_macro("first");

        let active = state.active.as_ref().unwrap();
        assert_eq!(active.configuration.macros.items.len(), 1);
        assert_eq!(active.configuration.macros.items[0].id, "second");
        assert_eq!(state.selected_macro.as_deref(), Some("second"));
        assert!(state.pending_delete_macro.is_none());
        assert!(state.dirty());
        assert_eq!(profile.configuration.macros.items.len(), 2);
    }

    #[test]
    fn profile_delete_confirmation_keeps_its_original_target_if_active_selection_changes() {
        let dir = tempfile::tempdir().unwrap();
        let service = ProfileService::new(dir.path());
        let first = Profile::new("Original delete target");
        let second = Profile::new("Later active profile");
        service.create(&first).unwrap();
        service.create(&second).unwrap();

        let mut state = AppState::new(service.clone());
        state.open(&first.id);
        state.pending_delete_profile = Some(first.id.clone());
        assert_eq!(state.pending_delete_profile.as_ref(), Some(&first.id));

        // Simulate a selection change from another state transition while the
        // confirmation is pending. Confirmation must still target `first`.
        state.active = Some(service.load(&second.id).unwrap());
        let captured_target = state.pending_delete_profile.clone().unwrap();
        state.delete_profile(&captured_target);

        assert!(service.load(&first.id).is_err());
        assert!(service.load(&second.id).is_ok());
        assert_eq!(
            state.active.as_ref().map(|profile| &profile.id),
            Some(&second.id)
        );
        assert!(state.pending_delete_profile.is_none());
        assert_eq!(state.page, Page::Keymap);
    }
}
