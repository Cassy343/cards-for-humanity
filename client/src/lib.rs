#![feature(unsize)]

mod ws;
#[macro_use]
mod console;

use wasm_bindgen::prelude::*;
use ws::WebSocket;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn client_main() {
    let socket = WebSocket::connect("ws://127.0.0.1:8080/ws").unwrap();

    socket.onopen(|socket| {
        console_log!("Socket opened");
        let _ = socket.send_packet(&common::protocol::serverbound::ServerBoundPacket::StartGame);
    });
    socket.onmessage(|_socket, event| console_log!("{:?}", event.data()));
    socket.onclose(|_socket, event| console_log!("{:?}", event));
    socket.onerror(|_socket, event| console_error!("WebSocket error: {}", event.message()));
}
