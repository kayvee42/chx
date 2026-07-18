use std::path::PathBuf;
use clap::Parser;
use anyhow::Result;

use chxss::cli::commands::Command;
use chxss::cli::io;
use chxss::pgn::PgnParser;
use chxss::pgn::write_game;

#[derive(Parser)]
#[command(name = "chxss", version, about = "Chess data processing toolkit")]
struct Cli {
    /// Input file (reads from stdin if omitted)
    #[arg(short = 'i', long = "input", global = true)]
    input: Option<PathBuf>,

    /// Output file (writes to stdout if omitted)
    #[arg(short = 'o', long = "output", global = true)]
    output: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let reader = io::open_input(cli.input.as_deref())?;
    let buf_reader = io::buf_reader(reader);

    let mut writer = io::open_output(cli.output.as_deref())?;

    let mut errors = 0usize;
    match cli.command {
        Command::MinElo { vals } => {
            let games = PgnParser::new(buf_reader).collect::<Vec<_>>();
            let filtered = chxss::tools::pgn::filter::min_elo(games, &vals)?;
            for game in filtered {
                match game {
                    Ok(game) => { write_game(&mut writer, &game)?; }
                    Err(e) => { eprintln!("warning: {e}"); errors += 1; }
                }
            }
        }
        Command::EloCheck => {
            let games = PgnParser::new(buf_reader).collect::<Vec<_>>();
            let filtered = chxss::tools::pgn::filter::elo_check(games);
            for game in filtered {
                match game {
                    Ok(game) => { write_game(&mut writer, &game)?; }
                    Err(e) => { eprintln!("warning: {e}"); errors += 1; }
                }
            }
        }
        Command::TagOrder => {
            let games = PgnParser::new(buf_reader).collect::<Vec<_>>();
            for game in games {
                match game {
                    Ok(mut game) => {
                        chxss::tools::pgn::filter::tag_order(&mut game);
                        write_game(&mut writer, &game)?;
                    }
                    Err(e) => { eprintln!("warning: {e}"); errors += 1; }
                }
            }
        }
        Command::TagNull { tag } => {
            if tag == "FEN" {
                anyhow::bail!("\"FEN\" values cannot be changed.");
            }
            let games = PgnParser::new(buf_reader).collect::<Vec<_>>();
            let mut changes = 0usize;
            for game in games {
                match game {
                    Ok(mut game) => {
                        changes += chxss::tools::pgn::filter::tag_null(&mut game, &tag)?;
                        write_game(&mut writer, &game)?;
                    }
                    Err(e) => { eprintln!("warning: {e}"); errors += 1; }
                }
            }
            eprintln!("Number of changes to default is {changes}");
        }
        Command::TagRemove { tag } => {
            if tag == "Event" || tag == "FEN" {
                anyhow::bail!("\"{tag}\" tags cannot be removed.");
            }
            let games = PgnParser::new(buf_reader).collect::<Vec<_>>();
            let mut removed = 0usize;
            for game in games {
                match game {
                    Ok(mut game) => {
                        removed += chxss::tools::pgn::filter::tag_remove(&mut game, &tag);
                        write_game(&mut writer, &game)?;
                    }
                    Err(e) => { eprintln!("warning: {e}"); errors += 1; }
                }
            }
            eprintln!("Number of tags removed is {removed}");
        }
    }
    if errors > 0 {
        eprintln!("{errors} game(s) skipped due to parse errors");
    }

    Ok(())
}
