use crate::types::{DisableCheck, Format};
use clap::{Arg, ArgAction, ArgGroup, Command, arg, value_parser};
use clap_complete::Shell;
use std::path::PathBuf;

pub fn build_cli() -> Command {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    Command::new("genlint")
        .version(VERSION)
        .about("A generic, configurable linter for multiple languages")
        .subcommand_required(false)
        .arg_required_else_help(true)
        .arg(
            arg!(-s --"stdin" "Read input from stdin")
                .action(ArgAction::SetTrue)
                .conflicts_with("input")
                .conflicts_with("exclude")
                .group("input-mode"),
        )
        .arg(
            arg!(-i --"input" <FILES> "Input file(s) to lint")
                .value_delimiter(',')
                .num_args(1..)
                .group("input-mode"),
        )
        .arg(
            arg!(-e --"exclude" <PATTERNS> "Glob patterns to exclude")
                .value_delimiter(',')
                .num_args(1..),
        )
        .arg(
            arg!(-f --"format" <FORMAT> "Output format")
                .value_parser(value_parser!(Format))
                .default_value("plain"),
        )
        .arg(arg!(-o --"output" <FILE> "Output file path").value_parser(value_parser!(PathBuf)))
        .arg(
            arg!(-d --"disable" <CHECKS> "Disable specific checks")
                .value_delimiter(',')
                .num_args(1..)
                .value_parser(value_parser!(DisableCheck)),
        )
        .arg(
            arg!(-a --"text" "Treat all input as text, bypassing binary detection")
                .action(ArgAction::SetTrue),
        )
        .arg(
            arg!(-l --"max-line-length" <NUM> "Maximum allowed line length")
                .value_parser(value_parser!(usize))
                .default_value("120"),
        )
        .arg(
            arg!(-c --"max-consecutive-blank" <NUM> "Maximum allowed consecutive blank lines")
                .value_parser(value_parser!(usize))
                .default_value("1"),
        )
        .arg(
            arg!(--"max-errors" <NUM> "Maximum number of errors to report (set to 0 to disable)")
                .value_parser(value_parser!(usize))
                .default_value("50"),
        )
        .arg(
            arg!(--"max-warnings" <NUM> "Maximum number of warnings to report (set to 0 to disable)")
                .value_parser(value_parser!(usize))
                .default_value("50"),
        )
        .subcommand(
            Command::new("generate-completion")
                .about("Generate shell completions")
                .arg(
                    Arg::new("shell")
                        .value_parser(value_parser!(Shell))
                        .required(true)
                        .help("Shell type (bash, zsh, fish, and powershell.)"),
                ),
        )
        .group(
            ArgGroup::new("input-mode")
                .required(true)
                .args(["stdin", "input"]),
        )
        .subcommand_negates_reqs(true)
}
