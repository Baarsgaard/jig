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
    pub time_spent_seconds: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorklogDuration(String);

impl WorklogDuration {
    pub fn to_seconds(&self) -> Result<u32, ()> {
        let mut dur = self.0.clone().to_lowercase();
        if dur.len() < 2 || dur.chars().last().unwrap().is_numeric() {
            return Err(());
        }

        let multiplier = match dur.chars().last() {
            Some(m) if m == 'h' => 3600,
            Some(m) if m == 'd' => 86400,
            Some(m) if m == 'w' => 604800,
            _ => return Err(()),
        };

        dur.pop();
        match dur.parse::<f32>() {
            Err(_) => Err(()),
            Ok(duration) if duration.is_sign_negative() => Err(()),
            Ok(duration) if duration.is_nan() => Err(()),
            Ok(duration) => Ok((multiplier as f32 * duration).round() as u32),
        }
    }
}

impl From<WorklogDuration> for u32 {
    fn from(value: WorklogDuration) -> Self {
        value.to_seconds().unwrap()
    }
}

impl TryFrom<String> for WorklogDuration {
    type Error = ();

    fn try_from(value: String) -> Result<Self, ()> {
        let item = WorklogDuration(value);
        match item.to_seconds() {
            Ok(_) => Ok(item),
            Err(e) => Err(e),
        }
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
