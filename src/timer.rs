//! Timing and measurement functions.
//!
//! good-web-game does not try to do any framerate limitation by default (because it can't).
//!
//! For a detailed tutorial in how to handle frame timings in games,
//! see <http://gafferongames.com/game-physics/fix-your-timestep/>

use crate::context::Context;

use std::cmp;
use std::f64;
use std::time;
use std::time::Duration;

type Instant = f64;

/// Returns the system time.
pub fn time() -> f64 {
    miniquad::date::now()
}

/// A simple buffer that fills
/// up to a limit and then holds the last
/// N items that have been inserted into it,
/// overwriting old ones in a round-robin fashion.
///
/// It's not quite a ring buffer 'cause you can't
/// remove items from it, it just holds the last N
/// things.
#[derive(Debug, Clone)]
struct LogBuffer<T>
where
    T: Clone,
{
    head: usize,
    size: usize,
    /// The number of actual samples inserted, used for
    /// smarter averaging.
    samples: usize,
    contents: Vec<T>,
}

impl<T> LogBuffer<T>
where
    T: Clone + Copy,
{
    fn new(size: usize, init_val: T) -> LogBuffer<T> {
        LogBuffer {
            head: 0,
            size,
            contents: vec![init_val; size],
            // Never divide by 0
            samples: 1,
        }
    }

    /// Pushes a new item into the `LogBuffer`, overwriting
    /// the oldest item in it.
    fn push(&mut self, item: T) {
        self.head = (self.head + 1) % self.contents.len();
        self.contents[self.head] = item;
        self.size = cmp::min(self.size + 1, self.contents.len());
        self.samples += 1;
    }

    /// Returns a slice pointing at the contents of the buffer.
    /// They are in *no particular order*, and if not all the
    /// slots are filled, the empty slots will be present but
    /// contain the initial value given to [`new()`](#method.new).
    ///
    /// We're only using this to log FPS for a short time,
    /// so we don't care for the second or so when it's inaccurate.
    fn contents(&self) -> &[T] {
        if self.samples > self.size {
            &self.contents
        } else {
            &self.contents[..self.samples]
        }
    }

    /// Returns the most recent value in the buffer.
    fn latest(&self) -> T {
        self.contents[self.head]
    }
}

/// A structure that contains our time-tracking state.
#[derive(Debug)]
pub struct TimeContext {
    init_instant: Instant,
    last_instant: Instant,
    frame_durations: LogBuffer<Duration>,
    residual_update_dt: Duration,
    frame_count: usize,
}

// How many frames we log update times for.
const TIME_LOG_FRAMES: usize = 200;

impl TimeContext {
    /// Creates a new `TimeContext` and initializes the start to this instant.
    pub fn new() -> TimeContext {
        let initial_dt = time::Duration::from_millis(16);
        TimeContext {
            init_instant: time(),
            last_instant: time(),
            frame_durations: LogBuffer::new(TIME_LOG_FRAMES, initial_dt),
            residual_update_dt: time::Duration::from_secs(0),
            frame_count: 0,
        }
    }

    /// Update the state of the `TimeContext` to record that
    /// another frame has taken place.  Necessary for the FPS
    /// tracking and [`check_update_time()`](fn.check_update_time.html)
    /// functions to work.
    ///
    /// It's usually not necessary to call this function yourself,
    /// [`event::run()`](../event/fn.run.html) will do it for you.
    pub fn tick(&mut self) {
        let now = time();
        let time_since_last = now - self.last_instant;
        self.frame_durations.push(f64_to_duration(time_since_last));
        self.last_instant = now;
        self.frame_count += 1;

        self.residual_update_dt += f64_to_duration(time_since_last);
    }
}

impl Default for TimeContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the time between the start of the last frame and the current one;
/// in other words, the length of the last frame.
pub fn delta(ctx: &Context) -> Duration {
    let tc = &ctx.timer_context;
    tc.frame_durations.latest()
}

/// Gets the average time of a frame, averaged
/// over the last 200 frames.
pub fn average_delta(ctx: &Context) -> Duration {
    let tc = &ctx.timer_context;
    let sum: Duration = tc.frame_durations.contents().iter().sum();
    // If our buffer is actually full, divide by its size.
    // Otherwise divide by the number of samples we've added
    if tc.frame_durations.samples > tc.frame_durations.size {
        sum / (tc.frame_durations.size as u32)
    } else {
        sum / (tc.frame_durations.samples as u32)
    }
}

