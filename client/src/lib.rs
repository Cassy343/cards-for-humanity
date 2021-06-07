#![feature(unsize)]

#[macro_use]
mod console;
mod game;
mod html;
mod ws;

use common::protocol::{clientbound::ClientBoundPacket, decode};
use std::sync::mpsc;
use wasm_bindgen::prelude::*;
use ws::WebSocket;

use crate::game::game_init;

#[wasm_bindgen]
pub fn client_main() {
    console_error_panic_hook::set_once();

    let host = web_sys::window().unwrap().location().host().unwrap();
    let socket = WebSocket::connect(&format!("ws://{}/ws", host)).unwrap();
    let (packet_pipe, packet_receiver) = mpsc::channel::<ClientBoundPacket>();

    socket.onopen(move |socket| {
        console_log!("Socket opened");
        let _ =
            socket.send_packet_with_id(common::protocol::serverbound::ServerBoundPacket::StartGame);
    });
    socket.onmessage(move |_socket, event| {
        let packet_string = match event.data().as_string() {
            Some(string) => string,
            None => return,
        };

        match decode::<'_, Vec<ClientBoundPacket>>(&packet_string) {
            Ok(packets) =>
                for packet in packets {
                    if let Err(e) = packet_pipe.send(packet) {
                        console_error!("Failed to forward packet to handler: {}", e);
                    }
                },

            Err(error) => console_error!(
                "Failed to parse packets: {}, raw data: {}",
                error,
                packet_string
            ),
        }
    });
    socket.onclose(|_socket, event| console_log!("{:?}", event));
    socket.onerror(|_socket, event| console_error!("WebSocket error: {}", event.message()));

    game_init(socket, packet_receiver)
}
