mod dfs;
mod move_check;
mod win_check;
use ballcube::{Board, CompactState, Player};
use rand::Rng;

use crate::move_check::MoveChecker;

fn other_player(player: Player) -> Player {
    match player {
        Player::Gold => Player::Silver,
        Player::Silver => Player::Gold,
    }
}

fn knuth_shuffle<T>(v: &mut [T]) {
    use rand;
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let l = v.len();

    for n in 0..l {
        let i = rng.gen_range(0..l - n);
        v.swap(i, l - n - 1);
    }
}

fn random_board(moves: u8) -> (Board, CompactState) {
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

            Some((silver, rand::thread_rng().gen::<bool>(), t))
        })
        .collect::<Vec<_>>();
    let gates: [_; 12] = gates.try_into().unwrap();
    let mut gates_horizontal = [false; 4];
    rand::thread_rng().fill(&mut gates_horizontal);
    let gates_horizontal: [Option<bool>; 4] = gates_horizontal
        .into_iter()
        .map(|x| Some(x))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    let board = ballcube::BoardBuilder {
        gold_balls,
        silver_balls,
        gates_horizontal,
        gates,
    }
    .finalize()
    .unwrap();

    let mut state = CompactState::build_from_board(&board);

    let move_generator = MoveChecker::new(&board);

    for i in 0..moves {
        let player = if (i % 2) == 0 {
            Player::Gold
        } else {
            Player::Silver
        };

        let move_array = move_generator.moves(&state, player);

        let chosen_move = move_array[rand::thread_rng().gen_range(0..move_array.len())];
        state.shift_gate(&board, chosen_move.layer(), chosen_move.gate());
    }
    (board, state)
}
