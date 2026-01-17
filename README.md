# genlint

![CI](https://github.com/henry-hsieh/genlint/actions/workflows/release.yml/badge.svg)
![Release](https://github.com/henry-hsieh/genlint/actions/workflows/release-please.yml/badge.svg)

A generic and flexible linter tool written in Rust. The genlint supports multiple rules, formats, and input sources.

---

## Features

- Check for common issues such as:
  - Mixed indentation
  - Trailing whitespace
  - Conflict markers (configurable styles: [Git](https://git-scm.com/docs/git-merge.html#_how_conflicts_are_presented), [Jujutsu](https://docs.jj-vcs.dev/latest/conflicts/))
  - Long lines
  - Consecutive blank lines
  - Missing final newline
- Automatic binary file detection and skipping
- Configurable rule disabling
- Input from stdin or multiple files
- Outputs in `plain`, `json`, or `jsonl` formats
- Shell completions for Bash, Zsh, Fish, and PowerShell

---

## Usage

### Command-Line Options
See [HELP.md](HELP.md) for full CLI usage documentation.

### Example
```sh
# Lint all Rust files
genlint --input "src/**/*.rs" --format plain

# Lint from stdin
cat main.rs | genlint --stdin --format json

# Disable certain checks
genlint --input "src/**/*.rs" --disable long-line,consecutive-blank

# Use JJ conflict marker style
genlint --input "src/**/*.rs" --conflict-marker-style jj
```

---

## Supported Rules

- `mixed-indent`: Detect mixed tabs and spaces
- `trailing-space`: Detect trailing whitespaces or tabs
- `conflict-marker`: Detect conflict markers (configurable style: git, git-diff3, jj, jj-diff3, jj-snapshot)
- `long-line`: Warn when line exceeds a max length (default: 120)
- `consecutive-blank`: Warn if more than two consecutive blank lines
- `final-newline`: Warn if missing newline at EOF

## Binary File Handling

The genlint automatically detects binary files by checking for null bytes (`\0`) in the first 8KB of content.
Binary files are skipped by default to avoid processing non-text content.
Use the `--text` flag to force processing of files as text, even if binary content is detected.
Control characters (except newlines and carriage returns) are displayed as dots (`.`) in diagnostic output.

---

## Shell Completion

```sh
genlint generate-completion bash > _genlint
source ./_genlint
```

Replace `bash` with `zsh`, `fish`, or `powershell` as needed.

---

## License
The project is using [MIT License](LICENSE).
