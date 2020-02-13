use std::cell::RefCell;
use std::rc::Rc;

use crate::{
    conf::{Cache, Conf},
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
    pub quad_ctx: miniquad::Context,
}

impl Context {
    pub(crate) fn new(mut quad_ctx: miniquad::Context, conf: Conf) -> Context {
        let tar = if let Cache::Tar(tar) = conf.cache {
            tar
        } else {
            unimplemented!("Only tar archive filesystem supported")
        };
        let input_handler = Rc::new(RefCell::new(InputHandler::new()));

        Context {
            filesystem: Filesystem::new(&tar),
            gfx_context: graphics::GraphicsContext::new(&mut quad_ctx),
            mouse_context: MouseContext::new(input_handler.clone()),
            keyboard_context: KeyboardContext::new(input_handler.clone()),
            timer_context: TimeContext::new(),
            quad_ctx,
        }
    }

    pub(crate) fn framebuffer(&mut self) -> Option<miniquad::RenderPass> {
        self.gfx_context
            .canvas
            .as_ref()
            .map(|canvas| canvas.offscreen_pass.clone())
    }
}
