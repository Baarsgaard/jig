use crate::config::{self, GitHooksRawConfig, RawConfig};
use clap::Args;
use color_eyre::eyre::{eyre, Result, WrapErr};
use color_eyre::owo_colors::OwoColorize;
use inquire::{Confirm, CustomType, Password, Select, Text};
use std::fs;
use std::path::PathBuf;
use std::{env, process::Command};
use url::Url;

use super::Hooks;

#[derive(Args, Debug)]
pub struct InitConfig {
    #[arg(short, long)]
    all: bool,
}

impl InitConfig {
    pub fn init(&self) -> Result<String> {
        let global_config = config::config_file();
        let local_config = config::workspace_config_file();
        let config_file_input = Select::new(
            "Where to save config",
            vec![
                global_config.to_string_lossy(),
                local_config.to_string_lossy(),
            ],
        )
        .prompt()?;
        let config_file = if config_file_input == global_config.to_string_lossy() {
            config::config_file()
        } else {
            config::workspace_config_file()
        };

        let jira_url = InitConfig::jira_url()?;
        let mut new_git_hooks = GitHooksRawConfig {
            allow_branch_missing_issue_key: Some(false),
        };
        let mut new_cfg = RawConfig {
            jira_url,
            user_login: None,
            api_token: None,
            pat_token: None,
            jira_timeout_seconds: Some(10),
            issue_query: String::from("assignee = currentUser() ORDER BY updated DESC"),
            retry_query: String::from("reporter = currentUser() ORDER BY updated DESC"),
            always_short_branch_names: Some(false),
            max_query_results: Some(100),
            enable_comment_prompts: Some(false),
            one_transition_auto_move: Some(false),
            #[cfg(feature = "cloud")]
            inclusive_filters: Some(true),
            git_hooks: Some(new_git_hooks.clone()),
        };

        InitConfig::set_credentials(&mut new_cfg)?;

        if Confirm::new("Install Git hook commit-msg")
            .with_help_message(
                "Auto prepends issue key in branch name to commit messages without it",
            )
            .with_default(true)
            .prompt()?
        {
            match Hooks::install(Hooks { force: false }) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("{}", format!("Failed to install hook with error: {}\ncd to repository and install with:\njig hook\n", e).bright_red());
                    println!()
                }
            }
        }

        if !self.all {
            return InitConfig::write_config(&new_cfg, config_file)
                .wrap_err("Failed to write partial config");
        }

        // Text prompts
        new_cfg.issue_query = Text::new("Issue query")
            .with_default(&new_cfg.issue_query)
            .with_help_message(
                "Try using existing filters: 'filter=<filterID> OR filter=<filterID>'",
            )
            .prompt()?;
        new_cfg.retry_query = Text::new("Retry query")
            .with_default(&new_cfg.retry_query)
            .prompt()?;
        new_cfg.max_query_results = Some(
            CustomType::<u32>::new("Maximum query results")
                .with_help_message("Lower is faster in case of large queries (max 1500)")
                .with_default(new_cfg.max_query_results.unwrap())
                .prompt()?,
        );
        new_cfg.jira_timeout_seconds = Some(
            CustomType::new("Rest Timeout (Seconds)")
                .with_default(10u64)
                .with_help_message("How long to wait on server to respond")
                .prompt()?,
        );

        // Boolean prompts
        new_cfg.always_short_branch_names = Some(
            Confirm::new("Always omit summary from branch names")
                .with_default(new_cfg.always_short_branch_names.unwrap())
                .with_help_message("Invert setting with 'branch --short'")
                .prompt()?,
        );
        new_cfg.enable_comment_prompts = Some(
            Confirm::new("Always prompt for comments (Worklog)")
                .with_default(new_cfg.enable_comment_prompts.unwrap())
                .with_help_message("Override with 'log -c \"\"'")
                .prompt()?,
        );
        new_cfg.one_transition_auto_move = Some(
            Confirm::new("Automatically pick if there is only one option")
                .with_default(new_cfg.one_transition_auto_move.unwrap())
                .prompt()?,
        );
        #[cfg(feature = "cloud")]
        {
            new_cfg.inclusive_filters = Some(
                Confirm::new("Join using 'OR' instead of 'AND'")
                    .with_default(new_cfg.inclusive_filters.unwrap())
                    .with_help_message("filter=10001 OR filter=10002")
                    .prompt()?,
            );
        }

        new_git_hooks.allow_branch_missing_issue_key = Some(
            Confirm::new("Githook: Skip 'branche contains Issue Key' checks")
                .with_help_message("Prompts with issue select when branch is missing an Issue key")
                .with_default(false)
                .prompt()?,
        );

        new_cfg.git_hooks = Some(new_git_hooks);

        InitConfig::write_config(&new_cfg, config_file).wrap_err("Failed to write full config")
    }

    fn write_config(cfg: &RawConfig, path: PathBuf) -> Result<String> {
        let str_cfg = toml::to_string(&cfg).wrap_err("Failed to serialize new Config file")?;

        let dir = match path.parent() {
            Some(parent_dir) => parent_dir,
            None => Err(eyre!("Unable to find parent directory"))?,
        };
        fs::create_dir_all(dir).wrap_err("Unable to create config directory")?;

        fs::write(path.clone().into_os_string(), str_cfg)
            .wrap_err("Failed to write config file to")?;

        Ok(format!("Overwrote config: {}", path.to_str().unwrap()))
    }

    fn jira_url() -> Result<String> {
        let url_input = Text::new("jira_url")
            .with_help_message("Type or paste any url/FQDN. Scheme defaults to HTTPS if missing")
            .prompt()?;

        let parsed_url = if !url_input.starts_with("http") {
            Url::parse(&format!("https://{}", url_input)).wrap_err("Unable to parse url")?
        } else {
            Url::parse(&url_input).wrap_err("Unable to parse url")?
        };

        Ok(format!(
            "{}://{}",
            parsed_url.scheme(),
            parsed_url.host_str().unwrap()
        ))
    }

    fn set_credentials(icfg: &mut RawConfig) -> Result<()> {
        let auth_url = if icfg.jira_url.contains("atlassian.net") {
            let url = String::from("https://id.atlassian.com/manage-profile/security/api-tokens");
            icfg.api_token = Some(String::default());
            url
        } else {
            let url = format!("{}/secure/ViewProfile.jspa", icfg.jira_url);
            icfg.pat_token = Some(String::default());
            url
        };

        match env::var("BROWSER") {
            Ok(browser) => {
                let _ = Command::new(browser).args([auth_url]).spawn();
            }
            Err(_) => {
                println!(
                    "Unable to find BROWSER env var, get token here:\n{}",
                    auth_url
                );
            }
        };

        let token = Password::new("Auth token")
            .without_confirmation()
            .prompt()
            .wrap_err("Missing authentication token input")?;

        if icfg.api_token.is_some() {
            icfg.api_token = Some(token);
            icfg.user_login = Some(
                Text::new("Username")
                    .prompt()
                    .wrap_err("Username prompt failed")?,
            );
        } else {
            icfg.pat_token = Some(token);
        }

        Ok(())
    }
}
