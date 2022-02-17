use std::{fmt, isize};
use std::cmp::{PartialEq, Eq, max};
use std::hash::Hash;

use bitvec::prelude::*;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub const GOBAN_SIZE: usize = 19;
pub const WIN_MINIMUM_LINE_SIZE: usize = 5;
const BIT_SIZE: usize = 380;

type Bitboard = BitArr!(for BIT_SIZE, in Msb0, u8);

#[derive(Debug, Clone, Copy)]
pub enum Player {
    Human,
    Computer,
}

#[derive(PartialEq)]
pub enum Cell {
    Empty,
    Human,
    Computer,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Goban {
    human: Bitboard,
    computer: Bitboard,
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

pub enum Eval {
    Won,
    Lost,
    Score(isize),
}

impl Position {
    pub fn new(row: usize, col: usize) -> Position {
        Position {
            row,
            col,
        }
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
                write!(f, "{}", match self.get(row, col) {
                    Cell::Empty => ". ",
                    Cell::Human => "X ",
                    Cell::Computer => "O ",
                })?;
            }
            writeln!(f, "{}", row)?;
        }

        writeln!(f, "0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8")
    }
}

impl Goban {
    pub fn new() -> Goban {
        Goban {
            human: bitarr![Msb0, u8; 0; BIT_SIZE],
            computer: bitarr![Msb0, u8; 0; BIT_SIZE],
        }
    }

    pub fn is_won(&self, player: Player) -> bool {
        for axis in [Direction::East, Direction::South, Direction::SouthWest, Direction::SouthEast] {
            if self.axis_maximum_line_size(player, axis) >= WIN_MINIMUM_LINE_SIZE {
                return true;
            }
        }

        false
    }

    pub fn set(&mut self, row: usize, col: usize, cell: Cell) {
        let index: usize = (row * (GOBAN_SIZE + 1)) + col;

        self.human.set(index, false);
        self.computer.set(index, false);

        match cell {
            Cell::Empty => (),
            Cell::Human => self.human.set(index, true),
            Cell::Computer => self.computer.set(index, true),
        }
    }

    pub fn get(&self, row: usize, col: usize) -> Cell {
        let index: usize = (row * (GOBAN_SIZE + 1)) + col;

        match self.human[index] {
            true => Cell::Human,
            false => match self.computer[index] {
                true => Cell::Computer,
                false => Cell::Empty,
            }
        }
    }

    pub fn evaluate(&self) -> Eval {
        let human_max_line_size = self.maximum_line_size(Player::Human);
        let computer_max_line_size = self.maximum_line_size(Player::Computer);

        if human_max_line_size >= WIN_MINIMUM_LINE_SIZE {
            return Eval::Lost;
        }

        if computer_max_line_size >= WIN_MINIMUM_LINE_SIZE {
            return Eval::Won;
        }

        Eval::Score(self.compute_score(computer_max_line_size, human_max_line_size))
    }

