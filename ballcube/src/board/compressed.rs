use std::fmt::Display;

use deku::prelude::*;

use crate::Board;

#[derive(Debug, Clone, DekuRead, DekuWrite)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
struct TwoWideInt(#[deku(bits = 2)] u8);

#[derive(Debug, Clone, DekuRead, DekuWrite)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
struct OneWideBool(#[deku(bits = 1)] bool);

#[derive(Debug, Clone, DekuRead, DekuWrite)]
#[deku(endian = "big")]
#[allow(clippy::module_name_repetitions)]
pub struct CompressedBoard {
    gold_balls: [OneWideBool; 9],
    #[deku(bits = 3)]
    empty_cell_index: u8,
    gates_horizontal: [OneWideBool; 4],
    gates_topleft: [[OneWideBool; 3]; 4],
    gates_silver: [[OneWideBool; 3]; 4],
    gates_type: [[TwoWideInt; 3]; 4],
}

#[allow(clippy::fallible_impl_from)]
impl From<&Board> for CompressedBoard {
    fn from(board: &Board) -> Self {
        let empty_cell_iterator =
            (0..9).find(|x| !board.gold_balls.contains(x) && !board.silver_balls.contains(x));
        let empty_cell = empty_cell_iterator.unwrap();
        let empty_cell_delta = board.gold_balls.iter().filter(|x| x < &&empty_cell).count() as u8;
        let range_array = [0, 1, 2, 3, 4, 5, 6, 7, 8];
        Self {
            gold_balls: range_array.map(|x| OneWideBool(board.gold_balls.contains(&x))),
            empty_cell_index: empty_cell - empty_cell_delta,
            gates_horizontal: board.gates_horizontal.map(OneWideBool),
            gates_topleft: board.gates_topleft.map(|x| x.map(OneWideBool)),
            gates_silver: board.gates_silver.map(|x| x.map(OneWideBool)),
            gates_type: board.gate_type.map(|x| x.map(TwoWideInt)),
        }
    }
}

#[derive(Clone, Debug, thiserror::Error)]
pub struct BallError {
    silver: Vec<u8>,
    gold: Vec<u8>,
}

impl Display for BallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = format!(
            "[{}]",
            self.silver
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(" ")
        );
        let g = format!(
            "[{}]",
            self.gold
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(" ")
        );
        write!(f, "({}, {})", s, g)
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum IncorrectCompBoardError {
    #[error("Incorrect ball amount: {0}")]
    IncorrectBallAmount(#[from] BallError),
}

impl TryFrom<CompressedBoard> for Board {
    type Error = IncorrectCompBoardError;

    fn try_from(compboard: CompressedBoard) -> Result<Self, Self::Error> {
        let gold_ball_indices = (0_u8..9)
            .filter(|x| compboard.gold_balls[*x as usize].0)
            .collect::<Vec<_>>();

        let empty_cell_index =
            gold_ball_indices
                .iter()
                .fold(compboard.empty_cell_index, |x, gold_index| {
                    if *gold_index <= x {
                        x + 1
                    } else {
                        x
                    }
                });

        let silver_ball_indices = (0..9)
            .filter(|x| *x != empty_cell_index && !gold_ball_indices.contains(x))
            .collect::<Vec<_>>();

        let gates_horizontal = compboard.gates_horizontal.map(|x| x.0);

        let gates_topleft = compboard.gates_topleft.map(|x| x.map(|v| v.0));
        let gates_silver = compboard.gates_silver.map(|x| x.map(|v| v.0));
        let gate_type = compboard.gates_type.map(|x| x.map(|v| v.0));
        let ball_error = BallError {
            silver: silver_ball_indices.clone(),
            gold: gold_ball_indices.clone(),
        };
        Ok(Self {
            gold_balls: gold_ball_indices
                .try_into()
                .map_err(|_x| ball_error.clone())?,
            silver_balls: silver_ball_indices
                .try_into()
                .map_err(|_x| ball_error.clone())?,
            gates_horizontal,
            gates_topleft,
            gates_silver,
            gate_type,
        })
    }
}
