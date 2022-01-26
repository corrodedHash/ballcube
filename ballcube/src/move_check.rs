use crate::{Board, Compact, Player};

pub struct MoveChecker {
    gold_gates: [Move; 6],
    silver_gates: [Move; 6],
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Move {
    layer: u8,
    gate: u8,
}
impl Move {
    #[must_use]
    pub fn layer(self) -> u8 {
        self.layer
    }
    #[must_use]
    pub fn gate(self) -> u8 {
        self.gate
    }
}

impl MoveChecker {
    /// # Panics
    /// Never
    #[must_use]
    pub fn new(board: &Board) -> Self {
        let mut gold_gates = vec![];
        let mut silver_gates = vec![];

        for layer in 0..4 {
            for gate in 0..3 {
                let chosen_board = match board.gate(layer, gate) {
                    Player::Gold => &mut gold_gates,
                    Player::Silver => &mut silver_gates,
                };
                chosen_board.push(Move { layer, gate });
            }
        }

        debug_assert_eq!(gold_gates.len(), 6);
        debug_assert_eq!(silver_gates.len(), 6);

        Self {
            gold_gates: gold_gates.try_into().unwrap(),
            silver_gates: silver_gates.try_into().unwrap(),
        }
    }

    #[must_use]
    pub fn moves(&self, state: &Compact, p: Player) -> Vec<Move> {
        let gates = match p {
            Player::Gold => &self.gold_gates,
            Player::Silver => &self.silver_gates,
        };

        gates
            .iter()
            .copied()
            .filter(|Move { layer, gate }| state.get_shift(*layer, *gate) < 3)
            .collect()
    }
}
