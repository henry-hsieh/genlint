use crate::types::Diagnostic;
use crate::util::{char_index_to_byte_range, coord_to_pos, pos_to_annotation};
use annotate_snippets::renderer::DecorStyle;
use annotate_snippets::{AnnotationKind, Group, Renderer, Snippet};
use genlint::util::severity_to_level;
use serde_partial::SerializePartial;
use std::io::{BufWriter, Write};

pub fn print_diagnostics_plain<W: Write>(writer: &mut BufWriter<W>, diagnostics: &[Diagnostic]) {
    let mut report = Vec::new();
    for diag in diagnostics {
        let char_pos = coord_to_pos(&diag.source, diag.source_lnum, diag.lnum, diag.col);
        let char_end_pos =
            coord_to_pos(&diag.source, diag.source_lnum, diag.end_lnum, diag.end_col);
        let (pos, _) = char_index_to_byte_range(&diag.source, char_pos);
        let (_, end_pos_exclusive) = char_index_to_byte_range(&diag.source, char_end_pos);
        let mut annotations = vec![pos_to_annotation(
            pos,
            end_pos_exclusive.saturating_sub(1),
            None,
            AnnotationKind::Primary,
        )];
        if let Some(helpers) = &diag.helpers {
            for helper in helpers.iter() {
                let char_pos =
                    coord_to_pos(&diag.source, diag.source_lnum, helper.lnum, helper.col);
                let char_end_pos = coord_to_pos(
                    &diag.source,
                    diag.source_lnum,
                    helper.end_lnum,
                    helper.end_col,
                );
                let (pos, _) = char_index_to_byte_range(&diag.source, char_pos);
                let (_, end_pos_exclusive) = char_index_to_byte_range(&diag.source, char_end_pos);
                annotations.push(pos_to_annotation(
                    pos,
                    end_pos_exclusive.saturating_sub(1),
                    Some(helper.message.as_str()),
                    AnnotationKind::Context,
                ));
            }
        };

        let message = Group::with_title(
            severity_to_level(diag.severity.as_str())
                .primary_title(&diag.message)
                .id(diag.code.as_str()),
        )
        .element(
            Snippet::source(diag.source.as_str())
                .line_start(diag.source_lnum + 1)
                .path(diag.file.as_str())
                .fold(true)
                .annotations(annotations),
        );

        report.push(message)
    }
    let renderer = Renderer::styled().decor_style(DecorStyle::Unicode);
    let _ = writer.write_all(&renderer.render(&report).into_bytes());
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
