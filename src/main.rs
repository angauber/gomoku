use std::io;
use std::time::Instant;
use gomoku::gomoku::{GameState, Gomoku};
use gomoku::goban::{Position, Player};

fn get_human_move(input: String) -> Result<Position, &'static str> {
    let tokens: Vec<&str> = input.trim().split(' ').collect();

    if tokens.len() != 2 {
        return Err("Invalid input");
    }

    let row = tokens[0].parse();
    let col = tokens[1].parse();

    if row.is_err() || col.is_err() {
        return Err("Invalid input");
    }

    Ok(Position {
        row: row.unwrap(),
        col: col.unwrap(),
    })
}

fn display_win(gomoku: Gomoku, player: Player) {
    gomoku.print_board();

    println!("{} Won !", match player {
        Player::Human => "You",
        Player::Computer => "Computer",
    });
}

fn main() {
    let mut gomoku = Gomoku::default();

    gomoku.print_board();

    loop {
        let mut input = String::new();

        println!("Input: row col");
        io::stdin().read_line(&mut input).expect("Could not read from stdin");

        match get_human_move(input) {
            Ok(position) => {
                let result = gomoku.play(position, Player::Human);

                if let Err(message) = result {
                    println!("{}", message);
                    gomoku.print_board();
                    continue;
                }
                if let GameState::Won(player) = result.unwrap() {
                    display_win(gomoku, player);
                    break;
                }

                let t0 = Instant::now();

                let state = gomoku.play_computer_move(4);
                println!("Took: {} ms", t0.elapsed().as_millis());
                gomoku.print_board();

                if let GameState::Won(player) = state {
                    display_win(gomoku, player);
                    break;
                }
            }
            Err(message) => println!("{}", message),
        }
    }
}