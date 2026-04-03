use std::cell::RefCell;

use futures::channel::oneshot;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    deck::{Card, CardError, Suit},
    helpers::*,
    player::{Announcement, AnnounceSuit, Bid, PlayerController, SharedHand, bid_from_value},
};

// ── thread-local channels ─────────────────────────────────────────────────────

thread_local! {
    static PLAY_TX:     RefCell<Option<oneshot::Sender<usize>>>        = const { RefCell::new(None) };
    static BID_TX:      RefCell<Option<oneshot::Sender<Bid>>>          = const { RefCell::new(None) };
    static LISTEN_TX:   RefCell<Option<oneshot::Sender<bool>>>         = const { RefCell::new(None) };
    static ANNOUNCE_TX: RefCell<Option<oneshot::Sender<Announcement>>> = const { RefCell::new(None) };
    static TAKE_OFF_TX: RefCell<Option<oneshot::Sender<usize>>>        = const { RefCell::new(None) };
    static SKAT_TX:     RefCell<Option<oneshot::Sender<[usize; 2]>>>   = const { RefCell::new(None) };
    static SKAT_FIRST:  RefCell<Option<usize>>                         = const { RefCell::new(None) };
}

// ── LocalPlayer ───────────────────────────────────────────────────────────────

#[wasm_bindgen]
pub struct LocalPlayer {
    hand: SharedHand,
}

impl LocalPlayer {
    pub fn new(hand: SharedHand) -> Self {
        Self { hand }
    }
}

/// JS-callable static methods
#[wasm_bindgen]
impl LocalPlayer {
    /// Spieler spielt Karte mit Index `idx`.
    pub fn play_card(idx: usize) {
        PLAY_TX.with(|t| { t.borrow_mut().take().map(|tx| tx.send(idx).ok()); });
    }

    /// Spieler bietet `value` (0 = Pass).
    pub fn submit_bid(value: u8) {
        let bid = bid_from_value(value).unwrap_or(Bid::Pass);
        BID_TX.with(|t| { t.borrow_mut().take().map(|tx| tx.send(bid).ok()); });
    }

    /// Spieler hört (true) oder passt (false).
    pub fn submit_listen(listens: bool) {
        LISTEN_TX.with(|t| { t.borrow_mut().take().map(|tx| tx.send(listens).ok()); });
    }

    /// Spieler sagt an (0=Grand, 1=Null, 2=Kreuz, 3=Pik, 4=Herz, 5=Karo).
    pub fn submit_announce(game_type: u8) {
        let suit = match game_type {
            0 => AnnounceSuit::Grand,
            1 => AnnounceSuit::Null,
            2 => AnnounceSuit::Suit(Suit::Clubs),
            3 => AnnounceSuit::Suit(Suit::Spades),
            4 => AnnounceSuit::Suit(Suit::Hearts),
            5 => AnnounceSuit::Suit(Suit::Diamonds),
            _ => AnnounceSuit::Grand,
        };
        ANNOUNCE_TX.with(|t| {
            t.borrow_mut().take().map(|tx| tx.send(Announcement {
                game: suit, hand: false, schneider: false, schwarz: false,
            }).ok());
        });
    }

    /// Spieler hebt ab bei Position `idx`.
    pub fn submit_take_off(idx: usize) {
        TAKE_OFF_TX.with(|t| { t.borrow_mut().take().map(|tx| tx.send(idx).ok()); });
    }

    /// Spieler wählt eine Skatkarte (idx). Beim zweiten Aufruf wird der Skat bestätigt.
    pub fn select_skat_card(idx: usize) {
        SKAT_FIRST.with(|first| {
            let mut f = first.borrow_mut();
            if let Some(first_idx) = f.take() {
                SKAT_TX.with(|t| { t.borrow_mut().take().map(|tx| tx.send([first_idx, idx]).ok()); });
            } else {
                *f = Some(idx);
            }
        });
    }
}

#[async_trait::async_trait(?Send)]
impl PlayerController for LocalPlayer {
    async fn play(&mut self, previous: &[Card], announcement: &Announcement) -> Result<Card, CardError> {
        hide("bid-area");
        hide("announce-area");
        hide("take-off-area");
        show("hand");
        loop {
            render_game_state(&*self.hand.borrow(), previous);
            set_status("Wähle eine Karte zum Ausspielen");
            let (tx, rx) = oneshot::channel();
            PLAY_TX.with(|t| *t.borrow_mut() = Some(tx));
            let idx = rx.await.unwrap();
            let card = self.hand.borrow()[idx];
            let lead = previous.first().copied();
            if lead.is_none() || card.is_legal(&*self.hand.borrow(), lead.unwrap(), announcement) {
                return Ok(std::mem::take(&mut self.hand.borrow_mut()[idx]));
            }
            set_status("Ungültige Karte — du musst Farbe bekennen!");
        }
    }

    async fn bid(&mut self, current_bid: Bid) -> Bid {
        render_game_state(&*self.hand.borrow(), &[]);
        show("hand");
        hide("announce-area");
        hide("take-off-area");
        show("bid-area");
        set_html("bid-current", &format!("Aktuelles Gebot: <b>{}</b>", current_bid as u8));
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
        set_html("bid-current", &format!("Gegner bietet: <b>{}</b>", bid as u8));
        set_status("Hören oder weg?");
        set_html("bid-mode", "listen");
        let (tx, rx) = oneshot::channel();
        LISTEN_TX.with(|t| *t.borrow_mut() = Some(tx));
        rx.await.unwrap()
    }

    async fn announce(&mut self) -> Announcement {
        render_game_state(&*self.hand.borrow(), &[]);
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

    async fn select_skat(&mut self) -> [usize; 2] {
        hide("bid-area");
        hide("announce-area");
        hide("take-off-area");
        render_game_state(&*self.hand.borrow(), &[]);
        show("skat-area");
        show("hand");
        set_status("Wähle zwei Karten für den Skat");
        SKAT_FIRST.with(|f| f.borrow_mut().take());
        let (tx, rx) = oneshot::channel();
        SKAT_TX.with(|t| *t.borrow_mut() = Some(tx));
        let indices = rx.await.unwrap();
        hide("skat-area");
        indices
    }
}
