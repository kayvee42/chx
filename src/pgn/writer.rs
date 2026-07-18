use std::io::{self, Write};
use crate::core::Game;

/// Write a Game as PGN text to the given writer.
///
/// Tags are written in order, followed by a blank line, then the movetext.
pub fn write_game(writer: &mut impl Write, game: &Game) -> io::Result<()> {
    for tag in &game.tags {
        writeln!(writer, "[{} \"{}\"]", tag.name, tag.value)?;
    }
    writeln!(writer)?;

    for (i, line) in game.movetext.iter().enumerate() {
        if i > 0 {
            writeln!(writer)?;
        }
        write!(writer, "{}", line)?;
    }
    // Ensure trailing newline after movetext
    if !game.movetext.is_empty() {
        writeln!(writer)?;
    }
    // PGN spec: blank line separates consecutive games
    writeln!(writer)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Tag;

    #[test]
    fn test_roundtrip() {
        let game = Game {
            tags: vec![
                Tag::new("Event", "Test"),
                Tag::new("Site", "Nowhere"),
                Tag::new("Date", "2024.01.01"),
                Tag::new("Round", "1"),
                Tag::new("White", "Alice"),
                Tag::new("Black", "Bob"),
                Tag::new("Result", "1-0"),
            ],
            movetext: vec!["1. e4 e5 2. Nf3 Nc6 1-0".to_string()],
        };

        let mut output = Vec::new();
        write_game(&mut output, &game).unwrap();
        let text = String::from_utf8(output).unwrap();

        assert!(text.contains("[Event \"Test\"]"));
        assert!(text.contains("1. e4 e5 2. Nf3 Nc6 1-0"));
    }
}
