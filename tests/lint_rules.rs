use genlint::enums::ConflictMarkerStyle;
use genlint::lint::lint_lines;
use genlint::types::{Diagnostic, LintOptions, LintRunner};
use genlint::util::coord_to_pos;
use std::io::Cursor;

fn default_opts() -> LintOptions {
    LintOptions {
        disables: Vec::new(),
        line_length: 120,
        consecutive_blank: 1,
        max_errors: 0,
        max_warnings: 0,
        max_info: 0,
        text_mode: false,
        conflict_marker_style: ConflictMarkerStyle::Git,
    }
}

fn run_lint(input: &str, opts: &LintOptions) -> Vec<Diagnostic> {
    let mut runner = LintRunner::new();
    let reader = Cursor::new(input);
    lint_lines("<stdin>", reader, &mut runner, opts);
    runner.diagnostics
}

#[test]
fn detects_mixed_indentation_tab() {
    let src = "\t  let x = 5;\n";
    let diags = run_lint(src, &default_opts());
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].lnum, 0);
    assert_eq!(diags[0].end_lnum, 0);
    assert_eq!(diags[0].col, 1);
    assert_eq!(diags[0].end_col, 1);
    assert_eq!(diags[0].source, "\t  let x = 5;\n");
    assert_eq!(diags[0].source_lnum, 0);
    assert_eq!(diags[0].code, "mix-indent");
}

#[test]
fn detects_mixed_indentation_space() {
    let src = "  \tlet x = 5;\n";
    let diags = run_lint(src, &default_opts());
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].lnum, 0);
    assert_eq!(diags[0].end_lnum, 0);
    assert_eq!(diags[0].col, 2);
    assert_eq!(diags[0].end_col, 2);
    assert_eq!(diags[0].source, "  \tlet x = 5;\n");
    assert_eq!(diags[0].source_lnum, 0);
    assert_eq!(diags[0].code, "mix-indent");
}

#[test]
fn detects_mixed_indentation_unicode() {
    let src = " \t α\n";
    let diags = run_lint(src, &default_opts());
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].lnum, 0);
    assert_eq!(diags[0].end_lnum, 0);
    assert_eq!(diags[0].col, 1); // Character index of the tab
    assert_eq!(diags[0].end_col, 1);
    assert_eq!(diags[0].source, " \t α\n");
    assert_eq!(diags[0].source_lnum, 0);
    assert_eq!(diags[0].code, "mix-indent");
}

#[test]
fn detects_mixed_indentation_whitespace_only() {
    let mut opts = default_opts();
    opts.disables
        .push(genlint::enums::DisableCheck::TrailingSpace);
    let src = "\t \n";
    let diags = run_lint(src, &opts);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].lnum, 0);
    assert_eq!(diags[0].end_lnum, 0);
    assert_eq!(diags[0].col, 1); // Position of the tab
    assert_eq!(diags[0].end_col, 1);
    assert_eq!(diags[0].source, "\t \n");
    assert_eq!(diags[0].source_lnum, 0);
    assert_eq!(diags[0].code, "mix-indent");
}

#[test]
fn detects_trailing_space() {
    let src = "let x = 5;  \nlet y = 10;\n";
    let diags = run_lint(src, &default_opts());
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].lnum, 0);
    assert_eq!(diags[0].end_lnum, 0);
    assert_eq!(diags[0].col, 10);
    assert_eq!(diags[0].end_col, 11);
    assert_eq!(diags[0].source, "let x = 5;  \n");
    assert_eq!(diags[0].source_lnum, 0);
    assert_eq!(diags[0].code, "trailing-space");
}

const CONFLICT_MARKER_SOURCE_SHORT: &str = r"should not match below
--no space after marker--
<<<<<<<
|||||||
%%%%%%%
+++++++
-------
\\\\\\\
>>>>>>>
--character number incorrect (>7)--
<<<<<<<< Error
|||||||| Error
%%%%%%%% Error
========
++++++++ Error
-------- Error
\\\\\\\\ Error
>>>>>>>> Error
--character number incorrect (<7)--
<<< E
||| E
%%% E
===
+++ E
--- E
\\\ E
>>> E
--character after `=`--
======= Error
should not match above

