#![allow(unused_variables)]
use crate::{Game, Move, MyRng, Player, Turn};
pub struct NaivePlayer;

impl Player for NaivePlayer {
    fn tutto_strat(&self, state: &Game, turn: &Turn, rng: &mut MyRng) -> Move {
        Move {
            takes: turn.catergorize_roll(),
            write: false,
        }
    }

    fn bonus_strat(&self, num: i32, state: &Game, turn: &Turn, rng: &mut MyRng) -> Move {
        Move {
            takes: turn.catergorize_roll(),
            write: false,
        }
    }

    fn double_strat(&self, state: &Game, turn: &Turn, rng: &mut MyRng) -> Move {
        Move {
            takes: turn.catergorize_roll(),
            write: false,
        }
    }

    fn fire_work_strat(&self, state: &Game, turn: &Turn, rng: &mut MyRng) -> Move {
        Move {
            takes: turn.catergorize_roll(),
            write: false,
        }
    }

    fn flush_strat(&self, state: &Game, turn: &Turn, rng: &mut MyRng) -> Move {
        todo!()
    }

    fn plus_minus_strat(&self, state: &Game, turn: &Turn, rng: &mut MyRng) -> Move {
        Move {
            takes: turn.catergorize_roll(),
            write: false,
        }
    }

    fn card_strat(&self, state: &Game, last_turn: &Turn, rng: &mut MyRng) -> bool {
        true
    }
}
