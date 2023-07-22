use anyhow::{anyhow, Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{Display, Error, Formatter},
    sync::OnceLock,
};

/// Comment related types
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostCommentBody {
    pub body: String,
}

/// Worklog related types
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostWorklogBody {
    pub comment: String,
    pub started: String,
    pub time_spent: String,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorklogDuration(String);

impl Display for WorklogDuration {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.0)
    }
}

pub(self) static WORKLOG_RE: OnceLock<Regex> = OnceLock::new();

impl TryFrom<String> for WorklogDuration {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self>
    where
        anyhow::Error: From<std::fmt::Error>,
    {
        let worklog_re = WORKLOG_RE.get_or_init(|| {
            Regex::new(r"([0-9](?:\.[0-9]+)?)[wdh]").expect("Unable to compile WORKLOG_RE")
        });

        let worklog = match worklog_re.captures(&value) {
            Some(c) => c.get(0).context("First capture is none: WORKLOG_RE")?,
            None => Err(anyhow!("Malformed worklog duration: {}", value))?,
        };

        Ok(WorklogDuration(worklog.as_str().to_string()))
    }
}

/// Issue related types
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostIssueQueryBody {
    pub fields: Vec<String>,
    pub jql: String,
    pub max_results: u32,
    pub start_at: u32,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostIssueQueryResponseBody {
    /// https://docs.atlassian.com/software/jira/docs/api/REST/7.6.1/#api/2/search
    pub expand: String,
    pub issues: Vec<Issue>,
    pub max_results: u32,
    pub start_at: u32,
    pub total: u32,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    pub expand: String,
    pub fields: PostIssueQueryResponseBodyFields,
    pub id: String,
    pub key: IssueKey,
    #[serde(alias = "self")]
    pub self_reference: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostIssueQueryResponseBodyFields {
    pub summary: String,
}

impl Display for Issue {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{} {}", self.key, self.fields.summary)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IssueKey(pub String);

impl Display for IssueKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.0)
    }
}

pub(self) static ISSUE_RE: OnceLock<Regex> = OnceLock::new();

impl TryFrom<String> for IssueKey {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self>
    where
        anyhow::Error: From<std::fmt::Error>,
    {
        let issue_re = ISSUE_RE
            .get_or_init(|| Regex::new(r"([A-Z]{2,}-[0-9]+)").expect("Unable to compile ISSUE_RE"));

        let issue_key = match issue_re.captures(&value) {
            Some(c) => c.get(0).context("First capture is none: ISSUE_RE")?,
            None => Err(anyhow!("Malformed issue key supplied: {}", value))?,
        };

        Ok(IssueKey(issue_key.as_str().to_string()))
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetTransitionsBody {
    pub expand: String,
    pub transitions: Vec<Transition>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Transition {
    pub fields: HashMap<String, TransitionExpandedFields>,
    pub id: String,
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TransitionExpandedFields {
    pub required: bool,
    pub name: String,
    pub operations: Vec<String>,
    pub allowed_values: Option<Vec<String>>,
}

#[derive(Serialize, Debug, Clone)]
pub struct PostTransitionBody {
    pub transition: Transition,
}

impl Display for Transition {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.name)
    }
}
