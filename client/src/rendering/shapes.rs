use nalgebra::{Vector2, Vector3, Vector4};
use web_sys::WebGlRenderingContext;

use super::webgl::*;


pub struct RoundedRect {
    pub position: Vector2<f32>, 
    pub dimensions: Vector2<f32>, 
    pub color: Vector3<f32>,
    pub radius: f32
}

impl RoundedRect {
    pub fn test(&mut self) {
        self.position.x = 0.0;
        self.position.y = 0.5;
    }
}

impl Renderable for RoundedRect {
    fn attributes(&self) -> Vec<Attribute> {
        vec![Attribute {
            name: "a_position".to_owned(),
            kind: AttributeType::Float(vec![
                -1.0, 1.0,
                -1.0, -1.0,
                1.0, 1.0,
                1.0, -1.0
            ]),
            vec_size: 2,
        }]
    }

    fn uniforms(&self) -> Vec<Uniform> {
        vec![
            Uniform {
                name: "color".to_owned(),
                kind: UniformType::FVec4(Vector4::new(self.color.x, self.color.y, self.color.z, 1.0)),
            },
            Uniform {
                name: "dimensions".to_owned(),
                kind: UniformType::FVec2(self.dimensions),
            },
            Uniform {
                name: "corner_radius".to_owned(),
                kind: UniformType::Float(self.radius),
            },
            Uniform {
                name: "center_position".to_owned(),
                kind: UniformType::FVec2(self.position)
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
