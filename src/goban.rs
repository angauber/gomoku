use std::cmp::{Eq, PartialEq};
use std::hash::Hash;
use std::{fmt, isize};

use bitvec::prelude::*;
use strum_macros::EnumIter;

use crate::evaluator::{Eval, Evaluator};

pub const GOBAN_SIZE: usize = 19;
pub const GOBAN_TOTAL_SIZE: usize = GOBAN_SIZE * GOBAN_SIZE;
pub const BIT_SIZE: usize = GOBAN_TOTAL_SIZE + GOBAN_SIZE;
pub const WIN_MINIMUM_LINE_SIZE: usize = 5;

pub type Bitboard = BitArr!(for BIT_SIZE, in Msb0, u8); // u32 ?? mybe more ?

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

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

impl Position {
    pub fn new(row: usize, col: usize) -> Position {
        Position { row, col }
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
            writeln!(f, "{}", row)?;
        }

        writeln!(f, "0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8")
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

    pub fn get_possible_moves(&self) -> Vec<Position> {
        let mut positions: Vec<Position> = Vec::new();
        let playable = !(self.opponent | self.computer);

        for index in playable.iter_ones() {
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