/// A convenience function to convert a Rust `Duration` type
/// to a (less precise but more useful) `f64`.
///
/// Does not make sure that the `Duration` is within the bounds
/// of the `f64`.
pub fn duration_to_f64(d: Duration) -> f64 {
    d.as_secs() as f64 + d.subsec_nanos() as f64 * 1e-9
}

/// A convenience function to create a Rust `Duration` type
/// from a (less precise but more useful) `f64`.
///
/// Only handles positive numbers correctly.
pub fn f64_to_duration(t: f64) -> Duration {
    let seconds = t.trunc();
    let nanos = t.fract() * 1.0e9;

    Duration::new(seconds as u64, nanos as u32)
}

/// Returns a `Duration` representing how long each
/// frame should be to match the given fps.
///
/// Approximately.
fn fps_as_duration(fps: u32) -> Duration {
    let target_dt_seconds = 1.0 / f64::from(fps);
    f64_to_duration(target_dt_seconds)
}

/// Gets the FPS of the game, averaged over the last
/// 200 frames.
pub fn fps(ctx: &Context) -> f64 {
    let duration_per_frame = average_delta(ctx);
    let seconds_per_frame = duration_to_f64(duration_per_frame);
    1.0 / seconds_per_frame
}

/// Returns the time since the game was initialized,
/// as reported by the system clock.
pub fn time_since_start(ctx: &Context) -> Duration {
    let tc = &ctx.timer_context;
    f64_to_duration(time() - tc.init_instant)
}

/// same as time_since_start, but without f64_to_duration inside
pub fn time_since_start_f64(ctx: &Context) -> f64 {
    let tc = &ctx.timer_context;
    time() - tc.init_instant
}

/// This function will return true if the time since the
/// last [`update()`](../event/trait.EventHandler.html#tymethod.update)
/// call has been equal to or greater to
/// the update FPS indicated by the `target_fps`.
/// It keeps track of fractional frames, so if you want
/// 60 fps (16.67 ms/frame) and the game stutters so that
/// there is 40 ms between `update()` calls, this will return
/// `true` twice, and take the remaining 6.67 ms into account
/// in the next frame.
///
/// The intention is to for it to be called in a while loop
/// in your `update()` callback:
///
/// ```rust
/// # use ggez::*;
/// # fn update_game_physics() -> GameResult { Ok(()) }
/// # struct State;
/// # impl ya2d::event::EventHandler for State {
/// fn update(&mut self, ctx: &mut Context) -> GameResult {
///     while(timer::check_update_time(ctx, 60)) {
///         update_game_physics()?;
///     }
///     Ok(())
/// }
/// # fn draw(&mut self, _ctx: &mut Context) -> GameResult { Ok(()) }
/// # }
/// ```
pub fn check_update_time(ctx: &mut Context, target_fps: u32) -> bool {
    let timedata = &mut ctx.timer_context;

    let target_dt = fps_as_duration(target_fps);
    if timedata.residual_update_dt > target_dt {
        timedata.residual_update_dt -= target_dt;
        true
    } else {
        false
    }
}

/// Returns the fractional amount of a frame not consumed
/// by  [`check_update_time()`](fn.check_update_time.html).
/// For example, if the desired
/// update frame time is 40 ms (25 fps), and 45 ms have
/// passed since the last frame, [`check_update_time()`](fn.check_update_time.html)
/// will return `true` and `remaining_update_time()` will
/// return 5 ms -- the amount of time "overflowing" from one
/// frame to the next.
///
/// The intention is for it to be called in your
/// [`draw()`](../event/trait.EventHandler.html#tymethod.draw) callback
/// to interpolate physics states for smooth rendering.
/// (see <http://gafferongames.com/game-physics/fix-your-timestep/>)
pub fn remaining_update_time(ctx: &mut Context) -> Duration {
    ctx.timer_context.residual_update_dt
}

/// Gets the number of times the game has gone through its event loop.
///
/// Specifically, the number of times that [`TimeContext::tick()`](struct.TimeContext.html#method.tick)
/// has been called by it.
pub fn ticks(ctx: &Context) -> usize {
    ctx.timer_context.frame_count
}
