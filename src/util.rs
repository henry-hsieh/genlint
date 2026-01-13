use std::sync::LazyLock;

use annotate_snippets::{Annotation, AnnotationKind, Level};
use unicode_width::UnicodeWidthChar;

static ASCII_WIDTH: LazyLock<[u8; 128]> = LazyLock::new(|| {
    let mut arr = [1u8; 128];
    arr['\t' as usize] = 4;
    arr
});

pub fn calculate_width(s: &str) -> usize {
    s.chars()
        .map(|c| {
            if c.is_ascii() {
                ASCII_WIDTH[c as usize] as usize
            } else {
                UnicodeWidthChar::width(c).unwrap_or(0)
            }
        })
        .sum()
}

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

pub fn find_non_space_col(source: &str) -> usize {
    for (index, c) in source.chars().enumerate() {
        if !c.is_whitespace() {
            return index;
        }
    }
    source.chars().count()
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

pub fn char_col_at_visual_width(line: &str, width: usize) -> usize {
    let mut visual_width = 0;
    let mut char_count = 0;

    for ch in line.chars() {
        let ch_width = if ch.is_ascii() {
            ASCII_WIDTH[ch as usize] as usize
        } else {
            UnicodeWidthChar::width(ch).unwrap_or(0)
        };

        visual_width += ch_width;

        if visual_width > width {
            break;
        }

        char_count += 1;
    }

    char_count
}

pub fn char_index_to_byte_range(s: &str, char_index: usize) -> (usize, usize) {
    if let Some((start, ch)) = s.char_indices().nth(char_index) {
        let end = start + ch.len_utf8();
        (start, end)
    } else {
        (s.len(), s.len())
    }
}
