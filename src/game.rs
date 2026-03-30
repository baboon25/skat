use std::default;

use rand::{
    Rng,
    seq::{IndexedRandom, SliceRandom},
};

use crate::{
    deck::{Card, CardError, Deck},
    player::{self, Announcement, Bid, Party, Player},
};

pub struct Game<S>
where
    S: Scoring + Default,
{
    deck: Deck,
    players: [Player; 3],
    games: u8,
    scoring: S,
}

impl<S> Default for Game<S>
where
    S: Scoring + Default,
{
    fn default() -> Self {
        Self {
            deck: Default::default(),
            players: Default::default(),
            games: 1,
            scoring: S::default(),
        }
    }
}

impl<S> Game<S>
where
    S: Scoring + Default,
{
    pub fn new(
        player1: Player,
        player2: Player,
        player3: Player,
        games: u8,
        scoring: S,
    ) -> Self {
        Self {
            players: [player1, player2, player3],
            scoring,
            games,
            ..Default::default()
        }
    }

    pub async fn play(&mut self, rng: &mut dyn Rng) -> Result<(), GameError> {
        self.players.shuffle(rng);

        self.deck.hindu_shuffle(rng)?;

        for round in 0..self.games * 3 {
            let curr_player = (round % 3) as usize;

            //shuffle
            self.deck.riffle_shuffle(rng)?;
            self.players[(curr_player + 1) % 3].take_off(&mut self.deck);
            self.deck.riffle_shuffle(rng)?;
            self.players[(curr_player + 1) % 3].take_off(&mut self.deck);

            //deal
            let skat = self.deal(curr_player);

            // bid
            let declarer_idx = self.bid(curr_player).await?;
            // let [p0, p1, p2] = &mut self.players;
            let (mut winner, [mut opponent0, mut opponent1]) = self.split_players(declarer_idx);

            winner.party = Some(Party::Re);
            opponent0.party = Some(Party::Contra);
            opponent1.party = Some(Party::Contra);

            // play
            let announcement = winner.announce().await;

            winner.tricks.append(&mut skat.into());

            for _ in 0..10 {
                let new_winner = winner.play(opponent0, opponent1, &announcement).await?;
                if std::ptr::eq(new_winner, winner) {
                    continue;
                }
                if std::ptr::eq(new_winner, opponent0 as *const _) {
                    std::mem::swap(&mut winner, &mut opponent0);
                } else {
                    std::mem::swap(&mut winner, &mut opponent1);
                }
                std::mem::swap(&mut opponent0, &mut opponent1);
            }

            self.score(&announcement)?;
        }
        Ok(())
    }

    fn deal(&mut self, curr_player: usize) -> [Card; 2] {
        let (mut deals, skat) = self.deck.deal();
        for (i, deal) in deals.iter_mut().enumerate() {
            let mut extended = [Card::default(); 12];
            self.players[(curr_player + i + 1) % 3].hand = {
                extended[..10].copy_from_slice(deal);
                extended
            }
        }
        skat
    }

    async fn bid(&mut self, curr_player: usize) -> Result<usize, GameError> {
        let mut listening_player = (curr_player + 1) % 3;
        let mut biding_player = (curr_player + 2) % 3;
        let mut current_bid = Bid::default();

        'outer: for _ in 0..2 {
            match self.players[biding_player].bid(current_bid).await {
                Bid::Pass => {
                    biding_player = (biding_player + 1) % 3;
                }
                bid => {
                    if bid <= current_bid {
                        return Err(GameError::InvalidBidError(bid, current_bid));
                    }

                    current_bid = bid;
                    while self.players[listening_player].listen(current_bid).await {
                        let bid = self.players[biding_player].bid(current_bid).await;
                        if bid == Bid::Pass {
                            biding_player = (biding_player + 1) % 3;
                            continue 'outer;
                        }
                        if bid <= current_bid {
                            return Err(GameError::InvalidBidError(bid, current_bid));
                        }
                        current_bid = bid;
                    }
                    listening_player = biding_player;
                    biding_player = (biding_player + 1) % 3;
                }
            }
        }

        Ok(listening_player)
    }

    fn score(&mut self, announcement: &Announcement) -> Result<(), ScoreError> {
        let re: Vec<_> = self
            .players
            .iter()
            .enumerate()
            .filter_map(|p| {
                if p.1.party.is_some_and(|p| p == Party::Re) {
                    return Some(p.0);
                }
                None
            })
            .collect();
        if re.is_empty() {
            return Err(ScoreError::NoReProvided);
        }
        if re.len() > 1 {
            return Err(ScoreError::TooManyReProvided);
        }
        let (re, kontra) = self.split_players(re[0]);
        S::score(re, &kontra, announcement)?;
        Ok(())
    }

    fn split_players(&mut self, idx: usize) -> (&mut Player, [&mut Player; 2])
    where
    {
        let p0 = &mut self.players[0] as *mut Player;
        let p1 = &mut self.players[1] as *mut Player;
        let p2 = &mut self.players[2] as *mut Player;
        unsafe {
            match idx {
                0 => (&mut *p0, [&mut *p1, &mut *p2]),
                1 => (&mut *p1, [&mut *p2, &mut *p0]),
                2 => (&mut *p2, [&mut *p0, &mut *p1]),
                _ => unreachable!(),
            }
        }
    }
}

pub enum GameError {
    CardError(CardError),
    InvalidBidError(Bid, Bid),
    ScoreError(ScoreError),
}

impl From<CardError> for GameError {
    fn from(value: CardError) -> Self {
        Self::CardError(value)
    }
}

pub trait Scoring {
    fn score(re: &mut Player, kontra: &[&mut Player], announcement: &Announcement) -> Result<(), ScoreError>;
}

pub enum ScoreError {
    NoReProvided,
    TooManyReProvided,
}

impl From<ScoreError> for GameError {
    fn from(value: ScoreError) -> Self {
        GameError::ScoreError(value)
    }
}

pub struct DefaultScore;

impl Scoring for DefaultScore {
    fn score(re: &mut Player, kontra: &[&mut Player], announcement: &Announcement) -> Result<(), ScoreError> {
        Ok(())
    }
}
