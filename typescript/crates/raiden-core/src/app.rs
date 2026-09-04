//! The cabinet around the game: boot, title, story, map, rank, play, pause,
//! continue, game over, ending.
//!
//! A port of `love2d/main.lua`. Two things moved deliberately:
//!
//!   * LÖVE drew the score and lives in *window* pixels so they stayed small
//!     on a wide screen. Here the whole frame is already at a uniform pixel
//!     scale, so the HUD is drawn in playfield coordinates anchored to the
//!     visible edges, which comes out the same size and keeps one coordinate
//!     system for everything.
//!   * There is no fullscreen button. The browser has its own gesture for
//!     that, and the shell wires it up.
//!
//! Progress (high score, which stages are cleared, where the walker stands)
//! belongs to the shell, which keeps it in local storage. It is handed in at
//! boot and read back whenever [`App::progress_version`] moves.

use crate::audio::{Cues, Music, Sfx};
use crate::balance::{Rank, RANKS};
use crate::defs::SKILLS;
use crate::draw::{Painter, GH, GW};
use crate::font;
use crate::palette::{self, Col};
use crate::sprites::Sprite;
use crate::world::{ease_cos, Pad, World};

// Held this frame.
pub const B_LEFT: u32 = 1 << 0;
pub const B_RIGHT: u32 = 1 << 1;
pub const B_UP: u32 = 1 << 2;
pub const B_DOWN: u32 = 1 << 3;
pub const B_SHOOT: u32 = 1 << 4;
// Pressed this frame — edges, so one press is one action.
pub const H_LEFT: u32 = 1 << 8;
pub const H_RIGHT: u32 = 1 << 9;
pub const H_UP: u32 = 1 << 10;
pub const H_DOWN: u32 = 1 << 11;
pub const H_CONFIRM: u32 = 1 << 12;
pub const H_BACK: u32 = 1 << 13;
pub const H_BOMB: u32 = 1 << 14;
pub const H_COIN: u32 = 1 << 15;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum State {
    Boot,
    Title,
    Story,
    Map,
    Rank,
    Play,
    Pause,
    Continue,
    GameOver,
    Ending,
}

struct MapNode {
    x: f32,
    y: f32,
    tag: &'static str,
    title: &'static str,
    boss: &'static str,
}

#[rustfmt::skip]
const MAP: [MapNode; 3] = [
    MapNode { x: 36.0, y: 176.0, tag: "1-1", title: "CAUSEWAYBAY", boss: "OVERFLOW" },
    MapNode { x: 96.0, y: 124.0, tag: "1-2", title: "MTR LINE", boss: "DEADLOCK" },
    MapNode { x: 156.0, y: 72.0, tag: "1-3", title: "HKU CASTLE", boss: "SEGFAULT" },
];

struct StoryPage {
    art: Sprite,
    lines: [&'static str; 4],
}

#[rustfmt::skip]
const STORY: [StoryPage; 3] = [
    StoryPage {
        art: Sprite::StoryApt,
        lines: ["CAUSEWAYBAY 3AM", "TINY RUST APT", "ONE BUG WONT DIE", "NEED A BURGER"],
    },
    StoryPage {
        art: Sprite::StoryMtr,
        lines: ["TAKE THE MTR", "BUGS ON THE TRAIN", "DEBUG ON THE WAY", "NEXT STOP HKU"],
    },
    StoryPage {
        art: Sprite::StoryHku,
        lines: ["HKU BURGER SHOP", "ALMOST THERE", "CLEAR THE BUGS", "THEN WE EAT"],
    },
];

const STORY_DWELL: f32 = 3.4;
const STORY_SLIDE: f32 = 0.7;

struct AttractBug {
    x: f32,
    y: f32,
    sp: f32,
}

pub struct App {
    pub state: State,
    state_t: f32,
    blink: f32,
    pub credits: i32,
    continue_n: f32,
    pub hiscore: i64,
    /// One bit per stage, set when it has been cleared at least once.
    pub cleared: u32,
    pub map_cursor: usize,
    rank_cursor: usize,
    progress_version: u32,

    title_scroll: f32,
    attract: Vec<AttractBug>,
    story_page: usize,
    story_pos: f32,
    story_from: f32,
    story_to: f32,
    story_anim: f32,
    story_auto: bool,
    story_hold: f32,
    map_px: f32,
    map_py: f32,

