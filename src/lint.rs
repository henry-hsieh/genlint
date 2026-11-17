use std::io::BufRead;

use crate::rules::{
    FileState, LineInfo, check_conflict_marker, check_consecutive_blank,
    check_consecutive_blank_eof, check_final_newline, check_long_line, check_mixed_indent,
    check_trailing_space,
};
use crate::types::{LintOptions, LintRunner};

pub fn lint_lines<R: BufRead>(
    filename: &str,
    mut reader: R,
    runner: &mut LintRunner,
    opts: &LintOptions,
) -> bool {
    // Check for binary content by peeking at the first 8KB, unless in text mode
    if !opts.text_mode {
        match reader.fill_buf() {
            Ok(peeked) if peeked.contains(&0) => {
                eprintln!(
                    "Binary file detected in '{}', skipping processing.",
                    filename
                );
                return false;
            }
            Err(e) => {
                eprintln!("Error reading '{}': {}. Skipping.", filename, e);
                return false;
            }
            _ => {}
        }
    }

    let mut buffer = String::with_capacity(1024);
    let mut state = FileState::new(filename.to_string());
    let mut line_idx = 0;

    loop {
        buffer.clear();
        match reader.read_line(&mut buffer) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let lnum = line_idx;
                line_idx += 1;
                let line = &buffer;

                let ends_with_eol = line.ends_with('\n') || line.ends_with('\r');
                let trimmed = line.trim_end_matches(['\r', '\n']);

                let ctx = LineInfo {
                    filename,
                    lnum,
                    line,
                    trimmed,
                };

                if !check_mixed_indent(&ctx, runner, opts) {
                    return false;
                }
                if !check_trailing_space(&ctx, runner, opts) {
                    return false;
                }
                if !check_conflict_marker(&ctx, &mut state, runner, opts) {
                    return false;
                }
                if !check_long_line(&ctx, runner, opts) {
                    return false;
                }
                if !check_consecutive_blank(&ctx, &mut state, runner, opts) {
                    return false;
                }

                // Store data for the final newline check: (lnum, col, raw_line, ends_with_eol)
                let raw_line = line.to_string();
                let char_col = trimmed.chars().count().saturating_sub(1);
                state.last_line_data = Some((lnum, char_col, raw_line, ends_with_eol));
            }
            Err(e) => {
                eprintln!("Error reading '{}': {}. Skipping.", filename, e);
                return false;
            }
        }
    }

    // Post-loop checks (ConsecutiveBlank and FinalNewline at EOF)
    if !check_consecutive_blank_eof(&state, runner, opts) {
        return false;
    }
    if !check_final_newline(&state, runner, opts) {
        return false;
    }
    true
}
