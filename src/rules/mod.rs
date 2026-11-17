use std::cmp::max;

use crate::enums::ConflictMarkerStyle;
use crate::enums::DisableCheck::{
    ConflictMarker, ConsecutiveBlank, FinalNewline, LongLine, MixIndent, TrailingSpace,
};
use crate::types::{Diagnostic, Helper, LintOptions, LintRunner};
use crate::util::{calculate_width, char_col_at_visual_width, find_non_space_col};

pub struct LineInfo<'a> {
    pub filename: &'a str,
    pub lnum: usize,
    pub line: &'a str,
    pub trimmed: &'a str,
}

pub struct FileState {
    pub filename: String,
    pub non_blank_lnum: isize,
    pub non_blank_line: String,
    pub trailing_blank_count: usize,
    pub last_line_data: Option<(usize, usize, String, bool)>,
    pub current_min_length: usize,
}

impl FileState {
    pub fn new(filename: String) -> Self {
        Self {
            filename,
            non_blank_lnum: -1,
            non_blank_line: String::new(),
            trailing_blank_count: 0,
            last_line_data: None,
            current_min_length: 7,
        }
    }
}

fn marker_chars_for(style: ConflictMarkerStyle) -> &'static [u8] {
    match style {
        ConflictMarkerStyle::Git => "<>=".as_bytes(),
        ConflictMarkerStyle::Jj => "<%+\\>".as_bytes(),
        ConflictMarkerStyle::GitDiff3 | ConflictMarkerStyle::JjDiff3 => "<>=|".as_bytes(),
        ConflictMarkerStyle::JjSnapshot => "<+->".as_bytes(),
    }
}

fn is_jj_style(style: ConflictMarkerStyle) -> bool {
    matches!(
        style,
        ConflictMarkerStyle::Jj | ConflictMarkerStyle::JjDiff3 | ConflictMarkerStyle::JjSnapshot
    )
}

fn is_conflict_marker(trimmed: &str, chars: &[u8], length: usize) -> bool {
    for &c in chars {
        let is_match = if trimmed.len() == length {
            trimmed.as_bytes().iter().all(|&b| b == c)
        } else if c != b'=' && trimmed.len() > length {
            let prefix = &trimmed.as_bytes()[..length];
            let next_char = trimmed.as_bytes()[length];
            next_char == b' ' && prefix.iter().all(|&b| b == c)
        } else {
            false
        };
        if is_match {
            return true;
        }
    }
    false
}

fn detect_conflict_marker(
    trimmed: &str,
    style: ConflictMarkerStyle,
    current_min_length: usize,
) -> Option<usize> {
    let chars = marker_chars_for(style);

    if is_jj_style(style) {
        for len in (11..=trimmed.len()).step_by(4) {
            if is_conflict_marker(trimmed, chars, len) {
                return Some(len);
            }
        }
    }

    if is_conflict_marker(trimmed, chars, current_min_length) {
        return Some(current_min_length);
    }
    None
}

pub fn check_mixed_indent(ctx: &LineInfo, runner: &mut LintRunner, opts: &LintOptions) -> bool {
    if opts.disables.contains(&MixIndent) || !runner.can_add_issue("warning") {
        return true;
    }

    if let Some(space_col) = ctx.trimmed.chars().position(|c| c == ' ')
        && let Some(tab_col) = ctx.trimmed.chars().position(|c| c == '\t')
        && {
            let non_space_col = find_non_space_col(ctx.trimmed);
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
            lnum: ctx.lnum,
            end_lnum: ctx.lnum,
            col: 0,
            end_col: if space_col == 0 {
                tab_col - 1
            } else {
                space_col - 1
            },
        };
        let diag = Diagnostic {
            file: ctx.filename.to_string(),
            lnum: ctx.lnum,
            end_lnum: ctx.lnum,
            col: if space_col == 0 { tab_col } else { space_col },
            end_col: if space_col == 0 { tab_col } else { space_col },
            severity: "warning".to_string(),
            source: ctx.line.to_string(),
            source_lnum: ctx.lnum,
            code: "mix-indent".to_string(),
            message: "Mixed tabs and whitespaces".to_string(),
            helpers: Some(vec![helper]),
        };
        return runner.add_diagnostic(opts, diag);
    }
    true
}

