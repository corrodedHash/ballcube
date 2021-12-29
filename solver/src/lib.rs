mod dfs;
mod move_check;
mod win_check;
use ballcube::{Board, CompactState, Player};
use move_check::{Move, MoveChecker};
use win_check::WinningChecker;

fn other_player(player: Player) -> Player {
    match player {
        Player::Gold => Player::Silver,
        Player::Silver => Player::Gold,
    }
}

fn random_board(moves: u8) -> (Board, State) {
    
}
