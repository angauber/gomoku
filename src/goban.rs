use std::{fmt, isize};
use std::cmp::{Eq, PartialEq};
use std::hash::Hash;

use bitvec::prelude::*;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::evaluator::{Eval, Evaluator};

pub const GOBAN_SIZE: usize = 19;
pub const GOBAN_TOTAL_SIZE: usize = GOBAN_SIZE * GOBAN_SIZE;
pub const BIT_SIZE: usize = GOBAN_TOTAL_SIZE + GOBAN_SIZE;
pub const WIN_MINIMUM_LINE_SIZE: usize = 5;

pub type Bitboard = BitArr!(for BIT_SIZE, in Msb0, u8);

#[derive(Debug, Clone, Copy)]
pub enum Player {
    Computer,
    Opponent,
}

#[derive(PartialEq)]
pub enum Cell {
    Empty,
    Computer,
    Opponent,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Goban {
    computer: Bitboard,
    opponent: Bitboard,
    eval: Option<Eval>,
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

#[derive(PartialEq, Eq, Hash)]
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

    pub fn from_coordinates(position: &str) -> Result<Position, String> {
        if position.len() < 2 {
            return Err(format!("Invalid position `{}`", position));
        }

        let (col_str, row_str) = position.split_at(1);

        let ascii_offset: isize = 97;
        let col: isize = col_str.chars().next().unwrap().to_lowercase().next().unwrap() as isize - ascii_offset;
        let row: isize = 19 - row_str.parse::<isize>().unwrap_or(-1);

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

impl Default for Goban {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Goban {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in 0..GOBAN_SIZE {
            for col in 0..GOBAN_SIZE {
                write!(
                    f,
                    "{}",
                    match self.get(row, col) {
                        Cell::Empty => ". ",
                        Cell::Opponent => "X ",
                        Cell::Computer => "O ",
                    }
                )?;
            }
            writeln!(f, "{}", 19 - row)?;
        }

        writeln!(f, "A B C D E F G H I J K L M N O P Q R S")
    }
}

impl Goban {
    pub fn new() -> Goban {
        Goban {
            opponent: bitarr![Msb0, u8; 0; BIT_SIZE],
            computer: bitarr![Msb0, u8; 0; BIT_SIZE],
            eval: None,
        }
    }

    pub fn set(&mut self, row: usize, col: usize, cell: Cell) {
        let index: usize = (row * (GOBAN_SIZE + 1)) + col;

        self.opponent.set(index, false);
        self.computer.set(index, false);

        match cell {
            Cell::Empty => (),
            Cell::Opponent => self.opponent.set(index, true),
            Cell::Computer => self.computer.set(index, true),
        }
    }

    pub fn get(&self, row: usize, col: usize) -> Cell {
        let index: usize = (row * (GOBAN_SIZE + 1)) + col;

        match self.opponent[index] {
            true => Cell::Opponent,
            false => match self.computer[index] {
                true => Cell::Computer,
                false => Cell::Empty,
            },
        }
    }

    pub fn evaluate(&mut self, evaluator: &mut dyn Evaluator, player: Player) {
        self.eval = Some(match player {
            Player::Computer => evaluator.evaluate(&self.computer, &self.opponent),
            Player::Opponent => evaluator.evaluate(&self.opponent, &self.computer),
        });
    }

    pub fn eval(&self) -> Option<Eval> {
        self.eval.clone()
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

    pub fn get_limited_moves(&self, steps: usize) -> Vec<Position>
    {
        let mut positions: Vec<Position> = Vec::new();
        let mut played_set = self.opponent | self.computer;
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
}
