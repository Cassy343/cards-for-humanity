mod ws;
#[macro_use]
mod console;

use wasm_bindgen::prelude::*;
use ws::WebSocket;

#[wasm_bindgen]
pub fn client_main() {
    let socket = WebSocket::connect("ws://127.0.0.1:8080/ws").unwrap();

    socket.onopen(|| console_log!("Socket opened"));
    socket.onmessage(|event| console_log!("{:?}", event.data()));
}
