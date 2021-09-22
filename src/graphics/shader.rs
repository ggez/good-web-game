use crate::{Context, GameError, GameResult};
use miniquad::{
    BlendFactor, BlendState, BlendValue, BufferLayout, Equation, PipelineParams, VertexAttribute,
    VertexFormat, VertexStep,
};
use std::cell::RefCell;
use std::io::Read;
use std::path::Path;
use std::rc::Rc;

use crate::graphics::context::default_shader;
use bytemuck::Pod;
use cgmath::Matrix4;
pub use miniquad::{ShaderMeta, UniformBlockLayout, UniformDesc, UniformType};

/// An enum for specifying default and custom blend modes
///
/// If you want to know what these actually do take a look at the implementation of `From<BlendMode> for Blend`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlendMode {
    /// When combining two fragments, add their values together, saturating
    /// at 1.0
    Add,
    /// When combining two fragments, subtract the source value from the
    /// destination value
    Subtract,
    /// When combining two fragments, add the value of the source times its
    /// alpha channel with the value of the destination multiplied by the inverse
    /// of the source alpha channel. Has the usual transparency effect: mixes the
    /// two colors using a fraction of each one specified by the alpha of the source.
    Alpha,
    /// When combining two fragments, multiply their values together (including alpha)
    Multiply,
    /// When combining two fragments, choose the source value (including source alpha)
    Replace,
    /// When using premultiplied alpha, use this.
    ///
    /// You usually want to use this blend mode for drawing canvases
    /// containing semi-transparent imagery.
    /// For an explanation on this see: `<https://github.com/ggez/ggez/issues/694#issuecomment-853724926>`
    Premultiplied,
}

impl From<BlendMode> for (BlendState, BlendState) {
    fn from(bm: BlendMode) -> Self {
        match bm {
            BlendMode::Add => (
                BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::One,
                ),
                BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::One,
                ),
            ),
            BlendMode::Subtract => (
                BlendState::new(
                    Equation::ReverseSubtract,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::One,
                ),
                BlendState::new(Equation::Add, BlendFactor::Zero, BlendFactor::One),
            ),
            BlendMode::Alpha => (
                BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                ),
                BlendState::new(
                    Equation::Add,
                    BlendFactor::OneMinusValue(BlendValue::DestinationAlpha),
                    BlendFactor::One,
                ),
            ),
            BlendMode::Premultiplied => (
                BlendState::new(
                    Equation::Add,
                    BlendFactor::One,
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                ),
                BlendState::new(
                    Equation::Add,
                    BlendFactor::OneMinusValue(BlendValue::DestinationAlpha),
                    BlendFactor::One,
                ),
            ),
            BlendMode::Multiply => (
                BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::DestinationColor),
                    BlendFactor::Zero,
                ),
                BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::DestinationAlpha),
                    BlendFactor::Zero,
                ),
            ),
            BlendMode::Replace => (
                BlendState::new(Equation::Add, BlendFactor::One, BlendFactor::Zero),
                BlendState::new(Equation::Add, BlendFactor::One, BlendFactor::Zero),
            ),
        }
    }
}

/// An ID used by the good-web-game graphics context to uniquely identify a shader
pub type ShaderId = usize;

const MATRIX_SIZE: isize = std::mem::size_of::<Matrix4<f32>>() as isize;

trait Uniform {}

#[derive(Debug)]
pub struct Shader {
    pub(crate) pipeline: miniquad::Pipeline,
    pub(crate) uniforms: Vec<u8>,
}

impl Shader {
    /// Creates a shader from a miniquad shader without adding it to the shader vec of the gfx context.
    /// Useful for creating a shader before the context is initialized.
    pub(crate) fn from_mini_shader(
        quad_ctx: &mut miniquad::Context,
        mini_shader: miniquad::Shader,
        blend_mode: Option<BlendMode>,
    ) -> Self {
        let (color_blend, alpha_blend) = blend_mode.unwrap_or(BlendMode::Alpha).into();

        let new_shader_pipeline = miniquad::Pipeline::with_params(
            quad_ctx,
            &[
                BufferLayout::default(),
                BufferLayout {
                    step_func: VertexStep::PerInstance,
                    ..Default::default()
                },
            ],
            &[
                VertexAttribute::with_buffer("position", VertexFormat::Float2, 0),
                VertexAttribute::with_buffer("texcoord", VertexFormat::Float2, 0),
                VertexAttribute::with_buffer("color0", VertexFormat::Float4, 0),
                VertexAttribute::with_buffer("Source", VertexFormat::Float4, 1),
                VertexAttribute::with_buffer("Color", VertexFormat::Float4, 1),
                VertexAttribute::with_buffer("Model", VertexFormat::Mat4, 1),
            ],
            mini_shader,
            PipelineParams {
                color_blend: Some(color_blend),
                alpha_blend: Some(alpha_blend),
                ..Default::default()
            },
        );
        Shader {
            pipeline: new_shader_pipeline,
            uniforms: vec![0u8; MATRIX_SIZE as usize], // we need to make space for the projection matrix here
        }
    }

