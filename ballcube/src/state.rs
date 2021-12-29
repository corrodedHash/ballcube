use crate::Board;

pub struct CompactState {
    balls: u64,
    gates: u64,
    gate_shifts: u64,
}

fn transpose_gates(gates: u64) -> u64 {
    // http://programming.sirrida.de/calcperm.php
    // 0 3 6 1 4 7 2 5 8 9 12 15 10 13 16 11 14 17 18 21 24 19 22 25 20 23 26 27 30 33 28 31 34 29 32 35 36 37 38 39 40 41 42 43 44 45 46 47 48 49 50 51 52 53 54 55 56 57 58 59 60 61 62 63
    (gates & 0xfffffff88c462311)
        | ((gates & 0x0000000110884422) << 2)
        | ((gates & 0x0000000020100804) << 4)
        | ((gates & 0x0000000201008040) >> 4)
        | ((gates & 0x0000000442211088) >> 2)
}

fn mirror_gates(gates: u64) -> u64 {
    // http://programming.sirrida.de/calcperm.php
    // 2 1 0 5 4 3 8 7 6 11 10 9 14 13 12 17 16 15 20 19 18 23 22 21 26 25 24 29 28 27 32 31 30 35 34 33 36 37 38 39 40 41 42 43 44 45 46 47 48 49 50 51 52 53 54 55 56 57 58 59 60 61 62 63
    (gates & 0xfffffff492492492)
        | ((gates & 0x0000000249249249) << 2)
        | ((gates & 0x0000000924924924) >> 2)
}

impl CompactState {
    pub fn build_from_board(board: &Board) -> Self {
        fn build_layer(board: &Board, layer: u8) -> u64 {
            let mut layer_bits = 0;
            for gate in 0..3 {
                let mut gatebits = 1 << board.gatetype(layer, gate) & 0b111;
                if !board.topleft(layer, gate) {
                    gatebits = mirror_gates(gatebits)
                };
                layer_bits |= gatebits << (gate * 3)
            }
            if !board.layer_horizontal(layer) {
                layer_bits = transpose_gates(layer_bits);
            }
            layer_bits
        }
        let mut balls = 0u64;
        for ball in board.gold_balls.iter().chain(board.silver_balls.iter()) {
            balls |= 1 << ball;
        }

        let mut gates = 0u64;
        for layer in 0..4 {
            gates |= build_layer(board, layer) << (layer * 9);
        }

        let mut result = Self {
            balls,
            gates,
            gate_shifts: 0,
        };
        result.drop_balls();
        result
    }

    fn increment_gate_shift(&mut self, layer: u8, gate: u8) {
        let gate_shift_bit_index = (layer * 3 + gate) * 2;
        debug_assert!(self.get_shift(layer, gate) < 3);

        self.gate_shifts += 1 << gate_shift_bit_index;

        debug_assert!(self.gate_shifts < (1 << 26));
    }

    pub fn shift_gate_raw(&mut self, board: &Board, layer: u8, gate: u8) {
        self.increment_gate_shift(layer, gate);

        let h = board.layer_horizontal(layer);
        let t = board.topleft(layer, gate);

        let mut gates = self.gates;
        gates = if !h { transpose_gates(gates) } else { gates };
        gates = if !t { mirror_gates(gates) } else { gates };

        let gate_offset = layer * 9 + gate * 3;
        let gate_mask = 0b111 << gate_offset;
        gates = (gates & !gate_mask) | ((gates >> 1) & gate_mask) | (0b100 << gate_offset);

        gates = if !t { mirror_gates(gates) } else { gates };
        gates = if !h { transpose_gates(gates) } else { gates };

        self.gates = gates;
    }

    pub fn shift_gate(&mut self, board: &Board, layer: u8, gate: u8) {
        self.shift_gate_raw(board, layer, gate);
        self.drop_balls();
    }

