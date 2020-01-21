use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::{
    conf::{Cache, Conf},
    filesystem::Filesystem,
    graphics,
    graphics::GpuText,
    input::{input_handler::InputHandler, KeyboardContext, MouseContext},
    timer::TimeContext,
};

pub struct ContextInternal {
    pub filesystem: Filesystem,
    pub gfx_context: graphics::GraphicsContext,
    pub mouse_context: MouseContext,
    pub keyboard_context: KeyboardContext,
    pub timer_context: TimeContext,
}

pub struct Context<'a, 'b> {
    pub internal: &'a mut ContextInternal,
    pub quad_ctx: &'b mut miniquad::Context,
}

impl<'a, 'b> Context<'a, 'b> {
    pub(crate) fn text_cache(&mut self) -> &mut HashMap<String, GpuText> {
        &mut self.internal.gfx_context.text_cache
    }

    pub(crate) fn framebuffer(&mut self) -> Option<miniquad::RenderPass> {
        self.internal
            .gfx_context
            .canvas
            .as_ref()
            .map(|canvas| canvas.offscreen_pass.clone())
    }
}
impl ContextInternal {
    pub(crate) fn new(quad_ctx: &mut miniquad::Context, conf: Conf) -> ContextInternal {
        let tar = if let Cache::Tar(tar) = conf.cache {
            tar
        } else {
            unimplemented!("Only tar archive filesystem supported")
        };
        let input_handler = Rc::new(RefCell::new(InputHandler::new()));

        ContextInternal {
            filesystem: Filesystem::new(&tar),
            gfx_context: graphics::GraphicsContext::new(quad_ctx),
            mouse_context: MouseContext::new(input_handler.clone()),
            keyboard_context: KeyboardContext::new(input_handler.clone()),
            timer_context: TimeContext::new(),
        }
    }
}
