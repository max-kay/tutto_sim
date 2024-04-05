use std::usize;

use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg as MyRng;
use rand_seeder::Seeder;
use thiserror::Error;

pub mod deck;
pub mod logging;
pub mod players;

pub use deck::{Card, Deck};
pub use logging::{CardLog, PlayerLog, TurnLog};
pub use players::NaivePlayer;
use players::Player;
use Card::*;

#[derive(Error, Debug, Copy, Clone)]
pub enum RuleError {
    #[error("card is flush")]
    CardIsFlush,
    #[error("card is not flush")]
    CardNotFlush,
    #[error("illegal take occured")]
    IllegalTake,
    #[error("dice was taken twice")]
    DuplicateDice,
    #[error("triple was invalid")]
    IllegalTriple,
}

const POINT_GOAL: i32 = 10_000;
const NUMBER_OF_DICE: usize = 6;

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
    SingleFlush(u8),
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
            CountedDice::SingleFlush(_) => unreachable!(),
        }
    }

    pub fn number_of_dice(&self) -> usize {
        match self {
            CountedDice::Triple(_) => 3,
            _ => 1,
        }
    }
}

pub struct Turn {
    card: Card,
    taken_dice: Vec<CountedDice>,
    previous_cards_total: i32,
    fire_work_points: Option<i32>,
    clover_win_next_tutto: bool,
    achieved_minus: u32,
    logs: Vec<CardLog>,
    roll: Vec<u8>,
    card_is_finished: bool,
}

impl Turn {
    /// creates a new Turn
    pub fn new(card: Card) -> Self {
        Self {
            card,
            taken_dice: Vec::new(),
            previous_cards_total: 0,
            fire_work_points: if card == FireWork { Some(0) } else { None },
            clover_win_next_tutto: false,
            achieved_minus: 0,
            logs: Vec::new(),
            roll: Vec::new(),
            card_is_finished: false,
        }
    }
}

impl Turn {
    /// accepts a new card and resets the dice
    /// assumes Tutto
    pub fn new_card(&mut self, card: Card) {
        assert!(self.card_is_finished);
        self.card = card;
        if card == FireWork {
            self.fire_work_points = Some(0);
        }
        self.card_is_finished = false;
    }

    /// generates a new roll
    pub fn roll_dice(&mut self, rng: &mut MyRng) {
        assert!(!self.card_is_finished);
        self.roll = (0..(NUMBER_OF_DICE
            - self
                .taken_dice
                .iter()
                .map(|x| x.number_of_dice())
                .sum::<usize>()))
            .map(|_| rng.gen_range(1..=6))
            .collect();
    }

    /// categorizes the roll into takes
    /// assumes card != Flush
    pub fn catergorize_roll(&self) -> Vec<Take> {
        debug_assert_ne!(self.card, Flush);
        if self.roll.len() < 3 {
            let mut takes = Vec::new();
            for (i, dice) in self.roll.iter().enumerate() {
                if *dice == 5 {
                    takes.push(Take::Single(i))
                }
            }
            for (i, dice) in self.roll.iter().enumerate() {
                if *dice == 1 {
                    takes.push(Take::Single(i))
                }
            }
            takes
        } else {
            let mut taken_idxs = Vec::new();
            let mut takes = Vec::new();
            for i in 2..=6 {
                let mut triplets = self.search_triplet(i);
                for chunk in triplets.chunks(3) {
                    takes.push(Take::Triple(chunk[0], chunk[1], chunk[2]))
                }
                taken_idxs.append(&mut triplets);
            }
            for (i, dice) in self.roll.iter().enumerate() {
                if *dice == 5 && !taken_idxs.contains(&i) {
                    takes.push(Take::Single(i))
                }
            }
            for (i, dice) in self.roll.iter().enumerate() {
                if *dice == 1 && !taken_idxs.contains(&i) {
                    takes.push(Take::Single(i))
                }
            }
            takes
        }
    }

