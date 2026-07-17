# chx

Chess data processing toolkit — a Rust port of Norman Pollock's [40H](https://nk-qy.info/40h/) suite.

92 command-line tools for processing PGN (game) and EPD (position) files, unified into a single binary with shared parsing and streaming I/O.

## Usage

```bash
chx clean-up -i games.pgn
chx min-elo -i games.pgn 2400 2800
chx min-ply -i games.pgn 40 > long_games.pgn

# Piping
chx clean-up -i raw.pgn | chx min-elo 2500 2800 > elite.pgn
```

## License

MIT
