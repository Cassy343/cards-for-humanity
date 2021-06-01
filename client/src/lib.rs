#![feature(unsize)]

mod game;
// mod rendering;
mod ws;
mod js_imports;

#[macro_use]
mod console;

use common::protocol::{clientbound::ClientBoundPacket, decode};
use game::game_init;
// use rendering::{
//     shapes::{RoundedRect, Text, TextBubble},
//     Color,
//     RenderManager,
//     Renderable,
// };
use std::sync::{mpsc, Arc};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::*;
use web_sys::{Request, RequestInit, RequestMode, Response, WebGlRenderingContext};
use ws::WebSocket;

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
    // spawn_local(async {render_test().await.unwrap()});
}


// pub async fn render_test() -> Result<(), JsValue> {
//     console_log!("Running the render test");
//     let window = web_sys::window().unwrap();
//     let document = window.document().unwrap();
//     let webgl_canvas = document.get_element_by_id("webgl").unwrap();
//     let webgl_canvas: web_sys::HtmlCanvasElement = webgl_canvas.dyn_into()?;
//     let text_canvas = document.get_element_by_id("text").unwrap();
//     let text_canvas: web_sys::HtmlCanvasElement = text_canvas.dyn_into()?;

//     let mut manager = RenderManager::new(&webgl_canvas, &text_canvas)?;

//     console_log!("making manager");
//     manager.register_shader(
//         "card",
//         &fetch("./shaders/card.vert")
//             .await
//             .expect("error getting shader"),
//         &fetch("./shaders/card.frag")
//             .await
//             .expect("error getting shader"),
//         WebGlRenderingContext::TRIANGLE_STRIP,
//     )?;

//     manager.set_background_color(Color::from_rgb(0x1e, 0x34, 0x54));
//     manager.clear();

//     let mut objects: Vec<&dyn Renderable> = Vec::new();

//     let rect = RoundedRect {
//         position: Vector2::new(0.0, 0.0),
//         dimensions: Vector2::new(500.0, 500.0),
//         color: Color::from_rgb(0xff, 0xff, 0xff),
//         radius: 0.125,
//     };

//     let text_bubble = TextBubble {
//         text: Text {
//             text: "This is a multiline\npiece of text!".to_owned(),
//             text_pos: rect.position,
//             width: None,
//             font: "Comic Sans MS".to_owned(),
//             font_size: 50,
//             fill_style: "Black".to_owned(),
//             outline: false,
//         },
//         rect,
//     };

//     objects.push(&text_bubble);

//     manager.draw_objects(objects)?;

//     let resize_callback = Closure::<dyn FnMut()>::new(move || {
//         let window = web_sys::window().unwrap();
//         let document = window.document().unwrap();
//         let text_canvas = document.get_element_by_id("text").unwrap();
//         let _: web_sys::HtmlCanvasElement = text_canvas.dyn_into().unwrap();
//         manager.clear();
//         let mut objects: Vec<&dyn Renderable> = Vec::new();
//         objects.push(&text_bubble);
//         manager.update_scale_factor();
//         manager.draw_objects(objects).unwrap();
//     });

//     window.add_event_listener_with_callback("resize", resize_callback.as_ref().unchecked_ref())?;

//     resize_callback.forget();

//     console_log!("returning");
//     Ok(())
// }

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
