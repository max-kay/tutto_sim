use std::{fmt::Display, mem::swap};

use crate::MyRng;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use Card::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Card {
    Bonus(i32),
    Double,
    FireWork,
    Flush,
    Clover,
    #[default]
    Stop,
    PlusMinus,
}

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct Deck {
    new: Vec<Card>,
    seen: Vec<Card>,
}

pub fn get_official_cards() -> Vec<Card> {
    let mut deck = Vec::with_capacity(56);
    deck.extend([Bonus(200); 5].into_iter());
    deck.extend([Bonus(300); 5].into_iter());
    deck.extend([Bonus(400); 5].into_iter());
    deck.extend([Bonus(500); 5].into_iter());
    deck.extend([Bonus(600); 5].into_iter());
    deck.extend([Double; 5].into_iter());
    deck.extend([FireWork; 5].into_iter());
    // deck.extend([Flush; 5].into_iter());
    deck.extend([Clover; 1].into_iter());
    deck.extend([Stop; 10].into_iter());
    deck.extend([PlusMinus; 5].into_iter());
    deck
}

impl Deck {
    pub fn shuffle_from_vec(mut cards: Vec<Card>, rng: &mut MyRng) -> Self {
        cards.shuffle(rng);
        Self {
            new: cards,
            seen: Vec::new(),
        }
    }

    pub fn draw_new(&mut self, rng: &mut MyRng) -> Card {
        if self.new.is_empty() {
            swap(&mut self.new, &mut self.seen);
            self.new.shuffle(rng)
        }
        let card = self.new.pop().unwrap();
        self.seen.push(card);
        card
    }

    pub fn open_card(&self) -> Card {
        *self.seen.last().unwrap()
    }
}
