use gomoku::goban::{Player, Position};
use gomoku::gomoku::{Gomoku};

fn main() -> Result<(),String> {
    let mut gomoku = Gomoku::default();

    gomoku.play(Position::from_coordinates("a15")?, Player::Opponent).ok();
    gomoku.play(Position::from_coordinates("b15")?, Player::Opponent).ok();
    gomoku.play(Position::from_coordinates("c15")?, Player::Opponent).ok();

    gomoku.play(Position::from_coordinates("f14")?, Player::Opponent).ok();
    gomoku.play(Position::from_coordinates("g13")?, Player::Opponent).ok();
    gomoku.play(Position::from_coordinates("i10")?, Player::Opponent).ok();
    gomoku.play(Position::from_coordinates("i9")?, Player::Opponent).ok();

    gomoku.print_board();

    Ok(())
}
