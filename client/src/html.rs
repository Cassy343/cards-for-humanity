use uuid::Uuid;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::game::{Player, PromptCard, ResponseCard};

static RESPONSE_TEMPLATE: &'static str = include_str!("./templates/white_card.html");
static PROMPT_TEMPLATE: &'static str = include_str!("./templates/black_card.html");
static PLAYER_TEMPLATE: &'static str = include_str!("./templates/player.html");
static PLAYER_RESPONSE_TEMPLATE: &'static str = include_str!("./templates/responses.html");
static GAME_PAGE: &'static str = include_str!("./templates/game.html");

// Template variables
// $ID the id of the card
// $TEXT the text of the card
// $HAND_INDEX the index in the hand, defaults to 99
pub fn response_card_html(card: &ResponseCard) -> String {
    RESPONSE_TEMPLATE
        .replace(
            "$ID",
            &format!("{}_{}", card.id.pack_number, card.id.card_number),
        )
        .replace("$TEXT", &card.text)
}

// Template variables
// $TEXT the text of the card
pub fn prompt_card_html(card: &PromptCard) -> String {
    PROMPT_TEMPLATE.replace("$TEXT", &card.text)
}

// Template variables
// $ID the internal id of the user
// $NAME the name of the user
// $POINTS the points of the user
pub fn player_html(player: &Player, uuid: &Uuid) -> String {
    PLAYER_TEMPLATE
        .replace("$ID", &format!("{}", uuid))
        .replace("$NAME", &player.name)
        .replace("$POINTS", &player.points.to_string())
}

// Template variables
// $ID the internal id of the user
pub fn player_response_html(uuid: &Uuid) -> String {
    PLAYER_RESPONSE_TEMPLATE.replace("$ID", &uuid.to_string())
}

pub fn init_game() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let root = document.get_element_by_id("root").unwrap();
    root.set_inner_html(GAME_PAGE);
}

pub fn set_player_responses(id: &Uuid, cards: &Vec<ResponseCard>) -> HtmlElement {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let responses = document.get_element_by_id("played-cards").unwrap();

    let base_element = document.create_element("div").unwrap();
    responses.append_child(&base_element).unwrap();
    let player_res = responses.last_element_child().unwrap();
    player_res.set_outer_html(&player_response_html(id));

    let player_res = document
        .get_element_by_id(&format!("player-{}-responses", id))
        .unwrap();

    player_res.set_inner_html(&cards.iter().fold(String::new(), |mut string, c| {
        string.push_str(&response_card_html(c));
        string
    }));

    player_res.dyn_into().unwrap()
}

pub fn place_blank_response() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let responses = document.get_element_by_id("played-cards").unwrap();

    responses.set_inner_html(&format!(
        "{}<div class=\"white-card card\"></div>",
        responses.inner_html()
    ));
}

pub fn clear_response_cards() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let responses = document.get_element_by_id("played-cards").unwrap();

    responses.set_inner_html("");
}

pub fn add_card_to_hand(card: &ResponseCard) -> HtmlElement {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let hand_div = document.get_element_by_id("hand").unwrap();

    let base_element = document.create_element("div").unwrap();
    hand_div.append_child(&base_element).unwrap();
    let card_element = hand_div.last_element_child().unwrap();
    card_element.set_outer_html(&response_card_html(card));


    document
        .get_element_by_id(&format!(
            "card-{}_{}",
            card.id.pack_number, card.id.card_number
        ))
        .unwrap()
        .dyn_into()
        .unwrap()
}

pub fn remove_card_from_hand(index: u8) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let hand_div = document.get_element_by_id("hand").unwrap();
    let hand_cards = hand_div.children();
    let to_remove = hand_cards.get_with_index(index as u32).unwrap();
    hand_div.remove_child(&to_remove).unwrap();
}

pub fn get_hand_element(index: usize) -> HtmlElement {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let hand_div = document.get_element_by_id("hand").unwrap();
    let hand_cards = hand_div.children();
    hand_cards
        .get_with_index(index as u32)
        .unwrap()
        .dyn_into()
        .unwrap()
}

pub fn set_prompt_card(card: &PromptCard) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let prompt_div = document.get_element_by_id("black-card-div").unwrap();
    prompt_div.set_inner_html(&prompt_card_html(card))
}

pub fn add_player(player: &Player, id: &Uuid) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let player_list = document.get_element_by_id("players-list").unwrap();
    player_list.set_inner_html(&format!(
        "{}{}",
        player_list.inner_html(),
        player_html(player, id)
    ))
}

pub fn remove_player(id: Uuid) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let player = document
        .get_element_by_id(&format!("player-{}", id))
        .unwrap();
    let player_list = document.get_element_by_id("players-list").unwrap();
    player_list.remove_child(&player).unwrap();
}

pub fn update_player_points(id: Uuid, points: u32) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let player_points = document
        .get_element_by_id(&format!("player-{}-points", id))
        .unwrap();
    player_points.set_inner_html(&points.to_string());
}

pub fn update_player_name(id: Uuid, name: &str) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let player_name = document
        .get_element_by_id(&format!("player-{}-name", id))
        .unwrap();
    player_name.set_inner_html(name);
}

pub fn mark_player_played(id: &Uuid) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let player = document
        .get_element_by_id(&format!("player-{}", id))
        .unwrap();
    player.class_list().add_1("picked").unwrap();
}

pub fn mark_player_czar(id: &Uuid) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let player = document
        .get_element_by_id(&format!("player-{}", id))
        .unwrap();
    player.class_list().add_1("czar").unwrap();
}

pub fn clear_player_marks(id: &Uuid) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let player = document
        .get_element_by_id(&format!("player-{}", id))
        .unwrap();

    player.class_list().remove_2("czar", "picked").unwrap();
}

pub fn set_user_name(name: &str) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let user_name = document.get_element_by_id("user-name").unwrap();
    user_name.set_inner_html(name);
}

pub fn set_user_points(points: u32) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let user_points = document.get_element_by_id("user-points").unwrap();
    user_points.set_inner_html(&points.to_string())
}
