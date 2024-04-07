use std::{
    io::{stdin, stdout, Write},
    usize,
};

use crate::{Game, Move, MyRng, Player, Turn};

pub struct CliPlayer;

impl Player for CliPlayer {
    fn make_move(&self, state: &Game, turn: &Turn, _rng: &mut MyRng) -> Move {
        println!("------------------------------------------------");
        println!("{}", state.get_cli_header());
        println!("{}", turn.cli_output());
        println!("which indexes should be taken?");
        println!(
            "use 0 to {} sperated by spaces to select",
            turn.categorize_roll().len() - 1
        );
        println!("use \'all\' to select all");
        println!("end with ! to take the points");
        loop {
            let mut write = false;
            let mut buffer = match get_user_input() {
                Ok(s) => s,
                Err(_) => {
                    println!("could not read stdin");
                    continue;
                }
            };
            if buffer.ends_with("!") {
                buffer.pop();
                write = true;
            }
            if buffer == "all" {
                println!("OK");
                println!("\n");
                return Move {
                    takes: turn.categorize_roll(),
                    write,
                };
            }
            let mut numbers: Vec<usize> =
                if let Ok(vec) = buffer.split(' ').map(|s| s.parse::<usize>()).collect() {
                    vec
                } else {
                    println!("invalid could not parse to integer");
                    continue;
                };
            let possible_takes = turn.categorize_roll();
            numbers.dedup();
            let takes: Vec<_> = numbers
                .into_iter()
                .filter(|&x| x < possible_takes.len())
                .map(|x| possible_takes[x])
                .collect();
            if takes.is_empty() {
                println!("takes was empty");
                continue;
            }
            return Move { takes, write };
        }
    }

    fn card_strat(&self, _state: &Game, last_turn: &Turn, _rng: &mut MyRng) -> bool {
        println!("------------------------------------------------");
        println!("current points: {}", last_turn.previous_cards_total);
        println!("do you want to take a new card? y/[n]");
        loop {
            let buffer = match get_user_input() {
                Ok(s) => s,
                Err(_) => {
                    println!("could not read stdin");
                    continue;
                }
            };

            if buffer.is_empty() {
                return false;
            }
            if buffer == "y" {
                return true;
            }
            if buffer == "n" {
                return true;
            }
            println!("invalid try again!");
        }
    }
}

fn get_user_input() -> std::io::Result<String> {
    stdout().flush()?;
    let mut buffer = String::new();
    stdin().read_line(&mut buffer)?;
    if buffer.ends_with("\n") {
        buffer.pop();
    }
    if buffer.ends_with("\r") {
        buffer.pop();
    }
    Ok(buffer)
}
