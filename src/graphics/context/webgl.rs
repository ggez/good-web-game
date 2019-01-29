use stdweb::{unstable::*, web::html_element::*, web::*};

use cgmath::{Matrix4, Point2, Vector2, Vector3};
use webgl_stdweb::{
    GLenum, WebGLBuffer, WebGLProgram, WebGLRenderingContext, WebGLRenderingContext as gl,
    WebGLShader, WebGLTexture, WebGLUniformLocation,
};

use crate::graphics::types::{Color, Rect};

#[derive(Clone, Debug)]
pub struct Texture {
    texture: WebGLTexture,
}

impl PartialEq for Texture {
    fn eq(&self, other: &Texture) -> bool {
        self.texture.as_ref() == other.texture.as_ref()
    }
}

impl Texture {
    pub fn new(context: &mut crate::Context, image_element: ImageElement) -> Texture {
        let gl_ctx = &mut context.gfx_context.webgl_context.gl_ctx;

        let texture = gl_ctx.create_texture().unwrap();
        gl_ctx.active_texture(gl::TEXTURE0);
        gl_ctx.bind_texture(gl::TEXTURE_2D, Some(&texture));
        gl_ctx.tex_image2_d_1(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            image_element,
        );
        gl_ctx.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl_ctx.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
        gl_ctx.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);

        Texture { texture }
    }

    pub fn from_rgba8(
        context: &mut crate::Context,
        width: u16,
        height: u16,
        bytes: &[u8],
    ) -> Texture {
        let gl_ctx = &context.gfx_context.webgl_context.gl_ctx;

        Self::from_rgba8_gl(gl_ctx, width, height, bytes)
    }

    pub(crate) fn from_rgba8_gl(
        gl_ctx: &WebGLRenderingContext,
        width: u16,
        height: u16,
        bytes: &[u8],
    ) -> Texture {
        let texture = gl_ctx.create_texture().unwrap();
        gl_ctx.active_texture(gl::TEXTURE0);
        gl_ctx.bind_texture(gl::TEXTURE_2D, Some(&texture));
        gl_ctx.tex_image2_d(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
            width as i32,
            height as i32,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            Some(bytes),
        );
        gl_ctx.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl_ctx.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
        gl_ctx.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);

        Texture { texture }
    }
}

pub trait UniformValue: Clone + PartialEq {
    fn set(self, ctx: &WebGLRenderingContext, location: &WebGLUniformLocation);
}

impl UniformValue for Matrix4<f32> {
    fn set(self, ctx: &WebGLRenderingContext, location: &WebGLUniformLocation) {
        let matrix: [f32; 16] = unsafe { std::mem::transmute(self) };
        ctx.uniform_matrix4fv(Some(location), false, &matrix[..]);
    }
}

impl UniformValue for Color {
    fn set(self, ctx: &WebGLRenderingContext, location: &WebGLUniformLocation) {
        let color: [f32; 4] = self.into();
        ctx.uniform4fv(Some(location), &color[..]);
    }
}

impl UniformValue for Texture {
    fn set(self, ctx: &WebGLRenderingContext, location: &WebGLUniformLocation) {
        ctx.active_texture(gl::TEXTURE0);
        ctx.bind_texture(gl::TEXTURE_2D, Some(&self.texture));
        ctx.uniform1i(Some(location), 0);
    }
}

pub struct Uniform<T: UniformValue> {
    location: WebGLUniformLocation,
    value: T,
}

impl<T: UniformValue> Uniform<T> {
    fn new(
        ctx: &WebGLRenderingContext,
        program: &WebGLProgram,
        name: &str,
        value: T,
    ) -> Uniform<T> {
        let location = ctx.get_uniform_location(program, name).unwrap();

        value.clone().set(ctx, &location);

        Uniform { location, value }
    }

    fn update(&mut self, ctx: &WebGLRenderingContext, value: T) {
        if value != self.value {
            self.value = value.clone();
            value.set(ctx, &self.location)
        }
    }
}

pub struct UniformsState {
    model: Uniform<Matrix4<f32>>,
    projection: Uniform<Matrix4<f32>>,
    color: Uniform<Color>,
    texture: Uniform<Texture>,
}

pub struct WebGlContext {
    pub canvas: CanvasElement,
    pub gl_ctx: WebGLRenderingContext,
    pub screen_rect: Rect,
    pub projection: Matrix4<f32>,
    uniforms: UniformsState,
}

fn get_context(canvas: &CanvasElement) -> WebGLRenderingContext {
    use stdweb::{_js_impl, js};

    js!(return @{canvas}.getContext("webgl", {alpha: false});)
        .try_into()
        .unwrap()
}

