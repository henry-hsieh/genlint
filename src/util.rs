use annotate_snippets::{Level, Annotation};

pub fn coord_to_pos(source: &String, source_lnum: usize, lnum: usize, col: usize) -> usize {
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

pub fn find_non_space_col(source: &String) -> Option<usize> {
    for (index, c) in source.chars().enumerate() {
        if !c.is_whitespace() {
            return Some(index);
        }
    }
    None
}

pub fn coord_to_annotation<'a>(
    source: &String,
    source_lnum: usize,
    lnum: usize,
    col: usize,
    end_lnum: usize,
    end_col: usize,
    label: Option<&'a str>,
    severity: &str,
) -> Annotation<'a> {
    let pos = coord_to_pos(source, source_lnum, lnum, col);
    let end_pos = coord_to_pos(source, source_lnum, end_lnum, end_col);

    let level = severity_to_level(&severity);
    let annotation = level.span(pos..end_pos + 1);

    if let Some(label_str) = label {
        return annotation.label(label_str);
    }

    return annotation;
}


pub fn severity_to_level (severity: &str) -> Level {
    return match severity {
        "error" => Level::Error,
        "warning" => Level::Warning,
        "information" => Level::Info,
        "hint" => Level::Note,
        _ => todo!("Unknown severity level: {}", severity),
    };
}
