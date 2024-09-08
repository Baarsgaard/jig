[![Build Status](https://github.com/baarsgaard/jig/actions/workflows/integration.yml/badge.svg)](https://github.com/baarsgaard/jig/actions)

# Jig

Jira Integration with Git CLI.

Jig is an attempt at making it easier for me and my colleagues to collaborate on jira issues.  
Specifically:
- Making it simpler to create branches with descriptive names.
- Automatically include issue key in commits.
- Simplifying issue administration and workflow
  - Creating Worklogs.
  - Transitioning issues.
  - Assigning to others.
  - Bonus: [Scripting](./scripts/weekly_worklogs.sh) Jira interactions.


## Usage

```bash
$ jig help

A Jira CLI integration with Git

Usage: jig <COMMAND>

Commands:
  assign      Assign user to issue
  branch      Create and checkout branch using issue key with(out) summary as branch name
  comment     Create comment on a Jira Issue
  completion  Generate completion script
  configs     List config file locations
  hook        Install git commit-msg hook
  init        Initialise config file(s)
  worklog     Create a work log entry on a Jira issue
  transition  Move ticket through transitions
  open        Open issue in your browser
  query       Interactively send JQL queries to Jira when tab is pressed
  upgrade     Download and install latest version
  help        Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

```bash
# Create or checkout branch named after an issue.
jig branch
# Work on that branch and commit as usual. git add/commit/push.

# Make comments as you progress with your work.
jig comment "Note: Changed impl due to X"

# Create worklogs as you finish up a session or at the end of the day.
jig log 1h --comment "Bug squashed"

# Transition issue according to your workflow.
jig move
```

[![asciicast](https://asciinema.org/a/609019.svg)](https://asciinema.org/a/609019)

## Installation

See releases for installation instructions: [releases](https://github.com/Baarsgaard/jig/releases)


### Compile from source

Compile from source with [Rust-lang](https://www.rust-lang.org/tools/install) with

```bash
cargo install --locked --git https://github.com/baarsgaard/jig
# Optionally add `--features cloud` to enable ApiV3/Cloud only features.
```


## Configuration

Supports Global and Local config files.  
`~/.config/jig/config.toml` and `.jig.toml` respectively ([XDG](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html)).  
If both exist, they are merged with the Local config taking priority.

This is useful when working across different repositories with different cobtribution workflows or issue queries when creating branches or worklogs.

See [example_config.toml](./example_config.toml)

Generate your configuration using:
```bash
jig init [--all]
```


<details>
<summary>Why?</summary>

I personally love a strict Git workflow with well designed PRs and every commit being attributed to an issue.  
But convincing others to adopt this can be a challenge without obvious benefits.

An obvious use case being my own workflow:  
Most if not all my work at $day_job is coordinated through and logged in Jira.  
additionally I'm not a fan of doing simple tasks in the Jira UI..

I always know the issue I am currently working on, I am on the branch already.  
Why could I not log my time or comment more easily? Maybe directly from the terminal even?

Looking at the existing CLI tools that interacted with Jira, none solved my exact problem.  
Hence, **[Jig](https://www.youtube.com/watch?v=3JcmQONgXJM)!**

Jig is designed to simplify working with a "Per issue" branching model.  
It therefore includes options and features I needed to support that.

Primarily:  
- Creating or checking out branches from existing Jira issues with(out) summaries.
- Quickly logging time and Commenting on the issue found in the branch name.
- Moving issues from one status to the next.
- [Scripting](./scripts/weekly_worklogs.sh), I live for automation.

</details>

