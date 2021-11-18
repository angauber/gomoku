use crate::goban::{Cell, Goban, GOBAN_SIZE, Player, Position};
use std::cmp;

pub struct Gomoku {
    goban: Goban,
}

impl Gomoku {
    pub fn new() -> Gomoku {
        Gomoku {
            goban: Goban::new(),
        }
    }

    pub fn play(&mut self, position: Position, player: Player) -> bool {
        if (position.row >= GOBAN_SIZE || position.col >= GOBAN_SIZE || self.goban.get(position.row, position.col) != Cell::Empty) {
            return false;
        }

        self.goban.set(position.row, position.col, match player {
            Player::Human => Cell::Human,
            Player::Computer => Cell::Computer,
        });

        true
    }

    pub fn play_computer_move(&mut self) {
        let mut best_move: Option<Position> = None;
        let mut best_value: Option<i32> = None;
        let mut value: i32;

        // println!("{:?}", self.goban.get_possible_moves());

        for possible_move in self.goban.get_possible_moves() {
            // println!("{:?}", possible_move);

            self.goban.
            let mut initial_node = self.goban.clone();

            initial_node.set(possible_move.row, possible_move.col, Cell::Computer);

            value = self.minmax(initial_node, 1, false);

            if (best_move.is_none() && best_value.is_none()) || value > best_value.unwrap() {
                best_move = Some(possible_move);
                best_value = Some(value);
            }
        }

        if (best_move.is_none()) {
            println!("ALERT NO MOVE FOUND");
        } else {
            self.play(best_move.unwrap(), Player::Computer);
        }
    }

    fn minmax(&self, node: Goban, depth: usize, maximizing: bool) -> i32 {
        if depth == 0 || node.is_won(Player::Computer) {
            return node.compute_heuristic(Player::Computer) as i32;
        }

        let mut res: i32 = if maximizing { i32::MIN } else { i32::MAX };

        for position in node.get_possible_moves() {
            let mut next_node = node.clone();

            next_node.set(position.row, position.col, if maximizing {Cell::Computer} else {Cell::Human});

            res = match maximizing {
                true => cmp::max(res, self.minmax(node.clone(), depth - 1, !maximizing)),
                false => cmp::min(res, self.minmax(node.clone(), depth - 1, !maximizing)),
            }
        }

        return res;
    }

    pub fn print_board(&self) {
        println!("{:?}", self.goban);
    }
}