    fn search_triplet(&self, num: u8) -> Vec<usize> {
        let mut incomplete = Vec::new();
        let mut out = Vec::new();
        for (i, val) in self.roll.iter().enumerate() {
            if *val == num {
                incomplete.push(i);
                if incomplete.len() == 3 {
                    out.append(&mut incomplete);
                }
            }
        }
        out
    }
    /// takes the dice returns an error if the take was illegal
    pub fn take_dice(&mut self, takes: Vec<Take>) -> Result<(), RuleError> {
        debug_assert!(!self.card_is_finished);
        if takes.is_empty() {
            return Err(RuleError::IllegalTake);
        }
        if self.card == Flush {
            return self.take_flush_dice(takes);
        }
        let mut taken_idxs = Vec::new();
        for take in takes {
            match take {
                Take::Single(idx) => {
                    if taken_idxs.contains(&idx) {
                        return Err(RuleError::DuplicateDice);
                    }
                    if self.roll[idx] == 5 {
                        self.taken_dice.push(CountedDice::Single5);
                        taken_idxs.push(idx);
                    } else if self.roll[idx] == 1 {
                        self.taken_dice.push(CountedDice::Single1);
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
                    if self.roll[a] != self.roll[b]
                        || self.roll[b] != self.roll[c]
                        || self.roll[c] != self.roll[a]
                    {
                        return Err(RuleError::IllegalTriple);
                    }
                    self.taken_dice.push(CountedDice::Triple(self.roll[a]));
                    taken_idxs.push(a);
                    taken_idxs.push(b);
                    taken_idxs.push(c);
                }
            }
        }
        if self.is_tutto() {
            match self.card {
                FireWork => {
                    *self.fire_work_points.as_mut().unwrap() +=
                        self.taken_dice.iter().map(|x| x.points()).sum::<i32>();
                    self.taken_dice = Vec::new();
                }
                Clover => {
                    if !self.clover_win_next_tutto {
                        self.clover_win_next_tutto = true;
                        self.taken_dice = Vec::new();
                    }
                }
                PlusMinus => {
                    self.achieved_minus += 1;
                }
                _ => (),
            }
        }
        Ok(())
    }

    /// handles spacial case flush
    fn take_flush_dice(&mut self, takes: Vec<Take>) -> Result<(), RuleError> {
        let mut taken_idxs = Vec::new();
        for take in takes {
            if let Take::Single(idx) = take {
                if (!self
                    .taken_dice
                    .contains(&CountedDice::SingleFlush(self.roll[idx])))
                    && !taken_idxs.contains(&idx)
                {
                    self.taken_dice
                        .push(CountedDice::SingleFlush(self.roll[idx]))
                } else {
                    return Err(RuleError::IllegalTake);
                }
                taken_idxs.push(idx)
            } else {
                return Err(RuleError::CardIsFlush);
            }
        }
        return Ok(());
    }
}

/// all about the game state
impl Turn {
    /// returns true if all dice are taken
    pub fn is_tutto(&self) -> bool {
        self.taken_dice
            .iter()
            .map(|x| x.number_of_dice())
            .sum::<usize>()
            == NUMBER_OF_DICE
    }

    /// returns true if the roll allows for valid takes
    pub fn contains_valid_dice(&self) -> bool {
        debug_assert!(!self.card_is_finished);
        if self.card == Flush {
            for dice in &self.roll {
                if !self.taken_dice.contains(&CountedDice::SingleFlush(*dice)) {
                    return true;
                }
            }
            return false;
        }
        !self.catergorize_roll().is_empty()
    }

    /// returns the points made during this card
    pub fn this_card_point(&self) -> i32 {
        if self.card == Flush {
            return 0;
        }
        let mut res = self.taken_dice.iter().map(|x| x.points()).sum();
        if self.card == FireWork {
            res += self.fire_work_points.unwrap_or(0);
        }
        return res;
    }
}

/// logging stuff
impl Turn {
    /// push a log
    pub fn push_card_log(&mut self, log: CardLog) {
        assert!(self.card_is_finished);
        self.logs.push(log)
    }
}

/// mutable impls
impl Turn {
    /// finish the card by counting the points not considering the tutto
    pub fn write_points(&mut self) {
        self.previous_cards_total += self.this_card_point();
        self.card_is_finished = true;
    }

