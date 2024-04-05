use std::usize;

use player::NaivePlayer;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg as MyRng;
use rand_seeder::Seeder;
use thiserror::Error;

pub mod deck;
pub mod logging;
pub mod player;

pub use deck::{
    Card::{self, *},
    Deck,
};
pub use logging::{CardLog, PlayerLog, TurnLog};

#[derive(Error, Debug, Copy, Clone)]
pub enum RuleError {
    #[error("card is street")]
    CardIsStreet,
    #[error("card is not street")]
    CardNotStreet,
    #[error("illegal take occured")]
    IllegalTake,
    #[error("dice was taken twice")]
    DuplicateDice,
    #[error("triple was invalid")]
    IllegalTriple,
}

const POINT_GOAL: i32 = 10_000;
const NUMBER_OF_DICE: usize = 6;

trait Player {
    fn make_move(&self, state: &Game, turn: &Turn, roll: &[u8], rng: &mut MyRng) -> Move {
        let card = state.card();
        match card {
            Bonus(num) => self.bonus_strat(num, state, turn, roll, rng),
            Double => self.double_strat(state, turn, roll, rng),
            FireWork => self.fire_work_strat(state, turn, roll, rng),
            Street => self.street_strat(state, turn, roll, rng),
            Clover => self.tutto_strat(state, turn, roll, rng),
            PlusMinus => self.plus_minus_strat(state, turn, roll, rng),
            Stop => unreachable!(),
        }
    }

    fn tutto_strat(&self, state: &Game, turn: &Turn, roll: &[u8], rng: &mut MyRng) -> Move;
    fn bonus_strat(
        &self,
        num: i32,
        state: &Game,
        turn: &Turn,
        roll: &[u8],
        rng: &mut MyRng,
    ) -> Move;
    fn double_strat(&self, state: &Game, turn: &Turn, roll: &[u8], rng: &mut MyRng) -> Move;
    fn fire_work_strat(&self, state: &Game, turn: &Turn, roll: &[u8], rng: &mut MyRng) -> Move;
    fn street_strat(&self, state: &Game, turn: &Turn, roll: &[u8], rng: &mut MyRng) -> Move;
    fn plus_minus_strat(&self, state: &Game, turn: &Turn, roll: &[u8], rng: &mut MyRng) -> Move;

    fn card_strat(&self, state: &Game, last_turn: &Turn, rng: &mut MyRng) -> bool;
}

pub struct Move {
    takes: Vec<Take>,
    write: bool,
}

pub enum Take {
    Single(usize),
    Triple(usize, usize, usize),
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum CountedDice {
    Single5,
    Single1,
    Triple(u8),
    SingleStreet(u8),
}

impl CountedDice {
    pub fn points(&self) -> i32 {
        match self {
            CountedDice::Single5 => 50,
            CountedDice::Single1 => 100,
            CountedDice::Triple(n) => {
                if *n != 1 {
                    *n as i32 * 100
                } else {
                    1_000
                }
            }
            CountedDice::SingleStreet(_) => unreachable!(),
        }
    }

    pub fn number_of_dice(&self) -> usize {
        match self {
            CountedDice::Triple(_) => 3,
            _ => 1,
        }
    }
}

struct Turn {
    card: Card,
    counted_dice: Vec<CountedDice>,
    previous_cards_total: i32,
    fire_work_points: Option<i32>,
    failed: bool,
}

impl Turn {
    pub fn new(card: Card) -> Self {
        Self {
            card,
            counted_dice: Vec::new(),
            previous_cards_total: 0,
            fire_work_points: if card == FireWork { Some(0) } else { None },
            failed: false,
        }
    }

    pub fn new_card(&mut self, card: Card) {
        self.finish_card();
        self.card = card;
        if card == FireWork {
            self.fire_work_points = Some(0);
        }
    }

    pub fn finish_card(&mut self) {
        let mut new_points = self.this_card_point();
        match self.card {
            Bonus(n) => new_points += n,
            Double => new_points *= 2,
            FireWork => new_points += self.fire_work_points.unwrap(),
            Street => (),
            Clover => new_points = POINT_GOAL,
            Stop => unreachable!(),
            PlusMinus => new_points = 1000,
        }
        self.counted_dice = Vec::new();
        self.previous_cards_total += new_points;
        self.fire_work_points = None;
    }

