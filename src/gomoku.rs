use std::cmp::{max, min, Ordering};
use std::collections::{BinaryHeap, HashMap};
use std::isize;

use crate::evaluator::Eval;
use crate::goban::{Cell, Goban, Player, Position, GOBAN_SIZE};
use crate::threat_evaluator::ThreatEvaluator;

const BRANCHING_FACTOR_THRESHOLD: usize = 10;

pub enum GameState {
    InProgress,
    Won(Player),
}

pub struct Gomoku {
    goban: Goban,
    evaluator: ThreatEvaluator,
}

#[derive(Eq, PartialEq)]
pub struct NodeScore {
    node: Goban,
    position: Position,
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
    pub fn new(node: Goban, position: Position, score: isize) -> Self {
        Self {
            node,
            position,
            score,
        }
    }
}

impl Default for Gomoku {
    fn default() -> Self {
        Self {
            goban: Goban::new(),
            evaluator: ThreatEvaluator::new(),
        }
    }
}

impl Gomoku {
    pub fn print_board(&self) {
        println!("{:?}", self.goban);
    }

    pub fn play(&mut self, position: Position, player: Player) -> Result<GameState, &str> {
        if position.row >= GOBAN_SIZE
            || position.col >= GOBAN_SIZE
            || self.goban.get(position.row, position.col) != Cell::Empty
        {
            return Err("Invalid move");
        }

        self.goban.set(
            position.row,
            position.col,
            match player {
                Player::Opponent => Cell::Opponent,
                Player::Computer => Cell::Computer,
            },
        );

        Ok(self.game_state())
    }

    pub fn play_computer_move(&mut self, depth: usize) -> GameState {
        if depth < 2 {
            panic!("depth search cannot be less than 2")
        }

        if depth % 2 != 0 {
            panic!("depth search cannot be odd")
        }

        self.goban.evaluate(&mut self.evaluator, Player::Opponent);
        println!("player score: {:?}", self.goban.eval());

        self.goban.evaluate(&mut self.evaluator, Player::Computer);
        println!("computer score: {:?}", self.goban.eval());

        let mut moves = HashMap::new();

        for child in self.get_child_nodes(&self.goban.clone(), Player::Computer) {
            let score = self.minimax(&child.node, depth - 1, isize::MIN, isize::MAX, false);

            moves.insert(child.position, score);
        }

        println!("{:?}", moves);

        if let Some(move_to_play) = Self::get_best_move(moves) {
            println!("move to play: {:?}", move_to_play);

            let state = match self.play(move_to_play, Player::Computer) {
                Ok(state) => state,
                Err(_) => panic!("Invalid move found"),
            };

            state
        } else {
            panic!("No move found");
        }
    }

    fn get_best_move(moves: HashMap<Position, isize>) -> Option<Position> {
        moves
            .into_iter()
            .max_by(|a, b| a.1.cmp(&b.1))
            .map(|(k, _v)| k)
    }

    fn get_child_nodes(&mut self, node: &Goban, player: Player) -> Vec<NodeScore> {
        let mut child_nodes: BinaryHeap<NodeScore> = BinaryHeap::new();

        for position in node.get_possible_moves() {
            let mut child = node.clone();

            child.set(
                position.row,
                position.col,
                match player {
                    Player::Computer => Cell::Computer,
                    Player::Opponent => Cell::Opponent,
                },
            );

            child.evaluate(&mut self.evaluator, player);

            let eval = match child.eval().unwrap() {
                Eval::Won => isize::MAX,
                Eval::Lost => isize::MIN,
                Eval::Score(n) => n,
            };

            child_nodes.push(NodeScore::new(child, position, eval));
        }

        child_nodes
            .into_iter_sorted()
            .take(BRANCHING_FACTOR_THRESHOLD)
            .collect()
    }

    pub fn minimax(
        &mut self,
        node: &Goban,
        depth: usize,
        mut alpha: isize,
        mut beta: isize,
        maximizing: bool,
    ) -> isize {
        match node.eval().unwrap() {
            Eval::Won => return if maximizing { isize::MIN } else { isize::MAX },
            Eval::Lost => return if maximizing { isize::MAX } else { isize::MIN },
            Eval::Score(n) if depth == 0 => return n as isize * if maximizing { -1 } else { 1 },
            _ => {}
        };

        let mut best;

        if maximizing {
            best = isize::MIN;

            for child in self.get_child_nodes(node, Player::Computer) {
                best = max(
                    best,
                    self.minimax(&child.node, depth - 1, alpha, beta, false),
                );

                if best >= beta {
                    break;
                }

                alpha = max(alpha, best);
            }
        } else {
            best = isize::MAX;

            for child in self.get_child_nodes(node, Player::Opponent) {
                best = min(
                    best,
                    self.minimax(&child.node, depth - 1, alpha, beta, true),
                );

                if best <= alpha {
                    break;
                }

                beta = min(beta, best);
            }
        }

        best
    }

    fn game_state(&mut self) -> GameState {
        self.goban.evaluate(&mut self.evaluator, Player::Computer);

        match self.goban.eval().unwrap() {
            Eval::Won => GameState::Won(Player::Computer),
            Eval::Lost => GameState::Won(Player::Opponent),
            Eval::Score(_) => GameState::InProgress
        }
    }
}
