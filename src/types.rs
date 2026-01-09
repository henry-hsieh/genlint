use clap::ValueEnum;
use serde::Serialize;
use serde_partial::SerializePartial;

#[derive(Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum Format {
    Json,
    Jsonl,
    Plain,
}

#[derive(Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum DisableCheck {
    MixIndent,
    TrailingSpace,
    ConflictMarker,
    LongLine,
    ConsecutiveBlank,
    FinalNewline,
}

#[derive(Debug)]
pub struct LintOptions {
    pub disables: Vec<DisableCheck>,
    pub line_length: usize,
    pub consecutive_blank: usize,
    pub max_errors: usize,
    pub max_warnings: usize,
    pub text_mode: bool,
}

#[derive(Debug)]
pub struct LintRunner {
    pub diagnostics: Vec<Diagnostic>,
    pub error_count: usize,
    pub warning_count: usize,
    pub has_printed_error_limit: bool,
    pub has_printed_warning_limit: bool,
}

#[derive(Debug, Serialize)]
pub struct Helper {
    pub message: String,
    pub lnum: usize,
    pub end_lnum: usize,
    pub col: usize,
    pub end_col: usize,
}

#[derive(Debug, Serialize, SerializePartial)]
pub struct Diagnostic {
    pub file: String,
    pub lnum: usize,
    pub end_lnum: usize,
    pub col: usize,
    pub end_col: usize,
    pub severity: String,
    pub source: String,
    pub source_lnum: usize,
    pub code: String,
    pub message: String,
    pub helpers: Option<Vec<Helper>>,
}
