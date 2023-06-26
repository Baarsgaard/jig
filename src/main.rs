pub mod config;

fn main() {
    let now = chrono::Utc::now();
    println!("Time {}", now.to_rfc3339());
    println!("Git is present: {:?}", config::ensure_git_is_available());
    let cfg = config::Config::load();
    println!("{:?}", cfg.unwrap());
}
