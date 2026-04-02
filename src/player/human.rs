use futures::channel::oneshot;

use crate::{
    deck::{Card, CardError, Suit},
    helpers::*,
    player::{ANNOUNCE_TX, Announcement, BID_TX, Bid, ControllerError, LISTEN_TX, PLAY_TX, Player, PlayerController, SharedHand, TAKE_OFF_TX},
};

pub struct HumanController {
    hand: SharedHand,
}

impl HumanController {
    pub fn new(hand: SharedHand) -> Self {
        Self { hand }
    }
}

#[async_trait::async_trait]
impl PlayerController for HumanController {
    async fn play(&mut self, previous: &[Card]) -> Result<Card, CardError> {
        render_game_state(&*self.hand.lock().await, previous);

        // TABLE_CARDS.with(|t| *t.borrow_mut() = previous.to_vec());
        // render_game_state();
        hide("bid-area");
        hide("announce-area");
        hide("take-off-area");
        show("hand");
        set_status("Wähle eine Karte zum Ausspielen");
        let (tx, rx) = oneshot::channel();
        PLAY_TX.with(|t| *t.borrow_mut() = Some(tx));
        Ok(rx.await.unwrap())
    }

    async fn bid(&mut self, current_bid: Bid) -> Bid {
        // hide("hand");
        render_game_state(&*self.hand.lock().await, &[]);
        show("hand");
        hide("announce-area");
        hide("take-off-area");
        show("bid-area");
        set_html(
            "bid-current",
            &format!("Aktuelles Gebot: <b>{}</b>", current_bid as u8),
        );
        set_status("Bieten oder passen?");
        let (tx, rx) = oneshot::channel();
        BID_TX.with(|t| *t.borrow_mut() = Some(tx));
        rx.await.unwrap()
    }

    async fn listen(&mut self, bid: Bid) -> bool {
        hide("hand");
        hide("announce-area");
        hide("take-off-area");
        show("bid-area");
        set_html(
            "bid-current",
            &format!("Gegner bietet: <b>{}</b>", bid as u8),
        );
        set_status("Hören oder weg?");
        set_html("bid-mode", "listen");
        let (tx, rx) = oneshot::channel();
        LISTEN_TX.with(|t| *t.borrow_mut() = Some(tx));
        rx.await.unwrap()
    }

    async fn announce(&mut self) -> Announcement {
        // let valid: Vec<Card> = hand
        //     .iter()
        //     .filter(|c| c.get_suit() != Suit::None)
        //     .copied()
        //     .collect();
        // self.hand.b.with(|h| *h.borrow_mut() = valid);
        // render_game_state(&*self.hand.borrow(), );
        hide("bid-area");
        hide("take-off-area");
        show("hand");
        show("announce-area");
        set_status("Wähle dein Spiel");
        let (tx, rx) = oneshot::channel();
        ANNOUNCE_TX.with(|t| *t.borrow_mut() = Some(tx));
        rx.await.unwrap()
    }

    async fn take_off(&mut self, deck_size: usize) -> usize {
        hide("bid-area");
        hide("announce-area");
        hide("hand");
        show("take-off-area");
        set_html("take-off-max", &deck_size.to_string());
        set_status("Hebe ab");
        let (tx, rx) = oneshot::channel();
        TAKE_OFF_TX.with(|t| *t.borrow_mut() = Some(tx));
        rx.await.unwrap()
    }
}
