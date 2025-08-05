# genlint

![CI](https://github.com/henry-hsieh/genlint/actions/workflows/test.yml/badge.svg)
![Release](https://github.com/henry-hsieh/genlint/actions/workflows/release-please.yml/badge.svg)

A generic and flexible linter tool written in Rust. The genlint supports multiple rules, formats, and input sources.

---

## Features

- Check for common issues such as:
  - Mixed indentation
  - Trailing whitespace
  - Git conflict markers
  - Long lines
  - Control characters
  - Consecutive blank lines
  - Missing final newline
- Configurable rule disabling
- Input from stdin or multiple files
- Outputs in `plain`, `json`, or `jsonl` formats
- Shell completions for Bash, Zsh, Fish, and PowerShell

---

## Usage

### Command-Line Options
```text
Usage: genlint [OPTIONS] <--stdin|--input <FILES>...>
       genlint [OPTIONS] <COMMAND>

Commands:
  generate-completion  Generate shell completions
  help                 Print this message or the help of the given subcommand(s)

Options:
  -s, --stdin                        Read input from stdin
  -i, --input <FILES>...             Input file(s) to lint
  -e, --exclude <PATTERNS>...        Glob patterns to exclude
  -f, --format <FORMAT>              Output format [default: plain] [possible values: json, jsonl, plain]
  -o, --output <FILE>                Output file path
  -d, --disable <CHECKS>...          Disable specific checks [possible values: mix-indent, trailing-space, conflict-marker, long-line, control-char, consecutive-blank, final-newline]
  -l, --max-line-length <NUM>        Maximum allowed line length [default: 120]
  -c, --max-consecutive-blank <NUM>  Maximum allowed consecutive blank lines [default: 1]
  -h, --help                         Print help
  -V, --version                      Print version
```

### Example
```sh
# Lint all Rust files
genlint --input "src/**/*.rs" --format plain

# Lint from stdin
cat main.rs | genlint --stdin --format json

# Disable certain checks
genlint --input "src/**/*.rs" --disable long-lines control-chars
```

---

## Supported Rules

- `mixed-indent`: Detect mixed tabs and spaces
- `trailing-space`: Detect trailing whitespaces or tabs
- `conflict-marker`: Detect Git conflict markers
- `long-line`: Warn when line exceeds a max length (default: 120)
- `control-char`: Detect ASCII control characters
- `consecutive-blank`: Warn if more than two consecutive blank lines
- `final-newline`: Warn if missing newline at EOF

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
