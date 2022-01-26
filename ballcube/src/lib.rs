#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]

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
pub enum Player {
    Gold,
    Silver,
}

impl Player {
    #[must_use]
    pub fn other(self) -> Self {
        match self {
            Player::Gold => Player::Silver,
            Player::Silver => Player::Gold,
        }
    }
}
