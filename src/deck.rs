use std::mem::swap;

use crate::MyRng;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use Card::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Card {
    Bonus(i32),
    Double,
    FireWork,
    Street,
    Clover,
    Stop,
    PlusMinus,
}

#[derive(Debug)]
pub struct Deck {
    new: Vec<Card>,
    seen: Vec<Card>,
}

impl Deck {
    pub fn new_official(rng: &mut MyRng) -> Self {
        let mut deck = Vec::with_capacity(56);
        deck.extend([Bonus(200); 5].into_iter());
        deck.extend([Bonus(300); 5].into_iter());
        deck.extend([Bonus(400); 5].into_iter());
        deck.extend([Bonus(500); 5].into_iter());
        deck.extend([Bonus(600); 5].into_iter());
        deck.extend([Double; 5].into_iter());
        deck.extend([FireWork; 5].into_iter());
        deck.extend([Street; 5].into_iter());
        deck.extend([Clover; 1].into_iter());
        deck.extend([Stop; 10].into_iter());
        deck.extend([PlusMinus; 5].into_iter());
        deck.shuffle(rng);
        Self {
            new: deck,
            seen: Vec::new(),
        }
    }

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
