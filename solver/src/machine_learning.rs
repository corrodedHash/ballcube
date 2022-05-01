use ballcube::{Compact, Player};
use rand::Rng;

fn generate_case(depth: usize) -> String {
    let board = ballcube::Board::random();

    let starting_player = if rand::thread_rng().gen() {
        Player::Silver
    } else {
        Player::Gold
    };

    let state_list = Compact::build_from_board(&board).random_game(&board, starting_player);
    let state = if let Some(x) = state_list.get(depth) {
        x.0
    } else {
        return generate_case(depth);
    };

    let mut result = vec![];

    for (i, d) in (0..).zip(state.depth()) {
        let ball_owner = match board.ball(i) {
            Some(Player::Gold) => 0,
            Some(Player::Silver) => 1,
            None => 2,
        };
        result.push(ball_owner);
        result.push(d);
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

    // First state is where starting player already moved, with index 0
    let current_player = if depth % 2 == 0 {
        starting_player.other()
    } else {
        starting_player
    };
    result.push(match current_player {
        Player::Gold => 0,
        Player::Silver => 1,
    });

    if state.shift_count() % 2 != 0 {
        match current_player {
            Player::Gold => {
                debug_assert_eq!(state.shift_count() / state.shift_count_silver(&board), 1);
            }
            Player::Silver => {
                debug_assert_eq!(state.shift_count() / state.shift_count_silver(&board), 2);
            }
        }
    }

    let win = match crate::dfs::DFSWinFinder::new(&board).evaluate(&state, current_player, true) {
        crate::dfs::DFSEvaluation::Loss(_) => 0,
        crate::dfs::DFSEvaluation::Win(_) => 1,
        crate::dfs::DFSEvaluation::Draw(_) => 2,
    };
    result.push(win);

    format!(
        "{:#018X}, {:#018X}, {}, {}",
        u64::from(&board),
        u64::from(&state),
        depth + 1,
        result
            .into_iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    )
}

pub fn generate_case_list() {
    use std::io::Write;

    let mut header_items = vec![];
    let filename = "data.csv";
    let file_already_exists = std::path::Path::new(filename).exists();
    let mut output_file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(filename)
        .expect("Could not open file");
    if !file_already_exists {
        for i in 1..=9 {
            header_items.push(format!("Ball{}_Owner", i));
            header_items.push(format!("Ball{}_Depth", i));
        }

        for layer in 1..=4 {
            header_items.push(format!("Layer{}_Horizontal", layer));
            for gate in 1..=3 {
                header_items.push(format!("Layer{}_Gate{}_Owner", layer, gate));
                header_items.push(format!("Layer{}_Gate{}_Type", layer, gate));
                header_items.push(format!("Layer{}_Gate{}_Topleft", layer, gate));
                header_items.push(format!("Layer{}_Gate{}_Shift", layer, gate));
            }
        }
        header_items.push("CurrentPlayer".to_string());
        header_items.push("CurrentPlayerWin".to_string());

        let header = header_items.join(",");

        writeln!(output_file, "Board,State,Depth,{}", header).expect("Could not write header");
    }

    for i in 0..1000 {
        writeln!(output_file, "{}", generate_case(14)).expect("Could not write line");
        println!("Wrote case #{:04}", i);
    }
}

#[test]
#[ignore]
fn bla() {
    use crate::dfs::DFSWinFinder;
    use ballcube::Board;

    let board = Board::try_from(0xC4DC_AC95_A46A_24E4).unwrap();
    let state = Compact::from_u64(0x06AC_50E1_7079, &board);

    dbg!(DFSWinFinder::new(&board)
        .evaluate(&state, Player::Gold, true)
        .is_win());
    dbg!(DFSWinFinder::new(&board)
        .evaluate(&state, Player::Silver, true)
        .is_win());
}
