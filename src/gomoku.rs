use std::cmp::{max, min, Ordering};
use std::collections::{BinaryHeap, HashMap};
use std::collections::btree_map::Entry;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::isize;
use std::rc::Rc;

use crate::evaluator::Eval;
use crate::goban::{Goban, GOBAN_SIZE, Move, Player, Position, Stone};
use crate::threat_evaluator::ThreatEvaluator;
use crate::transposition_table::TranspositionTable;
use crate::zobrist_hashing::ZobristHasher;

const BRANCHING_FACTOR_THRESHOLD: usize = 10;

pub enum GameState {
    InProgress,
    Won(Player),
}

pub struct Gomoku {
    goban: Goban,
    evaluator: ThreatEvaluator,
    transposition_table: TranspositionTable,
    zobrist_hasher: Rc<ZobristHasher>,
    visited_nodes: usize,
    evaluated_nodes: usize,
    evaluated_nodes_hit: usize,
    evaluated_nodes_miss: usize,
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
        let hasher = Rc::new(ZobristHasher::initialize());

        Self {
            goban: Goban::new(Rc::clone(&hasher)),
            evaluator: ThreatEvaluator::new(),
            transposition_table: TranspositionTable::new(),
            zobrist_hasher: hasher,
            visited_nodes: 0,
            evaluated_nodes: 0,
            evaluated_nodes_hit: 0,
            evaluated_nodes_miss: 0,
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
            || self.goban.get(position.row, position.col) != None
        {
            return Err("Invalid move");
        }

        let stone = match player {
            Player::Opponent => Stone::Black,
            Player::Computer => Stone::White,
        };

        self.goban.apply_move(Move::new(stone, position));

        Ok(self.game_state())
    }

    pub fn play_computer_move(&mut self, depth: usize) -> GameState {
        if depth < 2 {
            panic!("depth search cannot be less than 2")
        }

        if depth % 2 != 0 {
            panic!("depth search cannot be odd")
        }

        let computer_eval = self.eval_current(Player::Computer);

        println!("computer score: {:?}", computer_eval);

        let mut moves = HashMap::new();

        self.visited_nodes = 0;
        self.evaluated_nodes = 0;
        self.evaluated_nodes_hit = 0;
        self.evaluated_nodes_miss = 0;

        for child in self.get_child_nodes(&self.goban.clone(), Player::Computer) {
            let score = self.minimax(&child.node, depth - 1, isize::MIN, isize::MAX, false);

            moves.insert(child.position, score);
        }

        // println!("{:?}", moves);

        if let Some(move_to_play) = Self::get_best_move(moves) {
            println!("move to play: {:?}", move_to_play);

            let state = match self.play(move_to_play, Player::Computer) {
                Ok(state) => state,
                Err(_) => panic!("Invalid move found"),
            };

            println!("visited {} nodes", self.visited_nodes);
            println!("evaluated {} nodes (cache hit {}, cache miss: {})", self.evaluated_nodes, self.evaluated_nodes_hit, self.evaluated_nodes_miss);

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

        for position in node.get_limited_moves(2) {
            let mut child = node.clone();

            let stone = match player {
                Player::Computer => Stone::White,
                Player::Opponent => Stone::Black,
            };

            child.apply_move(Move::new(stone, position.clone()));

            // We should use a custom evaluation function for this
            // With this solution we will miss winning / losing nodes
            // one idea: include only move that create threat or block some
            let eval = self.eval(&mut child, player);

            let score_eval = match eval {
                Eval::Won => isize::MAX,
                Eval::Lost => isize::MIN,
                Eval::Score(n) => n,
            };

            child_nodes.push(NodeScore::new(child, position, score_eval));
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
        let side = match maximizing {
            true => Player::Computer,
            false => Player::Opponent,
        };

        self.visited_nodes += 1;

        match self.eval(&node, side) {
            Eval::Won => return if maximizing { isize::MAX } else { isize::MIN },
            Eval::Lost => return if maximizing { isize::MIN } else { isize::MAX },
            Eval::Score(n) if depth == 0 => return n as isize * if maximizing { 1 } else { -1 },
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
        let eval = self.eval_current(Player::Computer);

        match eval {
            Eval::Won => GameState::Won(Player::Computer),
            Eval::Lost => GameState::Won(Player::Opponent),
            Eval::Score(_) => GameState::InProgress
        }
    }

    fn eval(&mut self, goban: &Goban, player: Player) -> Eval
    {
        // self.transposition_table
        //     .entry((goban.get_hash(), player))
        //     .or_insert_with(|| goban.evaluate(&mut self.evaluator, player))
        //     .to_owned()

        self.evaluated_nodes += 1;

        let key = (goban.get_hash(), player);
        let entry = self.transposition_table.get(&key);

        match entry {
            Some(eval) => {
                self.evaluated_nodes_hit += 1;
                eval.to_owned()
            },
            None => {
                self.evaluated_nodes_miss += 1;

                let eval = goban.evaluate(&mut self.evaluator, player);

                self.transposition_table.insert(key, eval.clone());

                eval
            }
        }
    }

    fn eval_current(&mut self, player: Player) -> Eval
    {
        self.eval(&self.goban.clone(), player)
    }
}
