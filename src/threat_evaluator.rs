use bitvec::prelude::*;

use crate::evaluator::{Eval, Evaluator};
use crate::goban::{BIT_SIZE, Bitboard, Direction, GOBAN_SIZE};

#[repr(isize)]
#[derive(PartialEq, Debug, Copy, Clone)]
enum Threat {
    Five,
    StraightFour = 500000,
    Four = 50000,
    Three = 20000,
    BrokenThree = 10000,
}

type Pattern = BitArray<Msb0, [u8; 1]>;
type PatternSlice = BitSlice<Msb0, u8>;

type PatternWithLength = (Pattern, Pattern, usize);

pub struct ThreatEvaluator {
    threat_cache: [[Option<Option<Threat>>; 255]; 3],
}

impl Evaluator for ThreatEvaluator {
    fn evaluate(&mut self, player: &Bitboard, opponent: &Bitboard) -> Eval {
        self.evaluate_player(player, opponent)
    }
}

impl ThreatEvaluator {
    pub fn new() -> Self {
        Self {
            threat_cache: [[None; 255]; 3],
        }
    }

    fn evaluate_player(&mut self, player: &Bitboard, opponent: &Bitboard) -> Eval {
        let mut total_score: isize = 0;

        for axis in [
            Direction::East,
            Direction::South,
            Direction::SouthWest,
            Direction::SouthEast,
        ] {
            let eval = self.evaluate_axis(player, opponent, axis);

            match eval {
                Eval::Score(score) => total_score += score,
                Eval::Won | Eval::Lost => return eval,
            }
        }

        Eval::Score(total_score)
    }

    fn evaluate_axis(&mut self, player: &Bitboard, opponent: &Bitboard, axis: Direction) -> Eval {
        let mut score: isize = 0;

        'bits: for index in 0..BIT_SIZE as isize {
            if let Some((player_pattern, opponent_pattern, mut length)) = self.extract_pattern(player, opponent, index, axis) {
                while length >= 5 {
                    if let Some(player_threat) = self.match_threat(
                        &player_pattern[0..length],
                        &opponent_pattern[0..length],
                    ) {
                        match player_threat {
                            Threat::Five => return Eval::Won,
                            _ => score += player_threat as isize,
                        }

                        continue 'bits;
                    } else if let Some(opponent_threat) = self.match_threat(
                        &opponent_pattern[0..length],
                        &player_pattern[0..length],
                    ) {
                        match opponent_threat {
                            Threat::Five => return Eval::Lost,
                            _ => score -= opponent_threat as isize,
                        }

                        continue 'bits;
                    }

                    length -= 1;
                }
            }
        }

        Eval::Score(score)
    }

    fn extract_pattern(
        &self,
        player: &Bitboard,
        opponent: &Bitboard,
        index: isize,
        axis: Direction,
    ) -> Option<PatternWithLength> {
        [7, 6, 5]
            .iter()
            .find_map(|length| self.extract_pattern_length(player, opponent, index, axis, *length))
    }

    fn extract_pattern_length(
        &self,
        player: &Bitboard,
        opponent: &Bitboard,
        index: isize,
        axis: Direction,
        length: usize,
    ) -> Option<PatternWithLength> {
        let mut player_pattern: Pattern = bitarr![Msb0, u8; 0];
        let mut opponent_pattern: Pattern = bitarr![Msb0, u8; 0];

        if !self.is_extractable(index, axis, length) {
            return None;
        }

        for pattern_index in 0..length {
            player_pattern.set(
                pattern_index,
                player[(index + (axis as isize * pattern_index as isize)) as usize] as bool,
            );
            opponent_pattern.set(
                pattern_index,
                opponent[(index + (axis as isize * pattern_index as isize)) as usize] as bool,
            );
        }

        Some((player_pattern, opponent_pattern, length))
    }

    fn is_extractable(&self, index: isize, axis: Direction, length: usize) -> bool {
        let end: isize = index + (axis as isize * length as isize);

        self.is_inbound(index) && self.is_inbound(end)
    }

    fn is_inbound(&self, index: isize) -> bool {
        index >= 0 && index < BIT_SIZE as isize && (index == 0 || index % GOBAN_SIZE as isize != 0)
    }

    fn match_threat(
        &mut self,
        player_slice: &PatternSlice,
        opponent_slice: &PatternSlice,
    ) -> Option<Threat> {
        if opponent_slice.any() {
            return None;
        }

        let group: usize = player_slice.len() - 5;
        let index: usize = player_slice.as_raw_slice()[0] as usize;

        if let Some(cache) = self.threat_cache[group][index] {
            return cache;
        }

        let cache = Self::compute_threat(player_slice);

        self.threat_cache[group][index] = Some(cache);

        cache
    }

    fn compute_threat(slice: &PatternSlice) -> Option<Threat> {
        if slice == bits![1, 1, 1, 1, 1] {
            return Some(Threat::Five);
        }

        let ones = slice.count_ones();

        if ones == 4 {
            if slice == bits![0, 1, 1, 1, 1, 0] {
                return Some(Threat::StraightFour);
            } else if slice.len() == 5 {
                return Some(Threat::Four);
            }
        } else if ones == 3 {
            if slice.eq(bits![0, 0, 1, 1, 1, 0, 0])
                || slice.eq(bits![0, 1, 1, 1, 0, 0])
                || slice.eq(bits![0, 0, 1, 1, 1, 0])
            {
                return Some(Threat::Three);
            } else if slice.eq(bits![0, 1, 0, 1, 1, 0])
                || slice.eq(bits![0, 1, 1, 0, 1, 0])
            {
                return Some(Threat::BrokenThree);
            }
        }

        None
    }
}

