use crate::config::{self, Config};
use anyhow::{Context, Result};
use clap::Args;
use inquire::{Password, Text};
use reqwest::Url;
use std::fs;
use std::{env, process::Command};

#[derive(Args, Debug)]
pub struct InitConfig {
    #[arg(short, long)]
    all: bool,
}

// jira_url: String,
// user_login: Option<String>,
// api_token: Option<String>,
// pat_token: Option<String>,
// issue_query: Option<String>,
// retry_query: Option<String>,
// always_confirm_date: Option<bool>,
// always_short_branch_names: Option<bool>,
// max_query_results: Option<u32>,
// enable_comment_prompts: Option<bool>,
// one_transition_auto_move: Option<bool>,

impl InitConfig {
    pub fn init(&self) -> Result<String> {
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
            max_query_results: None,
            enable_comment_prompts: None,
            one_transition_auto_move: None,
        };

        InitConfig::set_credentials(&mut new_cfg)?;

        if !self.all {
            return InitConfig::write_config(&new_cfg);
        }

        Ok(String::from(""))
    }

    fn write_config(cfg: &Config) -> Result<String> {
        let str_cfg = toml::to_string(&cfg).context("Failed to serialize new Config file")?;

        let cfg_file = config::workspace_config_file();
        fs::write(cfg_file.clone().into_os_string(), str_cfg)
            .context("Failed to write config file to")?;

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
