use crate::{Card::*, Game, Move, MyRng, Turn};

mod naive;
pub use naive::NaivePlayer;

mod cli_player;
pub use cli_player::CliPlayer;

pub trait Player {
    fn make_move(&self, state: &Game, turn: &Turn, rng: &mut MyRng) -> Move;
    fn card_strat(&self, state: &Game, last_turn: &Turn, rng: &mut MyRng) -> bool;
}

impl<T> Player for T
where
    T: SplitPlayer,
{
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

    fn card_strat(&self, state: &Game, last_turn: &Turn, rng: &mut MyRng) -> bool {
        <Self as SplitPlayer>::card_strat(&self, state, last_turn, rng)
    }
}

pub trait SplitPlayer {
    fn tutto_strat(&self, state: &Game, turn: &Turn, rng: &mut MyRng) -> Move;
    fn bonus_strat(&self, num: i32, state: &Game, turn: &Turn, rng: &mut MyRng) -> Move;
    fn double_strat(&self, state: &Game, turn: &Turn, rng: &mut MyRng) -> Move;
    fn fire_work_strat(&self, state: &Game, turn: &Turn, rng: &mut MyRng) -> Move;
    fn flush_strat(&self, state: &Game, turn: &Turn, rng: &mut MyRng) -> Move;
    fn plus_minus_strat(&self, state: &Game, turn: &Turn, rng: &mut MyRng) -> Move;

    fn card_strat(&self, state: &Game, last_turn: &Turn, rng: &mut MyRng) -> bool;
}
