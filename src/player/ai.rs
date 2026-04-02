use crate::{deck::{Card, CardError, Suit}, player::{AnnounceSuit, Announcement, Bid, PlayerController, SharedHand}};

#[derive(Debug, Default)]
pub struct AiController {
    hand: SharedHand
}

impl AiController{
    pub fn new(hand: SharedHand) -> Self{
        Self{hand}
    }
}

#[async_trait::async_trait]
impl PlayerController for AiController {
    async fn play(&mut self, previous: &[Card]) -> Result<Card, CardError> {
        web_sys::console::log_1(&format!("AI hand: {:?}", *self.hand.lock().await).into());
        if previous.is_empty() {
            Ok(std::mem::take(self.hand.lock().await.iter_mut().find(|c| c.has_rank() && c.has_suit()).ok_or(CardError::Uninitialized)?))


            // let idx = self.hand.lock().await.iter().enumerate()
            //     .max_by_key(|(_, c)| c.get_rank().map(|r| r as u8).unwrap_or(0))
            //     .map(|(i, _)| i)
            //     .unwrap_or(0);
            // std::mem::take(&mut self.hand.lock().await[idx])
        } else {
            Ok(std::mem::take(self.hand.lock().await.iter_mut().filter(|c| c.has_rank() && c.has_suit())
                .min_by_key(|c| c.get_rank()).ok_or(CardError::Uninitialized)?))
        }
    }

    async fn bid(&mut self, _current_bid: Bid) -> Bid {
        Bid::Pass
    }

    async fn listen(&mut self, _bid: Bid) -> bool {
        false
    }

    async fn announce(&mut self) -> Announcement {
        Announcement { game: AnnounceSuit::Grand, hand: false, schneider: false, schwarz: false }
    }

    async fn take_off(&mut self, deck_size: usize) -> usize {
        deck_size / 2
    }
}