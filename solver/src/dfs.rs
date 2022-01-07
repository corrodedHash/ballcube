use crate::move_check::{Move, MoveChecker};
use crate::win_check::{Winner, WinningChecker};
use ballcube::{Board, Compact, Player};

#[derive(Clone, Debug)]
pub struct MoveChain {
    chain: Vec<Move>,
    starting_player: Player,
}
impl MoveChain {
    fn new(starting_player: Player) -> Self {
        Self {
            chain: vec![],
            starting_player,
        }
    }
    fn prepend(&mut self, m: Move) {
        self.chain.push(m);
        self.starting_player = self.starting_player.other();
    }

    #[must_use]
    pub fn moves(&self) -> &Vec<Move> {
        &self.chain
    }

    #[must_use]
    pub fn starting_player(&self) -> Player {
        self.starting_player
    }
}

pub struct DFSWinFinder<'a> {
    checker: WinningChecker,
    move_generator: MoveChecker,
    board: &'a Board,
}

#[derive(Clone, Debug)]
pub enum DFSEvaluation {
    Win(MoveChain),
    Draw(MoveChain),
    Loss(MoveChain),
}

impl Eq for DFSEvaluation {
    fn assert_receiver_is_total_eq(&self) {}
}

impl DFSEvaluation {
    fn flip(mut self) -> Self {
        self = match self {
            Self::Win(x) => Self::Loss(x),
            Self::Draw(x) => Self::Draw(x),
            Self::Loss(x) => Self::Win(x),
        };
        self
    }

    fn add_move(&mut self, m: Move) {
        self.moves_mut().prepend(m);
    }
    fn moves(&self) -> &MoveChain {
        match self {
            Self::Win(x) | Self::Draw(x) | Self::Loss(x) => x,
        }
    }
    fn moves_mut(&mut self) -> &mut MoveChain {
        match self {
            Self::Win(x) | Self::Draw(x) | Self::Loss(x) => x,
        }
    }
}

impl PartialEq for DFSEvaluation {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Win(l0), Self::Win(r0))
            | (Self::Draw(l0), Self::Draw(r0))
            | (Self::Loss(l0), Self::Loss(r0)) => l0.moves().len() == r0.moves().len(),
            _ => false,
        }
    }
}

impl Ord for DFSEvaluation {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self {
            DFSEvaluation::Win(x) => {
                if let Self::Win(y) = other {
                    x.moves().len().cmp(&y.moves().len()).reverse()
                } else {
                    std::cmp::Ordering::Greater
                }
            }
            DFSEvaluation::Draw(x) => match other {
                DFSEvaluation::Win(_y) => std::cmp::Ordering::Less,
                DFSEvaluation::Draw(y) => x.moves().len().cmp(&y.moves().len()).reverse(),
                DFSEvaluation::Loss(_y) => std::cmp::Ordering::Greater,
            },
            DFSEvaluation::Loss(x) => {
                if let Self::Loss(y) = other {
                    x.moves().len().cmp(&y.moves().len())
                } else {
                    std::cmp::Ordering::Less
                }
            }
        }
    }
}

impl PartialOrd for DFSEvaluation {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> DFSWinFinder<'a> {
    #[must_use]
    pub fn new(board: &'a Board) -> Self {
        let checker = WinningChecker::new(board);
        let move_generator = MoveChecker::new(board);

        Self {
            checker,
            move_generator,
            board,
        }
    }

    /// # Panics
    /// Panics when the state is in an invalid state
    #[must_use]
    pub fn evaluate(
        &self,
        state: &Compact,
        player: Player,
        prune_alpha_beta: bool,
    ) -> DFSEvaluation {
        match self.checker.won(state) {
            Winner::None => (),
            Winner::Both => return DFSEvaluation::Draw(MoveChain::new(player)),
            Winner::One(x) if x == player => return DFSEvaluation::Win(MoveChain::new(player)),
            Winner::One(_) => return DFSEvaluation::Loss(MoveChain::new(player)),
        };

        let mut best_option = None;
        for m in self.move_generator.moves(state, player) {
            let mut new_state = *state;
            new_state.shift_gate(self.board, m.layer(), m.gate());

            let mut ev = self
                .evaluate(&new_state, player.other(), prune_alpha_beta)
                .flip();
            ev.add_move(m);

            if let Some(ref b) = best_option {
                if &ev > b {
                    best_option = Some(ev);
                }
            } else {
                best_option = Some(ev);
            }
            if prune_alpha_beta {
                if let Some(DFSEvaluation::Win(x)) = best_option {
                    return DFSEvaluation::Win(x);
                }
            }
        }

        if let Some(x) = best_option {
            x
        } else {
            dbg!(state.shift_count(), state.shift_count_silver(self.board));
            dbg!(state.depth());
            panic!("No moves but no winner?");
        }
    }
}

