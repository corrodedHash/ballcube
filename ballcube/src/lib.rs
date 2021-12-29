mod state;
mod visualize_state;

pub use state::CompactState;
pub use visualize_state::visualize_state;

pub struct Board {
    gold_balls: [u8; 4],
    silver_balls: [u8; 4],
    gates_horizontal: [bool; 4],
    gates_topleft: [[bool; 3]; 4],
    gates_silver: [[bool; 3]; 4],
    gate_type: [[u8; 3]; 4],
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Player {
    Gold,
    Silver,
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
