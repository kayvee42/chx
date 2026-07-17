use crate::core::{Game, ChxError};

/// Filter games by Elo range.
///
/// Keeps games where both White and Black Elo fall within [min, max].
/// Games missing either Elo tag are excluded.
pub fn min_elo(
    games: impl IntoIterator<Item = Result<Game, ChxError>>,
    min: u16,
    max: u16,
) -> Vec<Result<Game, ChxError>> {
    games
        .into_iter()
        .filter(|r| {
            match r {
                Ok(game) => {
                    match (game.white_elo(), game.black_elo()) {
                        (Some(w), Some(b)) => w.get() >= min && w.get() <= max && b.get() >= min && b.get() <= max,
                        _ => false,
                    }
                }
                Err(_) => true, // propagate errors
            }
        })
        .collect()
}

/// Filter games by minimum ply count.
///
/// Counts moves in the movetext (rough: counts move-number tokens like "1." or "1...").
/// A game with "1. e4 e5 2. Nf3" has 3 plies.
pub fn min_ply(
    games: impl IntoIterator<Item = Result<Game, ChxError>>,
    min_plies: usize,
) -> Vec<Result<Game, ChxError>> {
    games
        .into_iter()
        .filter(|r| {
            match r {
                Ok(game) => count_plies(game) >= min_plies,
                Err(_) => true,
            }
        })
        .collect()
}

/// Count plies in a game's movetext.
///
/// This is a rough count — counts tokens that look like SAN moves,
/// skipping move numbers, comments, and annotations.
fn count_plies(game: &Game) -> usize {
    let mut count = 0;
    for line in &game.movetext {
        for token in line.split_whitespace() {
            // Skip move numbers like "1." "1..." "23."
            if token.ends_with('.') || token.ends_with("...") {
                continue;
            }
            // Skip results
            if token == "1-0" || token == "0-1" || token == "1/2-1/2" || token == "*" {
                continue;
            }
            // Skip NAGs
            if token.starts_with('$') {
                continue;
            }
            // Skip comments (rough)
            if token.starts_with('{') || token.starts_with(';') {
                continue;
            }
            // Anything else is probably a move
            count += 1;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Tag;

    fn make_game(white_elo: Option<u16>, black_elo: Option<u16>, movetext: &str) -> Game {
        let mut tags = vec![
            Tag::new("Event", "Test"),
            Tag::new("White", "A"),
            Tag::new("Black", "B"),
            Tag::new("Result", "*"),
        ];
        if let Some(e) = white_elo {
            tags.push(Tag::new("WhiteElo", e.to_string()));
        }
        if let Some(e) = black_elo {
            tags.push(Tag::new("BlackElo", e.to_string()));
        }
        Game {
            tags,
            movetext: vec![movetext.to_string()],
        }
    }

    #[test]
    fn test_min_elo_both_present() {
        let games = vec![
            Ok(make_game(Some(2500), Some(2600), "1. e4 e5 *")),
            Ok(make_game(Some(2200), Some(2300), "1. d4 d5 *")),
            Ok(make_game(Some(2500), Some(2200), "1. e4 e5 *")),
        ];
        let result = min_elo(games, 2400, 2800);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_min_elo_missing_elo() {
        let games = vec![
            Ok(make_game(Some(2500), None, "1. e4 e5 *")),
            Ok(make_game(None, Some(2600), "1. d4 d5 *")),
        ];
        let result = min_elo(games, 2400, 2800);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_min_ply() {
        let games = vec![
            Ok(make_game(None, None, "1. e4 e5 2. Nf3 Nc6 3. Bb5 *")),
            Ok(make_game(None, None, "1. d4 *")),
        ];
        let result = min_ply(games, 3);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_count_plies() {
        let game = make_game(None, None, "1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 *");
        assert_eq!(count_plies(&game), 6);
    }
}