    /// The attract-mode game that plays itself while nobody is at the cabinet.
    demo: bool,
    pub world: Option<World>,
    cues: Cues,
    painter: Painter,
    seed: u64,
    view_l: f32,
    view_r: f32,
}

impl App {
    pub fn new(seed: u64) -> App {
        let attract = (1..=8)
            .map(|i| AttractBug {
                x: 20.0 + i as f32 * 20.0,
                y: -20.0 - i as f32 * 30.0,
                sp: 30.0 + i as f32 * 8.0,
            })
            .collect();
        let mut app = App {
            state: State::Boot,
            state_t: 0.0,
            blink: 0.0,
            credits: 1,
            continue_n: 9.0,
            hiscore: 50_000,
            cleared: 0,
            map_cursor: 0,
            rank_cursor: crate::balance::DEFAULT.index(),
            progress_version: 0,
            title_scroll: 0.0,
            attract,
            story_page: 0,
            story_pos: 0.0,
            story_from: 0.0,
            story_to: 0.0,
            story_anim: 1.0,
            story_auto: true,
            story_hold: 0.0,
            map_px: MAP[0].x,
            map_py: MAP[0].y,
            demo: false,
            world: None,
            cues: Cues::default(),
            painter: Painter::default(),
            seed,
            view_l: 0.0,
            view_r: GW,
        };
        app.cues.music(Music::Title);
        app
    }

    /// Restore what the shell had in local storage.
    pub fn set_progress(&mut self, hiscore: i64, cursor: usize, cleared: u32) {
        self.hiscore = self.hiscore.max(hiscore);
        self.map_cursor = cursor.min(MAP.len() - 1);
        self.cleared = cleared;
        self.map_px = MAP[self.map_cursor].x;
        self.map_py = MAP[self.map_cursor].y;
    }

    /// Bumped whenever something worth writing to storage changed.
    pub fn progress_version(&self) -> u32 {
        self.progress_version
    }

    fn touch_progress(&mut self) {
        self.progress_version = self.progress_version.wrapping_add(1);
    }

    pub fn set_view(&mut self, left: f32, right: f32) {
        self.view_l = left;
        self.view_r = right;
        self.painter.set_view(left, right);
        if let Some(w) = self.world.as_mut() {
            w.set_view(left, right);
        }
    }

    pub fn shake(&self) -> f32 {
        match (self.state, &self.world) {
            (State::Play | State::Pause | State::Continue | State::GameOver, Some(w)) => w.shake,
            _ => 0.0,
        }
    }

    /// A white-out level for the shell's screen flash, 0..1.
    pub fn flash(&self) -> f32 {
        self.world.as_ref().map_or(0.0, |w| w.flash)
    }

    pub fn commands(&self) -> &[f32] {
        self.painter.commands()
    }

    pub fn text_pool(&self) -> String {
        self.painter.text_pool()
    }

    pub fn drain_audio(&mut self) -> Vec<u8> {
        let mut out = self.cues.drain();
        if let Some(w) = self.world.as_mut() {
            out.extend(w.cues.drain());
        }
        out
    }

    // ------------------------------------------------------------ transitions

    fn start_story(&mut self) {
        self.story_page = 0;
        self.story_pos = 0.0;
        self.story_from = 0.0;
        self.story_to = 0.0;
        self.story_anim = 1.0;
        self.story_auto = true;
        self.story_hold = 0.0;
        self.state = State::Story;
        self.state_t = 0.0;
        self.cues.sfx(Sfx::Select);
    }

    fn story_goto(&mut self, page: usize, keep_auto: bool) {
        let page = page.min(STORY.len() - 1);
        if page == self.story_page && self.story_anim >= 1.0 {
            return;
        }
        self.story_from = self.story_pos;
        self.story_to = page as f32;
        self.story_page = page;
        self.story_anim = 0.0;
        self.story_hold = 0.0;
        if !keep_auto {
            self.story_auto = false;
        }
        self.cues.sfx(Sfx::Select);
    }

    fn start_map(&mut self) {
        self.state = State::Map;
        self.state_t = 0.0;
        self.map_px = MAP[self.map_cursor].x;
        self.map_py = MAP[self.map_cursor].y;
        self.cues.sfx(Sfx::Select);
        self.cues.music(Music::Title);
    }

    fn start_rank(&mut self) {
        self.state = State::Rank;
        self.state_t = 0.0;
        self.cues.sfx(Sfx::Select);
    }

    fn start_game(&mut self) {
        self.demo = false;
        let stage = self.map_cursor as i32 + 1;
        let rank = RANKS[self.rank_cursor];
        self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let mut w = World::new(self.seed, self.hiscore, stage, rank);
        w.set_view(self.view_l, self.view_r);
        self.world = Some(w);
        self.state = State::Play;
        self.state_t = 0.0;
        self.cues.sfx(Sfx::Start);
        self.cues.music(Music::Stage);
    }

    fn start_demo(&mut self) {
        self.demo = true;
        self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let mut w = World::new(self.seed, self.hiscore, 1, Rank::Easy);
        w.set_view(self.view_l, self.view_r);
        // The demo must not end on a life counter, only on its own clock.
        w.lives = 99;
        w.ready_t = 0.0;
        self.world = Some(w);
        self.state = State::Play;
        self.state_t = 0.0;
        self.cues.music(Music::Stage);
    }

