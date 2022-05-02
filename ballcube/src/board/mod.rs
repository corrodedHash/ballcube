use crate::Player;

pub mod builder;

use rand::Rng;

use self::builder::Gate;

use deku::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Board {
    gold_balls: [u8; 4],
    silver_balls: [u8; 4],
    gates_horizontal: [bool; 4],
    gates_topleft: [[bool; 3]; 4],
    gates_silver: [[bool; 3]; 4],
    gate_type: [[u8; 3]; 4],
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct BitPacker {
    result: u64,
    count: usize,
}

impl BitPacker {
    fn pack<I>(&mut self, vals: I, stride: usize)
    where
        I: Iterator<Item = u64>,
    {
        for i in vals {
            debug_assert!(i.leading_zeros() as usize >= (u64::BITS as usize - stride));
            self.result |= i << self.count;
            self.count += stride;
        }
    }
}

#[derive(Debug, Clone, DekuRead, DekuWrite)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
struct TwoWideInt(#[deku(bits = 2)] u8);

#[derive(Debug, Clone, DekuRead, DekuWrite)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
struct OneWideBool(#[deku(bits = 1)] bool);

#[derive(Debug, Clone, DekuRead, DekuWrite)]
#[deku(endian = "little")]
struct CompressedBoard {
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

#[allow(clippy::fallible_impl_from)]
impl From<&Board> for u64 {
    fn from(b: &Board) -> Self {
        let mut bp = BitPacker::default();

        let btu = |x: &bool| if *x { 1_u64 } else { 0 };

        let empty_cell_iterator =
            (0..9).find(|x| !b.gold_balls.contains(x) && !b.silver_balls.contains(x));
        let empty_cell = empty_cell_iterator.unwrap();
        let empty_cell_delta = b.gold_balls.iter().filter(|x| x < &&empty_cell).count() as u8;

        // 9 bit
        bp.pack((0..9).map(|x| btu(&b.gold_balls.contains(&x))), 1);
        // 3 bit
        bp.pack(
            std::iter::once(Self::from(empty_cell - empty_cell_delta)),
            3,
        );
        // 4 bit
        bp.pack(b.gates_horizontal.iter().map(btu), 1);
        // 12 bit
        bp.pack(b.gates_topleft.iter().flat_map(|x| x.iter()).map(btu), 1);
        // 12 bit
        bp.pack(b.gates_silver.iter().flat_map(|x| x.iter()).map(btu), 1);
        // 24 bit
        bp.pack(
            b.gate_type
                .iter()
                .flat_map(|x| x.iter().copied())
                .map(Self::from),
            2,
        );

        bp.result
    }
}

impl TryFrom<u64> for Board {
    type Error = builder::BoardBuildingError;

    fn try_from(mut value: u64) -> Result<Self, Self::Error> {
        let mut bb = builder::BoardBuilder::default();

        for i in 0..9 {
            if (value & 1) == 1 {
                bb.gold_balls.push(i);
            }
            value >>= 1;
        }

        let mut empty_cell_compressed = (value & 0b111) as u8;
        value >>= 3;

        for gb in &bb.gold_balls {
            if gb <= &empty_cell_compressed {
                empty_cell_compressed += 1;
            } else {
                break;
            }
        }

        let empty_cell = empty_cell_compressed as u8;
        let silver_balls = (0..9).filter(|x| !bb.gold_balls.contains(x) && x != &empty_cell);
        bb.silver_balls.extend(silver_balls);

        for i in &mut bb.gates_horizontal {
            *i = Some((value & 1) == 1);
            value >>= 1;
        }

        let (mut topleft, mut sg, mut ty) = (vec![], vec![], vec![]);

        for _ in 0..12 {
            topleft.push((value & 1) == 1);
            value >>= 1;
        }
        for _ in 0..12 {
            sg.push((value & 1) == 1);
            value >>= 1;
        }
        for _ in 0..12 {
            ty.push(value & 0b11);
            value >>= 2;
        }

        for ((a, (b, c)), g) in topleft
            .into_iter()
            .zip(sg.into_iter().zip(ty.into_iter()))
            .zip(bb.gates.iter_mut())
        {
            *g = Some(builder::Gate {
                allegiance: if b { Player::Silver } else { Player::Gold },
                topleft: a,
                gatetype: c as u8,
            });
        }
        bb.finalize()
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
    use deku::DekuContainerWrite;

    #[test]
    #[allow(clippy::expect_used)]
    fn board_serialize() {
        for _ in 0..1 {
            let board = crate::Board::random();
            let serialized = u64::from(&board);
            let x = crate::board::CompressedBoard::from(&board);
            dbg!(board.gold_balls);
            dbg!(x.to_bytes());
            dbg!(serialized.to_be_bytes());
            let deserialized_board =
                crate::Board::try_from(serialized).expect("Could not deserialize board");

            assert_eq!(board, deserialized_board);
        }
    }
}
