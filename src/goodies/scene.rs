//! The Scene system is basically for transitioning between
//! *completely* different states that have entirely different game
//! loops and but which all share a state.  It operates as a stack, with new
//! scenes getting pushed to the stack (while the old ones stay in
//! memory unchanged).  Apparently this is basically a push-down automata.
//!
//! Also there's no reason you can't have a Scene contain its own
//! Scene subsystem to do its own indirection.  With a different state
//! type, as well!  What fun!  Though whether you want to go that deep
//! down the rabbit-hole is up to you.  I haven't found it necessary
//! yet.
//!
//! This is basically identical in concept to the Amethyst engine's scene
//! system, the only difference is the details of how the pieces are put
//! together.

/// A command to change to a new scene, either by pushign a new one,
/// popping one or replacing the current scene (pop and then push).
pub enum SceneSwitch<C> {
    None,
    Push(Box<dyn Scene<C>>),
    Replace(Box<dyn Scene<C>>),
    Pop,
}

/// A trait for you to implement on a scene.
/// Defines the callbacks the scene uses:
/// a common context type `C`
pub trait Scene<C> {
    fn update(&mut self, gameworld: &mut C, ctx: &mut crate::Context) -> SceneSwitch<C>;
    fn draw(&mut self, gameworld: &mut C, ctx: &mut crate::Context) -> crate::GameResult<()>;
    fn resize_event(
        &mut self,
        _gameworld: &mut C,
        _ctx: &mut crate::Context,
        _width: f32,
        _height: f32,
    ) {
    }
    fn mouse_motion_event(
        &mut self,
        _gameworld: &mut C,
        _ctx: &mut crate::Context,
        _x: f32,
        _y: f32,
        _dx: f32,
        _dy: f32,
    ) {
    }
    fn mouse_wheel_event(
        &mut self,
        _gameworld: &mut C,
        _ctx: &mut crate::Context,
        _x: f32,
        _y: f32,
    ) {
    }
    fn mouse_button_down_event(
        &mut self,
        _gameworld: &mut C,
        _ctx: &mut crate::Context,
        _button: crate::event::MouseButton,
        _x: f32,
        _y: f32,
    ) {
    }
    fn mouse_button_up_event(
        &mut self,
        _gameworld: &mut C,
        _ctx: &mut crate::Context,
        _button: crate::event::MouseButton,
        _x: f32,
        _y: f32,
    ) {
    }
    fn key_down_event(&mut self, _gameworld: &mut C, _ctx: &mut crate::Context, _key: crate::event::KeyCode) {}
    fn key_up_event(&mut self, _gameworld: &mut C, _ctx: &mut crate::Context, _key: crate::event::KeyCode) {}

    /// Only used for human-readable convenience (or not at all, tbh)
    fn name(&self) -> &str;
    /// This returns whether or not to draw the next scene down on the
    /// stack as well; this is useful for layers or GUI stuff that
    /// only partially covers the screen.
    fn draw_previous(&self) -> bool {
        false
    }
}

impl<C> SceneSwitch<C> {
    /// Convenient shortcut function for boxing scenes.
    ///
    /// Slightly nicer than writing
    /// `SceneSwitch::Replace(Box::new(x))` all the damn time.
    pub fn replace<S>(scene: S) -> Self
    where
        S: Scene<C> + 'static,
    {
        SceneSwitch::Replace(Box::new(scene))
    }

    /// Same as `replace()` but returns SceneSwitch::Push
    pub fn push<S>(scene: S) -> Self
    where
        S: Scene<C> + 'static,
    {
        SceneSwitch::Push(Box::new(scene))
    }
}

/// A stack of `Scene`'s, together with a context object.
pub struct SceneStack<C> {
    pub world: C,
    scenes: Vec<Box<dyn Scene<C>>>,
}

impl<C> SceneStack<C> {
    pub fn new(_ctx: &mut crate::Context, global_state: C) -> Self {
        Self {
            world: global_state,
            scenes: Vec::new(),
        }
    }

    /// Add a new scene to the top of the stack.
    pub fn push(&mut self, scene: Box<dyn Scene<C>>) {
        self.scenes.push(scene)
    }

    /// Remove the top scene from the stack and returns it;
    /// panics if there is none.
    pub fn pop(&mut self) -> Box<dyn Scene<C>> {
        self.scenes
            .pop()
            .expect("ERROR: Popped an empty scene stack.")
    }

    /// Returns the current scene; panics if there is none.
    pub fn current(&self) -> &dyn Scene<C> {
        &**self
            .scenes
            .last()
            .expect("ERROR: Tried to get current scene of an empty scene stack.")
    }

