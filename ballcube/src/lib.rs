mod state;
pub struct Board {
    gold_balls: [u8; 4],
    silver_balls: [u8; 4],
    gates_horizontal: [bool; 4],
    gates_topleft: [[bool; 3]; 4],
    gates_silver: [[bool; 3]; 4],
    gate_type: [[u8; 3]; 4],
}

impl Board {
    pub fn layer_horizontal(&self, layer_index: u8) -> bool {
        self.gates_horizontal[layer_index as usize]
    }
    pub fn topleft(&self, layer_index: u8, gate_index: u8) -> bool {
        self.gates_topleft[layer_index as usize][gate_index as usize]
    }
    pub fn gatetype(&self, layer_index: u8, gate_index: u8) -> u8 {
        self.gate_type[layer_index as usize][gate_index as usize]
    }
}
