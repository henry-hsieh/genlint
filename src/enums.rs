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
