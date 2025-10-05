use crate::config::Config;
use chrono::Utc;
use color_eyre::eyre::{Result, WrapErr, eyre};
use jira::{
    JiraAPIClient,
    models::{Issue, IssueKey},
};

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
) -> Result<Issue> {
    if let Ok(issue_key) = IssueKey::try_from(head_name) {
        return query_issue_details(client, issue_key).await;
    }

    let query = cfg.issue_query.clone();

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
        Ok(query_res) if query_res.issues.is_none() => {
            Err(eyre!("No issues found using given issue_query"))
        }
        Ok(query_res) => Ok(query_res.issues.unwrap()),
        Err(e) => Err(eyre!(e)),
    }
}