    pub fn get_new_dice(&mut self, rng: &mut MyRng) -> Vec<u8> {
        (0..(NUMBER_OF_DICE
            - self
                .counted_dice
                .iter()
                .map(|x| x.number_of_dice())
                .sum::<usize>()))
            .map(|_| rng.gen_range(1..=6))
            .collect()
    }

    pub fn take_street_dice(&mut self, roll: &[u8], takes: Vec<Take>) -> Result<(), RuleError> {
        let mut taken_idxs = Vec::new();
        for take in takes {
            if let Take::Single(idx) = take {
                if (!self
                    .counted_dice
                    .contains(&CountedDice::SingleStreet(roll[idx])))
                    && !taken_idxs.contains(&idx)
                {
                    self.counted_dice.push(CountedDice::SingleStreet(roll[idx]))
                } else {
                    return Err(RuleError::IllegalTake);
                }
                taken_idxs.push(idx)
            } else {
                return Err(RuleError::CardIsStreet);
            }
        }
        return Ok(());
    }

    pub fn take_dice(&mut self, roll: &[u8], takes: Vec<Take>) -> Result<(), RuleError> {
        if takes.is_empty() {
            return Err(RuleError::IllegalTake);
        }
        if self.card == Street {
            return self.take_street_dice(roll, takes);
        }
        let mut taken_idxs = Vec::new();
        for take in takes {
            match take {
                Take::Single(idx) => {
                    if taken_idxs.contains(&idx) {
                        return Err(RuleError::DuplicateDice);
                    }
                    if roll[idx] == 5 {
                        self.counted_dice.push(CountedDice::Single5);
                        taken_idxs.push(idx);
                    } else if roll[idx] == 1 {
                        self.counted_dice.push(CountedDice::Single1);
                        taken_idxs.push(idx);
                    } else {
                        return Err(RuleError::IllegalTake);
                    }
                }
                Take::Triple(a, b, c) => {
                    if taken_idxs.contains(&a)
                        || taken_idxs.contains(&b)
                        || taken_idxs.contains(&c)
                        || a == b
                        || b == c
                        || c == a
                    {
                        return Err(RuleError::DuplicateDice);
                    }
                    if roll[a] != roll[b] || roll[b] != roll[c] || roll[c] != roll[a] {
                        return Err(RuleError::IllegalTriple);
                    }
                    self.counted_dice.push(CountedDice::Triple(roll[a]));
                    taken_idxs.push(a);
                    taken_idxs.push(b);
                    taken_idxs.push(c);
                }
            }
        }
        if self.card == FireWork && self.is_tutto() {
            *self.fire_work_points.as_mut().unwrap() +=
                self.counted_dice.iter().map(|x| x.points()).sum::<i32>();
            self.counted_dice = Vec::new();
        }
        Ok(())
    }

    pub fn is_tutto(&self) -> bool {
        self.counted_dice
            .iter()
            .map(|x| x.number_of_dice())
            .sum::<usize>()
            == NUMBER_OF_DICE
    }

    pub fn set_failed(&mut self) {
        if self.card == FireWork {
            self.finish_card();
            return;
        }
        self.failed = true;
        self.previous_cards_total = 0;
        self.counted_dice = Vec::new()
    }

    pub fn contains_valid_dice(&self, roll: &[u8]) -> bool {
        if self.card == Street {
            for dice in roll {
                if !self
                    .counted_dice
                    .contains(&CountedDice::SingleStreet(*dice))
                {
                    return true;
                }
            }
            return false;
        }
        if roll.contains(&1) {
            return true;
        }
        if roll.contains(&5) {
            return true;
        }
        for i in 0..(roll.len().checked_sub(2).unwrap_or(0)) {
            let mut count = 1;
            for j in i + 1..roll.len() {
                if roll[i] == roll[j] {
                    count += 1
                }
            }
            if count >= 3 {
                return true;
            }
        }
        return false;
    }

    pub fn this_card_point(&self) -> i32 {
        if self.card == Street {
            // this check might be unnecessary
            if self.is_tutto() {
                return 2000;
            } else {
                return 0;
            }
        }
        let mut res = self.counted_dice.iter().map(|x| x.points()).sum();
        if self.card == FireWork {
            res += self.fire_work_points.unwrap_or(0);
        }
        return res;
    }

