mod state;
mod visualize_state;

pub use state::CompactState;
pub use visualize_state::visualize_state;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Player {
    Gold,
    Silver,
}

#[derive(Default, Clone, Debug)]
pub struct BoardBuilder {
    pub gold_balls: Vec<u8>,
    pub silver_balls: Vec<u8>,
    pub gates_horizontal: [Option<bool>; 4],
    pub gates: [Option<(bool, bool, u8)>; 12],
}

impl BoardBuilder {
    fn finalize(mut self) -> Option<Board> {
        if self.gates_horizontal.iter().any(|x| x.is_none()) {
            return None;
        }
        let gates_horizontal = self
            .gates_horizontal
            .into_iter()
            .map(|x| x.unwrap())
            .collect::<Vec<bool>>()
            .try_into()
            .unwrap();

        let (mut gates_topleft_v, mut gates_silver_v, mut gate_type_v) = (vec![], vec![], vec![]);

        for x in self.gates {
            if let Some((t, s, ty)) = x {
                gates_topleft_v.push(t);
                gates_silver_v.push(s);
                gate_type_v.push(ty);
            } else {
                return None;
            }
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

        Some(Board {
            gold_balls: self.gold_balls.try_into().unwrap(),
            silver_balls: self.silver_balls.try_into().unwrap(),
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
