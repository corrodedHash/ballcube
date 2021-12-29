mod move_check;
mod win_check;
use ballcube::{Board, CompactState, Player};
use move_check::MoveChecker;
use win_check::WinningChecker;

fn dfs_win(board: &Board, state: &CompactState, player: Player) {
    let checker = WinningChecker::new(board);
    let move_generator = MoveChecker::new(board);

    let mut stack = vec![];
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
