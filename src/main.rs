use std::io;
use gomoku::gomoku::Gomoku;
use gomoku::goban::{Position, Player};

fn get_human_move(input: String) -> Option<Position> {
    let tokens: Vec<&str> = input.trim().split(' ').collect();

    if tokens.len() != 2 {
        return None;
    }

    let row = tokens[0].parse();
    let col = tokens[1].parse();

    if row.is_err() || col.is_err() {
        return None;
    }

    Some(Position {
        row: row.unwrap(),
        col: col.unwrap(),
    })
}

fn main() {
    let mut gomoku = Gomoku::default();

    loop {
        let mut input = String::new();

        println!("Input: row col");
        io::stdin().read_line(&mut input).expect("Could not read from stdin");

        let human_move = get_human_move(input);

        if let Some(position) = human_move {
            if gomoku.play(position, Player::Human) {
                gomoku.play_computer_move(3);

                gomoku.print_board();
            } else {
                panic!("Forbidden move");
            }
        } else {
            println!("Invalid input");
        }
    }
}