    pub fn depth(&self) -> [u8; 9] {
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

    pub fn get_shift(&self, layer: u8, gate: u8) -> u8 {
        ((self.gate_shifts >> ((layer * 3 + gate) * 2)) & 0b11) as u8
    }

    pub fn shift_count(&self) -> u8 {
        let it_one = (self.gate_shifts & 0x0033_3333) + ((self.gate_shifts & 0x00CC_CCCC) >> 2);
        let it_two = (it_one & 0x000F_0F0F) + ((it_one & 0x00F0_F0F0) >> 4);
        let it_three = (it_two & 0x00FF_00FF) + ((it_two & 0x0000_FF00) >> 8);
        let it_four = (it_three & 0xFFFF) + (it_three >> 16);
        debug_assert!(it_four <= 255, "{:b}", it_four);
        it_four as u8
    }

    pub fn get_gate_bits(&self) -> u64 {
        self.gates
    }

    pub fn drop_balls(&mut self) {
        while self.balls & self.gates != 0 {
            let dropped_balls = self.balls & self.gates;
            debug_assert_eq!(self.balls & dropped_balls, dropped_balls);
            debug_assert_eq!(self.balls & (dropped_balls << 9), 0);
            self.balls ^= dropped_balls | (dropped_balls << 9);
        }
    }
}

#[cfg(test)]
mod test {
    use super::CompactState;
    use crate::visualize_state::visualize_state;
    use crate::Board;

    fn generate_test_board() -> Board {
        Board {
            gold_balls: [0, 1, 2, 3],
            silver_balls: [4, 5, 6, 7],
            gates_horizontal: [true, false, true, false],
            gates_topleft: [
                [true, true, false],
                [false, true, true],
                [true, false, false],
                [false, false, true],
            ],
            gates_silver: [
                [true, false, false],
                [false, false, true],
                [true, true, false],
                [false, true, true],
            ],
            gate_type: [[3, 3, 3], [0, 0, 1], [0, 1, 0], [3, 2, 2]],
        }
    }

    #[test]
    fn test_shifting() {
        let b = generate_test_board();
        let mut s = CompactState::build_from_board(&b);

        assert_eq!(s.depth(), [0, 0, 0, 0, 0, 0, 0, 0, 4]);
        assert_eq!(s.gates & 0b1_1111_1111_u64, 0);
        assert_eq!(s.shift_count(), 0);
        visualize_state(&b, &s);
        s.shift_gate_raw(&b, 0, 0);
        assert_eq!(s.get_shift(0, 0), 1);
        assert_eq!(s.shift_count(), 1);

        assert_eq!(s.gates & 0b1_1111_1111_u64, 0b0_0000_0100);
        visualize_state(&b, &s);
        s.drop_balls();
        visualize_state(&b, &s);
        assert_eq!(s.depth(), [0, 0, 1, 0, 0, 0, 0, 0, 4]);

        s.shift_gate(&b, 1, 2);
        assert_eq!(s.get_shift(1, 2), 1);
        assert_eq!(s.shift_count(), 2);

        assert_eq!(s.depth(), [0, 0, 2, 0, 0, 0, 0, 0, 4]);
        visualize_state(&b, &s);

        s.shift_gate(&b, 2, 0);
        assert_eq!(s.get_shift(2, 0), 1);
        assert_eq!(s.shift_count(), 3);

        assert_eq!(s.depth(), [0, 0, 3, 0, 0, 0, 0, 0, 4]);
        visualize_state(&b, &s);

        s.shift_gate(&b, 3, 2);
        assert_eq!(s.get_shift(3, 2), 1);
        assert_eq!(s.shift_count(), 4);

        s.shift_gate(&b, 3, 2);
        assert_eq!(s.get_shift(3, 2), 2);
        assert_eq!(s.shift_count(), 5);

        assert_eq!(s.depth(), [0, 0, 4, 0, 0, 0, 0, 0, 4]);
        visualize_state(&b, &s);
    }
}
