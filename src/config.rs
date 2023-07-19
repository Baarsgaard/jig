/// Shamelessly lifted from Helix-editor/helix/helix-loader/src/lib.rs
use anyhow::{anyhow, Context, Result};
use etcetera::base_strategy::{choose_base_strategy, BaseStrategy};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;
use toml::from_str;

// Proof of concept
static CONFIG_FILE: OnceLock<PathBuf> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub jira_url: String,
    pub user_login: Option<String>,
    pub api_token: Option<String>,
    pub pat_token: Option<String>,
    pub issue_query: Option<String>,
    pub retry_query: Option<String>,
    pub always_confirm_date: Option<bool>,
    pub always_short_branch_names: Option<bool>,
    pub max_query_results: Option<u32>,
    pub enable_comment_prompts: Option<bool>,
    pub one_transition_auto_move: Option<bool>,
}

impl Config {
    pub fn load() -> Result<Config> {
        let global_raw_config = fs::read_to_string(config_file()).context("Config load error");
        let local_raw_config =
            fs::read_to_string(workspace_config_file()).context("Config load error");

        let global_config: Result<toml::Value> = global_raw_config
            .and_then(|file| from_str(&file).context("Config load error: Bad config"));
        let local_config: Result<toml::Value> = local_raw_config
            .and_then(|file| from_str(&file).context("Config load error: Bad config"));

        let cfg: Config = match (global_config, local_config) {
            (Ok(global), Ok(local)) => merge_toml_values(global, local, 3)
                .try_into::<Config>()
                .context("Config load error: Bad configs"),

            (Ok(cfg), Err(_)) | (Err(_), Ok(cfg)) => cfg
                .try_into::<Config>()
                .context("Config load error: Bad config"),
            (Err(e), Err(_)) => Err(e).context("Config load error"),
        }?;

        if cfg.pat_token.is_none() && cfg.api_token.is_none() {
            Err(anyhow!("Neither api_token nor pat_token specified"))
                .context("Config load error: Bad config")?
        } else if cfg.api_token.is_some() && cfg.user_login.is_none() {
            Err(anyhow!("'user_login' missing, required with api_token"))
                .context("Config load error: Bad config")?
        } else if cfg.api_token.is_none() && cfg.user_login.is_some() {
            Err(anyhow!("'api_token' missing, required with user_login"))
                .context("Config load error: Bad config")?
        }

        Ok(cfg)
    }
}

pub fn config_file() -> PathBuf {
    CONFIG_FILE
        .get_or_init(|| config_dir().join("config.toml"))
        .to_owned()
}

pub fn workspace_config_file() -> PathBuf {
    find_workspace().join(".jig.toml")
}

fn config_dir() -> PathBuf {
    let strategy = choose_base_strategy().expect("Unable to find the config directory!");
    let mut path = strategy.config_dir();
    path.push("jig");
    path
}

#[allow(dead_code)]
pub fn cache_dir() -> PathBuf {
    let strategy = choose_base_strategy().expect("Unable to find the config directory!");
    let mut path = strategy.cache_dir();
    path.push("jig");
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
