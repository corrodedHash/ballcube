use ballcube::{Board, CompactState, Player};

pub struct MoveChecker {
    gold_gates: [(u8, u8); 6],
    silver_gates: [(u8, u8); 6],
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Move {
    layer: u8,
    gate: u8,
}
impl Move {
    pub fn layer(&self) -> u8 {
        self.layer
    }
    pub fn gate(&self) -> u8 {
        self.gate
    }
}

impl MoveChecker {
    pub fn new(board: &Board) -> Self {
        let mut gold_gates = vec![];
        let mut silver_gates = vec![];

        for layer in 0..4 {
            for gate in 0..3 {
                match board.gate(layer, gate) {
                    Player::Gold => {
                        gold_gates.push((layer, gate));
                    }
                    Player::Silver => {
                        silver_gates.push((layer, gate));
                    }
                }
            }
        }

        debug_assert_eq!(gold_gates.len(), 6);
        debug_assert_eq!(silver_gates.len(), 6);

        Self {
            gold_gates: gold_gates.try_into().unwrap(),
            silver_gates: silver_gates.try_into().unwrap(),
        }
    }

    pub fn moves(&self, state: &CompactState, p: Player) -> Vec<Move> {
        let gates = match p {
            Player::Gold => &self.gold_gates,
            Player::Silver => &self.silver_gates,
        };

        gates
            .iter()
            .copied()
            .filter(|(a, b)| state.get_shift(*a, *b) < 3)
            .map(|(layer, gate)| Move { layer, gate })
            .collect()
    }
}
