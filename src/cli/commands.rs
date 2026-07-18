use clap::Subcommand;

#[derive(Subcommand)]
pub enum Command {
    /// Filter games by Elo range(s): 1 value (lower bound), 2 values (range), or 4 values (two ranges)
    MinElo {
        /// Elo values: 1 (lower bound), 2 (min max), or 4 (min1 max1 min2 max2)
        #[arg(num_args = 1..=4, required = true)]
        vals: Vec<u16>,
    },

    /// Remove games missing WhiteElo or BlackElo tags
    EloCheck,

    /// Reorder tags to standard order (STR first, PlyCount last)
    TagOrder,

    /// Replace all values of a tag type with its default/null value
    TagNull {
        /// Tag name (case-sensitive)
        tag: String,
    },

    /// Remove all instances of a tag type (except Event and FEN)
    TagRemove {
        /// Tag name to remove (case-sensitive)
        tag: String,
    },
}
