/// Shamelessly lifted from Helix-editor/helix/helix-loader/src/lib.rs
use etcetera::base_strategy::{choose_base_strategy, BaseStrategy};
use serde::Deserialize;
use std::fmt::Display;
use std::fs;
use std::io::Error as IOError;
use std::path::PathBuf;
use std::sync::OnceLock;
use toml::de::Error as TomlError;

// Proof of concept
static CONFIG_FILE: OnceLock<PathBuf> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub jira_url: String,
    pub user_login: String,
    pub api_token: String,
    pub default_issue_key: Option<String>,
    /// This overrides the content of the retry query.
    pub retry_query_override: Option<String>,
    pub always_confirm_date: Option<bool>,
    pub always_short_branch_names: Option<bool>,
    pub max_query_results: Option<u32>,
    pub skip_branch_confirmation: Option<bool>,
    // pub issue_transitions: Option<Vec<String>>,
}

impl Config {
    pub fn load() -> Result<Config, ConfigLoadError> {
        let global_raw_config = fs::read_to_string(config_file()).map_err(ConfigLoadError::Error);
        let local_raw_config =
            fs::read_to_string(workspace_config_file()).map_err(ConfigLoadError::Error);

        let global_config: Result<toml::Value, ConfigLoadError> = global_raw_config
            .and_then(|file| toml::from_str(&file).map_err(ConfigLoadError::BadConfig));
        let local_config: Result<toml::Value, ConfigLoadError> = local_raw_config
            .and_then(|file| toml::from_str(&file).map_err(ConfigLoadError::BadConfig));

        let result: Result<Config, ConfigLoadError> = match (global_config, local_config) {
            (Ok(global), Ok(local)) => merge_toml_values(global, local, 3)
                .try_into::<Config>()
                .map_err(ConfigLoadError::BadConfig),
            (Ok(cfg), Err(_)) | (Err(_), Ok(cfg)) => {
                cfg.try_into::<Config>().map_err(ConfigLoadError::BadConfig)
            }
            (Err(e), Err(_)) => Err(e),
        };

        result
    }
}

#[derive(Debug)]
pub enum ConfigLoadError {
    BadConfig(TomlError),
    Error(IOError),
}

impl Default for ConfigLoadError {
    fn default() -> ConfigLoadError {
        ConfigLoadError::Error(IOError::new(std::io::ErrorKind::NotFound, "placeholder"))
    }
}

impl Display for ConfigLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigLoadError::BadConfig(err) => err.fmt(f),
            ConfigLoadError::Error(err) => err.fmt(f),
        }
    }
}

pub fn config_file() -> PathBuf {
    CONFIG_FILE
        .get_or_init(|| config_dir().join("config.toml"))
        .to_owned()
}

pub fn workspace_config_file() -> PathBuf {
    find_workspace().join(".jit.toml")
}

fn config_dir() -> PathBuf {
    let strategy = choose_base_strategy().expect("Unable to find the config directory!");
    let mut path = strategy.config_dir();
    path.push("jit");
    path
}

#[allow(dead_code)]
pub fn cache_dir() -> PathBuf {
    let strategy = choose_base_strategy().expect("Unable to find the config directory!");
    let mut path = strategy.cache_dir();
    path.push("jit");
    path
}

/// Search parent folders from PWD
/// and returns the first directory that contains `.git`.
pub fn find_workspace() -> PathBuf {
    let current_dir = std::env::current_dir().expect("unable to determine current directory");
    for ancestor in current_dir.ancestors() {
        if ancestor.join(".git").exists() {
            return ancestor.to_owned();
        }
    }
    current_dir
}

pub fn merge_toml_values(left: toml::Value, right: toml::Value, merge_depth: usize) -> toml::Value {
    use toml::Value;

    fn get_name(v: &Value) -> Option<&str> {
        v.get("name").and_then(Value::as_str)
    }

    match (left, right) {
        (Value::Array(mut left_items), Value::Array(right_items)) => {
            // The top-level arrays should be merged but nested arrays should
            // act as overrides. For the `languages.toml` config, this means
            // that you can specify a sub-set of languages in an overriding
            // `languages.toml` but that nested arrays like Language Server
            // arguments are replaced instead of merged.
            if merge_depth > 0 {
                left_items.reserve(right_items.len());
                for rvalue in right_items {
                    let lvalue = get_name(&rvalue)
                        .and_then(|rname| {
                            left_items.iter().position(|v| get_name(v) == Some(rname))
                        })
                        .map(|lpos| left_items.remove(lpos));
                    let mvalue = match lvalue {
                        Some(lvalue) => merge_toml_values(lvalue, rvalue, merge_depth - 1),
                        None => rvalue,
                    };
                    left_items.push(mvalue);
                }
                Value::Array(left_items)
            } else {
                Value::Array(right_items)
            }
        }
        (Value::Table(mut left_map), Value::Table(right_map)) => {
            if merge_depth > 0 {
                for (rname, rvalue) in right_map {
                    match left_map.remove(&rname) {
                        Some(lvalue) => {
                            let merged_value = merge_toml_values(lvalue, rvalue, merge_depth - 1);
                            left_map.insert(rname, merged_value);
                        }
                        None => {
                            left_map.insert(rname, rvalue);
                        }
                    }
                }
                Value::Table(left_map)
            } else {
                Value::Table(right_map)
            }
        }
        // Catch everything else we didn't handle, and use the right value
        (_, value) => value,
    }
}
