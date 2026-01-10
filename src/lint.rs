use std::cmp::max;
use std::io::BufRead;
use unicode_width::UnicodeWidthChar;

use crate::types::{Diagnostic, DisableCheck::*, Helper, LintOptions, LintRunner};
use crate::util::{byte_col_at_visual_width, decorate_line, find_non_space_col};

fn handle_diagnostic_limit<F>(
    diagnostics: &mut Vec<Diagnostic>,
    diag: Diagnostic,
    max_limit: usize,
    count: &mut usize,
    has_printed_flag: &mut bool,
    message_fn: F,
) -> bool
where
    F: FnOnce() -> String,
{
    if max_limit == 0 || *count < max_limit {
        diagnostics.push(diag);
        *count += 1;
        if max_limit > 0 && *count == max_limit && !*has_printed_flag {
            eprintln!("{}", message_fn());
            *has_printed_flag = true;
        }
        true
    } else {
        false
    }
}

fn add_diagnostic(runner: &mut LintRunner, opts: &LintOptions, diag: Diagnostic) -> bool {
    match diag.severity.as_str() {
        "error" => handle_diagnostic_limit(
            &mut runner.diagnostics,
            diag,
            opts.max_errors,
            &mut runner.error_count,
            &mut runner.has_printed_error_limit,
            || {
                format!(
                    "found {} errors, please fix the errors or increase the --max-errors limit",
                    opts.max_errors
                )
            },
        ),
        "warning" => {
            let _ = handle_diagnostic_limit(
                &mut runner.diagnostics,
                diag,
                opts.max_warnings,
                &mut runner.warning_count,
                &mut runner.has_printed_warning_limit,
                || {
                    format!(
                        "found {} warnings, please fix the warnings or increase the --max-warnings limit",
                        opts.max_warnings
                    )
                },
            );
            true
        }
        _ => {
            runner.diagnostics.push(diag);
            true
        }
    }
}

// Helper to calculate display width without allocating a retabbed string
fn calculate_width(s: &str) -> usize {
    s.chars()
        .map(|c| {
            if c == '\t' {
                4
            } else {
                UnicodeWidthChar::width(c).unwrap_or(0)
            }
        })
        .sum()
}

