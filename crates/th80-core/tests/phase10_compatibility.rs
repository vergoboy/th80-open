use th80_core::{validation::validate_profile, Profile, PROFILE_SCHEMA_VERSION};

/// Synthetic regression fixture matching the schema-v1 profile format persisted in Phase 10.
#[test]
fn phase10_schema_v1_profile_still_deserializes_validates_and_round_trips() {
    let source = include_str!("fixtures/phase10_profile_v1.json");
    let profile: Profile = serde_json::from_str(source).expect("Phase 10 fixture must deserialize");

    assert_eq!(profile.schema_version, PROFILE_SCHEMA_VERSION);
    assert_eq!(profile.id.0, "phase10-fixture");
    assert!(validate_profile(&profile).is_empty());

    let encoded = serde_json::to_string(&profile).expect("fixture must serialize");
    let decoded: Profile = serde_json::from_str(&encoded).expect("round-trip must deserialize");
    assert_eq!(decoded, profile);
}
