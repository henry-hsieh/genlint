mod args;
mod lint;
mod output;
mod types;
mod util;

use clap_complete::{Shell, generate};
use glob::glob;
use std::fs::File;
use std::io::{self, BufReader, BufWriter};

use crate::args::build_cli;
use crate::lint::lint_lines;
use crate::output::{print_diagnostics_json, print_diagnostics_jsonl, print_diagnostics_plain};
use crate::types::{
    DisableCheck,
    DisableCheck::{ConsecutiveBlank, LongLine},
    Format, LintOptions, LintRunner,
};

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
    if disables.contains(&LongLine)
        && matches
            .value_source("max-line-length")
            .is_some_and(|src| src != clap::parser::ValueSource::DefaultValue)
    {
        eprintln!("Error: Cannot use --max-line-length when 'long-lines' is disabled.");
        std::process::exit(1);
    }
    if disables.contains(&ConsecutiveBlank)
        && matches
            .value_source("max-consecutive-blank")
            .is_some_and(|src| src != clap::parser::ValueSource::DefaultValue)
    {
        eprintln!(
            "Error: Cannot use --max-consecutive-blank when 'consecutive-blank' is disabled."
        );
        std::process::exit(1);
    }

    let max_line_length = *matches.get_one::<usize>("max-line-length").unwrap();
    let max_consecutive_blank = *matches.get_one::<usize>("max-consecutive-blank").unwrap();
    let max_errors = *matches.get_one::<usize>("max-errors").unwrap();
    let max_warnings = *matches.get_one::<usize>("max-warnings").unwrap();
    let lint_opts = LintOptions {
        disables,
        line_length: max_line_length,
        consecutive_blank: max_consecutive_blank,
        max_errors,
        max_warnings,
    };

    let mut runner = LintRunner {
        diagnostics: Vec::new(),
        error_count: 0,
        warning_count: 0,
        has_printed_error_limit: false,
        has_printed_warning_limit: false,
    };

    if matches.get_flag("stdin") {
        let stdin = io::stdin().lock();
        let reader = BufReader::new(stdin);
        lint_lines("<stdin>", reader, &mut runner, &lint_opts);
    }

    if let Some(inputs) = matches.get_many::<String>("input") {
        for pattern in inputs {
            for entry in glob(pattern).expect("Failed to read glob pattern") {
                let path = entry.unwrap();
                if let Ok(file) = File::open(&path) {
                    let reader = BufReader::new(file);
                    lint_lines(
                        path.to_string_lossy().as_ref(),
                        reader,
                        &mut runner,
                        &lint_opts,
                    );
                    if lint_opts.max_errors > 0 && runner.error_count >= lint_opts.max_errors {
                        break;
                    }
                }
            }
            if lint_opts.max_errors > 0 && runner.error_count >= lint_opts.max_errors {
                break;
            }
        }
    }

    let mut writer = BufWriter::new(std::io::stdout());
    match matches.get_one::<Format>("format") {
        Some(Format::Plain) | None => print_diagnostics_plain(&mut writer, &runner.diagnostics),
        Some(Format::Json) => print_diagnostics_json(&mut writer, &runner.diagnostics),
        Some(Format::Jsonl) => print_diagnostics_jsonl(&mut writer, &runner.diagnostics),
    }
}
