use crate::Player;
use deku::{DekuContainerRead, DekuContainerWrite};
pub mod builder;
mod compressed;
use compressed::CompressedBoard;

use rand::Rng;

use self::{builder::Gate, compressed::IncorrectCompBoardError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Board {
    gold_balls: [u8; 4],
    silver_balls: [u8; 4],
    gates_horizontal: [bool; 4],
    gates_topleft: [[bool; 3]; 4],
    gates_silver: [[bool; 3]; 4],
    gate_type: [[u8; 3]; 4],
}

#[allow(clippy::fallible_impl_from)]
impl From<&Board> for u64 {
    fn from(board: &Board) -> Self {
        let compressed = CompressedBoard::from(board);
        let bytes = compressed.to_bytes().unwrap();
        debug_assert_eq!(bytes.len(), 8);
        // bytes.resize(8, 0);
        Self::from_le_bytes(bytes.try_into().unwrap())
    }
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum DeserializationError {
    #[error("Could not read bitstring")]
    IncorrectBitstring,
    #[error("Deserialized board incorrect: {0}")]
    IncorrectBoard(#[from] IncorrectCompBoardError),
}

impl TryFrom<u64> for Board {
    type Error = DeserializationError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        let bytes = value.to_le_bytes();
        let (_, compressed) = CompressedBoard::from_bytes((&bytes, 0))
            .map_err(|_x| DeserializationError::IncorrectBitstring)?;
        Ok(Self::try_from(compressed)?)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LayerProxy<'board> {
    layer_id: u8,
    board: &'board Board,
}

impl<'board> LayerProxy<'board> {
    #[must_use]
    pub const fn horizontal(&self) -> bool {
        self.board.gates_horizontal[self.layer_id as usize]
    }

    #[must_use]
    pub const fn gate(&self, gate_id: u8) -> GateProxy<'board> {
        GateProxy {
            gate_id,
            layer: *self,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GateProxy<'layer> {
    gate_id: u8,
    layer: LayerProxy<'layer>,
}

impl<'layer> GateProxy<'layer> {
    #[must_use]
    pub const fn owner(&self) -> Player {
        if self.layer.board.gates_silver[self.layer.layer_id as usize][self.gate_id as usize] {
            Player::Silver
        } else {
            Player::Gold
        }
    }

    #[must_use]
    pub const fn topleft(&self) -> bool {
        self.layer.board.gates_topleft[self.layer.layer_id as usize][self.gate_id as usize]
    }

    #[must_use]
    pub const fn gatetype(&self) -> u8 {
        self.layer.board.gate_type[self.layer.layer_id as usize][self.gate_id as usize]
    }
}

impl Board {
    #[must_use]
    pub fn ball(&self, cell_index: u8) -> Option<Player> {
        let finder = move |x: &u8| *x == cell_index;
        if self.gold_balls.iter().any(finder) {
            Some(Player::Gold)
        } else if self.silver_balls.iter().any(finder) {
            Some(Player::Silver)
        } else {
            None
        }
    }

    #[must_use]
    pub const fn layer(&self, layer_id: u8) -> LayerProxy<'_> {
        LayerProxy {
            layer_id,
            board: self,
        }
    }

    /// # Panics
    /// Never
    #[must_use]
    pub fn random() -> Self {
        use rand::seq::SliceRandom;

        let mut balls = (0_u8..9).collect::<Vec<_>>();
        balls.shuffle(&mut rand::thread_rng());

        let gold_balls = balls[0..4].to_vec();
        let silver_balls = balls[4..8].to_vec();
        let gate_types = [0_u8, 0, 1, 2, 3, 3];
        let mut gold_gates = gate_types.to_vec();
        let mut silver_gates = gate_types.to_vec();
        let mut gate_distribution = vec![false; 6];
        gate_distribution.extend(vec![true; 6]);
        gold_gates.shuffle(&mut rand::thread_rng());
        silver_gates.shuffle(&mut rand::thread_rng());
        gate_distribution.shuffle(&mut rand::thread_rng());

        let gates_vec = gate_distribution
            .into_iter()
            .map(|silver| {
                let relevant_gate = if silver {
                    &mut silver_gates
                } else {
                    &mut gold_gates
                };
                let t = relevant_gate.pop().unwrap();

                Some(Gate {
                    allegiance: if silver { Player::Silver } else { Player::Gold },
                    gatetype: t,
                    topleft: rand::thread_rng().gen::<bool>(),
                })
            })
            .collect::<Vec<_>>();
        let gates: [_; 12] = gates_vec.try_into().unwrap();
        let mut gates_horizontal = [true; 4];
        rand::thread_rng().fill(&mut gates_horizontal);
        let gates_horizontal_option: [Option<bool>; 4] = gates_horizontal.map(Some);
        crate::BoardBuilder {
            gold_balls,
            silver_balls,
            gates_horizontal: gates_horizontal_option,
            gates,
        }
        .finalize()
        .unwrap()
    }
}

#[cfg(test)]
mod test {
    #[test]
    #[allow(clippy::expect_used)]
    fn board_serialize() {
        for _ in 0..100 {
            let board = crate::Board::random();
            let serialized = u64::from(&board);
            let deserialized_board =
                crate::Board::try_from(serialized).expect("Could not deserialize board");

            assert_eq!(board, deserialized_board);
        }
    }
}
