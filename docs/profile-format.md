# Offline profile JSON format

## Versioning

The current profile format is JSON schema version 1, independent of the Rust crate version. Future schema versions are rejected until an explicit migration exists. Import failure leaves the source file unchanged and does not create a partial repository entry.

## Example

    {
      "schema_version": 1,
      "id": "profile-example",
      "name": "Example",
      "description": "A local profile",
      "device_model": "th80_pro",
      "configuration": {
        "keymap": {
          "layers": [{
            "id": "base",
            "name": "Base",
            "positions": {
              "logical-01": {"action": {"type": "key", "code": "KEY_A"}}
            }
          }]
        },
        "macros": {
          "items": [{
            "id": "tap-a",
            "name": "Tap A",
            "steps": [
              {"type": "key_down", "key": "KEY_A"},
              {"type": "delay", "milliseconds": 20},
              {"type": "key_up", "key": "KEY_A"}
            ]
          }]
        },
        "lighting": {
          "enabled": null,
          "mode": null,
          "brightness": null,
          "speed": null,
          "color": null,
          "per_key": [],
          "effect_parameters": {}
        },
        "options": {"values": {"local.example": {"enabled": true}}}
      },
      "metadata": {
        "created_at_unix_ms": 1700000000000,
        "modified_at_unix_ms": 1700000000000,
        "tags": ["example"]
      },
      "provenance": "local_draft",
      "future_top_level_extension": {"retained": true}
    }

All positions are opaque logical identifiers. The example does not describe physical TH80 geometry, a firmware keycode table or a vendor packet.

## Optional lighting values

The fields **enabled**, **mode**, **brightness**, **speed** and **color** are optional. Null/missing means unspecified; explicit false, numeric zero and an explicit black RGB color are values and must not be treated as absence. Brightness and speed use an application-normalized 0–100 representation, not a documented TH80 firmware range.

Lighting modes are generic local values (static, breathing, wave, reactive, or custom). The effect_parameters map is opaque JSON for application-side values; its keys have no assumed device meaning.

## Compatibility and unknown data

- Unknown **top-level** fields are captured in the profile extension map and round-trip through serialization and repository save.
- The configuration options.values and lighting effect_parameters maps are explicitly extensible JSON maps.
- Unknown fields inside typed nested structures, or unknown tagged Action/MacroStep variants, are rejected with a deserialization error. They are not silently discarded or reinterpreted.
- A profile can be locally valid while incompatible with a specific firmware. Device compatibility remains NOT VERIFIED.

## Persistence

Use th80-core::ProfileRepository for storage. It validates schema and application policy, refuses duplicate create/import IDs, writes new exports without clobbering, atomically replaces saved profiles and rejects stale revisions. See [core API and repository contract](core-api.md).
