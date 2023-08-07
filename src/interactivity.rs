use crate::config::Config;
use crate::jira::client::JiraAPIClient;
use crate::jira::types::{Filter, Issue, IssueKey};
use chrono::{Utc, Weekday};
use color_eyre::eyre::{eyre, Result, WrapErr};
#[cfg(feature = "fuzzy_filter")]
use fuzzy_matcher::skim::SkimMatcherV2;
#[cfg(feature = "fuzzy_filter")]
use fuzzy_matcher::FuzzyMatcher;
#[cfg(not(feature = "fuzzy_finder"))]
use inquire::Select;
use inquire::{DateSelect, MultiSelect};
use std::fmt::Display;

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

#[cfg(feature = "fuzzy_filter")]
fn select_fuzzy_filter<T: Display>(input: &str, issue: &T, matcher: &SkimMatcherV2) -> bool {
    let maybe_score = matcher.fuzzy_match(issue.to_string().as_str(), input);

    match maybe_score {
        Some(score) => score.gt(&0),
        None => false,
    }
}

#[cfg(feature = "fuzzy_filter")]
pub fn prompt_user_with_issue_select(issues: Vec<Issue>) -> Result<Issue> {
    if issues.is_empty() {
        Err(eyre!("Select Prompt: Empty issue list"))?
    }

    let matcher = SkimMatcherV2::default().ignore_case();

    let issue = Select::new("Jira issue:", issues)
        .with_filter(&|input, issue, _value, _size| select_fuzzy_filter(input, issue, &matcher))
        .prompt()?;

    Ok(issue)
}

pub fn query_issue_details(client: &JiraAPIClient, issue_key: IssueKey) -> Result<Issue> {
    let issues = client
        .query_issues(format!("issuekey = {}", issue_key))?
        .issues
        .unwrap();

    match issues.first() {
        Some(i) => Ok(i.to_owned()),
        None => Err(eyre!("Error issue not found: {}", issue_key)),
    }
}

#[cfg(not(feature = "fuzzy_filter"))]
pub fn prompt_user_with_issue_select(issues: Vec<Issue>) -> Result<Issue> {
    if issues.is_empty() {
        Err(eyre!("Select Prompt: Empty issue list"))?
    }

    Select::new("Jira issue:", issues)
        .prompt()
        .wrap_err("No issue selected")
}

pub fn query_issues_with_retry(
    client: &JiraAPIClient,
    cfg: &Config,
    query: String,
) -> Result<Vec<Issue>> {
    let issues = match client
        .query_issues(query)
        .wrap_err("First issue query failed")
    {
        Ok(issue_body) => issue_body.issues.unwrap(),
        Err(_) => client
            .query_issues(cfg.retry_query.clone())
            .wrap_err(eyre!("Retry query failed"))?
            .issues
            .unwrap(),
    };

    Ok(issues)
}

pub fn issue_from_branch_or_prompt(
    client: &JiraAPIClient,
    cfg: &Config,
    head_name: String,
    use_filter: bool,
) -> Result<Issue> {
    if let Ok(issue_key) = IssueKey::try_from(head_name) {
        return query_issue_details(client, issue_key);
    }

    let query = if use_filter {
        let mut filters = client.search_filters(None)?.filters;

        if filters.is_empty() {
            return Err(eyre!("List of filters is empty"))?;
        }

        let matcher = SkimMatcherV2::default().ignore_case();
        filters = MultiSelect::new("Saved issue filter:", filters)
            .with_help_message("Only displays favourited filters")
            .with_filter(&|input, filter, _value, _size| {
                select_fuzzy_filter(input, filter, &matcher)
            })
            .prompt()
            .wrap_err("Filter prompt interrupted")?;

        filters
            .iter()
            .map(Filter::filter_query)
            .collect::<Vec<String>>()
            .join(" OR ")
    } else {
        cfg.issue_query.clone()
    };

    let issues = query_issues_with_retry(client, cfg, query)?;

    prompt_user_with_issue_select(issues)
}
