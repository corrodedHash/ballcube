use ballcube::{Board, Compact, Player};

#[derive(Debug, Clone, Copy)]
pub struct Island {
    distance: u8,
    ball_id: u8,
    layer: u8,
    gate: u8,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct IslandMeasure {
    pub silver_definite: Option<Island>,
    pub silver_heuristic: Option<Island>,
    pub gold_definite: Option<Island>,
    pub gold_heuristic: Option<Island>,
}

fn relevant_balls(board: &Board, state: &Compact, layer: u8, gate: u8) -> [Option<u8>; 3] {
    let h = board.layer(layer).horizontal();

    let a = if h { [0, 1, 2] } else { [0, 3, 6] };
    let deltas = if h { [3, 3, 3] } else { [1, 1, 1] };

    let scaled_deltas = deltas.map(|x| x * gate);
    let positioned_cells = a.iter().zip(scaled_deltas).map(|(a, b)| a + b);
    let cell_ids: [u8; 3] = if board.layer(layer).gate(gate).topleft() {
        positioned_cells.collect::<Vec<_>>().try_into().unwrap()
    } else {
        positioned_cells
            .rev()
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    };

    cell_ids.map(|x| {
        if state.depth()[x as usize] > layer {
            None
        } else {
            Some(x)
        }
    })
}

#[allow(clippy::too_many_lines)]
fn measure_gate_island(
    board: &Board,
    state: &Compact,
    layer: u8,
    gate: u8,
) -> (Option<(u8, u8)>, Option<(u8, u8)>) {
    #[derive(Debug, Clone, Copy)]
    struct BallInfo {
        owned: bool,
        on_gate: bool,
        ball_id: u8,
    }

    let balls = relevant_balls(board, state, layer, gate);
    let gate_p = board.layer(layer).gate(gate);

    let ball_info = balls
        .iter()
        .map(|x| {
            x.as_ref().map(|x| BallInfo {
                owned: board.ball(*x).unwrap() == gate_p.owner(),
                on_gate: state.depth()[*x as usize] == layer,
                ball_id: *x,
            })
        })
        .collect::<Vec<_>>();

    let (mut definite, mut heuristic) = ((4, 9), (4, 9));
    let update_var = |mut a: &mut (u8, u8), distance, ball| {
        if a.0 > distance {
            a.0 = distance;
            a.1 = ball;
        }
    };

    if let Some(a) = ball_info[2] {
        if !a.owned {
            let first_ball_fine = ball_info[0].map_or(true, |i| !i.owned || gate_p.gatetype() == 0);
            let second_ball_fine =
                ball_info[1].map_or(true, |i| !i.owned || gate_p.gatetype() == 1);

            let no_hole_last = gate_p.gatetype() != 2;

            // Gate cannot be shifted, otherwise ball would not be relevant.
            // Just here for clarity
            let not_shifted = state.get_shift(layer, gate) == 0;

            if no_hole_last && not_shifted && first_ball_fine && second_ball_fine {
                update_var(&mut definite, 3, a.ball_id);
                update_var(&mut heuristic, 3, a.ball_id);
            }
        }
    }

    if let Some(a) = ball_info[1] {
        if !a.owned {
            match state.get_shift(layer, gate) {
                0 => match gate_p.gatetype() {
                    0 => {
                        let first_ball_fine = ball_info[0].map_or(true, |i| !i.owned);
                        if first_ball_fine {
                            update_var(&mut definite, 2, a.ball_id);
                            update_var(&mut heuristic, 2, a.ball_id);
                        } else {
                            debug_assert!(ball_info[0].unwrap().owned);
                            update_var(&mut heuristic, 2, a.ball_id);
                        }
                    }
                    1 => {
                        // We need to shift this gate before the ball drops on it, and if the first ball is ours too, we need to wait for it to pass before we shift
                        update_var(&mut heuristic, 2, a.ball_id);
                    }
                    2 => {
                        let first_ball_fine = ball_info[0].map_or(true, |i| !i.owned);
                        if first_ball_fine {
                            update_var(&mut definite, 2, a.ball_id);
                            update_var(&mut heuristic, 2, a.ball_id);
                        }
                    }
                    3 => {
                        let first_ball_fine = ball_info[0].map_or(true, |i| !i.owned);
                        if first_ball_fine {
                            update_var(&mut definite, 2, a.ball_id);
                            update_var(&mut heuristic, 2, a.ball_id);
                        }
                    }
                    _ => panic!("Gatetype larger than 3"),
                },
                1 => {
                    let no_hole = gate_p.gatetype() != 2;
                    let first_ball_fine =
                        ball_info[0].map_or(true, |i| !i.owned || gate_p.gatetype() == 1);
                    if no_hole && first_ball_fine {
                        update_var(&mut definite, 2, a.ball_id);
                        update_var(&mut heuristic, 2, a.ball_id);
                    }
                }
                _ => panic!("How is ball 1 relevant with a shift of more than 1?"),
            }
        }
    }

    if let Some(a) = ball_info[0] {
        if !a.owned {
            match state.get_shift(layer, gate) {
                0 => match gate_p.gatetype() {
                    0 => {
                        update_var(&mut heuristic, 1, a.ball_id);
                    }
                    1 => {
                        let third_ball_fine = ball_info[2].map_or(true, |i| !i.owned);
                        if a.on_gate {
                            if third_ball_fine {
                                update_var(&mut definite, 3, a.ball_id);
                                update_var(&mut heuristic, 3, a.ball_id);
                            }
                        } else {
                            update_var(&mut heuristic, 1, a.ball_id);
                        }
                    }
                    2 => {
                        update_var(&mut definite, 2, a.ball_id);
                        update_var(&mut heuristic, 2, a.ball_id);
                    }
                    3 => {
                        update_var(&mut definite, 1, a.ball_id);
                        update_var(&mut heuristic, 1, a.ball_id);
                    }
                    _ => panic!("Unknown gatetype"),
                },
                1 => match gate_p.gatetype() {
                    0 | 3 => {
                        update_var(&mut definite, 1, a.ball_id);
                        update_var(&mut heuristic, 1, a.ball_id);
                    }
                    1 => {
                        update_var(&mut heuristic, 1, a.ball_id);
                    }

                    2 => {
                        update_var(&mut definite, 2, a.ball_id);
                        update_var(&mut heuristic, 2, a.ball_id);
                    }
                    _ => panic!("Unknown gatetype"),
                },
                2 => {
                    if gate_p.gatetype() != 2 {
                        update_var(&mut definite, 1, a.ball_id);
                        update_var(&mut heuristic, 1, a.ball_id);
                    }
                }
                _ => panic!("We shifted the gate completely?"),
            }
        }
    }

    (
        if definite.0 >= 4 {
            None
        } else {
            Some(definite)
        },
        if heuristic.0 >= 4 {
            None
        } else {
            Some(heuristic)
        },
    )
}

pub fn measure_island(board: &Board, state: &Compact) -> IslandMeasure {
    let option_max = |a: Option<Island>, b: Option<Island>| {
        if let Some(a) = a {
            if let Some(b) = b {
                if a.distance > b.distance {
                    Some(b)
                } else {
                    Some(a)
                }
            } else {
                Some(a)
            }
        } else {
            b
        }
    };

    let mut i = IslandMeasure::default();
    for layer_id in 0..4 {
        for gate_id in 0..3 {
            let (d, h) = measure_gate_island(board, state, layer_id, gate_id);
            let di = d.map(|d| Island {
                ball_id: d.1,
                distance: d.0,
                gate: gate_id,
                layer: layer_id,
            });
            let hi = h.map(|h| Island {
                ball_id: h.1,
                distance: h.0,
                gate: gate_id,
                layer: layer_id,
            });
            if board.layer(layer_id).gate(gate_id).owner() == Player::Silver {
                i.silver_definite = option_max(i.silver_definite, di);
                i.silver_heuristic = option_max(i.silver_heuristic, hi);
            } else {
                i.gold_definite = option_max(i.gold_definite, di);
                i.gold_heuristic = option_max(i.gold_heuristic, hi);
            }
        }
    }
    i
}
