#![feature(test)]
extern crate test;

use gomoku::gomoku::Gomoku;
use gomoku::goban::{Position,Player};

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn minmax(bencher: &mut Bencher) {
        let mut gomoku: Gomoku = Gomoku::default();

        gomoku.play(Position { row: 3, col: 3}, Player::Human);
        gomoku.play(Position { row: 4, col: 4}, Player::Human);
        gomoku.play(Position { row: 5, col: 5}, Player::Human);
        gomoku.play(Position { row: 7, col: 7}, Player::Human);

        bencher.iter(|| gomoku.play_computer_move(2));
    }
}
