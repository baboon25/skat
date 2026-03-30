use std::default;

use crate::{
    deck::{Card, CardError, Deck, Skat, Suit},
    game::GameError,
};

#[async_trait::async_trait]
pub trait PlayerController {
    async fn play(&mut self, previous_played_cards: &[Card]) -> Card;
    async fn bid(&mut self, current_bid: Bid) -> Bid;
    async fn listen(&mut self, bid: Bid) -> bool;
    async fn announce(&mut self, hand: &[Card]) -> Announcement;
    async fn take_off(&mut self, deck_size: usize) -> usize;
}

#[derive(Debug, Default)]
pub struct AiController {
    hand: Vec<Card>,
}

#[async_trait::async_trait]
impl PlayerController for AiController {
    async fn play(&mut self, previous: &[Card]) -> Card {
        // Einfache Strategie: wenn anführend → höchste Karte
        // wenn folgend → niedrigste Karte die sticht, sonst niedrigste
        todo!()
    }

    async fn bid(&mut self, current_bid: Bid) -> Bid {
        Bid::Pass // simpel: KI passt immer
    }

    async fn listen(&mut self, bid: Bid) -> bool {
        false
    }

    async fn announce(&mut self, hand: &[Card]) -> Announcement {
        todo!() // basierend auf Handstärke
    }
    
    async fn take_off(&mut self, deck_size: usize) -> usize{
        todo!()
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
            ..Default::default()
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
