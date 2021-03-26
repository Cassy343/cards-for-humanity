pub mod render_manager;
pub mod shapes;
pub mod webgl;

use std::num::ParseIntError;

use nalgebra::Vector3;
pub use render_manager::*;

pub struct Color {
    inner: Vector3<f32>,
}

macro_rules! rgb_get {
    ($($fn_name: ident, $fn_name2: ident, $corresponding: ident),*) => {
        $(
            pub fn $fn_name(&self) -> f32 {
                self.inner.$corresponding
            }

            pub fn $fn_name2(&self) -> u8 {
                (self.inner.$corresponding * 256.0) as u8
            }
        )*
    };
}

#[allow(dead_code)]
impl Color {
    rgb_get! {r, r_int, x, g, g_int, y, b, b_int, z}

    pub fn from_hex_str(str: &str) -> Result<Color, ParseIntError> {
        Ok(Color {
            inner: Vector3::new(
                u8::from_str_radix(&str[0 .. 2], 16)? as f32,
                u8::from_str_radix(&str[2 .. 4], 16)? as f32,
                u8::from_str_radix(&str[4 .. 6], 16)? as f32,
            ),
        })
    }

    pub fn from_rgb(r: u8, g: u8, b: u8) -> Color {
        Color {
            inner: Vector3::new(r as f32 / 256.0, g as f32 / 256.0, b as f32 / 256.0),
        }
    }

    pub fn from_gl_color(r: f32, g: f32, b: f32) -> Color {
        Color {
            inner: Vector3::new(r, g, b),
        }
    }

    pub fn to_gl_color(&self) -> Vector3<f32> {
        self.inner.clone()
    }

    pub fn to_rgb(&self) -> Vector3<u8> {
        self.inner.map(|f| (f * 256.0) as u8)
    }
}
