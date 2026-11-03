use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};
static SEQ: AtomicU64 = AtomicU64::new(0);
fn temp() -> PathBuf {
    std::env::temp_dir().join(format!(
        "th80ctl-it-{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, Ordering::Relaxed)
    ))
}
fn run(root: &PathBuf, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_th80ctl"))
        .args(args)
        .env("TH80_PROFILE_DIR", root)
        .output()
        .unwrap()
}
fn ok(root: &PathBuf, args: &[&str]) {
    let out = run(root, args);
    assert!(
        out.status.success(),
        "args={args:?} stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
}
#[test]
fn full_offline_profile_edit_validate_export_import_delete_workflow() {
    let root = temp();
    let export = root.with_extension("json");
    ok(&root, &["profile", "create", "--name", "Integration"]);
    let listed = run(&root, &["profile", "list", "--output", "json"]);
    assert_eq!(listed.status.code(), Some(0));
    assert!(
        listed.stderr.is_empty(),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&listed.stderr)
    );
    let items: serde_json::Value = serde_json::from_slice(&listed.stdout).unwrap();
    let id = items[0]["id"].as_str().unwrap().to_owned();
    ok(
        &root,
        &[
            "profile",
            "add-layer",
            &id,
            "--layer-id",
            "base",
            "--name",
            "Base",
        ],
    );
    ok(
        &root,
        &[
            "profile",
            "add-layer",
            &id,
            "--layer-id",
            "scratch",
            "--name",
            "Scratch",
        ],
    );
    ok(
        &root,
        &[
            "profile",
            "rename-layer",
            &id,
            "--layer-id",
            "scratch",
            "--name",
            "Temporary",
        ],
    );
    ok(
        &root,
        &["profile", "remove-layer", &id, "--layer-id", "scratch"],
    );
    ok(
        &root,
        &[
            "profile",
            "add-layer",
            &id,
            "--layer-id",
            "fn",
            "--name",
            "Function",
        ],
    );
    ok(
        &root,
        &[
            "profile",
            "set-key",
            &id,
            "--layer",
            "base",
            "--position",
            "logical-01",
            "--key",
            "KEY_A",
        ],
    );
    ok(
        &root,
        &[
            "profile",
            "set-action",
            &id,
            "--layer",
            "base",
            "--position",
            "logical-02",
            "--action",
            r#"{"type":"combination","keys":["KEY_A","KEY_B"]}"#,
        ],
    );
    ok(
        &root,
        &[
            "profile",
            "macro",
            "add",
            &id,
            "--macro-id",
            "tap",
            "--name",
            "Tap",
        ],
    );
    ok(
        &root,
        &[
            "profile",
            "macro",
            "step-add",
            &id,
            "--macro-id",
            "tap",
            "--step",
            "key-down:KEY_A",
            "--step",
            "delay:10",
            "--step",
            "key-up:KEY_A",
        ],
    );
    ok(
        &root,
        &[
            "profile",
            "macro",
            "step-move",
            &id,
            "--macro-id",
            "tap",
            "--from",
            "0",
            "--to",
            "1",
        ],
    );
    ok(
        &root,
        &[
            "profile",
            "macro",
            "step-move",
            &id,
            "--macro-id",
            "tap",
            "--from",
            "1",
            "--to",
            "0",
        ],
    );
    ok(
        &root,
        &[
            "profile",
            "macro",
            "step-replace",
            &id,
            "--macro-id",
            "tap",
            "--index",
            "1",
            "--step",
            "delay:20",
        ],
    );
    ok(
        &root,
        &[
            "profile",
            "macro",
            "step-remove",
            &id,
            "--macro-id",
            "tap",
            "--index",
            "1",
        ],
    );
    ok(
        &root,
        &[
            "profile",
            "macro",
            "rename",
            &id,
            "--macro-id",
            "tap",
            "--name",
            "Tap A",
        ],
    );
    ok(
        &root,
        &["profile", "macro", "show", &id, "--macro-id", "tap"],
    );
    ok(
        &root,
        &[
            "profile",
            "lighting",
            "set",
            &id,
            "--enabled",
            "true",
            "--mode",
            "static",
            "--brightness",
            "45",
            "--speed",
            "20",
            "--color",
            "10,20,30",
        ],
    );
    ok(
        &root,
        &[
            "profile",
            "lighting",
            "key-set",
            &id,
            "--position",
            "logical-01",
            "--color",
            "1,2,3",
        ],
    );
    ok(&root, &["profile", "lighting", "show", &id]);
    ok(
        &root,
        &[
            "profile",
            "lighting",
            "param-set",
            &id,
            "--key",
            "effect.local",
            "--value",
            "{\"amount\":2}",
        ],
    );
    ok(
        &root,
        &[
            "profile",
            "lighting",
            "param-remove",
            &id,
            "--key",
            "effect.local",
        ],
    );
    ok(
        &root,
        &[
            "profile",
            "lighting",
            "key-remove",
            &id,
            "--position",
            "logical-01",
        ],
    );
    ok(
        &root,
        &[
            "profile",
            "options",
            "set",
            &id,
            "--key",
            "local.test",
            "--value",
            "{\"enabled\":true}",
        ],
    );
    ok(&root, &["profile", "options", "list", &id]);
    ok(
        &root,
        &["profile", "options", "remove", &id, "--key", "local.test"],
    );
    let json = run(
        &root,
        &[
            "profile",
            "validate",
            root.join(format!("{id}.json")).to_str().unwrap(),
            "--output",
            "json",
        ],
    );
    assert!(json.status.success());
    let report: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(report["valid"], true);
    assert_eq!(report["device_compatibility"], "not_verified");
    ok(&root, &["profile", "show", &id, "--format", "text"]);
    ok(
        &root,
        &[
            "profile",
            "export",
            &id,
            "--output",
            export.to_str().unwrap(),
        ],
    );
    ok(&root, &["profile", "delete", &id]);
    ok(&root, &["profile", "import", export.to_str().unwrap()]);
    ok(
        &root,
        &["profile", "macro", "remove", &id, "--macro-id", "tap"],
    );
    ok(&root, &["profile", "delete", &id]);
    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_file(export);
}
#[test]
fn malformed_json_has_json_envelope_and_distinct_exit_code() {
    let root = temp();
    fs::create_dir_all(&root).unwrap();
    let bad = root.join("bad.json");
    fs::write(&bad, b"{").unwrap();
    let out = run(
        &root,
        &[
            "profile",
            "validate",
            bad.to_str().unwrap(),
            "--output",
            "json",
        ],
    );
    assert_eq!(out.status.code(), Some(3));
    assert!(
        !out.stderr.is_empty(),
        "input error should be reported on stderr"
    );
    let value: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(value["error_code"], "PROFILE_JSON_INVALID");
    let _ = fs::remove_dir_all(root);
}
#[test]
fn duplicate_profile_create_is_nonzero_and_does_not_replace_original() {
    let root = temp();
    ok(&root, &["profile", "create", "--name", "Original"]);
    let items: serde_json::Value =
        serde_json::from_slice(&run(&root, &["profile", "list", "--output", "json"]).stdout)
            .unwrap();
    let id = items[0]["id"].as_str().unwrap();
    let path = root.join(format!("{id}.json"));
    let before = fs::read(&path).unwrap();
    // Importing a copy with the same identifier must fail and preserve the original.
    let copy = root.with_extension("copy.json");
    fs::write(&copy, &before).unwrap();
    let out = run(&root, &["profile", "import", copy.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(4));
    assert_eq!(fs::read(path).unwrap(), before);
    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_file(copy);
}

#[test]
fn profile_rename_and_duplicate_preserve_data_without_reusing_identity() {
    let root = temp();
    ok(
        &root,
        &[
            "profile",
            "create",
            "--name",
            "Original",
            "--description",
            "Portable",
        ],
    );
    let listed: serde_json::Value =
        serde_json::from_slice(&run(&root, &["profile", "list", "--output", "json"]).stdout)
            .unwrap();
    let original_id = listed[0]["id"].as_str().unwrap().to_owned();
    let original_path = root.join(format!("{original_id}.json"));
    let mut original_json: serde_json::Value =
        serde_json::from_slice(&fs::read(&original_path).unwrap()).unwrap();
    original_json["future_top_level_extension"] = serde_json::json!({"retained": [1, 2, 3]});
    original_json["configuration"]["future_section"] =
        serde_json::json!({"opaque": ["keep", "all"]});
    original_json["metadata"]["future_metadata"] = serde_json::json!({"revision": 4});
    original_json["configuration"]["keymap"]["future_layout"] =
        serde_json::json!({"logical_slots": 88});
    original_json["configuration"]["macros"]["future_macro_format"] =
        serde_json::json!({"version": 9});
    original_json["configuration"]["lighting"]["future_color_space"] =
        serde_json::json!("vendor-space");
    original_json["configuration"]["options"]["future_option_schema"] =
        serde_json::json!({"opaque": true});
    fs::write(
        &original_path,
        serde_json::to_vec_pretty(&original_json).unwrap(),
    )
    .unwrap();

    ok(
        &root,
        &["profile", "rename", &original_id, "--name", "Renamed"],
    );
    let renamed_json: serde_json::Value =
        serde_json::from_slice(&fs::read(&original_path).unwrap()).unwrap();
    assert_eq!(
        renamed_json["configuration"]["future_section"],
        serde_json::json!({"opaque": ["keep", "all"]})
    );
    assert_eq!(
        renamed_json["metadata"]["future_metadata"],
        serde_json::json!({"revision": 4})
    );
    ok(&root, &["profile", "duplicate", &original_id]);
    let listed: serde_json::Value =
        serde_json::from_slice(&run(&root, &["profile", "list", "--output", "json"]).stdout)
            .unwrap();
    assert_eq!(listed.as_array().unwrap().len(), 2);
    let renamed = listed
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == original_id)
        .unwrap();
    let duplicate = listed
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] != original_id)
        .unwrap();
    assert_eq!(renamed["name"], "Renamed");
    assert_eq!(duplicate["name"], "Renamed copy");
    assert_eq!(duplicate["description"], "Portable");
    let duplicate_id = duplicate["id"].as_str().unwrap();
    let duplicate_json: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join(format!("{duplicate_id}.json"))).unwrap())
            .unwrap();
    assert_eq!(
        duplicate_json["future_top_level_extension"],
        serde_json::json!({"retained": [1, 2, 3]})
    );
    assert_eq!(
        duplicate_json["configuration"]["future_section"],
        serde_json::json!({"opaque": ["keep", "all"]})
    );
    assert_eq!(
        duplicate_json["metadata"]["future_metadata"],
        serde_json::json!({"revision": 4})
    );
    assert_eq!(
        duplicate_json["configuration"]["keymap"]["future_layout"],
        serde_json::json!({"logical_slots": 88})
    );
    assert_eq!(
        duplicate_json["configuration"]["macros"]["future_macro_format"],
        serde_json::json!({"version": 9})
    );
    assert_eq!(
        duplicate_json["configuration"]["lighting"]["future_color_space"],
        serde_json::json!("vendor-space")
    );
    assert_eq!(
        duplicate_json["configuration"]["options"]["future_option_schema"],
        serde_json::json!({"opaque": true})
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn validation_failure_json_contains_structured_domain_and_stable_code() {
    let root = temp();
    fs::create_dir_all(&root).unwrap();
    let invalid = root.join("invalid.json");
    fs::write(&invalid, br#"{"schema_version":1,"id":"invalid","name":"","configuration":{},"metadata":{"created_at_unix_ms":1,"modified_at_unix_ms":1},"provenance":"local_draft"}"#).unwrap();
    let out = run(
        &root,
        &[
            "profile",
            "validate",
            invalid.to_str().unwrap(),
            "--output",
            "json",
        ],
    );
    assert_eq!(out.status.code(), Some(2));
    assert!(
        !out.stderr.is_empty(),
        "validation summary should be reported on stderr"
    );
    let value: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(value["valid"], false);
    assert_eq!(value["device_compatibility"], "not_verified");
    let diagnostics = value["diagnostics"].as_array().unwrap();
    assert!(diagnostics.iter().any(|d| d["code"] == "required"
        && d["domain"] == "application_validation"
        && d["path"] == "name"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn cli_usage_has_a_distinct_exit_code_and_help_needs_no_repository() {
    let root = temp();
    let help = run(&root, &["--help"]);
    assert_eq!(help.status.code(), Some(0));
    assert!(help.stderr.is_empty());
    assert!(!root.exists());
    let usage = run(&root, &["profile", "not-a-command"]);
    assert_eq!(usage.status.code(), Some(64));
    assert!(!usage.stderr.is_empty());
    let _ = fs::remove_dir_all(root);
}
