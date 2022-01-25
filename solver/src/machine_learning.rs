use ballcube::{Compact, Player};
use rand::Rng;

fn generate_case() -> String {
    let depth = 15;
    let board = ballcube::Board::random();
    let starting_player = if rand::thread_rng().gen() {
        Player::Silver
    } else {
        Player::Gold
    };
    let initial_state = Compact::build_from_board(&board);

    let state_list = crate::random_moves(&board, &initial_state, depth, starting_player);
    let state = state_list.last().unwrap().0;

    let mut result = vec![];

    for i in 0..9 {
        let ball_owner = match board.ball(i) {
            Some(Player::Gold) => 0,
            Some(Player::Silver) => 1,
            None => 2,
        };
        result.push(ball_owner);
        result.push(state.depth()[i as usize]);
    }

    for layer in 0..4 {
        result.push(if board.layer_horizontal(layer) { 1 } else { 0 });
        for gate in 0..3 {
            let gate_owner = match board.gate(layer, gate) {
                Player::Gold => 0,
                Player::Silver => 1,
            };
            let gate_type = board.gatetype(layer, gate);
            let gate_topleft = board.topleft(layer, gate);
            let gate_shift = state.get_shift(layer, gate);

            result.push(gate_owner);
            result.push(gate_type);
            result.push(if gate_topleft { 1 } else { 0 });
            result.push(gate_shift);
        }
    }
    let current_player = if depth % 2 == 0 {
        starting_player
    } else {
        starting_player.other()
    };
    result.push(match current_player {
        Player::Gold => 0,
        Player::Silver => 1,
    });

    let win = match crate::dfs::DFSWinFinder::new(&board).evaluate(&state, current_player, true) {
        crate::dfs::DFSEvaluation::Loss(_) => 0,
        crate::dfs::DFSEvaluation::Win(_) => 1,
        crate::dfs::DFSEvaluation::Draw(_) => 2,
    };
    result.push(win);

    result
        .into_iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn generate_case_list() {
    
}

#[test]
fn bla() {
    println!("{}", generate_case());
}
