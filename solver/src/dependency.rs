use ballcube::{Board, CompactState};

fn gate_id(horizontal: bool, cell: u8) -> u8 {
    if horizontal {
        cell / 3
    } else {
        cell % 3
    }
}

fn ball_depth(topleft: bool, horizontal: bool, cell: u8) -> u8 {
    let topleft_distance = if horizontal { cell % 3 } else { cell / 3 };

    if topleft {
        topleft_distance
    } else {
        2 - topleft_distance
    }
}

fn dependency(board: &Board, state: &CompactState, ball: u8) -> [Vec<u8>; 4] {
    let mut result: [Vec<u8>; 4] = Default::default();
    for (layer_index, output) in (0u8..)
        .zip(result.iter_mut())
        .skip(state.depth()[ball as usize] as usize)
    {
        let gate = gate_id(board.layer_horizontal(layer_index), ball);
        let s = state.get_shift(layer_index, gate);
        let ball_depth = ball_depth(
            board.topleft(layer_index, gate),
            board.layer_horizontal(layer_index),
            ball,
        );

        let gatetype = board.gatetype(layer_index, gate);
        if gatetype != 3 && gatetype >= s && ball_depth <= (gatetype - s) {
            output.push(gatetype - s - ball_depth);
        }
        if gatetype != 2 && s + ball_depth <= 3 {
            output.push(3 - s - ball_depth);
        }
    }
    result
}

#[cfg(test)]
mod test {
    use super::{dependency, gate_id};
    use ballcube::{visualize_state, Board, CompactState};

    fn write_shifts(shift_possibilities: &[Vec<u8>; 4], board: &Board, cell: u8) -> String {
        shift_possibilities
            .iter()
            .enumerate()
            .map(|(layer_index, layer)| {
                let gate = gate_id(board.layer_horizontal(layer_index as u8), cell);
                let gatecolor = board.gate(layer_index as u8, gate);
                let gatecolor_str = match gatecolor {
                    ballcube::Player::Gold => "G",
                    ballcube::Player::Silver => "S",
                };
                format!(
                    "[{}{}] {:<4}",
                    gatecolor_str,
                    gate,
                    layer
                        .clone()
                        .iter()
                        .map(|x| format!("{}", x))
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            })
            .collect::<Vec<String>>()
            .join("; ")
    }

    #[test]
    fn set_evaluation() {
        let board = Board::try_from(0xbf5230d34b00ce90b).unwrap();
        let state = CompactState::from(0x000081021430400002087b);
        for i in 0..9 {
            println!(
                "{} {}",
                board.ball(i).map_or("X", |x| match x {
                    ballcube::Player::Gold => "G",
                    ballcube::Player::Silver => "S",
                }),
                write_shifts(&dependency(&board, &state, i), &board, i)
            );
        }
        visualize_state(&board, &state);
        println!(
            "Board: {:#018x}, State: {:#024x}",
            u128::from(&board),
            u128::from(&state)
        )
    }

    #[test]
    fn random_evaluation() {
        let board = crate::random_board();
        let state = CompactState::build_from_board(&board);

        visualize_state(&board, &state);

        println!(
            "Board: {:#018x}, State: {:#024x}",
            u128::from(&board),
            u128::from(&state)
        );

        for i in 0..9 {
            write_shifts(&dependency(&board, &state, i), &board, i);
        }
    }
}
