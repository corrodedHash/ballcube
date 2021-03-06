use rand::seq::IteratorRandom;

use crate::{Board, Move, MoveChecker, Player, Winner, WinningChecker};
mod ball_bitmask;
use ball_bitmask::BallBitmask;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Compact representation of the position of the balls and the gates.
/// Complements a board which provides the placements of the gates and balls
pub struct Compact {
    balls: BallBitmask,
    gates: u64,
    gate_shifts: u64,
}

impl From<&Compact> for u64 {
    fn from(c: &Compact) -> Self {
        c.balls.compress() | (c.gate_shifts << BallBitmask::COMPRESSED_BITSIZE)
    }
}

const fn transpose_gates(gates: u64) -> u64 {
    // http://programming.sirrida.de/calcperm.php
    // 0 3 6 1 4 7 2 5 8 9 12 15 10 13 16 11 14 17 18 21 24 19 22 25 20 23 26 27 30 33 28 31 34 29 32 35 36 37 38 39 40 41 42 43 44 45 46 47 48 49 50 51 52 53 54 55 56 57 58 59 60 61 62 63
    (gates & 0xffff_fff8_8c46_2311)
        | ((gates & 0x0000_0001_1088_4422) << 2)
        | ((gates & 0x0000_0000_2010_0804) << 4)
        | ((gates & 0x0000_0002_0100_8040) >> 4)
        | ((gates & 0x0000_0004_4221_1088) >> 2)
}

const fn mirror_gates(gates: u64) -> u64 {
    // http://programming.sirrida.de/calcperm.php
    // 2 1 0 5 4 3 8 7 6 11 10 9 14 13 12 17 16 15 20 19 18 23 22 21 26 25 24 29 28 27 32 31 30 35 34 33 36 37 38 39 40 41 42 43 44 45 46 47 48 49 50 51 52 53 54 55 56 57 58 59 60 61 62 63
    (gates & 0xffff_fff4_9249_2492)
        | ((gates & 0x0000_0002_4924_9249) << 2)
        | ((gates & 0x0000_0009_2492_4924) >> 2)
}

fn two_bit_array_add(tba: u64) -> u8 {
    let it_one = (tba & 0x0033_3333) + ((tba & 0x00CC_CCCC) >> 2);
    let it_two = (it_one & 0x000F_0F0F) + ((it_one & 0x00F0_F0F0) >> 4);
    let it_three = (it_two & 0x00FF_00FF) + ((it_two & 0x0000_FF00) >> 8);
    let it_four = (it_three & 0xFFFF) + (it_three >> 16);
    debug_assert!(it_four <= 255, "{:b}", it_four);
    it_four as u8
}

impl Compact {
    #[must_use]
    pub fn from_u64(mut int: u64, board: &Board) -> Self {
        let mut result = Self::build_from_board(board);

        let ball_bits = int & ((1_u64 << BallBitmask::COMPRESSED_BITSIZE) - 1);
        int >>= BallBitmask::COMPRESSED_BITSIZE;
        result.balls = BallBitmask::decompress(ball_bits, board);

        let gate_shifts = int;
        for layer in 0..4 {
            for gate in 0..3 {
                for _ in 0..(int & 0b11) {
                    result.shift_gate_raw(board, layer, gate);
                }
                int >>= 2;
            }
        }
        debug_assert_eq!(result.gate_shifts, gate_shifts);
        result
    }

    #[must_use]
    pub fn random_game(mut self, board: &Board, starting_player: Player) -> Vec<(Self, Move)> {
        let move_generator = MoveChecker::new(board);
        let win_checker = WinningChecker::new(board);
        let mut result = vec![];
        while win_checker.won(&self) == Winner::None {
            let player = if result.len() % 2 == 0 {
                starting_player
            } else {
                starting_player.other()
            };
            let m = *move_generator
                .moves(&self, player)
                .iter()
                .choose(&mut rand::thread_rng())
                .expect("No moves left, but no one won yet");

            self.shift_gate(board, m.layer(), m.gate());
            result.push((self, m));
        }
        result
    }

