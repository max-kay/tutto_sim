#![allow(unused_variables)]
use crate::{Game, Move, MyRng, Player, Turn};

pub struct NaivePlayer;

impl Player for NaivePlayer {
    fn make_move(&self, state: &Game, turn: &Turn, rng: &mut MyRng) -> Move {
        Move {
            takes: turn.categorize_roll(),
            write: false,
        }
    }

    fn card_strat(&self, state: &Game, last_turn: &Turn, rng: &mut MyRng) -> bool {
        true
    }
}