<<<<<<< git/git-diff3/jj/jj-snapshot/jj-diff3
||||||| git-diff3/jj-diff3
%%%%%%% jj
git/git-diff3/jj-diff3:
=======
+++++++ jj/jj-snapshot
------- jj-snapshot
\\\\\\\ jj
>>>>>>> git/git-diff3/jj/jj-snapshot/jj-diff3
";
const CONFLICT_MARKER_SOURCE_LONG: &str = r"should not match or update minimum length below
--no space after marker--
<<<<<<<<<<<<<<<
|||||||||||||||
%%%%%%%%%%%%%%%
+++++++++++++++
---------------
\\\\\\\\\\\\\\\
>>>>>>>>>>>
--character number incorrect (>15)--
<<<<<<<<<<<<<<<< Error
|||||||||||||||| Error
%%%%%%%%%%%%%%%% Error
================
++++++++++++++++ Error
---------------- Error
\\\\\\\\\\\\\\\\ Error
>>>>>>>>>>>>>>>> Error
--character after `=`--
=============== Error
should not match or update minimum length above

update minimum length for jj variants below
<<<<<<<<<<< jj/jj-snapshot/jj-diff3
||||||||||| jj-diff3
%%%%%%%%%%% jj
jj-diff3:
===========
+++++++++++ jj/jj-snapshot
----------- jj-snapshot
\\\\\\\\\\\ jj
>>>>>>>>>>> jj/jj-snapshot/jj-diff3

should be only matched by git variants below
<<<<<<< git/git-diff3
||||||| git-diff3
%%%%%%% jj
git/git-diff3:
=======
>>>>>>> git/git-diff3
";

#[test]
fn detects_git_conflict_markers() {
    let src = &format!(
        "{}{}",
        CONFLICT_MARKER_SOURCE_SHORT, CONFLICT_MARKER_SOURCE_LONG
    );
    let diags = run_lint(src, &default_opts());
    let lnums: Vec<usize> = diags.iter().map(|d| d.lnum).collect();
    let end_lnums: Vec<usize> = diags.iter().map(|d| d.end_lnum).collect();
    let cols: Vec<usize> = diags.iter().map(|d| d.col).collect();
    let end_cols: Vec<usize> = diags.iter().map(|d| d.end_col).collect();
    let sources: Vec<&str> = diags.iter().map(|d| d.source.as_str()).collect();
    let source_lnums: Vec<usize> = diags.iter().map(|d| d.source_lnum).collect();

    assert_eq!(diags.len(), 6);
    assert_eq!(lnums, [31, 35, 39, 74, 78, 79]);
    assert_eq!(end_lnums, [31, 35, 39, 74, 78, 79]);
    assert_eq!(cols, [0, 0, 0, 0, 0, 0]);
    assert_eq!(end_cols, [44, 6, 44, 20, 6, 20]);
    assert_eq!(
        sources,
        [
            "<<<<<<< git/git-diff3/jj/jj-snapshot/jj-diff3\n",
            "=======\n",
            ">>>>>>> git/git-diff3/jj/jj-snapshot/jj-diff3\n",
            "<<<<<<< git/git-diff3\n",
            "=======\n",
            ">>>>>>> git/git-diff3\n"
        ]
    );
    assert_eq!(source_lnums, [31, 35, 39, 74, 78, 79]);
    for diag in &diags {
        assert_eq!(diag.code, "conflict-marker");
        assert!(diag.message.contains("Git conflict marker"));
    }
}

