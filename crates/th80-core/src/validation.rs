use crate::{
    capability::offline_capabilities,
    config::*,
    error::{Diagnostic, ValidationIssue},
};
use std::collections::{BTreeMap, BTreeSet};

/// Explicit application-side resource policy; these are not firmware limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApplicationPolicy {
    pub max_macro_steps: usize,
    pub max_macro_delay_ms: u32,
    pub normalized_lighting_max: u8,
}

/// Current local safety limits. No device or firmware specification is implied.
pub const APPLICATION_POLICY: ApplicationPolicy = ApplicationPolicy {
    max_macro_steps: 10_000,
    max_macro_delay_ms: 600_000,
    normalized_lighting_max: 100,
};

/// Offline compatibility assessment; no device is contacted.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct DeviceCompatibilityReport {
    pub status: DeviceCompatibility,
    pub explanation: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceCompatibility {
    ModelMismatch,
    NotVerified,
}

/// Application validation and device compatibility are intentionally separate.
/// This function never communicates with a device or asserts firmware support.
/// Reports model mismatch or `NotVerified`; local validation never proves firmware acceptance.
pub fn assess_device_compatibility(
    profile: &Profile,
    device: crate::DeviceKind,
) -> DeviceCompatibilityReport {
    if profile.device_model.is_some_and(|model| model != device) {
        DeviceCompatibilityReport {
            status: DeviceCompatibility::ModelMismatch,
            explanation: "profile is associated with a different device model".into(),
        }
    } else {
        DeviceCompatibilityReport {
            status: DeviceCompatibility::NotVerified,
            explanation: "application-level validity does not establish firmware support, device ranges, or protocol compatibility".into(),
        }
    }
}

