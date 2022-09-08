use std::time::Instant;
use gomoku::goban::{Player, Position};
use gomoku::gomoku::{Gomoku};

fn main() {
    let mut gomoku = Gomoku::default();

    gomoku.play(Position {row: 3, col: 3}, Player::Opponent).ok();
    gomoku.play(Position {row: 0, col: 2}, Player::Computer).ok();

    gomoku.play(Position {row: 4, col: 4}, Player::Opponent).ok();
    gomoku.play(Position {row: 0, col: 3}, Player::Computer).ok();

    gomoku.play(Position {row: 5, col: 5}, Player::Opponent).ok();
    gomoku.play(Position {row: 6, col: 6}, Player::Computer).ok();

    gomoku.play(Position {row: 3, col: 7}, Player::Opponent).ok();
    gomoku.play(Position {row: 0, col: 1}, Player::Computer).ok();

    gomoku.play(Position {row: 0, col: 4}, Player::Opponent).ok();

    gomoku.print_board();

    let t0 = Instant::now();

    gomoku.play_computer_move(4);

    println!("Took: {} ms", t0.elapsed().as_millis());

    gomoku.print_board();
}
