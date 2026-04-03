use std::{cell::RefCell, rc::Rc};

use crate::{
    deck::{Card, CardError, Deck, Suit},
    player::ai::AiController,
};

pub mod ai;
pub mod local;

pub type SharedHand = Rc<RefCell<Vec<Card>>>;

#[async_trait::async_trait(?Send)]
pub trait PlayerController {
    async fn play(
        &mut self,
        previous: &[Card],
        announcement: &Announcement,
    ) -> Result<Card, CardError>;
    async fn bid(&mut self, current_bid: Bid) -> Bid;
    async fn listen(&mut self, bid: Bid) -> bool;
    async fn announce(&mut self) -> Announcement;
    async fn take_off(&mut self, deck_size: usize) -> usize;
    async fn select_skat(&mut self) -> [usize; 2];
}

pub enum ControllerError {
    NoControllerProvided,
}

pub struct Player {
    pub hand: SharedHand,
    pub party: Option<Party>,
    pub score: i32,
    pub tricks: Vec<Card>,
    pub controller: Box<dyn PlayerController>,
}

impl Player {
    pub fn new<C: PlayerController + 'static>(controller: C, hand: SharedHand) -> Self {
        Self {
            controller: Box::new(controller),
            hand,
            party: None,
            score: 0,
            tricks: Vec::new(),
        }
    }

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
        self.controller.announce().await
    }

    pub async fn play<'b>(
        &'b mut self,
        op1: &'b mut Player,
        op2: &'b mut Player,
        announcement: &Announcement,
    ) -> Result<*const Player, CardError> {
        let played_card = self.controller.play(&[], announcement).await?;
        let op1_card = op1.controller.play(&[played_card], announcement).await?;
        let op2_card = op2
            .controller
            .play(&[played_card, op1_card], announcement)
            .await?;

        if played_card.surpasses(&op1_card, announcement)?
            && played_card.surpasses(&op2_card, announcement)?
        {
            self.tricks
                .append(&mut vec![played_card, op1_card, op2_card]);
            return Ok(self);
        }
        if op1_card.surpasses(&played_card, announcement)?
            && op1_card.surpasses(&op2_card, announcement)?
        {
            op1.tricks
                .append(&mut vec![played_card, op1_card, op2_card]);
            return Ok(op1);
        }
        op2.tricks
            .append(&mut vec![played_card, op1_card, op2_card]);
        Ok(op2)
    }

    pub async fn push_skat(&mut self) {
        let indices = self.controller.select_skat().await;
        for idx in indices {
            self.tricks.push(std::mem::take(&mut self.hand.borrow_mut()[idx]));
        }
    }
}

impl Default for Player {
    fn default() -> Self {
        let hand: SharedHand = Rc::new(RefCell::new(Vec::with_capacity(12)));
        Self {
            controller: Box::new(AiController::new(hand.clone())),
            hand,
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

pub const ALL_BIDS: &[Bid] = &[
    Bid::B18,
    Bid::B20,
    Bid::B22,
    Bid::B23,
    Bid::B24,
    Bid::B27,
    Bid::B30,
    Bid::B33,
    Bid::B35,
    Bid::B36,
    Bid::B40,
    Bid::B44,
    Bid::B46,
    Bid::B48,
    Bid::B50,
    Bid::B54,
    Bid::B55,
    Bid::B59,
    Bid::B60,
    Bid::B63,
    Bid::B66,
    Bid::B70,
    Bid::B72,
    Bid::B77,
    Bid::B80,
    Bid::B81,
    Bid::B84,
    Bid::B88,
    Bid::B90,
    Bid::B92,
    Bid::B96,
    Bid::B100,
    Bid::B102,
    Bid::B108,
    Bid::B110,
];

pub fn bid_from_value(v: u8) -> Option<Bid> {
    ALL_BIDS.iter().copied().find(|b| *b as u8 == v)
}

// ── Announcement, AnnounceSuit, Party ─────────────────────────────────────────

#[derive(Debug, Default)]
pub struct Announcement {
    pub game: AnnounceSuit,
    pub hand: bool,
    pub schneider: bool,
    pub schwarz: bool,
}

#[derive(Debug)]
pub enum AnnounceSuit {
    Grand,
    Null,
    Suit(Suit),
}

impl Default for AnnounceSuit {
    fn default() -> Self {
        Self::Suit(Default::default())
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum Party {
    Re,
    Contra,
}
