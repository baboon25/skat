use rand_distr::NormalError;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

mod deck;
mod player;
mod game;

pub enum RandError{
    NormalError(NormalError)
}

// Panic-Meldungen in der Browser-Konsole anzeigen
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    log("skat WASM geladen");

    // Asynchronen Task starten (läuft auf dem Browser-Event-Loop)
    spawn_local(async {
        init().await;
    });
}

async fn init() {
    log("Async init gestartet");

    // Beispiel: 1 Sekunde warten (nicht-blockierend)
    gloo_timers::future::TimeoutFuture::new(1_000).await;

    log("1 Sekunde vergangen – async runtime funktioniert");
}

/// Wird von JavaScript aufgerufen; gibt ein Promise zurück
#[wasm_bindgen]
pub async fn fetch_data(url: String) -> Result<JsValue, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("kein window"))?;
    let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_str(&url)).await?;
    let resp: web_sys::Response = resp_value.dyn_into()?;
    let json = wasm_bindgen_futures::JsFuture::from(resp.json()?).await?;
    Ok(json)
}

fn log(msg: &str) {
    web_sys::console::log_1(&JsValue::from_str(msg));
}
