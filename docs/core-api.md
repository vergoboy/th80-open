# `th80-core` library API

`th80-core` is the reusable synchronous domain library. It has no dependency on the CLI, HIDAPI, USB libraries, OS device APIs, async runtime, UI framework or physical transport.

## Public modules and intended consumers

| Module | Public surface | Intended use |
|---|---|---|
| `config` | `Profile`, `ProfileId`, `Configuration`, metadata/provenance, keymap/actions, macro, lighting and keyboard-option models | Construct and edit local profile data. |
| `capability` | `CapabilityDescriptor`, `FirmwareConstraintEvidence`, `EvidenceClass`, `SupportStatus`, `offline_capabilities` | Inspect evidence-labelled capabilities; `UNKNOWN` remains unknown. |
| `device` | `DeviceDescriptor` | Carry known identity plus explicitly optional revision/firmware metadata. |
| `validation` | `validate_profile`, `assess_device_compatibility`, `DeviceCompatibility*`, `ApplicationPolicy` | Run deterministic local validation and separately inspect compatibility status. |
| `error` | `CoreError`, `Diagnostic`, severity/domain enums, `ValidationIssue` alias | Handle structured failures without parsing messages. |
| `repository` | `ProfileRepository` | Load, create, update, list, import, export and delete local JSON profiles. |

The root re-exports the common `Profile`, `ProfileId`, `Configuration`, `DeviceDescriptor`, `DeviceIdentity`, `DeviceKind`, `CoreError`, diagnostics, `ProfileRepository`, `ApplicationPolicy` and `PROFILE_SCHEMA_VERSION`. The CLI is not part of this library API.

Public domain fields are intentionally accessible to support different frontends. Consumers should use `validate_profile` before persistence and `ProfileRepository` for storage; direct struct construction does not imply device compatibility. `ProfileId::new` is an inexpensive wrapper constructor; repository path validation enforces safe identifier characters and bounds.

## Compatibility policy

The package is `0.1.0` and is pre-1.0. Public Rust APIs may change between `0.1.x` releases. Intentional breaking changes should be called out in release notes and migration guidance; no full stable-API guarantee is made yet. Persisted JSON `schema_version` is independent of the crate version. Schema version 1 is accepted; unsupported future versions are rejected. A schema migration must be explicit, tested against prior fixtures, and preserve original bytes if it fails.

## Validation domains

- `ApplicationValidation`: local structural and application-policy checks.
- `CapabilityEvidence`: sourced facts with an evidence grade and rationale.
- `FirmwareCompatibility`: whether a particular firmware is known to accept a profile; currently unverified.

`validate_profile` returns only application-validation diagnostics. `assess_device_compatibility` reports `NotVerified` unless the profile is explicitly associated with a different model. An application-valid profile does not establish firmware support.

`APPLICATION_POLICY` currently specifies a maximum of 10,000 macro steps, a maximum individual delay of 600,000 ms and normalized lighting values up to 100. These are local application safety/representation bounds, not firmware constraints. RGB channels use the representable `u8` range; no device-specific encoding is implied.

## Diagnostic code registry

Validation codes are stable identifiers. Message wording may improve without changing the code.

| Code | Meaning |
|---|---|
| `unsupported_schema` | Profile JSON schema is not supported. |
| `invalid_profile_id` | Profile ID violates local identifier policy. |
| `required` | Required profile field is empty. |
| `invalid_timestamp` | Modification timestamp precedes creation timestamp. |
| `unsupported_provenance` | Profile claims device-reported/applied state without a verified operation. |
| `duplicate_layer_id`, `invalid_layer_id` | Layer IDs are duplicated or empty. |
| `invalid_position`, `invalid_action`, `invalid_combination` | Invalid logical position or typed action. |
| `invalid_layer_name`, `invalid_macro_name` | Layer or macro display name is empty. |
| `invalid_layer_reference`, `invalid_macro_reference` | Action references a missing layer or macro. |
| `duplicate_macro_id`, `invalid_macro_id` | Macro IDs are duplicated or empty. |
| `macro_too_large`, `delay_too_long` | Application resource policy exceeded; not a firmware limit. |
| `invalid_key_down`, `invalid_key_up`, `unreleased_keys` | Macro key state sequence is inconsistent. |
| `empty_macro_text` | Macro text step is empty. |
| `brightness_out_of_range`, `speed_out_of_range` | Application-normalized lighting range exceeded. |
| `invalid_lighting_position`, `duplicate_per_key_color` | Invalid or duplicate logical per-key assignment. |
| `invalid_effect_parameter` | Effect parameter key is empty. |
| `unsafe_capability` | A capability descriptor incorrectly enables device communication. |

