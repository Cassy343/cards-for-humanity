use wasm_bindgen::{JsCast, prelude::*};
use web_sys::{WebSocket, console};

#[wasm_bindgen]
pub fn client_main() {
    let socket = WebSocket::new("ws://127.0.0.1:8080/ws").unwrap();

    let onopen = Closure::wrap(Box::new(move || {
        console::log_1(&"here".into());
    }) as Box<dyn FnMut()>);

    socket.set_onopen(Some(onopen.as_ref().unchecked_ref()));
    onopen.forget();
}