/// Validates application schema and local policy only.
pub fn validate_profile(profile: &Profile) -> Vec<ValidationIssue> {
    let mut out = Vec::new();
    let mut add = |code: &'static str, path: String, message: String| {
        out.push(Diagnostic::application(
            code,
            Some(&profile.id.0),
            path,
            message,
        ));
    };
    if profile.schema_version != crate::PROFILE_SCHEMA_VERSION {
        add(
            "unsupported_schema",
            "schema_version".into(),
            format!("supported version is {}", crate::PROFILE_SCHEMA_VERSION),
        );
    }
    if profile.id.0.trim().is_empty()
        || profile.id.0.len() > 128
        || !profile
            .id
            .0
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_".contains(c))
    {
        add(
            "invalid_profile_id",
            "id".into(),
            "use 1-128 ASCII letters, digits, '-' or '_'".into(),
        );
    }
    if profile.name.trim().is_empty() {
        add(
            "required",
            "name".into(),
            "profile name must not be empty".into(),
        );
    }
    if profile.metadata.modified_at_unix_ms < profile.metadata.created_at_unix_ms {
        add(
            "invalid_timestamp",
            "metadata.modified_at_unix_ms".into(),
            "must not precede creation time".into(),
        );
    }
    if matches!(
        profile.provenance,
        ConfigurationProvenance::DeviceApplied | ConfigurationProvenance::DeviceReported
    ) {
        add(
            "unsupported_provenance",
            "provenance".into(),
            "Phase 10 cannot claim device-reported or device-applied state".into(),
        );
    }
    let layers: BTreeSet<_> = profile
        .configuration
        .keymap
        .layers
        .iter()
        .map(|l| l.id.as_str())
        .collect();
    if layers.len() != profile.configuration.keymap.layers.len() {
        add(
            "duplicate_layer_id",
            "configuration.keymap.layers".into(),
            "layer identifiers must be unique".into(),
        );
    }
    for (li, layer) in profile.configuration.keymap.layers.iter().enumerate() {
        if layer.id.trim().is_empty() {
            add(
                "invalid_layer_id",
                format!("configuration.keymap.layers[{li}].id"),
                "must not be empty".into(),
            );
        }
        if layer.name.trim().is_empty() {
            add(
                "invalid_layer_name",
                format!("configuration.keymap.layers[{li}].name"),
                "must not be empty".into(),
            );
        }
        for (position, assignment) in &layer.positions {
            if position.trim().is_empty() {
                add(
                    "invalid_position",
                    format!("configuration.keymap.layers[{li}]"),
                    "logical key position must not be empty".into(),
                );
            }
            match &assignment.action {
                Action::Key { code } | Action::Modifier { modifier: code }
                    if code.trim().is_empty() =>
                {
                    add(
                        "invalid_action",
                        format!("layers[{}].positions[{position}]", layer.id),
                        "key/modifier value must not be empty".into(),
                    )
                }
                Action::Combination { keys }
                    if keys.is_empty() || keys.iter().any(|k| k.trim().is_empty()) =>
                {
                    add(
                        "invalid_combination",
                        format!("layers[{}].positions[{position}]", layer.id),
                        "combination requires non-empty key names".into(),
                    )
                }
                Action::LayerSwitch { layer_id } | Action::LayerMomentary { layer_id }
                    if !layers.contains(layer_id.as_str()) =>
                {
                    add(
                        "invalid_layer_reference",
                        format!("layers[{}].positions[{position}]", layer.id),
                        format!("referenced layer '{layer_id}' does not exist"),
                    )
                }
                Action::Macro { macro_id }
                    if !profile
                        .configuration
                        .macros
                        .items
                        .iter()
                        .any(|m| m.id == *macro_id) =>
                {
                    add(
                        "invalid_macro_reference",
                        format!("layers[{}].positions[{position}]", layer.id),
                        format!("referenced macro '{macro_id}' does not exist"),
                    )
                }
                _ => {}
            }
        }
    }
    let macro_ids: BTreeSet<_> = profile
        .configuration
        .macros
        .items
        .iter()
        .map(|m| m.id.as_str())
        .collect();
    if macro_ids.len() != profile.configuration.macros.items.len() {
        add(
            "duplicate_macro_id",
            "configuration.macros".into(),
            "macro identifiers must be unique".into(),
        );
    }
    for (mi, item) in profile.configuration.macros.items.iter().enumerate() {
        let path = format!("configuration.macros.items[{mi}]");
        if item.id.trim().is_empty() {
            add(
                "invalid_macro_id",
                format!("{path}.id"),
                "must not be empty".into(),
            );
        }
        if item.name.trim().is_empty() {
            add(
                "invalid_macro_name",
                format!("{path}.name"),
                "must not be empty".into(),
            );
        }
        if item.steps.len() > APPLICATION_POLICY.max_macro_steps {
            add(
                "macro_too_large",
                format!("{path}.steps"),
                format!(
                    "local safety limit is {} steps",
                    APPLICATION_POLICY.max_macro_steps
                ),
            );
        }
        let mut down = BTreeSet::new();
        for (si, step) in item.steps.iter().enumerate() {
            match step {
                MacroStep::KeyDown { key } => {
                    if key.trim().is_empty() || !down.insert(key) {
                        add(
                            "invalid_key_down",
                            format!("{path}.steps[{si}]"),
                            "key must be named and not already held".into(),
                        );
                    }
                }
                MacroStep::KeyUp { key } => {
                    if !down.remove(key) {
                        add(
                            "invalid_key_up",
                            format!("{path}.steps[{si}]"),
                            "key-up has no matching preceding key-down".into(),
                        );
                    }
                }
                MacroStep::Delay { milliseconds }
                    if *milliseconds > APPLICATION_POLICY.max_macro_delay_ms =>
                {
                    add(
                        "delay_too_long",
                        format!("{path}.steps[{si}]"),
                        format!(
                            "local safety limit is {} ms",
                            APPLICATION_POLICY.max_macro_delay_ms
                        ),
                    )
                }
                MacroStep::Text { value } if value.is_empty() => add(
                    "empty_macro_text",
                    format!("{path}.steps[{si}]"),
                    "text step must not be empty".into(),
                ),
                _ => {}
            }
        }
        if !down.is_empty() {
            add(
                "unreleased_keys",
                format!("{path}.steps"),
                format!(
                    "keys remain held: {}",
                    down.into_iter().cloned().collect::<Vec<_>>().join(", ")
                ),
            );
        }
    }
    if profile
        .configuration
        .lighting
        .brightness
        .is_some_and(|v| v > APPLICATION_POLICY.normalized_lighting_max)
    {
        add(
            "brightness_out_of_range",
            "configuration.lighting.brightness".into(),
            "application-normalized range is 0..=100; not a firmware range".into(),
        );
    }
    if profile
        .configuration
        .lighting
        .speed
        .is_some_and(|v| v > APPLICATION_POLICY.normalized_lighting_max)
    {
        add(
            "speed_out_of_range",
            "configuration.lighting.speed".into(),
            "application-normalized range is 0..=100; not a firmware range".into(),
        );
    }
    let mut positions = BTreeMap::new();
    for (i, entry) in profile.configuration.lighting.per_key.iter().enumerate() {
        if entry.position.trim().is_empty() {
            add(
                "invalid_lighting_position",
                format!("configuration.lighting.per_key[{i}]"),
                "logical position must not be empty".into(),
            );
        }
        if positions.insert(&entry.position, i).is_some() {
            add(
                "duplicate_per_key_color",
                format!("configuration.lighting.per_key[{i}]"),
                format!(
                    "duplicate assignment for logical position '{}'",
                    entry.position
                ),
            );
        }
    }
    for key in profile.configuration.lighting.effect_parameters.keys() {
        if key.trim().is_empty() {
            add(
                "invalid_effect_parameter",
                "configuration.lighting.effect_parameters".into(),
                "parameter names must not be empty".into(),
            );
        }
    }
    for capability in offline_capabilities() {
        if capability.device_communication_approved {
            add(
                "unsafe_capability",
                capability.feature_id,
                "device communication must remain disabled".into(),
            );
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn local_resource_limits_are_application_policy_not_firmware_evidence() {
        let mut profile = Profile::new("policy");
        profile.configuration.macros.items.push(MacroDefinition {
            id: "large".into(),
            name: "large".into(),
            steps: vec![MacroStep::Delay {
                milliseconds: APPLICATION_POLICY.max_macro_delay_ms + 1,
            }],
        });
        let diagnostics = validate_profile(&profile);
        let limit = diagnostics
            .iter()
            .find(|d| d.code == "delay_too_long")
            .unwrap();
        assert_eq!(
            limit.domain,
            crate::error::DiagnosticDomain::ApplicationValidation
        );
        assert_eq!(limit.severity, crate::error::DiagnosticSeverity::Error);
        assert!(limit.message.contains("local safety limit"));
        let evidence = offline_capabilities();
        assert!(evidence
            .iter()
            .all(|cap| cap.evidence == crate::capability::EvidenceClass::Unknown));
        assert!(evidence
            .iter()
            .all(|cap| !cap.device_communication_approved));
    }

    #[test]
    fn generic_editable_features_do_not_upgrade_device_evidence() {
        let profile = Profile::new("generic");
        assert!(profile.configuration.lighting.mode.is_none());
        let evidence = offline_capabilities();
        assert!(evidence
            .iter()
            .filter(
                |cap| ["lighting", "macros", "keyboard_options"].contains(&cap.feature_id.as_str())
            )
            .all(|cap| cap.evidence == crate::capability::EvidenceClass::Unknown));
        assert_eq!(
            assess_device_compatibility(&profile, crate::DeviceKind::Th80Pro).status,
            DeviceCompatibility::NotVerified
        );
    }

    #[test]
    fn validation_diagnostics_have_stable_codes_and_structured_domain() {
        let mut profile = Profile::new("diagnostics");
        profile.name.clear();
        let diagnostics = validate_profile(&profile);
        let required = diagnostics.iter().find(|d| d.code == "required").unwrap();
        assert_eq!(
            required.domain,
            crate::error::DiagnosticDomain::ApplicationValidation
        );
        assert_eq!(required.profile_id.as_deref(), Some(profile.id.0.as_str()));
        assert_eq!(required.path.as_deref(), Some("name"));
        assert_eq!(required.code, "required");
    }

    #[test]
    fn validates_layer_macro_references_and_balanced_macro_keys() {
        let mut p = Profile::new("test");
        p.configuration.keymap.layers.push(Layer {
            id: "base".into(),
            name: "Base".into(),
            positions: BTreeMap::from([(
                "logical-1".into(),
                KeyAssignment {
                    action: Action::LayerSwitch {
                        layer_id: "missing".into(),
                    },
                },
            )]),
        });
        p.configuration.macros.items.push(MacroDefinition {
            id: "m1".into(),
            name: "bad".into(),
            steps: vec![MacroStep::KeyUp { key: "A".into() }],
        });
        let issues = validate_profile(&p);
        assert!(issues.iter().any(|e| e.code == "invalid_layer_reference"));
        assert!(issues.iter().any(|e| e.code == "invalid_key_up"));
    }

    #[test]
    fn requires_non_empty_layer_and_macro_display_names() {
        let mut profile = Profile::new("display names");
        profile.configuration.keymap.layers.push(Layer {
            id: "base".into(),
            name: " ".into(),
            positions: BTreeMap::new(),
        });
        profile.configuration.macros.items.push(MacroDefinition {
            id: "macro".into(),
            name: String::new(),
            steps: Vec::new(),
        });
        let diagnostics = validate_profile(&profile);
        assert!(diagnostics
            .iter()
            .any(|item| item.code == "invalid_layer_name"));
        assert!(diagnostics
            .iter()
            .any(|item| item.code == "invalid_macro_name"));
    }
    #[test]
    fn rejects_device_applied_provenance() {
        let mut p = Profile::new("x");
        p.provenance = ConfigurationProvenance::DeviceApplied;
        assert!(validate_profile(&p)
            .iter()
            .any(|e| e.code == "unsupported_provenance"));
    }

    #[test]
    fn device_compatibility_is_not_claimed_by_application_validation() {
        let profile = Profile::new("local");
        let report = assess_device_compatibility(&profile, crate::DeviceKind::Th80Pro);
        assert_eq!(report.status, DeviceCompatibility::NotVerified);
        assert!(report
            .explanation
            .contains("does not establish firmware support"));
        let mismatch = assess_device_compatibility(&profile, crate::DeviceKind::RoyuanReceiver);
        assert_eq!(mismatch.status, DeviceCompatibility::ModelMismatch);
    }

    #[test]
    fn rejects_duplicate_layers_and_macros_and_invalid_macro_references() {
        let mut p = Profile::new("duplicates");
        let layer = Layer {
            id: "base".into(),
            name: "Base".into(),
            positions: BTreeMap::from([(
                "P1".into(),
                KeyAssignment {
                    action: Action::Macro {
                        macro_id: "missing".into(),
                    },
                },
            )]),
        };
        p.configuration.keymap.layers = vec![layer.clone(), layer];
        p.configuration.macros.items = vec![
            MacroDefinition {
                id: "dup".into(),
                name: "one".into(),
                steps: vec![],
            },
            MacroDefinition {
                id: "dup".into(),
                name: "two".into(),
                steps: vec![],
            },
        ];
        let issues = validate_profile(&p);
        for code in [
            "duplicate_layer_id",
            "duplicate_macro_id",
            "invalid_macro_reference",
        ] {
            assert!(issues.iter().any(|e| e.code == code), "missing {code}");
        }
    }

    #[test]
    fn validates_macro_resource_and_lighting_bounds_and_duplicates() {
        let mut p = Profile::new("limits");
        p.configuration.macros.items.push(MacroDefinition {
            id: "m".into(),
            name: "long delay".into(),
            steps: vec![MacroStep::Delay {
                milliseconds: APPLICATION_POLICY.max_macro_delay_ms + 1,
            }],
        });
        p.configuration.lighting.brightness = Some(101);
        p.configuration.lighting.speed = Some(101);
        let color = crate::config::RgbColor {
            red: 255,
            green: 0,
            blue: 127,
        };
        p.configuration.lighting.per_key = vec![
            PerKeyColor {
                position: "P1".into(),
                color,
            },
            PerKeyColor {
                position: "P1".into(),
                color,
            },
        ];
        let issues = validate_profile(&p);
        for code in [
            "delay_too_long",
            "brightness_out_of_range",
            "speed_out_of_range",
            "duplicate_per_key_color",
        ] {
            assert!(issues.iter().any(|e| e.code == code), "missing {code}");
        }
    }
}