`CoreError::code()` provides stable repository/serialization error categories: `REPOSITORY_IO_ERROR`, `PROFILE_JSON_INVALID`, `PROFILE_VALIDATION_FAILED`, `PROFILE_ALREADY_EXISTS`, `PROFILE_NOT_FOUND`, `PROFILE_ID_INVALID`, `PROFILE_SCHEMA_UNSUPPORTED`, `PROFILE_LOCK_BUSY`, and `PROFILE_CONCURRENT_MODIFICATION`.

## Repository replacement and recovery contract

`ProfileRepository` mutations use one hidden lock file per profile ID, created exclusively. The lock records the owning process ID. Same-profile mutations from another process fail with `PROFILE_LOCK_BUSY`; they are not queued and are never silently last-writer-wins. A loaded profile is also revision-checked under the lock, so a stale copy fails with `PROFILE_CONCURRENT_MODIFICATION`. Different profile IDs use independent locks. On case-insensitive filesystems, identifiers differing only by case may refer to the same underlying filename and therefore cannot safely be treated as distinct profiles.

Normal lock release removes the lock file. Abrupt process termination leaves it behind. The repository never decides that a lock is stale based on age and never removes it automatically. Recovery is an explicit operator action: inspect `.PROFILE_ID.lock`, read its `pid=...` owner, confirm that process is no longer running (and that no other writer is active), then remove only that lock file. If ownership cannot be established, leave it in place and fail closed.

Updates write a unique, exclusively created temporary file beside the destination, write all JSON, and call `sync_all` before replacement. Linux and macOS use same-directory rename-over-existing; Windows uses the native `ReplaceFileW` operation. The destination’s portable permission bits are copied to the prepared replacement before its final sync. Ownership, ACLs and extended attributes remain platform/filesystem-specific. A Windows process holding the destination without delete sharing can make replacement fail; the original remains intact. Readers opening and closing the file normally see a complete old or new profile, not a partially written JSON document. A Windows-specific locked-destination test is included in the native CI job; Windows behavior is not considered verified until that workflow has run on a Windows runner.

The replacement operation is the commit/visibility boundary. Before it, the original is authoritative and an interrupted writer may leave a hidden `.tmp` file; the repository ignores it and does not auto-promote it. After it, the new complete profile is authoritative. A crash after visibility can leave the lock behind. On Unix, the parent directory is synchronized after replacement; if that operation fails, the API returns an I/O error even though the new profile is already visible, so callers must reload before retrying. Windows synchronizes the replacement file but does not claim parent-directory durability. None of these operations guarantee survival of every sudden power loss, storage-controller failure, network filesystem behavior or filesystem corruption. There is no transaction journal or automatic rollback because atomic replacement already leaves a complete old-or-new file and an automatic recovery decision could discard user data.

Abandoned temporary files are retained only when cleanup itself fails or the process terminates before cleanup. Inspect them only after confirming no writer remains; do not rename one over the primary profile automatically. Corrupt primary files are never overwritten by load/recovery. Export and create remain no-clobber operations.

## Examples

Run the complete library example:

```sh
cargo run --locked -p th80-core --example offline_profile
```

The example constructs multiple layers and typed actions, a macro, generic lighting and an application-owned option; validates and round-trips JSON; then saves and reloads through the repository. All data is synthetic and local.
