mod goban;
mod gomoku;

use crate::gomoku::Gomoku;
use crate::goban::{Axis, Cell, Goban, Player, Position, WIN_MINIMUM_LINE_SIZE};
use std::io;

fn get_human_move(input: String) -> Option<Position> {
    let tokens: Vec<&str> = input.trim().split(" ").collect();
    let row;
    let col;

    if tokens.len() != 2 {
        return None;
    }

    row = tokens[0].parse();
    col = tokens[1].parse();

    if row.is_err() || col.is_err() {
        return None;
    }

    Some(Position {
        row: row.unwrap(),
        col: col.unwrap(),
    })
}

fn main() {
    let mut gomoku: Gomoku = Gomoku::new();

    gomoku.play_computer_move();

    return ;

    // while true {
    //     let mut input = String::new();
    //
    //     println!("Input: row col");
    //     io::stdin().read_line(&mut input);
    //
    //     let human_move = get_human_move(input);
    //
    //     if human_move.is_none() {
    //         println!("Invalid move");
    //     } else {
    //         let position = human_move.unwrap();
    //
    //         if gomoku.play(position, Player::Human) {
    //             gomoku.play_computer_move();
    //
    //             gomoku.print_board();
    //         } else {
    //             println!("Forbidden move");
    //         }
    //     }
    // }
}
