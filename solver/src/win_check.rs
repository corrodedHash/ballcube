use ballcube::{Board, CompactState, Player};

pub struct WinningChecker {
    gold_ball_mask: u64,
    silver_ball_mask: u64,
}

pub enum Winner {
    None,
    Both,
    One(Player),
}

impl WinningChecker {
    pub fn new(board: &Board) -> Self {
        let mut gold_ball_mask = 0u64;
        let mut silver_ball_mask = 0u64;

        for i in 0..9 {
            match board.ball(i) {
                Some(Player::Gold) => gold_ball_mask |= 1 << i,
                Some(Player::Silver) => silver_ball_mask |= 1 << i,
                None => (),
            }
        }
        debug_assert_eq!(gold_ball_mask.count_ones(), 4);
        debug_assert_eq!(silver_ball_mask.count_ones(), 4);

        gold_ball_mask |= gold_ball_mask << 9;
        gold_ball_mask |= gold_ball_mask << 18;
        silver_ball_mask |= silver_ball_mask << 9;
        silver_ball_mask |= silver_ball_mask << 18;

        Self {
            gold_ball_mask,
            silver_ball_mask,
        }
    }

    pub fn won(&self, state: &CompactState) -> Winner {
        let gw = state.get_gate_bits() & self.gold_ball_mask == 0;
        let sw = state.get_gate_bits() & self.silver_ball_mask == 0;
        match (gw, sw) {
            (false, false) => Winner::None,
            (true, false) => Winner::One(Player::Gold),
            (false, true) => Winner::One(Player::Silver),
            (true, true) => Winner::Both,
        }
    }
}
