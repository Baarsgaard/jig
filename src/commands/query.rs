use crate::config::Config;
use clap::Args;
use color_eyre::eyre::{Context, Result};
use inquire::Autocomplete;
use jira::JiraAPIClient;

use super::shared::{ExecCommand, UseFilter};

#[derive(Args, Debug)]
pub struct Query {
    /// Skip querying Jira for Issue summary
    #[arg(value_name = "QUERY")]
    query: String,

    /// Pretty print JSON
    #[arg(short, long)]
    pretty: bool,

    /// Comma separated list of fields to return
    #[arg(short, long, num_args = 1, value_delimiter = ',')]
    fields: Option<Vec<String>>,

    #[command(flatten)]
    use_filter: UseFilter,
}

impl ExecCommand for Query {
    async fn exec(self, cfg: &Config) -> Result<String> {
        let client = JiraAPIClient::new(&cfg.jira_cfg)?;

        // Avoid query_issues_
        let query_response = client
            .query_issues(&self.query, self.fields, None)
            .await
            .wrap_err("Issue query failed")?;

        if self.pretty {
            serde_json::to_string_pretty(&query_response.issues).wrap_err("failed exporting issues")
        } else {
            serde_json::to_string(&query_response.issues).wrap_err("failed exporting issues")
        }
    }
}

#[derive(Clone)] //Default
pub struct IssueQueryBuilder {}

impl IssueQueryBuilder {}

impl Autocomplete for IssueQueryBuilder {
    fn get_suggestions(
        &mut self,
        _input: &str,
    ) -> std::result::Result<Vec<String>, inquire::CustomUserError> {
        todo!()
    }

    fn get_completion(
        &mut self,
        _input: &str,
        _highlighted_suggestion: Option<String>,
    ) -> std::result::Result<inquire::autocompletion::Replacement, inquire::CustomUserError> {
        todo!()
    }
}
