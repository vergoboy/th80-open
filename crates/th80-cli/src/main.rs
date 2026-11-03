use std::{collections::BTreeMap, env, path::PathBuf, process::ExitCode};
use th80_core::{
    config::{
        lighting::{LightingMode, PerKeyColor, RgbColor},
        macros::{MacroDefinition, MacroStep},
        Action, KeyAssignment, Layer, Profile, ProfileId,
    },
    validation::{assess_device_compatibility, validate_profile},
    CoreError, Diagnostic, ProfileRepository,
};

const USAGE: &str = "TH80 offline profile manager\n\nUSAGE:\n  th80ctl profile create --name NAME [--description TEXT]\n  th80ctl profile list [--output json]\n  th80ctl profile show PROFILE_ID [--format text|json]\n  th80ctl profile rename PROFILE_ID --name NAME\n  th80ctl profile duplicate PROFILE_ID [--name NAME]\n  th80ctl profile add-layer PROFILE_ID --layer-id ID --name NAME\n  th80ctl profile rename-layer PROFILE_ID --layer-id ID --name NAME\n  th80ctl profile remove-layer PROFILE_ID --layer-id ID\n  th80ctl profile set-key PROFILE_ID --layer ID --position POSITION --key KEY\n  th80ctl profile set-action PROFILE_ID --layer ID --position POSITION --action JSON\n  th80ctl profile macro add PROFILE_ID --macro-id ID --name NAME\n  th80ctl profile macro list PROFILE_ID\n  th80ctl profile macro show PROFILE_ID --macro-id ID\n  th80ctl profile macro rename PROFILE_ID --macro-id ID --name NAME\n  th80ctl profile macro remove PROFILE_ID --macro-id ID\n  th80ctl profile macro step-add PROFILE_ID --macro-id ID --step key-down:KEY|key-up:KEY|delay:MS|text:TEXT\n  th80ctl profile macro step-remove PROFILE_ID --macro-id ID --index N\n  th80ctl profile macro step-move PROFILE_ID --macro-id ID --from N --to N\n  th80ctl profile macro step-replace PROFILE_ID --macro-id ID --index N --step TYPE:VALUE\n  th80ctl profile lighting show PROFILE_ID\n  th80ctl profile lighting set PROFILE_ID [--enabled true|false] [--mode MODE] [--brightness 0..100] [--speed 0..100] [--color R,G,B]\n  th80ctl profile lighting key-set PROFILE_ID --position POS --color R,G,B\n  th80ctl profile lighting key-remove PROFILE_ID --position POS\n  th80ctl profile lighting param-set PROFILE_ID --key KEY --value JSON\n  th80ctl profile lighting param-remove PROFILE_ID --key KEY\n  th80ctl profile options list PROFILE_ID\n  th80ctl profile options set PROFILE_ID --key KEY --value JSON\n  th80ctl profile options remove PROFILE_ID --key KEY\n  th80ctl profile validate FILE.json [--output human|json]\n  th80ctl profile export PROFILE_ID --output FILE.json\n  th80ctl profile import FILE.json\n  th80ctl profile delete PROFILE_ID\n\nOffline only: profile operations never enumerate, open or communicate with devices.\nRepository: $TH80_PROFILE_DIR or ~/.local/share/th80-profiles";

#[derive(Debug)]
enum CliError {
    Usage(String),
    Input(String),
    Validation(String),
    Repository(CoreError),
}
impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Usage(s) | Self::Input(s) | Self::Validation(s) => f.write_str(s),
            Self::Repository(e) => write!(f, "{e}"),
        }
    }
}
impl From<CoreError> for CliError {
    fn from(e: CoreError) -> Self {
        Self::Repository(e)
    }
}

