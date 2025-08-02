use std::io::BufRead;
use std::cmp::max;

use crate::types::{Diagnostic, LintOptions, DisableCheck::*, Helper};
use crate::util::find_non_space_col;

pub fn lint_lines<R: BufRead>(filename: &str, mut reader: R, diagnostics: &mut Vec<Diagnostic>, opts: &LintOptions) {

    let mut lines = Vec::new();
    let mut buffer = String::new();
    while let Ok(bytes_read) = reader.read_line(&mut buffer) {
        if bytes_read == 0 {
            break;
        }
        lines.push(buffer.clone());
        buffer.clear();
    }

    let mut non_blank_lnum: isize = -1;
    let mut non_blank_lines = String::from("");
    for (i, line) in lines.iter().enumerate() {
        let lnum = i;
        let trimmed = line.trim_end_matches(['\r', '\n']).to_string();
        let eol_col = max(trimmed.len(), 1) - 1;
        let line_retab = line.replace('\t', " ");

        if !opts.disables.contains(&MixIndent) {
            if let Some(non_space_col) = find_non_space_col(&trimmed) &&
                let Some(space_col) = trimmed.find(" ") && 
                let Some(tab_col) = trimmed.find('\t') {
                if tab_col < non_space_col && space_col < non_space_col {
                    let helper = Helper {
                        message: format!("This line starts with {}", if space_col == 0 { "whitespaces" } else { "tabs" }),
                        lnum: lnum,
                        end_lnum: lnum,
                        col: 0,
                        end_col: if space_col == 0 { tab_col - 1 } else { space_col - 1 }
                    };
                    diagnostics.push(Diagnostic {
                        file: filename.to_string(),
                        lnum: lnum,
                        end_lnum: lnum,
                        col: max(space_col, tab_col),
                        end_col: max(space_col, tab_col),
                        severity: "warning".into(),
                        source: line_retab.clone(),
                        source_lnum: lnum,
                        code: "mix-indent".to_string(),
                        message: "Mixed tabs and whitespaces".to_string(),
                        helpers: Some(vec![helper]),
                    });
                }
            }
        }

        if !opts.disables.contains(&TrailingSpace) {
            let trimmed_trailing_space = line.trim_end_matches(['\r', '\n', ' ', '\t']);
            if trimmed.len() > trimmed_trailing_space.len() {
                diagnostics.push(Diagnostic {
                    file: filename.to_string(),
                    lnum: lnum,
                    end_lnum: lnum,
                    col: trimmed_trailing_space.len(),
                    end_col: eol_col,
                    severity: "warning".into(),
                    source: line_retab.clone(),
                    source_lnum: lnum,
                    code: "trailing-space".into(),
                    message: "Trailing whitespaces or tabs".into(),
                    helpers: None,
                });
            }
        }

        if !opts.disables.contains(&ConflictMarker) {
            if trimmed.starts_with("<<<<<<<")
                || trimmed.starts_with("=======")
                || trimmed.starts_with(">>>>>>>")
            {
                diagnostics.push(Diagnostic {
                    file: filename.to_string(),
                    lnum: lnum,
                    end_lnum: lnum,
                    col: 0,
                    end_col: eol_col,
                    severity: "error".into(),
                    source: line_retab.clone(),
                    source_lnum: lnum,
                    code: "conflict-marker".into(),
                    message: format!("Git conflict marker: {trimmed}"),
                    helpers: None,
                });
            }
        }

        if !opts.disables.contains(&LongLine) {
            if trimmed.len() > opts.line_length {
                diagnostics.push(Diagnostic {
                    file: filename.to_string(),
                    lnum: lnum,
                    end_lnum: lnum,
                    col: opts.line_length,
                    end_col: eol_col,
                    severity: "information".into(),
                    source: line_retab.clone(),
                    source_lnum: lnum,
                    code: "long-line".into(),
                    message: format!("Too long line ({}/{})", eol_col + 1, opts.line_length),
                    helpers: None,
                });
            }
        }

        if !opts.disables.contains(&ControlChar) {
            for (index, c) in trimmed.chars().enumerate() {
                if c.is_control() && c != '\t' {
                    diagnostics.push(Diagnostic {
                        file: filename.to_string(),
                        lnum: lnum,
                        end_lnum: lnum,
                        col: index,
                        end_col: index,
                        severity: "warning".into(),
                        source: line_retab.clone(),
                        source_lnum: lnum,
                        code: "control-char".into(),
                        message: "Line contains a control character".into(),
                        helpers: None,
                    });
                }
            }
        }

        if !opts.disables.contains(&ConsecutiveBlank) {
            if !trimmed.is_empty() {
                if (lnum as isize) - non_blank_lnum > (opts.consecutive_blank as isize) + 1 {
                    let mut helpers: Vec<Helper> = Vec::new();
                    if non_blank_lnum.is_positive() {
                        helpers.push(Helper {
                            message: "Previous non-blank line".to_string(),
                            lnum: non_blank_lnum as usize,
                            end_lnum: non_blank_lnum as usize,
                            col: 0,
                            end_col: non_blank_lines.clone().replace("\n", "").len() - 1
                        });
                    }
                    helpers.push(Helper {
                        message: "Next non-blank line".to_string(),
                        lnum: lnum,
                        end_lnum: lnum,
                        col: 0,
                        end_col: trimmed.len() - 1
                    });
                    diagnostics.push(Diagnostic {
                        file: filename.to_string(),
                        lnum: max(0, non_blank_lnum + 1) as usize,
                        end_lnum: lnum - 1,
                        col: 0,
                        end_col: 0,
                        severity: "information".into(),
                        source: format!("{}{}", non_blank_lines, line_retab),
                        source_lnum: max(0, non_blank_lnum) as usize,
                        code: "consecutive-blank".into(),
                        message: format!("Too many consecutive blank lines ({}/{})", (lnum as isize) - non_blank_lnum - 1, opts.consecutive_blank),
                        helpers: Some(helpers),
                    });
                }
                non_blank_lnum = lnum as isize;
                non_blank_lines = line_retab.clone();
            } else {
                non_blank_lines = format!("{}{}", non_blank_lines, line_retab);
            }
        }
    }
    if let Some(line) = lines.last() {
        let lnum = lines.len() - 1;
        let col = line.len() - 1;
        let trimmed = line.trim_end_matches(['\r', '\n']);
        let line_retab = line.replace('\t', " ");

        if !opts.disables.contains(&ConsecutiveBlank) {
            if trimmed.is_empty() {
                if (lnum as isize) + 1 - non_blank_lnum > (opts.consecutive_blank as isize) + 1 {
                    let helper = Helper {
                        message: "Previous non-blank line".to_string(),
                        lnum: non_blank_lnum as usize,
                        end_lnum: non_blank_lnum as usize,
                        col: 0,
                        end_col: non_blank_lines.clone().replace("\n", "").len() - 1
                    };
                    diagnostics.push(Diagnostic {
                        file: filename.to_string(),
                        lnum: max(0, non_blank_lnum + 1) as usize,
                        end_lnum: lnum,
                        col: 0,
                        end_col: 0,
                        severity: "information".into(),
                        source: non_blank_lines,
                        source_lnum: max(0, non_blank_lnum) as usize,
                        code: "consecutive-blank".to_string(),
                        message: format!("Too many consecutive blank lines ({}/{})", (lnum as isize) - non_blank_lnum, opts.consecutive_blank),
                        helpers: Some(vec![helper]),
                    });
                }
            }
        }

        if !opts.disables.contains(&FinalNewline) {
            if !line.ends_with('\n') && !line.ends_with('\r') {
                diagnostics.push(Diagnostic {
                    file: filename.to_string(),
                    lnum: lnum,
                    end_lnum: lnum,
                    col: col,
                    end_col: col,
                    severity: "information".into(),
                    source: line_retab.clone(),
                    source_lnum: lnum,
                    code: "final-newline".to_string(),
                    message: "Missing final newline".to_string(),
                    helpers: None,
                });
            }
        }
    }
}
