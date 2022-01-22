use std::fmt;
use std::cmp::{PartialEq,Eq};
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

#[derive(Clone)]
pub struct Goban {
    player: Bitboard,
    computer: Bitboard,
}

#[derive(EnumIter, Clone, Copy)]
pub enum Axis {
    Row,
    Col,
    DiagLeft,
    DiagRight,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Position {
    pub row: usize,
    pub col: usize,
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
                    Cell::Empty => "_",
                    Cell::Human => "X",
                    Cell::Computer => "O",
                })?;
            }
            writeln!(f)?;
        }

        writeln!(f)
    }
}

impl Goban {
    pub fn new() -> Goban {
        Goban {
            player: bitarr![Msb0, u8; 0; BIT_SIZE],
            computer: bitarr![Msb0, u8; 0; BIT_SIZE],
        }
    }

    pub fn print_bitboard(bitboard: &Bitboard) {
        for row in 0..GOBAN_SIZE {
            for col in 0..GOBAN_SIZE {
                print!("{}", match bitboard[(row * (GOBAN_SIZE + 1)) + col] {
                    true => 1,
                    false => 0,
                });
            }
            println!();
        }

        println!();
    }

    /**
     * https://en.wikipedia.org/wiki/Erosion_(morphology)
     */
    pub fn erode(bitboard: &mut Bitboard, axis: Axis, length: usize) {
        let mut rhs = *bitboard;

        rhs.shift_right(Goban::get_shift(axis) * length);

        *bitboard &= rhs
    }

    /**
     * Eroding the player bitboard in each axis to find the maximum length
     */
    pub fn maximum_line_size(&self, player: Player, axis: Axis) -> usize {
        let mut size: usize = 0;
        let mut bitboard = match player {
            Player::Human => self.player,
            Player::Computer => self.computer,
        };

        while bitboard.any() {
            Goban::erode(&mut bitboard, axis, 1);
            size += 1;
        }

        size
    }

    pub fn is_won(&self, player: Player) -> bool {
        for axis in Axis::iter() {
            if self.maximum_line_size(player, axis) >= WIN_MINIMUM_LINE_SIZE {
                return true;
            }
        }

        false
    }

    pub fn set(&mut self, row: usize, col: usize, cell: Cell) {
        let index: usize = (row * (GOBAN_SIZE + 1)) + col;

        self.player.set(index, false);
        self.computer.set(index, false);

        match cell {
            Cell::Empty => (),
            Cell::Human => self.player.set(index, true),
            Cell::Computer => self.computer.set(index, true),
        }
    }

    pub fn get(&self, row: usize, col: usize) -> Cell {
        let index: usize = (row * (GOBAN_SIZE + 1)) + col;

        match self.player[index] {
            true => Cell::Human,
            false => match self.computer[index] {
                true => Cell::Computer,
                false => Cell::Empty,
            }
        }
    }

    pub fn evaluate(&self) -> i32 {
        self.compute_heuristic(Player::Computer) as i32 - self.compute_heuristic(Player::Human) as i32
    }

    pub fn compute_heuristic(&self, player: Player) -> usize {
        let mut max: usize = 0;

        /* Could we erode in each axis at the same time ? */
        for axis in Axis::iter() {
            max = std::cmp::max(max, self.maximum_line_size(player, axis));
        }

        max
    }

    pub fn get_possible_moves(&self) -> Vec<Position>
    {
        let mut positions: Vec<Position> = Vec::new();
        let playable = !(self.player | self.computer);

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

    fn get_shift(axis: Axis) -> usize {
        match axis {
            Axis::Row => 1,
            Axis::Col => GOBAN_SIZE + 1,
            Axis::DiagLeft => GOBAN_SIZE,
            Axis::DiagRight => GOBAN_SIZE + 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::goban::{Axis, Cell, Goban, GOBAN_SIZE, Player, WIN_MINIMUM_LINE_SIZE};

    #[test]
    fn it_correctly_detects_win() {
        let mut goban: Goban = Goban::new();

        for i in 0..WIN_MINIMUM_LINE_SIZE {
            goban.set(0, i, Cell::Computer);
        }

        assert_eq!(goban.maximum_line_size(Player::Computer, Axis::Row), WIN_MINIMUM_LINE_SIZE);
        assert_eq!(goban.is_won(Player::Computer), true);

        goban = Goban::new();

        for i in 0..GOBAN_SIZE {
            goban.set(18, 18 - i, Cell::Human);
        }

        goban.set(18, 10, Cell::Computer);


        assert_eq!(goban.maximum_line_size(Player::Human, Axis::Row), 10);
        assert_eq!(goban.is_won(Player::Human), true);

        goban = Goban::new();

        goban.set(3, 3, Cell::Human);
        goban.set(4, 4, Cell::Human);
        goban.set(5, 5, Cell::Human);
        goban.set(6, 6, Cell::Human);
        goban.set(7, 7, Cell::Human);

        assert_eq!(goban.maximum_line_size(Player::Human, Axis::DiagRight), 5);
        assert_eq!(goban.is_won(Player::Human), true);
    }

    #[test]
    fn it_correctly_erodes_columns() {
        let mut goban: Goban = Goban::new();

        for i in 0..WIN_MINIMUM_LINE_SIZE - 1 {
            goban.set(i, 0, Cell::Computer);
        }

        assert_eq!(goban.maximum_line_size(Player::Computer, Axis::Col), WIN_MINIMUM_LINE_SIZE - 1);
        assert_eq!(goban.is_won(Player::Computer), false);

        goban = Goban::new();

        for i in 0..WIN_MINIMUM_LINE_SIZE {
            goban.set(18 - i, 3, Cell::Human);
        }

        assert_eq!(goban.maximum_line_size(Player::Human, Axis::Col), WIN_MINIMUM_LINE_SIZE);
        assert_eq!(goban.is_won(Player::Human), true);
    }

    #[test]
    fn it_correctly_erodes_diagonally() {
        let mut goban: Goban = Goban::new();

        for i in 0..WIN_MINIMUM_LINE_SIZE + 1 {
            goban.set(i, i, Cell::Computer);
        }

        assert_eq!(goban.maximum_line_size(Player::Computer, Axis::DiagRight), WIN_MINIMUM_LINE_SIZE + 1);
        assert_eq!(goban.is_won(Player::Computer), true);

        goban = Goban::new();

        for i in 0..WIN_MINIMUM_LINE_SIZE {
            goban.set(i, 18 - i, Cell::Human);
        }

        assert_eq!(goban.maximum_line_size(Player::Human, Axis::DiagLeft), WIN_MINIMUM_LINE_SIZE);
        assert_eq!(goban.is_won(Player::Human), true);
    }

    #[test]
    fn it_correctly_compute_heuristic() {
        let mut goban: Goban = Goban::new();

        for i in 0..7 {
            goban.set(0, i, Cell::Computer);
        }

        for i in 0..4 {
            goban.set(i, 12, Cell::Human);
        }

        assert_eq!(goban.compute_heuristic(Player::Computer), 7);
        assert_eq!(goban.compute_heuristic(Player::Human), 4);
    }
}
