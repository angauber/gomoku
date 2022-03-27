use crate::goban::Bitboard;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Eval {
    Won,
    Lost,
    Score(isize),
}

pub trait Evaluator {
    fn evaluate(&mut self, player: &Bitboard, opponent: &Bitboard) -> Eval;
}
