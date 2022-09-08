use std::io;
use std::time::Instant;
use clap::Parser;

use gomoku::goban::{Player, Position};
use gomoku::gomoku::{GameState, Gomoku};

fn display_win(gomoku: Gomoku, player: Player) {
    gomoku.print_board();

    println!(
        "{} Won !",
        match player {
            Player::Opponent => "You",
            Player::Computer => "Computer",
        }
    );
}

#[derive(Parser, Debug)]
#[clap(name = "Gomoku")]
#[clap(author = "angauber")]
struct Args {
    /// minmax search tree depth
    #[clap(short, long, value_parser, default_value_t = 4)]
    search_depth: usize,
}

fn main() {
    let args = Args::parse();
    let mut gomoku = Gomoku::default();

    gomoku.print_board();

    loop {
        let mut input = String::new();

        println!("Input: col row");
        io::stdin()
            .read_line(&mut input)
            .expect("Could not read from stdin");

        match Position::from_coordinates(input.trim()) {
            Ok(position) => {
                let result = gomoku.play(position, Player::Opponent);

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

                let state = gomoku.play_computer_move(args.search_depth);
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
