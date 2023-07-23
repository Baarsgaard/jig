use crate::config::{self, Config};
use anyhow::{Context, Result};
use clap::Args;
use inquire::{Confirm, CustomType, Password, Select, Text};
use reqwest::Url;
use std::fs;
use std::path::PathBuf;
use std::{env, process::Command};

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
        let mut new_cfg = Config {
            jira_url,
            user_login: None,
            api_token: None,
            pat_token: None,
            issue_query: String::from("assignee = currentUser() ORDER BY updated DESC"),
            retry_query: String::from("reporter = currentUser() ORDER BY updated DESC"),
            always_confirm_date: None,
            always_short_branch_names: None,
            max_query_results: Some(50),
            enable_comment_prompts: None,
            one_transition_auto_move: None,
        };

        InitConfig::set_credentials(&mut new_cfg)?;

        if !self.all {
            return InitConfig::write_config(&new_cfg, config_file)
                .context("Failed to write partial config");
        }

        // Text prompts
        new_cfg.issue_query = Text::new("Issue query:")
            .with_default(&new_cfg.issue_query)
            .with_help_message(
                "Suggestion: Use existing filters in Jira 'filter=<filterID> OR filter=<filterID>'",
            )
            .prompt()?;
        new_cfg.retry_query = Text::new("Retry query:")
            .with_default(&new_cfg.retry_query)
            .prompt()?;
        new_cfg.max_query_results = Some(
            CustomType::<u32>::new("Maximum query results:")
                .with_help_message("Lower is faster in case of greedy queries")
                .with_default(new_cfg.max_query_results.unwrap())
                .prompt()?,
        );

        // Boolean prompts
        new_cfg.always_confirm_date = Some(
            Confirm::new("Always ask date when posting worklog:")
                .with_default(true)
                .with_help_message("Invert setting with 'log --date'")
                .prompt()?,
        );
        new_cfg.always_short_branch_names = Some(
            Confirm::new("Use only issue key as branch name:")
                .with_default(false)
                .with_help_message("Invert setting with 'branch --short'")
                .prompt()?,
        );
        new_cfg.enable_comment_prompts = Some(
            Confirm::new("Prompt for optional comments (Worklog):")
                .with_default(false)
                .with_help_message("Override with 'log -c \"\"'")
                .prompt()?,
        );
        new_cfg.one_transition_auto_move = Some(
            Confirm::new("Skip transition select on one valid transition:")
                .with_default(false)
                .prompt()?,
        );

        InitConfig::write_config(&new_cfg, config_file).context("Failed to write full config")
    }

    fn write_config(cfg: &Config, path: PathBuf) -> Result<String> {
        let str_cfg = toml::to_string(&cfg).context("Failed to serialize new Config file")?;

        let cfg_file = config::workspace_config_file();
        fs::write(path.into_os_string(), str_cfg).context("Failed to write config file to")?;

        Ok(format!("Overwrote config: {}", cfg_file.to_str().unwrap()))
    }

    fn jira_url() -> Result<String> {
        let url_input = Text::new("jira_url")
            .with_help_message("Type or paste any url/FQDN. Scheme defaults to HTTPS if missing")
            .prompt()?;

        let parsed_url = if !url_input.starts_with("http") {
            Url::parse(&format!("https://{}", url_input)).context("Unable to parse url")?
        } else {
            Url::parse(&url_input).context("Unable to parse url")?
        };

        Ok(format!(
            "{}://{}",
            parsed_url.scheme(),
            parsed_url.host_str().unwrap()
        ))
    }

    fn set_credentials(icfg: &mut Config) -> Result<()> {
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
            .context("Missing authentication token input")?;

        if icfg.api_token.is_some() {
            icfg.api_token = Some(token);
            icfg.user_login = Some(
                Text::new("Username")
                    .prompt()
                    .context("Username prompt failed")?,
            );
        } else {
            icfg.pat_token = Some(token);
        }

        Ok(())
    }
}
