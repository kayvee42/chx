# chxss

Norm Pollock's 40H chess tools ([nk-qy.info/40h](https://nk-qy.info/40h/)) — 92 command-line utilities for processing PGN and EPD files, built over many years for the chess community.

This Rust port brings them to Windows, macOS, and Linux as a single binary. It isn't officially endorsed by Norm — I didn't ask. I'm keeping it compatible, but small differences may exist. Treat it as a community port.

## Current status

**5 of 92 tools ported** (Batch 1: Clean & Validate). See [docs/](docs/) for details.

## Usage

```bash
# Filter by Elo range
chxss min-elo -i games.pgn 2500 2700

# Lower bound only (both Elos >= 2500)
chxss min-elo -i games.pgn 2500

# Two ranges (one Elo in each range)
chxss min-elo -i games.pgn 2500 2700 2200 2400

# Remove games missing Elo tags
chxss elo-check -i games.pgn -o clean.pgn

# Reorder tags to standard order
chxss tag-order -i games.pgn -o ordered.pgn

# Replace tag values with defaults
chxss tag-null -i games.pgn Round

# Remove all instances of a tag
chxss tag-remove -i games.pgn ECO

# Piping (stdin/stdout by default)
cat games.pgn | chxss elo-check | chxss min-elo 2500 > elite.pgn
```

All tools accept `-i`/`-o` flags. Omit for stdin/stdout.

## Tools

### Cleaning & Repair

| Tool | What it does |
|------|-------------|
| `elo-check` | Remove games missing WhiteElo or BlackElo |
| `min-elo` | Filter by Elo range(s): 1, 2, or 4 values |
| `tag-order` | Reorder tags to Seven Tag Roster, PlyCount last |
| `tag-null` | Replace tag values with default/null values |
| `tag-remove` | Remove all instances of a tag type |
| `clean-up` | Validate games (coming soon) |
| `tag-fix` | Repair malformed tags (coming soon) |

### Tag dependency grid

Some tools read tags that other tools write:

| Tag | Written by | Read by |
|-----|-----------|---------|
| `[PlyCount]` | `ply-count` | `min-ply` |
| `[WhiteElo]` `[BlackElo]` | source, `elo-extend`, `elo-insert` | `min-elo`, `elo-check`, `elo-gap`, `elo-list` |
| `[White]` `[Black]` | source, `name-change` | `player-extract`, `name-extract`, `name-list`, etc. |

Tools where both Elo tags must exist are marked in their help text.

## License

MIT
