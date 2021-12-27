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

fn transpose_gates(gates: u64) -> u64 {
    // http://programming.sirrida.de/calcperm.php
    (gates & 0x00000111)
        | ((gates & 0x00000022) << 2)
        | ((gates & 0x00000004) << 4)
        | ((gates & 0x00000040) >> 4)
        | ((gates & 0x00000088) >> 2)
}

fn flip_row(row: u64) -> u64 {
    row & 0b10 | (row << 2) & 0b100 | (row >> 2) & 0b1
}
impl CompactState {
    fn build_from_board(board: &Board) -> Self {
        fn build_layer(board: &Board, layer: u8) -> u64 {
            let mut layer_bits = 0;
            for gate in 0..3 {
                let mut gatebits = 1 << board.gatetype(layer, gate) & 0b111;
                if !board.topleft(layer, gate) {
                    gatebits = flip_row(gatebits)
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

        let mut result = Self { balls, gates };
        result.drop_balls();
        result
    }

    fn shift_gate_raw(&mut self, board: &Board, layer: u8, gate: u8) {
        fn handle_layer(mut layer_gates: u64, gate: u8, topleft: bool, horizontal: bool) -> u64 {
            fn handle_row(mut gate_bits: u64, topleft: bool) -> u64 {
                if !topleft {
                    gate_bits = flip_row(gate_bits);
                }
                gate_bits = (gate_bits >> 1) | 0b100;
                if !topleft {
                    gate_bits = flip_row(gate_bits);
                }
                gate_bits
            }
            if !horizontal {
                layer_gates = transpose_gates(layer_gates);
            };
            let gate_bitmask = 0b111 << (gate * 3);
            let gate_bits = (layer_gates >> (gate * 3)) & 0b111;

            layer_gates =
                (layer_gates & !gate_bitmask) | handle_row(gate_bits, topleft) << (gate * 3);
            if !horizontal {
                layer_gates = transpose_gates(layer_gates)
            };

            layer_gates
        }
        let h = board.layer_horizontal(layer);
        let t = board.topleft(layer, gate);
        let layer_gates = (self.gates >> (layer * 9)) & 0b1_1111_1111_u64;

        self.gates = (self.gates & !(0b1_1111_1111_u64 << (layer * 9)))
            | (handle_layer(layer_gates, gate, t, h) << (layer * 9));
    }

    fn shift_gate(&mut self, board: &Board, layer: u8, gate: u8) {
        self.shift_gate_raw(board, layer, gate);
        self.drop_balls();
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
            self.balls ^= dropped_balls | (dropped_balls << 9);
        }
    }
}

fn visualize_state(board: &Board, state: &CompactState) {
    let mut result = "".to_owned();
    let mut first_row = " ".to_owned();
    let mut last_row = " ".to_owned();
    for layer in 0..4 {
        if board.layer_horizontal(layer) {
            first_row += "___";
            last_row += "‾‾‾";
        } else {
            for gate in 0..3 {
                let s = board.gates_silver[layer as usize][gate as usize];
                let char = if s { "S" } else { "G" };

                let t = board.topleft(layer, gate);
                let (fc, lc) = if t { (char, "↑") } else { ("↓", char) };
                first_row += fc;
                last_row += lc;
            }
        }

        first_row += "   ";
        last_row += "   ";
    }
    for row in 0..3 {
        for layer in 0..4 {
            let row_bits = (state.gates >> (layer * 9 + row * 3)) & 0b111;
            let ball_bits = (state.balls >> (layer * 9 + row * 3)) & 0b111;
            let mut row_str = "".to_owned();

            let (first_char, last_char) = if board.layer_horizontal(layer) {
                let gate_silver = board.gates_silver[layer as usize][row as usize];
                let char = if gate_silver { "S" } else { "G" };

                if board.topleft(layer, row) {
                    (char, "←")
                } else {
                    ("→", char)
                }
            } else {
                ("|", "|")
            };
            for row_bit in 0..3 {
                let cell_blocked = (row_bits >> row_bit) & 1 == 0;
                let ball_present = (ball_bits >> row_bit) & 1 == 1;
                row_str += match (cell_blocked, ball_present) {
                    (true, true) => "B",
                    (true, false) => "X",
                    (false, true) => "F",
                    (false, false) => "O",
                }
            }
            result = format!("{result}{first_char}{row_str}{last_char} ");
        }
        result += "\n"
    }
    println!("{}\n{}{}\n", first_row, result, last_row);
}

#[cfg(test)]
mod test {
    use crate::ballcube::visualize_state;

    use super::{Board, CompactState};

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
    fn test_generation() {
        let b = generate_test_board();
        let mut s = CompactState::build_from_board(&b);

        assert_eq!(s.balls, 0b0_1111_1111_u64);
        assert_eq!(s.gates & 0b1_1111_1111_u64, 0);
        visualize_state(&b, &s);
        s.shift_gate_raw(&b, 0, 0);
        assert_eq!(s.gates & 0b1_1111_1111_u64, 0b0_0000_0100);
        visualize_state(&b, &s);
        s.drop_balls();
        visualize_state(&b, &s);
        assert_eq!(s.balls, 0b1000_1111_1011_u64, "{:b}", s.balls);

        s.shift_gate(&b, 1, 2);
        visualize_state(&b, &s);

        s.shift_gate(&b, 2, 0);
        visualize_state(&b, &s);

        s.shift_gate(&b, 3, 2);
        s.shift_gate(&b, 3, 2);
        visualize_state(&b, &s);
    }
}
