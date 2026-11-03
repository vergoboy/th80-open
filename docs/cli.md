# CLI usage

## Offline profile management

Build or run `th80ctl` from the workspace root. The default profile repository is `~/.local/share/th80-profiles`; set `TH80_PROFILE_DIR` to use an isolated or alternate directory.

```sh
cargo run --locked -p th80-cli -- --help
cargo run --locked -p th80-cli -- profile create --name "Daily" --description "Local work profile"
cargo run --locked -p th80-cli -- profile list
cargo run --locked -p th80-cli -- profile list --output json
cargo run --locked -p th80-cli -- profile show PROFILE_ID --format text
cargo run --locked -p th80-cli -- profile rename PROFILE_ID --name "Work"
cargo run --locked -p th80-cli -- profile duplicate PROFILE_ID --name "Work copy"
cargo run --locked -p th80-cli -- profile add-layer PROFILE_ID --layer-id base --name Base
cargo run --locked -p th80-cli -- profile rename-layer PROFILE_ID --layer-id base --name "Main"
cargo run --locked -p th80-cli -- profile remove-layer PROFILE_ID --layer-id scratch
cargo run --locked -p th80-cli -- profile set-key PROFILE_ID --layer base --position logical-01 --key KEY_A
cargo run --locked -p th80-cli -- profile set-action PROFILE_ID --layer base --position logical-02 --action '{"type":"modifier","modifier":"CTRL"}'
cargo run --locked -p th80-cli -- profile validate ./profile.json
cargo run --locked -p th80-cli -- profile validate ./profile.json --output json
cargo run --locked -p th80-cli -- profile export PROFILE_ID --output ./profile.json
cargo run --locked -p th80-cli -- profile import ./profile.json
cargo run --locked -p th80-cli -- profile delete PROFILE_ID
```

`show` defaults to JSON for backwards compatibility; `--format text` prints a concise summary. `list --output json` emits a JSON array of profiles.

Rename changes the display name and updates the modification timestamp. Duplicate copies the saved profile description, model association, configuration, tags and top-level extension fields to a new ID with fresh local timestamps. Neither command changes a physical device.

### Macro editing

```sh
th80ctl profile macro add PROFILE_ID --macro-id tap --name "Tap A"
th80ctl profile macro list PROFILE_ID
th80ctl profile macro show PROFILE_ID --macro-id tap
th80ctl profile macro rename PROFILE_ID --macro-id tap --name "Tap key A"
th80ctl profile macro step-add PROFILE_ID --macro-id tap --step key-down:KEY_A --step key-up:KEY_A
th80ctl profile macro step-replace PROFILE_ID --macro-id tap --index 0 --step key-down:KEY_B
th80ctl profile macro step-remove PROFILE_ID --macro-id tap --index 0
th80ctl profile macro step-move PROFILE_ID --macro-id tap --from 2 --to 1
th80ctl profile macro remove PROFILE_ID --macro-id tap
```

Repeat `--step` to append a balanced sequence in one save. Every edit validates the entire profile; an intermediate unmatched key-down cannot be saved. Step indexes are zero-based.

The typed set-action command accepts one JSON Action object using the same tagged variants as the profile format. Unknown variants are rejected by serde; no firmware-specific action is invented. Removing a layer still referenced by a LayerSwitch/LayerMomentary assignment fails core validation and leaves the saved profile unchanged.

### Lighting and options editing

```sh
th80ctl profile lighting show PROFILE_ID
th80ctl profile lighting set PROFILE_ID --enabled true --mode static --brightness 50 --speed 25 --color 255,80,20
th80ctl profile lighting key-set PROFILE_ID --position logical-01 --color 255,80,20
th80ctl profile lighting key-remove PROFILE_ID --position logical-01
th80ctl profile lighting param-set PROFILE_ID --key effect.local --value '{"amount":2}'
th80ctl profile lighting param-remove PROFILE_ID --key effect.local
th80ctl profile options list PROFILE_ID
th80ctl profile options set PROFILE_ID --key local.preference --value '{"enabled":true}'
th80ctl profile options remove PROFILE_ID --key local.preference
```

Lighting modes and option keys are generic application abstractions, not confirmed TH80 firmware settings. Brightness/speed values use the local normalized range 0–100. `--value` must be valid JSON. Existing unrelated options, lighting parameters and profile extension fields are retained.

### Validation diagnostics and exit codes

