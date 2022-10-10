//! A more sophisticated example of how to use shaders
//! and canvas's to do 2D GPU shadows.
//!
//! Sadly, it doesn't work as intended yet. (see the original ggez example for comparison)
//! If you know how to fix it, please speak up! Your help is very welcome.

extern crate good_web_game as ggez;

use ggez::event;
use ggez::graphics::{
    self, set_uniforms, BlendMode, Canvas, Color, DrawParam, Drawable, Shader, ShaderId,
    ShaderMeta, UniformBlockLayout, UniformDesc, UniformType,
};
use ggez::miniquad;
use ggez::timer;
use ggez::{Context, GameResult};
use glam::{Vec2, Vec4};

// Define the input struct for our shader.
fn shader_meta() -> ShaderMeta {
    ShaderMeta {
        images: vec!["t_Texture".to_string()],
        uniforms: UniformBlockLayout {
            uniforms: vec![
                UniformDesc::new("u_LightColor", UniformType::Float4),
                UniformDesc::new("u_ShadowColor", UniformType::Float4),
                UniformDesc::new("u_Pos", UniformType::Float2),
                UniformDesc::new("u_ScreenSize", UniformType::Float2),
                UniformDesc::new("u_Glow", UniformType::Float1),
                UniformDesc::new("u_Strength", UniformType::Float1),
                UniformDesc::new("_padding", UniformType::Int2),
                // `Projection` always comes last, as it is appended by gwg internally
                UniformDesc::new("Projection", UniformType::Mat4),
            ],
        },
    }
}

use bytemuck_derive::{Pod, Zeroable};
#[repr(C)]
#[derive(Copy, Clone, Zeroable, Pod)]
// The attributes above ensure that this struct derives bytemuck::Pod,
// which is necessary for it to be passed to `graphics::set_uniforms`.
// Luckily glam implemented Pod for these types so we can simply derive it.
pub struct Light {
    pub u_light_color: Vec4,
    pub u_shadow_color: Vec4,
    pub u_pos: Vec2,
    pub u_screen_size: Vec2,
    pub u_glow: f32,
    pub u_strength: f32,
    pub _padding: u64,
}

/// Shader source for calculating a 1D shadow map that encodes half distances
/// in the red channel. The idea is that we scan X rays (X is the horizontal
/// size of the output) and calculate the distance to the nearest pixel at that
/// angle that has transparency above a threshold. The distance gets halved
/// and encoded in the red channel (it is halved because if the distance can be
/// greater than 1.0 - think bottom left to top right corner, that sqrt(1) and
/// will not get properly encoded).
const OCCLUSIONS_SHADER_SOURCE: &[u8] = b"#version 150 core
uniform sampler2D t_Texture;
in vec2 v_Uv;
out vec4 Target0;
layout (std140) uniform Light {
    vec4 u_LightColor;
    vec4 u_ShadowColor;
    vec2 u_Pos;
    vec2 u_ScreenSize;
    float u_Glow;
    float u_Strength;
    vec2 _padding;
};
void main() {
    float dist = 1.0;
    float theta = radians(v_Uv.x * 360.0);
    vec2 dir = vec2(cos(theta), sin(theta));
    for(int i = 0; i < 1024; i++) {
        float fi = i;
        float r = fi / 1024.0;
        vec2 rel = r * dir;
        vec2 p = clamp(u_Pos+rel, 0.0, 1.0);
        if (texture(t_Texture, p).a > 0.8) {
            dist = distance(u_Pos, p) * 0.5;
            break;
        }
    }
    float others = dist == 1.0 ? 0.0 : dist;
    Target0 = vec4(dist, others, others, 1.0);
}
";

const VERTEX_SHADER_SOURCE: &[u8] = include_bytes!("../resources/basic_150.glslv");

