use std::collections::HashMap;

use crate::config::Config;
use clap::Args;
use color_eyre::eyre::{Context, Result};
use jira::{
    JiraAPIClient,
    models::{Issue, PostIssueQueryResponseBody},
};
use serde::Serialize;
use serde_json::Value as JsonValue;

use super::shared::ExecCommand;

#[derive(Args, Debug)]
pub struct Query {
    /// Skip querying Jira for Issue summary
    #[arg(value_name = "QUERY")]
    query: String,

    /// Override max_query_results in config (default: 50)
    #[arg(short, long, value_name = "COUNT")]
    count: Option<u32>,

    /// Comma separated list of fields to return
    #[arg(short, long, num_args = 1, value_delimiter = ',')]
    fields: Option<Vec<String>>,

    /// Comma separated list of expansions
    #[arg(short, long, num_args = 1, value_delimiter = ',')]
    expand: Option<Vec<String>>,

    /// Pretty print JSON
    #[arg(short, long)]
    pretty: bool,
}

impl ExecCommand for Query {
    async fn exec(self, cfg: &Config) -> Result<String> {
        let client = match self.count {
            None => {
                JiraAPIClient::new(&cfg.jira_cfg).with_context(|| "Failed to construct API client")
            }
            Some(count) => {
                let mut jira_cfg = cfg.jira_cfg.to_owned();
                jira_cfg.max_query_results = count;
                JiraAPIClient::new(&jira_cfg)
                    .with_context(|| "Failed to construct API client with override")
            }
        }?;

        // Avoid query_issues_
        let query_response = client
            .query_issues(&self.query, self.fields, self.expand)
            .await
            .wrap_err("Issue query failed")?;

        if self.pretty {
            let formatted_response = PrintIssueQuery::from(query_response);
            serde_json::to_string_pretty(&formatted_response).wrap_err("failed exporting issues")
        } else {
            serde_json::to_string(&query_response).wrap_err("failed exporting issues")
        }
    }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct PrintIssueQuery {
    /// https://docs.atlassian.com/software/jira/docs/api/REST/7.6.1/#api/2/search
    pub issues: Vec<PrintIssue>,
    /// Some when expanding names on query_issues
    pub names: Option<HashMap<String, String>>,
}

impl From<PostIssueQueryResponseBody> for PrintIssueQuery {
    fn from(body: PostIssueQueryResponseBody) -> Self {
        let issues = body
            .issues
            .unwrap_or_default()
            .into_iter()
            .map(PrintIssue::from)
            .collect();

        PrintIssueQuery {
            issues,
            names: body.names,
        }
    }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct PrintIssue(HashMap<String, JsonValue>);

impl From<Issue> for PrintIssue {
    fn from(issue: Issue) -> Self {
        let orig_fields = serde_json::to_value(issue.fields).unwrap();
        let orig_fields = orig_fields.as_object().unwrap();

        let mut fields: HashMap<String, JsonValue> = HashMap::new();

        for (k, v) in orig_fields {
            let value = match v {
                JsonValue::Null => None,
                JsonValue::Array(a) if a.is_empty() => None,
                any => Some(any),
            };

            if let Some(value) = value {
                fields.insert(k.to_owned(), value.to_owned());
            }
        }
        PrintIssue(fields)
    }
}