#[cfg(test)]
mod test {
    use ballcube::{visualize_state, Board, Compact, Player};

    use crate::{
        dfs::DFSWinFinder,
        win_check::{Winner, WinningChecker},
    };

    use super::MoveChain;

    fn check_moves(board: &Board, state: &Compact, moves: &MoveChain) {
        let mut state = *state;
        for (i, m) in moves.moves().iter().rev().enumerate() {
            let current_player = if i % 2 == 0 {
                moves.starting_player()
            } else {
                moves.starting_player().other()
            };
            if board.gate(m.layer(), m.gate()) != current_player {
                dbg!(moves, i);
                visualize_state(board, &state);
                panic!("Move does not fit player");
            }
            if state.get_shift(m.layer(), m.gate()) >= 3 {
                dbg!(moves, i);
                visualize_state(board, &state);
                panic!("Move already removed gate");
            }
            state.shift_gate(board, m.layer(), m.gate());
        }
    }
    #[test]
    fn set_evaluation() {
        let board = Board::try_from(0x0033_84c6_f527_b099_a923).unwrap();
        let state = Compact::from(0xee3f_bafe_f3ff_7bf0_10c0_0840);
        let start_player = Player::Silver;
        let (ev_str, moves) = match DFSWinFinder::new(&board).evaluate(&state, start_player, true) {
            crate::dfs::DFSEvaluation::Win(x) => ("Win", x),
            crate::dfs::DFSEvaluation::Draw(x) => ("Draw", x),
            crate::dfs::DFSEvaluation::Loss(x) => ("Loss", x),
        };

        let mut layer_string = "".to_owned();
        let mut gate_string = "".to_owned();
        let mut turn_string = "".to_owned();

        for m in moves.moves().iter().copied() {
            layer_string = format!("{} {}", layer_string, m.layer());
            gate_string = format!("{} {}", gate_string, m.gate());
            let p = match board.gate(m.layer(), m.gate()) {
                Player::Gold => "G",
                Player::Silver => "S",
            };
            turn_string = format!("{} {}", turn_string, p);
        }
        println!(
            "{}\nPlayer: {}\nLayer:  {}\nGate:   {}",
            ev_str, turn_string, layer_string, gate_string
        );

        visualize_state(&board, &state);
        println!(
            "Board: {:#018x}, State: {:#024x}",
            u128::from(&board),
            u128::from(&state)
        );

        assert_eq!(moves.starting_player(), start_player);
        check_moves(&board, &state, &moves);
    }

    #[test]
    fn random_evaluation() {
        let board = crate::random_board();
        let state = Compact::build_from_board(&board);

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
            let ev = DFSWinFinder::new(&board).evaluate(&chosen_state, player, true);
            let ev_str = match &ev {
                crate::dfs::DFSEvaluation::Win(_) => "Win",
                crate::dfs::DFSEvaluation::Draw(_) => "Draw",
                crate::dfs::DFSEvaluation::Loss(_) => "Loss",
            };

            println!(
                "{} for {:#?} in {:02} turns",
                ev_str,
                player,
                ev.moves().moves().len()
            );
            visualize_state(&board, &chosen_state);

            println!(
                "[{:02}] Board: {:#018x}, State: {:#024x}",
                state_stack.len(),
                u128::from(&board),
                u128::from(&chosen_state)
            );

            check_moves(&board, &chosen_state, ev.moves());

            state_stack.pop();
            if state_stack.len() <= 13 {
                break;
            }
        }
    }
}
