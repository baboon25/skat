use futures::channel::oneshot;
use wasm_bindgen::prelude::*;

use crate::deck::{Card, CardError, Deck, Rank, Suit};

#[async_trait::async_trait]
pub trait PlayerController {
    async fn play(&mut self, previous_played_cards: &[Card]) -> Card;
    async fn bid(&mut self, current_bid: Bid) -> Bid;
    async fn listen(&mut self, bid: Bid) -> bool;
    async fn announce(&mut self, hand: &[Card]) -> Announcement;
    async fn take_off(&mut self, deck_size: usize) -> usize;
}

/// Einfache regelbasierte KI. Die Hand wird in `announce` gespeichert
/// und in `play` verwendet, da der Trait die Hand dort nicht übergibt.
#[derive(Debug, Default)]
pub struct AiController {
    hand: Vec<Card>,
}

#[async_trait::async_trait]
impl PlayerController for AiController {
    async fn play(&mut self, previous: &[Card]) -> Card {
        if previous.is_empty() {
            // Anführend: höchste Karte spielen
            let best = self.hand.iter().enumerate()
                .max_by_key(|(_, c)| c.get_rank().map(|r| r as u8).unwrap_or(0));
            let idx = best.map(|(i, _)| i).unwrap_or(0);
            self.hand.remove(idx)
        } else {
            // Folgend: niedrigste Karte (Strategie später verfeinern)
            let idx = self.hand.iter().enumerate()
                .min_by_key(|(_, c)| c.get_rank().map(|r| r as u8).unwrap_or(u8::MAX))
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.hand.remove(idx)
        }
    }

    async fn bid(&mut self, _current_bid: Bid) -> Bid {
        Bid::Pass
    }

    async fn listen(&mut self, _bid: Bid) -> bool {
        false
    }

    async fn announce(&mut self, hand: &[Card]) -> Announcement {
        self.hand = hand.to_vec();
        // Simpel: immer Grand ansagen
        Announcement {
            game: AnnounceSuit::Grand,
            hand: false,
            schneider: false,
            schwarz: false,
        }
    }

    async fn take_off(&mut self, deck_size: usize) -> usize {
        deck_size / 2
    }
}

/// Human-Controller: wartet per oneshot-Channel auf Eingaben aus JS/UI.
pub struct HumanController {
    play_sender: Option<oneshot::Sender<Card>>,
    bid_sender: Option<oneshot::Sender<Bid>>,
    listen_sender: Option<oneshot::Sender<bool>>,
    announce_sender: Option<oneshot::Sender<Announcement>>,
    take_off_sender: Option<oneshot::Sender<usize>>,
    play_receiver: Option<oneshot::Receiver<Card>>,
    bid_receiver: Option<oneshot::Receiver<Bid>>,
    listen_receiver: Option<oneshot::Receiver<bool>>,
    announce_receiver: Option<oneshot::Receiver<Announcement>>,
    take_off_receiver: Option<oneshot::Receiver<usize>>,
}

impl HumanController {
    pub fn new() -> Self {
        let (play_tx, play_rx) = oneshot::channel();
        let (bid_tx, bid_rx) = oneshot::channel();
        let (listen_tx, listen_rx) = oneshot::channel();
        let (announce_tx, announce_rx) = oneshot::channel();
        let (take_off_tx, take_off_rx) = oneshot::channel();
        Self {
            play_sender: Some(play_tx),
            bid_sender: Some(bid_tx),
            listen_sender: Some(listen_tx),
            announce_sender: Some(announce_tx),
            take_off_sender: Some(take_off_tx),
            play_receiver: Some(play_rx),
            bid_receiver: Some(bid_rx),
            listen_receiver: Some(listen_rx),
            announce_receiver: Some(announce_rx),
            take_off_receiver: Some(take_off_rx),
        }
    }

    fn reset_channel<T>() -> (Option<oneshot::Sender<T>>, Option<oneshot::Receiver<T>>) {
        let (tx, rx) = oneshot::channel();
        (Some(tx), Some(rx))
    }

    /// Wird von JS aufgerufen wenn der Spieler eine Karte auswählt.
    pub fn resolve_play(&mut self, card: Card) {
        if let Some(tx) = self.play_sender.take() {
            tx.send(card).ok();
        }
        let (tx, rx) = oneshot::channel();
        self.play_sender = Some(tx);
        self.play_receiver = Some(rx);
    }

    /// Wird von JS aufgerufen wenn der Spieler ein Gebot macht.
    pub fn resolve_bid(&mut self, bid: Bid) {
        if let Some(tx) = self.bid_sender.take() {
            tx.send(bid).ok();
        }
        let (tx, rx) = oneshot::channel();
        self.bid_sender = Some(tx);
        self.bid_receiver = Some(rx);
    }

    /// Wird von JS aufgerufen wenn der Spieler hört/passt.
    pub fn resolve_listen(&mut self, listens: bool) {
        if let Some(tx) = self.listen_sender.take() {
            tx.send(listens).ok();
        }
        let (tx, rx) = oneshot::channel();
        self.listen_sender = Some(tx);
        self.listen_receiver = Some(rx);
    }

