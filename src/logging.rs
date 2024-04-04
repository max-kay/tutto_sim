use serde::{Deserialize, Serialize};

use crate::Card;

#[derive(Serialize, Deserialize)]
pub struct PlayerLog(Vec<TurnLog>);

impl PlayerLog {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, log: TurnLog) {
        self.0.push(log)
    }
}

#[derive(Serialize, Deserialize)]
pub enum TurnLog {
    Normal { cards: Vec<CardLog>, total: i32 },
    Minus1000,
}

impl TurnLog {
    pub fn from_vec(vec: Vec<CardLog>, total: i32) -> Self {
        Self::Normal { cards: vec, total }
    }
}

#[derive(Serialize, Deserialize)]
pub struct CardLog {
    pub card: Card,
    pub points_with_card: i32,
}
