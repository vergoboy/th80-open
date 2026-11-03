use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command},
    thread,
    time::{Duration, Instant},
};
use th80_core::{Profile, ProfileId, ProfileRepository};

fn temp_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "th80-process-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn wait_for(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(15);
    while !path.exists() {
        assert!(
            Instant::now() < deadline,
            "timed out waiting for {}",
            path.display()
        );
        thread::sleep(Duration::from_millis(10));
    }
}

fn spawn(
    mode: &str,
    root: &Path,
    id: &str,
    ready: &Path,
    gate: &Path,
    result: &Path,
    extra: &str,
) -> Child {
    Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "child_worker_entrypoint", "--nocapture"])
        .env("TH80_CHILD_MODE", mode)
        .env("TH80_CHILD_ROOT", root)
        .env("TH80_CHILD_ID", id)
        .env("TH80_CHILD_READY", ready)
        .env("TH80_CHILD_GATE", gate)
        .env("TH80_CHILD_RESULT", result)
        .env("TH80_CHILD_EXTRA", extra)
        .spawn()
        .unwrap()
}

fn release_and_wait(children: &mut [Child], gate: &Path) {
    fs::write(gate, b"go").unwrap();
    for child in children {
        let status = child.wait().unwrap();
        assert!(status.success(), "child exited with {status}");
    }
}

#[test]
fn child_worker_entrypoint() {
    let Ok(mode) = std::env::var("TH80_CHILD_MODE") else {
        return;
    };
    let root = PathBuf::from(std::env::var_os("TH80_CHILD_ROOT").unwrap());
    let id = std::env::var("TH80_CHILD_ID").unwrap();
    let ready = PathBuf::from(std::env::var_os("TH80_CHILD_READY").unwrap());
    let gate = PathBuf::from(std::env::var_os("TH80_CHILD_GATE").unwrap());
    let result = PathBuf::from(std::env::var_os("TH80_CHILD_RESULT").unwrap());
    let extra = std::env::var("TH80_CHILD_EXTRA").unwrap_or_default();
    let repo = ProfileRepository::new(root.clone());
    match mode.as_str() {
        "update" => {
            let mut profile = repo.load(&ProfileId::new(&id)).unwrap();
            fs::write(&ready, b"ready").unwrap();
            wait_for(&gate);
            profile.name = extra;
            let outcome = match repo.save(&profile) {
                Ok(()) => "ok",
                Err(th80_core::CoreError::Busy(_)) => "busy",
                Err(th80_core::CoreError::Conflict(_)) => "conflict",
                Err(error) => panic!("unexpected save error: {error}"),
            };
            fs::write(result, outcome).unwrap();
        }
        "create" => {
            fs::write(&ready, b"ready").unwrap();
            wait_for(&gate);
            let mut profile = Profile::new(extra);
            profile.id = ProfileId::new(id);
            let outcome = match repo.create(&profile) {
                Ok(()) => "ok",
                Err(th80_core::CoreError::Busy(_)) => "busy",
                Err(th80_core::CoreError::AlreadyExists(_)) => "exists",
                Err(error) => panic!("unexpected create error: {error}"),
            };
            fs::write(result, outcome).unwrap();
        }
        "import" => {
            fs::write(&ready, b"ready").unwrap();
            wait_for(&gate);
            let outcome = match repo.import(&extra) {
                Ok(_) => "ok",
                Err(th80_core::CoreError::Busy(_)) => "busy",
                Err(th80_core::CoreError::AlreadyExists(_)) => "exists",
                Err(error) => panic!("unexpected import error: {error}"),
            };
            fs::write(result, outcome).unwrap();
        }
        "read" => {
            fs::write(&ready, b"ready").unwrap();
            while !gate.exists() {
                let profile = repo.load(&ProfileId::new(&id)).unwrap();
                assert!(profile.name.starts_with("version-"));
                thread::yield_now();
            }
            let _ = repo.load(&ProfileId::new(&id)).unwrap();
            fs::write(result, b"ok").unwrap();
        }
        "orphan-lock" => {
            fs::create_dir_all(&root).unwrap();
            let lock = root.join(format!(".{id}.lock"));
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(lock)
                .unwrap();
            use std::io::Write as _;
            writeln!(file, "pid={}", std::process::id()).unwrap();
            file.sync_all().unwrap();
            fs::write(&ready, b"locked").unwrap();
            // Deliberately simulate abrupt termination: no Rust Drop cleanup runs.
            std::process::exit(0);
        }
        _ => panic!("unknown child mode {mode}"),
    }
}