    /// Wird von JS aufgerufen wenn der Spieler ansagt.
    pub fn resolve_announce(&mut self, announcement: Announcement) {
        if let Some(tx) = self.announce_sender.take() {
            tx.send(announcement).ok();
        }
        let (tx, rx) = oneshot::channel();
        self.announce_sender = Some(tx);
        self.announce_receiver = Some(rx);
    }

    /// Wird von JS aufgerufen wenn der Spieler abhebt.
    pub fn resolve_take_off(&mut self, idx: usize) {
        if let Some(tx) = self.take_off_sender.take() {
            tx.send(idx).ok();
        }
        let (tx, rx) = oneshot::channel();
        self.take_off_sender = Some(tx);
        self.take_off_receiver = Some(rx);
    }
}

#[async_trait::async_trait]
impl PlayerController for HumanController {
    async fn play(&mut self, _previous: &[Card]) -> Card {
        self.play_receiver.take().unwrap().await.unwrap()
    }

    async fn bid(&mut self, _current_bid: Bid) -> Bid {
        self.bid_receiver.take().unwrap().await.unwrap()
    }

    async fn listen(&mut self, _bid: Bid) -> bool {
        self.listen_receiver.take().unwrap().await.unwrap()
    }

    async fn announce(&mut self, _hand: &[Card]) -> Announcement {
        self.announce_receiver.take().unwrap().await.unwrap()
    }

    async fn take_off(&mut self, _deck_size: usize) -> usize {
        self.take_off_receiver.take().unwrap().await.unwrap()
    }
}


pub struct Player {
    pub hand: [Card; 12],
    pub party: Option<Party>,
    pub score: i32,
    pub tricks: Vec<Card>,
    controller: Box<dyn PlayerController>,
}

impl Player {
    pub async fn take_off(&mut self, deck: &mut Deck) {
        let idx = self.controller.take_off(deck.len()).await;
        deck.cut(idx);
    }

    pub async fn bid(&mut self, current_bid: Bid) -> Bid {
        self.controller.bid(current_bid).await
    }

    pub async fn listen(&mut self, bid: Bid) -> bool {
        self.controller.listen(bid).await
    }

    pub async fn announce(&mut self) -> Announcement {
        self.controller.announce(&self.hand).await
    }

    pub async fn play<'b>(
        &'b mut self,
        op1: &'b mut Player,
        op2: &'b mut Player,
        announcement: &Announcement,
    ) -> Result<*const Player, CardError> {
        let played_card = self.controller.play(&[]).await;
        let op1_card = op1.controller.play(&[played_card]).await;
        let op2_card = op2.controller.play(&[played_card, op1_card]).await;
        self.tricks.push(played_card);

        if played_card.surpases(&op1_card, announcement)?
            && played_card.surpases(&op2_card, announcement)?
        {
            self.tricks
                .append(&mut vec![played_card, op1_card, op2_card]);
            return Ok(self);
        }
        if op1_card.surpases(&played_card, announcement)?
            && op1_card.surpases(&op2_card, announcement)?
        {
            op1.tricks
                .append(&mut vec![played_card, op1_card, op2_card]);
            return Ok(op1);
        }
        if op2_card.surpases(&played_card, announcement)?
            && op2_card.surpases(&op1_card, announcement)?
        {
            op2.tricks
                .append(&mut vec![played_card, op1_card, op2_card]);
        }
        Ok(op2)
    }

}

impl Default for Player {
    fn default() -> Self {
        Self {
            controller: Box::new(AiController::default()),
            hand: Default::default(),
            party: None,
            score: 0,
            tricks: Vec::new(),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Bid {
    #[default]
    Pass = 0,
    B18 = 18,
    B20 = 20,
    B22 = 22,
    B23 = 23,
    B24 = 24,
    B27 = 27,
    B30 = 30,
    B33 = 33,
    B35 = 35,
    B36 = 36,
    B40 = 40,
    B44 = 44,
    B46 = 46,
    B48 = 48,
    B50 = 50,
    B54 = 54,
    B55 = 55,
    B59 = 59,
    B60 = 60,
    B63 = 63,
    B66 = 66,
    B70 = 70,
    B72 = 72,
    B77 = 77,
    B80 = 80,
    B81 = 81,
    B84 = 84,
    B88 = 88,
    B90 = 90,
    B92 = 92,
    B96 = 96,
    B100 = 100,
    B102 = 102,
    B108 = 108,
    B110 = 110,
}

pub struct Announcement {
    pub game: AnnounceSuit,
    pub hand: bool,
    pub schneider: bool,
    pub schwarz: bool,
}
pub enum AnnounceSuit {
    Grand,
    Null,
    Suit(Suit),
}

#[derive(PartialEq, Clone, Copy)]
pub enum Party {
    Re,
    Contra,
}