impl WebGlContext {
    pub fn new(canvas: CanvasElement) -> WebGlContext {
        let gl_ctx: WebGLRenderingContext = get_context(&canvas);
        let sprite_shader = init_shader_program(&gl_ctx);
        let quad = create_quad(&gl_ctx);
        let projection = cgmath::One::one();
        let screen_rect = Rect::new(-1., -1., 2., 2.);

        gl_ctx.color_mask(true, true, true, false);
        gl_ctx.enable(gl::BLEND);
        gl_ctx.blend_func(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

        gl_ctx.bind_buffer(gl::ARRAY_BUFFER, Some(&quad));

        let pos_attrib = gl_ctx.get_attrib_location(&sprite_shader, "position") as u32;
        let stride_distance = (2 * std::mem::size_of::<f32>()) as i32;
        gl_ctx.vertex_attrib_pointer(pos_attrib, 2, gl::FLOAT, false, stride_distance, 0);
        gl_ctx.enable_vertex_attrib_array(pos_attrib);

        gl_ctx.use_program(Some(&sprite_shader));

        let default_texture = Texture::from_rgba8_gl(&gl_ctx, 1, 1, &[255, 0, 0, 255]);

        let model_uniform = Uniform::new(&gl_ctx, &sprite_shader, "Model", cgmath::One::one());
        let projection_uniform =
            Uniform::new(&gl_ctx, &sprite_shader, "Projection", cgmath::One::one());
        let color_uniform = Uniform::new(&gl_ctx, &sprite_shader, "Color", [0., 0., 0., 0.].into());
        let texture_uniform = Uniform::new(&gl_ctx, &sprite_shader, "Texture", default_texture);

        WebGlContext {
            canvas,
            gl_ctx,
            projection,
            screen_rect,
            uniforms: UniformsState {
                model: model_uniform,
                projection: projection_uniform,
                color: color_uniform,
                texture: texture_uniform,
            },
        }
    }

    pub fn clear(&self, color: Color) {
        self.gl_ctx.clear_color(color.r, color.g, color.b, color.a);
        self.gl_ctx
            .clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }

    pub(crate) fn set_projection_rect(&mut self, rect: Rect) {
        self.screen_rect = rect;
        self.projection =
            cgmath::ortho(rect.x, rect.x + rect.w, rect.y + rect.h, rect.y, -1.0, 1.0);

        self.uniforms
            .projection
            .update(&self.gl_ctx, self.projection);
    }

    pub(crate) fn resize(&self, w: u32, h: u32) {
        self.gl_ctx.viewport(0, 0, w as i32, h as i32);
    }

    pub fn draw_image(
        &mut self,
        dest: Point2<f32>,
        scale: Vector2<f32>,
        size: Vector2<f32>,
        color: Color,
        texture: &Texture,
    ) {
        let scale = Matrix4::from_nonuniform_scale(scale.x * size.x, scale.y * size.y, 0.);
        let pos = Matrix4::from_translation(Vector3::new(dest.x, dest.y, 0.));
        let transform = pos * scale;

        self.uniforms.model.update(&self.gl_ctx, transform);
        self.uniforms.color.update(&self.gl_ctx, color);
        self.uniforms.texture.update(&self.gl_ctx, texture.clone());

        self.gl_ctx.draw_arrays(gl::TRIANGLE_STRIP, 0, 4);
    }
}

pub fn init_shader_program(ctx: &WebGLRenderingContext) -> WebGLProgram {
    let vertex_shader = load_shader(ctx, gl::VERTEX_SHADER, VERTEX_SHADER);
    let fragment_shader = load_shader(ctx, gl::FRAGMENT_SHADER, FRAGMENT_SHADER);

    let program = ctx.create_program().unwrap();
    ctx.attach_shader(&program, &vertex_shader);
    ctx.attach_shader(&program, &fragment_shader);
    ctx.link_program(&program);

    if ctx.get_program_parameter(&program, gl::LINK_STATUS) == false {
        panic!(ctx.get_program_info_log(&program));
    }

    program
}

pub fn load_shader(ctx: &WebGLRenderingContext, shader_type: GLenum, source: &str) -> WebGLShader {
    let shader = ctx.create_shader(shader_type).unwrap();

    ctx.shader_source(&shader, source);
    ctx.compile_shader(&shader);

    if ctx.get_shader_parameter(&shader, gl::COMPILE_STATUS) == false {
        panic!(ctx.get_shader_info_log(&shader));
    }

    shader
}

pub fn create_quad(ctx: &WebGLRenderingContext) -> WebGLBuffer {
    let position_buffer = ctx.create_buffer().unwrap();

    ctx.bind_buffer(gl::ARRAY_BUFFER, Some(&position_buffer));
    let positions = TypedArray::<f32>::from(&[0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0][..]).buffer();
    ctx.buffer_data_1(gl::ARRAY_BUFFER, Some(&positions), gl::STATIC_DRAW);

    position_buffer
}

const VERTEX_SHADER: &str = r#"attribute vec2 position;
varying lowp vec4 color;
varying lowp vec2 uv;

uniform mat4 Projection;
uniform mat4 Model;
uniform vec4 Color;

void main() {
    gl_Position = Projection * Model * vec4(position, 0, 1);
    color = Color;
    uv = position;
}"#;

const FRAGMENT_SHADER: &str = r#"    
varying lowp vec4 color;
varying lowp vec2 uv;

uniform sampler2D Texture;

void main() {
    gl_FragColor = texture2D(Texture, uv) * color;
}"#;
