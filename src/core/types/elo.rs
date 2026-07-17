use std::fmt;
use crate::core::ChxError;

/// An Elo rating value (1–5000).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Elo(pub u16);

impl Elo {
    pub fn parse(s: &str) -> Result<Option<Self>, ChxError> {
        let s = s.trim().trim_matches('"');
        if s.is_empty() || s == "?" {
            return Ok(None);
        }
        let val: u16 = s
            .parse()
            .map_err(|_| ChxError::InvalidElo(s.to_string()))?;
        if val == 0 {
            Ok(None)
        } else {
            Ok(Some(Elo(val)))
        }
    }

    pub fn get(self) -> u16 {
        self.0
    }
}

impl fmt::Display for Elo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