pub fn check_trailing_space(ctx: &LineInfo, runner: &mut LintRunner, opts: &LintOptions) -> bool {
    if opts.disables.contains(&TrailingSpace) || !runner.can_add_issue("warning") {
        return true;
    }

    let trimmed_trailing_space = ctx.trimmed.trim_end_matches([' ', '\t']);
    if ctx.trimmed.len() > trimmed_trailing_space.len() {
        let col = trimmed_trailing_space.chars().count();
        let end_col = ctx.trimmed.chars().count() - 1;

        let diag = Diagnostic {
            file: ctx.filename.to_string(),
            lnum: ctx.lnum,
            end_lnum: ctx.lnum,
            col,
            end_col,
            severity: "warning".to_string(),
            source: ctx.line.to_string(),
            source_lnum: ctx.lnum,
            code: "trailing-space".to_string(),
            message: "Trailing whitespaces or tabs".to_string(),
            helpers: None,
        };
        return runner.add_diagnostic(opts, diag);
    }
    true
}

pub fn check_conflict_marker(
    ctx: &LineInfo,
    state: &mut FileState,
    runner: &mut LintRunner,
    opts: &LintOptions,
) -> bool {
    if opts.disables.contains(&ConflictMarker) {
        return true;
    }

    let Some(marker_length) = detect_conflict_marker(
        ctx.trimmed,
        opts.conflict_marker_style,
        state.current_min_length,
    ) else {
        return true;
    };

    // For JJ styles, update min length if longer marker found
    if is_jj_style(opts.conflict_marker_style)
        && marker_length > state.current_min_length
        && marker_length % 4 == 3
    {
        runner.clear_conflict_markers();
        state.current_min_length = marker_length;
    }

    // Check if can add issue
    if !runner.can_add_issue("error") {
        if !is_jj_style(opts.conflict_marker_style) {
            return false; // Early terminate for non-JJ styles
        }
        // For JJ styles, continue processing
    } else {
        let style_name = match opts.conflict_marker_style {
            ConflictMarkerStyle::Git => "Git",
            ConflictMarkerStyle::GitDiff3 => "Git diff3",
            ConflictMarkerStyle::Jj => "Jujutsu",
            ConflictMarkerStyle::JjDiff3 => "Jujutsu diff3",
            ConflictMarkerStyle::JjSnapshot => "Jujutsu snapshot",
        };
        let diag = Diagnostic {
            file: ctx.filename.to_string(),
            lnum: ctx.lnum,
            end_lnum: ctx.lnum,
            col: 0,
            end_col: ctx.trimmed.chars().count().saturating_sub(1),
            severity: "error".to_string(),
            source: ctx.line.to_string(),
            source_lnum: ctx.lnum,
            code: "conflict-marker".to_string(),
            message: format!("{} conflict marker: {}", style_name, ctx.trimmed),
            helpers: None,
        };
        if !runner.add_diagnostic(opts, diag) {
            return false;
        }
    }
    true
}

pub fn check_long_line(ctx: &LineInfo, runner: &mut LintRunner, opts: &LintOptions) -> bool {
    if opts.disables.contains(&LongLine) || !runner.can_add_issue("information") {
        return true;
    }

    // Only check if line length is somewhat large to avoid cost on short lines
    if ctx.trimmed.len() > opts.line_length / 4 {
        let visual_width = calculate_width(ctx.trimmed);
        if visual_width > opts.line_length {
            let limit = char_col_at_visual_width(ctx.trimmed, opts.line_length);
            let diag = Diagnostic {
                file: ctx.filename.to_string(),
                lnum: ctx.lnum,
                end_lnum: ctx.lnum,
                col: limit,
                end_col: ctx.trimmed.chars().count().saturating_sub(1),
                severity: "information".to_string(),
                source: ctx.line.to_string(),
                source_lnum: ctx.lnum,
                code: "long-line".to_string(),
                message: format!("Too long line ({}/{})", visual_width, opts.line_length),
                helpers: None,
            };
            return runner.add_diagnostic(opts, diag);
        }
    }
    true
}

