use bitvec::prelude::*;
use std::fmt;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub const GOBAN_SIZE: usize = 19;
pub const WIN_MINIMUM_LINE_SIZE: usize = 5;
const BIT_SIZE: usize = self::GOBAN_SIZE * self::GOBAN_SIZE + self::GOBAN_SIZE;

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
    player: BitVec,
    computer: BitVec,
}

#[derive(EnumIter, Clone, Copy)]
pub enum Axis {
    Row,
    Col,
    DiagLeft,
    DiagRight,
}

#[derive(Debug)]
pub struct Position {
    pub row: usize,
    pub col: usize,
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
            write!(f, "\n")?;
        }

        write!(f, "\n")
    }
}

impl Goban {
    pub fn new() -> Goban {
        Goban {
            player: bitvec![0; BIT_SIZE],
            computer: bitvec![0; BIT_SIZE],
        }
    }

    pub fn print_bitboard(bitboard: &BitVec) {
        for row in 0..GOBAN_SIZE {
            for col in 0..GOBAN_SIZE {
                print!("{}", match bitboard[(row * (GOBAN_SIZE + 1)) + col] {
                    false => 0,
                    true => 1,
                });
            }
            print!("\n");
        }

        print!("\n");
    }

    /**
    https://en.wikipedia.org/wiki/Erosion_(morphology)
     */
    pub fn erode(bitboard: BitVec, axis: Axis, length: usize) -> BitVec {
        let mut rhs = bitboard.clone();

        rhs.shift_right(Goban::get_shift(axis) * length);

        bitboard & rhs
    }

    /**
    Eroding the player bitboard in each axis to find the maximum length\
     */
    pub fn maximum_line_size(&self, player: Player, axis: Axis) -> usize {
        let mut bitboard: BitVec = match player {
            Player::Human => self.player.clone(),
            Player::Computer => self.computer.clone(),
        };
        let mut size: usize = 0;

        while bitboard.any() {
            bitboard = Goban::erode(bitboard, axis, 1);
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

    pub fn compute_heuristic(&self, player: Player) -> usize {
        let mut max: usize = 0;

        for axis in Axis::iter() {
            max = std::cmp::max(max, self.maximum_line_size(player, axis));
        }

        max
    }

    pub fn get_possible_moves(&self) -> Vec<Position>
    {
        let mut moves: Vec<Position> = Vec::new();
        let intersection: BitVec = !(self.player.clone() | self.computer.clone());

        for index in intersection.iter_ones() {
            // dirty fix
            if (index / GOBAN_SIZE < GOBAN_SIZE && index % GOBAN_SIZE < GOBAN_SIZE) {
                moves.push(Position { row: index / GOBAN_SIZE, col: index % GOBAN_SIZE });
            }
        }

        moves
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
    use bitvec::prelude::*;

    use crate::goban::{Axis, Cell, Goban, GOBAN_SIZE, Player, WIN_MINIMUM_LINE_SIZE};

    #[test]
    fn it_correctly_detects_row_win() {
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

        println!("{:?}", goban);

        assert_eq!(goban.maximum_line_size(Player::Human, Axis::Row), 10);
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
