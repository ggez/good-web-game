use stdweb::{unstable::*, web::html_element::*, web::*};

use cgmath::{Matrix4, Point2, Vector2, Vector3, Vector4, Rad};
use webgl_stdweb::{
    GLenum, WebGLBuffer, WebGLProgram, WebGLRenderingContext, WebGLRenderingContext as gl,
    WebGLShader, WebGLTexture, WebGLUniformLocation,
};

use crate::graphics::types::{Color, Rect};

pub const LINEAR_FILTER: i32 = gl::LINEAR as i32;
pub const NEAREST_FILTER: i32 = gl::NEAREST as i32;

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
        gl_ctx.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

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
        gl_ctx.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

        Texture { texture }
    }

    pub(crate) fn set_filter(&self, context: &mut crate::Context, filter: i32) {
        let gl_ctx = &mut context.gfx_context.webgl_context.gl_ctx;
        gl_ctx.bind_texture(gl::TEXTURE_2D, Some(&self.texture));

        gl_ctx.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, filter);
        gl_ctx.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, filter);
    }
}

pub trait UniformValue: Clone + PartialEq {
    fn set(self, ctx: &WebGLRenderingContext, location: &WebGLUniformLocation);
}

impl UniformValue for f32 {
    fn set(self, ctx: &WebGLRenderingContext, location: &WebGLUniformLocation) {
        ctx.uniform1f(Some(location), self);
    }
}

impl UniformValue for Matrix4<f32> {
    fn set(self, ctx: &WebGLRenderingContext, location: &WebGLUniformLocation) {
        let matrix: [f32; 16] = unsafe { std::mem::transmute(self) };
        ctx.uniform_matrix4fv(Some(location), false, &matrix[..]);
    }
}

impl UniformValue for Vector4<f32> {
    fn set(self, ctx: &WebGLRenderingContext, location: &WebGLUniformLocation) {
        let vec4: [f32; 4] = self.into();
        ctx.uniform4fv(Some(location), &vec4[..]);
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
        let location = ctx
            .get_uniform_location(program, name)
            .expect(&format!("Cant get \"{}\" uniform location", name));

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
    source: Uniform<Vector4<f32>>,
    color: Uniform<Color>,
    texture: Option<Uniform<Texture>>,
}

pub struct ShaderObject {
    pub program: WebGLProgram,
    pub uniforms: UniformsState,
}

impl ShaderObject {
    pub fn apply(
        &mut self,
        gl_ctx: &WebGLRenderingContext,
        projection: Matrix4<f32>,
        model: Matrix4<f32>,
        source: Vector4<f32>,
        color: Color,
        texture: Option<Texture>,
    ) {
        let pos_attrib = gl_ctx.get_attrib_location(&self.program, "position") as u32;
        let stride_distance = (2 * std::mem::size_of::<f32>()) as i32;
        gl_ctx.vertex_attrib_pointer(pos_attrib, 2, gl::FLOAT, false, stride_distance, 0);
        gl_ctx.enable_vertex_attrib_array(pos_attrib);

        gl_ctx.use_program(Some(&self.program));

        self.uniforms.projection.update(gl_ctx, projection);
        self.uniforms.model.update(gl_ctx, model);
        self.uniforms.color.update(gl_ctx, color);
        self.uniforms.source.update(gl_ctx, source);
        if let Some(texture) = texture {
            let program = &self.program;
            self.uniforms
                .texture
                .get_or_insert_with(|| Uniform::new(&gl_ctx, &program, "Texture", texture.clone()))
                .update(gl_ctx, texture.clone());
        }
    }

    pub fn set_uniform<T: UniformValue>(
        &mut self,
        gl_ctx: &WebGLRenderingContext,
        name: &str,
        uniform: T,
    ) {
        let location = gl_ctx.get_uniform_location(&self.program, name).unwrap();
        uniform.set(gl_ctx, &location);
    }
}

pub struct WebGlContext {
    pub canvas: CanvasElement,
    pub gl_ctx: WebGLRenderingContext,
    pub screen_rect: Rect,
    pub projection: Matrix4<f32>,
    sprite_shader: ShaderObject,
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
        let sprite_shader = load_shader_object(&gl_ctx, &VERTEX_SHADER, &FRAGMENT_SHADER);
        let quad = create_quad(&gl_ctx);
        let projection = cgmath::One::one();
        let screen_rect = Rect::new(-1., -1., 2., 2.);

        gl_ctx.color_mask(true, true, true, false);
        gl_ctx.enable(gl::BLEND);
        gl_ctx.blend_func(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

        gl_ctx.bind_buffer(gl::ARRAY_BUFFER, Some(&quad));

        WebGlContext {
            canvas,
            gl_ctx,
            projection,
            screen_rect,
            sprite_shader,
        }
    }

