use stdweb::{
    traits::*,
    web::{
        event::{
            ContextMenuEvent, KeyDownEvent, KeyUpEvent, MouseButton as WebMouseButton,
            MouseDownEvent, MouseMoveEvent, MouseUpEvent, MouseWheelEvent, ResizeEvent, TouchEnd,
            TouchStart,
        },
        window,
    },
};

use std::{cell::RefCell, rc::Rc};

use crate::{context::Context, error::GameResult, input::input_handler::InputHandler};

mod keycode;

pub use keycode::KeyCode;
pub use crate::input::MouseButton;
pub use crate::input::keyboard::KeyMods;

pub trait EventHandler {
    fn update(&mut self, _ctx: &mut Context) -> GameResult;
    fn draw(&mut self, _ctx: &mut Context) -> GameResult;
    fn resize_event(&mut self, _ctx: &mut Context, _width: f32, _height: f32) {}
    fn mouse_motion_event(&mut self, _ctx: &mut Context, _x: f32, _y: f32, _dx: f32, _dy: f32) {}
    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, _y: f32) {}
    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        _button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
    }
    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        _button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
    }

    fn key_down_event(&mut self, _ctx: &mut Context, _keycode: KeyCode, _keymods: KeyMods, _repeat: bool) {}

    fn key_up_event(&mut self, _ctx: &mut Context, _keycode: KeyCode, _keymods: KeyMods) {}
}

fn animate<S>(
    ctx: Rc<RefCell<Context>>,
    state: Rc<RefCell<S>>,
    input_handler: Rc<RefCell<InputHandler>>,
) where
    S: 'static + EventHandler,
{
    window().request_animation_frame(move |_| {
        {
            let mut state = state.borrow_mut();
            let mut ctx = ctx.borrow_mut();

            ctx.timer_context.tick();

            state.update(&mut *ctx).unwrap();
            state.draw(&mut *ctx).unwrap();

            let mut input_handler = input_handler.borrow_mut();
            input_handler.handle_end_frame();
        }
        animate(ctx, state, input_handler);
    });
}

