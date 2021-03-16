use std::{collections::HashMap};

use nalgebra::{
    Matrix2,
    Matrix3,
    Matrix4,
    Vector2,
    Vector3,
    Vector4,
};

use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{HtmlCanvasElement, WebGlProgram, WebGlRenderingContext, WebGlShader};

pub struct WebGLManager {
    canvas: HtmlCanvasElement,
    context: WebGlRenderingContext,
    shaders: HashMap<String, WebGlProgram>,
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
                        WebGlRenderingContext::ARRAY_BUFFER,
                        &arr,
                        WebGlRenderingContext::STATIC_DRAW
                    );
                }

                // get the index of the attribute
                let attr_index = $self.context.get_attrib_location(&$shader_program, &$attr.name);

                // store the current buffer in the attribute
                $self.context.vertex_attrib_pointer_with_i32(
                    attr_index as u32,
                    $attr.vec_size as i32,
                    WebGlRenderingContext::$web_gl,
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


impl WebGLManager {
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, JsValue> {
        let context = canvas
            .get_context("webgl")?
            .unwrap()
            .dyn_into::<WebGlRenderingContext>()?;

        Ok(Self {
            canvas,
            context,
            shaders: HashMap::new(),
        })
    }

    pub fn register_shader(
        &mut self,
        shader_name: &str,
        vert_src: &str,
        frag_src: &str,
    ) -> Result<(), String> {
        if !self.shaders.contains_key(shader_name) {
            let vert = self.compile_shader(WebGlRenderingContext::VERTEX_SHADER, vert_src)?;
            let frag = self.compile_shader(WebGlRenderingContext::FRAGMENT_SHADER, frag_src)?;

            let program = self.link_program(&vert, &frag)?;

            self.shaders.insert(shader_name.to_owned(), program);

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
            .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
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
            .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
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

    pub fn draw(&self, objects: Vec<&dyn Renderable>) -> Result<(), String> {
        self.context.clear_color(0.0, 0.0, 0.0, 1.0);
        self.context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

        for obj in objects {
            self.draw_object(obj)?;
        }

        Ok(())
    }

    fn draw_object(&self, obj: &dyn Renderable) -> Result<(), String> {
        let shader_name = obj.shader();
        let shader_program = self.shaders.get(&shader_name).unwrap();

        self.context.use_program(Some(shader_program));
        self.register_uniforms(obj, shader_program)?;
        self.register_attrs(obj, shader_program)?;

        self.context
            .draw_arrays(obj.render_type(), 0, obj.num_elements());

        Ok(())
    }

    fn register_attrs(
        &self,
        obj: &dyn Renderable,
        shader_program: &WebGlProgram,
    ) -> Result<(), String> {
        for attr in obj.attributes() {
            // Create and bind a new buffer to hold the attribute
            let buffer = self
                .context
                .create_buffer()
                .ok_or("failed to create buffer")?;
            self.context
                .bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));

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

    fn register_uniforms(
        &self,
        obj: &dyn Renderable,
        shader_program: &WebGlProgram,
    ) -> Result<(), String> {
        for uniform in obj.uniforms() {
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

        Ok(())
    }
}
pub trait Renderable {
    fn attributes(&self) -> Vec<Attribute>;
    fn uniforms(&self) -> Vec<Uniform>;
    fn shader(&self) -> String;
    fn render_type(&self) -> u32;
    fn num_elements(&self) -> i32;
}

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