# Jit

(J)ira G(it)
Most if not all my work at $day_job was coordinated or logged in Jira.
And I grow old using the Jira UI..

I looked at a couple of Jira CLI tools, none of them solve my exact problem.
Hence, Jit!

Jit is opinionated towards working with a healthy "Per issue" branching model, also known as "My workflow".
It therefore includes options and features I value.

Primarily:
Creating new branches using Jira issue key with(out) summaries.
Quickly logging time and Commenting on the issue found in the branch name.

## Configuration

Supports global and local repository config files.
If both exist, they are merge with the local taking priority.

See [example_config.toml](./example_config.toml)

Generate your configuraiton using:
```bash
jit init [-a]
```

## Usage

```bash
jit --help
```

`CD` into a repository.
Be on or create a branch with an issue key in the name.
`jit branch`

Work in the repository as normal.

Log work/Comment progress as you work normally.
`jit log/comment`

That's it.

More Jira actions might come in the future.
