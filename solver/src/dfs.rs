use crate::move_check::{Move, MoveChecker};
use crate::other_player;
use crate::win_check::{Winner, WinningChecker};
use ballcube::{Board, CompactState, Player};

struct DFSWinFinder {
    checker: WinningChecker,
    move_generator: MoveChecker,
}

#[derive(Clone, Copy, Debug)]
enum DFSEvaluation {
    ForcedWin(Move),
    ForcedDraw(Move),
    ForcedLoss,
}

impl DFSWinFinder {
    fn new(board: &Board) -> Self {
        let checker = WinningChecker::new(board);
        let move_generator = MoveChecker::new(board);

        Self {
            checker,
            move_generator,
        }
    }
    fn evaluate(&self, state: &CompactState, player: Player) -> DFSEvaluation {
        let mut draw = None;

        for m in self.move_generator.moves(state, player) {
            let new_state = *state;

            match self.checker.won(&new_state) {
                Winner::None => (),
                Winner::Both => draw = Some(DFSEvaluation::ForcedDraw(m)),
                Winner::One(x) if x == player => return DFSEvaluation::ForcedWin(m),
                _ => continue,
            };
            let ev = self.evaluate(&new_state, other_player(player));
            match ev {
                DFSEvaluation::ForcedWin(_) => continue,
                DFSEvaluation::ForcedLoss => return DFSEvaluation::ForcedWin(m),
                DFSEvaluation::ForcedDraw(_) => draw = Some(DFSEvaluation::ForcedDraw(m)),
            }
        }

        if let Some(x) = draw {
            x
        } else {
            DFSEvaluation::ForcedLoss
        }
    }
}
