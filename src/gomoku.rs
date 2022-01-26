use std::{cmp, isize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::goban::{Cell, Goban, GOBAN_SIZE, Player, Position};

pub struct Gomoku {
    goban: Goban,
}

impl Default for Gomoku {
    fn default() -> Self {
        Self {
            goban: Goban::new(),
        }
    }
}

impl Gomoku {
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

                let eval = Self::minimax(
                    &initial_node,
                    match depth > 0 {
                        true => depth - 1,
                        false => depth,
                    },
                    isize::MIN,
                    isize::MAX,
                    false);

                moves.lock().unwrap().insert(possible_move, eval);
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

    fn get_best_move(moves: HashMap<Position, isize>) -> Option<Position> {
        moves.into_iter()
            .max_by(|a, b| a.1.cmp(&b.1))
            .map(|(k, _v)| k)
    }

    fn get_child_nodes(node: &Goban, player: Player) -> Vec<Goban>
    {
        let mut child_nodes = Vec::new();

        for position in node.get_possible_moves() {
            let mut child = node.clone();

            child.set(position.row, position.col, match player {
                Player::Computer => Cell::Computer,
                Player::Human => Cell::Human
            });

            child_nodes.push(child);
        }

        child_nodes
    }

    fn minimax(node: &Goban, depth: usize, mut alpha: isize, mut beta: isize, maximizing: bool) -> isize {
        if node.is_won(Player::Computer) {
            return isize::MAX;
        }
        if node.is_won(Player::Human) {
            return isize::MIN;
        }
        if depth == 0 {
            return node.evaluate();
        }

        let mut result = match maximizing {
            true => isize::MIN,
            false => isize::MAX,
        };

        for child in Self::get_child_nodes(node, match maximizing {
            true => Player::Computer,
            false => Player::Human
        }) {
            let eval = Self::minimax(&child, depth - 1, alpha, beta, !maximizing);

            result = match maximizing {
                true => cmp::max(result, eval),
                false => cmp::min(result, eval),
            };

            let pruning = match maximizing {
                true => result >= beta,
                false => alpha >= result,
            };

            if pruning {
                return result;
            }

            match maximizing {
                true => alpha = cmp::max(alpha, result),
                false => beta = cmp::min(beta, result),
            };
        }

        result
    }

    pub fn print_board(&self) {
        println!("{:?}", self.goban);
    }
}
