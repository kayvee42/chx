use crate::core::{Game, ChxssError, Tag};

/// Filter games by Elo range(s). Matches original `minElo` behavior.
///
/// Three modes (based on number of values provided):
/// - 1 value: both Elos must be ≥ `vals[0]` (max defaults to 5000)
/// - 2 values: both Elos must be in `[vals[0], vals[1]]`
/// - 4 values: one Elo in `[vals[0], vals[1]]`, the other in `[vals[2], vals[3]]`
///   (order doesn't matter — either pairing works)
///
/// Games missing either Elo tag are excluded.
/// Returns an error if vals.len() is not 1, 2, or 4.
pub fn min_elo(
    games: impl IntoIterator<Item = Result<Game, ChxssError>>,
    vals: &[u16],
) -> Result<Vec<Result<Game, ChxssError>>, ChxssError> {
    let (min1, max1, min2, max2) = match vals.len() {
        1 => (vals[0], 5000, 0, 5000),
        2 => (vals[0], vals[1], 0, 5000),
        4 => (vals[0], vals[1], vals[2], vals[3]),
        n => return Err(ChxssError::InvalidArgument(format!(
            "min-elo requires 1, 2, or 4 Elo values, got {n}"
        ))),
    };
    let two_ranges = vals.len() == 4;

    let result = games
        .into_iter()
        .filter(|r| match r {
            Ok(game) => {
                let w = game.white_elo().map(|e| e.get());
                let b = game.black_elo().map(|e| e.get());
                match (w, b) {
                    (Some(w), Some(b)) => {
                        if two_ranges {
                            // White in range1 AND Black in range2, OR White in range2 AND Black in range1
                            (w >= min1 && w <= max1 && b >= min2 && b <= max2)
                                || (w >= min2 && w <= max2 && b >= min1 && b <= max1)
                        } else {
                            // Both in the same range
                            w >= min1 && w <= max1 && b >= min1 && b <= max1
                        }
                    }
                    _ => false,
                }
            }
            Err(_) => true,
        })
        .collect();
    Ok(result)
}

/// Remove games missing WhiteElo or BlackElo tags.
///
/// Keeps only games where both Elo tags are present and valid (> 0, not "?").
/// The original `eloCheck` outputs two files: outW.pgn (passed) and excludeW.pgn (removed).
pub fn elo_check(
    games: impl IntoIterator<Item = Result<Game, ChxssError>>,
) -> Vec<Result<Game, ChxssError>> {
    games
        .into_iter()
        .filter(|r| match r {
            Ok(game) => game.white_elo().is_some() && game.black_elo().is_some(),
            Err(_) => true,
        })
        .collect()
}

/// Seven Tag Roster order + extended tags.
const STR_ORDER: &[&str] = &[
    "Event", "Site", "Date", "Round", "White", "Black", "Result",
    "WhiteElo", "BlackElo", "ECO", "SetUp", "FEN",
];

/// Reorder tags to standard order: STR first, then other tags in original order,
/// with PlyCount always last. Matches the original `tagOrder` behavior.
///
/// WhiteElo and BlackElo tags with empty values are dropped (matching original).
pub fn tag_order(game: &mut Game) {
    let mut ordered: Vec<Tag> = Vec::with_capacity(game.tags.len());

    // STR tags in order (only if present and value is non-empty)
    for &name in STR_ORDER {
        if let Some(idx) = game.tags.iter().position(|t| t.name == name) {
            let tag = &game.tags[idx];
            // Drop WhiteElo/BlackElo with empty values (matches original behavior)
            if (name == "WhiteElo" || name == "BlackElo") && tag.value.is_empty() {
                continue;
            }
            ordered.push(game.tags.remove(idx));
        }
    }

    // Remaining tags: keep original order, but pull PlyCount to the end
    let mut ply_count: Option<Tag> = None;
    // PlyCount is unique per PGN spec — use if-let, not while-let
    if let Some(idx) = game.tags.iter().position(|t| t.name == "PlyCount") {
        ply_count = Some(game.tags.remove(idx));
    }

    ordered.append(&mut game.tags);

    if let Some(tag) = ply_count {
        ordered.push(tag);
    }

    game.tags = ordered;
}