#[cfg(test)]
mod evaluator_tests {
    use crate::evaluator::{Eval};
    use crate::goban::{Cell, Goban, Player, WIN_MINIMUM_LINE_SIZE};
    use crate::threat_evaluator::ThreatEvaluator;

    #[test]
    fn it_correctly_detects_win() {
        let mut evaluator = ThreatEvaluator::new();

        let mut board = Goban::new();

        for i in 0..WIN_MINIMUM_LINE_SIZE {
            board.set(0, i, Cell::Computer);
        }

        board.evaluate(&mut evaluator, Player::Computer);

        assert_eq!(board.eval(), Some(Eval::Won));

        board = Goban::new();

        for i in 0..WIN_MINIMUM_LINE_SIZE {
            board.set(i, 0, Cell::Opponent);
        }

        board.evaluate(&mut evaluator, Player::Computer);

        assert_eq!(board.eval(), Some(Eval::Lost));

        board = Goban::new();

        board.set(3, 3, Cell::Opponent);
        board.set(4, 4, Cell::Opponent);
        board.set(5, 5, Cell::Opponent);
        board.set(6, 6, Cell::Opponent);
        board.set(7, 7, Cell::Opponent);

        board.evaluate(&mut evaluator, Player::Computer);

        assert_eq!(board.eval(), Some(Eval::Lost));
    }
}

#[cfg(test)]
mod pattern_tests {
    use bitvec::prelude::*;

    use crate::threat_evaluator::{Threat, ThreatEvaluator};

    #[test]
    fn it_correctly_detects_straight_fours() {
        let mut evaluator = ThreatEvaluator::new();

        let mut computer = bitarr![Msb0, u8; 0];
        let opponent = bitarr![Msb0, u8; 0];

        computer.set(1, true);
        computer.set(2, true);
        computer.set(3, true);
        computer.set(4, true);

        assert_eq!(
            evaluator.match_threat(&computer[0..6], &opponent[0..6]),
            Some(Threat::StraightFour)
        );
    }

    #[test]
    fn it_correctly_detects_fours() {
        let mut evaluator = ThreatEvaluator::new();

        let mut computer = bitarr![Msb0, u8; 0; 380];
        let opponent = bitarr![Msb0, u8; 0; 380];

        computer.set(1, true);
        computer.set(2, true);
        computer.set(3, true);
        computer.set(4, true);

        assert_eq!(
            evaluator.match_threat(&computer[0..5], &opponent[0..5]),
            Some(Threat::Four)
        );
    }

    #[test]
    fn it_correctly_detects_threes() {
        let mut evaluator = ThreatEvaluator::new();

        let mut computer = bitarr![Msb0, u8; 0; 380];
        let opponent = bitarr![Msb0, u8; 0; 380];

        computer.set(2, true);
        computer.set(3, true);
        computer.set(4, true);

        assert_eq!(
            evaluator.match_threat(&computer[0..7], &opponent[0..7]),
            Some(Threat::Three)
        );

        computer.set_all(false);

        computer.set(1, true);
        computer.set(2, true);
        computer.set(3, true);

        assert_eq!(
            evaluator.match_threat(&computer[0..6], &opponent[0..6]),
            Some(Threat::Three)
        );
    }

    #[test]
    fn it_correctly_detects_broken_threes() {
        let mut evaluator = ThreatEvaluator::new();

        let mut computer = bitarr![Msb0, u8; 0; 380];
        let opponent = bitarr![Msb0, u8; 0; 380];

        computer.set(1, true);
        computer.set(2, true);
        computer.set(4, true);

        assert_eq!(
            evaluator.match_threat(&computer[0..6], &opponent[0..6]),
            Some(Threat::BrokenThree)
        );
    }
}
