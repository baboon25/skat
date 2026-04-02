use rand_distr::NormalError;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

mod deck;
mod game;
mod helpers;
mod player;

use game::{DefaultScore, Game};
use rand::SeedableRng;
use rand::rngs::SmallRng;

use crate::{
    helpers::set_status,
    player::{Player, SharedHand, local::LocalPlayer},
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
        let human = Player::new(LocalPlayer::new(hand.clone()), hand);
        let ai1 = Player::default();
        let ai2 = Player::default();

        let mut game = Game::<DefaultScore>::new(human, ai1, ai2, 1);
        match game.play(&mut rng).await {
            Ok(_) => set_status("Spiel beendet!"),
            Err(e) => set_status(&format!("Fehler: {:?}", e)),
        }
    });
}
