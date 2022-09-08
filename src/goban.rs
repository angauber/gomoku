use std::{fmt, isize};
use std::cmp::{Eq, PartialEq};
use std::hash::Hash;
use std::rc::Rc;

use bitvec::prelude::*;
use strum::IntoEnumIterator;
use strum_macros::{EnumIter};

use crate::evaluator::{Eval, Evaluator};
use crate::zobrist_hashing::{ZobristHasher, ZobristHash};

pub const GOBAN_SIZE: usize = 19;
pub const GOBAN_TOTAL_SIZE: usize = GOBAN_SIZE * GOBAN_SIZE;
pub const BIT_SIZE: usize = GOBAN_TOTAL_SIZE + GOBAN_SIZE;
pub const WIN_MINIMUM_LINE_SIZE: usize = 5;

pub type Bitboard = BitArr!(for BIT_SIZE, in Msb0, u8);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Player {
    Computer,
    Opponent,
}

#[derive(PartialEq, Debug)]
pub enum Stone {
    Black,
    White
}

#[derive(Clone)]
pub struct Goban {
    white_stones: Bitboard,
    black_stones: Bitboard,
    zobrist_hasher: Rc<ZobristHasher>,
    zobrist_hash: ZobristHash,
}

#[derive(EnumIter, Copy, Clone, Debug)]
pub enum Direction {
    North = -((GOBAN_SIZE + 1) as isize),
    NorthEast = -((GOBAN_SIZE + 1) as isize) + 1,
    East = 1,
    SouthEast = (GOBAN_SIZE + 2) as isize,
    South = (GOBAN_SIZE + 1) as isize,
    SouthWest = GOBAN_SIZE as isize,
    West = -1,
    NorthWest = -((GOBAN_SIZE + 1) as isize) - 1,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (col, row) = self.to_coordinates();

        write!(f, "{}{}", col, row)
    }
}

impl Position {
    pub fn new(row: usize, col: usize) -> Position {
        Position { row, col }
    }

    pub fn index(&self) -> usize {
        (self.row * (GOBAN_SIZE + 1)) + self.col
    }

    pub fn from_coordinates(position: &str) -> Result<Position, String> {
        if position.len() < 2 {
            return Err(format!("Invalid position `{}`", position));
        }

        let (col_str, row_str) = position.split_at(1);

        let ascii_offset: isize = 97;
        let col: isize = col_str.trim().chars().next().unwrap().to_lowercase().next().unwrap() as isize - ascii_offset;
        let row: isize = 19 - row_str.trim().parse::<isize>().unwrap_or(-1);

        if !(0..19).contains(&col) {
            return Err(format!("invalid col value `{}`", col_str));
        }

        if !(0..19).contains(&row) {
            return Err(format!("invalid row value `{}`", row_str));
        }


        Ok(Position::new(row as usize, col as usize))
    }

    pub fn to_coordinates(&self) -> (char, usize) {
        let ascii_offset: usize = 65;

        let col: char = (self.col + ascii_offset) as u8 as char;
        let row: usize = 19 - self.row;

        (col, row)
    }
}

#[derive(Debug)]
pub struct Move {
    pub stone: Stone,
    pub position: Position,
}

impl Move {
    pub fn new(stone: Stone, position: Position) -> Self {
        Move {
            stone,
            position
        }
    }
}

impl PartialEq<Self> for Goban {
    fn eq(&self, other: &Self) -> bool {
        self.zobrist_hash == other.zobrist_hash
    }
}

impl Eq for Goban {}

impl fmt::Debug for Goban {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in 0..GOBAN_SIZE {
            for col in 0..GOBAN_SIZE {
                write!(
                    f,
                    "{}",
                    match self.get(row, col) {
                        None => ". ",
                        Some(Stone::Black) => "X ",
                        Some(Stone::White) => "O ",
                    }
                )?;
            }
            writeln!(f, "{}", 19 - row)?;
        }

        writeln!(f, "A B C D E F G H I J K L M N O P Q R S")
    }
}

impl Goban {
    pub fn new(hasher: Rc<ZobristHasher>) -> Goban {
        Goban {
            black_stones: bitarr![Msb0, u8; 0; BIT_SIZE],
            white_stones: bitarr![Msb0, u8; 0; BIT_SIZE],
            zobrist_hasher: hasher,
            zobrist_hash: 0,
        }
    }

    pub fn apply_move(&mut self, move_to_play: Move) {
        let position = move_to_play.position.index();

        self.black_stones.set(position, false);
        self.white_stones.set(position, false);

        match move_to_play.stone {
            Stone::Black => self.black_stones.set(position, true),
            Stone::White => self.white_stones.set(position, true),
        }

        self.zobrist_hash = self.zobrist_hasher.update_hash(self.zobrist_hash, move_to_play);
    }

    pub fn get(&self, row: usize, col: usize) -> Option<Stone> {
        let index: usize = (row * (GOBAN_SIZE + 1)) + col;

        if self.black_stones[index] {
            return Some(Stone::Black);
        }

        if self.white_stones[index] {
            return Some(Stone::White);
        }

        None
    }

    pub fn evaluate(&self, evaluator: &mut dyn Evaluator, player: Player) -> Eval {
        match player {
            Player::Computer => evaluator.evaluate(&self.white_stones, &self.black_stones),
            Player::Opponent => evaluator.evaluate(&self.black_stones, &self.white_stones),
        }
    }

    pub fn get_limited_moves(&self, steps: usize) -> Vec<Position>
    {
        let mut positions: Vec<Position> = Vec::new();
        let mut played_set = self.black_stones | self.white_stones;
        let playable_set = !played_set;
        let mut limited_set = played_set;

        for _ in 0..steps {
            for axis in Direction::iter() {
                limited_set |= Self::dilate(&played_set, axis);
            }
            played_set = limited_set;
        }

        limited_set &= playable_set;

        for index in limited_set.iter_ones() {
            let row = index / (GOBAN_SIZE + 1);
            let col = index - (row * (GOBAN_SIZE + 1));

            if col == GOBAN_SIZE || index >= BIT_SIZE {
                continue;
            }

            positions.push(Position::new(row, col));
        }

        positions
    }

    pub fn get_hash(&self) -> ZobristHash
    {
        self.zobrist_hash
    }

    fn dilate(bitboard: &Bitboard, axis: Direction) -> Bitboard {
        let mut rhs = *bitboard;

        match axis as isize {
            n if n > 0 => rhs.shift_right(n as usize),
            n if n < 0 => rhs.shift_left(-n as usize),
            _ => {}
        }

        *bitboard | rhs
    }
}
