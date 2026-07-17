use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChxError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("parse error at line {line}: {kind}")]
    Parse { line: usize, kind: ParseErrorKind },

    #[error("invalid Elo value: {0}")]
    InvalidElo(String),

    #[error("invalid game result: {0}")]
    InvalidResult(String),

    #[error("missing required tag '{0}' in game at line {1}")]
    MissingTag(String, usize),
}

#[derive(Error, Debug)]
pub enum ParseErrorKind {
    #[error("invalid tag format")]
    InvalidTag,
    #[error("unexpected end of file")]
    UnexpectedEof,
}
