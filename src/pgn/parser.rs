use std::io::BufRead;
use crate::core::{Game, Tag, ChxError};
use crate::core::util::bom::strip_bom_str;

enum State {
    /// Before any content, or between games
    Between,
    /// Accumulating tag lines
    InTags,
    /// Accumulating movetext lines
    InMoves,
}

/// Streaming PGN parser. Reads from a `BufRead` and yields `Game` values.
///
/// PGN format: tags → blank line → movetext → blank line (separator).
/// This parser tracks state to handle the blank line between tags and movetext.
pub struct PgnParser<R: BufRead> {
    reader: R,
    buf: String,
    done: bool,
    first_line: bool,
    /// Buffered line from the next game (when we read past the boundary)
    pending_line: Option<String>,
}

impl<R: BufRead> PgnParser<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buf: String::new(),
            done: false,
            first_line: true,
            pending_line: None,
        }
    }

    fn read_line_raw(&mut self) -> Option<Result<String, ChxError>> {
        self.buf.clear();
        match self.reader.read_line(&mut self.buf) {
            Ok(0) => None,
            Ok(_) => {
                let mut line = self.buf.trim_end_matches('\n').trim_end_matches('\r').to_string();

                // Strip BOM on first line
                if self.first_line {
                    self.first_line = false;
                    line = strip_bom_str(&line).to_string();
                }

                Some(Ok(line))
            }
            Err(e) => Some(Err(ChxError::Io(e))),
        }
    }

    /// Read the next line, checking the pending buffer first.
    fn next_line(&mut self) -> Option<Result<String, ChxError>> {
        if let Some(line) = self.pending_line.take() {
            return Some(Ok(line));
        }
        self.read_line_raw()
    }
}

impl<R: BufRead> Iterator for PgnParser<R> {
    type Item = Result<Game, ChxError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let mut game = Game::new();
        let mut state = State::Between;

        loop {
            match self.next_line() {
                None => {
                    // EOF
                    self.done = true;
                    return match state {
                        State::InMoves | State::InTags => Some(Ok(game)),
                        State::Between => None,
                    };
                }
                Some(Err(e)) => return Some(Err(e)),
                Some(Ok(line)) => {
                    let trimmed = line.trim();

                    match &state {
                        State::Between => {
                            if trimmed.is_empty() {
                                continue;
                            }
                            if trimmed.starts_with('[') {
                                if let Some(tag) = parse_tag_line(trimmed) {
                                    game.tags.push(tag);
                                    state = State::InTags;
                                }
                            } else {
                                // Movetext without tags? Treat as movetext.
                                game.movetext.push(line);
                                state = State::InMoves;
                            }
                        }
                        State::InTags => {
                            if trimmed.is_empty() {
                                // Blank line after tags → transition to movetext
                                state = State::InMoves;
                                continue;
                            }
                            if trimmed.starts_with('[') {
                                if let Some(tag) = parse_tag_line(trimmed) {
                                    game.tags.push(tag);
                                }
                            } else {
                                // Non-tag line after tags = movetext
                                game.movetext.push(line);
                                state = State::InMoves;
                            }
                        }
                        State::InMoves => {
                            if trimmed.is_empty() {
                                // Blank line after movetext → game boundary
                                // But peek: if next non-blank line is a tag, this is a separator.
                                // If next non-blank line is movetext, it's continuation (multi-line movetext).
                                // For simplicity: blank line after movetext = end of game.
                                return Some(Ok(game));
                            }
                            if trimmed.starts_with('[') {
                                // Tag after movetext = next game starting
                                // Save this line for the next call
                                self.pending_line = Some(line);
                                return Some(Ok(game));
                            }
                            game.movetext.push(line);
                        }
                    }
                }
            }
        }
    }
}

/// Parse a `[Name "Value"]` tag line.
fn parse_tag_line(line: &str) -> Option<Tag> {
    let line = line.trim();
    if !line.starts_with('[') || !line.ends_with(']') {
        return None;
    }

    let inner = &line[1..line.len() - 1];
    let name_end = inner.find(|c: char| c.is_whitespace())?;
    let name = &inner[..name_end];

    let rest = inner[name_end..].trim();
    if !rest.starts_with('"') {
        return None;
    }

    // Find closing quote (handle escaped quotes)
    let value_start = 1;
    let mut escaped = false;
    let value_end = rest[value_start..].find(|c: char| {
        if escaped {
            escaped = false;
            return false;
        }
        if c == '\\' {
            escaped = true;
            return false;
        }
        c == '"'
    })?;

    let value = &rest[value_start..value_start + value_end];
    Some(Tag::new(name, value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    fn parse_games(input: &str) -> Vec<Game> {
        PgnParser::new(BufReader::new(input.as_bytes()))
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    }

    #[test]
    fn test_single_game() {
        let input = "[Event \"Test\"]\n[Site \"Nowhere\"]\n[Date \"2024.01.01\"]\n[Round \"1\"]\n[White \"Alice\"]\n[Black \"Bob\"]\n[Result \"1-0\"]\n\n1. e4 e5 2. Nf3 Nc6 1-0\n";
        let games = parse_games(input);
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].tag("Event"), Some("Test"));
        assert_eq!(games[0].tag("White"), Some("Alice"));
        assert_eq!(games[0].tag("Black"), Some("Bob"));
        assert_eq!(games[0].result(), crate::core::GameResult::WhiteWin);
        assert_eq!(games[0].movetext.len(), 1);
        assert_eq!(games[0].movetext[0], "1. e4 e5 2. Nf3 Nc6 1-0");
    }

    #[test]
    fn test_two_games() {
        let input = "[Event \"Game1\"]\n[White \"A\"]\n[Black \"B\"]\n[Result \"1-0\"]\n\n1. e4 e5 1-0\n\n[Event \"Game2\"]\n[White \"C\"]\n[Black \"D\"]\n[Result \"0-1\"]\n\n1. d4 d5 0-1\n";
        let games = parse_games(input);
        assert_eq!(games.len(), 2);
        assert_eq!(games[0].tag("Event"), Some("Game1"));
        assert_eq!(games[1].tag("Event"), Some("Game2"));
        assert_eq!(games[1].movetext[0], "1. d4 d5 0-1");
    }

    #[test]
    fn test_bom_handling() {
        let input = "\u{FEFF}[Event \"Test\"]\n[White \"A\"]\n[Black \"B\"]\n[Result \"*\"]\n\n1. e4 *\n";
        let games = parse_games(input);
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].tag("Event"), Some("Test"));
    }

    #[test]
    fn test_empty_input() {
        let games = parse_games("");
        assert_eq!(games.len(), 0);
    }

    #[test]
    fn test_multiline_movetext() {
        let input = "[Event \"Test\"]\n[White \"A\"]\n[Black \"B\"]\n[Result \"*\"]\n\n1. e4 e5\n2. Nf3 Nc6\n3. Bb5 *\n";
        let games = parse_games(input);
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].movetext.len(), 3);
    }

    #[test]
    fn test_no_trailing_newline() {
        let input = "[Event \"Test\"]\n[White \"A\"]\n[Black \"B\"]\n[Result \"*\"]\n\n1. e4 *";
        let games = parse_games(input);
        assert_eq!(games.len(), 1);
    }
}