fn repository() -> Result<ProfileRepository, CliError> {
    let root = env::var_os("TH80_PROFILE_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share/th80-profiles"))
        })
        .ok_or_else(|| CliError::Input("HOME is not set; set TH80_PROFILE_DIR".into()))?;
    Ok(ProfileRepository::new(root))
}
fn value_after(args: &[String], flag: &str) -> Result<String, CliError> {
    let i = args
        .iter()
        .position(|v| v == flag)
        .ok_or_else(|| CliError::Usage(format!("missing {flag}")))?;
    args.get(i + 1)
        .cloned()
        .ok_or_else(|| CliError::Usage(format!("missing value after {flag}")))
}
fn values_after(args: &[String], flag: &str) -> Vec<String> {
    args.iter()
        .enumerate()
        .filter(|(_, value)| value.as_str() == flag)
        .filter_map(|(i, _)| args.get(i + 1).cloned())
        .collect()
}
fn optional(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|v| v == flag)
        .and_then(|i| args.get(i + 1))
        .cloned()
}
fn profile_id(args: &[String]) -> Result<ProfileId, CliError> {
    args.get(2)
        .map(ProfileId::new)
        .ok_or_else(|| CliError::Usage("missing PROFILE_ID".into()))
}
fn nested_profile_id(args: &[String]) -> Result<ProfileId, CliError> {
    args.get(3)
        .map(ProfileId::new)
        .ok_or_else(|| CliError::Usage("missing PROFILE_ID after subcommand".into()))
}
fn load(repo: &ProfileRepository, id: &ProfileId) -> Result<Profile, CliError> {
    Ok(repo.load(id)?)
}
fn save(repo: &ProfileRepository, profile: &Profile) -> Result<(), CliError> {
    repo.save(profile)?;
    Ok(())
}
fn parse_color(value: &str) -> Result<RgbColor, CliError> {
    let parts: Vec<_> = value.split(',').collect();
    if parts.len() != 3 {
        return Err(CliError::Input(
            "color must be R,G,B with channels 0..255".into(),
        ));
    }
    let parse = |s: &str| {
        s.parse::<u8>()
            .map_err(|_| CliError::Input("color channels must be integers 0..255".into()))
    };
    Ok(RgbColor {
        red: parse(parts[0])?,
        green: parse(parts[1])?,
        blue: parse(parts[2])?,
    })
}
fn parse_step(value: &str) -> Result<MacroStep, CliError> {
    let (kind, payload) = value
        .split_once(':')
        .ok_or_else(|| CliError::Input("step must be TYPE:VALUE".into()))?;
    match kind {
        "key-down" => Ok(MacroStep::KeyDown {
            key: payload.into(),
        }),
        "key-up" => Ok(MacroStep::KeyUp {
            key: payload.into(),
        }),
        "delay" => Ok(MacroStep::Delay {
            milliseconds: payload.parse().map_err(|_| {
                CliError::Input("delay must be an unsigned integer in milliseconds".into())
            })?,
        }),
        "text" => Ok(MacroStep::Text {
            value: payload.into(),
        }),
        _ => Err(CliError::Input(
            "step type must be key-down, key-up, delay or text".into(),
        )),
    }
}
fn print_json<T: serde::Serialize>(value: &T) -> Result<(), CliError> {
    println!(
        "{}",
        serde_json::to_string_pretty(value).map_err(|e| CliError::Input(e.to_string()))?
    );
    Ok(())
}

#[derive(serde::Serialize)]
struct ValidationEnvelope<'a> {
    schema_version: u32,
    valid: bool,
    device_compatibility: &'static str,
    diagnostics: &'a [Diagnostic],
}
#[derive(serde::Serialize)]
struct ErrorEnvelope<'a> {
    schema_version: u32,
    valid: bool,
    error_code: &'a str,
    message: &'a str,
}

fn validate_file(path: &str, json: bool) -> Result<(), CliError> {
    let bytes = std::fs::read(path)
        .map_err(|e| CliError::Input(format!("cannot read profile file: {e}")))?;
    let profile: Profile = match serde_json::from_slice(&bytes) {
        Ok(p) => p,
        Err(e) => {
            if json {
                print_json(&ErrorEnvelope {
                    schema_version: 1,
                    valid: false,
                    error_code: "PROFILE_JSON_INVALID",
                    message: &e.to_string(),
                })?;
            }
            return Err(CliError::Input(format!("invalid profile JSON: {e}")));
        }
    };
    let diagnostics = validate_profile(&profile);
    let compatibility = assess_device_compatibility(
        &profile,
        profile
            .device_model
            .unwrap_or(th80_core::DeviceKind::Th80Pro),
    );
    if json {
        print_json(&ValidationEnvelope {
            schema_version: 1,
            valid: diagnostics.is_empty(),
            device_compatibility: match compatibility.status {
                th80_core::validation::DeviceCompatibility::ModelMismatch => "model_mismatch",
                th80_core::validation::DeviceCompatibility::NotVerified => "not_verified",
            },
            diagnostics: &diagnostics,
        })?;
    } else if diagnostics.is_empty() {
        println!(
            "Valid local profile {} ({}); device compatibility: NOT VERIFIED",
            profile.name, profile.id.0
        );
    } else {
        for d in &diagnostics {
            eprintln!("{d}");
        }
    }
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(CliError::Validation(format!(
            "validation failed with {} issue(s)",
            diagnostics.len()
        )))
    }
}

