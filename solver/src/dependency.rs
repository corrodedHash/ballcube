use ballcube::{Board, Compact};

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

#[must_use]
pub fn dependency(board: &Board, state: &Compact, ball: u8) -> [Vec<u8>; 4] {
    let mut result: [Vec<u8>; 4] = Default::default();
    for (layer_index, output) in (0_u8..)
        .zip(result.iter_mut())
        .skip(state.depth()[ball as usize] as usize)
    {
        let layer = board.layer(layer_index);
        let gate_id = gate_id(layer.horizontal(), ball);
        let gate = layer.gate(gate_id);
        let s = state.get_shift(layer_index, gate_id);
        let ball_depth = ball_depth(gate.topleft(), layer.horizontal(), ball);

        let gatetype = gate.gatetype();
        if gatetype != 3 && gatetype >= s && ball_depth <= (gatetype - s) {
            output.push(gatetype - s - ball_depth);
        }
        if gatetype != 2 && s + ball_depth <= 3 {
            output.push(3 - s - ball_depth);
        }
    }
    result
}

fn shift_possibility_string(shift_possibilities: &[Vec<u8>; 4], board: &Board, cell: u8) -> String {
    let x = (0_u8..)
        .zip(shift_possibilities.iter())
        .map(|(layer_index, layer_ints)| {
            let layer = board.layer(layer_index as u8);
            let gate_id = gate_id(layer.horizontal(), cell);
            let gate = layer.gate(gate_id);
            let gatecolor = gate.owner();
            let gatecolor_str = match gatecolor {
                ballcube::Player::Gold => "G",
                ballcube::Player::Silver => "S",
            };
            format!(
                "[{}{}] {:<4}",
                gatecolor_str,
                gate_id,
                layer_ints
                    .clone()
                    .iter()
                    .map(|x| format!("{}", x))
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        })
        .collect::<Vec<String>>()
        .join("; ");
    format!(
        "{} {}",
        board.ball(cell).map_or("X", |x| match x {
            ballcube::Player::Gold => "G",
            ballcube::Player::Silver => "S",
        }),
        x
    )
}

#[cfg(test)]
mod test {
    use super::{dependency, shift_possibility_string};
    use ballcube::{visualize_state, Board, Compact};

    // #[test]
    // fn set_evaluation() {
    //     let board = Board::try_from(0x48c7_8ff0_3e2b_5189).unwrap();
    //     let state = Compact::from(0x0000_0031_410c_1000_0002_00fd);
    //     for i in 0..9 {
    //         println!(
    //             "{}",
    //             shift_possibility_string(&dependency(&board, &state, i), &board, i)
    //         );
    //     }
    //     visualize_state(&board, &state);
    //     println!(
    //         "Board: {:#018x}, State: {:#024x}",
    //         u64::from(&board),
    //         u128::from(&state)
    //     );
    // }

    #[test]
    fn random_evaluation() {
        let board = Board::random();
        let state = Compact::build_from_board(&board);

        visualize_state(&board, &state);

        println!(
            "Board: {:#018x}, State: {:#024x}",
            u64::from(&board),
            u64::from(&state)
        );

        for i in 0..9 {
            println!(
                "{}",
                shift_possibility_string(&dependency(&board, &state, i), &board, i)
            );
        }
    }
}
