use rand::prelude::*;

use crate::goban::{GOBAN_SIZE, GOBAN_TOTAL_SIZE, Move, Stone};

pub type ZobristHash = u64;

pub const INITIAL_BOARD_HASH: ZobristHash = 0;

// Either Black or White
const STONE_COLORS: usize = 2;

type ZobristTable = [[u64; STONE_COLORS]; GOBAN_TOTAL_SIZE];

pub struct ZobristHasher {
    table: ZobristTable,
}

impl ZobristHasher {
    pub fn initialize() -> Self {
        Self {
            table: initialize_zobrist_table()
        }
    }

    pub fn update_hash(&self, hash: u64, played_move: Move) -> ZobristHash {
        let stone_index = match played_move.stone {
            Stone::Black => 0,
            Stone::White => 1,
        };

        let position_index = (played_move.position.row * GOBAN_SIZE) + played_move.position.col;

        hash ^ self.table[position_index][stone_index] as u64
    }
}

fn initialize_zobrist_table() -> ZobristTable {
    let mut table: ZobristTable = [[0; STONE_COLORS]; GOBAN_TOTAL_SIZE];

    for stone in table.iter_mut() {
        for color in stone.iter_mut() {
            *color = random();
        }
    }

    table
}

#[cfg(test)]
mod zobrist_hash_tests {
    use std::rc::Rc;
    use crate::goban::{Goban, Move, Position, Stone};
    use crate::zobrist_hashing::{ZobristHash, ZobristHasher};

    fn compute_hash_from_moves(zobrist: Rc<ZobristHasher>, moves: Vec<Move>) -> ZobristHash {
        let mut goban = Goban::new(zobrist);

        moves.into_iter().for_each(|m| goban.apply_move(m));

        goban.get_hash()
    }

    #[test]
    fn it_correctly_hash_a_goban() {
        let zobrist_hasher = Rc::new(ZobristHasher::initialize());

        let ordered_moves = vec![
            Move::new(Stone::Black, Position::new(5, 5)),
            Move::new(Stone::White, Position::new(5, 6)),
            Move::new(Stone::Black, Position::new(5, 7)),
            Move::new(Stone::White, Position::new(5, 8)),
        ];

        let reversed_moves = vec![
            Move::new(Stone::Black, Position::new(5, 7)),
            Move::new(Stone::White, Position::new(5, 6)),
            Move::new(Stone::Black, Position::new(5, 5)),
            Move::new(Stone::White, Position::new(5, 8)),
        ];

        let ordered_hash = compute_hash_from_moves(Rc::clone(&zobrist_hasher), ordered_moves);
        let reversed_hash = compute_hash_from_moves(Rc::clone(&zobrist_hasher), reversed_moves);

        assert_eq!(ordered_hash, reversed_hash);
    }
}