fn run(args: &[String]) -> Result<(), CliError> {
    if args.is_empty() || args.iter().any(|a| a == "--help" || a == "-h") {
        println!("{USAGE}");
        return Ok(());
    }
    if args.first().map(String::as_str) != Some("profile") {
        return Err(CliError::Usage("expected 'profile' command".into()));
    }
    let command = args
        .get(1)
        .map(String::as_str)
        .ok_or_else(|| CliError::Usage("missing profile subcommand".into()))?;
    if command == "validate" {
        let path = args
            .get(2)
            .ok_or_else(|| CliError::Usage("usage: profile validate FILE.json".into()))?;
        return validate_file(path, optional(args, "--output").as_deref() == Some("json"));
    }
    let repo = repository()?;
    match command {
        "create" => {
            let mut p = Profile::new(value_after(args, "--name")?);
            if let Some(v) = optional(args, "--description") {
                p.description = v;
            }
            repo.create(&p)?;
            println!("Created local profile {} ({})", p.name, p.id.0);
        }
        "list" => {
            let items = repo.list()?;
            if optional(args, "--output").as_deref() == Some("json") {
                print_json(&items)?;
            } else {
                for p in items {
                    println!(
                        "{}\t{}\t{}",
                        p.id.0,
                        p.name,
                        p.device_model
                            .map(|m| format!("{m:?}"))
                            .unwrap_or_else(|| "unbound".into())
                    );
                }
            }
        }
        "show" => {
            let p = load(&repo, &profile_id(args)?)?;
            if optional(args, "--format").as_deref() == Some("text") {
                println!("Profile: {}\nID: {}\nSchema: {}\nProvenance: {:?}\nLayers: {}\nMacros: {}\nLighting: {:?}\nOptions: {}\nDevice compatibility: NOT VERIFIED",p.name,p.id.0,p.schema_version,p.provenance,p.configuration.keymap.layers.len(),p.configuration.macros.items.len(),p.configuration.lighting.mode,p.configuration.options.values.len());
            } else {
                print_json(&p)?;
            }
        }
        "rename" => {
            let id = profile_id(args)?;
            let mut p = load(&repo, &id)?;
            let name = value_after(args, "--name")?;
            if name.trim().is_empty() {
                return Err(CliError::Input("profile name must not be empty".into()));
            }
            p.name = name.trim().to_owned();
            save(&repo, &p)?;
            println!("Renamed local profile {}.", id.0);
        }
        "duplicate" => {
            let id = profile_id(args)?;
            let source = load(&repo, &id)?;
            let default_name = format!("{} copy", source.name);
            let name = optional(args, "--name").unwrap_or(default_name);
            if name.trim().is_empty() {
                return Err(CliError::Input("profile name must not be empty".into()));
            }
            let mut copy = Profile::new(name.trim().to_owned());
            copy.description = source.description;
            copy.device_model = source.device_model;
            copy.configuration = source.configuration;
            copy.metadata.tags = source.metadata.tags;
            copy.metadata.extensions = source.metadata.extensions;
            copy.extensions = source.extensions;
            repo.create(&copy)?;
            println!("Duplicated local profile {} as {}.", id.0, copy.id.0);
        }
        "add-layer" => {
            let id = profile_id(args)?;
            let mut p = load(&repo, &id)?;
            let layer_id = value_after(args, "--layer-id")?;
            if p.configuration
                .keymap
                .layers
                .iter()
                .any(|l| l.id == layer_id)
            {
                return Err(CliError::Input(format!("layer already exists: {layer_id}")));
            }
            p.configuration.keymap.layers.push(Layer {
                id: layer_id,
                name: value_after(args, "--name")?,
                positions: BTreeMap::new(),
            });
            save(&repo, &p)?;
            println!("Added layer to local profile {}.", id.0);
        }
        "rename-layer" => {
            let id = profile_id(args)?;
            let mut p = load(&repo, &id)?;
            let layer_id = value_after(args, "--layer-id")?;
            let layer = p
                .configuration
                .keymap
                .layers
                .iter_mut()
                .find(|layer| layer.id == layer_id)
                .ok_or_else(|| CliError::Input(format!("layer not found: {layer_id}")))?;
            layer.name = value_after(args, "--name")?;
            save(&repo, &p)?;
            println!("Renamed local layer {layer_id}.");
        }
        "remove-layer" => {
            let id = profile_id(args)?;
            let mut p = load(&repo, &id)?;
            let layer_id = value_after(args, "--layer-id")?;
            let before = p.configuration.keymap.layers.len();
            p.configuration
                .keymap
                .layers
                .retain(|layer| layer.id != layer_id);
            if p.configuration.keymap.layers.len() == before {
                return Err(CliError::Input(format!("layer not found: {layer_id}")));
            }
            save(&repo, &p)?;
            println!("Removed local layer {layer_id}.");
        }
        "set-key" => {
            let id = profile_id(args)?;
            let mut p = load(&repo, &id)?;
            let layer_id = value_after(args, "--layer")?;
            let position = value_after(args, "--position")?;
            let key = value_after(args, "--key")?;
            let l = p
                .configuration
                .keymap
                .layers
                .iter_mut()
                .find(|l| l.id == layer_id)
                .ok_or_else(|| CliError::Input(format!("layer not found: {layer_id}")))?;
            l.positions.insert(
                position,
                KeyAssignment {
                    action: Action::Key { code: key },
                },
            );
            save(&repo, &p)?;
            println!("Updated local key assignment; no device was changed.");
        }
        "set-action" => {
            let id = profile_id(args)?;
            let mut p = load(&repo, &id)?;
            let layer_id = value_after(args, "--layer")?;
            let position = value_after(args, "--position")?;
            let raw_action = value_after(args, "--action")?;
            let action: Action = serde_json::from_str(&raw_action)
                .map_err(|error| CliError::Input(format!("invalid action JSON: {error}")))?;
            let layer = p
                .configuration
                .keymap
                .layers
                .iter_mut()
                .find(|layer| layer.id == layer_id)
                .ok_or_else(|| CliError::Input(format!("layer not found: {layer_id}")))?;
            layer.positions.insert(position, KeyAssignment { action });
            save(&repo, &p)?;
            println!("Updated typed local action; no device was changed.");
        }
        "macro" => macro_command(&repo, args)?,
        "lighting" => lighting_command(&repo, args)?,
        "options" => options_command(&repo, args)?,
        "export" => {
            let id = profile_id(args)?;
            repo.export(&id, value_after(args, "--output")?)?;
            println!("Exported local profile; no device was changed.");
        }
        "import" => {
            let path = args
                .get(2)
                .ok_or_else(|| CliError::Usage("missing FILE.json".into()))?;
            let p = repo.import(path)?;
            println!("Imported local profile {} ({})", p.name, p.id.0);
        }
        "delete" => {
            let id = profile_id(args)?;
            repo.delete(&id)?;
            println!("Deleted local profile {}.", id.0);
        }
        _ => {
            return Err(CliError::Usage(format!(
                "unknown profile subcommand: {command}"
            )))
        }
    }
    Ok(())
}

