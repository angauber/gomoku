use crate::goban::{Cell, Goban, GOBAN_SIZE, Player, Position};
use std::cmp;
use std::collections::HashMap;
use std::thread;
use std::sync::{Arc, Mutex};

pub struct Gomoku {
    goban: Goban,
}

impl Default for Gomoku {
    fn default() -> Self {
        Self::new()
    }
}

impl Gomoku {
    pub fn new() -> Gomoku {
        Gomoku {
            goban: Goban::new(),
        }
    }

    pub fn play(&mut self, position: Position, player: Player) -> bool {
        if position.row >= GOBAN_SIZE ||
            position.col >= GOBAN_SIZE ||
            self.goban.get(position.row, position.col) != Cell::Empty {
            return false;
        }

        self.goban.set(position.row, position.col, match player {
            Player::Human => Cell::Human,
            Player::Computer => Cell::Computer,
        });

        true
    }

    pub fn play_computer_move(&mut self, depth: usize) {
        let moves = Arc::new(Mutex::new(HashMap::new()));
        let mut handles = vec![];

        for possible_move in self.goban.get_possible_moves() {
            let moves = Arc::clone(&moves);
            let mut initial_node = self.goban.clone();

            let handle = thread::spawn(move || {
                initial_node.set(possible_move.row, possible_move.col, Cell::Computer);

                let value = Self::minmax(
                    initial_node,
                    match depth > 0 {
                        true => depth - 1,
                        false => depth,
                    },
                    false);

                moves.lock().unwrap().insert(possible_move, value);
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let moves = Arc::try_unwrap(moves).unwrap().into_inner().unwrap();

        if let Some(move_to_play) = Self::get_best_move(moves) {
            self.play(move_to_play, Player::Computer);
        } else {
            panic!("No move found");
        }
    }

    fn get_best_move(moves: HashMap<Position, i32>) -> Option<Position> {
        moves.into_iter()
            .max_by(|a, b| a.1.cmp(&b.1))
            .map(|(k, _v)| k)
    }

    fn minmax(node: Goban, depth: usize, maximizing: bool) -> i32 {
        if node.is_won(Player::Computer) {
            return i32::MAX;
        }
        if node.is_won(Player::Human) {
            return i32::MIN;
        }
        if depth == 0 {
            return node.evaluate();
        }

        let mut res: i32 = if maximizing { i32::MIN } else { i32::MAX };

        for position in node.get_possible_moves() {
            let mut next_node = node.clone();

            next_node.set(position.row, position.col, if maximizing {Cell::Computer} else {Cell::Human});

            res = match maximizing {
                true => cmp::max(res, Self::minmax(next_node.clone(), depth - 1, !maximizing)),
                false => cmp::min(res, Self::minmax(next_node.clone(), depth - 1, !maximizing)),
            }
        }

        res
    }

    pub fn print_board(&self) {
        println!("{:?}", self.goban);
    }
}