    fn next_stage(&mut self) {
        let (stage, score) = match &self.world {
            Some(w) => (w.stage, w.score),
            None => return,
        };
        self.cleared |= 1 << (stage - 1).max(0);
        self.hiscore = self.hiscore.max(score);
        self.touch_progress();
        if stage >= 3 {
            self.state = State::Ending;
            self.state_t = 0.0;
            self.cues.music(Music::Title);
        } else {
            self.map_cursor = (stage as usize).min(MAP.len() - 1);
            self.start_map();
        }
    }

    fn back_to_title(&mut self) {
        self.state = State::Title;
        self.state_t = 0.0;
        self.cues.music(Music::Title);
    }

    fn bank_hiscore(&mut self) {
        if let Some(w) = &self.world {
            if w.hiscore > self.hiscore {
                self.hiscore = w.hiscore;
                self.touch_progress();
            }
        }
    }

    // ----------------------------------------------------------------- update

    pub fn update(&mut self, dt: f32, bits: u32) {
        let dt = dt.clamp(0.0, 0.05);
        self.state_t += dt;
        self.blink += dt;

        if bits & H_COIN != 0 {
            self.credits = (self.credits + 1).min(9);
            self.cues.sfx(Sfx::Coin);
        }

        match self.state {
            State::Boot => self.update_boot(dt, bits),
            State::Title => self.update_title(dt, bits),
            State::Story => self.update_story(dt, bits),
            State::Map => self.update_map(dt, bits),
            State::Rank => self.update_rank(bits),
            State::Play => self.update_play(dt, bits),
            State::Pause => self.update_pause(bits),
            State::Continue => self.update_continue(dt, bits),
            State::GameOver => self.update_gameover(dt, bits),
            State::Ending => self.update_ending(dt, bits),
        }

        self.draw();
    }

    fn insert_credit(&mut self) -> bool {
        if self.credits > 0 {
            self.credits -= 1;
            true
        } else {
            self.cues.sfx(Sfx::Blip);
            false
        }
    }

    fn update_boot(&mut self, dt: f32, bits: u32) {
        self.title_scroll += 10.0 * dt;
        // The logos are skippable; nobody watches them twice.
        if self.state_t > 6.2 || bits & (H_CONFIRM | H_BACK) != 0 {
            self.state = State::Title;
            self.state_t = 0.0;
        }
    }

    fn update_title(&mut self, dt: f32, bits: u32) {
        self.title_scroll += 18.0 * dt;
        for b in self.attract.iter_mut() {
            b.y += b.sp * dt;
            // Kept inside the hero art, never over the menu below it.
            if b.y > 96.0 {
                b.y = -24.0;
                b.x = 28.0 + (b.sp * 7.3).fract() * 136.0;
            }
        }
        if bits & H_CONFIRM != 0 && self.credits > 0 {
            self.start_story();
            return;
        }
        if bits & H_CONFIRM != 0 {
            self.cues.sfx(Sfx::Blip);
        }
        if self.state_t > 18.0 {
            self.start_demo();
        }
    }

    fn update_story(&mut self, dt: f32, bits: u32) {
        self.title_scroll += 12.0 * dt;
        let last = STORY.len() - 1;
        if bits & H_LEFT != 0 {
            if self.story_page > 0 {
                self.story_goto(self.story_page - 1, false);
            } else {
                self.story_auto = false;
            }
        } else if bits & H_RIGHT != 0 {
            if self.story_page < last {
                self.story_goto(self.story_page + 1, false);
            } else {
                self.story_auto = false;
                self.start_map();
                return;
            }
        } else if bits & (H_CONFIRM | H_BACK) != 0 {
            self.start_map();
            return;
        }

        if self.story_anim < 1.0 {
            self.story_anim = (self.story_anim + dt / STORY_SLIDE).min(1.0);
            self.story_pos =
                self.story_from + (self.story_to - self.story_from) * ease_cos(self.story_anim);
            return;
        }
        self.story_pos = self.story_to;
        if !self.story_auto {
            return;
        }
        self.story_hold += dt;
        if self.story_hold >= STORY_DWELL {
            self.story_hold = 0.0;
            if self.story_page < last {
                self.story_goto(self.story_page + 1, true);
            } else {
                self.start_map();
            }
        }
    }

    fn update_map(&mut self, dt: f32, bits: u32) {
        let last = MAP.len() - 1;
        if bits & H_LEFT != 0 {
            self.map_cursor = if self.map_cursor == 0 { last } else { self.map_cursor - 1 };
            self.touch_progress();
            self.cues.sfx(Sfx::Select);
        } else if bits & H_RIGHT != 0 {
            self.map_cursor = if self.map_cursor >= last { 0 } else { self.map_cursor + 1 };
            self.touch_progress();
            self.cues.sfx(Sfx::Select);
        } else if bits & H_CONFIRM != 0 {
            self.start_rank();
            return;
        } else if bits & H_BACK != 0 {
            self.back_to_title();
            return;
        }
        // The walker eases over to the chosen node rather than snapping.
        let n = &MAP[self.map_cursor];
        let k = (dt * 8.0).min(1.0);
        self.map_px += (n.x - self.map_px) * k;
        self.map_py += (n.y - self.map_py) * k;
    }

