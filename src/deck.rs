use std::ops::Index;

use rand::{Rng, RngExt};
use rand_distr::{Distribution, Normal, NormalError};

use crate::{
    RandError,
    player::{AnnounceSuit, Announcement, Player, PlayerController},
};

pub type Skat = [Card; 2];

#[derive(Debug, Default, Clone, Copy)]
pub struct Card(u8);

impl Card {
    pub const fn new(suit: Suit, rank: Rank) -> Self {
        Self(suit as u8 + ((rank as u8) << 4))
    }

    pub fn get_suit(&self) -> Option<Suit> {
        let suit = self.0 & 0b0000_1111;
        Suit::try_from(suit).ok()
    }

    pub fn get_rank(&self) -> Option<Rank> {
        Rank::try_from(self.0).ok()
    }

    /// Returns `Ok(true)` if `self` beats `other` given the announcement.
    ///
    /// Non-commutative: `self` is assumed to be the leading (first-played) card.
    /// A card of a different suit can only win if it is trump.
    pub fn surpases(&self, other: &Card, announcement: &Announcement) -> Result<bool, CardError> {
        if self.get_suit() == None
            || other.get_suit() == None
            || self.get_rank() == None
            || other.get_rank() == None
        {
            return Err(CardError::Uninitialized);
        }
        let self_suit = self.get_suit().unwrap();
        let self_rank = self.get_rank().unwrap();
        let other_suit = other.get_suit().unwrap();
        let other_rank = other.get_rank().unwrap();

        Ok(match announcement.game {
            AnnounceSuit::Grand => {
                let self_jack = self_rank == Rank::Jack;
                let other_jack = other_rank == Rank::Jack;
                match (self_jack, other_jack) {
                    (true, true) => self_suit > other_suit,
                    (true, false) => true,
                    (false, true) => false,
                    (false, false) => {
                        if self_suit != other_suit {
                            true // other didn't follow suit
                        } else {
                            self_rank > other_rank
                        }
                    }
                }
            }
            AnnounceSuit::Null => self_rank > other_rank,
            AnnounceSuit::Suit(trump) => {
                let self_trump = self_suit == trump || self_rank == Rank::Jack;
                let other_trump = other_suit == trump || other_rank == Rank::Jack;
                let self_jack = self_rank == Rank::Jack;
                let other_jack = other_rank == Rank::Jack;

                match (self_trump, other_trump) {
                    (true, true) => match (self_jack, other_jack) {
                        (true, true) => self_suit > other_suit,
                        (true, false) => true,
                        (false, true) => false,
                        (false, false) => self_rank > other_rank,
                    },
                    (true, false) => true,
                    (false, true) => false,
                    (false, false) => {
                        if self_suit != other_suit {
                            true // other didn't follow suit
                        } else {
                            self_rank > other_rank
                        }
                    }
                }
            }
        })
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[repr(u8)]
pub enum Suit {
    Clubs = 4,
    Spades = 3,
    Hearts = 2,
    Diamonds = 1,
}

impl TryFrom<u8> for Suit {
    type Error = CardError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if !(1..=4).contains(&value) {
            return Err(CardError::ConversionError(value));
        }
        Ok(unsafe { std::mem::transmute(value) })
    }
}

impl From<Suit> for u8 {
    fn from(value: Suit) -> Self {
        value as u8
    }
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Rank {
    Ace = 8,
    Ten = 7,
    King = 6,
    Queen = 5,
    Jack = 4,
    Nine = 3,
    Eight = 2,
    Seven = 1,
}

impl TryFrom<u8> for Rank {
    type Error = CardError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let shifted = value >> 4;
        if !(1..=8).contains(&shifted) {
            return Err(CardError::ConversionError(shifted));
        }
        Ok(unsafe { std::mem::transmute(shifted) })
    }
}

impl From<Rank> for u8 {
    fn from(value: Rank) -> Self {
        value as u8
    }
}

impl Rank {
    const fn get_eyes(&self) -> u8 {
        match self {
            Rank::Ace => 11,
            Rank::Ten => 10,
            Rank::King => 4,
            Rank::Queen => 3,
            Rank::Jack => 2,
            _ => 0,
        }
    }
}

pub enum CardError {
    ConversionError(u8),
    RandError(RandError),
    Uninitialized,
}

pub struct Deck {
    deck: [Card; 32],
}

impl Default for Deck {
    fn default() -> Self {
        Self {
            deck: [
                Card::new(Suit::Clubs, Rank::Ace),
                Card::new(Suit::Clubs, Rank::Ten),
                Card::new(Suit::Clubs, Rank::King),
                Card::new(Suit::Clubs, Rank::Queen),
                Card::new(Suit::Clubs, Rank::Jack),
                Card::new(Suit::Clubs, Rank::Nine),
                Card::new(Suit::Clubs, Rank::Eight),
                Card::new(Suit::Clubs, Rank::Seven),
                Card::new(Suit::Spades, Rank::Ace),
                Card::new(Suit::Spades, Rank::Ten),
                Card::new(Suit::Spades, Rank::King),
                Card::new(Suit::Spades, Rank::Queen),
                Card::new(Suit::Spades, Rank::Jack),
                Card::new(Suit::Spades, Rank::Nine),
                Card::new(Suit::Spades, Rank::Eight),
                Card::new(Suit::Spades, Rank::Seven),
                Card::new(Suit::Hearts, Rank::Ace),
                Card::new(Suit::Hearts, Rank::Ten),
                Card::new(Suit::Hearts, Rank::King),
                Card::new(Suit::Hearts, Rank::Queen),
                Card::new(Suit::Hearts, Rank::Jack),
                Card::new(Suit::Hearts, Rank::Nine),
                Card::new(Suit::Hearts, Rank::Eight),
                Card::new(Suit::Hearts, Rank::Seven),
                Card::new(Suit::Diamonds, Rank::Ace),
                Card::new(Suit::Diamonds, Rank::Ten),
                Card::new(Suit::Diamonds, Rank::King),
                Card::new(Suit::Diamonds, Rank::Queen),
                Card::new(Suit::Diamonds, Rank::Jack),
                Card::new(Suit::Diamonds, Rank::Nine),
                Card::new(Suit::Diamonds, Rank::Eight),
                Card::new(Suit::Diamonds, Rank::Seven),
            ],
        }
    }
}

impl Deck {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn riffle_shuffle(&mut self, rng: &mut dyn Rng) -> Result<(), CardError> {
        let split = Normal::new(16f64, 32f64 * 0.1)
            .map_err(|e| CardError::RandError(RandError::NormalError(e)))?
            .sample(rng)
            .round() as usize;
        let mut left = self.deck[..split].to_vec();
        let mut right = self.deck[split..].to_vec();
        let mut result = [Card::default(); 32];
        let mut head = 0usize;

