use crate::{
    config::{ConfigurationProvenance, Profile, ProfileId},
    error::CoreError,
    validation::validate_profile,
    PROFILE_SCHEMA_VERSION,
};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Local JSON profile repository. Mutations use cross-process exclusive lock files.
#[derive(Debug, Clone)]
pub struct ProfileRepository {
    root: PathBuf,
}

impl ProfileRepository {
    /// Creates a repository handle. The directory is created by the first mutation.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
    /// Returns the configured storage directory.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Creates a profile without replacing an existing profile with the same ID.
    /// Creates a profile without replacing an existing profile.
    pub fn create(&self, profile: &Profile) -> Result<(), CoreError> {
        let path = self.path_for(&profile.id)?;
        self.validate(profile)?;
        fs::create_dir_all(&self.root)?;
        let _lock = self.lock(&profile.id)?;
        let mut persisted = profile.clone();
        if matches!(
            persisted.provenance,
            ConfigurationProvenance::LocalDraft | ConfigurationProvenance::ValidatedLocally
        ) {
            persisted.provenance = ConfigurationProvenance::SerializedToDisk;
        }
        let bytes = serde_json::to_vec_pretty(&persisted)?;
        self.write_new(&path, &bytes)
    }

    /// Saves a profile only if its stored modification timestamp still matches the loaded copy.
    /// A stale copy is rejected instead of silently overwriting another writer's update.
    /// Saves a loaded profile if its stored revision has not changed.
    pub fn save(&self, profile: &Profile) -> Result<(), CoreError> {
        let path = self.path_for(&profile.id)?;
        self.validate(profile)?;
        let _lock = self.lock(&profile.id)?;
        let current: Profile = serde_json::from_slice(&fs::read(&path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                CoreError::NotFound(profile.id.0.clone())
            } else {
                CoreError::Io(e)
            }
        })?)?;
        if current.id != profile.id
            || current.metadata.modified_at_unix_ms != profile.metadata.modified_at_unix_ms
        {
            return Err(CoreError::Conflict(profile.id.0.clone()));
        }
        let mut persisted = profile.clone();
        persisted.metadata.modified_at_unix_ms =
            now_unix_ms().max(current.metadata.modified_at_unix_ms.saturating_add(1));
        if matches!(
            persisted.provenance,
            ConfigurationProvenance::LocalDraft | ConfigurationProvenance::ValidatedLocally
        ) {
            persisted.provenance = ConfigurationProvenance::SerializedToDisk;
        }
        self.atomic_replace(&path, &serde_json::to_vec_pretty(&persisted)?)
    }

    /// Loads and application-validates a profile. Readers see the old or new complete file
    /// when the platform's replacement primitive provides atomic visibility.
    pub fn load(&self, id: &ProfileId) -> Result<Profile, CoreError> {
        let path = self.path_for(id)?;
        let bytes = fs::read(&path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                CoreError::NotFound(id.0.clone())
            } else {
                CoreError::Io(e)
            }
        })?;
        let profile: Profile = serde_json::from_slice(&bytes)?;
        self.validate(&profile)?;
        if profile.id != *id {
            return Err(CoreError::InvalidId(
                "stored identifier does not match requested identifier".into(),
            ));
        }
        Ok(profile)
    }

    /// Lists valid JSON profiles. Hidden temporary and lock files are ignored.
    /// Lists all valid non-hidden profile JSON files.
    pub fn list(&self) -> Result<Vec<Profile>, CoreError> {
        if !self.root.exists() {
            return Ok(Vec::new());
        }
        let mut profiles = Vec::new();
        for entry in fs::read_dir(&self.root)? {
            let path = entry?.path();
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with('.'))
                || path.extension().and_then(|v| v.to_str()) != Some("json")
            {
                continue;
            }
            let profile: Profile = serde_json::from_slice(&fs::read(&path)?)?;
            self.validate(&profile)?;
            if self.path_for(&profile.id)? != path {
                return Err(CoreError::InvalidId(format!(
                    "filename does not match profile id: {}",
                    path.display()
                )));
            }
            profiles.push(profile);
        }
        profiles.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.id.cmp(&b.id)));
        Ok(profiles)
    }

    /// Deletes a profile while holding its cross-process mutation lock.
    /// Deletes a profile under its mutation lock.
    pub fn delete(&self, id: &ProfileId) -> Result<(), CoreError> {
        let path = self.path_for(id)?;
        let _lock = self.lock(id)?;
        fs::remove_file(&path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                CoreError::NotFound(id.0.clone())
            } else {
                CoreError::Io(e)
            }
        })?;
        sync_parent(&self.root)?;
        Ok(())
    }

    /// Exports a profile to a new path without clobbering an existing file.
    /// Exports a profile without clobbering an existing destination.
    pub fn export(&self, id: &ProfileId, output: impl AsRef<Path>) -> Result<(), CoreError> {
        let profile = self.load(id)?;
        let path = output.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        self.write_new(path, &serde_json::to_vec_pretty(&profile)?)
    }

    /// Imports a profile after full schema and application validation.
    /// Imports a file after schema and application validation.
    pub fn import(&self, input: impl AsRef<Path>) -> Result<Profile, CoreError> {
        let mut profile: Profile = serde_json::from_slice(&fs::read(input)?)?;
        self.validate(&profile)?;
        profile.provenance = ConfigurationProvenance::ImportedFile;
        self.create(&profile)?;
        Ok(profile)
    }

    /// Applies application validation; this does not establish firmware compatibility.
    /// Runs application validation; this does not prove firmware compatibility.
    pub fn validate(&self, profile: &Profile) -> Result<(), CoreError> {
        if profile.schema_version != PROFILE_SCHEMA_VERSION {
            return Err(CoreError::UnsupportedSchema(profile.schema_version));
        }
        let issues = validate_profile(profile);
        if issues.is_empty() {
            Ok(())
        } else {
            Err(CoreError::InvalidProfile(issues))
        }
    }

    fn path_for(&self, id: &ProfileId) -> Result<PathBuf, CoreError> {
        if id.0.is_empty()
            || id.0.len() > 128
            || !id
                .0
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "-_".contains(c))
        {
            return Err(CoreError::InvalidId(id.0.clone()));
        }
        Ok(self.root.join(format!("{}.json", id.0)))
    }

    fn lock(&self, id: &ProfileId) -> Result<MutationLock, CoreError> {
        fs::create_dir_all(&self.root)?;
        let path = self.root.join(format!(".{}.lock", id.0));
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut file) => {
                use std::io::Write as _;
                if let Err(error) =
                    writeln!(file, "pid={}", std::process::id()).and_then(|()| file.sync_all())
                {
                    let _ = fs::remove_file(&path);
                    return Err(CoreError::Io(error));
                }
                Ok(MutationLock { path })
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                Err(CoreError::Busy(id.0.clone()))
            }
            Err(e) => Err(e.into()),
        }
    }

    fn write_new(&self, target: &Path, bytes: &[u8]) -> Result<(), CoreError> {
        let temp = temp_path(target);
        let result = (|| {
            let mut file = create_new(&temp)?;
            file.write_all(bytes)?;
            file.sync_all()?;
            fs::hard_link(&temp, target).map_err(|e| {
                if e.kind() == std::io::ErrorKind::AlreadyExists {
                    CoreError::AlreadyExists(target.to_path_buf())
                } else {
                    CoreError::Io(e)
                }
            })?;
            fs::remove_file(&temp)?;
            if let Some(parent) = target.parent() {
                sync_parent(parent)?;
            }
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temp);
        }
        result
    }

    fn atomic_replace(&self, target: &Path, bytes: &[u8]) -> Result<(), CoreError> {
        self.atomic_replace_with_hook(target, bytes, |_| Ok(()))
    }

    fn atomic_replace_with_hook<F>(
        &self,
        target: &Path,
        bytes: &[u8],
        mut hook: F,
    ) -> Result<(), CoreError>
    where
        F: FnMut(WriteStage) -> Result<(), CoreError>,
    {
        let temp = temp_path(target);
        let result = (|| {
            hook(WriteStage::BeforeTempCreate)?;
            let destination_permissions = fs::metadata(target)?.permissions();
            let mut file = create_new(&temp)?;
            hook(WriteStage::AfterTempCreate)?;
            let split = bytes.len() / 2;
            file.write_all(&bytes[..split])?;
            hook(WriteStage::DuringWrite)?;
            file.write_all(&bytes[split..])?;
            // Preserve the destination's portable permission bits across replacement.
            // Ownership, ACLs and extended attributes remain platform/filesystem-specific.
            file.set_permissions(destination_permissions)?;
            file.sync_all()?;
            hook(WriteStage::AfterFileSync)?;
            // Windows replacement requires the prepared source handle to be closed; the
            // destination remains untouched until the native replacement call succeeds.
            drop(file);
            hook(WriteStage::BeforeReplace)?;
            replace_existing(&temp, target)?;
            hook(WriteStage::AfterReplaceVisible)?;
            if let Some(parent) = target.parent() {
                hook(WriteStage::BeforeDirectorySync)?;
                sync_parent(parent)?;
                hook(WriteStage::AfterDirectorySync)?;
            }
            Ok(())
        })();
        if result.is_err() && temp.exists() {
            // The destination is still authoritative because replacement did not consume the
            // temp path. A cleanup failure is returned and the temp is retained for inspection.
            hook(WriteStage::DuringCleanup)?;
            fs::remove_file(&temp)?;
        }
        result
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WriteStage {
    BeforeTempCreate,
    AfterTempCreate,
    DuringWrite,
    AfterFileSync,
    BeforeReplace,
    AfterReplaceVisible,
    BeforeDirectorySync,
    AfterDirectorySync,
    DuringCleanup,
}

struct MutationLock {
    path: PathBuf,
}
impl Drop for MutationLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn temp_path(target: &Path) -> PathBuf {
    let name = target
        .file_name()
        .and_then(|v| v.to_str())
        .unwrap_or("profile.json");
    target.parent().unwrap_or(Path::new(".")).join(format!(
        ".{name}.{}.{}.tmp",
        std::process::id(),
        TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ))
}
fn create_new(path: &Path) -> Result<fs::File, CoreError> {
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
    {
        Ok(file) => Ok(file),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            Err(CoreError::AlreadyExists(path.to_path_buf()))
        }
        Err(e) => Err(e.into()),
    }
}
#[cfg(unix)]
fn replace_existing(source: &Path, destination: &Path) -> Result<(), CoreError> {
    fs::rename(source, destination)?;
    Ok(())
}