    fn update_rank(&mut self, bits: u32) {
        let last = RANKS.len() - 1;
        if bits & (H_UP | H_LEFT) != 0 {
            self.rank_cursor = if self.rank_cursor == 0 { last } else { self.rank_cursor - 1 };
            self.cues.sfx(Sfx::Select);
        } else if bits & (H_DOWN | H_RIGHT) != 0 {
            self.rank_cursor = if self.rank_cursor >= last { 0 } else { self.rank_cursor + 1 };
            self.cues.sfx(Sfx::Select);
        } else if bits & H_CONFIRM != 0 {
            if self.insert_credit() {
                self.start_game();
            }
        } else if bits & H_BACK != 0 {
            self.start_map();
        }
    }

    fn update_play(&mut self, dt: f32, bits: u32) {
        if bits & H_BACK != 0 {
            if self.demo {
                self.back_to_title();
            } else {
                self.state = State::Pause;
                self.cues.sfx(Sfx::Select);
            }
            return;
        }
        if self.demo && bits & H_CONFIRM != 0 {
            if self.credits > 0 {
                self.credits -= 1;
                self.start_game();
            } else {
                self.back_to_title();
            }
            return;
        }

        let pad = if self.demo {
            self.demo_pad()
        } else {
            Pad {
                left: bits & B_LEFT != 0,
                right: bits & B_RIGHT != 0,
                up: bits & B_UP != 0,
                down: bits & B_DOWN != 0,
                shoot: bits & B_SHOOT != 0,
                bomb: bits & H_BOMB != 0,
            }
        };
        let Some(w) = self.world.as_mut() else {
            self.back_to_title();
            return;
        };
        w.update(dt, pad);
        let (clear, clear_t, over) = (w.clear, w.clear_t, w.over);

        if clear && clear_t <= 0.0 {
            self.next_stage();
            return;
        }
        if over {
            if self.demo {
                self.back_to_title();
            } else {
                self.bank_hiscore();
                self.state = State::Continue;
                self.continue_n = 9.9;
                self.cues.music(Music::Stop);
                self.cues.sfx(Sfx::Death);
            }
            return;
        }
        if self.demo && self.state_t > 25.0 {
            self.back_to_title();
        }
    }

    /// The attract ship, flown by two sine waves.
    fn demo_pad(&self) -> Pad {
        let t = self.world.as_ref().map_or(0.0, |w| w.time);
        Pad {
            left: (t * 1.3).sin() < -0.3,
            right: (t * 1.3).sin() > 0.3,
            up: (t * 0.7).sin() > 0.4,
            down: (t * 0.7).sin() < -0.2,
            shoot: true,
            bomb: false,
        }
    }

    fn update_pause(&mut self, bits: u32) {
        if bits & (H_BACK | H_CONFIRM) != 0 {
            self.state = State::Play;
            self.cues.sfx(Sfx::Select);
        }
    }

    fn update_continue(&mut self, dt: f32, bits: u32) {
        if bits & H_CONFIRM != 0 && self.credits > 0 {
            self.credits -= 1;
            let boss_fight = self.world.as_ref().is_some_and(|w| w.boss().is_some());
            if let Some(w) = self.world.as_mut() {
                w.revive();
            }
            self.state = State::Play;
            self.cues.sfx(Sfx::Start);
            self.cues.music(if boss_fight { Music::Boss } else { Music::Stage });
            return;
        }
        if bits & H_BACK != 0 {
            self.bank_hiscore();
            self.start_map();
            return;
        }
        self.continue_n -= dt;
        if self.continue_n <= 0.0 {
            self.bank_hiscore();
            self.state = State::GameOver;
            self.state_t = 0.0;
            self.cues.music(Music::Over);
        }
    }

    fn update_gameover(&mut self, _dt: f32, bits: u32) {
        if bits & (H_CONFIRM | H_BACK) != 0 || self.state_t > 8.0 {
            self.start_map();
        }
    }

    fn update_ending(&mut self, _dt: f32, bits: u32) {
        if bits & (H_CONFIRM | H_BACK) != 0 || self.state_t > 16.0 {
            self.back_to_title();
        }
    }

    // ------------------------------------------------------------------- draw

