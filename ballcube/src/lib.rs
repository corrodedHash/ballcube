//! Storage and utility access function to a game of ballcube
//!
#![warn(clippy::pedantic, clippy::restriction)]
#![warn(clippy::nursery)]
#![allow(clippy::cast_possible_truncation)]
#![allow(
    clippy::integer_arithmetic,
    clippy::integer_division,
    clippy::implicit_return,
    clippy::missing_inline_in_public_items,
    clippy::exhaustive_enums,
    clippy::exhaustive_structs,
    clippy::separated_literal_suffix,
    clippy::missing_docs_in_private_items,
    clippy::mod_module_files,
    clippy::default_numeric_fallback,
    clippy::as_conversions,
    clippy::indexing_slicing,
    clippy::expect_used,
    clippy::unwrap_used
)]

mod board;
mod move_check;
mod state;
mod visualize_state;
mod win_check;

pub use move_check::{Move, MoveChecker};
pub use win_check::{Winner, WinningChecker};

pub use board::builder::{BoardBuilder, BoardBuildingError};
pub use board::Board;
pub use state::Compact;
pub use visualize_state::visualize_state;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
/// Color of balls a player controls
pub enum Player {
    Gold,
    Silver,
}

impl Player {
    #[must_use]
    /// The player opposing current player
    pub const fn other(self) -> Self {
        match self {
            Self::Gold => Self::Silver,
            Self::Silver => Self::Gold,
        }
    }
}
