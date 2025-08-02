mod args;
mod types;
mod output;
mod util;
mod lint;

use std::fs::File;
use std::io::{self, BufReader, BufWriter};
use clap_complete::{generate, Shell};
use glob::glob;

use crate::args::build_cli;
use crate::types::{DisableCheck, DisableCheck::{LongLine, ConsecutiveBlank}, Format, LintOptions};
use crate::output::{print_diagnostics_plain, print_diagnostics_json, print_diagnostics_jsonl};
use crate::lint::lint_lines;

fn main() {
    let cmd = build_cli();
    let matches = cmd.get_matches();

    if let Some(("generate-completion", sub_m)) = matches.subcommand() {
        let shell = *sub_m.get_one::<Shell>("shell").unwrap();
        let mut cmd = build_cli();
        generate(shell, &mut cmd, "genlint", &mut std::io::stdout());
        return;
    }

    let disables: Vec<_> = matches
        .get_many::<DisableCheck>("disable")
        .map(|vals| vals.cloned().collect())
        .unwrap_or_default();
    if disables.contains(&LongLine) && matches
        .value_source("max-line-length")
        .is_some_and(|src| src != clap::parser::ValueSource::DefaultValue)
    {
        eprintln!("Error: Cannot use --max-line-length when 'long-lines' is disabled.");
        std::process::exit(1);
    }
    if disables.contains(&ConsecutiveBlank) && matches
        .value_source("max-consecutive-blank")
        .is_some_and(|src| src != clap::parser::ValueSource::DefaultValue)
    {
        eprintln!("Error: Cannot use --max-consecutive-blank when 'consecutive-blank' is disabled.");
        std::process::exit(1);
    }

    let max_line_length = *matches.get_one::<usize>("max-line-length").unwrap();
    let max_consecutive_blank = *matches.get_one::<usize>("max-consecutive-blank").unwrap();
    let lint_opts = LintOptions {
        disables: disables,
        line_length: max_line_length,
        consecutive_blank: max_consecutive_blank
    };

    let mut diagnostics = Vec::new();

    if matches.get_flag("stdin") {
        let stdin = io::stdin().lock();
        let reader = BufReader::new(stdin);
        lint_lines("<stdin>", reader, &mut diagnostics, &lint_opts);
    }

    if let Some(inputs) = matches.get_many::<String>("input") {
        for pattern in inputs {
            for entry in glob(pattern).expect("Failed to read glob pattern") {
                if let Ok(path) = entry {
                    if let Ok(file) = File::open(&path) {
                        let reader = BufReader::new(file);
                        lint_lines(path.to_string_lossy().as_ref(), reader, &mut diagnostics, &lint_opts);
                    }
                }
            }
        }
    }

    let mut writer = BufWriter::new(std::io::stdout());
    match matches.get_one::<Format>("format") {
        Some(Format::Plain) | None => print_diagnostics_plain(&mut writer, &diagnostics),
        Some(Format::Json) => print_diagnostics_json(&mut writer, &diagnostics),
        Some(Format::Jsonl) => print_diagnostics_jsonl(&mut writer, &diagnostics),
    }
}