fn macro_command(repo: &ProfileRepository, args: &[String]) -> Result<(), CliError> {
    let id = nested_profile_id(args)?;
    let mut p = load(repo, &id)?;
    let sub = args
        .get(2)
        .map(String::as_str)
        .ok_or_else(|| CliError::Usage("missing macro operation".into()))?;
    let mid = optional(args, "--macro-id");
    match sub {
        "add" => {
            let macro_id = value_after(args, "--macro-id")?;
            if p.configuration
                .macros
                .items
                .iter()
                .any(|m| m.id == macro_id)
            {
                return Err(CliError::Input(format!("macro already exists: {macro_id}")));
            }
            p.configuration.macros.items.push(MacroDefinition {
                id: macro_id,
                name: value_after(args, "--name")?,
                steps: vec![],
            });
            save(repo, &p)?;
        }
        "list" => {
            for m in &p.configuration.macros.items {
                println!("{}\t{}\t{} step(s)", m.id, m.name, m.steps.len());
            }
            return Ok(());
        }
        "show" => {
            let m = p
                .configuration
                .macros
                .items
                .iter()
                .find(|m| Some(&m.id) == mid.as_ref())
                .ok_or_else(|| CliError::Input("macro not found".into()))?;
            print_json(m)?;
            return Ok(());
        }
        "rename" => {
            let m = p
                .configuration
                .macros
                .items
                .iter_mut()
                .find(|m| Some(&m.id) == mid.as_ref())
                .ok_or_else(|| CliError::Input("macro not found".into()))?;
            m.name = value_after(args, "--name")?;
            save(repo, &p)?;
        }
        "remove" => {
            let before = p.configuration.macros.items.len();
            p.configuration
                .macros
                .items
                .retain(|m| Some(&m.id) != mid.as_ref());
            if before == p.configuration.macros.items.len() {
                return Err(CliError::Input("macro not found".into()));
            }
            save(repo, &p)?;
        }
        "step-add" => {
            let steps = values_after(args, "--step");
            if steps.is_empty() {
                return Err(CliError::Usage("step-add requires at least one --step TYPE:VALUE (repeat --step for a balanced sequence)".into()));
            }
            let parsed_steps = steps
                .iter()
                .map(|step| parse_step(step))
                .collect::<Result<Vec<_>, _>>()?;
            let m = p
                .configuration
                .macros
                .items
                .iter_mut()
                .find(|m| Some(&m.id) == mid.as_ref())
                .ok_or_else(|| CliError::Input("macro not found".into()))?;
            m.steps.extend(parsed_steps);
            save(repo, &p)?;
        }
        "step-replace" => {
            let index: usize = value_after(args, "--index")?
                .parse()
                .map_err(|_| CliError::Input("index must be a non-negative integer".into()))?;
            let step = parse_step(&value_after(args, "--step")?)?;
            let m = p
                .configuration
                .macros
                .items
                .iter_mut()
                .find(|m| Some(&m.id) == mid.as_ref())
                .ok_or_else(|| CliError::Input("macro not found".into()))?;
            if index >= m.steps.len() {
                return Err(CliError::Input("macro step index out of range".into()));
            }
            m.steps[index] = step;
            save(repo, &p)?;
        }
        "step-remove" => {
            let index: usize = value_after(args, "--index")?
                .parse()
                .map_err(|_| CliError::Input("index must be a non-negative integer".into()))?;
            let m = p
                .configuration
                .macros
                .items
                .iter_mut()
                .find(|m| Some(&m.id) == mid.as_ref())
                .ok_or_else(|| CliError::Input("macro not found".into()))?;
            if index >= m.steps.len() {
                return Err(CliError::Input("macro step index out of range".into()));
            }
            m.steps.remove(index);
            save(repo, &p)?;
        }
        "step-move" => {
            let from: usize = value_after(args, "--from")?
                .parse()
                .map_err(|_| CliError::Input("from index must be a non-negative integer".into()))?;
            let to: usize = value_after(args, "--to")?
                .parse()
                .map_err(|_| CliError::Input("to index must be a non-negative integer".into()))?;
            let m = p
                .configuration
                .macros
                .items
                .iter_mut()
                .find(|m| Some(&m.id) == mid.as_ref())
                .ok_or_else(|| CliError::Input("macro not found".into()))?;
            if from >= m.steps.len() || to >= m.steps.len() {
                return Err(CliError::Input("macro step index out of range".into()));
            }
            m.steps.swap(from, to);
            save(repo, &p)?;
        }
        _ => return Err(CliError::Usage(format!("unknown macro operation: {sub}"))),
    }
    println!("Updated local macro data; no device was changed.");
    Ok(())
}