#[test]
fn separate_processes_conflict_on_same_profile_and_allow_different_profiles() {
    let root = temp_root("writers");
    let repo = ProfileRepository::new(&root);
    let profile = Profile::new("version-0");
    repo.create(&profile).unwrap();

    let gate = root.join("same-go");
    let mut children = Vec::new();
    for n in 1..=2 {
        let base = root.join(format!("same-{n}"));
        children.push(spawn(
            "update",
            &root,
            &profile.id.0,
            &base.with_extension("ready"),
            &gate,
            &base.with_extension("result"),
            &format!("version-{n}"),
        ));
    }
    for n in 1..=2 {
        wait_for(&root.join(format!("same-{n}.ready")));
    }
    release_and_wait(&mut children, &gate);
    let outcomes: Vec<_> = (1..=2)
        .map(|n| fs::read_to_string(root.join(format!("same-{n}.result"))).unwrap())
        .collect();
    assert_eq!(
        outcomes.iter().filter(|s| s.as_str() == "ok").count(),
        1,
        "{outcomes:?}"
    );
    assert_eq!(
        outcomes
            .iter()
            .filter(|s| s.as_str() == "busy" || s.as_str() == "conflict")
            .count(),
        1,
        "{outcomes:?}"
    );

    let a = Profile::new("version-a");
    let b = Profile::new("version-b");
    repo.create(&a).unwrap();
    repo.create(&b).unwrap();
    let gate = root.join("different-go");
    let mut children = Vec::new();
    for (label, p) in [("a", &a), ("b", &b)] {
        let base = root.join(format!("different-{label}"));
        children.push(spawn(
            "update",
            &root,
            &p.id.0,
            &base.with_extension("ready"),
            &gate,
            &base.with_extension("result"),
            &format!("version-{label}-updated"),
        ));
    }
    wait_for(&root.join("different-a.ready"));
    wait_for(&root.join("different-b.ready"));
    release_and_wait(&mut children, &gate);
    assert_eq!(
        fs::read_to_string(root.join("different-a.result")).unwrap(),
        "ok"
    );
    assert_eq!(
        fs::read_to_string(root.join("different-b.result")).unwrap(),
        "ok"
    );
    assert!(!root.join(format!(".{}.lock", a.id.0)).exists());
    assert!(!root.join(format!(".{}.lock", b.id.0)).exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn later_cross_process_writer_cannot_overwrite_an_earlier_committed_revision() {
    let root = temp_root("stale-writer");
    let repo = ProfileRepository::new(&root);
    let profile = Profile::new("version-0");
    repo.create(&profile).unwrap();
    let gate_a = root.join("writer-a.go");
    let gate_b = root.join("writer-b.go");
    let ready_a = root.join("writer-a.ready");
    let ready_b = root.join("writer-b.ready");
    let result_a = root.join("writer-a.result");
    let result_b = root.join("writer-b.result");
    let mut a = spawn(
        "update",
        &root,
        &profile.id.0,
        &ready_a,
        &gate_a,
        &result_a,
        "version-first",
    );
    let mut b = spawn(
        "update",
        &root,
        &profile.id.0,
        &ready_b,
        &gate_b,
        &result_b,
        "version-stale",
    );
    wait_for(&ready_a);
    wait_for(&ready_b);
    fs::write(&gate_a, b"go").unwrap();
    assert!(a.wait().unwrap().success());
    assert_eq!(fs::read_to_string(&result_a).unwrap(), "ok");
    fs::write(&gate_b, b"go").unwrap();
    assert!(b.wait().unwrap().success());
    assert_eq!(fs::read_to_string(&result_b).unwrap(), "conflict");
    assert_eq!(repo.load(&profile.id).unwrap().name, "version-first");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn separate_processes_race_create_and_import_without_overwriting() {
    let root = temp_root("create-import");
    fs::create_dir_all(&root).unwrap();
    let id = "same-profile-id";
    let gate = root.join("create-go");
    let mut children = Vec::new();
    for n in 1..=2 {
        let base = root.join(format!("create-{n}"));
        children.push(spawn(
            "create",
            &root,
            id,
            &base.with_extension("ready"),
            &gate,
            &base.with_extension("result"),
            &format!("created-{n}"),
        ));
    }
    wait_for(&root.join("create-1.ready"));
    wait_for(&root.join("create-2.ready"));
    release_and_wait(&mut children, &gate);
    let outcomes: Vec<_> = (1..=2)
        .map(|n| fs::read_to_string(root.join(format!("create-{n}.result"))).unwrap())
        .collect();
    assert_eq!(
        outcomes.iter().filter(|s| s.as_str() == "ok").count(),
        1,
        "{outcomes:?}"
    );
    assert!(outcomes
        .iter()
        .all(|s| s == "ok" || s == "busy" || s == "exists"));

    let import_source = root.join("import-source.json");
    let mut imported = Profile::new("import source");
    imported.id = ProfileId::new("import-race-id");
    fs::write(&import_source, serde_json::to_vec(&imported).unwrap()).unwrap();
    let gate = root.join("import-go");
    let mut children = Vec::new();
    for n in 1..=2 {
        let base = root.join(format!("import-{n}"));
        children.push(spawn(
            "import",
            &root,
            &imported.id.0,
            &base.with_extension("ready"),
            &gate,
            &base.with_extension("result"),
            import_source.to_str().unwrap(),
        ));
    }
    wait_for(&root.join("import-1.ready"));
    wait_for(&root.join("import-2.ready"));
    release_and_wait(&mut children, &gate);
    let outcomes: Vec<_> = (1..=2)
        .map(|n| fs::read_to_string(root.join(format!("import-{n}.result"))).unwrap())
        .collect();
    assert_eq!(
        outcomes.iter().filter(|s| s.as_str() == "ok").count(),
        1,
        "{outcomes:?}"
    );
    assert!(outcomes
        .iter()
        .all(|s| s == "ok" || s == "busy" || s == "exists"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn reader_process_observes_only_complete_replacements_and_orphan_lock_fails_closed() {
    let root = temp_root("reader-crash");
    let repo = ProfileRepository::new(&root);
    let mut profile = Profile::new("version-0");
    profile.id = ProfileId::new("reader-profile");
    repo.create(&profile).unwrap();
    let ready = root.join("reader.ready");
    let result = root.join("reader.result");
    let gate = root.join("reader.stop");
    let mut reader = spawn("read", &root, &profile.id.0, &ready, &gate, &result, "");
    wait_for(&ready);
    for n in 1..=20 {
        let mut p = repo.load(&profile.id).unwrap();
        p.name = format!("version-{n}");
        repo.save(&p).unwrap();
    }
    fs::write(&gate, b"stop").unwrap();
    assert!(reader.wait().unwrap().success());
    assert_eq!(fs::read_to_string(result).unwrap(), "ok");

    let lock_ready = root.join("orphan.ready");
    let lock_result = root.join("orphan.result");
    let mut orphan = spawn(
        "orphan-lock",
        &root,
        &profile.id.0,
        &lock_ready,
        &root.join("unused"),
        &lock_result,
        "",
    );
    wait_for(&lock_ready);
    assert!(orphan.wait().unwrap().success());
    let current = repo.load(&profile.id).unwrap();
    assert!(matches!(
        repo.save(&current),
        Err(th80_core::CoreError::Busy(_))
    ));
    let lock_contents = fs::read_to_string(root.join(format!(".{}.lock", profile.id.0))).unwrap();
    assert!(lock_contents.starts_with("pid="));
    // Explicit operator recovery after confirming the owner process has exited.
    fs::remove_file(root.join(format!(".{}.lock", profile.id.0))).unwrap();
    repo.save(&current).unwrap();
    let _ = fs::remove_dir_all(root);
}
