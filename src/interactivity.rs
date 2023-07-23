use anyhow::{anyhow, Context, Result};
use chrono::{Utc, Weekday};
use inquire::{DateSelect, Select};

use crate::config::Config;
use crate::jira::client::JiraAPIClient;
use crate::jira::types::{Issue, IssueKey};

pub fn get_date(cfg: &Config, force_toggle_prompt: bool) -> Result<String> {
    // Jira sucks and can't parse correct rfc3339 due to the ':' in tz.. https://jira.atlassian.com/browse/JRASERVER-61378
    // Fix: https://docs.rs/chrono/latest/chrono/format/strftime/index.html
    let now = Utc::now().format("%FT%X%.3f%z").to_string();
    let mut do_prompt = cfg.always_confirm_date.unwrap_or(true);

    if force_toggle_prompt {
        do_prompt = !do_prompt;
    }

    if do_prompt {
        let date = DateSelect::new("")
            .with_week_start(Weekday::Mon)
            .prompt()?
            .to_string();

        // A lot prettier than a complex of format!()
        Ok(date + &now[10..])
    } else {
        Ok(now)
    }
}

// fn issue_fuzzer(input: &str, issues: Vec<Issue>) -> Result<Vec<String>, CustomUserError> {
//     let input = input;
//     let mut names = issues
//         .iter()
//         .map(|i| i.key + " " + i.fields.summary.as_str())
//         .collect::<Vec<String>>();

//     // Loop through names, score each against input, store results
//     let matcher = fuzzy_matcher::skim::SkimMatcherV2::default().ignore_case();
//     let mut matches: Vec<(String, i64)> = names
//         .into_iter()
//         .filter_map(|name| {
//             matcher
//                 .fuzzy_match(&name, input)
//                 .map(|score| (name.to_owned(), score))
//         })
//         .collect();

//     // Sort by score and retrieve names.
//     matches.sort_unstable_by_key(|(_file, score)| Reverse(*score));
//     names = matches.into_iter().map(|(name, _)| name).collect();

//     Ok(names)
// }

pub fn query_issue_details(client: &JiraAPIClient, issue_key: IssueKey) -> Result<Issue> {
    let issues = client
        .query_issues(format!("issuekey = {}", issue_key))?
        .issues
        .unwrap();

    match issues.first() {
        Some(i) => Ok(i.to_owned()),
        None => Err(anyhow!("Error issue not found: {}", issue_key)),
    }
}

pub fn prompt_user_with_issue_select(issues: Vec<Issue>) -> Result<Issue> {
    if issues.is_empty() {
        Err(anyhow!("Select Prompt: Empty issue list"))?
    }

    Select::new("Jira issue:", issues)
        .prompt()
        .context("No issue selected")
}

pub fn query_issues_with_retry(client: &JiraAPIClient, cfg: &Config) -> Result<Vec<Issue>> {
    let issues = match client
        .query_issues(cfg.issue_query.clone())
        .context("First issue query failed")
    {
        Ok(issue_body) => issue_body.issues.unwrap(),
        Err(_) => client
            .query_issues(cfg.retry_query.clone())
            .context(anyhow!("Retry query failed"))?
            .issues
            .unwrap(),
    };

    Ok(issues)
}

pub fn issue_from_branch_or_prompt(
    client: &JiraAPIClient,
    cfg: &Config,
    head_name: String,
) -> Result<Issue> {
    if let Ok(issue_key) = IssueKey::try_from(head_name) {
        return query_issue_details(client, issue_key);
    }

    let issues = query_issues_with_retry(client, cfg)?;

    prompt_user_with_issue_select(issues)
}