fn lighting_command(repo: &ProfileRepository, args: &[String]) -> Result<(), CliError> {
    let id = nested_profile_id(args)?;
    let mut p = load(repo, &id)?;
    let sub = args
        .get(2)
        .map(String::as_str)
        .ok_or_else(|| CliError::Usage("missing lighting operation".into()))?;
    match sub {
        "show" => {
            print_json(&p.configuration.lighting)?;
            return Ok(());
        }
        "set" => {
            if let Some(v) = optional(args, "--enabled") {
                p.configuration.lighting.enabled = Some(
                    v.parse()
                        .map_err(|_| CliError::Input("--enabled must be true or false".into()))?,
                );
            }
            if let Some(v) = optional(args, "--mode") {
                p.configuration.lighting.mode = Some(match v.as_str() {
                    "static" => LightingMode::Static,
                    "breathing" => LightingMode::Breathing,
                    "wave" => LightingMode::Wave,
                    "reactive" => LightingMode::Reactive,
                    _ if v.starts_with("custom:") => LightingMode::Custom(v[7..].into()),
                    _ => return Err(CliError::Input("unknown generic lighting mode".into())),
                });
            }
            if let Some(v) = optional(args, "--brightness") {
                p.configuration.lighting.brightness = Some(
                    v.parse()
                        .map_err(|_| CliError::Input("brightness must be 0..100".into()))?,
                );
            }
            if let Some(v) = optional(args, "--speed") {
                p.configuration.lighting.speed = Some(
                    v.parse()
                        .map_err(|_| CliError::Input("speed must be 0..100".into()))?,
                );
            }
            if let Some(v) = optional(args, "--color") {
                p.configuration.lighting.color = Some(parse_color(&v)?);
            }
            save(repo, &p)?;
        }
        "key-set" => {
            let pos = value_after(args, "--position")?;
            let color = parse_color(&value_after(args, "--color")?)?;
            if let Some(k) = p
                .configuration
                .lighting
                .per_key
                .iter_mut()
                .find(|k| k.position == pos)
            {
                k.color = color;
            } else {
                p.configuration.lighting.per_key.push(PerKeyColor {
                    position: pos,
                    color,
                });
            }
            save(repo, &p)?;
        }
        "key-remove" => {
            let pos = value_after(args, "--position")?;
            let n = p.configuration.lighting.per_key.len();
            p.configuration
                .lighting
                .per_key
                .retain(|k| k.position != pos);
            if n == p.configuration.lighting.per_key.len() {
                return Err(CliError::Input("per-key color not found".into()));
            }
            save(repo, &p)?;
        }
        "param-set" => {
            let key = value_after(args, "--key")?;
            let raw = value_after(args, "--value")?;
            let value: serde_json::Value = serde_json::from_str(&raw)
                .map_err(|e| CliError::Input(format!("--value must be valid JSON: {e}")))?;
            p.configuration
                .lighting
                .effect_parameters
                .insert(key, value);
            save(repo, &p)?;
        }
        "param-remove" => {
            let key = value_after(args, "--key")?;
            if p.configuration
                .lighting
                .effect_parameters
                .remove(&key)
                .is_none()
            {
                return Err(CliError::Input("lighting parameter not found".into()));
            }
            save(repo, &p)?;
        }
        _ => {
            return Err(CliError::Usage(format!(
                "unknown lighting operation: {sub}"
            )))
        }
    }
    println!("Updated generic local lighting preferences; firmware support is UNKNOWN.");
    Ok(())
}

