use clap::ValueEnum;
use serde::Serialize;
use serde_partial::SerializePartial;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DiagnosticType {
    Error,
    Warning,
    Information,
}

impl DiagnosticType {
    fn from_severity(severity: &str) -> Option<Self> {
        match severity {
            "error" => Some(Self::Error),
            "warning" => Some(Self::Warning),
            "information" => Some(Self::Information),
            _ => None,
        }
    }

    fn max_limit(&self, opts: &LintOptions) -> usize {
        match self {
            Self::Error => opts.max_errors,
            Self::Warning => opts.max_warnings,
            Self::Information => opts.max_info,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct DiagnosticStats {
    count: usize,
    has_printed_limit: bool,
    limit_reached: bool,
}

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
    pub max_info: usize,
    pub text_mode: bool,
}

#[derive(Debug, Default)]
pub struct LintRunner {
    pub diagnostics: Vec<Diagnostic>,
    limited_stats: HashMap<DiagnosticType, DiagnosticStats>,
    should_terminate: bool,
    processing_blocked: HashSet<DiagnosticType>,
}

impl LintRunner {
    pub fn new() -> Self {
        let limited_stats = [
            DiagnosticType::Error,
            DiagnosticType::Warning,
            DiagnosticType::Information,
        ]
        .into_iter()
        .map(|diag_type| (diag_type, DiagnosticStats::default()))
        .collect();

        Self {
            diagnostics: Vec::new(),
            limited_stats,
            should_terminate: false,
            processing_blocked: HashSet::new(),
        }
    }

    pub fn diagnostic_counts(&self) -> (usize, usize, usize) {
        let error_count = self.limited_stats[&DiagnosticType::Error].count;
        let warning_count = self.limited_stats[&DiagnosticType::Warning].count;
        let info_count = self.limited_stats[&DiagnosticType::Information].count;
        (error_count, warning_count, info_count)
    }

    pub fn limit_reached(&self, diag_type: &DiagnosticType) -> bool {
        self.limited_stats[diag_type].limit_reached
    }

    pub fn can_add_issue(&self, severity: &str) -> bool {
        if let Some(diag_type) = DiagnosticType::from_severity(severity) {
            !self.processing_blocked.contains(&diag_type)
        } else {
            true
        }
    }

    pub fn add_diagnostic(&mut self, opts: &LintOptions, diag: Diagnostic) -> bool {
        if self.should_terminate {
            return false;
        }

        if let Some(diag_type) = DiagnosticType::from_severity(diag.severity.as_str()) {
            if self.processing_blocked.contains(&diag_type) {
                return true; // Skip but continue processing other types
            }

            let max_limit = diag_type.max_limit(opts);
            let continue_processing =
                self.add_limited_diagnostic_internal(diag, max_limit, diag_type);

            if !continue_processing {
                match diag_type {
                    DiagnosticType::Error => {
                        self.should_terminate = true;
                        return false; // Stop all processing
                    }
                    DiagnosticType::Warning | DiagnosticType::Information => {
                        self.processing_blocked.insert(diag_type);
                        // Continue with other types
                    }
                }
            }
        } else {
            self.diagnostics.push(diag);
        }

        true
    }

    fn add_limited_diagnostic_internal(
        &mut self,
        diag: Diagnostic,
        max_limit: usize,
        diag_type: DiagnosticType,
    ) -> bool {
        let stats = self.limited_stats.get_mut(&diag_type).unwrap();

        if max_limit > 0 && stats.count >= max_limit {
            stats.limit_reached = true;
            return false; // Limit reached
        }

        self.diagnostics.push(diag);
        stats.count += 1;

        if max_limit > 0 && stats.count == max_limit && !stats.has_printed_limit {
            let message = match diag_type {
                DiagnosticType::Error => format!(
                    "found {} errors, please fix the errors or increase the --max-errors limit",
                    max_limit
                ),
                DiagnosticType::Warning => format!(
                    "found {} warnings, please fix the warnings or increase the --max-warnings limit",
                    max_limit
                ),
                DiagnosticType::Information => format!(
                    "found {} information, please review or increase the --max-info limit",
                    max_limit
                ),
            };
            eprintln!("{}", message);
            stats.has_printed_limit = true;
        }

        true
    }
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