/// Shader for drawing shadows based on a 1D shadow map. It takes current
/// fragment coordinates and converts them to polar coordinates centered
/// around the light source, using the angle to sample from the 1D shadow map.
/// If the distance from the light source is greater than the distance of the
/// closest reported shadow, then the output is the shadow color, else it calculates some
/// shadow based on the distance from light source based on strength and glow
/// uniform parameters.
const SHADOWS_SHADER_SOURCE: &[u8] = b"#version 150 core
uniform sampler2D t_Texture;
in vec2 v_Uv;
out vec4 Target0;
layout (std140) uniform Light {
    vec4 u_LightColor;
    vec4 u_ShadowColor;
    vec2 u_Pos;
    vec2 u_ScreenSize;
    float u_Glow;
    float u_Strength;
    vec2 _padding;
};
void main() {
    vec2 coord = gl_FragCoord.xy / u_ScreenSize;
    vec2 rel = coord - u_Pos;
    float theta = atan(rel.y, rel.x);
    float ox = degrees(theta) / 360.0;
    if (ox < 0) {
        ox += 1.0;
    }
    float r = length(rel);
    float occl = texture(t_Texture, vec2(ox, 0.5)).r * 2.0;
    float intensity = 1.0;
    if (r < occl) {
        vec2 g = u_ScreenSize / u_ScreenSize.y;
        float p = u_Strength + u_Glow;
        float d = distance(g * coord, g * u_Pos);
        intensity = 1.0 - clamp(p/(d*d), 0.0, 1.0);
    }
    Target0 = mix(vec4(1.0, 1.0, 1.0, 1.0), vec4(u_ShadowColor.rgb, 1.0), intensity);
}
";

/// Shader for drawing lights based on a 1D shadow map. It takes current
/// fragment coordinates and converts them to polar coordinates centered
/// around the light source, using the angle to sample from the 1D shadow map.
/// If the distance from the light source is greater than the distance of the
/// closest reported shadow, then the output is black, else it calculates some
/// light based on the distance from light source based on strength and glow
/// uniform parameters. It is meant to be used additively for drawing multiple
/// lights.
const LIGHTS_SHADER_SOURCE: &[u8] = b"#version 150 core
uniform sampler2D t_Texture;
in vec2 v_Uv;
out vec4 Target0;
layout (std140) uniform Light {
    vec4 u_LightColor;
    vec4 u_ShadowColor;
    vec2 u_Pos;
    vec2 u_ScreenSize;
    float u_Glow;
    float u_Strength;
    vec2 _padding;
};
void main() {
    vec2 coord = gl_FragCoord.xy / u_ScreenSize;
    vec2 rel = coord - u_Pos;
    float theta = atan(rel.y, rel.x);
    float ox = degrees(theta) / 360.0;
    if (ox < 0) {
        ox += 1.0;
    }
    float r = length(rel);
    float occl = texture(t_Texture, vec2(ox, 0.5)).r * 2.0;
    float intensity = 0.0;
    if (r < occl) {
        vec2 g = u_ScreenSize / u_ScreenSize.y;
        float p = u_Strength + u_Glow;
        float d = distance(g * coord, g * u_Pos);
        intensity = clamp(p/(d*d), 0.0, 0.6);
    }
    Target0 = mix(vec4(0.0, 0.0, 0.0, 1.0), vec4(u_LightColor.rgb, 1.0), intensity);
}
";

struct MainState {
    background: graphics::Image,
    tile: graphics::Image,
    text: graphics::Text,
    torch: Light,
    static_light: Light,
    foreground: Canvas,
    occlusions: Canvas,
    shadows: Canvas,
    lights: Canvas,
    occlusions_shader: ShaderId,
    shadows_shader: ShaderId,
    lights_shader: ShaderId,
}

/// The color cast things take when not illuminated
const AMBIENT_COLOR: [f32; 4] = [0.25, 0.22, 0.34, 1.0];
/// The default color for the static light
const STATIC_LIGHT_COLOR: [f32; 4] = [0.37, 0.69, 0.75, 1.0];
/// The default color for the mouse-controlled torch
const TORCH_COLOR: [f32; 4] = [0.80, 0.73, 0.44, 1.0];
/// The number of rays to cast to. Increasing this number will result in better
/// quality shadows. If you increase too much you might hit some GPU shader
/// hardware limits.
const LIGHT_RAY_COUNT: u16 = 1440;
/// The strength of the light - how far it shines
const LIGHT_STRENGTH: f32 = 0.0035;
/// The factor at which the light glows - just for fun
const LIGHT_GLOW_FACTOR: f32 = 0.0001;
/// The rate at which the glow effect oscillates
const LIGHT_GLOW_RATE: f32 = 50.0;