pub fn check_consecutive_blank(
    ctx: &LineInfo,
    state: &mut FileState,
    runner: &mut LintRunner,
    opts: &LintOptions,
) -> bool {
    if opts.disables.contains(&ConsecutiveBlank) || !runner.can_add_issue("information") {
        return true;
    }

    if !ctx.trimmed.is_empty() {
        if state.trailing_blank_count > opts.consecutive_blank {
            let mut helpers: Vec<Helper> = Vec::new();
            if !state.non_blank_lnum.is_negative() {
                helpers.push(Helper {
                    message: "Previous non-blank line".to_string(),
                    lnum: state.non_blank_lnum as usize,
                    end_lnum: state.non_blank_lnum as usize,
                    col: 0,
                    end_col: state.non_blank_line.chars().count().saturating_sub(1),
                });
            }
            helpers.push(Helper {
                message: "Next non-blank line".to_string(),
                lnum: ctx.lnum,
                end_lnum: ctx.lnum,
                col: 0,
                end_col: ctx.trimmed.chars().count().saturating_sub(1),
            });

            let diag = Diagnostic {
                file: ctx.filename.to_string(),
                lnum: (state.non_blank_lnum + 1) as usize,
                end_lnum: ctx.lnum - 1,
                col: 0,
                end_col: 0,
                severity: "information".to_string(),
                source: if state.non_blank_lnum < 0 {
                    format!(
                        "{}{}\n",
                        "\n".repeat(state.trailing_blank_count),
                        ctx.trimmed
                    )
                } else {
                    format!(
                        "{}\n{}{}\n",
                        state.non_blank_line,
                        "\n".repeat(state.trailing_blank_count),
                        ctx.trimmed
                    )
                },
                source_lnum: max(0, state.non_blank_lnum) as usize,
                code: "consecutive-blank".to_string(),
                message: format!(
                    "Too many consecutive blank lines ({}/{})",
                    state.trailing_blank_count, opts.consecutive_blank
                ),
                helpers: Some(helpers),
            };
            if !runner.add_diagnostic(opts, diag) {
                return false;
            }
        }
        state.non_blank_lnum = ctx.lnum as isize;
        state.non_blank_line = ctx.trimmed.to_string();
        state.trailing_blank_count = 0;
    } else {
        state.trailing_blank_count += 1;
    }
    true
}

pub fn check_consecutive_blank_eof(
    state: &FileState,
    runner: &mut LintRunner,
    opts: &LintOptions,
) -> bool {
    if opts.disables.contains(&ConsecutiveBlank) || !runner.can_add_issue("information") {
        return true;
    }

    if let Some((lnum, _, raw_line, _)) = &state.last_line_data {
        let trimmed_last = raw_line.trim_end_matches(['\r', '\n']);
        if trimmed_last.is_empty() && state.trailing_blank_count > opts.consecutive_blank {
            let helpers = if state.non_blank_lnum >= 0 {
                Some(vec![Helper {
                    message: "Previous non-blank line".to_string(),
                    lnum: state.non_blank_lnum as usize,
                    end_lnum: state.non_blank_lnum as usize,
                    col: 0,
                    end_col: state.non_blank_line.chars().count().saturating_sub(1),
                }])
            } else {
                None
            };

            let diag = Diagnostic {
                file: state.filename.clone(),
                lnum: (state.non_blank_lnum + 1) as usize,
                end_lnum: *lnum,
                col: 0,
                end_col: 0,
                severity: "information".to_string(),
                source: if state.non_blank_lnum < 0 {
                    "\n".repeat(state.trailing_blank_count)
                } else {
                    format!(
                        "{}\n{}",
                        state.non_blank_line,
                        "\n".repeat(state.trailing_blank_count)
                    )
                },
                source_lnum: max(0, state.non_blank_lnum) as usize,
                code: "consecutive-blank".to_string(),
                message: format!(
                    "Too many consecutive blank lines ({}/{})",
                    state.trailing_blank_count, opts.consecutive_blank
                ),
                helpers,
            };
            runner.add_diagnostic(opts, diag);
        }
    }
    true
}

pub fn check_final_newline(state: &FileState, runner: &mut LintRunner, opts: &LintOptions) -> bool {
    if opts.disables.contains(&FinalNewline) || !runner.can_add_issue("information") {
        return true;
    }

    let Some((lnum, col, raw_line, has_eol)) = &state.last_line_data else {
        return true;
    };
    if !*has_eol {
        let diag = Diagnostic {
            file: state.filename.clone(),
            lnum: *lnum,
            end_lnum: *lnum,
            col: *col,
            end_col: *col,
            severity: "information".to_string(),
            source: raw_line.clone(),
            source_lnum: *lnum,
            code: "final-newline".to_string(),
            message: "Missing final newline".to_string(),
            helpers: None,
        };
        return runner.add_diagnostic(opts, diag);
    }
    true
}
