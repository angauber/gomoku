use std::{isize};
use std::cmp::{max, min, Ordering};
use std::collections::{BinaryHeap, HashMap};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::goban::{Cell, Eval, Goban, GOBAN_SIZE, Player, Position};

const BRANCHING_FACTOR_THRESHOLD: usize = 15;

pub enum GameState {
    InProgress,
    Won(Player),
}

pub struct Gomoku {
    goban: Goban,
}

#[derive(Eq, PartialEq)]
pub struct NodeScore {
    node: Goban,
    score: isize,
}

impl Ord for NodeScore {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score)
    }
}

impl PartialOrd for NodeScore {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl NodeScore {
    pub fn new(node: Goban, score: isize) -> Self {
        Self {
            node,
            score,
        }
    }
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

        for possible_move in self.goban.get_limited_moves(2) {
            let moves = Arc::clone(&moves);
            let mut initial_node = self.goban.clone();

            let handle = thread::spawn(move || {
                initial_node.set(possible_move.row, possible_move.col, Cell::Computer);

                let score = Self::minimax(
                    &initial_node,
                    depth - 1,
                    isize::MIN,
                    isize::MAX,
                    false,
                );

                moves.lock().unwrap().insert(possible_move, score);
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let moves = Arc::try_unwrap(moves).unwrap().into_inner().unwrap();

        if let Some(move_to_play) = Self::get_best_move(moves) {
            let state= match self.play(move_to_play, Player::Computer) {
                Ok(state) => state,
                Err(_) => panic!("Invalid move found"),
            };

            println!("Final board eval: {}", Self::evaluate(&self.goban));

            state
        } else {
            panic!("No move found");
        }
    }

    fn get_best_move(moves: HashMap<Position, isize>) -> Option<Position> {
        moves.into_iter()
            .max_by(|a, b| a.1.cmp(&b.1))
            .map(|(k, _v)| k)
    }

    fn get_child_nodes(node: &Goban, player: Player) -> Vec<NodeScore> {
        let mut child_nodes: BinaryHeap<NodeScore> = BinaryHeap::new();

        // limiting the BF by selecting close neighbours only for now
        for position in node.get_limited_moves(1) {
            let mut child = node.clone();

            child.set(position.row, position.col, match player {
                Player::Computer => Cell::Computer,
                Player::Human => Cell::Human
            });

            // TODO custom eval
            let eval = 0;

            child_nodes.push(NodeScore::new(child, eval));
        }

        // child_nodes.into_iter_sorted().take(BRANCHING_FACTOR_THRESHOLD).collect()
        child_nodes.into_iter_sorted().collect()
    }

    fn evaluate(node: &Goban) -> isize {
        match node.evaluate() {
            Eval::Won => isize::MAX,
            Eval::Lost => isize::MIN,
            Eval::Score(n) => n,
        }
    }

    fn minimax(node: &Goban, depth: usize, mut alpha: isize, mut beta: isize, maximizing: bool) -> isize {
        match node.evaluate() {
            Eval::Won => return isize::MAX,
            Eval::Lost => return isize::MIN,
            Eval::Score(n) if depth == 0 => return n,
            _ => (),
        }

        let mut best;

        if maximizing {
            best = isize::MIN;

            for child in Self::get_child_nodes(node, Player::Computer) {
                best = max(best, Self::minimax(&child.node, depth - 1, alpha, beta, false));

                if best >= beta {
                    break ;
                }

                alpha = max(alpha, best);
            }
        } else {
            best = isize::MAX;

            for child in Self::get_child_nodes(node, Player::Human) {
                best = min(best, Self::minimax(&child.node, depth - 1, alpha, beta, true));

                if best <= alpha {
                    break ;
                }

                beta = min(beta, best);
            }
        }

        best
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