        while !left.is_empty() && !right.is_empty() {
            let p_left = left.len() as f64 / (left.len() + right.len()) as f64;
            if rng.random::<f64>() < p_left {
                result[head] = left.remove(0);
            } else {
                result[head] = right.remove(0);
            }
            head += 1;
        }
        self.deck = result;
        Ok(())
    }

    /// Simulates a Hindu shuffle: repeatedly takes small packets from the top
    /// of the held deck and drops them onto a growing pile, reversing packet order.
    pub fn hindu_shuffle(&mut self, rng: &mut dyn Rng) -> Result<(), CardError> {
        let mut remaining = self.deck.to_vec();
        let mut result: Vec<Card> = Vec::with_capacity(32);

        while !remaining.is_empty() {
            let packet_size = Normal::new(4f64, 1.5f64)
                .map_err(|e| CardError::RandError(RandError::NormalError(e)))?
                .sample(rng)
                .round() as usize;
            let packet_size = packet_size.clamp(1, remaining.len());
            let packet: Vec<Card> = remaining.drain(..packet_size).collect();
            // each packet is dropped on top of the growing pile
            let mut new_result = packet;
            new_result.extend(result);
            result = new_result;
        }

        self.deck.copy_from_slice(&result);
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.deck.len()
    }

    pub fn cut(&mut self, idx: usize) {
        let len = self.deck.len();
        self.deck.rotate_left(idx % len);
    }

    pub fn deal(&self) -> ([[Card; 10]; 3], [Card; 2]) {
        let deck = &self.deck;
        let second = [
            deck[0], deck[1], deck[2], deck[11], deck[12], deck[13], deck[14], deck[23], deck[24],
            deck[25],
        ];
        let third = [
            deck[3], deck[4], deck[5], deck[15], deck[16], deck[17], deck[18], deck[26], deck[27],
            deck[28],
        ];
        let first = [
            deck[6], deck[7], deck[8], deck[19], deck[20], deck[21], deck[22], deck[29], deck[30],
            deck[31],
        ];
        let skat = [deck[9], deck[10]];
        ([first, second, third], skat)
    }
}
