use gomoku::goban::{Player, Position};
use gomoku::gomoku::{Gomoku};

fn main() {
    let mut gomoku = Gomoku::default();

    gomoku.play(Position { row: 1, col: 1 }, Player::Opponent).ok();
    gomoku.play(Position {row: 2, col: 2}, Player::Opponent).ok();
    gomoku.play(Position {row: 3, col: 3}, Player::Opponent).ok();

    gomoku.play_computer_move(4);
}
