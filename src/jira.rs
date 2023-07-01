use base64::{engine::general_purpose, Engine as _};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JiraAPIClient {
    pub url: String,
    pub user_token: String,
    pub user_email: String,
    pub version: u8,
}

impl JiraAPIClient {
    pub fn get_basic_auth(&self) -> HeaderMap {
        let content_header = HeaderValue::from_static("application/json");
        let mut jira_token_header =
            HeaderValue::from_str(format!("Basic {}", self.user_token).as_str()).unwrap();
        jira_token_header.set_sensitive(true);

        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, content_header.clone());
        headers.insert(CONTENT_TYPE, content_header);
        headers.insert(AUTHORIZATION, jira_token_header);

        headers
    }
}

impl JiraAPIClient {
    pub fn get_issues(&self, _query: &str) -> Vec<String> {
        vec![]
    }

    pub fn my_issues(&self) -> Vec<String> {
        self.get_issues("")
    }

    pub fn post_worklog(&self) -> bool {
        true
    }

    pub fn new(url: String, user_token: String, user_email: String) -> Self {
        JiraAPIClient {
            url,
            user_token,
            user_email,
            version: 3,
        }
    }
}
