# Jig

(Ji)ra (G)it  
Most if not all my work at $day_job is coordinated through and logged in Jira.  
additionally I'm not a fan of doing simple tasks in the Jira UI..

I looked at existing Jira CLI tools, but none solve my exact use case.  
Hence, [Jig](https://www.youtube.com/watch?v=3JcmQONgXJM)!

Jig is opinionated towards working with a healthy "Per issue" branching model, also known as "My workflow".  
It therefore includes options and features I need to support that.

Primarily:  
Creating new branches from Jira issues with(out) summaries.  
Quickly logging time and Commenting on the issue found in the branch name.  
Moving issues from one status to the next.  

## Installation

Install [Rust-lang](https://www.rust-lang.org/tools/install) to compile from source.
```bash
cargo install --locked --git https://github.com/raunow/jig
```
add `--features cloud` to enable just `cloud` only filters selections

> Requirements:
> 'cc' linker for compilation
> `sudo apt update && sudo apt install build-essential`

## Configuration

Supports global and local repository config files.  
If both exist, they are merged with the local taking priority.

See [example_config.toml](./example_config.toml)

Generate your configuration using:
```bash
jig init [--all]
```

## Usage

[![asciicast](https://asciinema.org/a/604710.png)](https://asciinema.org/a/604710)

```bash
$ jig help

A Jira CLI integration with Git

Usage: jig
       jig <COMMAND>

Commands:
  branch   Create and checkout branch using issue key with(out) summary as branch name
  comment  Create comment on a Jira Issue
  configs  List config file locations
  init     Initialise config file(s)
  log      Create a work log entry on a Jira issue
  move     Move ticket through transitions
  open     Open issue using BROWSER var
  upgrade  Download and install latest version (Cloud only)
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Initialise a workspace or global config file:
```bash
jig init
# --all to alter default behaviour
```
The workspace config file (`.jig.toml`) will be created at the root of the current repository or if not found, in the current directory.
The global file is created according to [XDG Conventions](https://docs.rs/etcetera/latest/etcetera/#conventions)

Be on or create a branch with an issue key in the name.  
```bash
jig branch
```

Work on that branch and commit as normal.

Log work/Comment progress as you work normally.  
```bash
# Optionally comment progress as you work.
jig comment "Note: Changed impl due to X"
# Create worklog as you finish up current session
jig log 1h --comment "Bug squashed"
# Transition ticket according to you workflow
jig move
```

That's it.

More Jira actions might come in the future.