    fn draw(&mut self) {
        self.painter.begin();
        self.draw_parallax();

        match self.state {
            State::Boot => self.draw_boot(),
            State::Title => self.draw_title(),
            State::Story => self.draw_story(),
            State::Map => self.draw_map(),
            State::Rank => self.draw_rank(),
            State::Ending => self.draw_ending(),
            State::Play | State::Pause | State::Continue | State::GameOver => {
                if let Some(w) = &self.world {
                    w.draw(&mut self.painter);
                }
                if self.demo
                    && self.state == State::Play
                    && (self.blink * 2.0).floor() as i32 % 2 == 0
                {
                    self.painter.center("DEMO", 88.0, palette::YELLOW, 1.0);
                    self.painter.center("SPACE START", 200.0, palette::WHITE, 1.0);
                }
                self.draw_play_hud();
                match self.state {
                    State::Pause => self.draw_pause(),
                    State::Continue => self.draw_continue(),
                    State::GameOver => self.draw_gameover(),
                    _ => {}
                }
            }
        }
        self.draw_menu_hud();
    }

    /// Raiden-style vertical parallax: far plate, ground, haze, clouds.
    fn draw_parallax(&mut self) {
        let in_play =
            matches!(self.state, State::Play | State::Pause | State::Continue | State::GameOver);
        let scroll = match (&self.world, in_play) {
            (Some(w), true) => w.bg_y,
            _ => self.title_scroll,
        };
        let st = match self.state {
            State::Story => self.story_page as i32 + 1,
            State::Map | State::Rank => self.map_cursor as i32 + 1,
            State::Ending => 3,
            _ => self.world.as_ref().map_or(1, |w| w.stage),
        };
        let (ground, far) = match st {
            2 => (Sprite::BgMtr, Sprite::BgMtrFar),
            3 => (Sprite::BgHku, Sprite::BgHkuFar),
            _ => (Sprite::BgApt, Sprite::BgAptFar),
        };
        // Causeway Bay at street level wants a clear sky; everywhere else the
        // haze and clouds sit heavier.
        let (haze_a, cloud_a) = if st == 1 && in_play { (0.14, 0.14) } else { (0.34, 0.48) };

        let layer = |p: &mut Painter, art: Sprite, mul: f32, col: Col| {
            let v0 = (-(scroll * mul)).rem_euclid(GH).floor();
            p.tiled(art, -v0, col);
        };
        layer(&mut self.painter, far, 0.32, [0.62, 0.66, 0.82, 1.0]);
        layer(&mut self.painter, ground, 1.0, palette::WHITE);
        layer(&mut self.painter, Sprite::Haze, 0.42, palette::a(palette::WHITE, haze_a));
        layer(&mut self.painter, Sprite::Clouds, 1.68, palette::a(palette::WHITE, cloud_a));
    }

    /// A fade that rises, holds, and falls again inside `t0..t1`.
    fn boot_fade(t: f32, t0: f32, t1: f32) -> f32 {
        const FADE: f32 = 0.85;
        if t < t0 || t > t1 {
            return 0.0;
        }
        let u = t - t0;
        let dur = t1 - t0;
        if u < FADE {
            return ease_cos(u / FADE);
        }
        if u > dur - FADE {
            return ease_cos((dur - u) / FADE);
        }
        1.0
    }

    fn draw_boot(&mut self) {
        let t = self.state_t;
        let a1 = Self::boot_fade(t, 0.05, 2.55);
        let a2 = Self::boot_fade(t, 2.5, 6.15);
        let (x, w) = self.painter.view_span();
        self.painter.rect(x, 0.0, w, GH, [0.0, 0.0, 0.0, 0.92]);
        if a1 > 0.02 {
            self.painter.center("HONG KONG", 96.0, palette::a(palette::CYAN, a1), 1.0);
            self.painter.center("NIGHT 2026", 118.0, palette::a(palette::YELLOW, a1), 1.0);
        }
        if a2 > 0.02 {
            self.painter.center("CAUSEWAYBAY", 88.0, palette::a(palette::RUST, a2), 1.0);
            self.painter.center("AI", 110.0, palette::a(palette::YELLOW, a2), 2.0);
            self.painter.center("PRESENTS", 144.0, palette::a(palette::CYAN, a2), 1.0);
            self.painter.center("CAUSEWAYBAY", 168.0, palette::a(palette::WHITE, a2), 1.0);
            self.painter.center("RAIDEN", 182.0, palette::a(palette::WHITE, a2), 1.0);
        }
    }