    /// Executes the given SceneSwitch command; if it is a pop or replace
    /// it returns `Some(old_scene)`, otherwise `None`
    pub fn switch(&mut self, next_scene: SceneSwitch<C>) -> Option<Box<dyn Scene<C>>> {
        match next_scene {
            SceneSwitch::None => None,
            SceneSwitch::Pop => {
                let s = self.pop();
                Some(s)
            }
            SceneSwitch::Push(s) => {
                self.push(s);
                None
            }
            SceneSwitch::Replace(s) => {
                let old_scene = self.pop();
                self.push(s);
                Some(old_scene)
            }
        }
    }

    // These functions must be on the SceneStack because otherwise
    // if you try to get the current scene and the world to call
    // update() on the current scene it causes a double-borrow.  :/
    pub fn update(&mut self, ctx: &mut crate::Context) {
        let next_scene = {
            let current_scene = &mut **self
                .scenes
                .last_mut()
                .expect("Tried to update empty scene stack");
            current_scene.update(&mut self.world, ctx)
        };
        self.switch(next_scene);
    }

    /// We walk down the scene stack until we find a scene where we aren't
    /// supposed to draw the previous one, then draw them from the bottom up.
    ///
    /// This allows for layering GUI's and such.
    fn draw_scenes(scenes: &mut [Box<dyn Scene<C>>], world: &mut C, ctx: &mut crate::Context) {
        assert!(scenes.len() > 0);
        if let Some((current, rest)) = scenes.split_last_mut() {
            if current.draw_previous() {
                SceneStack::draw_scenes(rest, world, ctx);
            }
            current
                .draw(world, ctx)
                .expect("I would hope drawing a scene never fails!");
        }
    }

    /// Draw the current scene.
    pub fn draw(&mut self, ctx: &mut crate::Context) {
        SceneStack::draw_scenes(&mut self.scenes, &mut self.world, ctx)
    }
}

impl<C> crate::event::EventHandler for SceneStack<C> {
    fn update(&mut self, ctx: &mut crate::Context) -> crate::error::GameResult {
        self.update(ctx);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut crate::Context) -> crate::error::GameResult {
        self.draw(ctx);
        Ok(())
    }

    fn resize_event(&mut self, ctx: &mut crate::Context, width: f32, height: f32) {
        let current_scene = &mut **self
            .scenes
            .last_mut()
            .expect("Tried to update empty scene stack");

        current_scene.resize_event(&mut self.world, ctx, width, height)
    }

    fn mouse_motion_event(&mut self, ctx: &mut crate::Context, x: f32, y: f32, dx: f32, dy: f32) {
        let current_scene = &mut **self
            .scenes
            .last_mut()
            .expect("Tried to update empty scene stack");

        current_scene.mouse_motion_event(&mut self.world, ctx, x, y, dx, dy);
    }

    fn mouse_wheel_event(&mut self, ctx: &mut crate::Context, x: f32, y: f32) {
        let current_scene = &mut **self
            .scenes
            .last_mut()
            .expect("Tried to update empty scene stack");

        current_scene.mouse_wheel_event(&mut self.world, ctx, x, y);
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut crate::Context,
        button: crate::event::MouseButton,
        x: f32,
        y: f32,
    ) {
        let current_scene = &mut **self
            .scenes
            .last_mut()
            .expect("Tried to update empty scene stack");

        current_scene.mouse_button_down_event(&mut self.world, ctx, button, x, y);
    }
    fn mouse_button_up_event(
        &mut self,
        ctx: &mut crate::Context,
        button: crate::event::MouseButton,
        x: f32,
        y: f32,
    ) {
        let current_scene = &mut **self
            .scenes
            .last_mut()
            .expect("Tried to update empty scene stack");

        current_scene.mouse_button_up_event(&mut self.world, ctx, button, x, y);
    }

    fn key_down_event(&mut self, ctx: &mut crate::Context, keycode: crate::event::KeyCode, _keymods: crate::event::KeyMods, _repeat: bool) {
        let current_scene = &mut **self
            .scenes
            .last_mut()
            .expect("Tried to update empty scene stack");

        current_scene.key_down_event(&mut self.world, ctx, keycode);
    }

    fn key_up_event(&mut self, ctx: &mut crate::Context, keycode: crate::event::KeyCode, _keymods: crate::event::KeyMods) {
        let current_scene = &mut **self
            .scenes
            .last_mut()
            .expect("Tried to update empty scene stack");

        current_scene.key_up_event(&mut self.world, ctx, keycode);
    }
}
