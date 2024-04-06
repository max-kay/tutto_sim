use std::usize;

use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg as MyRng;
use rand_seeder::Seeder;

pub mod deck;
pub mod logging;
pub mod players;

pub use deck::{Card, Deck};
pub use logging::{CardLog, PlayerLog, TurnLog};
pub use players::NaivePlayer;
use players::Player;
use Card::*;

pub const POINT_GOAL: i32 = 10_000;
pub const NUMBER_OF_DICE: usize = 6;

pub struct Move {
    takes: Vec<Take>,
    write: bool,
}

pub enum Take {
    Single(usize, u8),
    Triple(usize, usize, usize, u8),
}

impl Take {
    pub fn into_taken_dice(self) -> TakenDice {
        match self {
            Self::Single(_, 1) => TakenDice::Single1,
            Self::Single(_, 5) => TakenDice::Single5,
            Self::Triple(_, _, _, val) => TakenDice::Triple(val),
            _ => unreachable!(),
        }
    }

    pub fn into_flush_dice(self) -> TakenDice {
        if let Self::Single(_, val) = self {
            TakenDice::SingleFlush(val)
        } else {
            unreachable!()
        }
    }

    pub fn idxs(&self) -> Vec<usize> {
        match self {
            Take::Single(i, _) => vec![*i],
            Take::Triple(i1, i2, i3, _) => vec![*i1, *i2, *i3],
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum TakenDice {
    Single5,
    Single1,
    Triple(u8),
    SingleFlush(u8),
}

impl TakenDice {
    pub fn points(&self) -> i32 {
        match self {
            TakenDice::Single5 => 50,
            TakenDice::Single1 => 100,
            TakenDice::Triple(n) => {
                if *n != 1 {
                    *n as i32 * 100
                } else {
                    1_000
                }
            }
            TakenDice::SingleFlush(_) => unreachable!(),
        }
    }

    pub fn number_of_dice(&self) -> usize {
        match self {
            TakenDice::Triple(_) => 3,
            _ => 1,
        }
    }
}

pub struct Turn {
    card: Card,
    taken_dice: Vec<TakenDice>,
    previous_cards_total: i32,
    fire_work_points: i32,
    clover_win_next_tutto: bool,
    achieved_minus: u32,
    logs: Vec<CardLog>,
    roll: Vec<u8>,
    card_is_finished: bool,
}

impl Turn {
    /// creates a new Turn
    pub fn new() -> Self {
        Self {
            card: Default::default(),
            taken_dice: Vec::new(),
            previous_cards_total: 0,
            fire_work_points: 0,
            clover_win_next_tutto: false,
            achieved_minus: 0,
            logs: Vec::new(),
            roll: Vec::new(),
            card_is_finished: true,
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
            self.fire_work_points = 0;
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

    /// takes the dice
    /// assumes the take is valid
    /// should be guaranteed by the methods categorize roll and categorize flush
    pub fn take_dice(&mut self, takes: Vec<Take>) {
        debug_assert!(!self.card_is_finished);
        debug_assert!(!takes.is_empty());
        if self.card == Flush {
            for take in takes {
                self.taken_dice.push(take.into_flush_dice());
            }
            return;
        }
        for take in takes {
            self.taken_dice.push(take.into_taken_dice())
        }
        if self.is_tutto() {
            match self.card {
                FireWork => {
                    self.fire_work_points +=
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

    /// categorizes the roll into takes
    /// assumes card != Flush
    /// this is the only way to see the dice for a player
    pub fn catergorize_roll(&self) -> Vec<Take> {
        debug_assert_ne!(self.card, Flush);
        if self.roll.len() < 3 {
            let mut takes = Vec::new();
            for (i, dice) in self.roll.iter().enumerate() {
                if *dice == 5 {
                    takes.push(Take::Single(i, 5))
                }
            }
            for (i, dice) in self.roll.iter().enumerate() {
                if *dice == 1 {
                    takes.push(Take::Single(i, 1))
                }
            }
            takes
        } else {
            let mut taken_idxs = Vec::new();
            let mut takes = Vec::new();
            for i in (2..=6).chain([1].into_iter()) {
                let mut triplets = self.search_triplet(i);
                for chunk in triplets.chunks(3) {
                    takes.push(Take::Triple(chunk[0], chunk[1], chunk[2], i))
                }
                taken_idxs.append(&mut triplets);
            }
            for (i, dice) in self.roll.iter().enumerate() {
                if *dice == 5 && !taken_idxs.contains(&i) {
                    takes.push(Take::Single(i, 5))
                }
            }
            for (i, dice) in self.roll.iter().enumerate() {
                if *dice == 1 && !taken_idxs.contains(&i) {
                    takes.push(Take::Single(i, 1))
                }
            }
            takes
        }
    }

    /// searches for triplets of num
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

    /// categorizes the dice according to flush rules
    pub fn categorize_flush(&self) -> Vec<Take> {
        debug_assert_eq!(self.card, Flush);
        let numbers_present: Vec<_> = self
            .taken_dice
            .iter()
            .map(|x| {
                if let TakenDice::SingleFlush(n) = x {
                    *n
                } else {
                    unreachable!()
                }
            })
            .collect();

        let mut this_take_numbers = Vec::new();
        let mut this_take = Vec::new();
        for (i, dice) in self.roll.iter().enumerate() {
            if !numbers_present.contains(dice) && !this_take_numbers.contains(dice) {
                this_take.push(Take::Single(i, *dice));
                this_take_numbers.push(*dice);
            }
        }
        this_take
    }

    /// returns true if the roll allows for valid takes
    pub fn contains_valid_dice(&self) -> bool {
        debug_assert!(!self.card_is_finished);
        if self.card == Flush {
            for dice in &self.roll {
                if !self.taken_dice.contains(&TakenDice::SingleFlush(*dice)) {
                    return true;
                }
            }
            return false;
        }
        !self.catergorize_roll().is_empty()
    }

    /// returns the points made during this card
    pub fn this_card_points(&self) -> i32 {
        if self.card == Flush {
            return 0;
        }
        let mut res = self.taken_dice.iter().map(|x| x.points()).sum();
        if self.card == FireWork {
            res += self.fire_work_points;
        }
        return res;
    }
}

/// functions about finishing cards and turns
impl Turn {
    /// finish the card by counting the points not considering the tutto
    pub fn write_points(&mut self) {
        self.logs.push(CardLog {
            card: self.card,
            points: self.this_card_points(),
        });
        self.previous_cards_total += self.this_card_points();
        self.card_is_finished = true;
    }

    /// sums the points and applies the tutto action.
    fn finish_card(&mut self) {
        let mut new_points = self.this_card_points();
        match self.card {
            Bonus(n) => new_points += n,
            Double => new_points *= 2,
            FireWork => new_points += self.fire_work_points,
            Flush => (),
            Clover => new_points = POINT_GOAL,
            Stop => unreachable!(),
            PlusMinus => new_points = 1000,
        }
        self.logs.push(CardLog {
            card: self.card,
            points: new_points,
        });
        self.taken_dice = Vec::new();
        self.previous_cards_total += new_points;
        self.fire_work_points = 0;
        self.card_is_finished = true;
    }

    /// resets previous points to 0 and finishes card
    pub fn set_failed(&mut self) {
        if self.card == FireWork {
            self.write_points();
            return;
        }
        self.logs.push(CardLog {
            card: self.card,
            points: 0,
        });
        self.previous_cards_total = 0;
        self.taken_dice = Vec::new();
        self.card_is_finished = true;
    }

    /// returns the points gotten in the turn and a turn log
    pub fn finish_turn(self) -> (i32, TurnLog) {
        debug_assert!(self.card_is_finished);
        let points = {
            let this = &self;
            this.previous_cards_total + this.this_card_points()
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

    /// gets the index of the player currently playing
    pub fn get_player_idx(&self) -> usize {
        self.turn % self.players.len()
    }

    /// gets the current player
    fn get_current_player(&self) -> &dyn Player {
        self.players[self.get_player_idx()].as_ref()
    }

    /// plays the turn
    pub fn next_turn(&mut self) {
        // note that the type Turn handles counting points and that the logic
        // for new card happens in the function play_card
        let mut turn = Turn::new();
        loop {
            turn.new_card(self.deck.draw_new(self.rng.as_mut().unwrap()));
            if self.deck.open_card() == Stop {
                turn.set_failed();
                break;
            }
            if self.deck.open_card() == PlusMinus
                && self.highest_score().1.contains(&self.get_player_idx())
            {
                turn.set_failed();
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

        let idx = self.get_player_idx();
        let (points, log) = turn.finish_turn();
        self.log[idx].push(log);
        self.scores[idx] += points;
        self.turn += 1;
    }

    /// returns true if the turn needs to end
    /// additionally guarantees that the turn is card-finished
    fn play_card(&mut self, turn: &mut Turn) -> bool {
        loop {
            turn.roll_dice(self.rng.as_mut().unwrap());
            if !turn.contains_valid_dice() {
                turn.set_failed();
                return true;
            }
            let mut rng = self.rng.take().unwrap();
            let this_move = self.get_current_player().make_move(&self, &*turn, &mut rng);
            self.rng = Some(rng);
            turn.take_dice(this_move.takes);
            if this_move.write {
                turn.write_points();
                return true;
            }
            if turn.is_tutto() {
                turn.finish_card();
                if [Clover, PlusMinus].contains(&self.card()) {
                    return true;
                }
                let mut rng = self.rng.take().unwrap();
                let new_card = self
                    .get_current_player()
                    .card_strat(&self, &*turn, &mut rng);
                self.rng = Some(rng);
                return !new_card;
            }
        }
    }

    /// plays the game until a player reaches the POINT_GOAL
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
