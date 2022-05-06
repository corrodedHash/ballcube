#[derive(Clone, Debug)]
pub struct MoveChain {
    chain: Vec<Move>,
    starting_player: Player,
}
impl MoveChain {
    fn new(starting_player: Player) -> Self {
        Self {
            chain: vec![],
            starting_player,
        }
    }

    fn prepend(&mut self, m: Move) {
        self.chain.push(m);
        self.starting_player = self.starting_player.other();
    }

    #[must_use]
    pub fn moves(&self) -> &Vec<Move> {
        &self.chain
    }

    #[must_use]
    pub fn starting_player(&self) -> Player {
        self.starting_player
    }
}
