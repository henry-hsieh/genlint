use genlint::lint::lint_lines;
use genlint::types::{Diagnostic, LintOptions};
use genlint::util::coord_to_pos;
use std::io::Cursor;

fn run_lint(input: &str, opts: &LintOptions) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let reader = Cursor::new(input);
    lint_lines("<stdin>", reader, &mut diags, &opts);
    diags
}

#[test]
fn detects_mixed_indentation() {
    let src = "\t  let x = 5;\n";
    let opts = LintOptions {
        disables: Vec::new(),
        line_length: 120,
        consecutive_blank: 1,
    };
    let diags = run_lint(src, &opts);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].lnum, 0);
    assert_eq!(diags[0].end_lnum, 0);
    assert_eq!(diags[0].col, 1);
    assert_eq!(diags[0].end_col, 1);
    assert_eq!(diags[0].source, "   let x = 5;\n");
    assert_eq!(diags[0].source_lnum, 0);
    assert_eq!(diags[0].code, "mix-indent");
}

#[test]
fn detects_trailing_space() {
    let src = "let x = 5;  \nlet y = 10;\n";
    let opts = LintOptions {
        disables: Vec::new(),
        line_length: 120,
        consecutive_blank: 1,
    };
    let diags = run_lint(src, &opts);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].lnum, 0);
    assert_eq!(diags[0].end_lnum, 0);
    assert_eq!(diags[0].col, 10);
    assert_eq!(diags[0].end_col, 11);
    assert_eq!(diags[0].source, "let x = 5;  \n");
    assert_eq!(diags[0].source_lnum, 0);
    assert_eq!(diags[0].code, "trailing-space");
}

#[test]
fn detects_conflict_markers() {
    let src = "<<<<<<< HEAD\nlet x = 1;\n=======\nlet x = 2;\n>>>>>>>\n";
    let opts = LintOptions {
        disables: Vec::new(),
        line_length: 120,
        consecutive_blank: 1,
    };
    let diags = run_lint(src, &opts);
    let lnums: Vec<usize> = diags.iter().map(|d| d.lnum).collect();
    let end_lnums: Vec<usize> = diags.iter().map(|d| d.end_lnum).collect();
    let cols: Vec<usize> = diags.iter().map(|d| d.col).collect();
    let end_cols: Vec<usize> = diags.iter().map(|d| d.end_col).collect();
    let sources: Vec<&str> = diags.iter().map(|d| d.source.as_str()).collect();
    let source_lnums: Vec<usize> = diags.iter().map(|d| d.source_lnum).collect();
    let codes: Vec<&str> = diags.iter().map(|d| d.code.as_str()).collect();

    assert_eq!(diags.len(), 3);
    assert_eq!(
        lnums,
        [0, 2, 4]
    );
    assert_eq!(
        end_lnums,
        [0, 2, 4]
    );
    assert_eq!(
        cols,
        [0, 0, 0]
    );
    assert_eq!(
        end_cols,
        [11, 6, 6]
    );
    assert_eq!(
        sources,
        ["<<<<<<< HEAD\n", "=======\n", ">>>>>>>\n"]
    );
    assert_eq!(
        source_lnums,
        [0, 2, 4]
    );
    assert_eq!(
        codes,
        ["conflict-marker", "conflict-marker", "conflict-marker"]
    );
}

#[test]
fn detects_long_line() {
    let err_str = format!("let z = \"{:075}\";\n", 150);
    let src = format!("let x = 5;\nlet y = 10;\n{err_str}");
    let opts = LintOptions {
        disables: Vec::new(),
        line_length: 50,
        consecutive_blank: 1,
    };
    let diags = run_lint(src.as_str(), &opts);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].lnum, 2);
    assert_eq!(diags[0].end_lnum, 2);
    assert_eq!(diags[0].col, 50);
    assert_eq!(diags[0].end_col, 85);
    assert_eq!(diags[0].source, err_str.as_str());
    assert_eq!(diags[0].source_lnum, 2);
    assert_eq!(diags[0].code, "long-line");
}

#[test]
fn detects_control_char() {
    let err_str = "  let z = \"abc\x01\"\n";
    let src = format!("let x = 5;\nlet y = 10;\n{err_str}");
    let opts = LintOptions {
        disables: Vec::new(),
        line_length: 120,
        consecutive_blank: 1,
    };
    let diags = run_lint(src.as_str(), &opts);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].lnum, 2);
    assert_eq!(diags[0].end_lnum, 2);
    assert_eq!(diags[0].col, 14);
    assert_eq!(diags[0].end_col, 14);
    assert_eq!(diags[0].source, err_str);
    assert_eq!(diags[0].source_lnum, 2);
    assert_eq!(diags[0].code, "control-char");
}

#[test]
fn detects_consecutive_blank() {
    let err_str = vec!["\n\n\nlet x = 5;\n", "\n\n\n\nlet y = 10;\n", "let z = 15;\n\n\n\n\n\n"];
    let src = err_str.join("");
    let opts = LintOptions {
        disables: Vec::new(),
        line_length: 120,
        consecutive_blank: 2,
    };
    let diags = run_lint(src.as_str(), &opts);
    let lnums: Vec<usize> = diags.iter().map(|d| d.lnum).collect();
    let end_lnums: Vec<usize> = diags.iter().map(|d| d.end_lnum).collect();
    let cols: Vec<usize> = diags.iter().map(|d| d.col).collect();
    let end_cols: Vec<usize> = diags.iter().map(|d| d.end_col).collect();
    let sources: Vec<&str> = diags.iter().map(|d| d.source.as_str()).collect();
    let source_lnums: Vec<usize> = diags.iter().map(|d| d.source_lnum).collect();
    let codes: Vec<&str> = diags.iter().map(|d| d.code.as_str()).collect();
    let pos: Vec<usize> = diags.iter().map(|d| coord_to_pos(&d.source, d.source_lnum, d.lnum, d.col)).collect();
    let end_pos: Vec<usize> = diags.iter().map(|d| coord_to_pos(&d.source, d.source_lnum, d.end_lnum, d.end_col)).collect();

    assert_eq!(diags.len(), 3);
    assert_eq!(
        lnums,
        [0, 4, 10]
    );
    assert_eq!(
        end_lnums,
        [2, 7, 14]
    );
    assert_eq!(
        cols,
        [0, 0, 0]
    );
    assert_eq!(
        end_cols,
        [0, 0, 0]
    );
    assert_eq!(
        sources,
        [err_str[0], format!("let x = 5;\n{}", err_str[1]).as_str(), err_str[2]]
    );
    assert_eq!(
        source_lnums,
        [0, 3, 9]
    );
    assert_eq!(
        codes,
        ["consecutive-blank", "consecutive-blank", "consecutive-blank"]
    );
    assert_eq!(
        pos,
        [0, 11, 12]
    );
    assert_eq!(
        end_pos,
        [2, 14, 16]
    );
}
