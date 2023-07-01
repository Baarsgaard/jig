pub mod config;
pub mod interactivity;
pub mod jira;
use interactivity::{get_or_input_issue_key, select_date};

fn main() {
    let cfg = config::Config::load().unwrap();
    let rfc3339 = select_date(&cfg);
    let issue = get_or_input_issue_key(&cfg);

    dbg!(rfc3339);
    dbg!(issue);
    dbg!(cfg);
}
