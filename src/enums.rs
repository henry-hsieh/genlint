use clap::ValueEnum;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum ConflictMarkerStyle {
    Git,
    GitDiff3,
    Jj,
    JjDiff3,
    JjSnapshot,
}
