use wasm_bindgen::prelude::*;
use js_sys::Array;
use crate::game::{PromptCard, ResponseCard};

#[wasm_bindgen(module = "/../frontend/dist/app.js")]
extern "C" {
    pub fn update_black_card(text: PromptCard);
    #[wasm_bindgen(js_name = "update_played_cards")]
    pub fn update_played_cards_sys(played_cards: JsValue);
    #[wasm_bindgen(js_name = "update_hand")]
    pub fn update_hand_sys(hand: JsValue);
}

pub fn update_played_cards(played_cards: Vec<ResponseCard>) {
    update_played_cards_sys(JsValue::from_serde(&played_cards).unwrap())
}

pub fn update_hand(hand: Vec<ResponseCard>) {
    update_hand_sys(JsValue::from_serde(&hand).unwrap())
}