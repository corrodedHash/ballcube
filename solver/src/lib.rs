#![warn(clippy::pedantic)]
#![allow(dead_code)]
pub mod dependency;
pub mod dfs;
mod move_check;
mod win_check;

use ballcube::{Board, Compact, Player};
use move_check::Move;
use rand::Rng;

use crate::move_check::MoveChecker;


fn random_moves(
    board: &Board,
    state: &Compact,
    moves: u8,
    starting_player: Player,
) -> Vec<(Compact, Move)> {
    let move_generator = MoveChecker::new(board);

    (0..moves)
        .scan(*state, |i, m| {
            let player = if (m % 2) == 0 {
                starting_player
            } else {
                starting_player.other()
            };

            let move_array = move_generator.moves(i, player);

            let m = move_array[rand::thread_rng().gen_range(0..move_array.len())];
            i.shift_gate(board, m.layer(), m.gate());
            Some((*i, m))
        })
        .collect()
}