#[test]
fn detects_git_diff3_conflict_markers() {
    let src = &format!(
        "{}{}",
        CONFLICT_MARKER_SOURCE_SHORT, CONFLICT_MARKER_SOURCE_LONG
    );
    let opts = LintOptions {
        conflict_marker_style: ConflictMarkerStyle::GitDiff3,
        ..default_opts()
    };
    let diags = run_lint(src, &opts);
    let lnums: Vec<usize> = diags.iter().map(|d| d.lnum).collect();
    let end_lnums: Vec<usize> = diags.iter().map(|d| d.end_lnum).collect();
    let cols: Vec<usize> = diags.iter().map(|d| d.col).collect();
    let end_cols: Vec<usize> = diags.iter().map(|d| d.end_col).collect();
    let sources: Vec<&str> = diags.iter().map(|d| d.source.as_str()).collect();
    let source_lnums: Vec<usize> = diags.iter().map(|d| d.source_lnum).collect();

    assert_eq!(diags.len(), 8);
    assert_eq!(lnums, [31, 32, 35, 39, 74, 75, 78, 79]);
    assert_eq!(end_lnums, [31, 32, 35, 39, 74, 75, 78, 79]);
    assert_eq!(cols, [0, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(end_cols, [44, 25, 6, 44, 20, 16, 6, 20]);
    assert_eq!(
        sources,
        [
            "<<<<<<< git/git-diff3/jj/jj-snapshot/jj-diff3\n",
            "||||||| git-diff3/jj-diff3\n",
            "=======\n",
            ">>>>>>> git/git-diff3/jj/jj-snapshot/jj-diff3\n",
            "<<<<<<< git/git-diff3\n",
            "||||||| git-diff3\n",
            "=======\n",
            ">>>>>>> git/git-diff3\n"
        ]
    );
    assert_eq!(source_lnums, [31, 32, 35, 39, 74, 75, 78, 79]);
    for diag in &diags {
        assert_eq!(diag.code, "conflict-marker");
        assert!(diag.message.contains("Git diff3 conflict marker"));
    }
}

#[test]
fn detects_jj_conflict_markers_static() {
    let src = &CONFLICT_MARKER_SOURCE_SHORT;
    let opts = LintOptions {
        conflict_marker_style: ConflictMarkerStyle::Jj,
        ..default_opts()
    };
    let diags = run_lint(src, &opts);
    let lnums: Vec<usize> = diags.iter().map(|d| d.lnum).collect();
    let end_lnums: Vec<usize> = diags.iter().map(|d| d.end_lnum).collect();
    let cols: Vec<usize> = diags.iter().map(|d| d.col).collect();
    let end_cols: Vec<usize> = diags.iter().map(|d| d.end_col).collect();
    let sources: Vec<&str> = diags.iter().map(|d| d.source.as_str()).collect();
    let source_lnums: Vec<usize> = diags.iter().map(|d| d.source_lnum).collect();

    assert_eq!(diags.len(), 5);
    assert_eq!(lnums, [31, 33, 36, 38, 39]);
    assert_eq!(end_lnums, [31, 33, 36, 38, 39]);
    assert_eq!(cols, [0, 0, 0, 0, 0]);
    assert_eq!(end_cols, [44, 9, 21, 9, 44]);
    assert_eq!(
        sources,
        [
            "<<<<<<< git/git-diff3/jj/jj-snapshot/jj-diff3\n",
            "%%%%%%% jj\n",
            "+++++++ jj/jj-snapshot\n",
            "\\\\\\\\\\\\\\ jj\n",
            ">>>>>>> git/git-diff3/jj/jj-snapshot/jj-diff3\n"
        ]
    );
    assert_eq!(source_lnums, [31, 33, 36, 38, 39]);
    for diag in &diags {
        assert_eq!(diag.code, "conflict-marker");
        assert!(diag.message.contains("Jujutsu conflict marker"));
    }
}

#[test]
fn detects_jj_conflict_markers_dynamic() {
    let src = &format!(
        "{}{}",
        CONFLICT_MARKER_SOURCE_SHORT, CONFLICT_MARKER_SOURCE_LONG
    );
    let opts = LintOptions {
        conflict_marker_style: ConflictMarkerStyle::Jj,
        ..default_opts()
    };
    let diags = run_lint(src, &opts);
    let lnums: Vec<usize> = diags.iter().map(|d| d.lnum).collect();
    let end_lnums: Vec<usize> = diags.iter().map(|d| d.end_lnum).collect();
    let cols: Vec<usize> = diags.iter().map(|d| d.col).collect();
    let end_cols: Vec<usize> = diags.iter().map(|d| d.end_col).collect();
    let sources: Vec<&str> = diags.iter().map(|d| d.source.as_str()).collect();
    let source_lnums: Vec<usize> = diags.iter().map(|d| d.source_lnum).collect();

    assert_eq!(diags.len(), 5);
    assert_eq!(lnums, [63, 65, 68, 70, 71]);
    assert_eq!(end_lnums, [63, 65, 68, 70, 71]);
    assert_eq!(cols, [0, 0, 0, 0, 0]);
    assert_eq!(end_cols, [34, 13, 25, 13, 34]);
    assert_eq!(
        sources,
        [
            "<<<<<<<<<<< jj/jj-snapshot/jj-diff3\n",
            "%%%%%%%%%%% jj\n",
            "+++++++++++ jj/jj-snapshot\n",
            "\\\\\\\\\\\\\\\\\\\\\\ jj\n",
            ">>>>>>>>>>> jj/jj-snapshot/jj-diff3\n"
        ]
    );
    assert_eq!(source_lnums, [63, 65, 68, 70, 71]);
    for diag in &diags {
        assert_eq!(diag.code, "conflict-marker");
        assert!(diag.message.contains("Jujutsu conflict marker"));
    }
}

#[test]
fn detects_jj_snapshot_conflict_markers_static() {
    let src = &CONFLICT_MARKER_SOURCE_SHORT;
    let opts = LintOptions {
        conflict_marker_style: ConflictMarkerStyle::JjSnapshot,
        ..default_opts()
    };
    let diags = run_lint(src, &opts);
    let lnums: Vec<usize> = diags.iter().map(|d| d.lnum).collect();
    let end_lnums: Vec<usize> = diags.iter().map(|d| d.end_lnum).collect();
    let cols: Vec<usize> = diags.iter().map(|d| d.col).collect();
    let end_cols: Vec<usize> = diags.iter().map(|d| d.end_col).collect();
    let sources: Vec<&str> = diags.iter().map(|d| d.source.as_str()).collect();
    let source_lnums: Vec<usize> = diags.iter().map(|d| d.source_lnum).collect();

    assert_eq!(diags.len(), 4);
    assert_eq!(lnums, [31, 36, 37, 39]);
    assert_eq!(end_lnums, [31, 36, 37, 39]);
    assert_eq!(cols, [0, 0, 0, 0]);
    assert_eq!(end_cols, [44, 21, 18, 44]);
    assert_eq!(
        sources,
        [
            "<<<<<<< git/git-diff3/jj/jj-snapshot/jj-diff3\n",
            "+++++++ jj/jj-snapshot\n",
            "------- jj-snapshot\n",
            ">>>>>>> git/git-diff3/jj/jj-snapshot/jj-diff3\n"
        ]
    );
    assert_eq!(source_lnums, [31, 36, 37, 39]);
    for diag in &diags {
        assert_eq!(diag.code, "conflict-marker");
        assert!(diag.message.contains("Jujutsu snapshot conflict marker"));
    }
}

#[test]
fn detects_jj_snapshot_conflict_markers_dynamic() {
    let src = &format!(
        "{}{}",
        CONFLICT_MARKER_SOURCE_SHORT, CONFLICT_MARKER_SOURCE_LONG
    );
    let opts = LintOptions {
        conflict_marker_style: ConflictMarkerStyle::JjSnapshot,
        ..default_opts()
    };
    let diags = run_lint(src, &opts);
    let lnums: Vec<usize> = diags.iter().map(|d| d.lnum).collect();
    let end_lnums: Vec<usize> = diags.iter().map(|d| d.end_lnum).collect();
    let cols: Vec<usize> = diags.iter().map(|d| d.col).collect();
    let end_cols: Vec<usize> = diags.iter().map(|d| d.end_col).collect();
    let sources: Vec<&str> = diags.iter().map(|d| d.source.as_str()).collect();
    let source_lnums: Vec<usize> = diags.iter().map(|d| d.source_lnum).collect();

    assert_eq!(diags.len(), 4);
    assert_eq!(lnums, [63, 68, 69, 71]);
    assert_eq!(end_lnums, [63, 68, 69, 71]);
    assert_eq!(cols, [0, 0, 0, 0]);
    assert_eq!(end_cols, [34, 25, 22, 34]);
    assert_eq!(
        sources,
        [
            "<<<<<<<<<<< jj/jj-snapshot/jj-diff3\n",
            "+++++++++++ jj/jj-snapshot\n",
            "----------- jj-snapshot\n",
            ">>>>>>>>>>> jj/jj-snapshot/jj-diff3\n"
        ]
    );
    assert_eq!(source_lnums, [63, 68, 69, 71]);
    for diag in &diags {
        assert_eq!(diag.code, "conflict-marker");
        assert!(diag.message.contains("Jujutsu snapshot conflict marker"));
    }
}

#[test]
fn detects_jj_diff3_conflict_markers_static() {
    let src = &CONFLICT_MARKER_SOURCE_SHORT;
    let opts = LintOptions {
        conflict_marker_style: ConflictMarkerStyle::JjDiff3,
        ..default_opts()
    };
    let diags = run_lint(src, &opts);
    let lnums: Vec<usize> = diags.iter().map(|d| d.lnum).collect();
    let end_lnums: Vec<usize> = diags.iter().map(|d| d.end_lnum).collect();
    let cols: Vec<usize> = diags.iter().map(|d| d.col).collect();
    let end_cols: Vec<usize> = diags.iter().map(|d| d.end_col).collect();
    let sources: Vec<&str> = diags.iter().map(|d| d.source.as_str()).collect();
    let source_lnums: Vec<usize> = diags.iter().map(|d| d.source_lnum).collect();

    assert_eq!(diags.len(), 4);
    assert_eq!(lnums, [31, 32, 35, 39]);
    assert_eq!(end_lnums, [31, 32, 35, 39]);
    assert_eq!(cols, [0, 0, 0, 0]);
    assert_eq!(end_cols, [44, 25, 6, 44]);
    assert_eq!(
        sources,
        [
            "<<<<<<< git/git-diff3/jj/jj-snapshot/jj-diff3\n",
            "||||||| git-diff3/jj-diff3\n",
            "=======\n",
            ">>>>>>> git/git-diff3/jj/jj-snapshot/jj-diff3\n"
        ]
    );
    assert_eq!(source_lnums, [31, 32, 35, 39]);
    for diag in &diags {
        assert_eq!(diag.code, "conflict-marker");
        assert!(diag.message.contains("Jujutsu diff3 conflict marker"));
    }
}

#[test]
fn detects_jj_diff3_conflict_markers_dynamic() {
    let src = &format!(
        "{}{}",
        CONFLICT_MARKER_SOURCE_SHORT, CONFLICT_MARKER_SOURCE_LONG
    );
    let opts = LintOptions {
        conflict_marker_style: ConflictMarkerStyle::JjDiff3,
        ..default_opts()
    };
    let diags = run_lint(src, &opts);
    let lnums: Vec<usize> = diags.iter().map(|d| d.lnum).collect();
    let end_lnums: Vec<usize> = diags.iter().map(|d| d.end_lnum).collect();
    let cols: Vec<usize> = diags.iter().map(|d| d.col).collect();
    let end_cols: Vec<usize> = diags.iter().map(|d| d.end_col).collect();
    let sources: Vec<&str> = diags.iter().map(|d| d.source.as_str()).collect();
    let source_lnums: Vec<usize> = diags.iter().map(|d| d.source_lnum).collect();

    assert_eq!(diags.len(), 4);
    assert_eq!(lnums, [63, 64, 67, 71]);
    assert_eq!(end_lnums, [63, 64, 67, 71]);
    assert_eq!(cols, [0, 0, 0, 0]);
    assert_eq!(end_cols, [34, 19, 10, 34]);
    assert_eq!(
        sources,
        [
            "<<<<<<<<<<< jj/jj-snapshot/jj-diff3\n",
            "||||||||||| jj-diff3\n",
            "===========\n",
            ">>>>>>>>>>> jj/jj-snapshot/jj-diff3\n"
        ]
    );
    assert_eq!(source_lnums, [63, 64, 67, 71]);
    for diag in &diags {
        assert_eq!(diag.code, "conflict-marker");
        assert!(diag.message.contains("Jujutsu diff3 conflict marker"));
    }
}

#[test]
fn detects_long_line() {
    let err_str = format!("let z = \"{:075}\";\n", 150);
    let src = format!("let x = 5;\nlet y = 10;\n{err_str}");
    let opts = LintOptions {
        line_length: 50,
        ..default_opts()
    };
    let diags = run_lint(&src, &opts);
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
fn skips_binary_file() {
    let src = "let x = 5;\0\nlet y = 10;\n";
    let diags = run_lint(src, &default_opts());
    assert_eq!(diags.len(), 0);
}

#[test]
fn processes_binary_in_text_mode() {
    let src = "let x = 5;\0 \nlet y = 10;\n";
    let opts = LintOptions {
        text_mode: true,
        ..default_opts()
    };
    let diags = run_lint(src, &opts);
    assert_eq!(diags.len(), 1);
    let diag = &diags[0];
    assert_eq!(diag.code, "trailing-space");
    assert_eq!(diag.source, "let x = 5;\0 \n");
}

#[test]
fn detects_consecutive_blank() {
    let err_str = [
        "\n\n\nlet x = 5;\n",
        "\n\n\n\nlet y = 10;\n",
        "let z = 15;\n\n\n\n\n\n",
    ];
    let src = err_str.join("");
    let opts = LintOptions {
        consecutive_blank: 2,
        ..default_opts()
    };
    let diags = run_lint(&src, &opts);
    let lnums: Vec<usize> = diags.iter().map(|d| d.lnum).collect();
    let end_lnums: Vec<usize> = diags.iter().map(|d| d.end_lnum).collect();
    let cols: Vec<usize> = diags.iter().map(|d| d.col).collect();
    let end_cols: Vec<usize> = diags.iter().map(|d| d.end_col).collect();
    let sources: Vec<&str> = diags.iter().map(|d| d.source.as_str()).collect();
    let source_lnums: Vec<usize> = diags.iter().map(|d| d.source_lnum).collect();
    let codes: Vec<&str> = diags.iter().map(|d| d.code.as_str()).collect();
    let pos: Vec<usize> = diags
        .iter()
        .map(|d| coord_to_pos(&d.source, d.source_lnum, d.lnum, d.col))
        .collect();
    let end_pos: Vec<usize> = diags
        .iter()
        .map(|d| coord_to_pos(&d.source, d.source_lnum, d.end_lnum, d.end_col))
        .collect();

    assert_eq!(diags.len(), 3);
    assert_eq!(lnums, [0, 4, 10]);
    assert_eq!(end_lnums, [2, 7, 14]);
    assert_eq!(cols, [0, 0, 0]);
    assert_eq!(end_cols, [0, 0, 0]);
    assert_eq!(
        sources,
        [
            err_str[0],
            format!("let x = 5;\n{}", err_str[1]).as_str(),
            err_str[2]
        ]
    );
    assert_eq!(source_lnums, [0, 3, 9]);
    assert_eq!(
        codes,
        [
            "consecutive-blank",
            "consecutive-blank",
            "consecutive-blank"
        ]
    );
    assert_eq!(pos, [0, 11, 12]);
    assert_eq!(end_pos, [2, 14, 16]);

    // Test helper positioning
    let helpers = diags[0].helpers.as_ref().unwrap();
    assert_eq!(helpers.len(), 1);
    assert_eq!(helpers[0].lnum, 3);
    assert_eq!(helpers[0].end_lnum, 3);
    assert_eq!(helpers[0].col, 0);
    assert_eq!(helpers[0].end_col, 9); // "let x = 5;".chars().count() - 1

    let helpers = diags[1].helpers.as_ref().unwrap();
    assert_eq!(helpers.len(), 2);
    assert_eq!(helpers[0].lnum, 3);
    assert_eq!(helpers[0].end_lnum, 3);
    assert_eq!(helpers[0].col, 0);
    assert_eq!(helpers[0].end_col, 9); // "let x = 5;".chars().count() - 1
    assert_eq!(helpers[1].lnum, 8);
    assert_eq!(helpers[1].end_lnum, 8);
    assert_eq!(helpers[1].col, 0);
    assert_eq!(helpers[1].end_col, 10); // "let y = 10;".chars().count() - 1

    let helpers = diags[2].helpers.as_ref().unwrap();
    assert_eq!(helpers.len(), 1);
    assert_eq!(helpers[0].lnum, 9);
    assert_eq!(helpers[0].end_lnum, 9);
    assert_eq!(helpers[0].col, 0);
    assert_eq!(helpers[0].end_col, 10); // "let z = 15;".chars().count() - 1
}

#[test]
fn detects_cjk_trailing_space() {
    let src = "let x = \"\t中文\";  \nlet y = 10;\n";
    let diags = run_lint(src, &default_opts());
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].lnum, 0);
    assert_eq!(diags[0].end_lnum, 0);
    assert_eq!(diags[0].col, 14);
    assert_eq!(diags[0].end_col, 15);
    assert_eq!(diags[0].source, "let x = \"\t中文\";  \n");
    assert_eq!(diags[0].source_lnum, 0);
    assert_eq!(diags[0].code, "trailing-space");
}

#[test]
fn detects_cjk_long_line() {
    let err_str = format!("let z = \"{}\";\n", "中文".repeat(30));
    let src = format!("let x = 5;\nlet y = 10;\n{err_str}");
    let diags = run_lint(src.as_str(), &default_opts());
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].lnum, 2);
    assert_eq!(diags[0].end_lnum, 2);
    assert_eq!(diags[0].col, 64);
    assert_eq!(diags[0].end_col, 70);
    assert_eq!(diags[0].source, err_str.as_str());
    assert_eq!(diags[0].source_lnum, 2);
    assert_eq!(diags[0].code, "long-line");
}

