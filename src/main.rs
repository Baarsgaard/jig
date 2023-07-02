//#![allow(unused_variables)]
pub mod config;
pub mod interactivity;
pub mod jira;

fn main() {
    let cfg = config::Config::load().unwrap();
    let rfc3339 = interactivity::select_date(&cfg);
    let issue = interactivity::get_or_input_issue_key(&cfg);
    let jiraclient = cfg.build_api_client();

    dbg!(rfc3339);
    dbg!(issue);
    dbg!(cfg);
    dbg!(jiraclient);
}