    fn draw_title(&mut self) {
        // Bugs behind the art and the text, so they never cover the menu.
        for i in 0..self.attract.len() {
            let (x, y) = (self.attract[i].x, self.attract[i].y);
            self.painter.sprite(
                Sprite::Beetle,
                x,
                y,
                0.0,
                1.0,
                1.0,
                13.0,
                13.0,
                palette::a(palette::WHITE, 0.9),
            );
        }
        self.painter.blit(Sprite::Title, 0.0, 0.0, palette::WHITE);
        let (x, w) = self.painter.view_span();
        self.painter.rect(x, 104.0, w, 152.0, [0.03, 0.02, 0.10, 0.78]);

        let flash = (self.blink * 8.0).floor() as i32 % 2 == 0;
        let hot = if flash { palette::YELLOW } else { palette::RUST };
        self.painter.center("CAUSEWAYBAY", 108.0, hot, 1.0);
        self.painter.center("RAIDEN", 122.0, palette::CYAN, 2.0);
        let msg = if self.credits > 0 { "SPACE START" } else { "C  INSERT COIN" };
        let msg_col = if flash { palette::YELLOW } else { palette::WHITE };
        self.painter.center(msg, 150.0, msg_col, 1.0);
        self.painter.center("Z SHOT   X BOMB", 166.0, palette::CYAN, 1.0);
        self.painter.center("ARROWS MOVE", 180.0, palette::WHITE, 1.0);
        self.painter.center("P PAUSE  F FULL", 194.0, palette::MAGENTA, 1.0);
    }

    fn draw_story(&mut self) {
        // The art slides page by page; the text band is drawn once across the
        // whole visible width, so a wide window is never double-dimmed where
        // two pages overlap mid-slide.
        for (i, page) in STORY.iter().enumerate() {
            let ox = (i as f32 - self.story_pos) * GW;
            if ox.abs() < GW {
                self.painter.blit(page.art, ox, 0.0, palette::WHITE);
            }
        }
        let (x, w) = self.painter.view_span();
        self.painter.rect(x, 138.0, w, 118.0, [0.03, 0.02, 0.10, 0.78]);
        // Only the page the slide is closest to gets its words. The art may
        // show two at once — that is the slide — but a wide window is wide
        // enough to show the neighbour's text too, and two captions at once
        // read as a mistake rather than as motion.
        let near = self.story_pos.round().clamp(0.0, (STORY.len() - 1) as f32);
        let page = &STORY[near as usize];
        let ox = (near - self.story_pos) * GW;
        let mut y = 158.0;
        for line in page.lines {
            // Centred on this page's slot, not on the screen.
            let tw = font::text_width(line, 1.0);
            self.painter.text_shadow(line, ox + ((GW - tw) / 2.0).floor(), y, palette::WHITE, 1.0);
            y += 14.0;
        }
        let label = format!("STORY {}/{}", self.story_page + 1, STORY.len());
        self.painter.center(&label, 142.0, palette::CYAN, 1.0);
        let flash = (self.blink * 4.0).floor() as i32 % 2 == 0;
        let col = if flash { palette::YELLOW } else { palette::WHITE };
        self.painter.center("SPACE FOR MAP", 216.0, col, 1.0);
        self.painter.center("LEFT RIGHT PAGE", 230.0, palette::WHITE, 1.0);
    }

    fn draw_map(&mut self) {
        self.painter.blit(Sprite::WorldMap, 0.0, 0.0, palette::WHITE);
        let (x, w) = self.painter.view_span();
        self.painter.rect(x, 0.0, w, 28.0, [0.03, 0.02, 0.10, 0.55]);
        self.painter.rect(x, 214.0, w, 42.0, [0.03, 0.02, 0.10, 0.55]);
        self.painter.center("WORLD 1 HONG KONG", 8.0, palette::YELLOW, 1.0);

        // A dotted road between the nodes.
        for i in 0..MAP.len() - 1 {
            let (a, b) = (&MAP[i], &MAP[i + 1]);
            for s in 1..=8 {
                let u = s as f32 / 9.0;
                let px = a.x + (b.x - a.x) * u;
                let py = a.y + (b.y - a.y) * u;
                self.painter.rect(
                    px.floor() - 1.0,
                    py.floor() - 1.0,
                    2.0,
                    2.0,
                    [1.0, 0.85, 0.35, 0.9],
                );
            }
        }

        for (i, n) in MAP.iter().enumerate() {
            let on = i == self.map_cursor;
            let cleared = self.cleared & (1 << i) != 0;
            let r = if on { 9.0 } else { 6.0 };
            self.painter.circle(n.x + 1.0, n.y + 2.0, r + 1.0, palette::a(palette::BLACK, 0.55));
            let fill = if cleared {
                [0.92, 0.82, 0.35, 1.0]
            } else if on {
                palette::CYAN
            } else {
                palette::WHITE
            };
            self.painter.circle(n.x, n.y, r, fill);
            self.painter.ring(n.x, n.y, r, 1.0, palette::NAVY);
            let tag_col = if on { palette::YELLOW } else { palette::WHITE };
            self.painter.text(n.tag, n.x - 12.0, n.y - 18.0, tag_col, 1.0);
            if cleared {
                // A rubber stamp, squashed and tilted.
                self.painter.ellipse_fill(
                    n.x + 2.0,
                    n.y + 1.0,
                    16.5,
                    7.2,
                    -0.38,
                    [0.82, 0.14, 0.16, 0.28],
                );
                self.painter.ellipse_ring(
                    n.x + 2.0,
                    n.y + 1.0,
                    16.5,
                    7.2,
                    -0.38,
                    2.0,
                    [0.82, 0.14, 0.16, 0.95],
                );
                self.painter.text("CLR", n.x - 10.0, n.y - 3.0, [0.92, 0.18, 0.16, 1.0], 1.0);
            }
        }

        let bob = (self.blink * 8.0).sin() * 1.5;
        self.painter.sprite(
            Sprite::Player,
            self.map_px.floor(),
            (self.map_py + bob).floor() - 8.0,
            0.0,
            0.55,
            0.55,
            20.0,
            20.0,
            palette::WHITE,
        );

        let n = &MAP[self.map_cursor];
        self.painter.center(&format!("{}  {}", n.tag, n.title), 218.0, palette::CYAN, 1.0);
        self.painter.center(&format!("BOSS {}", n.boss), 232.0, palette::RUST, 1.0);
        let msg = if self.cleared & (1 << self.map_cursor) != 0 { "CLEARED" } else { "SPACE RANK" };
        self.painter.center(msg, 244.0, palette::YELLOW, 1.0);
    }