#[test]
fn detects_long_line_edge_case_one_over_limit() {
    let src = "0123456789X\n"; // 11 chars, limit is 10
    let opts = LintOptions {
        line_length: 10,
        ..default_opts()
    };
    let diags = run_lint(src, &opts);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].lnum, 0);
    assert_eq!(diags[0].end_lnum, 0);
    assert_eq!(diags[0].col, 10);
    assert_eq!(diags[0].end_col, 10);
    assert_eq!(diags[0].code, "long-line");
    assert_eq!(diags[0].message, "Too long line (11/10)");
}

#[test]
fn line_at_limit_not_flagged() {
    let src = "0123456789\n"; // exactly 10 chars
    let opts = LintOptions {
        line_length: 10,
        ..default_opts()
    };
    let diags = run_lint(src, &opts);
    assert_eq!(diags.len(), 0);
}

#[test]
fn detects_missing_final_newline() {
    let src = "let x = 5;"; // No trailing newline
    let diags = run_lint(src, &default_opts());
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code, "final-newline");
    assert_eq!(diags[0].lnum, 0);
    assert_eq!(diags[0].end_lnum, 0);
    assert_eq!(diags[0].col, 9);
    assert_eq!(diags[0].end_col, 9);
}

#[test]
fn detects_consecutive_blank_at_eof() {
    let src = "let x = 5;\n\n\n\n"; // 3 trailing newlines at EOF (exceeds limit of 2)
    let opts = LintOptions {
        consecutive_blank: 2,
        ..default_opts()
    };
    let diags = run_lint(src, &opts);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code, "consecutive-blank");
    // lnum is the start of the consecutive blank section (line 1, after the non-blank line 0)
    assert_eq!(diags[0].lnum, 1);
    assert_eq!(diags[0].end_lnum, 3);

    // Test helper positioning
    let helpers = diags[0].helpers.as_ref().unwrap();
    assert_eq!(helpers.len(), 1);
    assert_eq!(helpers[0].lnum, 0);
    assert_eq!(helpers[0].end_lnum, 0);
    assert_eq!(helpers[0].col, 0);
    assert_eq!(helpers[0].end_col, 9); // "let x = 5;".chars().count() - 1
}