    pub fn get_possible_moves(&self) -> Vec<Position>
    {
        let mut positions: Vec<Position> = Vec::new();
        let playable = !(self.human | self.computer);

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

    pub fn get_limited_moves(&self, steps: usize) -> Vec<Position>
    {
        let mut positions: Vec<Position> = Vec::new();
        let mut played_set = self.human | self.computer;
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

    fn print_bitboard(bitboard: &Bitboard) {
        for row in 0..GOBAN_SIZE {
            for col in 0..GOBAN_SIZE {
                print!("{} ", match bitboard[(row * (GOBAN_SIZE + 1)) + col] {
                    true => 1,
                    false => 0,
                });
            }
            println!();
        }

        println!();
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

    fn erode(bitboard: &mut Bitboard, axis: Direction) {
        let mut rhs = *bitboard;

        match axis as isize {
            n if n > 0 => rhs.shift_right(n as usize),
            n if n < 0 => rhs.shift_left(-n as usize),
            _ => {}
        }

        *bitboard &= rhs
    }

    /**
     * Eroding the player bitboard in each axis to find the maximum length
     */
    fn axis_maximum_line_size(&self, player: Player, axis: Direction) -> usize {
        let mut size: usize = 0;
        let mut bitboard = match player {
            Player::Human => self.human,
            Player::Computer => self.computer,
        };

        while bitboard.any() {
            Goban::erode(&mut bitboard, axis);
            size += 1;
        }

        size
    }

    fn compute_score(&self, computer_line_size: usize, human_line_size: usize) -> isize {
        computer_line_size.pow(2) as isize - human_line_size.pow(2) as isize
    }

    fn maximum_line_size(&self, player: Player) -> usize {
        let mut result: usize = 0;

        /* Could we erode in each axis at the same time ? */
        for axis in [Direction::East, Direction::South, Direction::SouthWest, Direction::SouthEast] {
            result = max(result, self.axis_maximum_line_size(player, axis));
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use crate::goban::{Direction, Cell, Goban, GOBAN_SIZE, Player, WIN_MINIMUM_LINE_SIZE};

    #[test]
    fn it_correctly_detects_win() {
        let mut goban = Goban::new();

        for i in 0..WIN_MINIMUM_LINE_SIZE {
            goban.set(0, i, Cell::Computer);
        }

        assert_eq!(goban.maximum_line_size(Player::Computer), WIN_MINIMUM_LINE_SIZE);
        assert_eq!(goban.axis_maximum_line_size(Player::Computer, Direction::East), WIN_MINIMUM_LINE_SIZE);
        assert_eq!(goban.axis_maximum_line_size(Player::Computer, Direction::West), WIN_MINIMUM_LINE_SIZE);
        assert_eq!(goban.is_won(Player::Computer), true);

        goban = Goban::new();

        for i in 0..GOBAN_SIZE {
            goban.set(18, 18 - i, Cell::Human);
        }

        goban.set(18, 10, Cell::Computer);

        assert_eq!(goban.maximum_line_size(Player::Human), 10);
        assert_eq!(goban.axis_maximum_line_size(Player::Human, Direction::East), 10);
        assert_eq!(goban.axis_maximum_line_size(Player::Human, Direction::West), 10);
        assert_eq!(goban.is_won(Player::Human), true);

        goban = Goban::new();

        goban.set(4, 4, Cell::Computer);
        goban.set(5, 5, Cell::Human);
        goban.set(6, 6, Cell::Human);
        goban.set(7, 7, Cell::Human);

        assert_eq!(goban.maximum_line_size(Player::Human), 3);
        assert_eq!(goban.axis_maximum_line_size(Player::Human, Direction::SouthEast), 3);
        assert_eq!(goban.axis_maximum_line_size(Player::Human, Direction::NorthWest), 3);
    }

    #[test]
    fn it_correctly_erodes_columns() {
        let mut goban: Goban = Goban::new();

        for i in 0..WIN_MINIMUM_LINE_SIZE - 1 {
            goban.set(i, 0, Cell::Computer);
        }

        assert_eq!(goban.maximum_line_size(Player::Computer), WIN_MINIMUM_LINE_SIZE - 1);
        assert_eq!(goban.axis_maximum_line_size(Player::Computer, Direction::South), WIN_MINIMUM_LINE_SIZE - 1);
        assert_eq!(goban.axis_maximum_line_size(Player::Computer, Direction::North), WIN_MINIMUM_LINE_SIZE - 1);
        assert_eq!(goban.is_won(Player::Computer), false);

        goban = Goban::new();

        for i in 0..WIN_MINIMUM_LINE_SIZE {
            goban.set(18 - i, 3, Cell::Human);
        }

        assert_eq!(goban.maximum_line_size(Player::Human), WIN_MINIMUM_LINE_SIZE);
        assert_eq!(goban.axis_maximum_line_size(Player::Human, Direction::South), WIN_MINIMUM_LINE_SIZE);
        assert_eq!(goban.axis_maximum_line_size(Player::Human, Direction::North), WIN_MINIMUM_LINE_SIZE);
        assert_eq!(goban.is_won(Player::Human), true);
    }

    #[test]
    fn it_correctly_erodes_diagonally() {
        let mut goban: Goban = Goban::new();

        for i in 0..WIN_MINIMUM_LINE_SIZE + 1 {
            goban.set(i, i, Cell::Computer);
        }

        assert_eq!(goban.maximum_line_size(Player::Computer), WIN_MINIMUM_LINE_SIZE + 1);
        assert_eq!(goban.axis_maximum_line_size(Player::Computer, Direction::NorthWest), WIN_MINIMUM_LINE_SIZE + 1);
        assert_eq!(goban.axis_maximum_line_size(Player::Computer, Direction::SouthEast), WIN_MINIMUM_LINE_SIZE + 1);
        assert_eq!(goban.is_won(Player::Computer), true);

        goban = Goban::new();

        for i in 0..WIN_MINIMUM_LINE_SIZE {
            goban.set(i, 18 - i, Cell::Human);
        }

        assert_eq!(goban.maximum_line_size(Player::Human), WIN_MINIMUM_LINE_SIZE);
        assert_eq!(goban.axis_maximum_line_size(Player::Human, Direction::SouthWest), WIN_MINIMUM_LINE_SIZE);
        assert_eq!(goban.axis_maximum_line_size(Player::Human, Direction::NorthEast), WIN_MINIMUM_LINE_SIZE);
        assert_eq!(goban.is_won(Player::Human), true);
    }
}
