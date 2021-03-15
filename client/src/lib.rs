#![feature(unsize)]

mod webgl;
mod ws;

#[macro_use]
mod console;


use std::{collections::HashMap, time::Duration};

use nalgebra::{DVector, Vector2, Vector4};
use wasm_bindgen::{JsCast, prelude::*};
use web_sys::WebGlRenderingContext;
use webgl::{Attribute, AttributeType, Renderable, Uniform, UniformType, WebGLManager};
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
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    let mut manager = WebGLManager::new(canvas)?;

    manager.register_shader(
        "card",
        include_str!("./shaders/card.vert"),
        include_str!("./shaders/card.frag"),
    )?;

    let mut objects: Vec<&dyn Renderable> = Vec::new();

    let mut rect = Rect(0.0, 0.0, 0.0);

    objects.push(&rect);

    manager.draw(objects)?;

    let func = Closure::<dyn FnMut()>::new(move || {
        rect.test();

        let mut objects: Vec<&dyn Renderable> = Vec::new();
        objects.push(&rect);
        manager.draw(objects).unwrap();
    });

    web_sys::window().unwrap().set_timeout_with_callback_and_timeout_and_arguments_0(func.as_ref().unchecked_ref(), 1000)?;

    func.forget();

    Ok(())
}

struct Rect(f32, f32, f32);

impl Rect {
    pub fn test(&mut self) {
        self.0 = 1.0;
        self.1 = 1.0;
    }
}

impl Renderable for Rect {
    fn attributes(&self) -> Vec<Attribute> {
        vec![
            Attribute {
                name: "a_position".to_owned(),
                kind: AttributeType::Float(vec![
                    -1.0, 1.0,
                    -1.0, -1.0,
                    1.0, 1.0,
                    1.0, -1.0
                ]),
                vec_size: 2,
            },
        ]
    }

    fn uniforms(&self) -> Vec<Uniform> {
        vec![
            Uniform {
                name: "vColor".to_owned(),
                kind: UniformType::FVec4(Vector4::new(self.0, self.1, self.2, 1.0))
            }
        ]
    }

    fn shader(&self) -> String {
        "card".to_owned()
    }

    fn render_type(&self) -> u32 {
        WebGlRenderingContext::TRIANGLE_STRIP
    }

    fn num_elements(&self) -> i32 {
        4
    }
}