fn options_command(repo: &ProfileRepository, args: &[String]) -> Result<(), CliError> {
    let id = nested_profile_id(args)?;
    let mut p = load(repo, &id)?;
    let sub = args
        .get(2)
        .map(String::as_str)
        .ok_or_else(|| CliError::Usage("missing options operation".into()))?;
    match sub {
        "list" => {
            print_json(&p.configuration.options.values)?;
            return Ok(());
        }
        "set" => {
            let key = value_after(args, "--key")?;
            let value: serde_json::Value = serde_json::from_str(&value_after(args, "--value")?)
                .map_err(|e| CliError::Input(format!("--value must be valid JSON: {e}")))?;
            p.configuration.options.values.insert(key, value);
            save(repo, &p)?;
        }
        "remove" => {
            let key = value_after(args, "--key")?;
            if p.configuration.options.values.remove(&key).is_none() {
                return Err(CliError::Input(format!("option not found: {key}")));
            }
            save(repo, &p)?;
        }
        _ => return Err(CliError::Usage(format!("unknown options operation: {sub}"))),
    }
    println!("Updated application-owned local options; device semantics remain UNKNOWN.");
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            let code = match e {
                CliError::Validation(_) => 2,
                CliError::Input(_) => 3,
                CliError::Repository(_) => 4,
                CliError::Usage(_) => 64,
            };
            ExitCode::from(code)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn help_is_offline_and_mentions_profile_commands() {
        assert!(USAGE.contains("Offline only"));
        assert!(USAGE.contains("macro step-add"));
        assert!(USAGE.contains("profile rename"));
        assert!(USAGE.contains("profile duplicate"));
        assert!(USAGE.contains("profile set-action"));
        assert!(USAGE.contains("profile rename-layer"));
        assert!(USAGE.contains("macro step-move"));
    }
    #[test]
    fn missing_arguments_are_reported() {
        let args = vec!["profile".into(), "create".into()];
        assert!(matches!(run(&args), Err(CliError::Usage(_))));
    }
    #[test]
    fn invalid_profile_identifier_is_rejected() {
        let repo = ProfileRepository::new("/tmp/th80-cli-invalid-id-test");
        assert!(matches!(
            repo.load(&ProfileId::new("../bad")),
            Err(CoreError::InvalidId(_))
        ));
    }
}
