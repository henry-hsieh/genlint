use annotate_snippets::{Annotation, AnnotationKind, Level};
use unicode_width::UnicodeWidthChar;

pub fn coord_to_pos(source: &str, source_lnum: usize, lnum: usize, col: usize) -> usize {
    let mut cur_lnum = source_lnum;
    if cur_lnum == lnum {
        return col;
    }
    for (index, c) in source.chars().enumerate() {
        if c == '\n' {
            cur_lnum += 1;
            if cur_lnum == lnum {
                return index + col + 1;
            }
        }
    }
    source.len().saturating_sub(1)
}

pub fn find_non_space_col(source: &str) -> Option<usize> {
    for (index, c) in source.chars().enumerate() {
        if !c.is_whitespace() {
            return Some(index);
        }
    }
    None
}

pub fn pos_to_annotation<'a>(
    pos: usize,
    end_pos: usize,
    label: Option<&'a str>,
    kind: AnnotationKind,
) -> Annotation<'a> {
    let annotation = kind.span(pos..end_pos + 1);

    if let Some(label_str) = label {
        return annotation.label(label_str);
    }

    annotation
}

#[allow(dead_code)]
pub fn severity_to_level(severity: &str) -> Level<'_> {
    match severity {
        "error" => Level::ERROR,
        "warning" => Level::WARNING,
        "information" => Level::INFO,
        "hint" => Level::NOTE,
        "help" => Level::HELP,
        _ => todo!("Unknown severity level: {}", severity),
    }
}

pub fn byte_col_at_visual_width(line: &str, width: usize) -> usize {
    let mut visual_width = 0;
    let mut last_index = line.len() - 1;

    for (i, ch) in line.char_indices() {
        let ch_width = if ch == '\t' {
            4
        } else {
            UnicodeWidthChar::width(ch).unwrap_or(0)
        };

        visual_width += ch_width;

        if visual_width > width {
            last_index = i;
            break;
        }
    }

    last_index
}
