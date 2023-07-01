use chrono::{Utc, Weekday};
use fuzzy_matcher::FuzzyMatcher;
use gix::ThreadSafeRepository;
use inquire::{validator::Validation, CustomUserError, DateSelect, Text};
use regex::Regex;
use std::cmp::Reverse;

use crate::config;
use config::Config;
// use crate::jira;

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
    let repo = ThreadSafeRepository::open(config::find_workspace())
        .unwrap()
        .to_thread_local();
    let head_ref = repo.head_ref().unwrap();
    let head_commit = repo.head_commit().unwrap();

    match head_ref {
        Some(reference) => reference.name().shorten().to_string(),
        None => head_commit.id.to_hex_with_len(8).to_string(),
    }
}

fn issue_fuzzer(input: &str) -> Result<Vec<String>, CustomUserError> {
    let input = input;
    let matcher = fuzzy_matcher::skim::SkimMatcherV2::default().ignore_case();
    let mut names = my_issues();

    // Loop through names, score each against input, store results
    let mut matches: Vec<(String, i64)> = names
        .into_iter()
        .filter_map(|name| {
            matcher
                .fuzzy_match(&name, input)
                .map(|score| (name.to_owned(), score))
        })
        .collect();

    // Sort by score and retrieve names.
    matches.sort_unstable_by_key(|(_file, score)| Reverse(*score));
    names = matches.into_iter().map(|(name, _)| name).collect();

    Ok(names)
}

// TODO Replace with JiraAPIClient::my_issues
fn my_issues() -> Vec<String> {
    vec![
        String::from("JB-2 TO DO Issue"),
        String::from("JB-1 Minimal issue"),
        String::from("JA-2 Other"),
        String::from("JC-3 Other"),
    ]
}

pub fn get_or_input_issue_key(cfg: &Config) -> String {
    let issue_re = Regex::new(r"([A-Z]{2,}-[0-9]+)").unwrap();
    let head = get_branch_name().to_uppercase();

    let captures = issue_re.captures(&head);

    if let Some(result) = captures {
        return result.get(0).unwrap().as_str().to_owned();
    }

    let mut issue_key = cfg.default_issue_key.clone().unwrap_or_default();
    if !String::is_empty(&issue_key) {
        issue_key.push('-');
    }

    let text_input = Text::new("Jira Issue: ")
        .with_initial_value(&issue_key)
        .with_autocomplete(&issue_fuzzer)
        .with_validator(
            move |s: &str| match issue_re.captures(s.to_uppercase().as_str()) {
                Some(_) => Ok(Validation::Valid),
                None => Ok(Validation::Invalid("Must be valid Jira Issue key".into())),
            },
        );
    text_input.prompt().unwrap().as_str().to_owned()
}
