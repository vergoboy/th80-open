use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

static PROFILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
/// Stable profile identifier; repository APIs enforce safe path characters.
pub struct ProfileId(pub String);

impl ProfileId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Versioned offline profile and its local configuration.
pub struct Profile {
    pub schema_version: u32,
    pub id: ProfileId,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub device_model: Option<crate::DeviceKind>,
    #[serde(default)]
    pub configuration: Configuration,
    #[serde(default)]
    pub metadata: ProfileMetadata,
    #[serde(default)]
    pub provenance: ConfigurationProvenance,
    /// Preserves unknown top-level extension fields across load/save cycles.
    #[serde(default, flatten)]
    pub extensions: BTreeMap<String, Value>,
}

impl Profile {
    pub fn new(name: impl Into<String>) -> Self {
        let now = now_unix_ms();
        Self {
            schema_version: crate::PROFILE_SCHEMA_VERSION,
            id: ProfileId::new(format!(
                "profile-{now}-{}",
                PROFILE_SEQUENCE.fetch_add(1, Ordering::Relaxed)
            )),
            name: name.into(),
            description: String::new(),
            device_model: Some(crate::DeviceKind::Th80Pro),
            configuration: Configuration::default(),
            metadata: ProfileMetadata {
                created_at_unix_ms: now,
                modified_at_unix_ms: now,
                tags: Vec::new(),
                extensions: BTreeMap::new(),
            },
            provenance: ConfigurationProvenance::LocalDraft,
            extensions: BTreeMap::new(),
        }
    }
}

fn now_unix_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis().min(u64::MAX as u128) as u64)
        .unwrap_or(0)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Local profile timestamps and user tags.
pub struct ProfileMetadata {
    pub created_at_unix_ms: u64,
    pub modified_at_unix_ms: u64,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Retains metadata introduced by newer profile producers.
    #[serde(default, flatten)]
    pub extensions: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
/// Describes the source/state of profile data, never inferred from successful file writes.
pub enum ConfigurationProvenance {
    #[default]
    LocalDraft,
    ImportedFile,
    ValidatedLocally,
    SerializedToDisk,
    DeviceReported,
    DeviceApplied,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
/// Complete offline configuration abstraction.
pub struct Configuration {
    #[serde(default)]
    pub keymap: keymap::Keymap,
    #[serde(default)]
    pub macros: macros::MacroSet,
    #[serde(default)]
    pub lighting: lighting::LightingConfig,
    #[serde(default)]
    pub options: KeyboardOptions,
    /// Retains future configuration sections without assigning them semantics.
    #[serde(default, flatten)]
    pub extensions: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
/// Application-owned extensible options; keys do not imply device settings.
pub struct KeyboardOptions {
    /// Extensible application-owned options. No TH80 firmware semantics are implied.
    #[serde(default)]
    pub values: BTreeMap<String, Value>,
    /// Retains future option metadata without assigning device semantics.
    #[serde(default, flatten)]
    pub extensions: BTreeMap<String, Value>,
}

pub mod keymap {
    use super::*;
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
    /// Logical layers and positions; no physical matrix geometry is implied.
    pub struct Keymap {
        #[serde(default)]
        pub layers: Vec<Layer>,
        /// Retains future keymap-level metadata without interpreting it.
        #[serde(default, flatten)]
        pub extensions: BTreeMap<String, Value>,
    }
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    /// Named logical layer.
    pub struct Layer {
        pub id: String,
        pub name: String,
        #[serde(default)]
        pub positions: BTreeMap<String, KeyAssignment>,
    }
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct KeyAssignment {
        pub action: Action,
    }
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
    /// Typed local action assigned to one logical position.
    pub enum Action {
        Key { code: String },
        Modifier { modifier: String },
        Combination { keys: Vec<String> },
        LayerSwitch { layer_id: String },
        LayerMomentary { layer_id: String },
        Macro { macro_id: String },
        Transparent,
        Unassigned,
    }
}

pub mod macros {
    use super::*;
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
    /// Collection of local macro definitions.
    pub struct MacroSet {
        #[serde(default)]
        pub items: Vec<MacroDefinition>,
        /// Retains future macro-set metadata without interpreting it.
        #[serde(default, flatten)]
        pub extensions: BTreeMap<String, Value>,
    }
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    /// Ordered macro definition in the application model.
    pub struct MacroDefinition {
        pub id: String,
        pub name: String,
        #[serde(default)]
        pub steps: Vec<MacroStep>,
    }
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
    /// One ordered macro step; not a firmware packet representation.
    pub enum MacroStep {
        KeyDown { key: String },
        KeyUp { key: String },
        Delay { milliseconds: u32 },
        Text { value: String },
    }
}

pub mod lighting {
    use super::*;
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
    /// Generic lighting preferences with no asserted TH80 firmware encoding.
    pub struct LightingConfig {
        pub enabled: Option<bool>,
        pub mode: Option<LightingMode>,
        pub brightness: Option<u8>,
        pub speed: Option<u8>,
        pub color: Option<RgbColor>,
        #[serde(default)]
        pub per_key: Vec<PerKeyColor>,
        #[serde(default)]
        pub effect_parameters: BTreeMap<String, Value>,
        /// Retains future lighting fields without assigning firmware semantics.
        #[serde(default, flatten)]
        pub extensions: BTreeMap<String, Value>,
    }
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    /// Generic application lighting mode.
    pub enum LightingMode {
        Static,
        Breathing,
        Wave,
        Reactive,
        Custom(String),
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    /// RGB channels represented as 8-bit local color values.
    pub struct RgbColor {
        pub red: u8,
        pub green: u8,
        pub blue: u8,
    }
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    /// Color assigned to an opaque logical position.
    pub struct PerKeyColor {
        pub position: String,
        pub color: RgbColor,
    }
}

pub use keymap::{Action, KeyAssignment, Keymap, Layer};
pub use lighting::{LightingConfig, LightingMode, PerKeyColor, RgbColor};
pub use macros::{MacroDefinition, MacroSet, MacroStep};

#[cfg(test)]
mod lighting_serde_tests {
    use super::*;

