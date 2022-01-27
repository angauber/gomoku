use std::{cmp, isize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::goban::{Cell, Eval, Goban, GOBAN_SIZE, Player, Position};

pub enum GameState {
    InProgress,
    Won(Player),
}

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
    pub fn print_board(&self) {
        println!("{:?}", self.goban);
    }

    pub fn play(&mut self, position: Position, player: Player) -> Result<GameState, &str> {
        if position.row >= GOBAN_SIZE ||
            position.col >= GOBAN_SIZE ||
            self.goban.get(position.row, position.col) != Cell::Empty {
            return Err("Invalid move");
        }

        self.goban.set(position.row, position.col, match player {
            Player::Human => Cell::Human,
            Player::Computer => Cell::Computer,
        });

        Ok(self.game_state())
    }

    pub fn play_computer_move(&mut self, depth: usize) -> GameState {
        if depth < 2 {
            panic!("depth search cannot be less than 2")
        }

        let moves = Arc::new(Mutex::new(HashMap::new()));
        let mut handles = vec![];

        for possible_move in self.goban.get_possible_moves() {
            let moves = Arc::clone(&moves);
            let mut initial_node = self.goban.clone();

            let handle = thread::spawn(move || {
                initial_node.set(possible_move.row, possible_move.col, Cell::Computer);

                let eval = Self::minimax(
                    &initial_node,
                    depth - 2,
                    isize::MIN,
                    isize::MAX,
                    false
                );

                moves.lock().unwrap().insert(possible_move, eval);
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let moves = Arc::try_unwrap(moves).unwrap().into_inner().unwrap();

        if let Some(move_to_play) = Self::get_best_move(moves) {
            match self.play(move_to_play, Player::Computer) {
                Ok(state) => state,
                Err(_) => panic!("Invalid move found"),
            }
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
            // skip this iteration if position closest non empty cell is greater than n
            // might we use dilation of some sort ?
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
        match node.evaluate() {
            Eval::Won => return isize::MAX,
            Eval::Lost => return isize::MIN,
            Eval::Score(n) if depth == 0 => return n,
            _ => (),
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

    fn game_state(&self) -> GameState {
        if self.goban.is_won(Player::Computer) {
            return GameState::Won(Player::Computer);
        }

        if self.goban.is_won(Player::Human) {
            return GameState::Won(Player::Human);
        }

        GameState::InProgress
    }
}
