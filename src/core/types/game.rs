use super::{Tag, GameResult, Elo};

/// A parsed PGN game: tags + movetext lines.
#[derive(Debug, Clone)]
pub struct Game {
    pub tags: Vec<Tag>,
    pub movetext: Vec<String>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            tags: Vec::new(),
            movetext: Vec::new(),
        }
    }

    /// Find a tag value by name.
    pub fn tag(&self, name: &str) -> Option<&str> {
        self.tags
            .iter()
            .find(|t| t.name == name)
            .map(|t| t.value.as_str())
    }

    /// Get the game result.
    pub fn result(&self) -> GameResult {
        self.tag("Result")
            .and_then(|s| GameResult::parse(s).ok())
            .unwrap_or(GameResult::Unknown)
    }

    /// Get White's Elo, if present.
    pub fn white_elo(&self) -> Option<Elo> {
        self.tag("WhiteElo")
            .and_then(|s| Elo::parse(s).ok())
            .flatten()
    }

    /// Get Black's Elo, if present.
    pub fn black_elo(&self) -> Option<Elo> {
        self.tag("BlackElo")
            .and_then(|s| Elo::parse(s).ok())
            .flatten()
    }
}
