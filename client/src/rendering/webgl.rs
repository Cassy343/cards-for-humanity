use std::{collections::HashMap, ops::Mul};

use nalgebra::{Matrix2, Matrix2xX, Matrix3, Matrix4, Vector2, Vector3, Vector4};

use serde::Serialize;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext, WebGlProgram, WebGlShader};

use crate::console_log;

pub struct WebGlManager {
    context: WebGl2RenderingContext,
    shaders: HashMap<String, (WebGlProgram, u32)>,
    aspect_ratio: Vector2<f32>,
}

macro_rules! attr_load {
    ($self: ident, $attr: ident, $shader_program: ident, $($attr_type: ident, $js_type: ident, $web_gl: ident);*) => {
        match &$attr.kind {
            $(AttributeType::$attr_type(values) => {
                // Fill the buffer with the values given for the attribute
                // Is unsafe because of view()
                // If we allocate any memory before dropping arr, arr is no longer valid
                unsafe {
                    let arr = js_sys::$js_type::view(values.iter().map(|b|*b).collect::<Vec<_>>().as_slice());
                    $self.context.buffer_data_with_array_buffer_view(
                        WebGl2RenderingContext::ARRAY_BUFFER,
                        &arr,
                        WebGl2RenderingContext::STATIC_DRAW
                    );
                }

                // get the index of the attribute
                let attr_index = $self.context.get_attrib_location(&$shader_program, &$attr.name);

                // store the current buffer in the attribute
                $self.context.vertex_attrib_pointer_with_i32(
                    attr_index as u32,
                    $attr.vec_size as i32,
                    WebGl2RenderingContext::$web_gl,
                    false,
                    0,
                    0
                );

                // enable the attribute to be used by shaders
                $self.context.enable_vertex_attrib_array(attr_index as u32);
            }),*
            #[allow(unreachable_patterns)]
            _ => unimplemented!()
        }
    };
}

#[derive(Serialize)]
struct ContextOptions {
    antialias: bool,
}


impl WebGlManager {
    pub fn new(
        webgl_canvas: &HtmlCanvasElement,
        canvas_size: Vector2<f32>,
    ) -> Result<Self, JsValue> {
        let context: WebGl2RenderingContext = webgl_canvas
            .get_context_with_context_options(
                "webgl2",
                &JsValue::from_serde(&ContextOptions { antialias: true })
                    .expect("Error serializing webgl2 context options"),
            )?
            .unwrap()
            .dyn_into()?;

        context.enable(WebGl2RenderingContext::BLEND);
        context.enable(WebGl2RenderingContext::SAMPLE_COVERAGE);
        context.enable(WebGl2RenderingContext::SAMPLE_ALPHA_TO_COVERAGE);
        // context.sample_coverage(0.5, false);


        Ok(WebGlManager {
            context,
            shaders: HashMap::new(),
            aspect_ratio: Vector2::new(
                canvas_size.x / canvas_size.y,
                canvas_size.y / canvas_size.x,
            ),
        })
    }

    pub fn register_shader(
        &mut self,
        shader_name: &str,
        vert_src: &str,
        frag_src: &str,
        render_type: u32,
    ) -> Result<(), String> {
        if !self.shaders.contains_key(shader_name) {
            let vert = self.compile_shader(WebGl2RenderingContext::VERTEX_SHADER, vert_src)?;
            let frag = self.compile_shader(WebGl2RenderingContext::FRAGMENT_SHADER, frag_src)?;

            let program = self.link_program(&vert, &frag)?;

            self.shaders
                .insert(shader_name.to_owned(), (program, render_type));

            Ok(())
        } else {
            Ok(())
        }
    }

