mod args;
mod enums;
mod lint;
mod output;
mod types;
mod util;

use clap_complete::{Shell, generate};
use glob::glob;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Write};

use crate::types::DiagnosticType;

use crate::args::build_cli;
use crate::enums::{
    DisableCheck::{self, ConsecutiveBlank, LongLine},
    Format,
};
use crate::lint::lint_lines;
use crate::output::{print_diagnostics_json, print_diagnostics_jsonl, print_diagnostics_plain};
use crate::types::{LintOptions, LintRunner};

const SMALL_FILE_THRESHOLD: u64 = 1024 * 1024;
const SMALL_BUFFER_SIZE: usize = 64 * 1024;
const LARGE_BUFFER_SIZE: usize = 256 * 1024;

fn buffer_size_for_file(path: &std::path::Path) -> usize {
    match path.metadata() {
        Ok(metadata) => {
            if metadata.len() < SMALL_FILE_THRESHOLD {
                SMALL_BUFFER_SIZE
            } else {
                LARGE_BUFFER_SIZE
            }
        }
        Err(_) => SMALL_BUFFER_SIZE,
    }
}

fn print_diagnostics<W: Write>(
    matches: &clap::ArgMatches,
    runner: &LintRunner,
    writer: &mut BufWriter<W>,
) {
    match matches.get_one::<Format>("format") {
        Some(Format::Plain) | None => print_diagnostics_plain(writer, &runner.diagnostics),
        Some(Format::Json) => print_diagnostics_json(writer, &runner.diagnostics),
        Some(Format::Jsonl) => print_diagnostics_jsonl(writer, &runner.diagnostics),
    }
    writer.flush().unwrap();
}

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
    let max_info = *matches.get_one::<usize>("max-info").unwrap();
    let text_mode = matches.get_flag("text");
    let lint_opts = LintOptions {
        disables,
        line_length: max_line_length,
        consecutive_blank: max_consecutive_blank,
        max_errors,
        max_warnings,
        max_info,
        text_mode,
    };

    let mut runner = LintRunner::new();
    let mut writer = BufWriter::new(std::io::stdout());

    if matches.get_flag("stdin") {
        let stdin = io::stdin().lock();
        let reader = BufReader::new(stdin);
        if !lint_lines("<stdin>", reader, &mut runner, &lint_opts) {
            print_diagnostics(&matches, &runner, &mut writer);
            print_summary(&runner);
            return;
        }
    }

    if let Some(inputs) = matches.get_many::<String>("input") {
        for pattern in inputs {
            for entry in glob(pattern).expect("Failed to read glob pattern") {
                let path = entry.unwrap();
                if let Ok(file) = File::open(&path) {
                    let buffer_size = buffer_size_for_file(&path);
                    let reader = BufReader::with_capacity(buffer_size, file);
                    if !lint_lines(
                        path.to_string_lossy().as_ref(),
                        reader,
                        &mut runner,
                        &lint_opts,
                    ) {
                        print_diagnostics(&matches, &runner, &mut writer);
                        print_summary(&runner);
                        return;
                    }
                }
            }
        }
    }

    print_diagnostics(&matches, &runner, &mut writer);
    print_summary(&runner);
}

fn print_summary(runner: &LintRunner) {
    let (error_count, warning_count, info_count) = runner.diagnostic_counts();

    let error_note = if runner.limit_reached(&DiagnosticType::Error) {
        " (limit reached)"
    } else {
        ""
    };
    let warning_note = if runner.limit_reached(&DiagnosticType::Warning) {
        " (limit reached)"
    } else {
        ""
    };
    let info_note = if runner.limit_reached(&DiagnosticType::Information) {
        " (limit reached)"
    } else {
        ""
    };

    eprintln!(
        "\nFound {} errors{}, {} warnings{}, {} information{}",
        error_count, error_note, warning_count, warning_note, info_count, info_note
    );
}
