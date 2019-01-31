use stdweb::traits::*;

use stdweb::{
    unstable::TryInto,
    web::{document, html_element::CanvasElement},
};

use std::cell::RefCell;
use std::rc::Rc;

use crate::{
    filesystem::Filesystem,
    graphics,
    input::{input_handler::InputHandler, KeyboardContext, MouseContext},
    timer::TimeContext,
};

pub struct Context {
    pub filesystem: Filesystem,
    pub gfx_context: graphics::GraphicsContext,
    pub mouse_context: MouseContext,
    pub keyboard_context: KeyboardContext,
    pub timer_context: TimeContext,
}

impl Context {
    pub(crate) fn build(filesystem: Filesystem) -> Context {
        let canvas: CanvasElement = document()
            .query_selector("#canvas")
            .expect("Can't find #canvas tagged element!")
            .expect("Can't find #canvas tagged element!")
            .try_into()
            .expect("#canvas element is not a canvas");
        let glcanvas: CanvasElement = document()
            .query_selector("#glcanvas")
            .expect("Can't find #glcanvas tagged element!")
            .expect("Can't find #glcanvas tagged element!")
            .try_into()
            .expect("#glcanvas element is not a canvas");

        let gfx_context = graphics::GraphicsContext::new(canvas, glcanvas);

        gfx_context.update_size();

        let input_handler = Rc::new(RefCell::new(InputHandler::new()));

        Context {
            filesystem,
            gfx_context,
            mouse_context: MouseContext::new(input_handler.clone()),
            keyboard_context: KeyboardContext::new(input_handler.clone()),
            timer_context: TimeContext::new(),
        }
    }

    pub fn canvas_context(&self) -> &graphics::CanvasContext {
        &self.gfx_context.canvas_context
    }
}
