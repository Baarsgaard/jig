use std::process::{exit, Command};

use crate::config::find_workspace;
use gix::{Repository as Gix_Repo, ThreadSafeRepository};

#[derive(Debug, Clone)]
pub struct Repository {
    repo: Gix_Repo,
}

impl Repository {
    pub fn open() -> Self {
        let repo = ThreadSafeRepository::open(find_workspace())
            .unwrap()
            .to_thread_local();

        Repository { repo }
    }

    pub fn get_branch_name(&self) -> String {
        let head_ref = self.repo.head_ref().unwrap();
        let head_commit = self.repo.head_commit().unwrap();

        match head_ref {
            Some(reference) => reference.name().shorten().to_string(),
            None => head_commit.id.to_hex_with_len(8).to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn create_branch(&self, branch_name: String) {
        let result = Command::new("git")
            .args(["checkout", "-b", branch_name.as_str()])
            .spawn();
        match result {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Failed to create branch:\n{}", e);
                exit(1);
            }
        }
    }
}
