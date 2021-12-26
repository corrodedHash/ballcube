struct Board {
    gold_balls: [u8; 4],
    silver_balls: [u8; 4],
    gates_horizontal: [bool; 4],
    gates_topleft: [[bool; 3]; 4],
    gates_silver: [[bool; 3]; 4],
    gate_type: [[u8; 3]; 4],
}

impl Board {
    fn layer_horizontal(&self, layer_index: u8) -> bool {
        self.gates_horizontal[layer_index as usize]
    }
    fn topleft(&self, layer_index: u8, gate_index: u8) -> bool {
        self.gates_topleft[layer_index as usize][gate_index as usize]
    }
    fn gatetype(&self, layer_index: u8, gate_index: u8) -> u8 {
        self.gate_type[layer_index as usize][gate_index as usize]
    }
}

struct CompactState {
    balls: u64,
    gates: u64,
}

impl CompactState {
    fn build_from_board(board: &Board) -> Self {
        let balls = 0b111111111u64;
        let mut gates = 0u64;
        for layer in 0..4 {
            let h = board.layer_horizontal(layer);
            for gate in 0..3 {
                let gatebits = if board.topleft(layer, gate) {
                    1 << board.gatetype(layer, gate) & 0b111
                } else if board.gatetype(layer, gate) < 3 {
                    1 << (2 - board.gatetype(layer, gate))
                } else {
                    0
                };
                let gatebits = if !h {
                    gatebits & 1 | ((gatebits & 0b10) << 3) | ((gatebits & 0b100) << 6)
                } else {
                    gatebits
                };
                let in_layer_offset = if h { gate * 3 } else { gate };
                gates |= gatebits << ((gates * 9) + in_layer_offset as u64)
            }
        }
        let mut result = Self { balls, gates };
        result.drop_balls();
        result
    }

    fn shift_gate(&mut self, board: &Board, layer: u8, gate: u8) {
        let h = board.layer_horizontal(layer);
        let t = board.topleft(layer, gate);
        let layer = (self.gates >> (layer * 9)) & 0b1_1111_1111_u64;
        let gate_bitmask = if h {
            0b111 << (gate * 3)
        } else {
            0b1001001 << gate
        };

        let g = layer & gate_bitmask;

        let new_layer = match (t, h) {
            (true, true) => {
                (layer & !gate_bitmask) | ((layer >> 1) & gate_bitmask) | (0b100 << (gate * 3))
            }
            (true, false) => {
                (layer & !gate_bitmask) | ((layer >> 3) & gate_bitmask) | (0b100_0000 << gate)
            }
            (false, true) => {
                (layer & !gate_bitmask) | ((layer << 1) & gate_bitmask) | (0b1 << (gate * 3))
            }
            (false, false) => {
                (layer & !gate_bitmask) | ((layer << 3) & gate_bitmask) | (0b1 << (gate))
            }
        };
    }

    fn depth(&self) -> [u8; 9] {
        let mut ballmask = 1u64;
        for _ in 0..4 {
            ballmask <<= 9;
            ballmask |= 1;
        }
        let mut result = [0u8; 9];

        for (i, r) in result.iter_mut().enumerate() {
            let found_ball = self.balls & (ballmask << i);
            debug_assert!(found_ball.count_ones() <= 1);
            *r = std::cmp::min(found_ball.trailing_zeros() as u8 / 9, 4);
        }

        result
    }

    fn drop_balls(&mut self) {
        while self.balls & self.gates != 0 {
            let dropped_balls = self.balls & self.gates;
            debug_assert_eq!(self.balls & dropped_balls, dropped_balls);
            debug_assert_eq!(self.balls & (dropped_balls << 9), 0);
            self.balls ^= dropped_balls;
            self.balls ^= dropped_balls << 9;
        }
    }
}