/// Replace all values of a tag type with its null/default value.
///
/// Null values by tag type:
/// - `*Date` tags → `????.??.??`
/// - `Time` → `??:??:??`
/// - `Result` → `*` (also updates result in movetext)
/// - `SetUp` → `0`
/// - everything else → `?`
///
/// Returns the number of replacements made.
/// Returns an error if tag_name is "FEN" (refused, matching original behavior).
pub fn tag_null(game: &mut Game, tag_name: &str) -> Result<usize, ChxssError> {
    if tag_name == "FEN" {
        return Err(ChxssError::InvalidArgument("FEN values cannot be changed".to_string()));
    }

    let null_value = null_value_for(tag_name);
    let mut count = 0;

    for tag in &mut game.tags {
        if tag.name == tag_name {
            tag.value = null_value.to_string();
            count += 1;
        }
    }

    // Special: if target is Event and game has no Event tag, add one
    if tag_name == "Event" && !game.tags.iter().any(|t| t.name == "Event") {
        game.tags.insert(0, Tag::new("Event", "?"));
        count += 1;
    }

    // Special: if target is Result, also update result in movetext.
    // Only replace the last result token outside brace comments.
    if tag_name == "Result" {
        if let Some(last) = game.movetext.last_mut() {
            // Find the last result token outside comments
            let mut brace_depth = 0usize;
            let mut last_result_idx: Option<usize> = None;
            let tokens: Vec<&str> = last.split_whitespace().collect();
            for (i, token) in tokens.iter().enumerate() {
                brace_depth += token.matches('{').count();
                if brace_depth > 0 {
                    brace_depth = brace_depth.saturating_sub(token.matches('}').count());
                    continue;
                }
                if *token == "1-0" || *token == "0-1" || *token == "1/2-1/2" {
                    last_result_idx = Some(i);
                }
            }
            if let Some(idx) = last_result_idx {
                let new_line = tokens
                    .iter()
                    .enumerate()
                    .map(|(i, t)| if i == idx { "*" } else { *t })
                    .collect::<Vec<_>>()
                    .join(" ");
                *last = new_line;
                count += 1;
            }
        }
    }

    Ok(count)
}

/// Map a tag name to its null/default value.
/// Remove all instances of a tag type. Refuses to remove Event or FEN.
///
/// Returns the number of tags removed.
pub fn tag_remove(game: &mut Game, tag_name: &str) -> usize {
    if tag_name == "Event" || tag_name == "FEN" {
        return 0;
    }
    let before = game.tags.len();
    game.tags.retain(|t| t.name != tag_name);
    before - game.tags.len()
}

fn null_value_for(tag_name: &str) -> &str {
    if tag_name.ends_with("Date") {
        return "????.??.??";
    }
    match tag_name {
        "Time" => "??:??:??",
        "Result" => "*",
        "SetUp" => "0",
        _ => "?",
    }
}

