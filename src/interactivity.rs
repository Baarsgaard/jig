use crate::{commands::shared::UseFilter, config::Config};
use chrono::Utc;
use color_eyre::eyre::{Result, WrapErr, eyre};
use jira::{
    JiraAPIClient,
    models::{Issue, IssueKey},
};

#[cfg(feature = "cloud")]
mod filter {
    use super::*;

    use inquire::MultiSelect;
    use jira::models::Filter;
    pub fn pick_filter(cfg: &Config, filters: Vec<Filter>) -> Result<String> {
        if filters.is_empty() {
            return Err(eyre!("List of filters is empty"));
        }

        let selected_filters = MultiSelect::new("Saved issue filter:", filters)
            .with_help_message("Only displays favourited filters")
            .prompt()
            .wrap_err("Filter prompt interrupted")?;

        let filter_list = selected_filters
            .iter()
            .map(|filter| format!("filter={}", filter.id))
            .collect::<Vec<String>>();

        if cfg.inclusive_filters.unwrap_or(true) {
            Ok(filter_list.join(" OR "))
        } else {
            Ok(filter_list.join(" AND "))
        }
    }
}

// Might be useful one day
#[allow(dead_code)]
pub async fn issue_key_from_branch_or_prompt(
    client: &JiraAPIClient,
    cfg: &Config,
    head_name: String,
) -> Result<IssueKey> {
    if let Ok(issue_key) = IssueKey::try_from(head_name) {
        return Ok(query_issue_details(client, issue_key).await?.key);
    }

    let issues = query_issues_empty_err(client, &cfg.issue_query).await?;

    Ok(prompt_user_with_issue_select(issues)?.key)
}

pub async fn issue_from_branch_or_prompt(
    client: &JiraAPIClient,
    cfg: &Config,
    head_name: String,
    _use_filter: UseFilter,
) -> Result<Issue> {
    if let Ok(issue_key) = IssueKey::try_from(head_name) {
        return query_issue_details(client, issue_key).await;
    }

    #[cfg(not(feature = "cloud"))]
    let query = cfg.issue_query.clone();
    #[cfg(feature = "cloud")]
    let query = match _use_filter.value {
        true => filter::pick_filter(cfg, client.search_filters(None).await?.filters)?,
        false => cfg.issue_query.clone(),
    };

    let issues = query_issues_empty_err(client, &query).await?;
    prompt_user_with_issue_select(issues)
}

pub fn prompt_user_with_issue_select(issues: Vec<Issue>) -> Result<Issue> {
    use inquire::Select;

    if issues.is_empty() {
        Err(eyre!("Select Prompt: Empty issue list"))?
    }

    Select::new("Jira issue:", issues)
        .prompt()
        .wrap_err("No issue selected")
}

/// Almost rfc3339 AKA Jira compatible
pub fn now() -> String {
    // Jira sucks and can't parse correct rfc3339 due to the ':' in tz.. https://jira.atlassian.com/browse/JRASERVER-61378
    // Fix: https://docs.rs/chrono/latest/chrono/format/strftime/index.html
    Utc::now().format("%FT%X%.3f%z").to_string()
}

pub async fn query_issue_details(client: &JiraAPIClient, issue_key: IssueKey) -> Result<Issue> {
    client
        .get_issue(&issue_key, None)
        .await
        .wrap_err("Fetching issue details failed")
}

pub async fn query_issues_empty_err(client: &JiraAPIClient, query: &str) -> Result<Vec<Issue>> {
    match client
        .query_issues(query, Some(vec!["summary".to_string()]), None)
        .await
        .wrap_err("Issue query failed")
    {
        Ok(query_res) if query_res.total == 0 => {
            Err(eyre!("No issues found using set issue_query"))
        }
        Ok(query_res) => Ok(query_res.issues),
        Err(e) => Err(eyre!(e)),
    }
}
