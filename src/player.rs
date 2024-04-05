#![allow(unused_variables)]
use crate::{Game, Move, MyRng, Player, Take, Turn};
pub struct NaivePlayer;

fn catergorize_roll(roll: &[u8]) -> Vec<Take> {
    if roll.len() < 3 {
        let mut takes = Vec::new();
        for (i, dice) in roll.iter().enumerate() {
            if *dice == 5 {
                takes.push(Take::Single(i))
            }
        }
        for (i, dice) in roll.iter().enumerate() {
            if *dice == 1 {
                takes.push(Take::Single(i))
            }
        }
        takes
    } else {
        let mut taken_idxs = Vec::new();
        let mut takes = Vec::new();
        for i in 2..=6 {
            let mut triplets = search_triplet(i, roll);
            for chunk in triplets.chunks(3) {
                takes.push(Take::Triple(chunk[0], chunk[1], chunk[2]))
            }
            taken_idxs.append(&mut triplets);
        }
        for (i, dice) in roll.iter().enumerate() {
            if *dice == 5 && !taken_idxs.contains(&i) {
                takes.push(Take::Single(i))
            }
        }
        for (i, dice) in roll.iter().enumerate() {
            if *dice == 1 && !taken_idxs.contains(&i) {
                takes.push(Take::Single(i))
            }
        }
        takes
    }
}

fn search_triplet(num: u8, list: &[u8]) -> Vec<usize> {
    let mut incomplete = Vec::new();
    let mut out = Vec::new();
    for (i, val) in list.iter().enumerate() {
        if *val == num {
            incomplete.push(i);
            if incomplete.len() == 3 {
                out.append(&mut incomplete);
            }
        }
    }
    out
}

impl Player for NaivePlayer {
    fn tutto_strat(&self, state: &Game, turn: &Turn, roll: &[u8], rng: &mut MyRng) -> Move {
        todo!()
    }

    fn bonus_strat(
        &self,
        num: i32,
        state: &Game,
        turn: &Turn,
        roll: &[u8],
        rng: &mut MyRng,
    ) -> Move {
        todo!()
    }

    fn double_strat(&self, state: &Game, turn: &Turn, roll: &[u8], rng: &mut MyRng) -> Move {
        todo!()
    }

    fn fire_work_strat(&self, state: &Game, turn: &Turn, roll: &[u8], rng: &mut MyRng) -> Move {
        Move {
            takes: catergorize_roll(roll),
            write: false,
        }
    }

    fn street_strat(&self, state: &Game, turn: &Turn, roll: &[u8], rng: &mut MyRng) -> Move {
        todo!()
    }

    fn plus_minus_strat(&self, state: &Game, turn: &Turn, roll: &[u8], rng: &mut MyRng) -> Move {
        todo!()
    }

    fn card_strat(&self, state: &Game, last_turn: &Turn, rng: &mut MyRng) -> bool {
        todo!()
    }
}