pub fn lint_lines<R: BufRead>(
    filename: &str,
    mut reader: R,
    runner: &mut LintRunner,
    opts: &LintOptions,
) {
    // Check for binary content by peeking at the first 8KB, unless in text mode
    if !opts.text_mode {
        match reader.fill_buf() {
            Ok(peeked) if peeked.contains(&0) => {
                eprintln!(
                    "Binary file detected in '{}', skipping processing.",
                    filename
                );
                return;
            }
            Err(e) => {
                eprintln!("Error reading '{}': {}. Skipping.", filename, e);
                return;
            }
            _ => {}
        }
    }

    let mut buffer = String::new();
    let mut non_blank_lnum: isize = -1;
    let mut non_blank_lines = String::new();
    let mut line_idx = 0;

    // Store data for the final newline check: (lnum, col, decorated_line, ends_with_eol)
    let mut last_line_data: Option<(usize, usize, String, bool)> = None;

    loop {
        buffer.clear();
        match reader.read_line(&mut buffer) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let lnum = line_idx;
                line_idx += 1;
                let line = &buffer;

                // Determine if the original line had an EOL character
                let ends_with_eol = line.ends_with('\n') || line.ends_with('\r');
                let trimmed = line.trim_end_matches(['\r', '\n']);

                let tab_count = trimmed.matches('\t').count();
                // Since decorate_line replaces 1 byte '\t' with 4 bytes spaces, length increases by 3 per tab.
                let eol_col = max(trimmed.len() + tab_count * 3, 1) - 1;

                let decorated_line = decorate_line(line);

                if !opts.disables.contains(&MixIndent)
                    && let Some(non_space_col) = find_non_space_col(trimmed)
                    && let Some(space_col) = trimmed.find(' ')
                    && let Some(tab_col) = trimmed.find('\t')
                    && tab_col < non_space_col
                    && space_col < non_space_col
                {
                    let helper = Helper {
                        message: format!(
                            "This line starts with {}",
                            if space_col == 0 {
                                "whitespaces"
                            } else {
                                "tabs"
                            }
                        ),
                        lnum,
                        end_lnum: lnum,
                        col: 0,
                        end_col: if space_col == 0 {
                            tab_col - 1
                        } else {
                            space_col * 4 - 1
                        },
                    };
                    let diag = Diagnostic {
                        file: filename.to_string(),
                        lnum,
                        end_lnum: lnum,
                        col: if space_col == 0 {
                            tab_col
                        } else {
                            space_col * 4
                        },
                        end_col: if space_col == 0 {
                            tab_col + 3
                        } else {
                            space_col * 4
                        },
                        severity: "warning".into(),
                        source: decorated_line.clone(),
                        source_lnum: lnum,
                        code: "mix-indent".to_string(),
                        message: "Mixed tabs and whitespaces".to_string(),
                        helpers: Some(vec![helper]),
                    };
                    let _ = add_diagnostic(runner, opts, diag);
                }

                if !opts.disables.contains(&TrailingSpace) {
                    let trimmed_trailing_space = trimmed.trim_end_matches([' ', '\t']);
                    if trimmed.len() > trimmed_trailing_space.len() {
                        // Calculate col for diagnostic. The trailing space starts after `trimmed_trailing_space`.
                        // We need to account for tabs in the prefix part to get the correct visual/expanded column.
                        let prefix_tabs = trimmed_trailing_space.matches('\t').count();
                        let col = trimmed_trailing_space.len() + prefix_tabs * 3;

                        let diag = Diagnostic {
                            file: filename.to_string(),
                            lnum,
                            end_lnum: lnum,
                            col,
                            end_col: eol_col,
                            severity: "warning".into(),
                            source: decorated_line.clone(),
                            source_lnum: lnum,
                            code: "trailing-space".into(),
                            message: "Trailing whitespaces or tabs".into(),
                            helpers: None,
                        };
                        let _ = add_diagnostic(runner, opts, diag);
                    }
                }

                if !opts.disables.contains(&ConflictMarker)
                    && (trimmed.starts_with("<<<<<<<")
                        || trimmed.starts_with("=======")
                        || trimmed.starts_with(">>>>>>>"))
                {
                    let diag = Diagnostic {
                        file: filename.to_string(),
                        lnum,
                        end_lnum: lnum,
                        col: 0,
                        end_col: eol_col,
                        severity: "error".into(),
                        source: decorated_line.clone(),
                        source_lnum: lnum,
                        code: "conflict-marker".into(),
                        message: format!("Git conflict marker: {trimmed}"),
                        helpers: None,
                    };
                    if !add_diagnostic(runner, opts, diag) {
                        return;
                    }
                }

                if !opts.disables.contains(&LongLine)
                    && let limit = byte_col_at_visual_width(&decorated_line, opts.line_length)
                    && eol_col > limit
                {
                    runner.diagnostics.push(Diagnostic {
                        file: filename.to_string(),
                        lnum,
                        end_lnum: lnum,
                        col: limit,
                        end_col: eol_col,
                        severity: "information".into(),
                        source: decorated_line.clone(),
                        source_lnum: lnum,
                        code: "long-line".into(),
                        message: format!(
                            "Too long line ({}/{})",
                            calculate_width(trimmed),
                            opts.line_length
                        ),
                        helpers: None,
                    });
                }

                if !opts.disables.contains(&ConsecutiveBlank) {
                    if !trimmed.is_empty() {
                        if (lnum as isize) - non_blank_lnum > (opts.consecutive_blank as isize) + 1
                        {
                            let mut helpers: Vec<Helper> = Vec::new();
                            if non_blank_lnum.is_positive() {
                                helpers.push(Helper {
                                    message: "Previous non-blank line".to_string(),
                                    lnum: non_blank_lnum as usize,
                                    end_lnum: non_blank_lnum as usize,
                                    col: 0,
                                    end_col: non_blank_lines.clone().replace('\n', "").len() - 1,
                                });
                            }
                            helpers.push(Helper {
                                message: "Next non-blank line".to_string(),
                                lnum,
                                end_lnum: lnum,
                                col: 0,
                                end_col: trimmed.len() - 1,
                            });
                            runner.diagnostics.push(Diagnostic {
                                file: filename.to_string(),
                                lnum: max(0, non_blank_lnum + 1) as usize,
                                end_lnum: lnum - 1,
                                col: 0,
                                end_col: 0,
                                severity: "information".into(),
                                source: format!("{non_blank_lines}{decorated_line}"),
                                source_lnum: max(0, non_blank_lnum) as usize,
                                code: "consecutive-blank".to_string(),
                                message: format!(
                                    "Too many consecutive blank lines ({}/{})",
                                    (lnum as isize) - non_blank_lnum - 1,
                                    opts.consecutive_blank
                                ),
                                helpers: Some(helpers),
                            });
                        }
                        non_blank_lnum = lnum as isize;
                        non_blank_lines = decorated_line.clone();
                    } else {
                        non_blank_lines.push_str(&decorated_line);
                    }
                }

                last_line_data = Some((lnum, eol_col, decorated_line, ends_with_eol));
            }
            Err(e) => {
                eprintln!("Error reading '{}': {}. Skipping.", filename, e);
                return;
            }
        }
    }

    // Post-loop checks (FinalNewline and end-of-file ConsecutiveBlank)
    if let Some((lnum, col, decorated_line, has_eol)) = last_line_data {
        let trimmed_last = decorated_line.trim_end_matches(['\r', '\n']);

        if !opts.disables.contains(&ConsecutiveBlank)
            && trimmed_last.is_empty()
            && (lnum as isize) + 1 - non_blank_lnum > (opts.consecutive_blank as isize) + 1
        {
            let helper = Helper {
                message: "Previous non-blank line".to_string(),
                lnum: non_blank_lnum as usize,
                end_lnum: non_blank_lnum as usize,
                col: 0,
                end_col: non_blank_lines.clone().replace('\n', "").len() - 1,
            };
            runner.diagnostics.push(Diagnostic {
                file: filename.to_string(),
                lnum: max(0, non_blank_lnum + 1) as usize,
                end_lnum: lnum,
                col: 0,
                end_col: 0,
                severity: "information".into(),
                source: non_blank_lines,
                source_lnum: max(0, non_blank_lnum) as usize,
                code: "consecutive-blank".to_string(),
                message: format!(
                    "Too many consecutive blank lines ({}/{})",
                    (lnum as isize) - non_blank_lnum,
                    opts.consecutive_blank
                ),
                helpers: Some(vec![helper]),
            });
        }

        if !opts.disables.contains(&FinalNewline) && !has_eol {
            runner.diagnostics.push(Diagnostic {
                file: filename.to_string(),
                lnum,
                end_lnum: lnum,
                col,
                end_col: col,
                severity: "information".into(),
                source: decorated_line,
                source_lnum: lnum,
                code: "final-newline".to_string(),
                message: "Missing final newline".to_string(),
                helpers: None,
            });
        }
    }
}
