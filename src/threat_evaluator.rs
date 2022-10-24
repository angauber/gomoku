use std::cmp::Ordering;
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

impl PartialOrd for Threat {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(match self {
            Self::Five => match other {
                Self::Five => Ordering::Equal,
                _ => Ordering::Less,
            },
            _ => match other {
                Self::Five => Ordering::Greater,
                _ => (*self as isize).cmp(&(*other as isize)),
            }
        })
    }
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
        let mut strongest_continuous_threat: Option<(Threat, bool)> = None;
        let mut no_threats = true;

        for index in 0..BIT_SIZE+1 {
            if let Some(
                (player_full_pattern, opponent_full_pattern, max_length)
            ) = self.extract_pattern(player, opponent, index as isize, axis) {
                no_threats = true;

                for (player_pattern, opponent_pattern, is_player) in [
                    (player_full_pattern, opponent_full_pattern, true),
                    (opponent_full_pattern, player_full_pattern, false)
                ] {
                    'patterns: for length in (5..max_length + 1).rev() {
                        if let Some(threat) = self.match_threat(
                            &player_pattern[0..length],
                            &opponent_pattern[0..length],
                            length,
                        ) {
                            if threat == Threat::Five {
                                return match is_player {
                                    true => Eval::Won,
                                    false => Eval::Lost
                                };
                            }

                            no_threats = false;

                            strongest_continuous_threat = Some((
                                Self::get_strongest_threat(
                                    strongest_continuous_threat.map_or_else(
                                        || None,
                                        |st| Some(st.0),
                                    ),
                                    threat,
                                ),
                                is_player
                            ));

                            break 'patterns;
                        }
                    }
                }
            }

            if no_threats || index == BIT_SIZE {
                if let Some((strongest_threat, is_player)) = strongest_continuous_threat {
                    score = match is_player {
                        true => score + strongest_threat as isize,
                        false => score - strongest_threat as isize,
                    }
                }
                strongest_continuous_threat = None;
            }
        }

        Eval::Score(score)
    }

    fn get_strongest_threat(current: Option<Threat>, new: Threat) -> Threat {
        match current {
            None => new,
            Some(old) => match old > new {
                true => old,
                false => new,
            }
        }
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
        length: usize,
    ) -> Option<Threat> {
        if opponent_slice.any() {
            return None;
        }

        let group: usize = length - 5;
        let index: usize = player_slice.as_raw_slice()[0] as usize;

        if let Some(cache) = self.threat_cache[group][index] {
            return cache;
        }

        let cache = Self::compute_threat(player_slice, length);

        self.threat_cache[group][index] = Some(cache);

        cache
    }

    fn compute_threat_seven(slice: &PatternSlice) -> Option<Threat> {
        if slice.eq(bits![0, 0, 1, 1, 1, 0, 0]) {
            return Some(Threat::Three);
        }

        None
    }

    fn compute_threat_six(slice: &PatternSlice) -> Option<Threat> {
        if slice.eq(bits![0, 1, 1, 1, 1, 0]) {
            return Some(Threat::StraightFour);
        }

        if slice.eq(bits![0, 1, 1, 1, 0, 0]) || slice.eq(bits![0, 0, 1, 1, 1, 0]) {
            return Some(Threat::Three);
        }

        if slice.eq(bits![0, 1, 0, 1, 1, 0]) || slice.eq(bits![0, 1, 1, 0, 1, 0]) {
            return Some(Threat::BrokenThree);
        }

        None
    }

    fn compute_threat_five(slice: &PatternSlice) -> Option<Threat> {
        match slice.count_ones() {
            5 => Some(Threat::Five),
            4 => Some(Threat::Four),
            _ => None,
        }
    }

    fn compute_threat(slice: &PatternSlice, length: usize) -> Option<Threat> {
        match length {
            7 => Self::compute_threat_seven(slice),
            6 => Self::compute_threat_six(slice),
            5 => Self::compute_threat_five(slice),
            _ => panic!("Unknown threat length {}", length),
        }
    }
}

#[cfg(test)]
mod evaluator_tests {
    use std::rc::Rc;
    use crate::evaluator::{Eval};
    use crate::goban::{Goban, Move, Player, Position, Stone, WIN_MINIMUM_LINE_SIZE};
    use crate::threat_evaluator::ThreatEvaluator;
    use crate::zobrist_hashing::ZobristHasher;