impl MainState {
    fn new(ctx: &mut Context, quad_ctx: &mut miniquad::GraphicsContext) -> GameResult<MainState> {
        let background = graphics::Image::new(ctx, quad_ctx, "/bg_top.png")?;
        let tile = graphics::Image::new(ctx, quad_ctx, "/tile.png")?;
        let text = {
            let font = graphics::Font::new(ctx, "/LiberationMono-Regular.ttf")?;
            graphics::Text::new(("SHADOWS...", font, 48.0))
        };
        let screen_size = {
            let size = graphics::drawable_size(quad_ctx);
            [size.0 as f32, size.1 as f32]
        };
        let torch = Light {
            u_light_color: Vec4::from(TORCH_COLOR),
            u_shadow_color: Vec4::from(AMBIENT_COLOR),
            u_pos: [0.0, 0.0].into(),
            u_screen_size: Vec2::from(screen_size),
            u_glow: 0.0,
            u_strength: LIGHT_STRENGTH,
            _padding: 0,
        };
        let (w, h) = graphics::drawable_size(quad_ctx);
        let (x, y) = (100.0 / w as f32, 1.0 - 75.0 / h as f32);
        let static_light = Light {
            u_pos: [x, y].into(),
            u_light_color: Vec4::from(STATIC_LIGHT_COLOR),
            u_shadow_color: Vec4::from(AMBIENT_COLOR),
            u_screen_size: Vec2::from(screen_size),
            u_glow: 0.0,
            u_strength: LIGHT_STRENGTH,
            _padding: 0,
        };
        let foreground = Canvas::with_window_size(ctx, quad_ctx)?;
        let occlusions = Canvas::new(ctx, quad_ctx, LIGHT_RAY_COUNT, 1)?;
        let mut shadows = Canvas::with_window_size(ctx, quad_ctx)?;
        // The shadow map will be drawn on top using the multiply blend mode
        shadows.set_blend_mode(Some(BlendMode::Multiply));
        let mut lights = Canvas::with_window_size(ctx, quad_ctx)?;
        // The light map will be drawn on top using the add blend mode
        lights.set_blend_mode(Some(BlendMode::Add));
        let occlusions_shader = Shader::from_u8(
            ctx,
            quad_ctx,
            VERTEX_SHADER_SOURCE,
            OCCLUSIONS_SHADER_SOURCE,
            shader_meta(),
            None,
        )
        .unwrap();
        let shadows_shader = Shader::from_u8(
            ctx,
            quad_ctx,
            VERTEX_SHADER_SOURCE,
            SHADOWS_SHADER_SOURCE,
            shader_meta(),
            None,
        )
        .unwrap();
        let lights_shader = Shader::from_u8(
            ctx,
            quad_ctx,
            VERTEX_SHADER_SOURCE,
            LIGHTS_SHADER_SOURCE,
            shader_meta(),
            Some(BlendMode::Add),
        )
        .unwrap();

        Ok(MainState {
            background,
            tile,
            text,
            torch,
            static_light,
            foreground,
            occlusions,
            shadows,
            lights,
            occlusions_shader,
            shadows_shader,
            lights_shader,
        })
    }
    fn render_light(
        &mut self,
        ctx: &mut Context,
        quad_ctx: &mut miniquad::GraphicsContext,
        light: Light,
        origin: DrawParam,
        canvas_origin: DrawParam,
    ) -> GameResult {
        let size = graphics::drawable_size(quad_ctx);
        // Now we want to run the occlusions shader to calculate our 1D shadow
        // distances into the `occlusions` canvas.
        graphics::set_canvas(ctx, Some(&self.occlusions));
        {
            let _shader_lock = graphics::use_shader(ctx, self.occlusions_shader);

            set_uniforms(ctx, self.occlusions_shader, light);
            graphics::draw(ctx, quad_ctx, &self.foreground, canvas_origin)?;
        }

        // Now we render our shadow map and light map into their respective
        // canvases based on the occlusion map. These will then be drawn onto
        // the final render target using appropriate blending modes.
        graphics::set_canvas(ctx, Some(&self.shadows));
        {
            let _shader_lock = graphics::use_shader(ctx, self.shadows_shader);

            let param = origin.scale(Vec2::new(
                (size.0 as f32) / (LIGHT_RAY_COUNT as f32),
                size.1 as f32,
            ));
            set_uniforms(ctx, self.shadows_shader, light);
            graphics::draw(ctx, quad_ctx, &self.occlusions, param)?;
        }
        graphics::set_canvas(ctx, Some(&self.lights));
        {
            let _shader_lock = graphics::use_shader(ctx, self.lights_shader);

            let param = origin.scale(Vec2::new(
                (size.0 as f32) / (LIGHT_RAY_COUNT as f32),
                size.1 as f32,
            ));
            set_uniforms(ctx, self.lights_shader, light);
            graphics::draw(ctx, quad_ctx, &self.occlusions, param)?;
        }
        Ok(())
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(
        &mut self,
        ctx: &mut Context,
        _quad_ctx: &mut miniquad::GraphicsContext,
    ) -> GameResult {
        if timer::ticks(ctx) % 100 == 0 {
            println!("Average FPS: {}", timer::fps(ctx));
        }

        self.torch.u_glow =
            LIGHT_GLOW_FACTOR * ((timer::ticks(ctx) as f32) / LIGHT_GLOW_RATE).cos();
        self.static_light.u_glow =
            LIGHT_GLOW_FACTOR * ((timer::ticks(ctx) as f32) / LIGHT_GLOW_RATE * 0.75).sin();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, quad_ctx: &mut miniquad::GraphicsContext) -> GameResult {
        let origin = DrawParam::new()
            .dest(Vec2::new(0.0, 0.0))
            .scale(Vec2::new(0.5, 0.5));
        let canvas_origin = DrawParam::new();

        // First thing we want to do it to render all the foreground items (that
        // will have shadows) onto their own Canvas (off-screen render). We will
        // use this canvas to:
        //  - run the occlusions shader to determine where the shadows are
        //  - render to screen once all the shadows are calculated and rendered
        {
            graphics::set_canvas(ctx, Some(&self.foreground));
            graphics::clear(ctx, quad_ctx, graphics::Color::new(0.0, 0.0, 0.0, 0.0));
            graphics::draw(
                ctx,
                quad_ctx,
                &self.tile,
                DrawParam::new().dest(Vec2::new(598.0, 124.0)),
            )?;
            graphics::draw(
                ctx,
                quad_ctx,
                &self.tile,
                DrawParam::new().dest(Vec2::new(92.0, 350.0)),
            )?;
            graphics::draw(
                ctx,
                quad_ctx,
                &self.tile,
                DrawParam::new().dest(Vec2::new(442.0, 468.0)).rotation(0.5),
            )?;
            graphics::draw(ctx, quad_ctx, &self.text, (Vec2::new(50.0, 200.0),))?;
        }

        // Then we draw our light and shadow maps
        {
            let torch = self.torch;
            let light = self.static_light;

            graphics::set_canvas(ctx, Some(&self.lights));
            graphics::clear(ctx, quad_ctx, graphics::Color::new(0.0, 0.0, 0.0, 1.0));

            graphics::set_canvas(ctx, Some(&self.shadows));
            graphics::clear(ctx, quad_ctx, graphics::Color::new(0.0, 0.0, 0.0, 1.0));
            self.render_light(ctx, quad_ctx, torch, origin, canvas_origin)?;
            self.render_light(ctx, quad_ctx, light, origin, canvas_origin)?;
        }

        // Now lets finally render to screen starting out with background, then
        // the shadows and lights overtop and finally our foreground.
        graphics::set_canvas(ctx, None);
        graphics::clear(ctx, quad_ctx, Color::WHITE);
        graphics::draw(ctx, quad_ctx, &self.background, DrawParam::default())?;
        graphics::draw(ctx, quad_ctx, &self.shadows, DrawParam::default())?;
        graphics::draw(ctx, quad_ctx, &self.foreground, DrawParam::default())?;
        graphics::draw(ctx, quad_ctx, &self.lights, DrawParam::default())?;

        // Uncomment following line to visualize the 1D occlusions canvas,
        // red pixels represent angles at which no shadows were found, and then
        // the greyscale pixels are the half distances of the nearest shadows to
        // the mouse position (equally encoded in all color channels).
        // graphics::draw(ctx, &self.occlusions, DrawParam::default())?;

        graphics::present(ctx, quad_ctx)?;
        Ok(())
    }

    fn mouse_motion_event(
        &mut self,
        _ctx: &mut Context,
        quad_ctx: &mut miniquad::GraphicsContext,
        x: f32,
        y: f32,
        _xrel: f32,
        _yrel: f32,
    ) {
        let (w, h) = graphics::drawable_size(quad_ctx);
        let (x, y) = (x / w as f32, 1.0 - y / h as f32);
        self.torch.u_pos = Vec2::from([x, y]);
    }
}

pub fn main() -> GameResult {
    ggez::start(
        ggez::conf::Conf::default().cache(Some(include_bytes!("resources.tar"))),
        |mut context, quad_ctx| Box::new(MainState::new(&mut context, quad_ctx).unwrap()),
    )
}