`profile validate FILE --output json` emits one JSON object on stdout with `schema_version`, `valid`, `device_compatibility` and a `diagnostics` array. Each diagnostic has a stable code, severity, domain, message, profile ID and field path; optional context/remediation fields are available. JSON stdout is kept separate from stderr error text. `device_compatibility` is currently `not_verified` for same/unspecified model associations; application validity is not firmware compatibility.

Exit codes:

| Code | Meaning |
|---:|---|
| `0` | Command succeeded; validation passed when applicable. |
| `2` | Profile application validation failed. |
| `3` | Input file, malformed JSON or argument value error. |
| `4` | Repository/filesystem error, duplicate ID, lock contention or stale-save conflict. |
| `64` | CLI usage or unknown command. |

Validation diagnostic identifiers and the core API are documented in [core-api.md](core-api.md). Human-readable output is the default. Machine consumers should use JSON fields and stable codes, never parse message strings.

## Repository safety and concurrency

Create/import/export do not overwrite existing files. New files are serialized to same-directory temporary files, synchronized, then installed with an exclusive hard link. Updates use a same-directory, exclusively created temporary file, complete serialization, copy the destination’s portable permission bits, call `sync_all`, then use a platform replacement primitive. Ownership, ACLs and extended attributes remain platform/filesystem-specific. No implementation deletes the destination before replacement.

### Platform guarantees

| Platform | Status | Replacement / durability |
|---|---|---|
| Linux | VERIFIED (native local tests) | Same-directory rename-over-existing; file sync and best-effort directory sync, with directory-sync errors surfaced after visibility. |
| Windows | COMPILES ONLY until native workflow passes | Native `ReplaceFileW`; failure is returned without a delete-first fallback. Destination handles lacking delete sharing can block replacement. File sync is performed; parent-directory durability is not claimed. |
| macOS | UNVERIFIED until native workflow passes | Uses the Unix same-directory rename and directory-sync path; native CI coverage is configured but must execute before marking verified. |

See [repository platform CI](../.github/workflows/repository-platform.yml). A successful cross-target compile is not native filesystem verification. Atomic visibility does not imply power-loss durability. Network filesystems and unusual mount configurations are outside the guarantee.

### Concurrency and stale locks

Mutations use a per-profile `create_new` lock file containing `pid=...`. A concurrent same-profile mutation receives a busy error; stale loaded profiles receive a conflict after the first writer commits. Different profile IDs use independent locks. Readers open complete files while updates replace the directory entry; they do not observe a partially written temporary file. Case-only distinct IDs can collide on case-insensitive filesystems and are not guaranteed to coexist.

Normal process exit removes its lock. Abrupt termination leaves it behind and all later mutations fail closed. Locks are never broken based on age. To recover, inspect `.PROFILE_ID.lock`, identify its PID, confirm that process is no longer running and that no other writer owns the profile, then manually remove that one lock file. If the owner cannot be established, do not remove it. The repository does not provide an automatic stale-lock command.

### Crash states and recovery

- Before replacement: the old profile remains authoritative. An abrupt process exit may leave a hidden `.tmp`; listing ignores it and does not auto-promote it.
- Immediately after replacement: the complete new profile is authoritative. An abrupt exit may leave the lock file behind.
- After Unix parent-directory sync: the directory entry has been synchronized as supported by the filesystem; this is not a universal power-loss guarantee.
- If a Unix directory sync fails after replacement, the operation reports an I/O error even though the new profile is visible. Reload before retrying.
- On Windows the replacement file is flushed before `ReplaceFileW`, but parent-directory durability is not claimed.

There is no journal or automatic rollback: the platform replacement primitive provides an old-or-new complete-file boundary, and ambiguous automatic recovery risks discarding user data. Malformed profiles and temporary files are preserved for explicit inspection. Cleanup failures leave the temporary file and return an error. A failed recovery never overwrites the primary profile.

## Offline and passive tools

The CLI is strictly offline. It never initializes the legacy HIDAPI executable, probes USB, opens a device node, or sends/receives reports. `th80-probe` is passive Linux sysfs inspection; `th80-hid-research` remains a separate research utility and is not needed for profile commands.

```sh
cargo run --locked -p th80-probe -- list
cargo run --locked -p th80-probe -- inspect --all
cargo run --locked -p th80-probe -- capabilities --json
```

The probe reads sysfs metadata and kernel-exposed report descriptors only. See [Phase 5A](../PHASE5A_DIAGNOSTIC_CLI.md) and [Phase 5B](../PHASE5B_HID_CAPABILITY_DISCOVERY.md).