/// Filter games by minimum ply count.
///
/// Counts moves in the movetext (rough: counts move-number tokens like "1." or "1...").
/// A game with "1. e4 e5 2. Nf3" has 3 plies.
pub fn min_ply(
    games: impl IntoIterator<Item = Result<Game, ChxssError>>,
    min_plies: usize,
) -> Vec<Result<Game, ChxssError>> {
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
/// Counts SAN move tokens, skipping move numbers, results, NAGs,
/// comments (brace-delimited and semicolon-to-EOL), and variations (parenthesized).
fn count_plies(game: &Game) -> usize {
    let mut count = 0;
    let mut brace_depth = 0usize;
    let mut paren_depth = 0usize;
    for line in &game.movetext {
        // Semicolon comments: skip from `;` to end of line
        let line_to_parse = if let Some(pos) = line.find(';') {
            &line[..pos]
        } else {
            line.as_str()
        };
        for token in line_to_parse.split_whitespace() {
            // Track brace comment depth
            brace_depth += token.matches('{').count();
            if brace_depth > 0 {
                brace_depth = brace_depth.saturating_sub(token.matches('}').count());
                continue;
            }
            // Track variation depth
            paren_depth += token.matches('(').count();
            if paren_depth > 0 {
                paren_depth = paren_depth.saturating_sub(token.matches(')').count());
                continue;
            }
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
    fn test_min_elo_single_range() {
        let games = vec![
            Ok(make_game(Some(2500), Some(2600), "1. e4 e5 *")),
            Ok(make_game(Some(2200), Some(2300), "1. d4 d5 *")),
            Ok(make_game(Some(2500), Some(2200), "1. e4 e5 *")),
        ];
        let result = min_elo(games, &[2400, 2800]).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_min_elo_lower_bound() {
        let games = vec![
            Ok(make_game(Some(2500), Some(2600), "1. e4 e5 *")),
            Ok(make_game(Some(2200), Some(2300), "1. d4 d5 *")),
        ];
        let result = min_elo(games, &[2400]).unwrap();
        assert_eq!(result.len(), 1); // only 2500/2600 game
    }

    #[test]
    fn test_min_elo_two_ranges() {
        let games = vec![
            // White 2500 in [2500,2700], Black 2200 in [2200,2400] → match
            Ok(make_game(Some(2500), Some(2200), "1. e4 e5 *")),
            // White 2500 in [2500,2700], Black 2600 in [2500,2700] → both in range1, not range2 → no match
            Ok(make_game(Some(2500), Some(2600), "1. d4 d5 *")),
        ];
        let result = min_elo(games, &[2500, 2700, 2200, 2400]).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_min_elo_rejects_three_values() {
        let games: Vec<Result<Game, ChxssError>> = vec![];
        assert!(min_elo(games, &[2000, 2400, 2600]).is_err());
    }

    #[test]
    fn test_min_elo_missing_elo() {
        let games = vec![
            Ok(make_game(Some(2500), None, "1. e4 e5 *")),
            Ok(make_game(None, Some(2600), "1. d4 d5 *")),
        ];
        let result = min_elo(games, &[2400, 2800]).unwrap();
        assert_eq!(result.len(), 0);
    }

    // --- elo_check tests ---

    #[test]
    fn test_elo_check_both_present() {
        let games = vec![
            Ok(make_game(Some(2500), Some(2600), "1. e4 e5 *")),
            Ok(make_game(Some(2200), Some(2300), "1. d4 d5 *")),
        ];
        let result = elo_check(games);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_elo_check_one_missing() {
        let games = vec![
            Ok(make_game(Some(2500), None, "1. e4 e5 *")),
            Ok(make_game(None, Some(2600), "1. d4 d5 *")),
        ];
        let result = elo_check(games);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_elo_check_both_missing() {
        let games = vec![
            Ok(make_game(None, None, "1. e4 e5 *")),
        ];
        let result = elo_check(games);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_elo_check_mixed() {
        let games = vec![
            Ok(make_game(Some(2500), Some(2600), "1. e4 e5 *")),
            Ok(make_game(Some(2200), None, "1. d4 d5 *")),
            Ok(make_game(Some(2400), Some(2300), "1. Nf3 *")),
        ];
        let result = elo_check(games);
        assert_eq!(result.len(), 2);
    }

    // --- tag_null tests ---

    #[test]
    fn test_tag_null_default() {
        let mut game = Game {
            tags: vec![
                Tag::new("Event", "Test"),
                Tag::new("Round", "1"),
            ],
            movetext: vec!["1. e4 *".to_string()],
        };
        tag_null(&mut game, "Round").unwrap();
        assert_eq!(game.tag("Round"), Some("?"));
    }

    #[test]
    fn test_tag_null_date() {
        let mut game = Game {
            tags: vec![Tag::new("Date", "2024.01.01")],
            movetext: vec!["1. e4 *".to_string()],
        };
        tag_null(&mut game, "Date").unwrap();
        assert_eq!(game.tag("Date"), Some("????.??.??"));
    }

    #[test]
    fn test_tag_null_event_date() {
        let mut game = Game {
            tags: vec![Tag::new("EventDate", "2024.01.01")],
            movetext: vec!["1. e4 *".to_string()],
        };
        tag_null(&mut game, "EventDate").unwrap();
        assert_eq!(game.tag("EventDate"), Some("????.??.??"));
    }

    #[test]
    fn test_tag_null_result_updates_movetext() {
        let mut game = make_game(Some(2500), Some(2600), "1. e4 1-0");
        // Add explicit Result tag
        game.tags.push(Tag::new("Result", "1-0"));
        tag_null(&mut game, "Result").unwrap();
        assert_eq!(game.tag("Result"), Some("*"));
        assert!(game.movetext.last().unwrap().contains('*'));
        assert!(!game.movetext.last().unwrap().contains("1-0"));
    }

    #[test]
    fn test_tag_null_result_preserves_comment() {
        let mut game = make_game(Some(2500), Some(2600), "1. e4 { ended in a convincing 1-0 } 1-0");
        game.tags.push(Tag::new("Result", "1-0"));
        tag_null(&mut game, "Result").unwrap();
        let movetext = game.movetext.last().unwrap();
        // The result in the comment should be preserved
        assert!(movetext.contains("convincing 1-0"));
        // The actual game result at the end should be replaced
        assert!(movetext.ends_with('*'));
    }

    #[test]
    fn test_tag_null_fen_refused() {
        let mut game = Game {
            tags: vec![Tag::new("FEN", "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")],
            movetext: vec!["1. e4 *".to_string()],
        };
        assert!(tag_null(&mut game, "FEN").is_err());
    }

    #[test]
    fn test_tag_null_event_adds_if_missing() {
        let mut game = Game {
            tags: vec![Tag::new("White", "Player")],
            movetext: vec!["1. e4 *".to_string()],
        };
        tag_null(&mut game, "Event").unwrap();
        assert_eq!(game.tag("Event"), Some("?"));
    }

    #[test]
    fn test_tag_null_setup() {
        let mut game = Game {
            tags: vec![Tag::new("SetUp", "1")],
            movetext: vec!["1. e4 *".to_string()],
        };
        tag_null(&mut game, "SetUp").unwrap();
        assert_eq!(game.tag("SetUp"), Some("0"));
    }

    // --- tag_remove tests ---

    #[test]
    fn test_tag_remove() {
        let mut game = Game {
            tags: vec![
                Tag::new("Event", "Test"),
                Tag::new("ECO", "B90"),
                Tag::new("White", "A"),
                Tag::new("ECO", "B90"),
            ],
            movetext: vec!["1. e4 *".to_string()],
        };
        let count = tag_remove(&mut game, "ECO");
        assert_eq!(count, 2);
        assert!(game.tags.iter().all(|t| t.name != "ECO"));
        assert_eq!(game.tags.len(), 2);
    }

    #[test]
    fn test_tag_remove_refuses_event() {
        let mut game = Game {
            tags: vec![Tag::new("Event", "Test")],
            movetext: vec!["1. e4 *".to_string()],
        };
        let count = tag_remove(&mut game, "Event");
        assert_eq!(count, 0);
        assert_eq!(game.tags.len(), 1);
    }

    // --- tag_order tests ---

    #[test]
    fn test_tag_order_reorders_to_str() {
        let mut game = Game {
            tags: vec![
                Tag::new("White", "Player"),
                Tag::new("BlackElo", "2500"),
                Tag::new("Event", "Test"),
                Tag::new("Result", "*"),
                Tag::new("PlyCount", "42"),
                Tag::new("Black", "Opponent"),
                Tag::new("ECO", "B90"),
                Tag::new("CustomTag", "custom_value"),
            ],
            movetext: vec!["1. e4 *".to_string()],
        };
        tag_order(&mut game);
        let names: Vec<&str> = game.tags.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names, vec![
            "Event", "White", "Black", "Result", "BlackElo", "ECO",
            "CustomTag",
            "PlyCount",
        ]);
    }

    #[test]
    fn test_tag_order_plycount_last() {
        let mut game = Game {
            tags: vec![
                Tag::new("Event", "Test"),
                Tag::new("PlyCount", "40"),
                Tag::new("Site", "Nowhere"),
                Tag::new("Result", "*"),
                Tag::new("ECO", "C54"),
            ],
            movetext: vec!["1. e4 *".to_string()],
        };
        tag_order(&mut game);
        let last = game.tags.last().unwrap();
        assert_eq!(last.name, "PlyCount");
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

    #[test]
    fn test_count_plies_with_comments() {
        // Multi-word brace comment should not count as plies
        let game = make_game(None, None, "1. e4 { good move } e5 2. Nf3 *");
        assert_eq!(count_plies(&game), 3);
    }

    #[test]
    fn test_count_plies_with_variations() {
        // Variation moves should not count as mainline plies
        let game = make_game(None, None, "1. e4 (1. d4 d5) e5 2. Nf3 *");
        assert_eq!(count_plies(&game), 3);
    }

    #[test]
    fn test_count_plies_with_semicolon_comment() {
        // Semicolon comment to end of line — everything after ; on that line is ignored
        let mut game = make_game(None, None, "");
        game.movetext = vec![
            "1. e4 e5 ; this is a comment".to_string(),
            "2. Nf3 *".to_string(),
        ];
        assert_eq!(count_plies(&game), 3);
    }
}
