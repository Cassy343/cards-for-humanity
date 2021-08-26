use std::sync::{Arc, Mutex};

use common::protocol::serverbound::ServerBoundPacket;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::HtmlElement;

use crate::{
    game::{GameManager, GameState},
    html::{
        get_name_input,
        get_name_input_value,
        get_settings,
        hide_game_end,
        init_lobby,
        update_player_name,
    },
};

pub fn register_events(manager: Arc<Mutex<GameManager>>) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let input = get_name_input();
    let input_manager = manager.clone();

    let input_change = Closure::<dyn FnMut()>::new(move || {
        let manager_arc = input_manager.clone();
        let mut manager = manager_arc.lock().unwrap();
        let new_name = get_name_input_value();
        update_player_name(&manager.id.to_string(), &new_name);
        manager.player.name = new_name.clone();
    });

    input.set_onchange(Some(input_change.as_ref().unchecked_ref()));

    input_change.forget();

    let create_game_button: HtmlElement = document
        .get_element_by_id("confirm-game-button")
        .unwrap()
        .dyn_into()
        .unwrap();
    let start_game_button: HtmlElement = document
        .get_element_by_id("game-start-button")
        .unwrap()
        .dyn_into()
        .unwrap();
    let start_game_button_clone = start_game_button.clone();
    let create_game_manager = manager.clone();

    let create_game_click = Closure::<dyn FnMut()>::new(move || {
        let manager_arc = create_game_manager.clone();
        let mut manager = manager_arc.lock().unwrap();
        let socket = manager.socket.lock().unwrap();
        let id = socket
            .send_packet_with_id(ServerBoundPacket::CreateServer(get_settings()))
            .unwrap();
        drop(socket);
        manager.packet_cache.insert(
            id,
            crate::game::CachedPacket::CreateServer(start_game_button_clone.clone()),
        );
    });

    create_game_button.set_onclick(Some(create_game_click.as_ref().unchecked_ref()));
    create_game_click.forget();

    let start_game_button_clone = start_game_button.clone();
    let start_game_manager = manager.clone();

    let start_game_click = Closure::<dyn FnMut()>::new(move || {
        let manager_arc = start_game_manager.clone();
        let manager = manager_arc.lock().unwrap();
        let socket = manager.socket.lock().unwrap();
        socket.send_packet(&ServerBoundPacket::StartGame).unwrap();
        start_game_button_clone.set_hidden(true);
    });

    start_game_button.set_onclick(Some(start_game_click.as_ref().unchecked_ref()));
    start_game_click.forget();

    let exit_button: HtmlElement = document
        .get_element_by_id("exit-button")
        .unwrap()
        .dyn_into()
        .unwrap();
    let exit_manager = manager.clone();

    let exit_game_click = Closure::<dyn FnMut()>::new(move || {
        let manager_arc = exit_manager.clone();
        let mut manager = manager_arc.lock().unwrap();
        manager.state = GameState::Lobby;
        let socket = manager.socket.lock().unwrap();
        socket.send_packet(&ServerBoundPacket::LeaveGame).unwrap();
        hide_game_end();
        init_lobby();
    });

    exit_button.set_onclick(Some(exit_game_click.as_ref().unchecked_ref()));
    exit_game_click.forget();

    let play_again_button: HtmlElement = document
        .get_element_by_id("play-again-button")
        .unwrap()
        .dyn_into()
        .unwrap();
    let again_manager = manager.clone();

    let again_click = Closure::<dyn FnMut()>::new(move || {
        let manager_arc = again_manager.clone();
        let manager = manager_arc.lock().unwrap();
        hide_game_end();
        if manager.host == manager.id {
            start_game_button.clone().set_hidden(false);
        }
    });

    play_again_button.set_onclick(Some(again_click.as_ref().unchecked_ref()));
    again_click.forget();

    let refresh_button: HtmlElement = document
        .get_element_by_id("refresh-button")
        .unwrap()
        .dyn_into()
        .unwrap();
    let refresh_manager = manager.clone();

    let refresh_click = Closure::<dyn FnMut()>::new(move || {
        let manager_arc = refresh_manager.clone();
        let manager = manager_arc.lock().unwrap();
        let socket = manager.socket.lock().unwrap();
        socket
            .send_packet(&ServerBoundPacket::RefreshServerList)
            .unwrap();
    });

    refresh_button.set_onclick(Some(refresh_click.as_ref().unchecked_ref()));
    refresh_click.forget();
}
