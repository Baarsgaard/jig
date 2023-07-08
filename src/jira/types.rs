use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Error, Formatter},
    sync::OnceLock,
};

/// Worklog related types
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorklogAddRequestBody {
    pub comment: String,
    pub started: String,
    pub time_spent: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorklogDuration(String);

impl Display for WorklogDuration {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.0)
    }
}

pub(self) static WORKLOG_RE: OnceLock<Regex> = OnceLock::new();

impl TryFrom<String> for WorklogDuration {
    type Error = ();

    fn try_from(value: String) -> Result<Self, ()> {
        let worklog_re =
            WORKLOG_RE.get_or_init(|| Regex::new(r"([0-9](?:\.[0-9]+)?)[wdh]").unwrap());

        if let Some(capture) = worklog_re.captures(&value) {
            return match capture.get(0) {
                Some(worklog_match) => Ok(WorklogDuration(worklog_match.as_str().to_string())),
                None => Err(()),
            };
        }
        Err(())
    }
}

/// Issue related types
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IssueQueryRequestBody {
    pub jql: String,
    pub start_at: u32,
    pub max_results: u32,
    pub fields: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IssueQueryResponseResult {
    /// https://docs.atlassian.com/software/jira/docs/api/REST/7.6.1/#api/2/search
    pub expand: String,
    pub start_at: u32,
    pub max_results: u32,
    pub total: u32,
    pub issues: Vec<Issue>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    pub expand: String,
    pub id: String,
    #[serde(alias = "self")]
    pub self_reference: String,
    pub key: IssueKey,
    pub fields: IssueQueryResultIssueFields,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IssueQueryResultIssueFields {
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
    type Error = ();

    fn try_from(value: String) -> Result<Self, ()> {
        let issue_re = ISSUE_RE.get_or_init(|| Regex::new(r"([A-Z]{2,}-[0-9]+)").unwrap());

        if let Some(capture) = issue_re.captures(&value) {
            return match capture.get(0) {
                Some(issue_match) => Ok(IssueKey(issue_match.as_str().to_string())),
                None => Err(()),
            };
        }
        Err(())
    }
}
