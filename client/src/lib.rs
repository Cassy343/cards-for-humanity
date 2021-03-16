#![feature(unsize)]

mod rendering;
mod ws;

#[macro_use]
mod console;



use nalgebra::{Vector2, Vector3};
use rendering::{shapes::RoundedRect, webgl::{Renderable, WebGLManager}};
use wasm_bindgen::{prelude::*, JsCast};
use ws::WebSocket;

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

    render_test().unwrap();
}


pub fn render_test() -> Result<(), JsValue> {
    console_log!("Running the render test");
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    let mut manager = WebGLManager::new(canvas)?;

    console_log!("making manager");
    manager.register_shader(
        "card",
        include_str!("./shaders/card.vert"),
        include_str!("./shaders/card.frag"),
    )?;

    let mut objects: Vec<&dyn Renderable> = Vec::new();

    let mut rect = RoundedRect { 
        position: Vector2::new(0.5, 0.0), 
        dimensions: Vector2::new(1.0, 0.5), 
        color: Vector3::new(1.0, 0.0, 1.0), 
        radius: 0.125
    };

    objects.push(&rect);

    manager.draw(objects)?;

    console_log!("registering set timeout");
    let func = Closure::<dyn FnMut()>::new(move || {
        rect.test();

        let mut objects: Vec<&dyn Renderable> = Vec::new();
        objects.push(&rect);
        manager.draw(objects).unwrap();
    });

    web_sys::window()
        .unwrap()
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            func.as_ref().unchecked_ref(),
            1000,
        )?;

    func.forget();

    console_log!("returning");
    Ok(())
}