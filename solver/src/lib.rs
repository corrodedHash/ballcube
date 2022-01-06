#![allow(dead_code)]
mod dependency;
pub mod dfs;
mod move_check;
mod win_check;

use ballcube::{Board, CompactState, Gate, Player};
use move_check::Move;
use rand::Rng;

use crate::move_check::MoveChecker;

fn knuth_shuffle<T>(v: &mut [T]) {
    let mut rng = rand::thread_rng();
    let l = v.len();

    for n in 0..l {
        let i = rng.gen_range(0..l - n);
        v.swap(i, l - n - 1);
    }
}

fn random_board() -> Board {
    let mut balls = (0u8..9).collect::<Vec<_>>();
    knuth_shuffle(&mut balls);

    let gold_balls = balls[0..4].to_vec();
    let silver_balls = balls[4..8].to_vec();
    let gate_types = [0u8, 0, 1, 2, 3, 3];
    let mut gold_gates = gate_types.to_vec();
    let mut silver_gates = gate_types.to_vec();
    let mut gate_distribution = vec![false; 6];
    gate_distribution.extend(vec![true; 6]);

    knuth_shuffle(&mut gold_gates);
    knuth_shuffle(&mut silver_gates);
    knuth_shuffle(&mut gate_distribution);

    let gates = gate_distribution
        .into_iter()
        .map(|silver| {
            let t = if silver {
                silver_gates.pop()
            } else {
                gold_gates.pop()
            }
            .unwrap();

            Some(Gate {
                allegiance: if silver { Player::Silver } else { Player::Gold },
                gatetype: t,
                topleft: rand::thread_rng().gen::<bool>(),
            })
        })
        .collect::<Vec<_>>();
    let gates: [_; 12] = gates.try_into().unwrap();
    let mut gates_horizontal = [true; 4];
    rand::thread_rng().fill(&mut gates_horizontal);
    let gates_horizontal: [Option<bool>; 4] = gates_horizontal
        .into_iter()
        .map(Some)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    ballcube::BoardBuilder {
        gold_balls,
        silver_balls,
        gates_horizontal,
        gates,
    }
    .finalize()
    .unwrap()
}

fn random_moves(
    board: &Board,
    state: &CompactState,
    moves: u8,
    starting_player: Player,
) -> Vec<(CompactState, Move)> {
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
