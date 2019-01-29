use crate::graphics::types::*;
use stdweb::{web::html_element::*, web::*};

use cgmath::Matrix3;

pub(crate) mod canvas;
pub(crate) mod webgl;

use self::{canvas::CanvasContext, webgl::WebGlContext};

pub struct GraphicsContext {
    pub canvas_context: CanvasContext,
    pub webgl_context: WebGlContext,
}

impl GraphicsContext {
    pub fn new(canvas: CanvasElement, glcanvas: CanvasElement) -> GraphicsContext {
        let canvas_context: CanvasRenderingContext2d = canvas.get_context().unwrap();

        GraphicsContext {
            canvas_context: CanvasContext::new(canvas_context),
            webgl_context: WebGlContext::new(glcanvas),
        }
    }
}

impl GraphicsContext {
    pub(crate) fn update_size(&self) -> (u32, u32) {
        let canvas = self.canvas().get_canvas();

        let (width, height) = self.canvas_context.size();

        resize_canvas(&canvas, width, height);
        resize_canvas(&self.webgl_context.canvas, width, height);

        self.webgl_context.resize(width as u32, height as u32);

        (width as u32, height as u32)
    }

    pub fn clear(&mut self, color: Color) {
        self.webgl_context.clear(color);
        self.canvas_context.clear();
    }

    pub fn canvas(&self) -> &CanvasRenderingContext2d {
        &self.canvas_context.canvas
    }

    pub fn set_transform(&self, transform: &Matrix3<f32>) {
        self.canvas_context.set_transform(transform);
    }

    pub fn push_transform(&self, transform: &Matrix3<f32>) {
        self.canvas_context.push_transform(transform);
    }

    pub fn pop_transform(&self) {
        self.canvas_context.pop_transform();
    }

    pub fn size(&self) -> (f64, f64) {
        self.canvas_context.size()
    }

    pub fn set_screen_coordinates(&mut self, rect: crate::graphics::types::Rect) {
        self.canvas_context.set_screen_coordinates(rect);
        self.webgl_context.set_projection_rect(rect);
    }
}

fn resize_canvas(canvas: &CanvasElement, w: f64, h: f64) {
    canvas.set_width(w as u32);
    canvas.set_height(h as u32);
}
