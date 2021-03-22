use nalgebra::{Matrix2xX, Vector2, Vector4};
use wasm_bindgen::JsCast;

use super::{Color, ExtendedTextMetrics, RenderManager, Renderable, webgl::*};

use crate::console_log;

pub struct Meme {
    pub rect: RoundedRect,
    pub text: String,
    pub text_pos: Vector2<f32>,
    pub text_width: Option<f32>,
}

impl Renderable for Meme {
    fn render(&self, render_manager: &RenderManager) -> Result<(), String> {
        render_manager.draw_object(&self.rect)?;

        render_manager.set_font("50px Comic Sans MS");

        let colors = [
            "#FF0018", "#FFA52C", "#FFFF41", "#008018", "#0000F9", "#86007D",
        ];
        let mut color_index = 0;
        let mut text_size = 0.0_f64;

        let full_text_size = render_manager.get_text_spacing(&self.text)?.width();

        for char in self.text.chars() {
            let char_width = render_manager.get_text_spacing(&char.to_string())?.width();
            render_manager.set_fill_style(colors[color_index]);
            render_manager.draw_text(
                &char.to_string(),
                self.text_pos.x - (full_text_size / 2.0) as f32 + text_size as f32,
                self.text_pos.y,
                self.text_width,
            )?;

            text_size += char_width;

            color_index += 1;
            if color_index == colors.len() {
                color_index = 0;
            }
        }

        Ok(())
    }
}

pub struct TextBubble {
    pub rect: RoundedRect,
    pub text: Text
}

impl Renderable for TextBubble {
    fn render(&self, render_manager: &RenderManager) -> Result<(), String> {
        render_manager.draw_object(&self.rect)?;
        render_manager.draw_object(&self.text)
    }
}

pub struct Text {
    pub text: String,
    pub text_pos: Vector2<f32>,
    pub width: Option<f32>,
    pub font: String,
    pub font_size: u32,
    pub fill_style: String,
    pub outline: bool,
}

impl Renderable for Text {
    fn render(&self, render_manager: &RenderManager) -> Result<(), String> {
        render_manager.set_font(&format!("{}px {}", self.font_size, self.font));
        render_manager.set_fill_style(&self.fill_style);

        
        let line_count = self.text.lines().count() - 1;
        let lines = self.text.split('\n').enumerate();
        
        for (line_num, line_text) in lines {
            let text_metrics: ExtendedTextMetrics = render_manager
                .get_text_spacing(line_text)?
                .unchecked_into();
            let text_spacing_left = text_metrics.actual_bounding_box_left();
            let text_spacing_right = text_metrics.actual_bounding_box_right();

            let adjust_val = self.font_size as f32 * 2.0
                * -((line_count as f32 / 2.0) - line_num as f32);

            console_log!("{} {} {} {} {}", line_num, line_count, self.text_pos.y, ((line_count as f32 / 2.0) - line_num as f32), adjust_val);

            if self.outline {
                render_manager.draw_text_outline(
                    line_text,
                    self.text_pos.x - ((text_spacing_right / 2.0) - text_spacing_left) as f32,
                    self.text_pos.y + adjust_val,
                    self.width,
                )?;
            } else {
                render_manager.draw_text(
                    line_text,
                    self.text_pos.x - ((text_spacing_right / 2.0) - text_spacing_left) as f32,
                    self.text_pos.y + adjust_val,
                    self.width,
                )?;
            }
        }

        Ok(())
    }
}

pub struct RoundedRect {
    pub position: Vector2<f32>,
    pub dimensions: Vector2<f32>,
    pub color: Color,
    pub radius: f32,
}

impl WebGLRenderable for RoundedRect {
    fn attributes(&self) -> Vec<Attribute> {
        vec![]
    }

    fn uniforms(&self) -> Vec<Uniform> {
        let color = self.color.to_gl_color();

        vec![
            Uniform {
                name: "color".to_owned(),
                kind: UniformType::FVec4(Vector4::new(
                    color.x,
                    color.y,
                    color.z,
                    1.0,
                )),
            },
            Uniform {
                name: "dimensions".to_owned(),
                kind: UniformType::FVec2(self.dimensions.component_div(&Vector2::from(super::BASE_RESOLUTION)) * 2.0),
            },
            Uniform {
                name: "corner_radius".to_owned(),
                kind: UniformType::Float(self.radius),
            },
            Uniform {
                name: "center_position".to_owned(),
                kind: UniformType::FVec2(self.position),
            },
        ]
    }

    fn shader(&self) -> String {
        "card".to_owned()
    }

    fn vertexes(&self) -> Matrix2xX<f32> {
        Matrix2xX::from_column_slice(&[-1.0, 1.0, -1.0, -1.0, 1.0, 1.0, 1.0, -1.0])
    }

    fn position(&self) -> Vector2<f32> {
        self.position.clone()
    }

    fn scale(&self) -> Vector2<f32> {
        self.dimensions.clone()
    }
}