    fn compile_shader(&self, shader_type: u32, source: &str) -> Result<WebGlShader, String> {
        let shader = self
            .context
            .create_shader(shader_type)
            .ok_or_else(|| String::from("Unable to create shader object"))?;
        self.context.shader_source(&shader, source);
        self.context.compile_shader(&shader);

        if self
            .context
            .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(shader)
        } else {
            Err(self
                .context
                .get_shader_info_log(&shader)
                .unwrap_or_else(|| String::from("Unknown error creating shader")))
        }
    }

    fn link_program(
        &self,
        vert_shader: &WebGlShader,
        frag_shader: &WebGlShader,
    ) -> Result<WebGlProgram, String> {
        let program = self
            .context
            .create_program()
            .ok_or_else(|| String::from("Unable to create shader object"))?;

        self.context.attach_shader(&program, vert_shader);
        self.context.attach_shader(&program, frag_shader);
        self.context.link_program(&program);

        if self
            .context
            .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(program)
        } else {
            Err(self
                .context
                .get_program_info_log(&program)
                .unwrap_or_else(|| String::from("Unknown error creating program object")))
        }
    }

    pub fn set_clear_color(&self, r: f32, g: f32, b: f32) {
        self.context.clear_color(r, g, b, 1.0);
    }

    pub fn clear(&self) {
        self.context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
    }

    pub fn clear_rect(&self, x: i32, y: i32, width: i32, height: i32) {
        self.context.enable(WebGl2RenderingContext::SCISSOR_TEST);
        self.context.scissor(x, y, width, height);
        self.context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
    }

    pub fn draw_object(&self, obj: &dyn WebGLRenderable) -> Result<(), String> {
        let shader_name = obj.shader();
        let (shader_program, render_type) = self.shaders.get(&shader_name).unwrap();

        self.context.use_program(Some(shader_program));

        let mut uniforms = obj.uniforms();

        uniforms.push(Uniform {
            name: "position".to_owned(),
            kind: UniformType::FVec2(
                obj.position()
                    .component_div(&Vector2::from(super::BASE_RESOLUTION)),
            ),
        });

        console_log!("{}", self.aspect_ratio);

        uniforms.push(Uniform {
            name: "aspect_ratio".to_owned(),
            kind: UniformType::Float(self.aspect_ratio.x),
        });

        self.register_uniforms(&uniforms, shader_program);

        let mut attrs = obj.attributes();

        let vertexes = obj.vertexes();
        let scale = obj.scale();

        let combined_scale = scale.component_div(&Vector2::from(super::BASE_RESOLUTION));

        let output = Matrix2::new(combined_scale.x, 0.0, 0.0, combined_scale.y).mul(&vertexes);

        console_log!("{} {} {} {}", vertexes, scale, combined_scale, output);

        attrs.push(Attribute {
            name: "vertex".to_owned(),
            vec_size: 2,
            kind: AttributeType::Float(output.as_slice().to_vec()),
        });

        self.register_attrs(&attrs, shader_program)?;

        self.context
            .draw_arrays(*render_type, 0, obj.vertexes().len() as i32 / 2);

        Ok(())
    }

    fn register_attrs(
        &self,
        attrs: &Vec<Attribute>,
        shader_program: &WebGlProgram,
    ) -> Result<(), String> {
        for attr in attrs {
            // Create and bind a new buffer to hold the attribute
            let buffer = self
                .context
                .create_buffer()
                .ok_or("failed to create buffer")?;
            self.context
                .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));

            attr_load!(self, attr, shader_program,
                Byte, Int8Array, BYTE;
                Short, Int16Array, SHORT;
                UByte, Uint8Array, UNSIGNED_BYTE;
                UShort, Uint16Array, UNSIGNED_SHORT;
                Float, Float32Array, FLOAT
            );
        }
        Ok(())
    }

    fn register_uniforms(&self, uniforms: &Vec<Uniform>, shader_program: &WebGlProgram) {
        for uniform in uniforms {
            let location_opt = self
                .context
                .get_uniform_location(shader_program, &uniform.name);
            let location = location_opt.as_ref();

            match uniform.kind {
                UniformType::Int(v) => self.context.uniform1i(location, v),
                UniformType::IVec2(v) => self.context.uniform2i(location, v.x, v.y),
                UniformType::IVec3(v) => self.context.uniform3i(location, v.x, v.y, v.z),
                UniformType::IVec4(v) => self.context.uniform4i(location, v.x, v.y, v.z, v.w),
                UniformType::Float(v) => self.context.uniform1f(location, v),
                UniformType::FVec2(v) => self.context.uniform2f(location, v.x, v.y),
                UniformType::FVec3(v) => self.context.uniform3f(location, v.x, v.y, v.z),
                UniformType::FVec4(v) => self.context.uniform4f(location, v.x, v.y, v.z, v.w),
                UniformType::FMat2(v) =>
                    self.context
                        .uniform_matrix2fv_with_f32_array(location, false, v.as_slice()),
                UniformType::FMat3(v) =>
                    self.context
                        .uniform_matrix3fv_with_f32_array(location, false, v.as_slice()),
                UniformType::FMat4(v) =>
                    self.context
                        .uniform_matrix4fv_with_f32_array(location, false, v.as_slice()),
            }
        }
    }
}

pub trait WebGLRenderable {
    // Use vec instead of Matrix because we don't know the size and DMatrix is hell
    fn vertexes(&self) -> Matrix2xX<f32>;
    fn position(&self) -> Vector2<f32>;
    fn attributes(&self) -> Vec<Attribute>;
    fn uniforms(&self) -> Vec<Uniform>;
    fn shader(&self) -> String;
    fn scale(&self) -> Vector2<f32>;
}

#[allow(dead_code)]
pub enum AttributeType {
    Byte(Vec<i8>),
    Short(Vec<i16>),
    UByte(Vec<u8>),
    UShort(Vec<u16>),
    Float(Vec<f32>),
}

pub struct Attribute {
    pub name: String,
    pub kind: AttributeType,
    pub vec_size: u8,
}

pub struct Uniform {
    pub name: String,
    pub kind: UniformType,
}

#[allow(dead_code)]
pub enum UniformType {
    Int(i32),
    IVec2(Vector2<i32>),
    IVec3(Vector3<i32>),
    IVec4(Vector4<i32>),
    Float(f32),
    FVec2(Vector2<f32>),
    FVec3(Vector3<f32>),
    FVec4(Vector4<f32>),
    FMat2(Matrix2<f32>),
    FMat3(Matrix3<f32>),
    FMat4(Matrix4<f32>),
}