    pub fn clear(&self, color: Color) {
        self.gl_ctx.clear_color(color.r, color.g, color.b, color.a);
        self.gl_ctx
            .clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }

    pub fn set_projection_matrix(&mut self, transform: Matrix4<f32>) {
        self.projection = transform;
    }

    pub(crate) fn set_projection_rect(&mut self, rect: Rect) {
        self.screen_rect = rect;
        self.projection =
            cgmath::ortho(rect.x, rect.x + rect.w, rect.y + rect.h, rect.y, -1.0, 1.0);
    }

    pub(crate) fn resize(&self, w: u32, h: u32) {
        self.gl_ctx.viewport(0, 0, w as i32, h as i32);
    }

    pub fn draw_image(
        &mut self,
        dest: Point2<f32>,
        size: Vector2<f32>,
        source: Vector4<f32>,
        color: Color,
        texture: &Texture,
    ) {
        let size = Matrix4::from_nonuniform_scale(size.x, size.y, 0.);
        let pos = Matrix4::from_translation(Vector3::new(dest.x, dest.y, 0.));
        let transform = pos * size;

        self.sprite_shader.apply(
            &self.gl_ctx,
            self.projection,
            transform,
            source,
            color,
            Some(texture.clone()),
        );

        self.gl_ctx.draw_arrays(gl::TRIANGLE_STRIP, 0, 4);
    }

    pub fn draw_rect(
        &self,
        dest: Point2<f32>,
        scale: Vector2<f32>,
        size: Vector2<f32>,
        angle: f32,
        color: Color,
        shader: &mut ShaderObject,
    ) {
        let scale = Matrix4::from_nonuniform_scale(scale.x * size.x, scale.y * size.y, 0.);
        let pos = Matrix4::from_translation(Vector3::new(dest.x + size.x / 2., dest.y + size.y / 2., 0.));
        let rot = Matrix4::from_angle_z(Rad(angle));
        let pos0 = Matrix4::from_translation(Vector3::new(-size.x / 2., -size.y / 2., 0.));
        let transform = pos * rot * pos0 * scale;

        shader.apply(&self.gl_ctx, self.projection, transform, [0., 0., 1., 1.].into(), color, None);

        self.gl_ctx.draw_arrays(gl::TRIANGLE_STRIP, 0, 4);
    }

    pub fn load_shader_object(&self, vertex_shader: &str, fragment_shader: &str) -> ShaderObject {
        load_shader_object(&self.gl_ctx, vertex_shader, fragment_shader)
    }
}

fn load_shader_object(
    ctx: &WebGLRenderingContext,
    vertex_shader: &str,
    fragment_shader: &str,
) -> ShaderObject {
    let vertex_shader = load_shader(ctx, gl::VERTEX_SHADER, vertex_shader);
    let fragment_shader = load_shader(ctx, gl::FRAGMENT_SHADER, fragment_shader);

    let program = ctx.create_program().expect("Cant create program");
    ctx.attach_shader(&program, &vertex_shader);
    ctx.attach_shader(&program, &fragment_shader);
    ctx.link_program(&program);

    if ctx.get_program_parameter(&program, gl::LINK_STATUS) == false {
        if let Some(error) = ctx.get_program_info_log(&program) {
            crate::console::log(&error);
        }
        panic!("cant link shader!");
    }

    ctx.use_program(Some(&program));

    let model_uniform = Uniform::new(&ctx, &program, "Model", cgmath::One::one());
    let projection_uniform = Uniform::new(&ctx, &program, "Projection", cgmath::One::one());
    let source_uniform = Uniform::new(&ctx, &program, "Source", [0., 0., 1., 1.].into());
    let color_uniform = Uniform::new(&ctx, &program, "Color", [0., 0., 0., 0.].into());

    ShaderObject {
        program,
        uniforms: UniformsState {
            model: model_uniform,
            projection: projection_uniform,
            source: source_uniform,
            color: color_uniform,
            texture: None,
        },
    }
}

pub fn load_shader(ctx: &WebGLRenderingContext, shader_type: GLenum, source: &str) -> WebGLShader {
    let shader = ctx.create_shader(shader_type).unwrap();

    ctx.shader_source(&shader, source);
    ctx.compile_shader(&shader);

    if ctx.get_shader_parameter(&shader, gl::COMPILE_STATUS) == false {
        if let Some(error) = ctx.get_shader_info_log(&shader) {
            crate::console::log(&error);
        }
        panic!("cant compile shader!");
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
uniform vec4 Source;
uniform vec4 Color;

void main() {
    gl_Position = Projection * Model * vec4(position, 0, 1);
    color = Color;
    uv = position * Source.zw + Source.xy;
}"#;

const FRAGMENT_SHADER: &str = r#"
varying lowp vec4 color;
varying lowp vec2 uv;

uniform sampler2D Texture;

void main() {
    gl_FragColor = texture2D(Texture, uv) * color;
}"#;
