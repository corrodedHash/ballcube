use crate::Player;

use super::Board;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Gate {
    pub allegiance: Player,
    pub topleft: bool,
    pub gatetype: u8,
}

impl Gate {
    pub fn build() -> GateBuilder {
        GateBuilder::default()
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Copy, Debug, Default)]
pub struct GateBuilder {
    allegiance: Option<Player>,
    topleft: Option<bool>,
    gatetype: Option<u8>,
}

impl GateBuilder {
    pub fn s(&mut self) -> &mut Self {
        self.allegiance = Some(Player::Silver);
        self
    }
    pub fn g(&mut self) -> &mut Self {
        self.allegiance = Some(Player::Gold);
        self
    }
    pub fn t(&mut self) -> &mut Self {
        self.topleft = Some(true);
        self
    }
    pub fn b(&mut self) -> &mut Self {
        self.topleft = Some(false);
        self
    }
    pub fn ty(&mut self, index: u8) -> &mut Self {
        self.gatetype = Some(index);
        self
    }
    pub fn finalize(self) -> Option<Gate> {
        let allegiance = self.allegiance?;
        let topleft = self.topleft?;
        let gatetype = self.gatetype?;
        Some(Gate {
            allegiance,
            topleft,
            gatetype,
        })
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Default, Clone, Debug)]
pub struct BoardBuilder {
    pub gold_balls: Vec<u8>,
    pub silver_balls: Vec<u8>,
    pub gates_horizontal: [Option<bool>; 4],
    pub gates: [Option<Gate>; 12],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoardBuildingError {
    GateDirectionUndefined(u8),
    GateUndefined(u8),
    BallUndefined,
    BallCountIncorrect,
    GateAllegianceIncorrect,
}

impl BoardBuilder {
    /// # Errors
    /// Will error when board is not properly defined yet
    pub fn finalize(mut self) -> Result<Board, BoardBuildingError> {
        let mut gates_horizontal = [false; 4];
        for (id, (g, r)) in (0_u8..).zip(
            self.gates_horizontal
                .iter()
                .zip(gates_horizontal.iter_mut()),
        ) {
            *r = (*g).ok_or(BoardBuildingError::GateDirectionUndefined(id))?;
        }

        let (mut gates_topleft_v, mut gates_silver_v, mut gate_type_v) = (vec![], vec![], vec![]);

        for (id, x) in (0_u8..).zip(self.gates.iter()) {
            if let Some(g) = x {
                gates_topleft_v.push(g.topleft);
                gates_silver_v.push(g.allegiance == Player::Silver);
                gate_type_v.push(g.gatetype);
            } else {
                return Err(BoardBuildingError::GateUndefined(id));
            }
        }

        if gates_silver_v.iter().filter(|x| x == &&true).count() != 6 {
            return Err(BoardBuildingError::GateAllegianceIncorrect);
        }

        let (mut gates_topleft, mut gates_silver, mut gate_type) =
            ([[false; 3]; 4], [[false; 3]; 4], [[0_u8; 3]; 4]);

        for (id, (t, (s, ty))) in gates_topleft_v
            .into_iter()
            .zip(gates_silver_v.into_iter().zip(gate_type_v.into_iter()))
            .enumerate()
        {
            gates_topleft[id / 3][id % 3] = t;
            gates_silver[id / 3][id % 3] = s;
            gate_type[id / 3][id % 3] = ty;
        }
        self.gold_balls.sort_unstable();
        self.silver_balls.sort_unstable();

        let gold_balls = self
            .gold_balls
            .try_into()
            .map_err(|_| BoardBuildingError::BallCountIncorrect)?;

        let silver_balls = self
            .silver_balls
            .try_into()
            .map_err(|_| BoardBuildingError::BallCountIncorrect)?;

        Ok(Board {
            gold_balls,
            silver_balls,
            gates_horizontal,
            gates_topleft,
            gates_silver,
            gate_type,
        })
    }
}
