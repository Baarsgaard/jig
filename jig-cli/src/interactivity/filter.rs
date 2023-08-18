use crate::config::Config;
use color_eyre::eyre::{eyre, Result, WrapErr};
use inquire::MultiSelect;
use jira::types::Filter;

#[cfg(feature = "fuzzy")]
mod fuzzy {
    use super::*;
    use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
    use inquire::Select;
    use jira::types::Issue;
    use std::fmt::Display;

    fn select_fuzzy_filter<T: Display>(
        input: &str,
        item_to_score: &T,
        matcher: &SkimMatcherV2,
    ) -> bool {
        let maybe_score = matcher.fuzzy_match(item_to_score.to_string().as_str(), input);

        match maybe_score {
            Some(score) => score.gt(&0),
            None => false,
        }
    }

    pub fn pick_filter(cfg: &Config, filters: Vec<Filter>) -> Result<String> {
        if filters.is_empty() {
            return Err(eyre!("List of filters is empty"))?;
        }

        let matcher = SkimMatcherV2::default().ignore_case();
        let selected_filters = MultiSelect::new("Saved issue filter:", filters)
            .with_help_message("Only displays favourited filters")
            .with_filter(&|input, filter, _value, _size| {
                select_fuzzy_filter(input, filter, &matcher)
            })
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
}

#[cfg(not(feature = "fuzzy"))]
mod fuzzy {
    use super::*;

    pub fn pick_filter(cfg: &Config, filters: Vec<Filter>) -> Result<String> {
        if filters.is_empty() {
            return Err(eyre!("List of filters is empty"))?;
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

pub use fuzzy::*;
