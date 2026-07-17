use clap::Subcommand;

#[derive(Subcommand)]
pub enum Command {
    /// Remove games missing Elo tags
    MinElo {
        /// Minimum Elo rating
        min: u16,
        /// Maximum Elo rating
        max: u16,
    },

    /// Filter games by minimum ply count
    MinPly {
        /// Minimum number of plies
        min: usize,
    },
}
