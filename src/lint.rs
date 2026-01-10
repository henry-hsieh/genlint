use std::cmp::max;
use std::io::BufRead;

use crate::types::{Diagnostic, DisableCheck::*, Helper, LintOptions, LintRunner};
use crate::util::{calculate_width, char_col_at_visual_width, find_non_space_col};

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
    let mut non_blank_lnum: isize = -1;
    let mut non_blank_line = String::new();
    let mut line_idx = 0;
    let mut trailing_blank_count: usize = 0;

    // Store data for the final newline check: (lnum, col, raw_line, ends_with_eol)
    let mut last_line_data: Option<(usize, usize, String, bool)> = None;

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

                if !opts.disables.contains(&MixIndent)
                    && runner.can_add_issue("warning")
                    && let Some(space_col) = trimmed.chars().position(|c| c == ' ')
                    && let Some(tab_col) = trimmed.chars().position(|c| c == '\t')
                    && {
                        let non_space_col = find_non_space_col(trimmed);
                        tab_col < non_space_col && space_col < non_space_col
                    }
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
                            space_col - 1
                        },
                    };
                    let diag = Diagnostic {
                        file: filename.to_string(),
                        lnum,
                        end_lnum: lnum,
                        col: if space_col == 0 { tab_col } else { space_col },
                        end_col: if space_col == 0 { tab_col } else { space_col },
                        severity: "warning".to_string(),
                        source: line.to_string(),
                        source_lnum: lnum,
                        code: "mix-indent".to_string(),
                        message: "Mixed tabs and whitespaces".to_string(),
                        helpers: Some(vec![helper]),
                    };
                    if !runner.add_diagnostic(opts, diag) {
                        return false;
                    }
                }

                if !opts.disables.contains(&TrailingSpace) && runner.can_add_issue("warning") {
                    let trimmed_trailing_space = trimmed.trim_end_matches([' ', '\t']);
                    if trimmed.len() > trimmed_trailing_space.len() {
                        let col = trimmed_trailing_space.chars().count();
                        let end_col = trimmed.chars().count() - 1;

                        let diag = Diagnostic {
                            file: filename.to_string(),
                            lnum,
                            end_lnum: lnum,
                            col,
                            end_col,
                            severity: "warning".to_string(),
                            source: line.to_string(),
                            source_lnum: lnum,
                            code: "trailing-space".to_string(),
                            message: "Trailing whitespaces or tabs".to_string(),
                            helpers: None,
                        };
                        if !runner.add_diagnostic(opts, diag) {
                            return false;
                        }
                    }
                }

                if !opts.disables.contains(&ConflictMarker)
                    && (trimmed.starts_with("<<<<<<<")
                        || trimmed.starts_with("=======")
                        || trimmed.starts_with(">>>>>>>"))
                {
                    if !runner.can_add_issue("error") {
                        return false; // Early terminate the linter when error counts exceed the limit
                    }

                    let diag = Diagnostic {
                        file: filename.to_string(),
                        lnum,
                        end_lnum: lnum,
                        col: 0,
                        end_col: trimmed.chars().count().saturating_sub(1),
                        severity: "error".to_string(),
                        source: line.to_string(),
                        source_lnum: lnum,
                        code: "conflict-marker".to_string(),
                        message: format!("Git conflict marker: {trimmed}"),
                        helpers: None,
                    };
                    if !runner.add_diagnostic(opts, diag) {
                        return false;
                    }
                }

                if !opts.disables.contains(&LongLine) && runner.can_add_issue("information") {
                    // Only check if line length is somewhat large to avoid cost on short lines
                    if trimmed.len() > opts.line_length / 4 {
                        let visual_width = calculate_width(trimmed);
                        if visual_width > opts.line_length {
                            let limit = char_col_at_visual_width(trimmed, opts.line_length);
                            let diag = Diagnostic {
                                file: filename.to_string(),
                                lnum,
                                end_lnum: lnum,
                                col: limit,
                                end_col: trimmed.chars().count().saturating_sub(1),
                                severity: "information".to_string(),
                                source: line.to_string(),
                                source_lnum: lnum,
                                code: "long-line".to_string(),
                                message: format!(
                                    "Too long line ({}/{})",
                                    visual_width, opts.line_length
                                ),
                                helpers: None,
                            };
                            if !runner.add_diagnostic(opts, diag) {
                                return false;
                            }
                        }
                    }
                }

                if !opts.disables.contains(&ConsecutiveBlank) && runner.can_add_issue("information")
                {
                    if !trimmed.is_empty() {
                        if trailing_blank_count > opts.consecutive_blank {
                            let mut helpers: Vec<Helper> = Vec::new();
                            if !non_blank_lnum.is_negative() {
                                helpers.push(Helper {
                                    message: "Previous non-blank line".to_string(),
                                    lnum: non_blank_lnum as usize,
                                    end_lnum: non_blank_lnum as usize,
                                    col: 0,
                                    end_col: non_blank_line.chars().count().saturating_sub(1),
                                });
                            }
                            helpers.push(Helper {
                                message: "Next non-blank line".to_string(),
                                lnum,
                                end_lnum: lnum,
                                col: 0,
                                end_col: trimmed.chars().count().saturating_sub(1),
                            });

                            let diag = Diagnostic {
                                file: filename.to_string(),
                                lnum: (non_blank_lnum + 1) as usize,
                                end_lnum: lnum - 1,
                                col: 0,
                                end_col: 0,
                                severity: "information".to_string(),
                                source: if non_blank_lnum < 0 {
                                    format!("{}{}\n", "\n".repeat(trailing_blank_count), trimmed)
                                } else {
                                    format!(
                                        "{}\n{}{}\n",
                                        non_blank_line,
                                        "\n".repeat(trailing_blank_count),
                                        trimmed
                                    )
                                },
                                source_lnum: max(0, non_blank_lnum) as usize,
                                code: "consecutive-blank".to_string(),
                                message: format!(
                                    "Too many consecutive blank lines ({}/{})",
                                    trailing_blank_count, opts.consecutive_blank
                                ),
                                helpers: Some(helpers),
                            };
                            if !runner.add_diagnostic(opts, diag) {
                                return false;
                            }
                        }
                        non_blank_lnum = lnum as isize;
                        non_blank_line = trimmed.to_string();
                        trailing_blank_count = 0;
                    } else {
                        trailing_blank_count += 1;
                    }
                }

                let raw_line = line.to_string();
                let char_col = trimmed.chars().count().saturating_sub(1);
                last_line_data = Some((lnum, char_col, raw_line, ends_with_eol));
            }
            Err(e) => {
                eprintln!("Error reading '{}': {}. Skipping.", filename, e);
                return false;
            }
        }
    }

    // Post-loop checks (FinalNewline and end-of-file ConsecutiveBlank)
    if let Some((lnum, col, raw_line, has_eol)) = last_line_data {
        let trimmed_last = &raw_line.trim_end_matches(['\r', '\n']);

        if !opts.disables.contains(&ConsecutiveBlank)
            && runner.can_add_issue("information")
            && trimmed_last.is_empty()
            && trailing_blank_count > opts.consecutive_blank
        {
            let helpers = if non_blank_lnum >= 0 {
                Some(vec![Helper {
                    message: "Previous non-blank line".to_string(),
                    lnum: non_blank_lnum as usize,
                    end_lnum: non_blank_lnum as usize,
                    col: 0,
                    end_col: non_blank_line.chars().count().saturating_sub(1),
                }])
            } else {
                None
            };

            let diag = Diagnostic {
                file: filename.to_string(),
                lnum: (non_blank_lnum + 1) as usize,
                end_lnum: lnum,
                col: 0,
                end_col: 0,
                severity: "information".to_string(),
                source: if non_blank_lnum < 0 {
                    "\n".repeat(trailing_blank_count)
                } else {
                    format!("{}\n{}", non_blank_line, "\n".repeat(trailing_blank_count))
                },
                source_lnum: max(0, non_blank_lnum) as usize,
                code: "consecutive-blank".to_string(),
                message: format!(
                    "Too many consecutive blank lines ({}/{})",
                    trailing_blank_count, opts.consecutive_blank
                ),
                helpers,
            };
            runner.add_diagnostic(opts, diag);
        }

        if !opts.disables.contains(&FinalNewline) && runner.can_add_issue("information") && !has_eol
        {
            let diag = Diagnostic {
                file: filename.to_string(),
                lnum,
                end_lnum: lnum,
                col,
                end_col: col,
                severity: "information".to_string(),
                source: raw_line,
                source_lnum: lnum,
                code: "final-newline".to_string(),
                message: "Missing final newline".to_string(),
                helpers: None,
            };
            runner.add_diagnostic(opts, diag);
        }
    }
    true
}