pub fn run<S>(ctx: Context, state: S) -> GameResult
where
    S: 'static + EventHandler,
{
    let input_handler = ctx.keyboard_context.input_handler.clone();
    let canvas = ctx.gfx_context.canvas().get_canvas();
    let ctx = Rc::new(RefCell::new(ctx));
    let state = Rc::new(RefCell::new(state));

    window().add_event_listener({
        let state = state.clone();
        let ctx = ctx.clone();

        move |_: ResizeEvent| {
            let mut ctx = ctx.borrow_mut();
            let (width, height) = ctx.gfx_context.update_size();

            state
                .borrow_mut()
                .resize_event(&mut *ctx, width as f32, height as f32);
        }
    });

    canvas.add_event_listener({
        let input_handler = input_handler.clone();
        let canvas = canvas.clone();
        let state = state.clone();
        let ctx = ctx.clone();

        move |event: MouseMoveEvent| {
            let rect = canvas.get_bounding_client_rect();

            let mouse_x = event.client_x() - rect.get_left() as i32;
            let mouse_y = event.client_y() - rect.get_top() as i32;

            input_handler
                .borrow_mut()
                .handle_mouse_move(mouse_x, mouse_y);
            state.borrow_mut().mouse_motion_event(
                &mut *ctx.borrow_mut(),
                mouse_x as f32,
                mouse_y as f32,
                event.movement_x() as f32,
                event.movement_y() as f32,
            );
        }
    });

    canvas.add_event_listener({
        let input_handler = input_handler.clone();
        let ctx = ctx.clone();
        let state = state.clone();

        move |event: MouseWheelEvent| {
            event.prevent_default();

            input_handler
                .borrow_mut()
                .handle_mouse_wheel(event.delta_y());
            state.borrow_mut().mouse_wheel_event(
                &mut *ctx.borrow_mut(),
                event.delta_x() as f32,
                event.delta_y() as f32,
            );
        }
    });

    canvas.add_event_listener({
        let input_handler = input_handler.clone();
        let state = state.clone();
        let ctx = ctx.clone();

        move |event: MouseDownEvent| {
            let mut input_handler = input_handler.borrow_mut();
            let button = event.button();

            input_handler.handle_mouse_down(button);
            state.borrow_mut().mouse_button_down_event(
                &mut *ctx.borrow_mut(),
                From::from(&button),
                input_handler.mouse_position.x as f32,
                input_handler.mouse_position.y as f32,
            );
        }
    });

    canvas.add_event_listener({
        let input_handler = input_handler.clone();
        let state = state.clone();
        let ctx = ctx.clone();

        move |event: TouchStart| {
            event.prevent_default();

            for touch in event.touches() {
                let mut input_handler = input_handler.borrow_mut();

                input_handler.handle_mouse_move(touch.page_x() as i32, touch.page_y() as i32);

                state.borrow_mut().mouse_motion_event(
                    &mut *ctx.borrow_mut(),
                    touch.page_x() as f32,
                    touch.page_y() as f32,
                    0.,
                    0.,
                );
                state.borrow_mut().mouse_button_down_event(
                    &mut *ctx.borrow_mut(),
                    MouseButton::Left,
                    touch.page_x() as f32,
                    touch.page_y() as f32,
                );
            }
        }
    });

    canvas.add_event_listener({
        let input_handler = input_handler.clone();
        let state = state.clone();
        let ctx = ctx.clone();

        move |event: TouchEnd| {
            event.prevent_default();

            let input_handler = input_handler.borrow();

            state.borrow_mut().mouse_button_up_event(
                &mut *ctx.borrow_mut(),
                MouseButton::Left,
                input_handler.mouse_position.x as f32,
                input_handler.mouse_position.y as f32,
            );
        }
    });

    canvas.add_event_listener({
        let input_handler = input_handler.clone();
        let ctx = ctx.clone();
        let state = state.clone();

        move |event: ContextMenuEvent| {
            let mut input_handler = input_handler.borrow_mut();
            event.prevent_default();

            input_handler.handle_mouse_down(WebMouseButton::Right);
            state.borrow_mut().mouse_button_down_event(
                &mut *ctx.borrow_mut(),
                MouseButton::Right,
                input_handler.mouse_position.x as f32,
                input_handler.mouse_position.y as f32,
            );
        }
    });

    canvas.add_event_listener({
        let input_handler = input_handler.clone();
        let state = state.clone();
        let ctx = ctx.clone();

        move |event: MouseUpEvent| {
            let mut input_handler = input_handler.borrow_mut();
            let button = event.button();

            input_handler.handle_mouse_up(button);
            state.borrow_mut().mouse_button_up_event(
                &mut *ctx.borrow_mut(),
                From::from(&button),
                input_handler.mouse_position.x as f32,
                input_handler.mouse_position.y as f32,
            );
        }
    });

    canvas.add_event_listener({
        let input_handler = input_handler.clone();
        let state = state.clone();
        let ctx = ctx.clone();

        move |event: KeyDownEvent| {
            let code = KeyCode::from(event.code());
            let repeat = event.repeat();
            let keymods = KeyMods::from_event(&event);

            if code.prevent_default() {
                event.prevent_default();
            }

            if !repeat {
                input_handler.borrow_mut().handle_key_down(event.code());
            }

            state
                .borrow_mut()
                .key_down_event(&mut *ctx.borrow_mut(), code, keymods, repeat);
        }
    });

    canvas.add_event_listener({
        let input_handler = input_handler.clone();
        let state = state.clone();
        let ctx = ctx.clone();

        move |event: KeyUpEvent| {
            let code = KeyCode::from(event.code());
            let repeat = event.repeat();
            let keymods = KeyMods::from_event(&event);

            if !repeat {
                input_handler.borrow_mut().handle_key_up(event.code());
            }

            state
                .borrow_mut()
                .key_up_event(&mut *ctx.borrow_mut(), code, keymods);
        }
    });

    animate(ctx, state, input_handler);

    Ok(())
}
