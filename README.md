# Jig

(Ji)ra (G)it<br>
Most if not all my work at $day_job was coordinated through or logged in Jira.<br>
And I'm not a massive fan of doing simple tasks in the Jira UI..

I looked at existing Jira CLI tools, but none of them solve my exact use case problem.<br>
Hence, [Jig!](https://www.youtube.com/watch?v=3JcmQONgXJM)

Jig is opinionated towards working with a healthy "Per issue" branching model, also known as "My workflow".<br>
It therefore includes options and features I value.

Primarily:<br>
Creating new branches using Jira issue key with(out) summaries.<br>
Quickly logging time and Commenting on the issue found in the branch name.

## Configuration

Supports global and local repository config files.<br>
If both exist, they are merge with the local taking priority.

See [example_config.toml](./example_config.toml)

Generate your configuration using:
```bash
jig init [-a]
```

## Usage

```bash
jig --help
```

`CD` into a repository.<br>
Be on or create a branch with an issue key in the name.<br>
`jig branch`

Work in the repository as normal.

Log work/Comment progress as you work normally.<br>
`jig log/comment`

That's it.

More Jira actions might come in the future.