    fn draw_rank(&mut self) {
        self.painter.blit(Sprite::WorldMap, 0.0, 0.0, palette::a(palette::WHITE, 0.35));
        let (x, w) = self.painter.view_span();
        self.painter.rect(x, 0.0, w, GH, [0.03, 0.02, 0.10, 0.78]);
        self.painter.center("LEVEL", 36.0, palette::YELLOW, 2.0);
        self.painter.center("CHOOSE RANK", 62.0, palette::CYAN, 1.0);
        let n = &MAP[self.map_cursor];
        self.painter.center(&format!("{}  {}", n.tag, n.title), 78.0, palette::WHITE, 1.0);

        let flash = (self.blink * 8.0).floor() as i32 % 2 == 0;
        for (i, r) in RANKS.iter().enumerate() {
            let y = 102.0 + i as f32 * 32.0;
            if i == self.rank_cursor {
                self.painter.rect(24.0, y - 4.0, 144.0, 28.0, [0.86, 0.40, 0.27, 0.35]);
                let col = if flash { palette::YELLOW } else { palette::WHITE };
                self.painter.center(&format!("> {} <", r.title()), y, col, 1.0);
                self.painter.center(r.sub(), y + 12.0, palette::CYAN, 1.0);
            } else {
                self.painter.center(r.title(), y + 4.0, palette::GRAY, 1.0);
            }
        }
        let col = if flash { palette::YELLOW } else { palette::WHITE };
        self.painter.center("SPACE START", 214.0, col, 1.0);
        self.painter.center("ESC MAP", 230.0, palette::MAGENTA, 1.0);
    }

    fn draw_ending(&mut self) {
        self.painter.blit(Sprite::StoryEnd, 0.0, 0.0, palette::WHITE);
        let (x, w) = self.painter.view_span();
        self.painter.rect(x, 132.0, w, 124.0, [0.03, 0.02, 0.10, 0.7]);
        self.painter.center("THE END", 138.0, palette::YELLOW, 1.0);
        self.painter.center("PRINCESS PITCH", 154.0, palette::MAGENTA, 1.0);
        self.painter.center("HKU BURGER CLEAR", 168.0, palette::CYAN, 1.0);
        self.painter.sprite(
            Sprite::Princess,
            96.0,
            210.0,
            0.0,
            1.0,
            1.0,
            16.0,
            16.0,
            palette::WHITE,
        );
        if let Some(w) = &self.world {
            let score = format!("{:06}", w.score);
            self.painter.center(&score, 186.0, palette::WHITE, 1.0);
        }
        let flash = (self.blink * 4.0).floor() as i32 % 2 == 0;
        let col = if flash { palette::YELLOW } else { palette::WHITE };
        self.painter.center("SPACE TITLE", 240.0, col, 1.0);
    }

    fn draw_pause(&mut self) {
        let (x, w) = self.painter.view_span();
        self.painter.rect(x, 0.0, w, GH, palette::a(palette::BLACK, 0.6));
        self.painter.center("PAUSE", 72.0, palette::YELLOW, 2.0);
        self.painter.center("Z SHOT", 112.0, palette::CYAN, 1.0);
        self.painter.center("X BOMB", 126.0, palette::RUST, 1.0);
        self.painter.center("C  COIN", 140.0, palette::YELLOW, 1.0);
        self.painter.center("ARROWS MOVE", 154.0, palette::WHITE, 1.0);
        self.painter.center("P ESC RESUME", 176.0, palette::WHITE, 1.0);
        self.painter.center("F FULLSCREEN", 190.0, palette::MAGENTA, 1.0);
        if let Some(world) = &self.world {
            let label = world.rank().title();
            self.painter.center(label, 210.0, palette::CYAN, 1.0);
        }
    }

