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
    #[deku(count = "9")]
    gold_balls: Vec<OneWideBool>,
    #[deku(bits = 3)]
    empty_cell_index: u8,
    #[deku(count = "4")]
    gates_horizontal: Vec<OneWideBool>,
    #[deku(count = "12")]
    gates_topleft: Vec<OneWideBool>,
    #[deku(count = "12")]
    gates_silver: Vec<OneWideBool>,
    #[deku(count = "12")]
    gates_type: Vec<TwoWideInt>,
}

#[allow(clippy::fallible_impl_from)]
impl From<&Board> for CompressedBoard {
    fn from(board: &Board) -> Self {
        let empty_cell_iterator =
            (0..9).find(|x| !board.gold_balls.contains(x) && !board.silver_balls.contains(x));
        let empty_cell = empty_cell_iterator.unwrap();
        let empty_cell_delta = board.gold_balls.iter().filter(|x| x < &&empty_cell).count() as u8;

        Self {
            gold_balls: (0..9)
                .map(|x| board.gold_balls.contains(&x))
                .map(OneWideBool)
                .collect(),
            empty_cell_index: empty_cell - empty_cell_delta,
            gates_horizontal: board.gates_horizontal.map(OneWideBool).to_vec(),
            gates_topleft: board
                .gates_topleft
                .iter()
                .flat_map(|x| x.iter())
                .copied()
                .map(OneWideBool)
                .collect(),
            gates_silver: board
                .gates_silver
                .iter()
                .flat_map(|x| x.iter())
                .copied()
                .map(OneWideBool)
                .collect(),
            gates_type: board
                .gate_type
                .iter()
                .flat_map(|x| x.iter())
                .copied()
                .map(TwoWideInt)
                .collect(),
        }
    }
}

#[derive(Clone, Debug)]
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
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        );
        let g = format!(
            "[{}]",
            self.gold
                .iter()
                .map(|x| x.to_string())
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
    type Error = ();

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

        let gates_horizontal = compboard
            .gates_horizontal
            .into_iter()
            .map(|x| x.0)
            .collect::<Vec<_>>();

        let gates_topleft = compboard
            .gates_topleft
            .chunks_exact(3)
            .map(|x| [x[0].0, x[1].0, x[2].0])
            .collect::<Vec<_>>();

        let gates_silver = compboard
            .gates_silver
            .chunks_exact(3)
            .map(|x| [x[0].0, x[1].0, x[2].0])
            .collect::<Vec<_>>();

        let gates_type = compboard
            .gates_type
            .chunks_exact(3)
            .map(|x| [x[0].0, x[1].0, x[2].0])
            .collect::<Vec<_>>();

        Ok(Self {
            gold_balls: gold_ball_indices.try_into().map_err(|_x| ())?,
            silver_balls: silver_ball_indices.try_into().map_err(|_x| ())?,
            gates_horizontal: gates_horizontal.try_into().map_err(|_x| ())?,
            gates_topleft: gates_topleft.try_into().map_err(|_x| ())?,
            gates_silver: gates_silver.try_into().map_err(|_x| ())?,
            gate_type: gates_type.try_into().map_err(|_x| ())?,
        })
    }
}
