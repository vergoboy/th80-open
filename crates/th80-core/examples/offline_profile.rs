use std::{collections::BTreeMap, fs};
use th80_core::{
    config::{
        lighting::{LightingMode, PerKeyColor, RgbColor},
        macros::{MacroDefinition, MacroStep},
        Action, KeyAssignment, Layer, Profile,
    },
    validation::validate_profile,
    ProfileRepository,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut profile = Profile::new("Example offline profile");
    profile.configuration.keymap.layers = vec![
        Layer {
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
        },
        Layer {
            id: "fn".into(),
            name: "Function".into(),
            positions: BTreeMap::from([(
                "logical-01".into(),
                KeyAssignment {
                    action: Action::LayerSwitch {
                        layer_id: "base".into(),
                    },
                },
            )]),
        },
    ];
    profile.configuration.macros.items.push(MacroDefinition {
        id: "hello".into(),
        name: "Example macro".into(),
        steps: vec![
            MacroStep::KeyDown {
                key: "KEY_A".into(),
            },
            MacroStep::KeyUp {
                key: "KEY_A".into(),
            },
        ],
    });
    profile.configuration.lighting.mode = Some(LightingMode::Static);
    profile.configuration.lighting.brightness = Some(50);
    profile.configuration.lighting.per_key.push(PerKeyColor {
        position: "logical-01".into(),
        color: RgbColor {
            red: 40,
            green: 120,
            blue: 200,
        },
    });
    profile.configuration.options.values.insert(
        "example.local_preference".into(),
        serde_json::json!({"enabled":true}),
    );

    let diagnostics = validate_profile(&profile);
    if !diagnostics.is_empty() {
        return Err(format!("invalid local profile: {diagnostics:?}").into());
    }
    let json = serde_json::to_string_pretty(&profile)?;
    let decoded: Profile = serde_json::from_str(&json)?;
    assert_eq!(profile, decoded);

    let root = std::env::temp_dir().join(format!("th80-core-example-{}", std::process::id()));
    let repository = ProfileRepository::new(&root);
    repository.create(&profile)?;
    let loaded = repository.load(&profile.id)?;
    println!(
        "Saved and loaded local profile '{}' ({}); firmware compatibility remains unverified.",
        loaded.name, loaded.id.0
    );
    fs::remove_dir_all(root)?;
    Ok(())
}
