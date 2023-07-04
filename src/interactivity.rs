use chrono::{Utc, Weekday};
use gix::ThreadSafeRepository;
use inquire::{DateSelect, Select};
use regex::Regex;

use crate::config::{find_workspace, Config};

pub fn select_date(cfg: &Config) -> String {
    let now = Utc::now().to_rfc3339();

    if cfg.always_confirm_date.unwrap_or(true) {
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

fn get_branch_name() -> String {
    let repo = ThreadSafeRepository::open(find_workspace())
        .unwrap()
        .to_thread_local();
    let head_ref = repo.head_ref().unwrap();
    let head_commit = repo.head_commit().unwrap();

    match head_ref {
        Some(reference) => reference.name().shorten().to_string(),
        None => head_commit.id.to_hex_with_len(8).to_string(),
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

pub fn get_or_input_issue_key(cfg: &Config) -> String {
    let issue_re = Regex::new(r"([A-Z]{2,}-[0-9]+)").unwrap();
    let head = get_branch_name().to_uppercase();

    let captures = issue_re.captures(&head);

    if let Some(result) = captures {
        return result.get(0).unwrap().as_str().to_owned();
    }

    let client = cfg.build_api_client();
    let my_issues = client.my_issues(cfg.default_issue_key.clone().unwrap_or_default());

    let issue = Select::new("Jira issue:", my_issues).prompt().unwrap();
    issue.key
}