    #[test]
    fn it_correctly_detects_win() {
        let mut evaluator = ThreatEvaluator::new();
        let hasher = Rc::new(ZobristHasher::initialize());

        let mut board = Goban::new(Rc::clone(&hasher));

        for i in 0..WIN_MINIMUM_LINE_SIZE {
            board.apply_move(Move::new(Stone::White, Position::new(0, i)));
        }

        let eval = board.evaluate(&mut evaluator, Player::Computer);

        assert_eq!(eval, Eval::Won);

        board = Goban::new(Rc::clone(&hasher));

        for i in 0..WIN_MINIMUM_LINE_SIZE {
            board.apply_move(Move::new(Stone::Black, Position::new(i, 0)));
        }

        let eval = board.evaluate(&mut evaluator, Player::Computer);

        assert_eq!(eval, Eval::Lost);

        board = Goban::new(Rc::clone(&hasher));

        board.apply_move(Move::new(Stone::Black, Position::new(3, 3)));
        board.apply_move(Move::new(Stone::Black, Position::new(4, 4)));
        board.apply_move(Move::new(Stone::Black, Position::new(5, 5)));
        board.apply_move(Move::new(Stone::Black, Position::new(6, 6)));
        board.apply_move(Move::new(Stone::Black, Position::new(7, 7)));

        let eval = board.evaluate(&mut evaluator, Player::Computer);

        assert_eq!(eval, Eval::Lost);
    }
}

#[cfg(test)]
mod pattern_tests {
    use bitvec::prelude::*;
    use crate::evaluator::{Eval, Evaluator};

    use crate::threat_evaluator::{Threat, ThreatEvaluator};

    #[test]
    fn it_correctly_detects_fives() {
        let mut evaluator = ThreatEvaluator::new();

        let mut computer = bitarr![Msb0, u8; 0; 380];
        let opponent = bitarr![Msb0, u8; 0; 380];

        computer.set(0, true);
        computer.set(1, true);
        computer.set(2, true);
        computer.set(3, true);
        computer.set(4, true);

        assert_eq!(
            evaluator.match_threat(&computer[0..5], &opponent[0..5], 5),
            Some(Threat::Five)
        );

        assert_eq!(evaluator.evaluate(&computer, &opponent), Eval::Won);
    }

    #[test]
    fn it_correctly_detects_straight_fours() {
        let mut evaluator = ThreatEvaluator::new();

        let mut computer = bitarr![Msb0, u8; 0; 380];
        let opponent = bitarr![Msb0, u8; 0; 380];

        computer.set(1, true);
        computer.set(2, true);
        computer.set(3, true);
        computer.set(4, true);

        assert_eq!(
            evaluator.match_threat(&computer[0..6], &opponent[0..6], 6),
            Some(Threat::StraightFour)
        );

        assert_eq!(evaluator.evaluate(&computer, &opponent), Eval::Score(Threat::StraightFour as isize));
    }

    #[test]
    fn it_correctly_detects_fours() {
        let mut evaluator = ThreatEvaluator::new();

        let mut computer = bitarr![Msb0, u8; 0; 380];
        let opponent = bitarr![Msb0, u8; 0; 380];

        computer.set(0, true);
        computer.set(1, true);
        computer.set(2, true);
        computer.set(3, true);

        assert_eq!(
            evaluator.match_threat(&computer[0..5], &opponent[0..5], 5),
            Some(Threat::Four)
        );

        assert_eq!(evaluator.evaluate(&computer, &opponent), Eval::Score(Threat::Four as isize));
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
            evaluator.match_threat(&computer[0..7], &opponent[0..7], 7),
            Some(Threat::Three)
        );

        computer.set_all(false);

        computer.set(1, true);
        computer.set(2, true);
        computer.set(3, true);

        assert_eq!(
            evaluator.match_threat(&computer[0..6], &opponent[0..6], 6),
            Some(Threat::Three)
        );

        assert_eq!(evaluator.evaluate(&computer, &opponent), Eval::Score(Threat::Three as isize));
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
            evaluator.match_threat(&computer[0..6], &opponent[0..6], 6),
            Some(Threat::BrokenThree)
        );

        assert_eq!(evaluator.evaluate(&computer, &opponent), Eval::Score(Threat::BrokenThree as isize));
    }
}