    pub fn all_points(&self) -> i32 {
        self.previous_cards_total + self.this_card_point()
    }
}

struct Game {
    players: Vec<Box<dyn Player>>,
    rng: Option<MyRng>,
    turn: usize,
    log: Vec<PlayerLog>,
    deck: Deck,
    scores: Vec<i32>,
}

impl Game {
    pub fn new(players: Vec<Box<dyn Player>>, cards: Vec<Card>, seed: Option<&str>) -> Self {
        let mut rng: MyRng = if let Some(seed) = seed {
            Seeder::from(seed).make_rng()
        } else {
            MyRng::from_entropy()
        };
        let deck = Deck::shuffle_from_vec(cards, &mut rng);
        Self {
            log: (0..players.len()).map(|_| PlayerLog::new()).collect(),
            scores: vec![0; players.len()],
            players,
            rng: Some(rng),
            turn: 0,
            deck,
        }
    }

    pub fn highest_score(&self) -> (i32, Vec<usize>) {
        let mut players = Vec::new();
        let mut highest_score = 0;
        for (p, score) in self.scores.iter().enumerate() {
            if highest_score < *score {
                highest_score = *score;
                players = vec![p]
            }
            if highest_score == *score {
                players.push(p)
            }
        }
        (highest_score, players)
    }

    pub fn card(&self) -> Card {
        self.deck.open_card()
    }

    pub fn next_turn(&mut self) {
        let player_index = self.turn % self.players.len();
        let active_player = &self.players[player_index];
        let mut rng = self.rng.take().unwrap();
        let mut turn = Turn::new(self.deck.draw_new(&mut rng));
        let mut turn_log = Vec::new();
        let mut achieved_minus = 0;
        'card: loop {
            if self.deck.open_card() == Stop {
                turn.set_failed();
                turn_log.push(CardLog {
                    card: Stop,
                    points_with_card: 0,
                });
                break 'card;
            }
            if self.deck.open_card() == PlusMinus && self.highest_score().1.contains(&player_index)
            {
                turn.set_failed();
                turn_log.push(CardLog {
                    card: PlusMinus,
                    points_with_card: 0,
                });
                break 'card;
            }
            'dice: loop {
                let roll = turn.get_new_dice(&mut rng);
                if !turn.contains_valid_dice(&roll) {
                    turn.set_failed();
                    turn_log.push(CardLog {
                        card: self.deck.open_card(),
                        points_with_card: 0,
                    });
                    break 'card;
                }
                let this_move = active_player.make_move(&self, &turn, &roll, &mut rng);
                turn.take_dice(&roll, this_move.takes).unwrap();
                if this_move.write {
                    turn_log.push(CardLog {
                        card: self.deck.open_card(),
                        points_with_card: turn.this_card_point(),
                    });
                    break 'card;
                }
                if turn.is_tutto() {
                    if self.deck.open_card() == PlusMinus {
                        achieved_minus += 1;
                    }
                    turn_log.push(CardLog {
                        card: self.deck.open_card(),
                        points_with_card: turn.this_card_point(),
                    });
                    if active_player.card_strat(&self, &turn, &mut rng) {
                        turn.new_card(self.deck.draw_new(&mut rng));
                        break 'dice;
                    } else {
                        turn.finish_card();
                        break 'card;
                    }
                }
            }
        }

        for _ in 0..achieved_minus {
            for this_player_idx in self.highest_score().1 {
                self.log[this_player_idx].push(TurnLog::Minus1000);
                self.scores[this_player_idx] -= 1000;
            }
        }
        self.rng = Some(rng);
        self.log[player_index].push(TurnLog::from_vec(turn_log, turn.all_points()));
        self.scores[player_index] += turn.all_points();
        self.turn += 1;
    }

    pub fn play_game(&mut self) {
        'outer: loop {
            self.next_turn();
            for (player_idx, score) in self.scores.iter().enumerate() {
                print!("player{player_idx}: {score}   ");
                if *score >= POINT_GOAL {
                    println!();
                    println!("player{player_idx} is winner");
                    break 'outer;
                }
            }
            println!();
        }
    }

    pub fn save_logs(&self) {
        for (i, log) in self.log.iter().enumerate() {
            std::fs::write(
                format!("out/player{i}.json"),
                &serde_json::to_string_pretty(log).unwrap(),
            )
            .unwrap()
        }
    }
}

fn main() {
    let mut game = Game::new(
        vec![Box::new(NaivePlayer), Box::new(NaivePlayer)],
        vec![FireWork; 30],
        None,
    );
    game.play_game();
    game.save_logs()
}
