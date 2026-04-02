use rand_distr::NormalError;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

mod deck;
mod game;
mod helpers;
mod player;

use deck::Suit;
use game::{DefaultScore, Game};
use rand::SeedableRng;
use rand::rngs::SmallRng;

use crate::{
    helpers::set_status,
    player::{AnnounceSuit, Announcement, Player, SharedHand, bid_from_value, human::HumanController, resolve_announce, resolve_bid, resolve_listen, resolve_play, resolve_take_off},
};

#[derive(Debug)]
pub enum RandError {
    NormalError(NormalError),
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}

/// Startet ein neues Spiel.
#[wasm_bindgen]
pub fn start_game() {
    spawn_local(async {
        set_status("Spiel wird gestartet...");
        let mut rng = SmallRng::from_rng(&mut rand::rng());
        let hand = SharedHand::new(Default::default());
        let human = Player::new(HumanController::new(hand.clone()), hand);
        let ai1 = Player::default();
        let ai2 = Player::default();

        let mut game = Game::<DefaultScore>::new(human, ai1, ai2, 1);
        match game.play(&mut rng).await {
            Ok(_) => set_status("Spiel beendet!"),
            Err(e) => set_status(&format!("Fehler: {:?}", e)),
        }
    });
}

/// Spieler spielt Karte mit Index `i` aus seiner Hand.
#[wasm_bindgen]
pub fn play_card(i: usize) {
    resolve_play(i);
}

/// Spieler bietet `value` (0 = Pass).
#[wasm_bindgen]
pub fn submit_bid(value: u8) {
    let bid = bid_from_value(value).unwrap_or(player::Bid::Pass);
    resolve_bid(bid);
}

/// Spieler hört beim Reizen (true = hören, false = weg).
#[wasm_bindgen]
pub fn submit_listen(listens: bool) {
    resolve_listen(listens);
}

/// Spieler sagt an (0=Grand, 1=Null, 2=Kreuz, 3=Pik, 4=Herz, 5=Karo).
#[wasm_bindgen]
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
    resolve_announce(Announcement {
        game: suit,
        hand: false,
        schneider: false,
        schwarz: false,
    });
}

/// Spieler hebt ab bei Position `idx`.
#[wasm_bindgen]
pub fn submit_take_off(idx: usize) {
    resolve_take_off(idx);
}
