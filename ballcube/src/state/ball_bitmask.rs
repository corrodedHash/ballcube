use crate::Board;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BallBitmask {
    mask: u64,
}

impl BallBitmask {
    pub const COMPRESSED_BITSIZE: u32 = u64::BITS - 5_u64.pow(9).leading_zeros();

    pub const fn new(mask: u64) -> Self {
        Self { mask }
    }

    pub fn depth(self) -> [u8; 9] {
        let mut ballmask = 1_u64;
        for _ in 0..4 {
            ballmask <<= 9;
            ballmask |= 1;
        }
        let mut result = [0_u8; 9];

        for (i, r) in result.iter_mut().enumerate() {
            let found_ball = self.mask & (ballmask << i);
            debug_assert!(found_ball.count_ones() <= 1);
            *r = std::cmp::min(found_ball.trailing_zeros() as u8 / 9, 4);
        }

        result
    }

    pub fn compress(self) -> u64 {
        let mut ball_bits = 0_u64;
        let mut power_of_five = 1;

        for ball in self.depth() {
            debug_assert!(ball <= 4);

            ball_bits += u64::from(ball) * power_of_five;
            power_of_five = power_of_five.saturating_mul(5);
        }
        ball_bits
    }

    pub fn decompress(mut compressed_balls: u64, board: &Board) -> Self {
        let mut depths = [0; 9];
        for current_depth in &mut depths {
            *current_depth = compressed_balls % 5;
            compressed_balls /= 5;
        }

        let mut result = 0;
        for (index, depth) in (0..).zip(depths) {
            if board.ball(index as u8).is_none() {
                debug_assert_eq!(depth, 4);
                continue;
            }
            result |= 1_u64 << (depth * 9 + index);
        }
        Self { mask: result }
    }

    pub fn drop(&mut self, gate_bitmask: u64) {
        while self.mask & gate_bitmask != 0 {
            let dropped_balls = self.mask & gate_bitmask;
            debug_assert_eq!(self.mask & dropped_balls, dropped_balls);
            debug_assert_eq!(self.mask & (dropped_balls << 9), 0);
            self.mask ^= dropped_balls | (dropped_balls << 9);
        }
    }

    pub const fn get_mask(self) -> u64 {
        self.mask
    }
}
