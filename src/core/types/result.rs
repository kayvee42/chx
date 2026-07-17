use std::fmt;
use crate::core::ChxError;

/// The result of a chess game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameResult {
    WhiteWin,
    BlackWin,
    Draw,
    Unknown,
}

impl GameResult {
    pub fn parse(s: &str) -> Result<Self, ChxError> {
        match s.trim() {
            "1-0" => Ok(Self::WhiteWin),
            "0-1" => Ok(Self::BlackWin),
            "1/2-1/2" => Ok(Self::Draw),
            "*" => Ok(Self::Unknown),
            _ => Err(ChxError::InvalidResult(s.to_string())),
        }
    }
}

impl fmt::Display for GameResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WhiteWin => write!(f, "1-0"),
            Self::BlackWin => write!(f, "0-1"),
            Self::Draw => write!(f, "1/2-1/2"),
            Self::Unknown => write!(f, "*"),
        }
    }
}
