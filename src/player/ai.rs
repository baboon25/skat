use crate::{
    deck::{Card, CardError},
    player::{AnnounceSuit, Announcement, Bid, PlayerController, SharedHand},
};

#[derive(Debug, Default)]
pub struct AiController {
    hand: SharedHand,
}

impl AiController {
    pub fn new(hand: SharedHand) -> Self {
        Self { hand }
    }
}

#[async_trait::async_trait(?Send)]
impl PlayerController for AiController {
    async fn play(
        &mut self,
        previous: &[Card],
        announcement: &Announcement,
    ) -> Result<Card, CardError> {
        let hand = &mut *self.hand.borrow_mut();
        if previous.is_empty() {
            Ok(std::mem::take(
                hand.iter_mut()
                    .find(|c| c.has_rank() && c.has_suit())
                    .ok_or(CardError::Uninitialized)?,
            ))
        } else {
            let lookup = *hand;
            Ok(std::mem::take(
                hand.iter_mut()
                    .filter(|c| c.has_rank() && c.has_suit() && c.is_legal(&lookup, previous[0], announcement))
                    .min_by_key(|c| c.get_rank())
                    .ok_or(CardError::Uninitialized)?,
            ))
        }
    }

    async fn bid(&mut self, _current_bid: Bid) -> Bid {
        Bid::Pass
    }

    async fn listen(&mut self, _bid: Bid) -> bool {
        false
    }

    async fn announce(&mut self) -> Announcement {
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

    async fn select_skat(&mut self) -> [usize; 2] {
        let hand = self.hand.borrow();
        let mut indices: Vec<usize> = hand
            .iter()
            .enumerate()
            .filter(|(_, c)| c.has_rank() && c.has_suit())
            .map(|(i, _)| i)
            .collect();
        indices.sort_by_key(|&i| hand[i].get_rank());
        [indices[0], indices[1]]
    }
}
