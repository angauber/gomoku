#![feature(test)]
extern crate test;

use gomoku::goban::{Player, Position};
use gomoku::gomoku::Gomoku;

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn minmax(bencher: &mut Bencher) {
        let mut gomoku: Gomoku = Gomoku::default();

        gomoku.play(Position { row: 3, col: 3 }, Player::Opponent);
        gomoku.play(Position { row: 4, col: 4 }, Player::Opponent);
        gomoku.play(Position { row: 5, col: 5 }, Player::Opponent);
        gomoku.play(Position { row: 7, col: 7 }, Player::Opponent);

        bencher.iter(|| gomoku.play_computer_move(3));
    }
}
