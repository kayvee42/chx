# Cleaning & Repair

Run these first on any PGN file — they validate, repair, and prepare data.

## elo-check

Remove games missing `[WhiteElo]` or `[BlackElo]` tags.

Original tool: `eloCheck` (output: `outW.pgn` + `excludeW.pgn`)

```
chxss elo-check -i input.pgn -o clean.pgn
```

A game passes if both Elo tags are present, numeric, and > 0. Values of `?` or `0` count as missing.

## min-elo

Filter games where Elo ratings fall within specified range(s).

Original tool: `minElo` (output: `outM.pgn` + `excludeM.pgn`)

Three modes based on argument count:

```bash
# 1 value: both Elos >= N
chxss min-elo -i input.pgn 2500

# 2 values: both Elos in [min, max]
chxss min-elo -i input.pgn 2500 2700

# 4 values: one Elo in [min1, max1], other in [min2, max2]
chxss min-elo -i input.pgn 2500 2700 2200 2400
```

Games missing either Elo are excluded.

## tag-order

Reorder tags to the standard Seven Tag Roster order, with `[PlyCount]` last.

Original tool: `tagOrder` (output: `out3.pgn`)

```
chxss tag-order -i input.pgn -o ordered.pgn
```

Order: Event, Site, Date, Round, White, Black, Result, WhiteElo, BlackElo, ECO, SetUp, FEN, (other tags), PlyCount.

WhiteElo and BlackElo with empty values are dropped.

## tag-null

Replace all values of a tag type with its standard default value.

Original tool: `tagNull` (output: `outH.pgn`)

```
chxss tag-null -i input.pgn Round
```

Default values by tag type:

| Tag | Default |
|-----|---------|
| `*Date` (Date, EventDate, ...) | `????.??.??` |
| `Time` | `??:??:??` |
| `Result` | `*` (also updates result in movetext) |
| `SetUp` | `0` |
| Everything else | `?` |

Refuses to modify `[FEN]` tags. If target is `Event` and no Event tag exists, one is added.

## tag-remove

Remove all instances of a tag type.

Original tool: `tagRemove` (output: `out2.pgn`)

```
chxss tag-remove -i input.pgn ECO
```

Refuses to remove `[Event]` or `[FEN]` tags.
