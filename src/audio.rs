//! Loading and playing sounds.
//!
//! Note that audio functionality in good-web-game is very different from ggez, as it uses quad-snd
//! instead of rodio, for maximum portability.

use crate::{filesystem, Context, GameResult};
use std::collections::HashMap;

#[cfg(all(feature = "audio", not(target_os = "ios")))]
use quad_snd::{AudioContext as QuadSndContext, Sound as QuadSndSound};

#[cfg(all(feature = "audio", not(target_os = "ios")))]
pub use quad_snd::PlaySoundParams;

#[cfg(any(not(feature = "audio"), target_os = "ios"))]
mod dummy_audio {
    use crate::audio::PlaySoundParams;

    pub struct AudioContext {}

    impl AudioContext {
        pub fn new() -> AudioContext {
            AudioContext {}
        }

        pub fn pause(&mut self) {}

        pub fn resume(&mut self) {}
    }

    pub struct Sound {}

    impl Sound {
        pub fn load(_ctx: &mut AudioContext, _data: &[u8]) -> Sound {
            Sound {}
        }

        pub fn is_loaded(&self) -> bool {
            true
        }

        pub fn play(&mut self, _ctx: &mut AudioContext, _params: PlaySoundParams) {}

        pub fn stop(&mut self, _ctx: &mut AudioContext) {}

        pub fn set_volume(&mut self, _ctx: &mut AudioContext, _volume: f32) {}
    }
}

#[cfg(any(not(feature = "audio"), target_os = "ios"))]
use dummy_audio::{AudioContext as QuadSndContext, Sound as QuadSndSound};

#[cfg(any(not(feature = "audio"), target_os = "ios"))]
pub struct PlaySoundParams {
    pub looped: bool,
    pub volume: f32,
}

pub struct AudioContext {
    native_ctx: QuadSndContext,
    sounds: HashMap<usize, QuadSndSound>,
    id: usize,
}

impl AudioContext {
    pub fn new() -> AudioContext {
        AudioContext {
            native_ctx: QuadSndContext::new(),
            sounds: HashMap::new(),
            id: 0,
        }
    }

    #[cfg(target_os = "android")]
    pub fn pause(&mut self) {
        self.native_ctx.pause()
    }

    #[cfg(target_os = "android")]
    pub fn resume(&mut self) {
        self.native_ctx.resume()
    }
}

impl Default for AudioContext {
    fn default() -> Self {
        AudioContext::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sound(usize);

pub struct Source {
    sound: Sound,
    params: PlaySoundParams,
}

impl Source {
    /// Load audio file.
    ///
    /// Attempts to automatically detect the format of the source of data.
    pub fn new(ctx: &mut Context, path: &str) -> GameResult<Source> {
        use std::io::Read;

        let mut file = filesystem::open(ctx, path)?;

        let mut bytes = vec![];
        file.bytes.read_to_end(&mut bytes)?;

        Self::from_bytes(ctx, bytes.as_slice())
    }

    /// Load audio from file.
    ///
    /// Attempts to automatically detect the format of the source of data.
    pub fn from_bytes(ctx: &mut Context, bytes: &[u8]) -> GameResult<Source> {
        let sound = QuadSndSound::load(&mut ctx.audio_context.native_ctx, bytes);

        // only on wasm the sound is not ready right away
        #[cfg(target_arch = "wasm32")]
        while sound.is_loaded() {
            std::thread::yield_now();
        }

        let id = ctx.audio_context.id;
        ctx.audio_context.sounds.insert(id, sound);
        ctx.audio_context.id += 1;
        Ok(Source {
            sound: Sound(id),
            params: PlaySoundParams::default(),
        })
    }

    pub fn play(&self, ctx: &mut Context) -> GameResult<()> {
        let ctx = &mut ctx.audio_context;
        let sound = &mut ctx.sounds.get_mut(&self.sound.0).unwrap();

        let params = PlaySoundParams {
            looped: self.params.looped,
            volume: self.params.volume,
        };
        sound.play(&mut ctx.native_ctx, params);
        Ok(())
    }

    pub fn stop(&self, ctx: &mut Context) -> GameResult {
        let ctx = &mut ctx.audio_context;
        let sound = &mut ctx.sounds.get_mut(&self.sound.0).unwrap();

        sound.stop(&mut ctx.native_ctx);
        Ok(())
    }

    pub fn set_volume(&mut self, ctx: &mut Context, volume: f32) -> GameResult<()> {
        let ctx = &mut ctx.audio_context;
        self.params.volume = volume;
        let sound = &mut ctx.sounds.get_mut(&self.sound.0).unwrap();

        sound.set_volume(&mut ctx.native_ctx, volume);
        Ok(())
    }

    pub fn volume(&self) -> f32 {
        self.params.volume
    }

    pub fn set_repeat(&mut self, repeat: bool) {
        self.params.looped = repeat;
    }

    pub fn repeat(&self) -> bool {
        self.params.looped
    }
}
