[![Build Status](https://github.com/baarsgaard/jig/actions/workflows/integration.yml/badge.svg)](https://github.com/baarsgaard/jig/actions)

# Jig

Jira Integration with Git CLI

## Why?

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
- Scriptable, I live for automation.

## Installation

See releases for installation instructions: [releases](https://github.com/Baarsgaard/jig/releases)


### Compile from source

Compile from source with [Rust-lang](https://www.rust-lang.org/tools/install) and:=

```bash
cargo install --locked --git https://github.com/raunow/jig
# Optionally add `--features cloud` to enable ApiV3/Cloud only features.
```


## Configuration

Supports Global and Local config files.  
`~/.config/jig/config.toml` and `.jig.toml` respectively ([XDG](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html)).  
If both exist, they are merged with the Local config taking priority.

See [example_config.toml](./example_config.toml)

Generate your configuration using:
```bash
jig init [--all]
```

## Usage

[![asciicast](https://asciinema.org/a/609019.svg)](https://asciinema.org/a/609019)

```bash
$ jig help

A Jira CLI integration with Git

Usage: 

Commands:
  assign   Assign user to issue
  branch   Create and checkout branch using issue key with(out) summary as branch name
  comment  Create comment on a Jira Issue
  configs  List config file locations
  hook     Install git commit-msg hook
  init     Initialise config file(s)
  log      Create a work log entry on a Jira issue
  move     Move ticket through transitions
  open     Open issue in your browser
  upgrade  Download and install latest version
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

```bash
# Create a branch from an issue
jig branch

# Work on that branch and commit as normal.
# Log work/Comment progress as you work normally.
# Optionally comment progress as you work.
jig comment "Note: Changed impl due to X"

# Create worklog as you finish up current session
jig log 1h --comment "Bug squashed"

# Transition ticket according to you workflow
jig move
```