#[cfg(windows)]
#[allow(unsafe_code)]
mod windows_replace {
    use super::*;
    pub(super) fn replace_existing(source: &Path, destination: &Path) -> Result<(), CoreError> {
        use std::{ffi::c_void, os::windows::ffi::OsStrExt};

        // ReplaceFileW performs a single filesystem replacement operation. Unlike a
        // remove-then-rename sequence, failure leaves the destination in place.
        #[link(name = "Kernel32")]
        unsafe extern "system" {
            fn ReplaceFileW(
                replaced_file_name: *const u16,
                replacement_file_name: *const u16,
                backup_file_name: *const u16,
                replace_flags: u32,
                exclude: *mut c_void,
                reserved: *mut c_void,
            ) -> i32;
        }

        let destination_w: Vec<u16> = destination
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect();
        let source_w: Vec<u16> = source.as_os_str().encode_wide().chain(Some(0)).collect();
        // Windows readers must have opened the destination with delete sharing. If not,
        // ReplaceFileW fails safely and the original remains in place.
        // SAFETY: both paths are NUL-terminated UTF-16 buffers alive for the call; optional
        // backup/metadata pointers are null as permitted by ReplaceFileW. The API does not retain
        // pointers after returning. Failure is propagated without deleting the destination.
        let result = unsafe {
            ReplaceFileW(
                destination_w.as_ptr(),
                source_w.as_ptr(),
                std::ptr::null(),
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        if result == 0 {
            return Err(CoreError::Io(std::io::Error::last_os_error()));
        }
        Ok(())
    }
}

#[cfg(windows)]
fn replace_existing(source: &Path, destination: &Path) -> Result<(), CoreError> {
    windows_replace::replace_existing(source, destination)
}
#[cfg(not(any(unix, windows)))]
fn replace_existing(_source: &Path, _destination: &Path) -> Result<(), CoreError> {
    Err(CoreError::Io(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "safe atomic replacement is unsupported on this platform",
    )))
}

#[cfg(unix)]
fn sync_parent(parent: &Path) -> Result<(), CoreError> {
    fs::File::open(parent)?.sync_all()?;
    Ok(())
}
#[cfg(windows)]
fn sync_parent(_parent: &Path) -> Result<(), CoreError> {
    // Rust's std filesystem API does not expose a portable directory FlushFileBuffers
    // operation. File contents are flushed before ReplaceFileW; directory durability
    // across power loss is not claimed on Windows.
    Ok(())
}
#[cfg(not(any(unix, windows)))]
fn sync_parent(_parent: &Path) -> Result<(), CoreError> {
    Ok(())
}
fn now_unix_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis().min(u64::MAX as u128) as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn temp() -> PathBuf {
        std::env::temp_dir().join(format!(
            "th80-core-test-{}-{}",
            std::process::id(),
            TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ))
    }
    #[test]
    fn creates_lists_loads_exports_imports_and_deletes() {
        let dir = temp();
        let repo = ProfileRepository::new(&dir);
        let p = Profile::new("Daily");
        repo.create(&p).unwrap();
        assert_eq!(repo.list().unwrap().len(), 1);
        let mut expected = p.clone();
        expected.provenance = ConfigurationProvenance::SerializedToDisk;
        assert_eq!(repo.load(&p.id).unwrap(), expected);
        let exported = dir.with_extension("export.json");
        repo.export(&p.id, &exported).unwrap();
        let imported = ProfileRepository::new(dir.join("imported"));
        let loaded = imported.import(&exported).unwrap();
        assert_eq!(loaded.provenance, ConfigurationProvenance::ImportedFile);
        assert!(matches!(
            repo.export(&p.id, &exported),
            Err(CoreError::AlreadyExists(_))
        ));
        repo.delete(&p.id).unwrap();
        assert!(repo.list().unwrap().is_empty());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn lighting_values_remain_stable_across_load_edit_save_export_and_import() {
        use crate::config::{LightingMode, RgbColor};

        let dir = temp();
        let export_path = dir.with_extension("lighting-export.json");
        let import_dir = dir.with_extension("lighting-import");
        let repo = ProfileRepository::new(&dir);
        let mut profile = Profile::new("Lighting round trip");
        profile.configuration.lighting.enabled = Some(false);
        profile.configuration.lighting.mode = Some(LightingMode::Custom("vendor-mode-x".into()));
        profile.configuration.lighting.brightness = Some(0);
        profile.configuration.lighting.speed = Some(37);
        profile.configuration.lighting.color = Some(RgbColor {
            red: 0,
            green: 128,
            blue: 255,
        });
        profile.configuration.lighting.effect_parameters.insert(
            "opaque.vendor.value".into(),
            serde_json::json!({"nested": [1, false, "x"]}),
        );
        let expected_lighting = profile.configuration.lighting.clone();

        repo.create(&profile).unwrap();
        let loaded = repo.load(&profile.id).unwrap();
        assert_eq!(loaded.configuration.lighting, expected_lighting);

        let mut edited = loaded;
        edited.description = "unrelated metadata edit".into();
        repo.save(&edited).unwrap();
        assert_eq!(
            repo.load(&profile.id).unwrap().configuration.lighting,
            expected_lighting
        );

        repo.export(&profile.id, &export_path).unwrap();
        let imported = ProfileRepository::new(&import_dir);
        let imported_profile = imported.import(&export_path).unwrap();
        assert_eq!(imported_profile.configuration.lighting, expected_lighting);
        assert_eq!(
            imported.load(&profile.id).unwrap().configuration.lighting,
            expected_lighting
        );

        let _ = fs::remove_dir_all(dir);
        let _ = fs::remove_dir_all(import_dir);
        let _ = fs::remove_file(export_path);
    }
    #[test]
    fn case_only_profile_id_collisions_never_overwrite_existing_data() {
        let dir = temp();
        let repo = ProfileRepository::new(&dir);
        let mut first = Profile::new("original-case");
        first.id = ProfileId::new("CaseSensitiveId");
        repo.create(&first).unwrap();
        let mut second = Profile::new("different-case");
        second.id = ProfileId::new("casesensitiveid");
        let result = repo.create(&second);
        if result.is_ok() {
            // Case-sensitive filesystems can represent both identifiers independently.
            assert_eq!(repo.load(&first.id).unwrap().name, "original-case");
            assert_eq!(repo.load(&second.id).unwrap().name, "different-case");
        } else {
            // Case-insensitive filesystems must fail closed without replacing the first.
            assert!(matches!(
                result,
                Err(CoreError::Busy(_) | CoreError::AlreadyExists(_))
            ));
            assert_eq!(repo.load(&first.id).unwrap().name, "original-case");
        }
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn save_atomically_replaces_only_an_existing_profile_and_rejects_stale_writers() {
        let dir = temp();
        let repo = ProfileRepository::new(&dir);
        let p = Profile::new("A");
        repo.create(&p).unwrap();
        let mut first = repo.load(&p.id).unwrap();
        let stale = repo.load(&p.id).unwrap();
        first.name = "B".into();
        repo.save(&first).unwrap();
        assert_eq!(repo.load(&p.id).unwrap().name, "B");
        assert!(matches!(repo.save(&stale), Err(CoreError::Conflict(_))));
        let missing = Profile::new("missing");
        assert!(matches!(repo.save(&missing), Err(CoreError::NotFound(_))));
        let _ = fs::remove_dir_all(dir);
    }
    #[test]
    fn refuses_mutation_when_cross_process_lock_exists() {
        let dir = temp();
        let repo = ProfileRepository::new(&dir);
        let p = Profile::new("locked");
        repo.create(&p).unwrap();
        fs::write(dir.join(format!(".{}.lock", p.id.0)), b"pid=other").unwrap();
        assert!(matches!(
            repo.save(&repo.load(&p.id).unwrap()),
            Err(CoreError::Busy(_))
        ));
        let _ = fs::remove_dir_all(dir);
    }
    #[test]
    fn failed_import_and_corrupt_load_preserve_original_bytes() {
        let dir = temp();
        let repo = ProfileRepository::new(&dir);
        fs::create_dir_all(&dir).unwrap();
        let bad = dir.join("bad.json");
        let original = b"{ truncated";
        fs::write(&bad, original).unwrap();
        assert!(matches!(repo.import(&bad), Err(CoreError::Json(_))));
        assert_eq!(fs::read(&bad).unwrap(), original);
        let _ = fs::remove_dir_all(dir);
    }
    #[test]
    fn preserves_unknown_top_level_extension_fields() {
        let mut p = Profile::new("extensions");
        p.extensions.insert(
            "future_vendor_metadata".into(),
            serde_json::json!({"opaque":true}),
        );
        let bytes = serde_json::to_vec(&p).unwrap();
        let decoded: Profile = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(decoded.extensions, p.extensions);
    }
    #[test]
    fn rejects_unsupported_schema_and_missing_required_fields() {
        let dir = temp();
        let repo = ProfileRepository::new(&dir);
        let mut p = Profile::new("schema");
        p.schema_version = 99;
        assert!(matches!(
            repo.validate(&p),
            Err(CoreError::UnsupportedSchema(99))
        ));
        let missing: Result<Profile, _> =
            serde_json::from_str(r#"{"schema_version":1,"name":"missing id"}"#);
        assert!(missing.is_err());
    }
    #[test]
    fn injected_failures_cover_pre_and_post_commit_replacement_states() {
        let dir = temp();
        let repo = ProfileRepository::new(&dir);
        let mut original = Profile::new("original");
        original.id = ProfileId::new("profile");
        repo.create(&original).unwrap();
        let original_bytes = fs::read(dir.join("profile.json")).unwrap();
        let mut replacement = repo.load(&original.id).unwrap();
        replacement.name = "replacement".into();
        replacement.metadata.modified_at_unix_ms += 1;
        let replacement_bytes = serde_json::to_vec_pretty(&replacement).unwrap();

        for failing_stage in [
            WriteStage::BeforeTempCreate,
            WriteStage::AfterTempCreate,
            WriteStage::DuringWrite,
            WriteStage::AfterFileSync,
            WriteStage::BeforeReplace,
            WriteStage::AfterReplaceVisible,
            WriteStage::BeforeDirectorySync,
            WriteStage::AfterDirectorySync,
        ] {
            fs::write(dir.join("profile.json"), &original_bytes).unwrap();
            let result = repo.atomic_replace_with_hook(
                &dir.join("profile.json"),
                &replacement_bytes,
                |stage| {
                    if stage == failing_stage {
                        Err(CoreError::Io(std::io::Error::other(
                            "injected interruption",
                        )))
                    } else {
                        Ok(())
                    }
                },
            );
            assert!(
                result.is_err(),
                "expected injected failure at {failing_stage:?}"
            );
            let committed = matches!(
                failing_stage,
                WriteStage::AfterReplaceVisible
                    | WriteStage::BeforeDirectorySync
                    | WriteStage::AfterDirectorySync
            );
            let loaded = repo.load(&original.id).unwrap();
            assert_eq!(
                loaded.name,
                if committed { "replacement" } else { "original" }
            );
            let leftovers: Vec<_> = fs::read_dir(&dir)
                .unwrap()
                .map(|entry| entry.unwrap().file_name())
                .collect();
            assert!(leftovers.iter().any(|name| name == "profile.json"));
            assert_eq!(leftovers.len(), 1, "stage {failing_stage:?}: {leftovers:?}");
        }
        // Simulate a failed cleanup after a pre-commit write error. The original remains
        // authoritative and the prepared temp is deliberately retained for manual inspection.
        fs::write(dir.join("profile.json"), &original_bytes).unwrap();
        let result =
            repo.atomic_replace_with_hook(&dir.join("profile.json"), &replacement_bytes, |stage| {
                match stage {
                    WriteStage::DuringWrite | WriteStage::DuringCleanup => Err(CoreError::Io(
                        std::io::Error::other("injected write/cleanup failure"),
                    )),
                    _ => Ok(()),
                }
            });
        assert!(result.is_err());
        assert_eq!(repo.load(&original.id).unwrap().name, "original");
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 2);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn subprocess_crash_after_replace_worker() {
        let Ok(root) = std::env::var("TH80_CRASH_TEST_ROOT") else {
            return;
        };
        let root = PathBuf::from(root);
        let repo = ProfileRepository::new(&root);
        let id = ProfileId::new("crash-profile");
        let _lock = repo.lock(&id).unwrap();
        let mut replacement = repo.load(&id).unwrap();
        replacement.name = "committed-before-crash".into();
        replacement.metadata.modified_at_unix_ms += 1;
        let bytes = serde_json::to_vec_pretty(&replacement).unwrap();
        let path = repo.path_for(&id).unwrap();
        let _ = repo.atomic_replace_with_hook(&path, &bytes, |stage| {
            if stage == WriteStage::AfterReplaceVisible {
                std::process::exit(86);
            }
            Ok(())
        });
        panic!("child did not reach the post-replacement checkpoint");
    }

    #[test]
    fn subprocess_termination_after_replacement_leaves_complete_new_profile_and_stale_lock() {
        let dir = temp();
        let repo = ProfileRepository::new(&dir);
        let mut profile = Profile::new("before-crash");
        profile.id = ProfileId::new("crash-profile");
        repo.create(&profile).unwrap();
        let child = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "repository::tests::subprocess_crash_after_replace_worker",
            ])
            .env("TH80_CRASH_TEST_ROOT", &dir)
            .status()
            .unwrap();
        assert_eq!(child.code(), Some(86));
        assert_eq!(
            repo.load(&profile.id).unwrap().name,
            "committed-before-crash"
        );
        assert!(dir.join(format!(".{}.lock", profile.id.0)).exists());
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 2);
        assert!(matches!(
            repo.save(&repo.load(&profile.id).unwrap()),
            Err(CoreError::Busy(_))
        ));
        // The child has exited; explicit operator recovery removes only its stale lock.
        fs::remove_file(dir.join(format!(".{}.lock", profile.id.0))).unwrap();
        assert_eq!(
            repo.load(&profile.id).unwrap().name,
            "committed-before-crash"
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn concurrent_different_profile_creates_and_atomic_readers_are_safe() {
        let dir = temp();
        let repo = ProfileRepository::new(&dir);
        let a = Profile::new("A");
        let b = Profile::new("B");
        let r1 = repo.clone();
        let r2 = repo.clone();
        let a1 = a.clone();
        let b1 = b.clone();
        let t1 = std::thread::spawn(move || r1.create(&a1));
        let t2 = std::thread::spawn(move || r2.create(&b1));
        t1.join().unwrap().unwrap();
        t2.join().unwrap().unwrap();
        let mut writer = repo.load(&a.id).unwrap();
        writer.name = "updated".into();
        let r = repo.clone();
        let id = a.id.clone();
        let reader = std::thread::spawn(move || {
            for _ in 0..100 {
                let p = r.load(&id).unwrap();
                assert!(p.name == "A" || p.name == "updated");
            }
        });
        repo.save(&writer).unwrap();
        reader.join().unwrap();
        assert_eq!(repo.list().unwrap().len(), 2);
        let _ = fs::remove_dir_all(dir);
    }

    #[cfg(unix)]
    #[test]
    fn replacement_preserves_existing_unix_mode_bits() {
        use std::os::unix::fs::PermissionsExt;
        let dir = temp();
        let repo = ProfileRepository::new(&dir);
        let profile = Profile::new("before");
        repo.create(&profile).unwrap();
        let path = dir.join(format!("{}.json", profile.id.0));
        fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();
        let mut changed = repo.load(&profile.id).unwrap();
        changed.name = "after".into();
        repo.save(&changed).unwrap();
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o640
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn open_reader_retains_complete_old_file_while_replacement_becomes_visible() {
        use std::io::Read as _;
        let dir = temp();
        let repo = ProfileRepository::new(&dir);
        let profile = Profile::new("before");
        repo.create(&profile).unwrap();
        let path = dir.join(format!("{}.json", profile.id.0));
        let mut reader = fs::File::open(&path).unwrap();
        let mut changed = repo.load(&profile.id).unwrap();
        changed.name = "after".into();
        repo.save(&changed).unwrap();
        let mut old_bytes = Vec::new();
        reader.read_to_end(&mut old_bytes).unwrap();
        let old_profile: Profile = serde_json::from_slice(&old_bytes).unwrap();
        assert_eq!(old_profile.name, "before");
        assert_eq!(repo.load(&profile.id).unwrap().name, "after");
        let _ = fs::remove_dir_all(dir);
    }

    #[cfg(windows)]
    #[test]
    fn windows_replace_fails_safely_when_destination_denies_delete_sharing() {
        use std::os::windows::fs::OpenOptionsExt;
        let dir = temp();
        let repo = ProfileRepository::new(&dir);
        let profile = Profile::new("before");
        repo.create(&profile).unwrap();
        let original = fs::read(dir.join(format!("{}.json", profile.id.0))).unwrap();
        let mut changed = repo.load(&profile.id).unwrap();
        changed.name = "after".into();
        let held = fs::OpenOptions::new()
            .read(true)
            .share_mode(0x0000_0001 | 0x0000_0002) // FILE_SHARE_READ | FILE_SHARE_WRITE; no DELETE
            .open(dir.join(format!("{}.json", profile.id.0)))
            .unwrap();
        assert!(repo.save(&changed).is_err());
        assert_eq!(
            fs::read(dir.join(format!("{}.json", profile.id.0))).unwrap(),
            original
        );
        drop(held);
        let mut retry = repo.load(&profile.id).unwrap();
        retry.name = "after".into();
        repo.save(&retry).unwrap();
        assert_eq!(repo.load(&profile.id).unwrap().name, "after");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn abandoned_temporary_files_are_ignored_by_listing() {
        let dir = temp();
        let repo = ProfileRepository::new(&dir);
        let p = Profile::new("visible");
        repo.create(&p).unwrap();
        fs::write(dir.join(".orphan.1.1.tmp"), b"partial").unwrap();
        fs::write(dir.join(".profile.lock"), b"stale").unwrap();
        assert_eq!(repo.list().unwrap().len(), 1);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn unknown_configuration_sections_are_preserved_and_unknown_layer_fields_fail_closed() {
        let json = r#"{"schema_version":1,"id":"p1","name":"strict","configuration":{"keymap":{"layers":[]},"macros":{"items":[]},"lighting":{"mystery":true},"options":{"values":{}}},"metadata":{"created_at_unix_ms":1,"modified_at_unix_ms":1,"tags":[]},"provenance":"local_draft"}"#;
        let profile: Profile = serde_json::from_str(json).unwrap();
        assert_eq!(profile.configuration.lighting.extensions["mystery"], true);

        let unknown_layer = r#"{"schema_version":1,"id":"p2","name":"strict","configuration":{"keymap":{"layers":[{"id":"base","name":"Base","positions":{},"future_geometry":{"x":1}}]},"macros":{"items":[]},"lighting":{},"options":{"values":{}}},"metadata":{"created_at_unix_ms":1,"modified_at_unix_ms":1,"tags":[]},"provenance":"local_draft"}"#;
        assert!(serde_json::from_str::<Profile>(unknown_layer).is_err());

        let unknown_action = r#"{"schema_version":1,"id":"p3","name":"strict","configuration":{"keymap":{"layers":[{"id":"base","name":"Base","positions":{"ESC":{"action":{"type":"key","code":"KEY_ESC","future_code_domain":"vendor"}}}}]},"macros":{"items":[]},"lighting":{},"options":{"values":{}}},"metadata":{"created_at_unix_ms":1,"modified_at_unix_ms":1,"tags":[]},"provenance":"local_draft"}"#;
        assert!(serde_json::from_str::<Profile>(unknown_action).is_err());

        let unknown_macro_step = r#"{"schema_version":1,"id":"p4","name":"strict","configuration":{"keymap":{"layers":[]},"macros":{"items":[{"id":"m1","name":"Macro","steps":[{"type":"text","value":"hello","future_encoding":"vendor"}]}]},"lighting":{},"options":{"values":{}}},"metadata":{"created_at_unix_ms":1,"modified_at_unix_ms":1,"tags":[]},"provenance":"local_draft"}"#;
        assert!(serde_json::from_str::<Profile>(unknown_macro_step).is_err());
    }
}