#[test]
fn line_with_newline_not_flagged_for_final_newline() {
    let src = "let x = 5;\n"; // Has trailing newline
    let diags = run_lint(src, &default_opts());
    let final_newline_diags: Vec<_> = diags.iter().filter(|d| d.code == "final-newline").collect();
    assert_eq!(final_newline_diags.len(), 0);
}

#[test]
fn detects_consecutive_blank_with_tabs() {
    let src = "\n\n\n\tlet x = 5;\n\n\n\n\tlet y = 10;\n\tlet z = 15;\n\n\n\n\n";
    let opts = LintOptions {
        consecutive_blank: 2,
        ..default_opts()
    };
    let diags = run_lint(src, &opts);

    assert_eq!(diags.len(), 3);

    // Test helper positioning with tabs (character-based columns)
    let helpers1 = diags[1].helpers.as_ref().unwrap();
    assert_eq!(helpers1.len(), 2);

    // Previous line: "\tlet x = 5;" (11 chars including tab)
    assert_eq!(helpers1[0].lnum, 3);
    assert_eq!(helpers1[0].end_lnum, 3);
    assert_eq!(helpers1[0].col, 0);
    assert_eq!(helpers1[0].end_col, 10); // chars().count() - 1

    // Next line: "\tlet y = 10;" (12 chars including tab)
    assert_eq!(helpers1[1].lnum, 7);
    assert_eq!(helpers1[1].end_lnum, 7);
    assert_eq!(helpers1[1].col, 0);
    assert_eq!(helpers1[1].end_col, 11); // chars().count() - 1
}