    #[allow(clippy::new_ret_no_self)]
    /// Create a new `Shader` given source files, constants and a name.
    pub fn new<P: AsRef<Path>>(
        ctx: &mut Context,
        vertex_path: P,
        pixel_path: P,
        shader_meta: ShaderMeta,
        blend_mode: Option<BlendMode>,
    ) -> GameResult<ShaderId> {
        let vertex_source = {
            let mut buf = Vec::new();
            let mut reader = ctx.filesystem.open(vertex_path)?;
            let _ = reader.read_to_end(&mut buf)?;
            buf
        };
        let pixel_source = {
            let mut buf = Vec::new();
            let mut reader = ctx.filesystem.open(pixel_path)?;
            let _ = reader.read_to_end(&mut buf)?;
            buf
        };
        Self::from_u8(ctx, &vertex_source, &pixel_source, shader_meta, blend_mode)
    }

    /// Create a new `Shader` directly from GLSL source code, given as byte slices.
    pub fn from_u8(
        ctx: &mut Context,
        vertex_source: &[u8],
        pixel_source: &[u8],
        shader_meta: ShaderMeta,
        blend_mode: Option<BlendMode>,
    ) -> GameResult<ShaderId> {
        fn to_shader_error(e: std::str::Utf8Error) -> GameError {
            GameError::ShaderProgramError(e.to_string())
        }

        let vertex_source = std::str::from_utf8(vertex_source).map_err(to_shader_error)?;
        let pixel_source = std::str::from_utf8(pixel_source).map_err(to_shader_error)?;

        Self::from_str(ctx, vertex_source, pixel_source, shader_meta, blend_mode)
    }

    /// Create a new `Shader` directly from GLSL source code.
    pub fn from_str(
        ctx: &mut Context,
        vertex_source: &str,
        pixel_source: &str,
        shader_meta: ShaderMeta,
        blend_mode: Option<BlendMode>,
    ) -> GameResult<ShaderId> {
        let miniquad_shader = miniquad::graphics::Shader::new(
            &mut ctx.quad_ctx,
            vertex_source,
            pixel_source,
            shader_meta,
        )?;

        let shader = Self::from_mini_shader(&mut ctx.quad_ctx, miniquad_shader, blend_mode);

        let id = ctx.gfx_context.shaders.len();
        ctx.gfx_context.shaders.push(shader);

        Ok(id)
    }
}

/// A lock for RAII shader regions. The shader automatically gets cleared once
/// the lock goes out of scope, restoring the previous shader (if any).
///
/// Essentially, binding a [`Shader`](type.Shader.html) will return one of these,
/// and the shader will remain active as long as this object exists.  When this is
/// dropped, the previous shader is restored.
#[derive(Debug, Clone)]
pub struct ShaderLock {
    cell: Rc<RefCell<ShaderId>>,
    previous_shader: ShaderId,
}

impl Drop for ShaderLock {
    fn drop(&mut self) {
        *self.cell.borrow_mut() = self.previous_shader;
    }
}

/// Use a shader until the returned lock goes out of scope
pub fn use_shader(ctx: &mut Context, ps: ShaderId) -> ShaderLock {
    let cell = Rc::clone(&ctx.gfx_context.current_shader);
    let previous_shader = *cell.borrow();
    set_shader(ctx, ps);
    ShaderLock {
        cell,
        previous_shader,
    }
}

/// Set the current shader for the `Context` to render with
pub fn set_shader(ctx: &mut Context, ps: ShaderId) {
    *ctx.gfx_context.current_shader.borrow_mut() = ps;
}

/// Clears the the current shader for the `Context`, restoring the default shader.
///
/// However, calling this and then dropping a [`ShaderLock`](struct.ShaderLock.html)
/// will still set the shader to whatever was set when the `ShaderLock` was created.
pub fn clear_shader(ctx: &mut Context) {
    *ctx.gfx_context.current_shader.borrow_mut() = default_shader::SHADER_ID;
}

/// Sets the additional uniforms used by the given shader.
///
/// Note that the `Projection` uniform is calculated and then appended by good-web-game internally.
pub fn set_uniforms<U: Pod>(ctx: &mut Context, id: ShaderId, extra_uniforms: U) {
    let shader = &mut ctx.gfx_context.shaders[id];
    shader.uniforms.clear();
    shader
        .uniforms
        .extend_from_slice(bytemuck::bytes_of(&extra_uniforms));
    shader
        .uniforms
        .extend_from_slice(&[0u8; std::mem::size_of::<Matrix4<f32>>()]);
}

/// Apply the uniforms for the given shader.
pub(crate) fn apply_uniforms(
    ctx: &mut Context,
    shader_id: ShaderId,
    batch_model: Option<Matrix4<f32>>,
) {
    let projection = if let Some(model) = batch_model {
        ctx.gfx_context.projection * model
    } else {
        ctx.gfx_context.projection
    };
    let projection_bytes = &projection as *const _ as *const u8;

    let current_shader = &mut ctx.gfx_context.shaders[shader_id];
    unsafe {
        let after_last_byte = current_shader
            .uniforms
            .as_mut_ptr_range()
            .end
            .offset(-MATRIX_SIZE);
        for offset in 0..MATRIX_SIZE {
            *after_last_byte.offset(offset) = *projection_bytes.offset(offset);
        }
    }
    ctx.quad_ctx.apply_uniforms_from_bytes(
        current_shader.uniforms.as_ptr(),
        current_shader.uniforms.len(),
    );
}
