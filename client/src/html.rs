use common::protocol::{clientbound::ResponseData, GameSettings};
use js_sys::Array;
use uuid::Uuid;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{HtmlElement, HtmlInputElement};

use crate::game::{Player, PromptCard};

static RESPONSE_TEMPLATE: &'static str = include_str!("./templates/white_card.html");
static PROMPT_TEMPLATE: &'static str = include_str!("./templates/black_card.html");
static PLAYER_TEMPLATE: &'static str = include_str!("./templates/player.html");
static PLAYER_RESPONSE_TEMPLATE: &'static str = include_str!("./templates/responses.html");
static SERVER_TEMPLATE: &'static str = include_str!("./templates/server_entry.html");

// Template variables
// $ID the id of the card
// $TEXT the text of the card
// $HAND_INDEX the index in the hand, defaults to 99
pub fn response_card_html(card: &ResponseData) -> String {
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
pub fn player_html(player: &Player, id: &Uuid) -> String {
    PLAYER_TEMPLATE
        .replace("$ID", &format!("{}", id))
        .replace("$NAME", &player.name)
        .replace("$POINTS", &player.points.to_string())
}

// Template variables
// $ID the internal id of the user
pub fn player_response_html(id: &Uuid) -> String {
    PLAYER_RESPONSE_TEMPLATE.replace("$ID", &id.to_string())
}

pub fn server_html(id: &Uuid, player_count: usize, max_players: usize) -> String {
    SERVER_TEMPLATE
        .replace("$SERVER_ID", &id.to_string())
        .replace("$PLAYER_NUM", &player_count.to_string())
        .replace("$MAX_PLAYERS", &max_players.to_string())
}

pub fn init_game() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let settings = document
        .get_element_by_id("settings-menu")
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap();
    let lobby = document
        .get_element_by_id("lobby")
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap();
    let game = document
        .get_element_by_id("game")
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap();
    settings.set_hidden(true);
    lobby.set_hidden(true);
    game.set_hidden(false);
}

pub fn init_lobby() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let settings = document
        .get_element_by_id("settings-menu")
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap();
    let lobby = document
        .get_element_by_id("lobby")
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap();
    let game = document
        .get_element_by_id("game")
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap();
    settings.set_hidden(true);
    lobby.set_hidden(false);
    game.set_hidden(true);
}

pub fn add_server(server_id: &Uuid, num_players: usize, max_players: Option<usize>) -> HtmlElement {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let server_list = document.get_element_by_id("server-list").unwrap();

    let base_element = document.create_element("div").unwrap();
    server_list.append_child(&base_element).unwrap();
    let server_entry = server_list.last_element_child().unwrap();
    server_entry.set_outer_html(&server_html(
        server_id,
        num_players,
        max_players.unwrap_or(0),
    ));

    if max_players.is_some() {
        let span: HtmlElement = document
            .get_element_by_id(&format!("{}-max-players", server_id))
            .unwrap()
            .dyn_into()
            .unwrap();

        span.set_hidden(false);
    }

    server_list
        .last_element_child()
        .unwrap()
        .dyn_into()
        .unwrap()
}

pub fn get_name_input() -> HtmlElement {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    document
        .get_element_by_id("player-name-input-lobby")
        .unwrap()
        .dyn_into()
        .unwrap()
}

pub fn get_name_input_value() -> String {
    let input: HtmlInputElement = get_name_input().dyn_into().unwrap();

    input.value()
}

pub fn set_player_responses(id: &Uuid, cards: &Vec<ResponseData>) -> HtmlElement {
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

pub fn add_card_to_hand(card: &ResponseData) -> HtmlElement {
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

pub fn get_selected_new_packs() -> Vec<String> {
    let arr: Array = get_selected_packs(true).dyn_into().unwrap();

    let mut output = Vec::new();

    for i in 0 .. arr.length() {
        output.push(arr.get(i).as_string().unwrap())
    }

    output
}

pub fn add_packs(packs: Vec<String>) {
    for pack in packs {
        add_pack(pack);
    }
}

pub fn current_packs() -> Vec<String> {
    let arr: Array = get_current_packs().dyn_into().unwrap();
    let mut output = Vec::new();

    for i in 0 .. arr.length() {
        output.push(arr.get(i).as_string().unwrap())
    }

    output
}

pub fn get_settings() -> GameSettings {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let max_players_ele: HtmlInputElement = document
        .get_element_by_id("max-players")
        .unwrap()
        .dyn_into()
        .unwrap();
    // let max_time_ele: HtmlInputElement = document.get_element_by_id("max-time").unwrap().dyn_into().unwrap();
    let points_ele: HtmlInputElement = document
        .get_element_by_id("points")
        .unwrap()
        .dyn_into()
        .unwrap();

    let max_players = max_players_ele.value().parse().ok();
    // let max_time = max_time_ele.value().parse().ok();
    let points = points_ele.value().parse().unwrap();
    let packs = current_packs();

    GameSettings {
        packs,
        max_players,
        // Since we don't support max_selection_time yet we don't enable it
        max_selection_time: None,
        points_to_win: points,
    }
}

#[wasm_bindgen]
extern "C" {
    // All ids are sent in as &str
    // This is because sending Rust types to JS is a pain
    // And since they get simplified to JS strings anyway when used this doesn't matter
    fn get_selected_packs(new_packs: bool) -> JsValue;
    fn add_pack(new_pack: String);
    pub fn set_user_points(points: u32);
    pub fn set_user_name(name: &str);
    pub fn clear_player_marks(id: &str);
    pub fn mark_player_czar(id: &str);
    pub fn mark_player_played(id: &str);
    pub fn update_player_name(id: &str, name: &str);
    pub fn update_player_points(id: &str, points: u32);
    pub fn remove_card_from_hand(index: u8);
    pub fn remove_player(id: &str);
    pub fn clear_response_cards();
    pub fn place_blank_response();
    pub fn clear_servers();
    fn get_current_packs() -> JsValue;
}