#[test]
fn detects_consecutive_blank_at_eof_with_tabs() {
    let src = "\tlet x = 5;\n\n\n\n"; // 3 trailing newlines
    let opts = LintOptions {
        consecutive_blank: 2,
        ..default_opts()
    };
    let diags = run_lint(src, &opts);

    assert_eq!(diags.len(), 1);

    // Test helper positioning with tabs
    let helpers = diags[0].helpers.as_ref().unwrap();
    assert_eq!(helpers.len(), 1);
    assert_eq!(helpers[0].lnum, 0);
    assert_eq!(helpers[0].end_lnum, 0);
    assert_eq!(helpers[0].col, 0);
    assert_eq!(helpers[0].end_col, 10); // "\tlet x = 5;".chars().count() - 1
}

#[test]
fn detects_consecutive_blank_with_control_chars() {
    let src = "let x = \"\t\x01\";\n\n\n\nlet y = \"\x02\";\n\n\n\n\n";
    let opts = LintOptions {
        consecutive_blank: 2,
        ..default_opts()
    };
    let diags = run_lint(src, &opts);

    // Should have 2 diagnostics (consecutive blanks between and at end)
    assert_eq!(diags.len(), 2);

    // First diagnostic
    let helpers = diags[0].helpers.as_ref().unwrap();
    assert_eq!(helpers.len(), 2);

    // Previous line: "let x = \"\t\x01\";" (13 chars including control char)
    assert_eq!(helpers[0].lnum, 0);
    assert_eq!(helpers[0].end_lnum, 0);
    assert_eq!(helpers[0].col, 0);
    assert_eq!(helpers[0].end_col, 12); // chars().count() - 1

    // Next line: "let y = \"\x02\";" (12 chars including control char)
    assert_eq!(helpers[1].lnum, 4);
    assert_eq!(helpers[1].end_lnum, 4);
    assert_eq!(helpers[1].col, 0);
    assert_eq!(helpers[1].end_col, 11); // chars().count() - 1
    //
    // Second diagnostic
    let helpers = diags[1].helpers.as_ref().unwrap();
    assert_eq!(helpers.len(), 1);

    // Previous line: "let y = \"\x02\";" (12 chars including control char)
    assert_eq!(helpers[0].lnum, 4);
    assert_eq!(helpers[0].end_lnum, 4);
    assert_eq!(helpers[0].col, 0);
    assert_eq!(helpers[0].end_col, 11); // chars().count() - 1
}

#[test]
fn detects_consecutive_blank_file_only_has_blank_lines() {
    let src = "\n\n\n\n\n"; // 5 blank lines (exceeds limit of 1)
    let diags = run_lint(src, &default_opts());
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code, "consecutive-blank");
    assert_eq!(diags[0].lnum, 0);
    assert_eq!(diags[0].end_lnum, 4);
    assert_eq!(diags[0].message, "Too many consecutive blank lines (5/1)");

    // No helpers since there is no previous non-blank line
    assert!(diags[0].helpers.is_none());
}
