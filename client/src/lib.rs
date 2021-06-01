#![feature(unsize)]

mod game;
// mod rendering;
mod ws;

#[macro_use]
mod console;

use std::sync::{Arc, mpsc};
use common::protocol::{clientbound::ClientBoundPacket, decode};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::*;
use web_sys::{Request, RequestInit, RequestMode, Response};
use ws::WebSocket;

use crate::game::game_init;

#[wasm_bindgen]
pub fn client_main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let socket = WebSocket::connect("ws://127.0.0.1:8080/ws").unwrap();
    let (packet_pipe, packet_receiver) = mpsc::channel::<ClientBoundPacket>();

    socket.onopen(|socket| {
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

    spawn_local(game_init(socket, Arc::new(packet_receiver)));
}

pub async fn fetch(url: &str) -> Result<String, JsValue> {
    let mut options = RequestInit::new();
    options.method("GET");
    options.mode(RequestMode::NoCors);
    let req = Request::new_with_str_and_init(url, &options)?;

    let window = web_sys::window().unwrap();
    let request_promise = window.fetch_with_request(&req);
    let future = JsFuture::from(request_promise);

    let response = future.await?;
    let response: Response = response.dyn_into()?;

    let text_future = JsFuture::from(response.text()?);
    let text = text_future.await?;
    Ok(text
        .as_string()
        .expect("Response.text() did not return a String and did not error"))
}
