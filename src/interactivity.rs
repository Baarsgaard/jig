use chrono::{Utc, Weekday};
use inquire::{DateSelect, Select};

use crate::config::Config;
use crate::jira::client::JiraAPIClient;
use crate::jira::types::{Issue, IssueKey};
use crate::repo::Repository;

pub fn get_date(cfg: &Config, force_toggle_prompt: bool) -> String {
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
            .prompt()
            .unwrap()
            .to_string();

        // A lot prettier than a complex of format!()
        date + &now[10..]
    } else {
        now
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

pub fn query_issues(
    client: &JiraAPIClient,
    mut query: String,
    project: Option<String>,
) -> Option<Vec<Issue>> {
    if let Some(p) = project {
        query = format!("project = {} AND {}", p, query);
    }

    match client.query_issues(query) {
        Ok(issue_query_result) => Some(issue_query_result.issues),
        Err(e) => {
            eprintln!("{}", e);
            None
        }
    }
}

pub fn query_assigned_issues(
    client: &JiraAPIClient,
    project: Option<String>,
) -> Option<Vec<Issue>> {
    let assignee_query = String::from("assignee = currentUser() ORDER BY updated DESC");
    query_issues(client, assignee_query, project)
}

pub fn query_issue_details(client: &JiraAPIClient, issue_key: IssueKey) -> Option<Issue> {
    let option = client.query_issues(format!("issuekey = {}", issue_key));
    match option {
        Ok(i) => Some(i.issues.first().unwrap().to_owned()),
        Err(e) => {
            eprintln!("{}", e);
            None
        }
    }
}

pub fn prompt_user_with_issue_select(issues: Vec<Issue>) -> Option<Issue> {
    match Select::new("Jira issue:", issues).prompt() {
        Ok(i) => Some(i),
        Err(e) => {
            eprintln!("{}", e);
            None
        }
    }
}

pub fn issue_from_branch_or_prompt(
    client: &JiraAPIClient,
    cfg: &Config,
    repo: &Repository,
) -> Option<Issue> {
    let head = repo.get_branch_name().to_uppercase();

    if let Ok(issue_key) = IssueKey::try_from(head) {
        return query_issue_details(client, issue_key);
    }

    let issues = match query_assigned_issues(client, cfg.default_issue_key.clone()) {
        Some(i) => i,
        None => {
            let retry_query = cfg.retry_query_override.clone().unwrap_or(String::from(
                "(reporter = currentUser()) ORDER BY updated DESC",
            ));
            query_issues(client, retry_query, cfg.default_issue_key.clone())?
        }
    };

    prompt_user_with_issue_select(issues)
}
