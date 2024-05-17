use crate::{commands::shared::UseFilter, config::Config};
use chrono::Utc;
use color_eyre::eyre::{eyre, Result, WrapErr};
use jira::{
    types::{Issue, IssueKey},
    JiraAPIClient,
};

#[cfg(feature = "cloud")]
mod filter {
    use super::*;

    use inquire::MultiSelect;
    use jira::types::Filter;
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
            .map(Filter::filter_query)
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

    let issues = query_issues_with_retry(client, cfg).await?;

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

    let issues = override_query_issues_with_retry(client, &query, &cfg.retry_query).await?;
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
    let issues = match client
        .query_issues(
            &format!("issuekey = {}", issue_key),
            Some(vec!["summary".to_string()]),
            None,
        )
        .await?
        .issues
    {
        Some(i) => i,
        None => Err(eyre!("Issue query response object empty"))?,
    };

    match issues.first() {
        Some(i) => Ok(i.to_owned()),
        None => Err(eyre!("Error issue not found: {}", issue_key)),
    }
}

// This exists to allow overriding the default configured retry query
pub async fn override_query_issues_with_retry(
    client: &JiraAPIClient,
    issue_query: &String,
    retry_query: &String,
) -> Result<Vec<Issue>> {
    let maybe_issues = match client
        .query_issues(issue_query, Some(vec!["summary".to_string()]), None)
        .await
        .wrap_err("Initial issue query failed")
    {
        Ok(issue_body) => issue_body.issues,
        Err(e) => {
            client
                .query_issues(retry_query, Some(vec!["summary".to_string()]), None)
                .await
                .wrap_err(e)
                .wrap_err(eyre!("Retry query failed"))?
                .issues
        }
    };

    match maybe_issues {
        Some(i) => Ok(i),
        None => Err(eyre!("Issue list is empty"))?,
    }
}

pub async fn query_issues_with_retry(client: &JiraAPIClient, cfg: &Config) -> Result<Vec<Issue>> {
    override_query_issues_with_retry(client, &cfg.issue_query, &cfg.retry_query).await
}
