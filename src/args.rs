use std::path::PathBuf;
use clap::{arg, value_parser, ArgAction, ArgGroup, Command, Arg};
use clap_complete::Shell;
use crate::types::{Format, DisableCheck};

pub fn build_cli() -> Command {
    Command::new("genlint")
        .version("0.1.0")
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
                .num_args(1..)
        )
        .arg(
            arg!(-f --"format" <FORMAT> "Output format")
                .value_parser(value_parser!(Format))
                .default_value("plain")
        )
        .arg(
            arg!(-o --"output" <FILE> "Output file path")
                .value_parser(value_parser!(PathBuf))
        )
        .arg(
            arg!(-d --"disable" <CHECKS> "Disable specific checks")
                .value_delimiter(',')
                .num_args(1..)
                .value_parser(value_parser!(DisableCheck))
        )
        .arg(
            arg!(-l --"max-line-length" <NUM> "Maximum allowed line length")
                .value_parser(value_parser!(usize))
                .default_value("120")
        )
        .arg(
            arg!(-c --"max-consecutive-blank" <NUM> "Maximum allowed consecutive blank lines")
                .value_parser(value_parser!(usize))
                .default_value("1")
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
        .group(ArgGroup::new("input-mode")
            .required(true)
            .args(["stdin", "input"]))
        .subcommand_negates_reqs(true)
}
