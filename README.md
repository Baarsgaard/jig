# Jig

(Ji)ra (G)it<br>
Most if not all my work at $day_job is coordinated through and logged in Jira.<br>
additionally I'm not a fan of doing simple tasks in the Jira UI..

I looked at existing Jira CLI tools, but none solve my exact use case.<br>
Hence, [Jig](https://www.youtube.com/watch?v=3JcmQONgXJM)!

Jig is opinionated towards working with a healthy "Per issue" branching model, also known as "My workflow".<br>
It therefore includes options and features I need to support that.

Primarily:<br>
Creating new branches from Jira issues with(out) summaries.<br>
Quickly logging time and Commenting on the issue found in the branch name.<br>
Moving issues from one status to the next.

## Installation

Install Rustup to compile
```bash
git clone git@github.com:Raunow/jig.git
cd jig
cargo install --path . --locked
```

## Configuration

Supports global and local repository config files.<br>
If both exist, they are merged with the local taking priority.

See [example_config.toml](./example_config.toml)

Generate your configuration using:
```bash
jig init [--all]
```

## Usage

```bash
# See all options
jig --help
jig <cmd> --help
```

Initialise a workspace or global config file:
```bash
jig init
# --all to override defaults.
```
The workspace config file (`.jig.toml`) will be created at the root of the repository or if not found, in the current directory.
The global file is created according to [XDG Conventions](https://docs.rs/etcetera/latest/etcetera/#conventions)

Be on or create a branch with an issue key in the name.<br>
```bash
jig branch
```

Work on that branch and commit as normal.

Log work/Comment progress as you work normally.<br>
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