    fn draw_continue(&mut self) {
        let (x, w) = self.painter.view_span();
        self.painter.rect(x, 0.0, w, GH, palette::a(palette::BLACK, 0.6));
        self.painter.center("CONTINUE?", 90.0, palette::YELLOW, 2.0);
        let n = self.continue_n.ceil().max(0.0) as i32;
        self.painter.center(&n.to_string(), 118.0, palette::LRED, 3.0);
        self.painter.center(&format!("CREDIT {}", self.credits), 160.0, palette::CYAN, 1.0);
        if self.credits > 0 {
            self.painter.center("SPACE START", 178.0, palette::WHITE, 1.0);
        } else {
            self.painter.center("C  INSERT COIN", 178.0, palette::GRAY, 1.0);
        }
    }

    fn draw_gameover(&mut self) {
        let (x, w) = self.painter.view_span();
        self.painter.rect(x, 0.0, w, GH, palette::a(palette::BLACK, 0.65));
        self.painter.center("GAME OVER", 96.0, palette::LRED, 2.0);
        if let Some(world) = &self.world {
            let (score, graze, kills, agents) =
                (world.score, world.graze, world.kills, world.agents.len());
            self.painter.center(&format!("SCORE {score:06}"), 128.0, palette::YELLOW, 1.0);
            self.painter.center(&format!("GRAZE {graze}"), 144.0, palette::CYAN, 1.0);
            self.painter.center(&format!("KILLS {kills}"), 156.0, palette::WHITE, 1.0);
            self.painter.center(&format!("AGENTS {agents}"), 168.0, palette::MAGENTA, 1.0);
        }
        self.painter.center("THE BUGS WON", 188.0, palette::GRAY, 1.0);
    }

    /// Score, rank, lives, power and bombs, pinned to the visible edges.
    fn draw_play_hud(&mut self) {
        let Some(w) = &self.world else { return };
        let (score, hiscore, power, bombs, lives) = (w.score, w.hiscore, w.power, w.bombs, w.lives);
        let rank = w.rank().title();
        let l = self.painter.view_left() + 3.0;
        let r = self.painter.view_right() - 3.0;

        self.painter.text_shadow(&format!("{score:06}"), l, 0.0, palette::WHITE, 1.0);
        let hi = format!("{hiscore:06}");
        self.painter.text_shadow(&hi, r - font::text_width(&hi, 1.0), 0.0, palette::YELLOW, 1.0);
        let skill = SKILLS[(power.clamp(1, 4) - 1) as usize];
        self.painter.text_shadow(skill, l, 8.0, palette::CYAN, 1.0);
        self.painter.text_shadow(rank, r - font::text_width(rank, 1.0), 8.0, palette::RUST, 1.0);

        let bot = 242.0;
        for i in 0..lives.clamp(0, 5) {
            self.painter.sprite(
                Sprite::Player,
                l + 4.0 + i as f32 * 11.0,
                bot + 4.0,
                0.0,
                0.28,
                0.28,
                20.0,
                20.0,
                palette::WHITE,
            );
        }
        // Power meter, centred: four pips and a label.
        let (pip_w, pip_h, gap) = (6.0, 5.0, 3.0);
        let meter_w = 8.0 + 6.0 + 4.0 * (pip_w + gap);
        let mx = ((self.painter.view_left() + self.painter.view_right() - meter_w) / 2.0).floor();
        self.painter.text_shadow("P", mx, bot, palette::CYAN, 1.0);
        for i in 0..4 {
            let on = i < power;
            let col = if on { palette::CYAN } else { palette::NAVY };
            self.painter.rect(mx + 14.0 + i as f32 * (pip_w + gap), bot + 2.0, pip_w, pip_h, col);
        }
        for i in 1..=bombs.max(0) {
            let bx = r - i as f32 * (pip_w + 4.0);
            self.painter.rect(bx, bot + 2.0, pip_w, pip_w, palette::RUST);
            self.painter.rect(bx + 2.0, bot + 4.0, 2.0, 2.0, palette::YELLOW);
        }
    }

    /// The cabinet numbers, outside a game: best score and credits.
    ///
    /// Each menu draws its own instructions where they belong against its own
    /// art, so this deliberately adds no third line of text to collide with.
    fn draw_menu_hud(&mut self) {
        if !matches!(
            self.state,
            State::Title | State::Story | State::Map | State::Rank | State::Ending
        ) {
            return;
        }
        let l = self.painter.view_left() + 3.0;
        let r = self.painter.view_right() - 3.0;
        let hi = format!("{:06}", self.hiscore);
        self.painter.text_shadow(&hi, l, 0.0, palette::YELLOW, 1.0);
        let cred = format!("CREDIT {}", self.credits);
        self.painter.text_shadow(&cred, r - font::text_width(&cred, 1.0), 0.0, palette::CYAN, 1.0);
    }
}