    /// sums the points and applies the tutto action.
    fn finish_card(&mut self) {
        assert!(self.is_tutto());
        let mut new_points = self.this_card_point();
        match self.card {
            Bonus(n) => new_points += n,
            Double => new_points *= 2,
            FireWork => new_points += self.fire_work_points.unwrap(),
            Flush => (),
            Clover => new_points = POINT_GOAL,
            Stop => unreachable!(),
            PlusMinus => new_points = 1000,
        }
        self.taken_dice = Vec::new();
        self.previous_cards_total += new_points;
        self.fire_work_points = None;
        self.card_is_finished = true;
    }

    /// resets previous points to 0 and finishes card
    pub fn set_failed(&mut self) {
        if self.card == FireWork {
            self.write_points();
            return;
        }
        self.previous_cards_total = 0;
        self.taken_dice = Vec::new();
        self.card_is_finished = true;
    }

    /// returns the points gotten in the turn and a turn log
    pub fn finish_turn(self) -> (i32, TurnLog) {
        debug_assert!(self.card_is_finished);
        let points = {
            let this = &self;
            this.previous_cards_total + this.this_card_point()
        };
        (points, TurnLog::from_vec(self.logs, points))
    }
}

pub struct Game {
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

    pub fn get_active_index(&self) -> usize {
        self.turn % self.players.len()
    }

    pub fn get_active_player(&self) -> &dyn Player {
        self.players[self.get_active_index()].as_ref()
    }

    /// plays the turn
    /// note that the type Turn handles counting points and that the logic
    /// for new card happens in the function play_card
    pub fn next_turn(&mut self) {
        let mut turn = Turn::new(self.deck.draw_new(self.rng.as_mut().unwrap()));
        loop {
            if self.deck.open_card() == Stop {
                turn.set_failed();
                turn.push_card_log(CardLog {
                    card: Stop,
                    points_with_card: 0,
                });
                break;
            }
            if self.deck.open_card() == PlusMinus
                && self.highest_score().1.contains(&self.get_active_index())
            {
                turn.set_failed();
                turn.push_card_log(CardLog {
                    card: PlusMinus,
                    points_with_card: 0,
                });
                break;
            }
            if self.play_card(&mut turn) {
                break;
            }
        }

        for _ in 0..turn.achieved_minus {
            for this_player_idx in self.highest_score().1 {
                self.log[this_player_idx].push(TurnLog::Minus1000);
                self.scores[this_player_idx] -= 1000;
            }
        }

        let idx = self.get_active_index();
        let (points, log) = turn.finish_turn();
        self.log[idx].push(log);
        self.scores[idx] += points;
        self.turn += 1;
    }

    /// returns true if the turn needs to end
    /// this function should also guarantee that if it returns true the card is
    /// in a state which allow the function all_points to return the total points
    fn play_card(&mut self, turn: &mut Turn) -> bool {
        loop {
            turn.roll_dice(self.rng.as_mut().unwrap());
            if !turn.contains_valid_dice() {
                turn.set_failed();
                turn.push_card_log(CardLog {
                    card: self.deck.open_card(),
                    points_with_card: 0,
                });
                return true;
            }
            let mut rng = self.rng.take().unwrap();
            let this_move = self.get_active_player().make_move(&self, &*turn, &mut rng);
            self.rng = Some(rng);
            turn.take_dice(this_move.takes).unwrap();
            if this_move.write {
                turn.write_points();
                turn.push_card_log(CardLog {
                    card: self.deck.open_card(),
                    points_with_card: turn.this_card_point(),
                });
                return true;
            }
            if turn.is_tutto() {
                turn.finish_card();
                turn.push_card_log(CardLog {
                    card: self.deck.open_card(),
                    points_with_card: turn.this_card_point(),
                });
                let mut rng = self.rng.take().unwrap();
                let new_card = self.get_active_player().card_strat(&self, &*turn, &mut rng);
                self.rng = Some(rng);
                if new_card {
                    turn.new_card(self.deck.draw_new(self.rng.as_mut().unwrap()));
                    return false;
                } else {
                    return true;
                }
            }
        }
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
                format!("out/player{i}.ron"),
                &ron::ser::to_string_pretty(log, ron::ser::PrettyConfig::default()).unwrap(),
            )
            .unwrap()
        }
    }
}
