use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde::de::{self, Deserializer, MapAccess, Visitor};

const CONFIG_FILE_NAME: &str = "minter.config.json";
const DEFAULT_SPECS_DIR: &str = "specs/";
const DEFAULT_TESTS_DIR: &str = "tests/";

/// Deserialized tests field: accepts a string or an array of strings.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum TestsDirRaw {
    Single(String),
    Multiple(Vec<String>),
}

/// Resolved project configuration.
#[derive(Debug, Clone)]
pub struct ProjectConfig {
    pub specs: PathBuf,
    pub tests: Vec<PathBuf>,
}

/// Load and resolve project configuration from the working directory.
///
/// Resolution:
/// 1. If `minter.config.json` exists, parse it, validate explicitly-set directory
///    paths exist on disk, and fill in defaults for missing fields.
/// 2. If no config file exists, use conventions (`specs/` and `tests/`).
///
/// Default paths are NOT validated at load time — they are only validated by
/// the command that actually uses them. This allows `minter validate` (which
/// only needs specs) to succeed even when `tests/` doesn't exist.
pub fn load_config(working_dir: &Path) -> Result<ProjectConfig, String> {
    let config_path = working_dir.join(CONFIG_FILE_NAME);

    if !config_path.exists() {
        // No config file: use conventions, no validation at this stage
        return Ok(ProjectConfig {
            specs: working_dir.join(DEFAULT_SPECS_DIR),
            tests: vec![working_dir.join(DEFAULT_TESTS_DIR)],
        });
    }

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("cannot read {}: {}", CONFIG_FILE_NAME, e))?;

    let raw: RawConfigWithPresence =
        serde_json::from_str(&content).map_err(|e| format_parse_error(&e))?;

    // Resolve specs path
    let specs_str = raw
        .specs
        .clone()
        .unwrap_or_else(|| DEFAULT_SPECS_DIR.to_string());
    let specs_path = working_dir.join(&specs_str);

    // Validate explicitly-configured specs dir exists
    if raw.specs_present && !specs_path.exists() {
        return Err(format!("specs directory '{}' does not exist", specs_str));
    }

    // Resolve tests paths
    let tests_strs = match raw.tests {
        Some(TestsDirRaw::Single(ref s)) => vec![s.clone()],
        Some(TestsDirRaw::Multiple(ref v)) => v.clone(),
        None => vec![DEFAULT_TESTS_DIR.to_string()],
    };

    let mut tests_paths = Vec::new();
    for t in &tests_strs {
        let p = working_dir.join(t);
        // Validate explicitly-configured tests dirs exist
        if raw.tests_present && !p.exists() {
            return Err(format!("test directory '{}' does not exist", t));
        }
        tests_paths.push(p);
    }

    Ok(ProjectConfig {
        specs: specs_path,
        tests: tests_paths,
    })
}

/// Validate that the specs directory actually exists on disk.
/// Call this from commands that need the specs path.
pub fn require_specs(config: &ProjectConfig) -> Result<(), String> {
    if !config.specs.exists() {
        // Extract the dir name for the error message
        let dir_name = config
            .specs
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("specs");
        return Err(format!("specs directory '{}' does not exist", dir_name));
    }
    Ok(())
}

/// Validate that all test directories actually exist on disk.
/// Call this from commands that need the tests paths.
pub fn require_tests(config: &ProjectConfig) -> Result<(), String> {
    for p in &config.tests {
        if !p.exists() {
            let dir_name = p.file_name().and_then(|n| n.to_str()).unwrap_or("tests");
            return Err(format!("test directory '{}' does not exist", dir_name));
        }
    }
    Ok(())
}

// ── Custom error formatting ──────────────────────────────

/// Format serde parse errors to include field names and be human-readable.
fn format_parse_error(err: &serde_json::Error) -> String {
    let msg = err.to_string();

    // Check for unknown field errors
    if msg.contains("unknown field") {
        return format!("{}: {}", CONFIG_FILE_NAME, msg);
    }

    // Always prefix with "invalid configuration" so the error message
    // contains "invalid" for JSON syntax and type errors alike.
    format!("{}: invalid configuration: {}", CONFIG_FILE_NAME, msg)
}

// ── Custom deserialization for better error messages ──────

/// Config struct that tracks which fields were explicitly present in JSON.
#[derive(Debug)]
struct RawConfigWithPresence {
    specs: Option<String>,
    specs_present: bool,
    tests: Option<TestsDirRaw>,
    tests_present: bool,
}

impl<'de> Deserialize<'de> for RawConfigWithPresence {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ConfigVisitor;

        impl<'de> Visitor<'de> for ConfigVisitor {
            type Value = RawConfigWithPresence;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a JSON object with optional 'specs' and 'tests' fields")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut specs: Option<String> = None;
                let mut specs_present = false;
                let mut tests: Option<TestsDirRaw> = None;
                let mut tests_present = false;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "specs" => {
                            specs_present = true;
                            specs = Some(
                                map.next_value::<String>()
                                    .map_err(|_| de::Error::custom("'specs' must be a string"))?,
                            );
                        }
                        "tests" => {
                            tests_present = true;
                            tests = Some(map.next_value::<TestsDirRaw>().map_err(|_| {
                                de::Error::custom("'tests' must be a string or array of strings")
                            })?);
                        }
                        other => {
                            return Err(de::Error::custom(format!(
                                "unknown field '{}' in configuration",
                                other
                            )));
                        }
                    }
                }

                Ok(RawConfigWithPresence {
                    specs,
                    specs_present,
                    tests,
                    tests_present,
                })
            }
        }

        deserializer.deserialize_map(ConfigVisitor)
    }
}
