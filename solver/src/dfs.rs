use crate::move_check::{Move, MoveChecker};
use crate::win_check::{Winner, WinningChecker};
use ballcube::{Board, CompactState, Player};

struct DFSWinFinder<'a> {
    checker: WinningChecker,
    move_generator: MoveChecker,
    board: &'a Board,
}

#[derive(Clone, Debug)]
enum DFSEvaluation {
    Win(Vec<Move>),
    Draw(Vec<Move>),
    Loss(Vec<Move>),
}

impl<'a> DFSWinFinder<'a> {
    fn new(board: &'a Board) -> Self {
        let checker = WinningChecker::new(board);
        let move_generator = MoveChecker::new(board);

        Self {
            checker,
            move_generator,
            board,
        }
    }
    fn evaluate(&self, state: &CompactState, player: Player) -> DFSEvaluation {
        let mut draw = None;
        let mut loss = None;
        for m in self.move_generator.moves(state, player) {
            let mut new_state = *state;
            new_state.shift_gate(self.board, m.layer(), m.gate());
            match self.checker.won(&new_state) {
                Winner::None => (),
                Winner::Both => {
                    draw = Some(DFSEvaluation::Draw(vec![m]));
                    continue;
                }
                Winner::One(x) if x == player => return DFSEvaluation::Win(vec![m]),
                Winner::One(_) => {
                    loss = Some(DFSEvaluation::Loss(vec![m]));
                    continue;
                }
            };
            let ev = self.evaluate(&new_state, player.other());
            match ev {
                DFSEvaluation::Win(mut moves) => {
                    moves.push(m);
                    loss = Some(DFSEvaluation::Loss(moves))
                }
                DFSEvaluation::Loss(mut moves) => {
                    moves.push(m);
                    return DFSEvaluation::Win(moves);
                }
                DFSEvaluation::Draw(mut moves) => {
                    moves.push(m);
                    draw = Some(DFSEvaluation::Draw(moves))
                }
            }
        }

        if let Some(x) = draw {
            x
        } else if let Some(x) = loss {
            x
        } else {
            match self.checker.won(state) {
                Winner::None => {
                    dbg!(state.shift_count(), state.shift_count_silver(self.board));
                    dbg!(state.depth());
                    panic!("No moves but no winner?")
                }
                Winner::Both => DFSEvaluation::Draw(vec![]),
                Winner::One(x) if x == player => DFSEvaluation::Win(vec![]),
                Winner::One(_) => DFSEvaluation::Loss(vec![]),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use ballcube::{visualize_state, Board, CompactState, Player};

    use crate::{
        dfs::DFSWinFinder,
        win_check::{Winner, WinningChecker},
    };

    #[test]
    fn set_evaluation() {
        let board = Board::try_from(0x207de0ed51c7d29495).unwrap();
        let state = CompactState::from(0xa520a9fa50cd3f0000032c09);

        let (ev_str, moves) = match DFSWinFinder::new(&board).evaluate(&state, Player::Silver) {
            crate::dfs::DFSEvaluation::Win(x) => ("Win", x),
            crate::dfs::DFSEvaluation::Draw(x) => ("Draw", x),
            crate::dfs::DFSEvaluation::Loss(x) => ("Loss", x),
        };

        let mut a = "".to_owned();
        let mut b = "".to_owned();
        for m in moves {
            a = format!("{} {}", a, m.layer());
            b = format!("{} {}", b, m.gate());
        }
        println!("{}\n{}\n{}", ev_str, a, b);

        visualize_state(&board, &state);
        println!(
            "Board: {:#018x}, State: {:#024x}",
            u128::from(&board),
            u128::from(&state)
        );
    }

    #[test]
    fn random_evaluation() {
        let board = crate::random_board();
        let state = CompactState::build_from_board(&board);

        let mut state_stack = crate::random_moves(&board, &state, 36, Player::Silver);

        let win_checker = WinningChecker::new(&board);
        while win_checker.won(&state_stack.last().unwrap().0) != Winner::None {
            state_stack.pop();
        }

        while let Some(chosen_state) = state_stack.last() {
            let chosen_state = chosen_state.0;
            let player = if state_stack.len() % 2 == 1 {
                Player::Gold
            } else {
                Player::Silver
            };
            let ev = DFSWinFinder::new(&board).evaluate(&chosen_state, player);
            let (ev_str, len) = match ev {
                crate::dfs::DFSEvaluation::Win(x) => ("Win", x.len()),
                crate::dfs::DFSEvaluation::Draw(x) => ("Draw", x.len()),
                crate::dfs::DFSEvaluation::Loss(x) => ("Loss", x.len()),
            };
            println!("{} for {:#?} in {:02} turns", ev_str, player, len);
            visualize_state(&board, &chosen_state);

            println!(
                "[{:02}] Board: {:#018x}, State: {:#024x}",
                state_stack.len(),
                u128::from(&board),
                u128::from(&chosen_state)
            );

            state_stack.pop();
            if state_stack.len() <= 13 {
                break;
            }
        }
    }
}
