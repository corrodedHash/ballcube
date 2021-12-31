mod state;
mod visualize_state;

pub use state::CompactState;
pub use visualize_state::visualize_state;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Player {
    Gold,
    Silver,
}

impl Player {
    #[must_use]
    pub fn other(self) -> Self {
        match self {
            Player::Gold => Player::Silver,
            Player::Silver => Player::Gold,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Gate {
    pub allegiance: Player,
    pub topleft: bool,
    pub gatetype: u8,
}

#[derive(Default, Clone, Debug)]
pub struct BoardBuilder {
    pub gold_balls: Vec<u8>,
    pub silver_balls: Vec<u8>,
    pub gates_horizontal: [Option<bool>; 4],
    pub gates: [Option<Gate>; 12],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoardBuildingError {
    GateDirectionUndefined(u8),
    GateUndefined(u8),
    BallUndefined,
    BallCountIncorrect,
    GateAllegianceIncorrect,
}

impl BoardBuilder {
    pub fn finalize(self) -> Result<Board, BoardBuildingError> {
        let mut gates_horizontal = [false; 4];
        for (id, (g, r)) in (0u8..).zip(
            self.gates_horizontal
                .iter()
                .zip(gates_horizontal.iter_mut()),
        ) {
            *r = (*g).ok_or(BoardBuildingError::GateDirectionUndefined(id))?;
        }

        let (mut gates_topleft_v, mut gates_silver_v, mut gate_type_v) = (vec![], vec![], vec![]);

        for (id, x) in (0u8..).zip(self.gates.iter()) {
            if let Some(g) = x {
                gates_topleft_v.push(g.topleft);
                gates_silver_v.push(g.allegiance == Player::Silver);
                gate_type_v.push(g.gatetype);
            } else {
                return Err(BoardBuildingError::GateUndefined(id));
            }
        }

        if gates_silver_v.iter().filter(|x| x == &&true).count() != 6 {
            return Err(BoardBuildingError::GateAllegianceIncorrect);
        }

        let (mut gates_topleft, mut gates_silver, mut gate_type) =
            ([[false; 3]; 4], [[false; 3]; 4], [[0u8; 3]; 4]);

        for (id, (t, (s, ty))) in gates_topleft_v
            .into_iter()
            .zip(gates_silver_v.into_iter().zip(gate_type_v.into_iter()))
            .enumerate()
        {
            gates_topleft[id / 3][id % 3] = t;
            gates_silver[id / 3][id % 3] = s;
            gate_type[id / 3][id % 3] = ty;
        }

        let gold_balls = self
            .gold_balls
            .try_into()
            .map_err(|_| BoardBuildingError::BallCountIncorrect)?;

        let silver_balls = self
            .silver_balls
            .try_into()
            .map_err(|_| BoardBuildingError::BallCountIncorrect)?;

        Ok(Board {
            gold_balls,
            silver_balls,
            gates_horizontal,
            gates_topleft,
            gates_silver,
            gate_type,
        })
    }
}

#[derive(Clone, Debug)]
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
    result: u128,
    count: usize,
}

impl BitPacker {
    fn pack<I>(&mut self, vals: I, stride: usize)
    where
        I: Iterator<Item = u128>,
    {
        for i in vals {
            debug_assert!(i.leading_zeros() as usize >= (128usize - stride));
            self.result |= i << self.count;
            self.count += stride;
        }
    }
}

impl From<&Board> for u128 {
    fn from(b: &Board) -> Self {
        let mut bp = BitPacker::default();

        let btu = |x: &bool| match *x {
            true => 1u128,
            false => 0,
        };

        bp.pack((0..9).map(|x| btu(&b.gold_balls.contains(&x))), 1);
        bp.pack((0..9).map(|x| btu(&b.silver_balls.contains(&x))), 1);
        bp.pack(b.gates_horizontal.iter().map(btu), 1);
        bp.pack(b.gates_topleft.iter().flat_map(|x| x.iter()).map(btu), 1);
        bp.pack(b.gates_silver.iter().flat_map(|x| x.iter()).map(btu), 1);
        bp.pack(
            b.gate_type
                .iter()
                .flat_map(|x| x.iter().copied())
                .map(u128::from),
            2,
        );

        bp.result
    }
}

impl TryFrom<u128> for Board {
    type Error = BoardBuildingError;

    fn try_from(mut value: u128) -> Result<Self, Self::Error> {
        let mut bb = BoardBuilder::default();

        for i in 0..9 {
            if (value & 1) == 1 {
                bb.gold_balls.push(i);
            }
            value >>= 1;
        }

        for i in 0..9 {
            if (value & 1) == 1 {
                bb.silver_balls.push(i);
            }
            value >>= 1;
        }

        for i in bb.gates_horizontal.iter_mut() {
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
            *g = Some(Gate {
                allegiance: if b { Player::Silver } else { Player::Gold },
                topleft: a,
                gatetype: c as u8,
            });
        }
        bb.finalize()
    }
}

impl Board {
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

    pub fn layer_horizontal(&self, layer_index: u8) -> bool {
        self.gates_horizontal[layer_index as usize]
    }

    pub fn gate(&self, layer: u8, gate: u8) -> Player {
        if self.gates_silver[layer as usize][gate as usize] {
            Player::Silver
        } else {
            Player::Gold
        }
    }

    pub fn topleft(&self, layer_index: u8, gate_index: u8) -> bool {
        self.gates_topleft[layer_index as usize][gate_index as usize]
    }

    pub fn gatetype(&self, layer_index: u8, gate_index: u8) -> u8 {
        self.gate_type[layer_index as usize][gate_index as usize]
    }
}
