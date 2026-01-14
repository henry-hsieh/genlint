```text
A generic, configurable linter for multiple languages

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
  -d, --disable <CHECKS>...          Disable specific checks [possible values: mix-indent, trailing-space, conflict-marker, long-line, consecutive-blank, final-newline]
  -a, --text                         Treat all input as text, bypassing binary detection
  -l, --max-line-length <NUM>        Maximum allowed line length [default: 120]
  -c, --max-consecutive-blank <NUM>  Maximum allowed consecutive blank lines [default: 1]
      --max-errors <NUM>             Maximum number of errors to report (set to 0 for no limit) [default: 50]
      --max-warnings <NUM>           Maximum number of warnings to report (set to 0 for no limit) [default: 50]
      --max-info <NUM>               Maximum number of information to report (set to 0 for no limit) [default: 0]
  -h, --help                         Print help
  -V, --version                      Print version

```
