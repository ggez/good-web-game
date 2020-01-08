use crate::graphics::{types::Rect, Canvas};

use cgmath::{Matrix3, Matrix4};

use std::collections::HashMap;

const FONT_TEXTURE_BYTES: &'static [u8] = include_bytes!("font.png");

pub struct GpuText {
    pub bindings: miniquad::Bindings,
    pub pipeline: miniquad::Pipeline,
}

pub struct GraphicsContext {
    pub(crate) screen_rect: Rect,
    pub(crate) projection: Matrix4<f32>,
    pub(crate) font_texture: miniquad::Texture,
    pub(crate) text_cache: HashMap<String, GpuText>,
    pub(crate) canvas: Option<Canvas>,
}

impl GraphicsContext {
    pub fn new() -> GraphicsContext {
        let projection = cgmath::One::one();
        let screen_rect = Rect::new(-1., -1., 2., 2.);

        let img = image::load_from_memory(FONT_TEXTURE_BYTES)
            .unwrap_or_else(|e| panic!(e))
            .to_rgba();
        let width = img.width() as u16;
        let height = img.height() as u16;
        let bytes = img.into_raw();

        let font_texture = miniquad::Texture::from_rgba8(width, height, &bytes);

        GraphicsContext {
            projection,
            screen_rect,
            font_texture,
            text_cache: HashMap::new(),
            canvas: None,
        }
    }
}

impl GraphicsContext {
    pub fn set_transform(&mut self, _transform: &Matrix3<f32>) {
        unimplemented!();
    }

    pub fn push_transform(&mut self, _transform: &Matrix3<f32>) {
        unimplemented!();
    }

    pub fn pop_transform(&mut self) {
        unimplemented!();
    }

    pub fn set_screen_coordinates(&mut self, rect: crate::graphics::types::Rect) {
        self.screen_rect = rect;
        self.projection =
            cgmath::ortho(rect.x, rect.x + rect.w, rect.y + rect.h, rect.y, -1.0, 1.0);
    }
}