    #[must_use]
    pub fn build_from_board(board: &Board) -> Self {
        fn build_layer(board: &Board, layer_id: u8) -> u64 {
            let mut layer_bits = 0;
            let layer = board.layer(layer_id);
            for gate_id in 0..3 {
                let gate = layer.gate(gate_id);
                let mut gatebits = (1 << gate.gatetype()) & 0b111;
                if !gate.topleft() {
                    gatebits = mirror_gates(gatebits);
                };
                layer_bits |= gatebits << (gate_id * 3);
            }
            if !layer.horizontal() {
                layer_bits = transpose_gates(layer_bits);
            }
            layer_bits
        }
        let mut balls = 0_u64;
        for ball in (0_u8..9).filter(|x| board.ball(*x).is_some()) {
            balls |= 1 << ball;
        }

        let mut gates = 0_u64;
        for layer in 0..4 {
            gates |= build_layer(board, layer) << (layer * 9);
        }

        let mut result = Self {
            balls: BallBitmask::new(balls),
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

        let h = board.layer(layer).horizontal();
        let t = board.layer(layer).gate(gate).topleft();

        let mut gates = self.gates;
        gates = if h { gates } else { transpose_gates(gates) };
        gates = if t { gates } else { mirror_gates(gates) };

        let gate_offset = layer * 9 + gate * 3;
        let gate_mask = 0b111 << gate_offset;
        gates = (gates & !gate_mask) | ((gates >> 1) & gate_mask) | (0b100 << gate_offset);

        gates = if t { gates } else { mirror_gates(gates) };
        gates = if h { gates } else { transpose_gates(gates) };

        self.gates = gates;
    }

    pub fn shift_gate(&mut self, board: &Board, layer: u8, gate: u8) {
        self.shift_gate_raw(board, layer, gate);
        self.drop_balls();
    }

    #[must_use]
    pub fn depth(&self) -> [u8; 9] {
        self.balls.depth()
    }

    #[must_use]
    pub const fn get_shift(&self, layer: u8, gate: u8) -> u8 {
        ((self.gate_shifts >> ((layer * 3 + gate) * 2)) & 0b11) as u8
    }

    /// Sum the number of times each gate has been shifted
    #[must_use]
    pub fn shift_count(&self) -> u8 {
        two_bit_array_add(self.gate_shifts)
    }

    /// Sum shifts on gates which belong to silver
    #[must_use]
    pub fn shift_count_silver(&self, board: &Board) -> u8 {
        let mut silver_mask = 0_u64;
        for i in 0..12 {
            if board.layer(i / 3).gate(i % 3).owner() == Player::Silver {
                silver_mask |= 0b11 << (i * 2);
            }
        }
        two_bit_array_add(self.gate_shifts & silver_mask)
    }

    #[must_use]
    pub const fn get_gate_bits(&self) -> u64 {
        self.gates
    }

    #[must_use]
    pub const fn get_ball_bits(&self) -> u64 {
        self.balls.get_mask()
    }

    pub fn drop_balls(&mut self) {
        self.balls.drop(self.gates);
    }
}

#[cfg(test)]
mod test {
    use super::Compact;
    use crate::board::builder::Gate;
    use crate::visualize_state::visualize_state;
    use crate::{Board, BoardBuilder, Player};

    #[allow(clippy::expect_used)]
    fn generate_test_board() -> Board {
        BoardBuilder {
            gold_balls: vec![0, 1, 2, 3],
            silver_balls: vec![4, 5, 6, 7],
            gates_horizontal: [true, false, true, false].map(Some),
            gates: [
                Gate::build().s().t().ty(3).finalize(),
                Gate::build().g().t().ty(3).finalize(),
                Gate::build().g().b().ty(3).finalize(),
                Gate::build().g().b().ty(0).finalize(),
                Gate::build().g().t().ty(0).finalize(),
                Gate::build().s().t().ty(1).finalize(),
                Gate::build().s().t().ty(0).finalize(),
                Gate::build().s().b().ty(1).finalize(),
                Gate::build().g().b().ty(0).finalize(),
                Gate::build().g().b().ty(3).finalize(),
                Gate::build().s().b().ty(2).finalize(),
                Gate::build().s().t().ty(2).finalize(),
            ],
        }
        .finalize()
        .expect("Could not generate board")
    }

    #[test]
    fn test_shifting() {
        let b = generate_test_board();
        let mut s = Compact::build_from_board(&b);

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

    #[test]
    fn state_serialize() {
        for i in 0..100 {
            let starting_player = if i & 1 == 0 {
                Player::Silver
            } else {
                Player::Gold
            };
            let board = crate::Board::random();
            let initial_state = Compact::build_from_board(&board);
            let states = initial_state.random_game(&board, starting_player);
            for s in std::iter::once(initial_state).chain(states.iter().map(|x| x.0)) {
                let serialized = u64::from(&s);
                let deserialized_state = Compact::from_u64(serialized, &board);

                assert_eq!(s, deserialized_state);
            }
        }
    }
}
