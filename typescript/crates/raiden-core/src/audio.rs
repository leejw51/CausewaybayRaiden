//! What the frame wants to hear.
//!
//! The synth lives in the browser (`src/audio/chip.ts`) because Web Audio is
//! the only sound hardware here; the simulation just names cues. They queue up
//! during a frame and the shell drains them as one byte array.
//!
//! Music is level-triggered rather than edge-triggered: asking for a track
//! that is already playing is a no-op on the far side, so a state that runs
//! for many frames can name its track every frame without restarting it.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Sfx {
    Shot = 1,
    Shot2,
    Hit,
    Coin,
    Start,
    Power,
    OneUp,
    Bomb,
    Explode,
    ExplodeBig,
    Death,
    Warn,
    Warn2,
    Blip,
    Graze,
    Select,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Music {
    Title = 20,
    Stage,
    Boss,
    Over,
    /// Silence, and forget what was playing.
    Stop,
}

#[derive(Default)]
pub struct Cues {
    queue: Vec<u8>,
}

impl Cues {
    pub fn sfx(&mut self, s: Sfx) {
        // A frame that fires twenty shots does not need twenty identical
        // clips stacked on top of each other; one is what LÖVE's reused
        // source did too.
        let id = s as u8;
        if !self.queue.contains(&id) {
            self.queue.push(id);
        }
    }

    pub fn music(&mut self, m: Music) {
        let id = m as u8;
        if !self.queue.contains(&id) {
            self.queue.push(id);
        }
    }

    pub fn drain(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.queue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeats_within_a_frame_collapse() {
        let mut c = Cues::default();
        for _ in 0..10 {
            c.sfx(Sfx::Shot);
        }
        c.sfx(Sfx::Hit);
        assert_eq!(c.drain(), vec![Sfx::Shot as u8, Sfx::Hit as u8]);
    }

    #[test]
    fn draining_empties_the_queue() {
        let mut c = Cues::default();
        c.sfx(Sfx::Bomb);
        assert!(!c.drain().is_empty());
        assert!(c.drain().is_empty());
    }

    #[test]
    fn sfx_and_music_ids_never_collide() {
        let sfx = [
            Sfx::Shot as u8,
            Sfx::Shot2 as u8,
            Sfx::Hit as u8,
            Sfx::Coin as u8,
            Sfx::Start as u8,
            Sfx::Power as u8,
            Sfx::OneUp as u8,
            Sfx::Bomb as u8,
            Sfx::Explode as u8,
            Sfx::ExplodeBig as u8,
            Sfx::Death as u8,
            Sfx::Warn as u8,
            Sfx::Warn2 as u8,
            Sfx::Blip as u8,
            Sfx::Graze as u8,
            Sfx::Select as u8,
        ];
        let music = [
            Music::Title as u8,
            Music::Stage as u8,
            Music::Boss as u8,
            Music::Over as u8,
            Music::Stop as u8,
        ];
        for m in music {
            assert!(!sfx.contains(&m), "id {m} means two different things");
        }
    }
}
