use crate::Player;

pub mod builder;

use rand::Rng;

use self::builder::Gate;

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
            debug_assert!(i.leading_zeros() as usize >= (u64::MAX.count_ones() as usize - stride));
            self.result |= i << self.count;
            self.count += stride;
        }
    }
}

impl From<&Board> for u64 {
    fn from(b: &Board) -> Self {
        let mut bp = BitPacker::default();

        let btu = |x: &bool| if *x { 1_u64 } else { 0 };

        let mut empty_cell_iterator = (0..9)
            .filter(|x| !b.gold_balls.contains(x))
            .filter(|x| !b.silver_balls.contains(x));
        let empty_cell = empty_cell_iterator.next().unwrap();
        assert!(empty_cell_iterator.next().is_none());
        let empty_cell_delta = b.gold_balls.iter().filter(|x| x < &&empty_cell).count() as u8;

        // 9 bit
        bp.pack((0..9).map(|x| btu(&b.gold_balls.contains(&x))), 1);
        // 3 bit
        bp.pack(std::iter::once(u64::from(empty_cell - empty_cell_delta)), 3);
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
                .map(u64::from),
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
    pub fn layer_horizontal(&self, layer_index: u8) -> bool {
        self.gates_horizontal[layer_index as usize]
    }

    #[must_use]
    pub fn gate(&self, layer: u8, gate: u8) -> Player {
        if self.gates_silver[layer as usize][gate as usize] {
            Player::Silver
        } else {
            Player::Gold
        }
    }

    #[must_use]
    pub fn topleft(&self, layer_index: u8, gate_index: u8) -> bool {
        self.gates_topleft[layer_index as usize][gate_index as usize]
    }

    #[must_use]
    pub fn gatetype(&self, layer_index: u8, gate_index: u8) -> u8 {
        self.gate_type[layer_index as usize][gate_index as usize]
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

        let gates = gate_distribution
            .into_iter()
            .map(|silver| {
                let t = if silver {
                    silver_gates.pop()
                } else {
                    gold_gates.pop()
                }
                .unwrap();

                Some(Gate {
                    allegiance: if silver { Player::Silver } else { Player::Gold },
                    gatetype: t,
                    topleft: rand::thread_rng().gen::<bool>(),
                })
            })
            .collect::<Vec<_>>();
        let gates: [_; 12] = gates.try_into().unwrap();
        let mut gates_horizontal = [true; 4];
        rand::thread_rng().fill(&mut gates_horizontal);
        let gates_horizontal: [Option<bool>; 4] = gates_horizontal
            .into_iter()
            .map(Some)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        crate::BoardBuilder {
            gold_balls,
            silver_balls,
            gates_horizontal,
            gates,
        }
        .finalize()
        .unwrap()
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn board_serialize() {
        for _ in 0..100 {
            let board = crate::Board::random();
            let serialized = u64::from(&board);
            let deserialized_board = crate::Board::try_from(serialized).unwrap();

            assert_eq!(board, deserialized_board);
        }
    }
}
