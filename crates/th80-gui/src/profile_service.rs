use std::path::{Path, PathBuf};

use th80_core::{CoreError, Profile, ProfileId, ProfileRepository};

/// Thin GUI adapter over the authoritative core repository.
#[derive(Clone)]
pub struct ProfileService {
    repository: ProfileRepository,
}

impl ProfileService {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            repository: ProfileRepository::new(root),
        }
    }

    pub fn root(&self) -> &Path {
        self.repository.root()
    }

    pub fn list(&self) -> Result<Vec<Profile>, CoreError> {
        self.repository.list()
    }

    pub fn load(&self, id: &ProfileId) -> Result<Profile, CoreError> {
        self.repository.load(id)
    }

    pub fn create(&self, profile: &Profile) -> Result<(), CoreError> {
        self.repository.create(profile)
    }

    pub fn save(&self, profile: &Profile) -> Result<(), CoreError> {
        self.repository.save(profile)
    }

    pub fn delete(&self, id: &ProfileId) -> Result<(), CoreError> {
        self.repository.delete(id)
    }

    pub fn import(&self, path: impl AsRef<Path>) -> Result<Profile, CoreError> {
        self.repository.import(path)
    }

    pub fn export(&self, id: &ProfileId, path: impl AsRef<Path>) -> Result<(), CoreError> {
        self.repository.export(id, path)
    }

    pub fn validate(&self, profile: &Profile) -> Result<(), CoreError> {
        self.repository.validate(profile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use th80_core::{
        config::{Action, KeyAssignment, Layer, MacroDefinition, MacroStep, PerKeyColor, RgbColor},
        validation::validate_profile,
    };

    #[test]
    fn gui_model_edits_round_trip_through_core_repository() {
        let dir = tempfile::tempdir().unwrap();
        let service = ProfileService::new(dir.path());
        let mut profile = Profile::new("Layer editor");
        profile.configuration.keymap.layers.push(Layer {
            id: "base".into(),
            name: "Base".into(),
            positions: [(
                "logical-alpha".into(),
                KeyAssignment {
                    action: Action::Key {
                        code: "KEY_A".into(),
                    },
                },
            )]
            .into_iter()
            .collect(),
        });
        profile.configuration.keymap.layers[0].positions.insert(
            "logical-beta".into(),
            KeyAssignment {
                action: Action::LayerMomentary {
                    layer_id: "base".into(),
                },
            },
        );
        profile.configuration.macros.items.push(MacroDefinition {
            id: "tap-a".into(),
            name: "Tap A".into(),
            steps: vec![
                MacroStep::KeyDown {
                    key: "KEY_A".into(),
                },
                MacroStep::KeyUp {
                    key: "KEY_A".into(),
                },
            ],
        });
        profile.configuration.lighting.enabled = Some(true);
        profile.configuration.lighting.mode = Some(th80_core::config::LightingMode::Custom(
            "vendor-mode-unknown".into(),
        ));
        profile.configuration.lighting.per_key.push(PerKeyColor {
            position: "logical-alpha".into(),
            color: RgbColor {
                red: 10,
                green: 20,
                blue: 30,
            },
        });
        assert!(validate_profile(&profile).is_empty());
        service.create(&profile).unwrap();
        let loaded = service.load(&profile.id).unwrap();
        assert_eq!(
            loaded.configuration.keymap.layers[0].positions["logical-alpha"].action,
            Action::Key {
                code: "KEY_A".into()
            }
        );
        assert_eq!(
            loaded.configuration.keymap.layers[0].positions["logical-beta"].action,
            Action::LayerMomentary {
                layer_id: "base".into()
            }
        );
        assert_eq!(loaded.configuration.macros.items[0].steps.len(), 2);
        assert_eq!(
            loaded.configuration.lighting.mode,
            Some(th80_core::config::LightingMode::Custom(
                "vendor-mode-unknown".into()
            ))
        );
    }

    #[test]
    fn core_macro_validation_is_displayable_without_gui_reimplementation() {
        let dir = tempfile::tempdir().unwrap();
        let service = ProfileService::new(dir.path());
        let mut profile = Profile::new("Invalid macro");
        profile.configuration.macros.items.push(MacroDefinition {
            id: "broken".into(),
            name: "Broken".into(),
            steps: vec![MacroStep::KeyUp {
                key: "KEY_A".into(),
            }],
        });
        let error = service.validate(&profile).unwrap_err();
        match error {
            CoreError::InvalidProfile(items) => {
                assert!(items.iter().any(|item| item.code == "invalid_key_up"))
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn lighting_validation_uses_core_normalized_policy() {
        let dir = tempfile::tempdir().unwrap();
        let service = ProfileService::new(dir.path());
        let mut profile = Profile::new("Lighting bounds");
        profile.configuration.lighting.brightness = Some(101);
        let error = service.validate(&profile).unwrap_err();
        match error {
            CoreError::InvalidProfile(items) => assert!(items
                .iter()
                .any(|item| item.code == "brightness_out_of_range")),
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn unknown_profile_and_configuration_extensions_survive_save_and_load() {
        let dir = tempfile::tempdir().unwrap();
        let service = ProfileService::new(dir.path());
        let mut profile = Profile::new("Extensions");
        profile.extensions.insert(
            "future_vendor_extension".into(),
            serde_json::json!({"nested": [1, 2, 3], "flag": true}),
        );
        profile
            .configuration
            .options
            .values
            .insert("local.custom".into(), serde_json::json!({"enabled": true}));
        profile.configuration.extensions.insert(
            "future_transport_section".into(),
            serde_json::json!({"mode": "uninterpreted"}),
        );
        profile.configuration.keymap.extensions.insert(
            "future_layout".into(),
            serde_json::json!({"logical_slots": 88}),
        );
        profile.configuration.lighting.extensions.insert(
            "future_color_space".into(),
            serde_json::json!("vendor-space"),
        );
        profile
            .metadata
            .extensions
            .insert("future_metadata".into(), serde_json::json!({"revision": 9}));
        service.create(&profile).unwrap();
        let loaded = service.load(&profile.id).unwrap();
        assert_eq!(
            loaded.extensions["future_vendor_extension"],
            serde_json::json!({"nested": [1, 2, 3], "flag": true})
        );
        assert_eq!(
            loaded.configuration.options.values["local.custom"],
            serde_json::json!({"enabled": true})
        );
        assert_eq!(
            loaded.configuration.extensions["future_transport_section"],
            serde_json::json!({"mode": "uninterpreted"})
        );
        assert_eq!(
            loaded.configuration.keymap.extensions["future_layout"],
            serde_json::json!({"logical_slots": 88})
        );
        assert_eq!(
            loaded.configuration.lighting.extensions["future_color_space"],
            serde_json::json!("vendor-space")
        );
        assert_eq!(
            loaded.metadata.extensions["future_metadata"],
            serde_json::json!({"revision": 9})
        );
    }

    #[test]
    fn import_export_and_stale_save_conflicts_use_core_repository() {
        let source_dir = tempfile::tempdir().unwrap();
        let export_dir = tempfile::tempdir().unwrap();
        let import_dir = tempfile::tempdir().unwrap();
        let source = ProfileService::new(source_dir.path());
        let target = ProfileService::new(import_dir.path());
        let profile = Profile::new("Portable");
        source.create(&profile).unwrap();
        let exported = export_dir.path().join("portable.json");
        source.export(&profile.id, &exported).unwrap();
        let imported = target.import(&exported).unwrap();
        assert_eq!(imported.id, profile.id);
        assert_eq!(target.load(&profile.id).unwrap().name, "Portable");

        let mut first = source.load(&profile.id).unwrap();
        let mut stale = source.load(&profile.id).unwrap();
        first.description = "newer writer".into();
        source.save(&first).unwrap();
        stale.description = "stale writer".into();
        assert!(matches!(source.save(&stale), Err(CoreError::Conflict(_))));
    }

    #[test]
    fn repository_errors_are_returned_as_structured_core_errors() {
        let dir = tempfile::tempdir().unwrap();
        let service = ProfileService::new(dir.path());
        assert!(matches!(
            service.load(&ProfileId::new("missing")),
            Err(CoreError::NotFound(_))
        ));
        let bad = dir.path().join("bad.json");
        std::fs::write(&bad, "{ not json }").unwrap();
        assert!(matches!(service.import(&bad), Err(CoreError::Json(_))));
    }
}
