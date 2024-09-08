use color_eyre::{
    eyre::{eyre, Result, WrapErr},
    Section,
};
use etcetera::base_strategy::{choose_base_strategy, BaseStrategy};
use jira::{Credential, JiraClientConfig};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, sync::OnceLock};
use toml::from_str;

// Proof of concept
static CONFIG_FILE: OnceLock<PathBuf> = OnceLock::new();

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RawConfig {
    /// server url or <domain>.atlassian.net
    pub jira_url: String,
    /// Primary query to use when fetching issues.
    pub issue_query: String,
    /// email or username used as login.
    pub user_login: Option<String>,
    /// API token for Cloud.
    pub api_token: Option<String>,
    /// Personal access token for Server.
    pub pat_token: Option<String>,
    /// How long to wait for a response.
    pub jira_timeout_seconds: Option<u64>,
    /// Accept invalid TLS certificates.
    pub tls_accept_invalid_certs: Option<bool>,
    /// Max number of issues to fetch.
    /// max 1500.
    pub max_query_results: Option<u32>,
    /// Prompt for comment if missing on worklogs.
    pub enable_comment_prompts: Option<bool>,
    /// When moving issue, skip prompt if there is only one option.
    pub one_transition_auto_move: Option<bool>,
    #[cfg(feature = "cloud")]
    /// Combine filters using `OR` instead of `AND`.
    pub inclusive_filters: Option<bool>,
    /// Git hooks specific config section.
    pub git_hooks: Option<GitHooksRawConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GitHooksRawConfig {
    pub allow_branch_missing_issue_key: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct GitHooksConfig {
    pub allow_branch_missing_issue_key: bool,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub issue_query: String,
    pub enable_comment_prompts: Option<bool>,
    pub one_transition_auto_move: Option<bool>,
    #[cfg(feature = "cloud")]
    pub inclusive_filters: Option<bool>,
    pub jira_cfg: JiraClientConfig,
    pub hooks_cfg: GitHooksConfig,
}

impl Config {
    pub fn load() -> Result<Config> {
        let global_raw_config =
            fs::read_to_string(config_file()).wrap_err("Config load error: global config");
        let local_raw_config = fs::read_to_string(workspace_config_file())
            .wrap_err("Config load error: workspace config");

        let global_config: Result<toml::Value> = global_raw_config
            .and_then(|file| from_str(&file).wrap_err("Config load error: Bad global config"));
        let local_config: Result<toml::Value> = local_raw_config
            .and_then(|file| from_str(&file).wrap_err("Config load error: Bad workspace config"));

        let mut cfg: RawConfig = match (global_config, local_config) {
            (Ok(global), Ok(local)) => merge_toml_values(global, local, 3)
                .try_into::<RawConfig>()
                .wrap_err("Config load error: Bad configs"),

            (Ok(cfg), Err(_)) | (Err(_), Ok(cfg)) => cfg
                .try_into::<RawConfig>()
                .wrap_err("Config load error: Bad config"),
            (Err(e), Err(_)) => Err(e).wrap_err("Config load error"),
        }
        .with_suggestion(|| "Create or overwrite config with: jig init")?;

        if cfg.api_token.is_some() && cfg.user_login.is_none() {
            Err(eyre!("'user_login' missing, required with api_token"))
                .wrap_err("Config load error: Bad config")?
        } else if cfg.api_token.is_none() && cfg.user_login.is_some() {
            Err(eyre!("'api_token' missing, required with user_login"))
                .wrap_err("Config load error: Bad config")?
        }

        let mut url = cfg.jira_url.clone();
        if !url.starts_with("http") {
            url = String::from("https://") + &url;
        };
        if url.ends_with('/') {
            url.pop();
        }
        cfg.jira_url = url;

        Ok(Config::from(cfg))
    }
}

impl From<Option<GitHooksRawConfig>> for GitHooksConfig {
    fn from(value: Option<GitHooksRawConfig>) -> Self {
        if let Some(cfg) = value {
            GitHooksConfig {
                allow_branch_missing_issue_key: cfg.allow_branch_missing_issue_key.unwrap_or(false),
            }
        } else {
            GitHooksConfig {
                allow_branch_missing_issue_key: false,
            }
        }
    }
}

impl From<RawConfig> for Config {
    fn from(cfg: RawConfig) -> Self {
        let credential = if let Some(pat) = cfg.pat_token {
            Credential::PersonalAccessToken(pat)
        } else if let Some(api_token) = cfg.api_token {
            Credential::ApiToken {
                login: cfg.user_login.expect("user_login is unset."),
                token: api_token,
            }
        } else {
            Credential::Anonymous
        };

        Config {
            issue_query: cfg.issue_query,
            enable_comment_prompts: cfg.enable_comment_prompts,
            one_transition_auto_move: cfg.one_transition_auto_move,
            #[cfg(feature = "cloud")]
            inclusive_filters: cfg.inclusive_filters,
            jira_cfg: JiraClientConfig {
                credential,
                max_query_results: cfg.max_query_results.unwrap_or(50u32),
                url: cfg.jira_url,
                timeout: cfg.jira_timeout_seconds.unwrap_or(10u64),
                tls_accept_invalid_certs: cfg.tls_accept_invalid_certs.unwrap_or(false),
            },
            hooks_cfg: GitHooksConfig::from(cfg.git_hooks),
        }
    }
}

pub fn config_file() -> PathBuf {
    CONFIG_FILE
        .get_or_init(|| config_dir().join("config.toml"))
        .to_owned()
}

pub fn workspace_config_file() -> PathBuf {
    find_workspace().0.join(".jig.toml")
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

/// Search parent folders from PWD and returns the first directory that
/// contains `.git`. If no directory is found, return current location.
/// the boolean is true if the workspace dir is a repository.
pub fn find_workspace() -> (PathBuf, bool) {
    let current_dir = std::env::current_dir().expect("unable to determine current directory");
    for ancestor in current_dir.ancestors() {
        if ancestor.join(".git").exists() {
            return (ancestor.to_owned(), true);
        }
    }
    (current_dir, false)
}

pub fn merge_toml_values(left: toml::Value, right: toml::Value, merge_depth: usize) -> toml::Value {
    use toml::Value;

    fn get_name(v: &Value) -> Option<&str> {
        v.get("name").and_then(Value::as_str)
    }

    match (left, right) {
        (Value::Array(mut left_items), Value::Array(right_items)) => {
            // The top-level arrays should be merged but nested arrays should
            // act as overrides.
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
