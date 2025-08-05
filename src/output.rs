use crate::types::Diagnostic;
use crate::util::{coord_to_pos, pos_to_annotation};
use annotate_snippets::{Level, Renderer, Snippet};
use genlint::util::severity_to_level;
use serde_partial::SerializePartial;
use std::io::{BufWriter, Write};

pub fn print_diagnostics_plain<W: Write>(writer: &mut BufWriter<W>, diagnostics: &[Diagnostic]) {
    let renderer = Renderer::styled();
    for diag in diagnostics {
        let pos = coord_to_pos(&diag.source, diag.source_lnum, diag.lnum, diag.col);
        let end_pos = coord_to_pos(&diag.source, diag.source_lnum, diag.end_lnum, diag.end_col);
        let mut annotations = vec![pos_to_annotation(
            pos,
            end_pos,
            None,
            diag.severity.as_str(),
        )];
        if let Some(helpers) = &diag.helpers {
            for helper in helpers.iter() {
                let pos = coord_to_pos(&diag.source, diag.source_lnum, helper.lnum, helper.col);
                let end_pos = coord_to_pos(
                    &diag.source,
                    diag.source_lnum,
                    helper.end_lnum,
                    helper.end_col,
                );
                annotations.push(pos_to_annotation(
                    pos,
                    end_pos,
                    Some(helper.message.as_str()),
                    "information",
                ));
            }
        };

        let level = severity_to_level(diag.severity.as_str());
        let message = Level::title(level, &diag.message).snippet(
            Snippet::source(diag.source.as_str())
                .line_start(diag.source_lnum + 1)
                .origin(diag.file.as_str())
                .fold(true)
                .annotations(annotations),
        );
        writeln!(writer, "{}", renderer.render(message)).ok();
    }
}

pub fn print_diagnostics_json<W: Write>(writer: &mut BufWriter<W>, diagnostics: &[Diagnostic]) {
    let _ = serde_json::to_writer_pretty(
        writer,
        &diagnostics
            .iter()
            .map(|d| d.without_fields(|d| [d.source, d.source_lnum, d.helpers]))
            .collect::<Vec<_>>(),
    );
}

pub fn print_diagnostics_jsonl<W: Write>(writer: &mut BufWriter<W>, diagnostics: &[Diagnostic]) {
    for diag in diagnostics {
        let _ = serde_json::to_writer(
            &mut *writer,
            &diag.without_fields(|d| [d.source, d.source_lnum, d.helpers]),
        );
        let _ = writeln!(writer);
    }
}
