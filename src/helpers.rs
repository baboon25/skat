use crate::{
    deck::{Card, Rank, Suit},
};

pub fn card_label(card: &Card) -> String {
    let suit = match card.get_suit() {
        Suit::Clubs => "♣",
        Suit::Spades => "♠",
        Suit::Hearts => "♥",
        Suit::Diamonds => "♦",
        Suit::None => "?",
    };
    let rank = match card.get_rank() {
        Rank::Ace => "A",
        Rank::Ten => "10",
        Rank::King => "K",
        Rank::Queen => "Q",
        Rank::Jack => "J",
        Rank::Nine => "9",
        Rank::Eight => "8",
        Rank::Seven => "7",
        Rank::None => "?",
    };
    let color_class = match card.get_suit() {
        Suit::Hearts | Suit::Diamonds => "red",
        _ => "black",
    };
    format!("<span class='card {}'>{}{}</span>", color_class, suit, rank)
}

pub fn get_el(id: &str) -> Option<web_sys::Element> {
    web_sys::window()?.document()?.get_element_by_id(id)
}

pub fn set_html(id: &str, html: &str) {
    if let Some(el) = get_el(id) {
        el.set_inner_html(html);
    }
}

pub fn set_style(id: &str, style: &str) {
    if let Some(el) = get_el(id) {
        el.set_attribute("style", style).ok();
    }
}

pub fn show(id: &str) {
    set_style(id, "display: block");
}
pub fn hide(id: &str) {
    set_style(id, "display: none");
}
pub fn set_status(msg: &str) {
    set_html("status", msg);
}

pub fn render_game_state(hand: &[Card], table: &[Card]) {
    let hand_html = hand
        .iter()
        .enumerate()
        .map(|(i, c)| {
            format!(
                "<button class='card-btn' onclick='window.skatApp.playCard({i})'>{}</button>",
                card_label(c)
            )
        })
        .collect::<Vec<_>>()
        .join("");
    set_html("hand", &hand_html);

    let table_html = table.iter().map(card_label).collect::<Vec<_>>().join("");
    set_html("table-cards", &table_html);
}
