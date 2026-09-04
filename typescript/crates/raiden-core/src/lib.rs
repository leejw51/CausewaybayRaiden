//! CAUSEWAYBAY RAIDEN, as a wasm module.
//!
//! Everything that decides what happens lives in here; the TypeScript shell
//! owns only the things a browser will not delegate — a canvas, a keyboard,
//! Web Audio, and local storage.
//!
//! The frame crosses the boundary as two views into wasm memory (a command
//! list and a pool of strings) rather than as a stream of calls, so a busy
//! frame with a thousand sparks in it still costs one read.

pub mod app;
pub mod audio;
pub mod balance;
pub mod defs;
pub mod draw;
pub mod font;
pub mod palette;
pub mod rng;
pub mod sprites;
pub mod stage;
pub mod world;

use wasm_bindgen::prelude::*;

use app::{App, State};

#[wasm_bindgen]
pub struct Game {
    app: App,
    /// Held between calls so the JavaScript side can read it as a view.
    audio: Vec<u8>,
}

#[wasm_bindgen]
impl Game {
    /// `seed` comes from the shell so a reload is a different run, and a test
    /// harness can pin it to get the same one back.
    #[wasm_bindgen(constructor)]
    pub fn new(seed: f64) -> Game {
        Game { app: App::new(seed.abs() as u64), audio: Vec::new() }
    }

    /// The visible span, in playfield pixels. On a window wider than 3:4 this
    /// reaches past both edges of the 192-wide playfield and the ship may fly
    /// out there; on a narrower one the sides are cropped instead.
    pub fn set_view(&mut self, left: f32, right: f32) {
        self.app.set_view(left, right);
    }

    /// Restore saved progress. `cleared` is one bit per stage.
    pub fn set_progress(&mut self, hiscore: f64, cursor: u32, cleared: u32) {
        self.app.set_progress(hiscore as i64, cursor as usize, cleared);
    }

    /// Advance one frame and lay out its draw list. `bits` is the input word
    /// built by `src/engine/input.ts`.
    pub fn update(&mut self, dt: f32, bits: u32) {
        self.app.update(dt, bits);
        self.audio = self.app.drain_audio();
    }

    /// Pointer to this frame's draw list. Valid until the next `update`, and
    /// only alongside the matching `commands_len`.
    pub fn commands_ptr(&self) -> *const f32 {
        self.app.commands().as_ptr()
    }

    pub fn commands_len(&self) -> usize {
        self.app.commands().len()
    }

    /// This frame's strings, newline separated; text commands index the lines.
    pub fn text_pool(&self) -> String {
        self.app.text_pool()
    }

    /// Sound cues raised by the frame just simulated.
    pub fn audio_ptr(&self) -> *const u8 {
        self.audio.as_ptr()
    }

    pub fn audio_len(&self) -> usize {
        self.audio.len()
    }

    /// Screen shake, in playfield pixels.
    pub fn shake(&self) -> f32 {
        self.app.shake()
    }

    /// Whole-screen white-out, 0..1.
    pub fn flash(&self) -> f32 {
        self.app.flash()
    }

    /// Which screen is up, as an index into the `State` order.
    pub fn state(&self) -> u32 {
        match self.app.state {
            State::Boot => 0,
            State::Title => 1,
            State::Story => 2,
            State::Map => 3,
            State::Rank => 4,
            State::Play => 5,
            State::Pause => 6,
            State::Continue => 7,
            State::GameOver => 8,
            State::Ending => 9,
        }
    }

    pub fn hiscore(&self) -> f64 {
        self.app.hiscore as f64
    }

    pub fn map_cursor(&self) -> u32 {
        self.app.map_cursor as u32
    }

    pub fn cleared(&self) -> u32 {
        self.app.cleared
    }

    /// Moves whenever there is something new worth saving.
    pub fn progress_version(&self) -> u32 {
        self.app.progress_version()
    }

    pub fn score(&self) -> f64 {
        self.app.world.as_ref().map_or(0.0, |w| w.score as f64)
    }
}

/// The characters the built-in font can draw, in bitmap order.
#[wasm_bindgen]
pub fn font_charset() -> String {
    font::CHARSET.to_string()
}

/// Eight bytes per character, bit 7 leftmost — enough for the shell to bake a
/// font atlas once and then draw text with one blit per glyph.
#[wasm_bindgen]
pub fn font_bitmap() -> Vec<u8> {
    font::bitmap().to_vec()
}

/// How many pieces of art the draw list can refer to.
#[wasm_bindgen]
pub fn sprite_count() -> usize {
    sprites::SPRITES.len()
}

/// The art names, newline separated, in draw-list index order. The shell also
/// gets this from `public/art/manifest.json`; having it here too lets the
/// loader check the two agree before a mismatch turns into wrong sprites.
#[wasm_bindgen]
pub fn sprite_names() -> String {
    sprites::SPRITES.iter().map(|s| s.0).collect::<Vec<_>>().join("\n")
}
