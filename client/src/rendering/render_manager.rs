use nalgebra::Vector2;
use wasm_bindgen::{prelude::*, JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, TextMetrics};

use super::{
    webgl::{WebGLRenderable, WebGlManager},
    Color,
};

pub static BASE_RESOLUTION: [f32; 2] = [1600.0, 900.0];

pub struct RenderManager {
    context_2d: CanvasRenderingContext2d,
    pub scale_factor: Vector2<f32>,
    text_canvas: HtmlCanvasElement,
    canvas_size: Vector2<f32>,
    webgl_manager: WebGlManager,
}

impl RenderManager {
    pub fn new(
        webgl_canvas: &HtmlCanvasElement,
        text_canvas: &HtmlCanvasElement,
    ) -> Result<RenderManager, JsValue> {
        let canvas_size = Vector2::new(text_canvas.width() as f32, text_canvas.height() as f32);
        let scale_factor = canvas_size.component_div(&Vector2::from(BASE_RESOLUTION));

        let webgl_manager = WebGlManager::new(webgl_canvas, canvas_size)?;

        let context_2d = text_canvas.get_context("2d")?.unwrap().dyn_into()?;

        Ok(RenderManager {
            webgl_manager,
            context_2d,
            text_canvas: text_canvas.clone(),
            canvas_size,
            scale_factor,
        })
    }

    pub fn update_scale_factor(&mut self) {
        let canvas_size = Vector2::new(
            self.text_canvas.width() as f32,
            self.text_canvas.height() as f32,
        );
        self.scale_factor = canvas_size.component_div(&Vector2::from(BASE_RESOLUTION));
    }

    pub fn clear(&self) {
        self.context_2d.clear_rect(
            0.0,
            0.0,
            self.canvas_size.x as f64,
            self.canvas_size.y as f64,
        );
        self.webgl_manager.clear();
    }

    pub fn set_background_color(&self, color: Color) {
        self.webgl_manager
            .set_clear_color(color.r(), color.g(), color.b())
    }

    pub fn get_text_spacing(&self, text: &str) -> Result<TextMetrics, String> {
        self.context_2d
            .measure_text(text)
            .map_err(|e| format!("Error drawing text: {:?}", e))
    }

    pub fn set_fill_style(&self, font_style: &str) {
        self.context_2d
            .set_fill_style(&JsValue::from_str(font_style));
    }

    pub fn set_font(&self, font: &str) {
        self.context_2d.set_font(font);
    }

    pub fn set_text_align(&self, text_align: &str) {
        self.context_2d.set_text_align(text_align);
    }

    pub fn set_text_baseline(&self, text_baseline: &str) {
        self.context_2d.set_text_baseline(text_baseline);
    }

    pub fn draw_text(
        &self,
        text: &str,
        x: f32,
        y: f32,
        max_width: Option<f32>,
    ) -> Result<(), String> {
        match max_width {
            Some(max_width) => self
                .context_2d
                .fill_text_with_max_width(
                    text,
                    ((x * self.scale_factor.x) + self.canvas_size.x / 2.0) as f64,
                    ((y * self.scale_factor.y) + self.canvas_size.y / 2.0) as f64,
                    max_width as f64,
                )
                .map_err(|e| format!("Error drawing text: {:?}", e)),
            None => self
                .context_2d
                .fill_text(
                    text,
                    ((x * self.scale_factor.x) + self.canvas_size.x / 2.0) as f64,
                    ((y * self.scale_factor.y) + self.canvas_size.y / 2.0) as f64,
                )
                .map_err(|e| format!("Error drawing text: {:?}", e)),
        }
    }

    pub fn draw_text_outline(
        &self,
        text: &str,
        x: f32,
        y: f32,
        max_width: Option<f32>,
    ) -> Result<(), String> {
        match max_width {
            Some(max_width) => self
                .context_2d
                .stroke_text_with_max_width(
                    text,
                    ((x * self.scale_factor.x) + self.canvas_size.x / 2.0) as f64,
                    ((y * self.scale_factor.y) + self.canvas_size.y / 2.0) as f64,
                    max_width as f64,
                )
                .map_err(|e| format!("Error drawing text: {:?}", e)),
            None => self
                .context_2d
                .stroke_text(
                    text,
                    ((x * self.scale_factor.x) + self.canvas_size.x / 2.0) as f64,
                    ((y * self.scale_factor.y) + self.canvas_size.y / 2.0) as f64,
                )
                .map_err(|e| format!("Error drawing text: {:?}", e)),
        }
    }

    pub fn draw_objects(&self, objects: Vec<&dyn Renderable>) -> Result<(), String> {
        for obj in objects {
            obj.render(self)?;
        }

        Ok(())
    }

    pub fn draw_object(&self, renderable: &dyn Renderable) -> Result<(), String> {
        renderable.render(self)
    }

    pub fn draw_webgl_object(&self, renderable: &dyn WebGLRenderable) -> Result<(), String> {
        self.webgl_manager.draw_object(renderable)
    }

    pub fn register_shader(
        &mut self,
        shader_name: &str,
        vert_src: &str,
        frag_src: &str,
        render_type: u32,
    ) -> Result<(), String> {
        self.webgl_manager
            .register_shader(shader_name, vert_src, frag_src, render_type)
    }
}

pub trait Renderable {
    fn render(&self, render_manager: &RenderManager) -> Result<(), String>;
}

impl<T: WebGLRenderable> Renderable for T {
    fn render(&self, render_manager: &RenderManager) -> Result<(), String> {
        render_manager.draw_webgl_object(self)
    }
}


#[wasm_bindgen]
extern "C" {
    pub type ExtendedTextMetrics;

    #[wasm_bindgen(method, getter, js_name = actualBoundingBoxLeft)]
    pub fn actual_bounding_box_left(this: &ExtendedTextMetrics) -> f64;

    #[wasm_bindgen(method, getter, js_name = actualBoundingBoxRight)]
    pub fn actual_bounding_box_right(this: &ExtendedTextMetrics) -> f64;
}
