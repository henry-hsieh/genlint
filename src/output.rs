use std::io::{Write, BufWriter};
use annotate_snippets::{Level, Renderer, Snippet};
use genlint::util::severity_to_level;
use crate::types::Diagnostic;
use crate::util::coord_to_annotation;
use serde_json;
use serde_partial::SerializePartial;

pub fn print_diagnostics_plain<W: Write>(writer: &mut BufWriter<W>, diagnostics: &[Diagnostic]) {
    let renderer = Renderer::styled();
    for diag in diagnostics {
        let mut annotations = vec![coord_to_annotation(&diag.source, diag.source_lnum, diag.lnum, diag.col, diag.end_lnum, diag.end_col, None, &diag.severity.as_str())];
        if let Some(helpers) = &diag.helpers {
            for helper in helpers.iter() {
                annotations.push(coord_to_annotation(&diag.source, diag.source_lnum, helper.lnum, helper.col, helper.end_lnum, helper.end_col, Some(helper.message.as_str()), "information"));
            }
        };

        let level = severity_to_level(&diag.severity.as_str());
        let message = Level::title(level, &diag.message).snippet(
            Snippet::source(diag.source.as_str())
                .line_start(diag.source_lnum + 1)
                .origin(diag.file.as_str())
                .fold(true)
                .annotations(annotations)
        );
        writeln!(writer, "{}", renderer.render(message)).ok();
    }
}

pub fn print_diagnostics_json<W: Write>(writer: &mut BufWriter<W>, diagnostics: &[Diagnostic]) {
    let _ = serde_json::to_writer_pretty(writer, &diagnostics.into_iter().map(|d| d.without_fields(|d| [d.source, d.source_lnum, d.helpers])).collect::<Vec<_>>());
}

pub fn print_diagnostics_jsonl<W: Write>(writer: &mut BufWriter<W>, diagnostics: &[Diagnostic]) {
    for diag in diagnostics {
        let _ = serde_json::to_writer(&mut *writer, &diag.without_fields(|d| [d.source, d.source_lnum, d.helpers]));
        let _ = writeln!(writer);
    }
}