    #[test]
    fn absent_lighting_section_and_optional_values_remain_unspecified() {
        let json = r#"{
            "schema_version": 1,
            "id": "lighting-absent",
            "name": "Absent lighting",
            "configuration": {"keymap": {}, "macros": {}, "options": {}},
            "metadata": {"created_at_unix_ms": 1, "modified_at_unix_ms": 1}
        }"#;
        let profile: Profile = serde_json::from_str(json).unwrap();
        let lighting = profile.configuration.lighting;
        assert_eq!(lighting.enabled, None);
        assert_eq!(lighting.mode, None);
        assert_eq!(lighting.brightness, None);
        assert_eq!(lighting.speed, None);
        assert_eq!(lighting.color, None);
        assert!(lighting.per_key.is_empty());
        assert!(lighting.effect_parameters.is_empty());
    }

    #[test]
    fn explicit_false_zero_empty_custom_mode_and_black_color_are_not_defaults() {
        let lighting = LightingConfig {
            enabled: Some(false),
            mode: Some(LightingMode::Custom(String::new())),
            brightness: Some(0),
            speed: Some(0),
            color: Some(RgbColor {
                red: 0,
                green: 0,
                blue: 0,
            }),
            per_key: Vec::new(),
            effect_parameters: BTreeMap::new(),
            extensions: BTreeMap::new(),
        };
        let encoded = serde_json::to_string(&lighting).unwrap();
        let decoded: LightingConfig = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, lighting);
        assert_eq!(decoded.enabled, Some(false));
        assert_eq!(decoded.brightness, Some(0));
        assert_eq!(decoded.speed, Some(0));
        assert_eq!(decoded.mode, Some(LightingMode::Custom(String::new())));
        assert_eq!(decoded.color.unwrap().red, 0);
    }

    #[test]
    fn absent_optional_lighting_fields_and_explicit_null_share_none_semantics() {
        let absent: LightingConfig = serde_json::from_str("{}").unwrap();
        let explicit_null: LightingConfig = serde_json::from_str(
            r#"{"enabled":null,"mode":null,"brightness":null,"speed":null,"color":null}"#,
        )
        .unwrap();
        assert_eq!(absent, explicit_null);
        let serialized = serde_json::to_value(absent).unwrap();
        assert_eq!(serialized["enabled"], serde_json::Value::Null);
        assert_eq!(serialized["brightness"], serde_json::Value::Null);
    }

    #[test]
    fn unknown_lighting_fields_are_preserved_without_assigning_semantics() {
        let lighting: LightingConfig = serde_json::from_str(r#"{"future_rgb_field":7}"#).unwrap();
        assert_eq!(lighting.extensions["future_rgb_field"], 7);
    }

    #[test]
    fn unknown_profile_and_configuration_sections_survive_round_trip() {
        let source = serde_json::json!({
            "schema_version": 1,
            "id": "forward-compatible",
            "name": "Forward compatible",
            "configuration": {
                "keymap": {"layers": []},
                "macros": {"items": []},
                "lighting": {},
                "options": {"values": {"local.option": true}},
                "future_transport_hints": {"vendor": "unknown", "raw": [1, 2, 3]}
            },
            "metadata": {
                "created_at_unix_ms": 1,
                "modified_at_unix_ms": 1,
                "future_metadata": {"revision": 9}
            },
            "future_top_level": {"opaque": true}
        });
        let profile: Profile = serde_json::from_value(source.clone()).unwrap();
        let encoded = serde_json::to_value(profile).unwrap();
        assert_eq!(encoded["future_top_level"], source["future_top_level"]);
        assert_eq!(
            encoded["configuration"]["future_transport_hints"],
            source["configuration"]["future_transport_hints"]
        );
        assert_eq!(
            encoded["metadata"]["future_metadata"],
            source["metadata"]["future_metadata"]
        );
        assert_eq!(
            encoded["configuration"]["options"]["values"]["local.option"],
            true
        );
    }

    #[test]
    fn unknown_fields_at_configuration_section_boundaries_are_retained() {
        let source = serde_json::json!({
            "keymap": {"layers": [], "future_layout": {"logical_slots": 88}},
            "macros": {"items": [], "future_macro_policy": {"steps": 32}},
            "lighting": {"future_color_space": "vendor-space"},
            "options": {"values": {}, "future_option_policy": {"scope": "local"}}
        });
        let config: Configuration = serde_json::from_value(source.clone()).unwrap();
        let encoded = serde_json::to_value(config).unwrap();
        assert_eq!(
            encoded["keymap"]["future_layout"],
            source["keymap"]["future_layout"]
        );
        assert_eq!(
            encoded["macros"]["future_macro_policy"],
            source["macros"]["future_macro_policy"]
        );
        assert_eq!(
            encoded["lighting"]["future_color_space"],
            source["lighting"]["future_color_space"]
        );
        assert_eq!(
            encoded["options"]["future_option_policy"],
            source["options"]["future_option_policy"]
        );
    }
}
