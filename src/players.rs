use crate::{Card::*, Game, Move, MyRng, Turn};
pub use naive::NaivePlayer;
mod naive;

pub trait Player {
    fn make_move(&self, state: &Game, turn: &Turn, rng: &mut MyRng) -> Move {
        let card = state.card();
        match card {
            Bonus(num) => self.bonus_strat(num, state, turn, rng),
            Double => self.double_strat(state, turn, rng),
            FireWork => self.fire_work_strat(state, turn, rng),
            Flush => self.flush_strat(state, turn, rng),
            Clover => self.tutto_strat(state, turn, rng),
            PlusMinus => self.plus_minus_strat(state, turn, rng),
            Stop => unreachable!(),
        }
    }

    fn tutto_strat(&self, state: &Game, turn: &Turn, rng: &mut MyRng) -> Move;
    fn bonus_strat(&self, num: i32, state: &Game, turn: &Turn, rng: &mut MyRng) -> Move;
    fn double_strat(&self, state: &Game, turn: &Turn, rng: &mut MyRng) -> Move;
    fn fire_work_strat(&self, state: &Game, turn: &Turn, rng: &mut MyRng) -> Move;
    fn flush_strat(&self, state: &Game, turn: &Turn, rng: &mut MyRng) -> Move;
    fn plus_minus_strat(&self, state: &Game, turn: &Turn, rng: &mut MyRng) -> Move;

    fn card_strat(&self, state: &Game, last_turn: &Turn, rng: &mut MyRng) -> bool;
}
