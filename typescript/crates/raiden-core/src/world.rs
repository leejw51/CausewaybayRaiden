//! The shmup itself: player, bugs, bullets, bombs, pickups, agents, particles.
//!
//! A port of `love2d/src/world.lua`, kept close enough to the original that a
//! constant can be looked up in either file and found in the other. The shape
//! of the code differs in two places where Lua's freedoms do not survive the
//! trip:
//!
//!   * enemies are addressed by index, not by reference, so an update can kill
//!     the thing it is updating;
//!   * the boss is remembered as an id plus a "did it die" flag, because Lua
//!     kept reading a table it had already removed from the list.
//!
//! Everything is in playfield pixels: 192 across, 256 down, y increasing
//! downward, which is the space the stage scripts and the art were authored in.

use crate::audio::{Cues, Music, Sfx};
use crate::balance::{Kit, Rank};
use crate::defs::{AgentDef, AgentId, Item, Kind, Path, AGENTS, SKILLS};
use crate::draw::{Painter, GH, GW};
use crate::font;
use crate::palette::{self, Col};
use crate::rng::Rng;
use crate::sprites::Sprite;
use crate::stage::{self, Act, Beat, Opts};

fn dist2(ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let (dx, dy) = (ax - bx, ay - by);
    dx * dx + dy * dy
}

/// A cosine ease over 0..1, clamped. Used for pops, fades and slides.
pub fn ease_cos(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    0.5 - 0.5 * (t * std::f32::consts::PI).cos()
}

// ------------------------------------------------------------------- entities

#[derive(Clone, Copy)]
pub struct Player {
    pub x: f32,
    pub y: f32,
    pub alive: bool,
    pub inv: f32,
    pub flash: f32,
    pub fire_t: f32,
    /// The death-bomb window: hit, but not gone yet.
    pub dying: bool,
    pub die_t: f32,
}

#[derive(Clone, Copy)]
pub struct Enemy {
    pub kind: Kind,
    pub x: f32,
    pub y: f32,
    pub x0: f32,
    pub y0: f32,
    pub hp: f32,
    pub maxhp: f32,
    pub r: f32,
    pub score: f32,
    pub t: f32,
    pub path: Path,
    pub speed: f32,
    pub vx: f32,
    pub fire: f32,
    pub fire_t: f32,
    pub art: Sprite,
    pub w: f32,
    pub h: f32,
    pub tint: Col,
    pub drop: Option<Item>,
    pub boss: bool,
    pub title: &'static str,
    pub midboss: bool,
    pub phase: i32,
    pub pt: f32,
    pub dead: bool,
    pub id: u32,
    pub visible: bool,
    pub flash: f32,
    pub hp_flash: f32,
}

#[derive(Clone)]
pub struct PBullet {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub dmg: f32,
    pub r: f32,
    pub life: f32,
    pub pierce: bool,
    /// Enemies already damaged, so a piercing shot cannot hit twice.
    pub hits: Vec<u32>,
    pub col: Option<Col>,
    pub agent: bool,
    pub trail: bool,
    pub homing: bool,
    pub missile: bool,
}

#[derive(Clone, Copy)]
pub struct EBullet {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub r: f32,
    pub life: f32,
    pub grazed: bool,
    pub col: Option<Col>,
}

#[derive(Clone, Copy)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub t: f32,
    pub r: f32,
    pub col: Col,
    pub glint: bool,
}

#[derive(Clone, Copy)]
pub struct Boom {
    pub x: f32,
    pub y: f32,
    pub t: f32,
    pub scale: f32,
}

#[derive(Clone)]
pub struct Floater {
    pub x: f32,
    pub y: f32,
    pub t: f32,
    pub text: String,
    pub col: Col,
}

#[derive(Clone, Copy)]
pub struct Pickup {
    pub x: f32,
    pub y: f32,
    pub vy: f32,
    pub t: f32,
    pub kind: Item,
}

/// The shining agent object that falls down the map; touching it recruits.
#[derive(Clone, Copy)]
pub struct TokenItem {
    pub id: AgentId,
    pub x: f32,
    pub y: f32,
    pub vy: f32,
    pub t: f32,
}

#[derive(Clone, Copy)]
pub struct Agent {
    pub id: AgentId,
    pub def: AgentDef,
    pub x: f32,
    pub y: f32,
    pub fire_t: f32,
    pub eat_r: f32,
    pub chomp: f32,
    pub eaten: u32,
    pub pop: f32,
    pub tokens: f32,
    pub low_warn: bool,
}

#[derive(Clone, Copy)]
pub struct Star {
    pub x: f32,
    pub y: f32,
    pub s: f32,
    pub v: f32,
    pub a: f32,
}

#[derive(Clone, Copy)]
pub struct Streak {
    pub x: f32,
    pub y: f32,
    pub len: f32,
    pub v: f32,
    pub a: f32,
}

/// Shopfronts and clouds sliding past under the fight.
#[derive(Clone, Copy)]
pub struct Prop {
    pub art: Sprite,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub vy: f32,
    pub sc: f32,
    pub a: f32,
}

#[derive(Clone, Copy)]
pub struct Ring {
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub t: f32,
    pub life: f32,
    pub col: Col,
}

#[derive(Clone, Copy)]
pub struct Ghost {
    pub x: f32,
    pub y: f32,
    pub t: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LaserState {
    Aim,
    Fire,
}

#[derive(Clone, Copy)]
pub struct Laser {
    pub x: f32,
    pub t: f32,
    pub state: LaserState,
}

#[derive(Clone)]
pub struct RecruitFx {
    pub name: &'static str,
    pub col: Col,
    pub x: f32,
    pub y: f32,
    pub t: f32,
    pub life: f32,
}

/// What the player is holding down this frame.
#[derive(Clone, Copy, Default)]
pub struct Pad {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub shoot: bool,
    /// Edge-triggered: one bomb per press.
    pub bomb: bool,
}

#[rustfmt::skip]
const SHOP_PROPS: [(Sprite, f32, f32, f32); 7] = [
    (Sprite::ShopHysan, 52.0, 52.0, 40.0),
    (Sprite::ShopMarket, 48.0, 48.0, 42.0),
    (Sprite::ShopRamen, 48.0, 48.0, 44.0),
    (Sprite::ShopDimsum, 48.0, 48.0, 42.0),
    (Sprite::ShopBakery, 48.0, 48.0, 40.0),
    (Sprite::ShopCoffee, 46.0, 46.0, 44.0),
    (Sprite::ShopCase, 46.0, 46.0, 46.0),
];

// ---------------------------------------------------------------------- world

pub struct World {
    pub rng: Rng,
    pub cues: Cues,

    pub stage: i32,
    pub loop_n: i32,
    pub rank: Rank,
    pub score: i64,
    pub hiscore: i64,
    pub lives: i32,
    pub bombs: i32,
    pub power: i32,
    pub extend_at: i64,

    // Rank-derived combat knobs.
    hp_mul: f32,
    spd_mul: f32,
    fire_rate_mul: f32,
    bullet_spd_mul: f32,
    boss_hp_mul: f32,
    agent_dmg_mul: f32,
    agent_rate_mul: f32,
    agent_eat_mul: f32,
    agent_auto_fire: bool,
    token_rain_empty: f32,
    token_rain_busy: f32,

    pub time: f32,
    pub player: Player,
    pub enemies: Vec<Enemy>,
    pub pbullets: Vec<PBullet>,
    pub ebullets: Vec<EBullet>,
    pub particles: Vec<Particle>,
    pub booms: Vec<Boom>,
    pub floaters: Vec<Floater>,
    pub pickups: Vec<Pickup>,
    pub token_items: Vec<TokenItem>,
    pub agents: Vec<Agent>,
    pub stars: Vec<Star>,
    pub streaks: Vec<Streak>,
    pub scenery: Vec<Prop>,
    pub rings: Vec<Ring>,
    pub ghosts: Vec<Ghost>,

    pub laser: Option<Laser>,
    pub own_ring: Option<Ring>,
    pub recruit_fx: Option<RecruitFx>,
    /// The BORROWCK lane: an x column that shreds bullets while shot is held.
    pub pbeam: Option<f32>,

    pub over: bool,
    pub clear: bool,
    pub clear_t: f32,
    pub ready_t: f32,
    pub warning_t: f32,
    pub boss_warn: [&'static str; 3],
    pub phase_name: &'static str,
    pub shake: f32,
    pub flash: f32,
    pub bg_y: f32,
    pub kills: u32,
    pub graze: u32,
    pub combo: u32,

    hitstop: f32,
    bombing: f32,
    bomb_flash: f32,
    combo_t: f32,
    unwrap_t: f32,
    unsafe_t: f32,
    tokio_t: f32,
    tokio_fire: f32,
    muzzle: f32,
    agent_boost: f32,
    prop_t: f32,
    shop_i: u32,
    agent_drop_i: u32,
    token_rain_t: f32,

    script: &'static [Beat],
    event_i: usize,
    next_id: u32,
    boss_id: Option<u32>,
    pub boss_dead: bool,

    /// Visible span in playfield coordinates, from the shell's window shape.
    view_l: f32,
    view_r: f32,
    cull_l: f32,
    cull_r: f32,
}

impl World {
    pub fn new(seed: u64, hiscore: i64, stage: i32, rank: Rank) -> World {
        let kit = rank.kit();
        let mut w = World {
            rng: Rng::new(seed),
            cues: Cues::default(),
            stage,
            loop_n: 1,
            rank,
            score: 0,
            hiscore,
            lives: kit.lives,
            bombs: kit.bombs,
            power: kit.power,
            extend_at: 30_000,
            hp_mul: 1.0,
            spd_mul: 1.0,
            fire_rate_mul: 1.0,
            bullet_spd_mul: 1.0,
            boss_hp_mul: 1.0,
            agent_dmg_mul: 1.0,
            agent_rate_mul: 1.0,
            agent_eat_mul: 1.0,
            agent_auto_fire: false,
            token_rain_empty: 6.4,
            token_rain_busy: 10.5,
            time: 0.0,
            player: Player {
                x: 96.0,
                y: 214.0,
                alive: true,
                inv: 3.2,
                flash: 0.0,
                fire_t: 0.0,
                dying: false,
                die_t: 0.0,
            },
            enemies: Vec::new(),
            pbullets: Vec::new(),
            ebullets: Vec::new(),
            particles: Vec::new(),
            booms: Vec::new(),
            floaters: Vec::new(),
            pickups: Vec::new(),
            token_items: Vec::new(),
            agents: Vec::new(),
            stars: Vec::new(),
            streaks: Vec::new(),
            scenery: Vec::new(),
            rings: Vec::new(),
            ghosts: Vec::new(),
            laser: None,
            own_ring: None,
            recruit_fx: None,
            pbeam: None,
            over: false,
            clear: false,
            clear_t: 0.0,
            ready_t: 2.2,
            warning_t: 0.0,
            boss_warn: ["WARNING", "BOSS", "INCOMING"],
            phase_name: "",
            shake: 0.0,
            flash: 0.0,
            bg_y: 0.0,
            kills: 0,
            graze: 0,
            combo: 0,
            hitstop: 0.0,
            bombing: 0.0,
            bomb_flash: 0.0,
            combo_t: 0.0,
            unwrap_t: 0.0,
            unsafe_t: 0.0,
            tokio_t: 0.0,
            tokio_fire: 0.0,
            muzzle: 0.0,
            agent_boost: 0.0,
            prop_t: 0.0,
            shop_i: 0,
            agent_drop_i: 0,
            token_rain_t: 0.0,
            script: stage::build(stage),
            event_i: 0,
            next_id: 1,
            boss_id: None,
            boss_dead: false,
            view_l: 0.0,
            view_r: GW,
            cull_l: -16.0,
            cull_r: GW + 16.0,
        };
        w.apply_rank();
        w.seed_stars();
        w
    }

    fn apply_rank(&mut self) -> Kit {
        let bal = self.rank.kit();
        let lp = (self.loop_n - 1) as f32;
        let st = (self.stage - 1) as f32;
        self.hp_mul = bal.hp_mul + lp * 0.18 + st * 0.06;
        self.spd_mul = bal.spd_mul + lp * 0.08 + st * 0.03;
        self.fire_rate_mul = bal.fire_rate_mul;
        self.bullet_spd_mul = bal.bullet_spd_mul;
        self.boss_hp_mul = bal.boss_hp_mul;
        self.agent_dmg_mul = bal.agent_dmg_mul;
        self.agent_rate_mul = bal.agent_rate_mul;
        self.agent_eat_mul = bal.agent_eat_mul;
        self.agent_auto_fire = bal.agent_auto_fire;
        self.token_rain_empty = bal.token_rain_empty;
        self.token_rain_busy = bal.token_rain_busy;
        bal
    }

    fn seed_stars(&mut self) {
        self.stars.clear();
        // Three layers: faint and slow behind, bright and quick in front.
        for (n, v0, v1, size, a) in
            [(28, 6.0, 14.0, 1.0, 0.28), (22, 18.0, 36.0, 1.4, 0.5), (14, 48.0, 90.0, 2.2, 0.8)]
        {
            for _ in 0..n {
                let (x0, x1) = self.ambient_span();
                let x = self.rng.int(x0 as i32, x1 as i32 - 1) as f32;
                let y = self.rng.int(0, 255) as f32;
                let v = self.rng.range(v0, v1);
                self.stars.push(Star { x, y, s: size, v, a });
            }
        }
    }

    pub fn set_view(&mut self, left: f32, right: f32) {
        self.view_l = left;
        self.view_r = right;
    }

    /// Wider than the playfield on a wide window, never narrower: ambient
    /// scenery should reach the edges of the screen even when the fight does not.
    pub fn ambient_span(&self) -> (f32, f32) {
        (self.view_l.min(0.0), self.view_r.max(GW))
    }

    /// Where the ship may fly.
    fn play_bounds(&self) -> (f32, f32) {
        (self.view_l + 10.0, self.view_r - 10.0)
    }

    pub fn rank(&self) -> Rank {
        self.rank
    }

    pub fn boss(&self) -> Option<&Enemy> {
        let id = self.boss_id?;
        self.enemies.iter().find(|e| e.id == id && !e.dead)
    }

    fn midboss(&self) -> Option<&Enemy> {
        self.enemies.iter().find(|e| e.midboss && !e.dead)
    }

    /// Restore the ship after a continue.
    pub fn revive(&mut self) {
        let kit = self.rank.kit();
        self.over = false;
        self.lives = kit.lives;
        self.bombs = kit.bombs;
        self.player.alive = true;
        self.player.dying = false;
        self.player.x = 96.0;
        self.player.y = 220.0;
        self.player.inv = 2.5;
    }

    // ------------------------------------------------------------ bookkeeping

    pub fn add_score(&mut self, n: f32) {
        self.score += n as i64;
        if self.score > self.hiscore {
            self.hiscore = self.score;
        }
        if self.score >= self.extend_at {
            self.lives = (self.lives + 1).min(9);
            self.extend_at += 100_000;
            self.cues.sfx(Sfx::OneUp);
            let (x, y) = (self.player.x, self.player.y - 16.0);
            self.floater(x, y, "1UP", palette::YELLOW);
        }
    }

    fn floater(&mut self, x: f32, y: f32, text: &str, col: Col) {
        self.floaters.push(Floater { x, y, t: 0.8, text: text.to_string(), col });
    }

    fn burst(&mut self, x: f32, y: f32, n: i32, col: Col, spd: f32) {
        for _ in 0..n {
            let a = self.rng.f() * std::f32::consts::TAU;
            let v = spd * (0.3 + self.rng.f());
            let life = 0.25 + self.rng.f() * 0.35;
            let r = 1.0 + self.rng.f() * 2.0;
            self.particles.push(Particle {
                x,
                y,
                vx: a.cos() * v,
                vy: a.sin() * v,
                life,
                t: 0.0,
                r,
                col,
                glint: false,
            });
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn spark(
        &mut self,
        x: f32,
        y: f32,
        vx: f32,
        vy: f32,
        col: Col,
        life: f32,
        r: f32,
        glint: bool,
    ) {
        // A hard cap, because a bomb over a full screen of bullets would
        // otherwise emit thousands of one-pixel sparks in a single frame.
        if self.particles.len() > 360 {
            return;
        }
        self.particles.push(Particle { x, y, vx, vy, life, t: 0.0, r, col, glint });
    }

    fn pop_ring(&mut self, x: f32, y: f32, col: Col, life: f32) {
        self.rings.push(Ring { x, y, r: 3.0, t: 0.0, life, col });
    }

    fn star_burst(&mut self, x: f32, y: f32, col: Col) {
        for i in 0..4 {
            let a = i as f32 * std::f32::consts::FRAC_PI_2 + 0.4;
            self.spark(x, y, a.cos() * 90.0, a.sin() * 90.0, col, 0.2, 2.0, false);
        }
    }

    fn boom(&mut self, x: f32, y: f32, scale: f32) {
        self.booms.push(Boom { x, y, t: 0.0, scale });
        self.burst(x, y, 10, palette::YELLOW, 110.0);
        self.burst(x, y, 8, palette::RUST, 70.0);
    }

    // ---------------------------------------------------------------- spawning

    fn spawn(&mut self, kind: Kind, x: f32, y: f32, o: Opts) -> u32 {
        let d = kind.def();
        let hp = if d.boss {
            let loop_b = 1.0 + (self.loop_n - 1) as f32 * 0.22;
            (d.hp * self.boss_hp_mul * loop_b).floor().max(1.0)
        } else {
            (d.hp * self.hp_mul).floor().max(1.0)
        };
        let speed = if o.speed > 0.0 { o.speed } else { d.speed } * self.spd_mul;
        let fire_t = 0.3 + self.rng.f() * 0.5;
        let id = self.next_id;
        self.next_id += 1;

        let mut e = Enemy {
            kind,
            x,
            y,
            x0: x,
            y0: y,
            hp,
            maxhp: hp,
            r: d.r,
            score: d.score,
            t: 0.0,
            path: o.path,
            speed,
            vx: o.vx,
            fire: d.fire,
            fire_t,
            art: d.art,
            w: d.w,
            h: d.h,
            tint: palette::WHITE,
            drop: o.drop,
            boss: d.boss,
            title: d.title,
            midboss: o.midboss,
            phase: 1,
            pt: 0.0,
            dead: false,
            id,
            visible: true,
            flash: 0.0,
            hp_flash: 0.0,
        };
        if o.midboss {
            // A midboss is an ordinary bug wearing eight times the health and
            // ten times the payout.
            e.hp = (d.hp * self.hp_mul * 8.0).floor();
            e.maxhp = e.hp;
            e.r = d.r + 6.0;
            e.score = d.score * 10.0;
        }
        if e.boss {
            self.boss_id = Some(id);
            self.boss_dead = false;
        }
        self.enemies.push(e);
        id
    }

    fn spawn_v(&mut self, kind: Kind, n: i32, cx: f32, y: f32, gap: f32) {
        for i in 0..n {
            let off = (i as f32 - (n - 1) as f32 / 2.0) * gap;
            let dy = -off.abs() * 0.6;
            self.spawn(kind, cx + off, y + dy, stage::DEFAULT_OPTS);
        }
    }

    fn spawn_line(&mut self, kind: Kind, n: i32, x0: f32, x1: f32, y: f32, path: Path) {
        for i in 0..n {
            let u = if n == 1 { 0.5 } else { i as f32 / (n - 1) as f32 };
            let o = Opts { path, ..stage::DEFAULT_OPTS };
            self.spawn(kind, x0 + (x1 - x0) * u, y - i as f32 * 12.0, o);
        }
    }

    fn drop_item(&mut self, x: f32, y: f32, item: Item) {
        // `A` is a wildcard: hand out Claude, Grok and Codex in rotation so a
        // run cannot be showered with the same assistant three times over.
        let item = if item == Item::Agent {
            self.agent_drop_i += 1;
            [Item::Claude, Item::Grok, Item::Codex][((self.agent_drop_i - 1) % 3) as usize]
        } else {
            item
        };
        if let Some(id) = item.agent() {
            self.spawn_token_item(id, x, y);
            return;
        }
        self.pickups.push(Pickup { x, y, vy: 18.0, t: 0.0, kind: item });
    }

    fn spawn_token_item(&mut self, id: AgentId, x: f32, y: f32) {
        self.token_items.push(TokenItem { id, x, y, vy: 52.0, t: 0.0 });
    }

    // ------------------------------------------------------------------ shots

    fn pshot(&mut self, x: f32, y: f32, vx: f32, vy: f32, dmg: f32, extra: ShotOpts) {
        let mut mul = if self.unsafe_t > 0.0 { 1.7 } else { 1.35 };
        if self.agent_boost > 0.0 {
            mul *= 1.28;
        }
        self.pbullets.push(PBullet {
            x,
            y,
            vx,
            vy,
            dmg: dmg * mul,
            r: extra.r,
            life: extra.life,
            pierce: extra.pierce || self.unwrap_t > 0.0,
            hits: Vec::new(),
            col: extra.col,
            agent: extra.agent,
            trail: extra.trail,
            homing: extra.homing,
            missile: extra.missile,
        });
    }

    fn eshot(&mut self, x: f32, y: f32, vx: f32, vy: f32, r: f32, col: Option<Col>) {
        // The screen has a bullet budget; past it, the next wave simply does
        // not fire rather than turning the playfield into a wall.
        if self.ebullets.len() > 120 {
            return;
        }
        self.ebullets.push(EBullet { x, y, vx, vy, r, life: 4.0, grazed: false, col });
    }

    /// `n` shots fanned around the line to the player.
    fn aimed(&mut self, x: f32, y: f32, n: i32, spread: f32, spd: f32) {
        let a = (self.player.y - y).atan2(self.player.x - x);
        if n <= 1 {
            self.eshot(x, y, a.cos() * spd, a.sin() * spd, 3.0, None);
            return;
        }
        for i in 0..n {
            let off = (i as f32 - (n - 1) as f32 / 2.0) * spread;
            self.eshot(x, y, (a + off).cos() * spd, (a + off).sin() * spd, 3.0, None);
        }
    }

    fn ring_shot(&mut self, x: f32, y: f32, n: i32, spd: f32, rot: f32) {
        for i in 0..n {
            let a = rot + i as f32 * std::f32::consts::TAU / n as f32;
            self.eshot(x, y, a.cos() * spd, a.sin() * spd, 3.0, None);
        }
    }

    fn fire_player(&mut self) {
        let (px, py) = (self.player.x, self.player.y);
        let y = py - 14.0;
        let pow = self.power;
        let pierce = self.unwrap_t > 0.0;
        let col = if self.unsafe_t > 0.0 {
            palette::LRED
        } else if pierce {
            palette::YELLOW
        } else {
            [palette::CYAN, palette::LGREEN, palette::YELLOW, palette::CYAN]
                [(pow.clamp(1, 4) - 1) as usize]
        };
        self.muzzle = 0.08;
        self.spark(px, y + 4.0, 0.0, 40.0, col, 0.12, 2.0, false);
        self.spark(px - 3.0, y + 6.0, -20.0, 30.0, palette::WHITE, 0.1, 1.0, false);
        self.spark(px + 3.0, y + 6.0, 20.0, 30.0, palette::WHITE, 0.1, 1.0, false);

        let base = ShotOpts { pierce, trail: true, col: Some(col), ..ShotOpts::DEFAULT };
        match pow {
            1 => self.pshot(px, y, 0.0, -320.0, 1.1, base),
            2 => {
                // TRAIT: two owned shots.
                let o = ShotOpts { col: Some(palette::LGREEN), ..base };
                self.pshot(px - 6.0, y + 2.0, 0.0, -330.0, 1.15, o);
                self.pshot(px + 6.0, y + 2.0, 0.0, -330.0, 1.15, o);
            }
            3 => {
                // ASYNC: three-way.
                let o = ShotOpts { col: Some(palette::YELLOW), ..base };
                self.pshot(px, y, 0.0, -350.0, 1.35, o);
                self.pshot(px - 9.0, y + 4.0, -50.0, -315.0, 1.1, o);
                self.pshot(px + 9.0, y + 4.0, 50.0, -315.0, 1.1, o);
            }
            _ => {
                // TOKIO: wide, plus homing missiles.
                let o = ShotOpts { col: Some(palette::CYAN), ..base };
                self.pshot(px, y, 0.0, -360.0, 1.5, o);
                self.pshot(px - 8.0, y + 2.0, -30.0, -340.0, 1.1, o);
                self.pshot(px + 8.0, y + 2.0, 30.0, -340.0, 1.1, o);
                self.fire_tokio(px, y);
            }
        }
        self.cues.sfx(if pow >= 3 { Sfx::Shot2 } else { Sfx::Shot });
    }

    fn fire_tokio(&mut self, x: f32, y: f32) {
        let o = ShotOpts {
            homing: true,
            missile: true,
            trail: true,
            r: 4.0,
            life: 1.6,
            col: Some(palette::RUST),
            ..ShotOpts::DEFAULT
        };
        self.pshot(x - 12.0, y + 6.0, -40.0, -220.0, 1.6, o);
        self.pshot(x + 12.0, y + 6.0, 40.0, -220.0, 1.6, o);
    }

    // ----------------------------------------------------------------- agents

    fn has_agent(&self, id: AgentId) -> bool {
        self.agents.iter().any(|a| a.id == id)
    }

    /// Bring an agent in, or top up one already flying.
    fn recruit_agent(&mut self, want: Option<AgentId>, quiet: bool) {
        let id = match want {
            Some(id) => Some(id),
            None => AGENTS.iter().copied().find(|id| !self.has_agent(*id)),
        };
        let Some(id) = id else {
            // All three are already out: the tokens go to the hungriest.
            if let Some(i) = self
                .agents
                .iter()
                .enumerate()
                .min_by(|a, b| a.1.tokens.total_cmp(&b.1.tokens))
                .map(|(i, _)| i)
            {
                self.refill_agent(i);
            }
            return;
        };
        if let Some(i) = self.agents.iter().position(|a| a.id == id) {
            self.refill_agent(i);
            return;
        }

        let def = id.def();
        let (px, py) = (self.player.x, self.player.y);
        self.agents.push(Agent {
            id,
            def,
            x: px,
            y: py,
            fire_t: 0.45,
            eat_r: def.eat_r * self.agent_eat_mul,
            chomp: 0.35,
            eaten: 0,
            pop: if quiet { 1.0 } else { 0.0 },
            tokens: def.token_max,
            low_warn: false,
        });
        if quiet {
            return;
        }
        self.agent_boost = 2.4;
        self.floater(px, py - 22.0, def.name, def.col);
        self.floater(px, py - 34.0, "TOKEN IN", def.col);
        self.start_recruit_fx(def.name, def.col, px, py);
        self.cues.sfx(Sfx::OneUp);
    }

    fn refill_agent(&mut self, i: usize) {
        let (x, y, name, col) = {
            let ag = &mut self.agents[i];
            let max = ag.def.token_max;
            ag.tokens = (ag.tokens + max).min(max * 1.15);
            ag.low_warn = false;
            ag.pop = 0.0;
            (ag.x, ag.y, ag.def.name, ag.def.col)
        };
        self.agent_boost = 1.6;
        self.floater(x, y - 18.0, "TOKEN +", col);
        self.start_recruit_fx(name, col, x, y);
        self.cues.sfx(Sfx::Power);
    }

    /// Out of tokens: the agent leaves, dropping its badge back on the map so
    /// it can be picked up again.
    fn expire_agent(&mut self, i: usize) {
        let ag = self.agents.remove(i);
        self.floater(ag.x, ag.y - 16.0, "TOKEN OUT", ag.def.col);
        self.burst(ag.x, ag.y, 18, ag.def.col, 160.0);
        self.pop_ring(ag.x, ag.y, ag.def.col, 0.4);
        let y = (ag.y - 10.0).max(28.0);
        self.spawn_token_item(ag.id, ag.x, (y - 28.0).max(20.0));
    }

    fn start_recruit_fx(&mut self, name: &'static str, col: Col, x: f32, y: f32) {
        self.recruit_fx = Some(RecruitFx { name, col, x, y, t: 0.0, life: 2.2 });
        self.flash = 0.18;
        self.burst(x, y, 28, col, 180.0);
        self.burst(x, y, 10, palette::WHITE, 90.0);
        for i in 0..12 {
            let a = i as f32 * (std::f32::consts::TAU / 12.0);
            self.spark(x, y, a.cos() * 140.0, a.sin() * 140.0, col, 0.45, 2.4, true);
            let b = a + 0.3;
            self.spark(x, y, b.cos() * 70.0, b.sin() * 70.0, palette::WHITE, 0.32, 1.6, true);
        }
        self.pop_ring(x, y, col, 0.55);
        self.pop_ring(x, y, palette::WHITE, 0.32);
    }

    fn nearest_enemy(&self, x: f32, y: f32) -> Option<(f32, f32)> {
        self.enemies
            .iter()
            .filter(|e| !e.dead)
            .min_by(|a, b| dist2(x, y, a.x, a.y).total_cmp(&dist2(x, y, b.x, b.y)))
            .map(|e| (e.x, e.y))
    }

    /// The closest bullet that is already threatening the ship — what an agent
    /// breaks formation to intercept.
    fn nearest_threat(&self, ax: f32, ay: f32, max_d: f32) -> Option<(f32, f32)> {
        let p = self.player;
        self.ebullets
            .iter()
            .filter(|b| dist2(b.x, b.y, p.x, p.y) <= max_d * max_d)
            .min_by(|a, b| dist2(ax, ay, a.x, a.y).total_cmp(&dist2(ax, ay, b.x, b.y)))
            .map(|b| (b.x, b.y))
    }

    /// Off Easy, an agent holds fire while a bullet is close enough to eat:
    /// blocking beats chipping.
    fn agent_should_shoot(&self, ag: &Agent) -> bool {
        if self.agent_auto_fire {
            return true;
        }
        let r = ag.eat_r * 2.2;
        !self.ebullets.iter().any(|b| dist2(b.x, b.y, ag.x, ag.y) <= r * r)
    }

    fn agent_shoot(&mut self, i: usize) {
        let ag = self.agents[i];
        let dmg_mul = self.agent_dmg_mul;
        let auto = self.agent_auto_fire;
        let o = ShotOpts { col: Some(ag.def.col), agent: true, trail: true, ..ShotOpts::DEFAULT };
        match ag.id {
            AgentId::Claude => {
                let a = match self.nearest_enemy(ag.x, ag.y) {
                    Some((tx, ty)) => (ty - ag.y).atan2(tx - ag.x),
                    None => -std::f32::consts::FRAC_PI_2,
                };
                let spd = if auto { 280.0 } else { 260.0 };
                let dmg = if auto { 1.15 } else { 0.5 } * dmg_mul;
                self.pshot(ag.x, ag.y, a.cos() * spd, a.sin() * spd, dmg, ShotOpts { r: 3.0, ..o });
                self.spark(ag.x, ag.y, a.cos() * 40.0, a.sin() * 40.0, ag.def.col, 0.18, 2.0, true);
            }
            AgentId::Grok => {
                let wob = (self.time * 11.0).sin() * if auto { 28.0 } else { 36.0 };
                if auto {
                    self.pshot(ag.x, ag.y, wob - 90.0, -240.0, 0.55 * dmg_mul, o);
                    self.pshot(ag.x, ag.y, wob - 40.0, -265.0, 0.55 * dmg_mul, o);
                    self.pshot(ag.x, ag.y, wob, -285.0, 0.7 * dmg_mul, o);
                    self.pshot(ag.x, ag.y, wob + 40.0, -265.0, 0.55 * dmg_mul, o);
                    self.pshot(ag.x, ag.y, wob + 90.0, -240.0, 0.55 * dmg_mul, o);
                } else {
                    self.pshot(ag.x, ag.y, wob, -250.0, 0.4 * dmg_mul, o);
                }
            }
            AgentId::Codex => {
                let o = ShotOpts { r: 2.0, ..o };
                if auto {
                    self.pshot(ag.x - 4.0, ag.y, 0.0, -300.0, 0.5 * dmg_mul, o);
                    self.pshot(ag.x + 4.0, ag.y, 0.0, -300.0, 0.5 * dmg_mul, o);
                    if (self.time * 8.0).floor() as i32 % 2 == 0 {
                        self.pshot(ag.x, ag.y, -18.0, -290.0, 0.4 * dmg_mul, o);
                        self.pshot(ag.x, ag.y, 18.0, -290.0, 0.4 * dmg_mul, o);
                    }
                } else {
                    self.pshot(ag.x, ag.y, 0.0, -280.0, 0.35 * dmg_mul, o);
                }
            }
        }
    }

    fn update_agents(&mut self, dt: f32) {
        let p = self.player;
        let n = self.agents.len();
        let boosted = self.agent_boost > 0.0;
        for i in (0..n).rev() {
            let ag = self.agents[i];
            let orbit = ag.def.orbit;
            let spin = ag.def.spin;
            let (mut tx, mut ty) = match ag.id {
                AgentId::Codex => (p.x, p.y - 22.0),
                AgentId::Grok => {
                    let ang = self.time * spin + (i + 1) as f32 * 2.2;
                    (p.x + ang.cos() * orbit, p.y - 18.0 + (ang * 1.4).sin() * 10.0)
                }
                AgentId::Claude => {
                    let ang =
                        self.time * spin + i as f32 * (std::f32::consts::TAU / n.max(1) as f32);
                    (p.x + ang.cos() * orbit, p.y - 16.0 + ang.sin() * 8.0)
                }
            };
            let threat = self.nearest_threat(ag.x, ag.y, 86.0);
            if let Some((bx, by)) = threat {
                tx += (bx - tx) * 0.62;
                ty += (by - ty) * 0.62;
            }
            tx = tx.clamp(12.0, 180.0);
            ty = ty.clamp(18.0, 236.0);
            let mut follow = if ag.id == AgentId::Grok { 16.0 } else { 11.0 };
            if threat.is_some() {
                follow += 6.0;
            }

            let working = p.alive && self.ready_t <= 0.0 && !self.clear && !self.over;
            {
                let ag = &mut self.agents[i];
                ag.x += (tx - ag.x) * (dt * follow).min(1.0);
                ag.y += (ty - ag.y) * (dt * follow).min(1.0);
                ag.chomp = (ag.chomp - dt).max(0.0);
                ag.pop += dt * 2.6;
                if working {
                    ag.tokens -= dt * ag.def.drain;
                }
            }
            let ag = self.agents[i];
            if working && ag.tokens <= ag.def.token_max * 0.25 && !ag.low_warn {
                self.agents[i].low_warn = true;
                self.floater(ag.x, ag.y - 14.0, "LOW TOKEN", ag.def.col);
            }

            let spark_rate = match ag.id {
                AgentId::Grok => 36.0,
                AgentId::Claude => 14.0,
                AgentId::Codex => 22.0,
            };
            let rate = if boosted { spark_rate * 2.0 } else { spark_rate };
            if self.rng.chance(dt * rate) {
                let vx = (self.rng.f() - 0.5) * 28.0;
                let vy = (self.rng.f() - 0.5) * 28.0;
                self.spark(ag.x, ag.y, vx, vy, ag.def.col, 0.2, 1.6, true);
            }

            if working {
                let fire_mul = if self.agent_auto_fire && boosted { 1.7 } else { 1.0 };
                self.agents[i].fire_t -= dt * fire_mul;
                if self.agents[i].fire_t <= 0.0 {
                    self.agents[i].fire_t = ag.def.rate * self.agent_rate_mul;
                    if self.agent_should_shoot(&self.agents[i]) {
                        self.agent_shoot(i);
                        self.agents[i].tokens -= ag.def.shot_cost;
                    } else {
                        // Try again soon: the moment the lane clears, shoot.
                        self.agents[i].fire_t = self.agents[i].fire_t.min(0.28);
                    }
                }
            }
            if self.agents[i].tokens <= 0.0 {
                self.expire_agent(i);
            }
        }
    }

    fn agent_chomp(&mut self, i: usize, x: f32, y: f32) {
        let (ax, ay, col, eaten) = {
            let ag = &mut self.agents[i];
            ag.chomp = 0.22;
            ag.eaten += 1;
            (ag.x, ag.y, ag.def.col, ag.eaten)
        };
        self.add_score(25.0);
        self.burst(x, y, 10, col, 150.0);
        self.burst(x, y, 5, palette::WHITE, 80.0);
        self.star_burst(x, y, col);
        self.pop_ring(x, y, col, 0.3);
        self.pop_ring(ax, ay, palette::WHITE, 0.16);
        for _ in 0..4 {
            self.spark(x, y, (ax - x) * 3.0, (ay - y) * 3.0, col, 0.18, 2.0, false);
        }
        self.flash = self.flash.max(0.07);
        self.cues.sfx(Sfx::Graze);
        if eaten % 5 == 0 {
            self.floater(ax, ay - 14.0, "EAT", col);
        }
    }

    /// True if an agent swallowed the bullet at `(bx, by)`.
    fn try_agent_eat(&mut self, bx: f32, by: f32) -> bool {
        let hit =
            self.agents.iter().position(|ag| dist2(bx, by, ag.x, ag.y) <= ag.eat_r * ag.eat_r);
        match hit {
            Some(i) => {
                self.agent_chomp(i, bx, by);
                true
            }
            None => false,
        }
    }

    fn agent_pop(&mut self, x: f32, y: f32, col: Col) {
        self.burst(x, y, 18, col, 170.0);
        self.burst(x, y, 10, palette::YELLOW, 110.0);
        self.star_burst(x, y, col);
        self.pop_ring(x, y, col, 0.42);
        self.pop_ring(x, y, palette::WHITE, 0.2);
        self.shake = self.shake.max(3.5);
        self.hitstop = self.hitstop.max(0.045);
        self.flash = self.flash.max(0.1);
    }

    // ------------------------------------------------------------ life, death

    fn kill_enemy(&mut self, i: usize, silent: bool) {
        if self.enemies[i].dead {
            return;
        }
        let e = {
            let e = &mut self.enemies[i];
            e.dead = true;
            e.hp = 0.0;
            *e
        };
        self.kills += 1;
        self.combo += 1;
        self.combo_t = 1.4;
        let mul = 1.0 + (self.combo / 8).min(4) as f32 * 0.25;
        self.add_score(e.score * mul * self.loop_n as f32);

        if !silent {
            self.boom(e.x, e.y, if e.boss { 2.2 } else { 1.0 });
            self.pop_ring(e.x, e.y, palette::YELLOW, if e.boss { 0.55 } else { 0.28 });
            self.star_burst(e.x, e.y, palette::WHITE);
            if e.boss {
                self.cues.sfx(Sfx::ExplodeBig);
                self.shake = 10.0;
                self.flash = 0.35;
            } else {
                self.cues.sfx(Sfx::Explode);
                self.shake = self.shake.max(2.0);
            }
        }

        if let Some(item) = e.drop {
            self.drop_item(e.x, e.y, item);
        } else if e.boss {
            self.drop_item(e.x, e.y, Item::Agent);
            self.drop_item(e.x - 16.0, e.y + 8.0, Item::Bomb);
            self.drop_item(e.x + 16.0, e.y + 8.0, Item::Power);
            self.drop_item(e.x, e.y + 16.0, Item::Unwrap);
        } else if e.midboss {
            self.drop_item(e.x, e.y, Item::Agent);
            self.drop_item(e.x - 14.0, e.y + 8.0, Item::Power);
            self.drop_item(e.x + 14.0, e.y + 8.0, Item::Bomb);
            self.shake = self.shake.max(6.0);
            self.cues.sfx(Sfx::ExplodeBig);
        } else if self.rng.chance(e.kind.drop_chance()) {
            let roll = self.rng.f();
            let mut k = if roll > 0.96 {
                Item::OneUp
            } else if roll > 0.62 {
                Item::Agent
            } else if roll > 0.52 {
                Item::Unwrap
            } else if roll > 0.44 {
                Item::Unsafe
            } else if roll > 0.32 {
                Item::Bomb
            } else if roll > 0.20 {
                Item::Score
            } else {
                Item::Power
            };
            if e.kind == Kind::Overflow && self.rng.chance(0.45) {
                k = Item::Agent;
            }
            // Flying alone is hard; the game offers a hand more often then.
            if self.agents.is_empty() && self.rng.chance(0.35) {
                k = Item::Agent;
            }
            self.drop_item(e.x, e.y, k);
        }

        if e.boss {
            self.boss_dead = true;
            self.clear = true;
            self.clear_t = 3.5;
            self.ebullets.clear();
            self.add_score(20000.0 * self.loop_n as f32);
        }
    }

    /// OWNERSHIP: the bomb takes the screen's bullets away from the bugs, and
    /// turns some of them into rust shots on the way out.
    fn do_bomb(&mut self) -> bool {
        if self.bombs <= 0 || self.bombing > 0.0 {
            return false;
        }
        let (px, py) = (self.player.x, self.player.y);
        self.bombs -= 1;
        self.bombing = 0.9;
        self.bomb_flash = 1.0;
        self.shake = 8.0;
        self.own_ring = Some(Ring { x: px, y: py, r: 6.0, t: 0.0, life: 0.72, col: palette::RUST });
        self.cues.sfx(Sfx::Bomb);
        self.cues.sfx(Sfx::ExplodeBig);
        self.floater(px, py - 20.0, "OWNERSHIP", palette::RUST);
        self.burst(px, py, 22, palette::RUST, 140.0);
        self.burst(px, py, 14, palette::YELLOW, 90.0);

        let taken: Vec<EBullet> = std::mem::take(&mut self.ebullets);
        for b in taken {
            self.add_score(25.0);
            self.spark(b.x, b.y, 0.0, -40.0, palette::RUST, 0.28, 2.0, false);
            if self.rng.chance(0.4) {
                self.pshot(
                    b.x,
                    b.y,
                    0.0,
                    -240.0,
                    1.3,
                    ShotOpts { col: Some(palette::RUST), r: 3.0, ..ShotOpts::DEFAULT },
                );
            }
        }
        for i in 0..self.enemies.len() {
            if !self.enemies[i].dead {
                self.enemies[i].hp -= 16.0;
                let (x, y) = (self.enemies[i].x, self.enemies[i].y);
                self.burst(x, y, 8, palette::CYAN, 90.0);
            }
        }
        self.player.inv = self.player.inv.max(2.0);
        if self.player.dying {
            self.player.dying = false;
            self.player.die_t = 0.0;
            self.player.inv = 2.6;
        }
        true
    }

    /// Hit — but not gone. The death-bomb window opens instead.
    fn hurt_player(&mut self) {
        let p = &mut self.player;
        if !p.alive || p.inv > 0.0 || p.dying {
            return;
        }
        if self.bombing > 0.0 {
            return;
        }
        p.dying = true;
        p.die_t = 0.28;
    }

    fn really_die(&mut self) {
        self.player.dying = false;
        self.player.alive = false;
        self.power = (self.power - 1).max(1);
        if let Some(lost) = self.agents.pop() {
            let (x, y) = (self.player.x, self.player.y - 22.0);
            let text = format!("{} DOWN", lost.def.name);
            self.floater(x, y, &text, lost.def.col);
        }
        let (px, py) = (self.player.x, self.player.y);
        self.boom(px, py, 1.4);
        self.cues.sfx(Sfx::Death);
        self.shake = 7.0;
        self.lives -= 1;
        if self.power >= 2 {
            self.drop_item(px, py - 10.0, Item::Power);
        }
        if self.lives < 0 {
            self.over = true;
            return;
        }
        self.player.alive = true;
        self.player.x = 96.0;
        self.player.y = 230.0;
        self.player.inv = 3.4;
        self.player.flash = 0.0;
    }

    // ------------------------------------------------------------ enemy brains

    fn update_enemy(&mut self, i: usize, dt: f32) {
        let mut e = self.enemies[i];
        e.t += dt;
        e.flash = (e.flash - dt).max(0.0);
        e.hp_flash = (e.hp_flash - dt).max(0.0);

        match e.path {
            Path::Down => e.y += e.speed * dt,
            Path::Sine => {
                e.y += e.speed * dt;
                e.x = e.x0 + (e.t * 3.2).sin() * 30.0;
            }
            Path::Zigzag => {
                e.y += e.speed * dt;
                e.x += e.vx * dt;
                if e.x < 16.0 || e.x > 176.0 {
                    e.vx = -e.vx;
                    e.x = e.x.clamp(16.0, 176.0);
                }
            }
            Path::Swoop => {
                e.y += e.speed * dt;
                e.x = e.x0 + (e.t * 2.2).sin() * 50.0;
                if e.t > 1.2 {
                    e.y += 20.0 * dt;
                }
            }
            Path::Hover => {
                if e.y < 54.0 {
                    e.y += 46.0 * dt;
                } else {
                    e.x = e.x0 + (e.t * 1.3).sin() * if e.midboss { 36.0 } else { 48.0 };
                    let leave = if e.midboss { 20.0 } else { 6.5 };
                    if e.t > leave {
                        e.y += if e.midboss { 22.0 } else { 40.0 } * dt;
                    }
                }
            }
            Path::Dive => {
                if e.t < 0.7 {
                    e.y += e.speed * dt;
                } else {
                    let a = (self.player.y - e.y).atan2(self.player.x - e.x);
                    e.x += a.cos() * e.speed * 1.15 * dt;
                    e.y += a.sin() * e.speed * 1.15 * dt;
                }
            }
            Path::Side => {
                let vx = if e.vx != 0.0 { e.vx } else { e.speed };
                e.x += vx * dt;
                e.y = e.y0 + 52.0 + (e.t * 2.1).sin() * 22.0;
            }
            Path::Teleport => {
                e.y += e.speed * 0.6 * dt;
                e.visible = (e.t * 8.0).floor() as i32 % 3 != 0;
                if (e.t * 2.0).floor() != ((e.t - dt) * 2.0).floor() && self.rng.chance(0.4) {
                    e.x = 24.0 + self.rng.f() * 144.0;
                }
            }
            Path::Orbit => {
                e.x = e.x0 + (e.t * 2.4).cos() * 42.0;
                e.y = e.y0 + e.t * 22.0 + (e.t * 2.4).sin() * 18.0;
            }
            Path::Boss => {}
        }

        // Bugs that flicker: the heisenbug is only there when you look away,
        // and the lifetime error keeps going out of scope.
        match e.kind {
            Kind::Heisen => {
                e.visible = (e.t * 10.0).floor() as i32 % 4 != 0;
                e.tint = if e.visible { [0.7, 0.9, 1.0, 0.85] } else { [0.7, 0.9, 1.0, 0.15] };
            }
            Kind::Lifetime => {
                e.visible = (e.t * 7.0).floor() as i32 % 5 != 0;
                e.tint = if e.visible { [0.65, 1.0, 1.0, 0.92] } else { [0.65, 1.0, 1.0, 0.18] };
            }
            Kind::Infloop => {
                e.tint = [1.0, 0.7 + 0.3 * (e.t * 8.0).sin(), 1.0, 1.0];
            }
            _ => {}
        }

        let mut fired = false;
        if e.fire > 0.0 && e.y > 8.0 && e.y < 190.0 && !e.boss {
            e.fire_t -= dt * self.fire_rate_mul;
            if e.fire_t <= 0.0 {
                e.fire_t = e.fire * (0.85 + self.rng.f() * 0.3);
                fired = true;
            }
        }
        let visible = e.visible;
        let panic_pop = e.kind == Kind::Panic
            && e.y > 40.0
            && (e.t > 1.4 || dist2(e.x, e.y, self.player.x, self.player.y) < 400.0);
        self.enemies[i] = e;

        if fired {
            let spd = (48.0 + self.loop_n as f32 * 5.0) * self.bullet_spd_mul;
            match e.kind {
                Kind::Moth => self.aimed(e.x, e.y + 8.0, 1, 0.0, spd),
                Kind::Spider => self.aimed(e.x, e.y + 8.0, 2, 0.18, spd - 8.0),
                Kind::Leak => {
                    let g = Some(palette::LGREEN);
                    self.eshot(e.x, e.y + 10.0, 0.0, spd * 0.55, 4.0, g);
                    self.eshot(e.x - 8.0, e.y + 8.0, -20.0, spd * 0.5, 3.0, g);
                    self.eshot(e.x + 8.0, e.y + 8.0, 20.0, spd * 0.5, 3.0, g);
                }
                Kind::Overflow => {
                    self.ring_shot(e.x, e.y, if e.midboss { 8 } else { 5 }, spd - 10.0, e.t);
                    if e.midboss {
                        self.aimed(e.x, e.y + 12.0, 1, 0.0, spd);
                    }
                }
                Kind::Deadlock => {
                    self.eshot(e.x - 10.0, e.y, -spd, 18.0, 3.0, Some(palette::MAGENTA));
                    self.eshot(e.x + 10.0, e.y, spd, 18.0, 3.0, Some(palette::CYAN));
                    self.aimed(e.x, e.y + 6.0, 1, 0.0, spd - 12.0);
                }
                Kind::Heisen if visible => self.aimed(e.x, e.y + 6.0, 2, 0.26, spd + 10.0),
                // Off by one: always one shot more than you counted on.
                Kind::Offby1 => self.aimed(e.x, e.y + 8.0, 2, 0.14, spd),
                Kind::Clippy => {
                    self.aimed(e.x, e.y + 8.0, 2, 0.2, spd - 12.0);
                    self.eshot(e.x, e.y + 10.0, 0.0, spd * 0.45, 4.0, Some(palette::YELLOW));
                }
                Kind::Lifetime if visible => self.aimed(e.x, e.y + 6.0, 1, 0.0, spd + 16.0),
                Kind::Infloop => self.ring_shot(e.x, e.y, 4, spd - 16.0, e.t * 2.0),
                _ => {}
            }
        }

        // A panic does not shoot; it unwinds, all over you.
        if panic_pop {
            self.ring_shot(e.x, e.y, 6, 58.0, e.t);
            self.kill_enemy(i, false);
        }
    }

    fn update_boss(&mut self, i: usize, dt: f32) {
        let mut e = self.enemies[i];
        e.pt += dt;
        e.t += dt;
        e.flash = (e.flash - dt).max(0.0);
        e.hp_flash = (e.hp_flash - dt).max(0.0);

        // The entrance: drift down into the arena before anything else.
        if e.y < 48.0 {
            e.y += 26.0 * dt;
            self.enemies[i] = e;
            self.phase_name = if e.title.is_empty() { "BOSS" } else { e.title };
            return;
        }

        let u = e.hp / e.maxhp.max(1.0);
        let phase = if u > 0.7 {
            1
        } else if u > 0.42 {
            2
        } else {
            3
        };
        let announce = phase != e.phase;
        if announce {
            e.phase = phase;
            e.pt = 0.0;
        }

        let sway = 28.0 + phase as f32 * 8.0;
        e.x = 96.0 + (e.t * (0.5 + phase as f32 * 0.1)).sin() * sway;
        e.fire_t -= dt * self.fire_rate_mul;
        let ready = e.fire_t <= 0.0;
        let spd =
            (40.0 + self.loop_n as f32 * 5.0 + phase as f32 * 3.0 + (self.stage - 1) as f32 * 3.0)
                * self.bullet_spd_mul;

        if announce {
            let name = e.kind.phase_names()[(phase - 1) as usize];
            self.phase_name = name;
            self.enemies[i] = e;
            self.floater(96.0, 70.0, name, palette::LRED);
            self.cues.sfx(Sfx::Warn);
            self.shake = 3.0;
            e = self.enemies[i];
        }

        match e.kind {
            Kind::BossOverflow => {
                if ready {
                    match phase {
                        1 => {
                            e.fire_t = 0.85;
                            self.enemies[i] = e;
                            self.aimed(e.x, e.y + 24.0, 2, 0.16, spd);
                        }
                        2 => {
                            e.fire_t = 1.05;
                            self.enemies[i] = e;
                            for k in -1..=1 {
                                self.eshot(
                                    e.x + k as f32 * 16.0,
                                    e.y + 20.0,
                                    0.0,
                                    spd + 6.0,
                                    3.0,
                                    None,
                                );
                            }
                        }
                        _ => {
                            e.fire_t = 0.42;
                            self.enemies[i] = e;
                            self.aimed(e.x, e.y + 22.0, 1, 0.0, spd + 8.0);
                            self.eshot(e.x - 20.0, e.y + 10.0, -24.0, spd, 3.0, None);
                            self.eshot(e.x + 20.0, e.y + 10.0, 24.0, spd, 3.0, None);
                        }
                    }
                } else {
                    self.enemies[i] = e;
                }
                self.laser = None;
                return;
            }
            Kind::BossDeadlock => {
                if ready {
                    match phase {
                        1 => {
                            e.fire_t = 0.95;
                            self.enemies[i] = e;
                            self.ring_shot(e.x, e.y + 8.0, 6, spd - 10.0, e.t);
                        }
                        2 => {
                            e.fire_t = 1.15;
                            self.enemies[i] = e;
                            self.aimed(e.x - 18.0, e.y + 16.0, 2, 0.2, spd);
                            self.aimed(e.x + 18.0, e.y + 16.0, 2, 0.22, spd);
                        }
                        _ => {
                            e.fire_t = 0.5;
                            self.enemies[i] = e;
                            self.ring_shot(e.x, e.y + 10.0, 7, spd - 4.0, e.t * 1.2);
                            // Phase three calls for help down the tunnel.
                            if self.rng.chance(0.2) {
                                let from_left = self.rng.chance(0.5);
                                let vx = if self.rng.chance(0.5) { 40.0 } else { -50.0 };
                                self.spawn(
                                    Kind::Deadlock,
                                    if from_left { -16.0 } else { 210.0 },
                                    10.0,
                                    Opts { path: Path::Side, vx, ..stage::DEFAULT_OPTS },
                                );
                            }
                        }
                    }
                } else {
                    self.enemies[i] = e;
                }
                self.laser = None;
                return;
            }
            _ => {}
        }

        // SEGMENTATION FAULT, the one at the end of the world.
        if ready {
            match phase {
                1 => {
                    e.fire_t = 0.8;
                    self.enemies[i] = e;
                    self.aimed(e.x, e.y + 28.0, 2, 0.18, spd);
                }
                2 => {
                    e.fire_t = 1.2;
                    self.enemies[i] = e;
                    self.ring_shot(e.x, e.y + 12.0, 8, spd - 8.0, e.t);
                }
                _ => {
                    e.fire_t = 0.28;
                    self.enemies[i] = e;
                    let a = e.t * 4.4;
                    self.eshot(e.x - 18.0, e.y + 10.0, a.cos() * spd, a.sin() * spd, 3.0, None);
                    self.eshot(
                        e.x + 18.0,
                        e.y + 10.0,
                        (-a).cos() * spd,
                        (-a).sin() * spd,
                        3.0,
                        None,
                    );
                }
            }
        } else {
            self.enemies[i] = e;
        }

        if phase < 3 {
            self.laser = None;
            return;
        }
        // The kill lane: it tracks, it locks, and then it fires.
        let mut l =
            self.laser.unwrap_or(Laser { x: self.player.x, t: 0.0, state: LaserState::Aim });
        l.t += dt;
        match l.state {
            LaserState::Aim => {
                l.x += (self.player.x - l.x) * dt * 1.4;
                if l.t > 1.5 {
                    l.state = LaserState::Fire;
                    l.t = 0.0;
                    self.cues.sfx(Sfx::Warn2);
                }
                self.laser = Some(l);
            }
            LaserState::Fire => {
                if self.player.alive && (self.player.x - l.x).abs() < 7.0 && self.player.y > 40.0 {
                    self.hurt_player();
                }
                self.laser = if l.t > 0.28 { None } else { Some(l) };
            }
        }
    }

    // ----------------------------------------------------------------- update

    pub fn update(&mut self, mut dt: f32, input: Pad) {
        let (vx0, vx1) = self.ambient_span();
        self.cull_l = vx0 - 16.0;
        self.cull_r = vx1 + 16.0;

        // Hit stop: the world crawls for a few frames when something big dies.
        if self.hitstop > 0.0 {
            self.hitstop -= dt;
            dt *= 0.15;
        }
        self.shake = (self.shake - dt * 18.0).max(0.0);
        self.bombing = (self.bombing - dt).max(0.0);
        self.bomb_flash = (self.bomb_flash - dt * 1.6).max(0.0);
        self.combo_t -= dt;
        if self.combo_t <= 0.0 {
            self.combo = 0;
        }
        self.bg_y += 52.0 * dt;
        self.time += dt;
        self.warning_t = (self.warning_t - dt).max(0.0);
        self.ready_t = (self.ready_t - dt).max(0.0);
        self.unwrap_t = (self.unwrap_t - dt).max(0.0);
        self.unsafe_t = (self.unsafe_t - dt).max(0.0);
        self.tokio_t = (self.tokio_t - dt).max(0.0);
        self.muzzle = (self.muzzle - dt).max(0.0);
        self.flash = (self.flash - dt * 2.8).max(0.0);
        self.agent_boost = (self.agent_boost - dt).max(0.0);

        self.update_recruit_fx(dt);
        self.update_ambient(dt, vx0, vx1);
        self.run_script();

        if self.clear {
            self.clear_t -= dt;
            // The stage is won; whatever is left on screen quietly gives up.
            for e in self.enemies.iter_mut() {
                if !e.dead && !e.boss {
                    e.hp -= dt * 20.0;
                }
            }
        }

        self.update_player(dt, input);
        self.update_agents(dt);
        self.update_pbullets(dt);
        self.update_enemies(dt);
        self.pbullets_vs_enemies();
        self.update_ebullets(dt);
        self.body_collisions();
        self.update_pickups(dt);
        self.update_token_items(dt);
        self.update_particles(dt);
    }

    fn update_recruit_fx(&mut self, dt: f32) {
        let Some(mut fx) = self.recruit_fx.clone() else {
            return;
        };
        fx.t += dt;
        let u = fx.t / fx.life;
        if u < 0.55 {
            let spin = fx.t * 14.0;
            for i in 0..3 {
                let a = spin + i as f32 * 2.094;
                let rad = 18.0 + ease_cos(u / 0.55) * 42.0;
                self.spark(
                    fx.x + a.cos() * rad,
                    fx.y + a.sin() * rad * 0.7,
                    a.cos() * 30.0,
                    a.sin() * 30.0,
                    fx.col,
                    0.28,
                    2.2,
                    true,
                );
            }
            if (fx.t * 10.0).floor() != ((fx.t - dt) * 10.0).floor() {
                self.pop_ring(fx.x, fx.y, fx.col, 0.28);
            }
        }
        self.recruit_fx = if fx.t >= fx.life { None } else { Some(fx) };
    }

    /// Stars, speed streaks, shopfronts, clouds, expanding rings — everything
    /// that decorates the scroll without being able to hurt anybody.
    fn update_ambient(&mut self, dt: f32, vx0: f32, vx1: f32) {
        let span = vx1 - vx0;
        let streak_cap = (18.0 * span / GW) as usize;
        if self.streaks.len() < streak_cap && self.rng.chance(dt * 22.0 * span / GW) {
            let x = self.rng.int(vx0 as i32, vx1 as i32 - 1) as f32;
            let len = 8.0 + self.rng.f() * 18.0;
            let v = 180.0 + self.rng.f() * 220.0;
            let a = 0.12 + self.rng.f() * 0.18;
            self.streaks.push(Streak { x, y: -8.0, len, v, a });
        }
        for s in self.streaks.iter_mut() {
            s.y += s.v * dt;
        }
        self.streaks.retain(|s| s.y <= 270.0);

        for r in self.rings.iter_mut() {
            r.t += dt;
            r.r += dt * 95.0;
        }
        self.rings.retain(|r| r.t < r.life);

        if let Some(mut r) = self.own_ring {
            r.t += dt;
            r.r = 8.0 + r.t * 180.0;
            self.own_ring = if r.t >= r.life { None } else { Some(r) };
        }

        self.prop_t += dt;
        let street = self.stage == 1;
        let gap = if street { 1.25 } else { 2.8 };
        let cap = if street { 6 } else { 4 };
        if self.prop_t > gap && !self.over && self.scenery.len() < cap {
            self.prop_t = 0.0;
            if street {
                // Causeway Bay at street level: shopfronts, alternating sides.
                self.shop_i += 1;
                let (art, w, h, _) = SHOP_PROPS[((self.shop_i - 1) % 7) as usize];
                let left = self.shop_i % 2 == 1;
                let x = if left { 36.0 } else { 156.0 };
                let sc = 0.7 + self.rng.f() * 0.2;
                let vy = 34.0 + self.rng.f() * 18.0;
                self.scenery.push(Prop { art, x, y: -32.0, w, h, vy, sc, a: 0.95 });
            } else {
                let x = (vx0 + 24.0) + self.rng.f() * (span - 48.0);
                let sc = 0.32 + self.rng.f() * 0.28;
                let vy = 82.0 + self.rng.f() * 28.0;
                let a = 0.16 + self.rng.f() * 0.16;
                let cloud = Prop { art: Sprite::Cloud, x, y: -16.0, w: 48.0, h: 22.0, vy, sc, a };
                self.scenery.push(cloud);
            }
        }

        // Agent badges rain down on their own, more often when you are alone.
        self.token_rain_t += dt;
        let rain_gap =
            if self.agents.is_empty() { self.token_rain_empty } else { self.token_rain_busy };
        if self.ready_t <= 0.0
            && !self.over
            && !self.clear
            && self.token_items.len() < 2
            && self.token_rain_t >= rain_gap
        {
            self.token_rain_t = 0.0;
            self.agent_drop_i += 1;
            let lane = [52.0, 96.0, 140.0][((self.agent_drop_i - 1) % 3) as usize];
            let want = AGENTS
                .iter()
                .copied()
                .find(|id| !self.has_agent(*id))
                .unwrap_or(AGENTS[(self.agent_drop_i % 3) as usize]);
            self.spawn_token_item(want, lane, -24.0);
        }

        for s in self.scenery.iter_mut() {
            s.y += s.vy * dt;
        }
        self.scenery.retain(|s| s.y <= 300.0);

        for g in self.ghosts.iter_mut() {
            g.t -= dt;
        }
        self.ghosts.retain(|g| g.t > 0.0);

        for st in self.stars.iter_mut() {
            st.y += st.v * dt;
        }
        for i in 0..self.stars.len() {
            if self.stars[i].y > GH {
                self.stars[i].y -= GH;
                self.stars[i].x = self.rng.int(vx0 as i32, vx1 as i32 - 1) as f32;
            }
        }
    }

    fn run_script(&mut self) {
        if self.clear || self.over {
            return;
        }
        while self.event_i < self.script.len() && self.time >= self.script[self.event_i].t {
            let acts = self.script[self.event_i].acts;
            self.event_i += 1;
            for act in acts {
                self.run_act(*act);
            }
        }
    }

    fn run_act(&mut self, act: Act) {
        match act {
            Act::Phase(name) => self.phase_name = name,
            Act::Spawn { kind, x, y, o } => {
                self.spawn(kind, x, y, o);
            }
            Act::SpawnV { kind, n, cx, y, gap } => self.spawn_v(kind, n, cx, y, gap),
            Act::SpawnLine { kind, n, x0, x1, y, path } => {
                self.spawn_line(kind, n, x0, x1, y, path)
            }
            Act::Drop { x, y, item } => self.drop_item(x, y, item),
            Act::Warn { secs, lines, double } => {
                self.warning_t = secs;
                self.boss_warn = lines;
                self.cues.sfx(Sfx::Warn);
                if double {
                    self.cues.sfx(Sfx::Warn2);
                }
            }
            Act::BossMusic => self.cues.music(Music::Boss),
            Act::Shake(n) => self.shake = n,
        }
    }

    fn update_player(&mut self, dt: f32, input: Pad) {
        if self.player.dying {
            self.player.die_t -= dt;
            if input.bomb {
                self.do_bomb();
            } else if self.player.die_t <= 0.0 {
                self.really_die();
            }
        }

        if self.player.alive && !self.over && !self.clear {
            // Holding the shot slows the ship: that is how you thread bullets.
            let spd = if input.shoot { 108.0 } else { 138.0 };
            let mut dx = 0.0;
            let mut dy = 0.0;
            if input.left {
                dx -= 1.0;
            }
            if input.right {
                dx += 1.0;
            }
            if input.up {
                dy -= 1.0;
            }
            if input.down {
                dy += 1.0;
            }
            if dx != 0.0 && dy != 0.0 {
                dx *= 0.707;
                dy *= 0.707;
            }
            let (x0, x1) = self.play_bounds();
            {
                let p = &mut self.player;
                p.x = (p.x + dx * spd * dt).clamp(x0.min(x1), x1.max(x0));
                p.y = (p.y + dy * spd * dt).clamp(18.0, 242.0);
                p.inv = (p.inv - dt).max(0.0);
                p.flash += dt;
                p.fire_t -= dt;
            }
            let (px, py) = (self.player.x, self.player.y);
            if (self.time * 40.0).floor() != ((self.time - dt) * 40.0).floor() {
                self.ghosts.push(Ghost { x: px, y: py, t: 0.16 });
            }
            let jitter = (self.rng.f() - 0.5) * 12.0;
            self.spark(px, py + 16.0, jitter, 50.0, palette::CYAN, 0.18, 1.2, false);

            if input.shoot && self.player.fire_t <= 0.0 && self.ready_t <= 0.0 {
                let mut interval = if self.power >= 4 { 0.055 } else { 0.07 };
                if self.agent_boost > 0.0 {
                    interval *= 0.62;
                }
                self.player.fire_t = interval;
                self.fire_player();
            }
            if self.tokio_t > 0.0 && self.player.fire_t > 0.0 {
                self.tokio_fire -= dt;
                if self.tokio_fire <= 0.0 && input.shoot {
                    self.tokio_fire = 0.2;
                    self.fire_tokio(px, py - 12.0);
                }
            }
            if input.bomb {
                self.do_bomb();
            }
        }

        // BORROWCK: at full power the held shot opens a lane that shreds
        // bullets and chips whatever is standing in it.
        self.pbeam = if self.player.alive
            && self.power >= 4
            && input.shoot
            && self.ready_t <= 0.0
            && !self.clear
            && !self.over
        {
            Some(self.player.x)
        } else {
            None
        };
    }

    fn update_pbullets(&mut self, dt: f32) {
        for i in (0..self.pbullets.len()).rev() {
            let b = self.pbullets[i].clone();
            if b.homing {
                if let Some((tx, ty)) = self.nearest_enemy(b.x, b.y) {
                    let a = (ty - b.y).atan2(tx - b.x);
                    let mut spd = (b.vx * b.vx + b.vy * b.vy).sqrt();
                    if spd < 80.0 {
                        spd = 240.0;
                    }
                    let (nx, ny) = (a.cos() * spd, a.sin() * spd);
                    let k = (dt * 5.0).min(1.0);
                    let b = &mut self.pbullets[i];
                    b.vx += (nx - b.vx) * k;
                    b.vy += (ny - b.vy) * k;
                }
            }
            let (x, y, vx, vy, life, col, agent, trail) = {
                let b = &mut self.pbullets[i];
                b.x += b.vx * dt;
                b.y += b.vy * dt;
                b.life -= dt;
                (b.x, b.y, b.vx, b.vy, b.life, b.col, b.agent, b.trail)
            };
            if trail {
                self.spark(
                    x,
                    y + 4.0,
                    -vx * 0.05,
                    -vy * 0.08,
                    col.unwrap_or(palette::CYAN),
                    0.14,
                    if agent { 1.0 } else { 1.6 },
                    false,
                );
            }
            if y < -12.0 || life <= 0.0 || x < self.cull_l || x > self.cull_r {
                self.pbullets.remove(i);
            }
        }
    }

    fn update_enemies(&mut self, dt: f32) {
        let py = self.player.y;
        for i in (0..self.enemies.len()).rev() {
            if self.enemies[i].dead {
                self.enemies.remove(i);
                continue;
            }
            if self.enemies[i].boss {
                self.update_boss(i, dt);
            } else {
                self.update_enemy(i, dt);
            }
            if i >= self.enemies.len() {
                continue;
            }

            if let Some(beam) = self.pbeam {
                let e = self.enemies[i];
                if e.visible && e.y < py && (e.x - beam).abs() < e.r * 0.55 {
                    self.enemies[i].hp -= 14.0 * dt;
                    if self.rng.chance(dt * 8.0) {
                        self.burst(e.x, e.y, 1, palette::CYAN, 36.0);
                    }
                }
            }

            let e = self.enemies[i];
            if e.hp <= 0.0 {
                self.kill_enemy(i, false);
            } else if !e.boss
                && (e.y > 276.0
                    || e.y < -60.0
                    || e.x < self.cull_l - 34.0
                    || e.x > self.cull_r + 34.0)
            {
                self.enemies.remove(i);
            }
        }
    }

    fn pbullets_vs_enemies(&mut self) {
        for i in (0..self.pbullets.len()).rev() {
            let (bx, by, br, dmg, pierce, col, agent) = {
                let b = &self.pbullets[i];
                (b.x, b.y, b.r, b.dmg, b.pierce, b.col, b.agent)
            };
            let mut hit = false;
            for j in 0..self.enemies.len() {
                let e = self.enemies[j];
                if e.dead || !e.visible {
                    continue;
                }
                let reach = e.r + br;
                if dist2(bx, by, e.x, e.y) >= reach * reach {
                    continue;
                }
                if !self.pbullets[i].hits.contains(&e.id) {
                    self.pbullets[i].hits.push(e.id);
                    self.enemies[j].hp -= dmg;
                    self.enemies[j].flash = 0.1;
                    hit = true;
                    self.burst(bx, by, 3, col.unwrap_or(palette::CYAN), 50.0);
                    self.cues.sfx(Sfx::Hit);
                    if e.boss {
                        self.enemies[j].hp_flash = 0.22;
                        self.shake = self.shake.max(1.8);
                    }
                    if self.enemies[j].hp <= 0.0 {
                        self.hitstop = 0.03;
                        if agent {
                            self.agent_pop(e.x, e.y, col.unwrap_or(palette::CYAN));
                        }
                    }
                }
                if !pierce {
                    break;
                }
            }
            if hit && !pierce {
                self.pbullets.remove(i);
            }
        }
    }

    fn update_ebullets(&mut self, dt: f32) {
        let p = self.player;
        for i in (0..self.ebullets.len()).rev() {
            let b = {
                let b = &mut self.ebullets[i];
                b.x += b.vx * dt;
                b.y += b.vy * dt;
                b.life -= dt;
                *b
            };
            if b.y < -16.0 || b.y > 272.0 || b.x < self.cull_l || b.x > self.cull_r || b.life <= 0.0
            {
                self.ebullets.remove(i);
            } else if self.pbeam.is_some_and(|x| (b.x - x).abs() < 3.4) && b.y < p.y {
                self.ebullets.remove(i);
                self.add_score(4.0);
                self.burst(b.x, b.y, 1, palette::CYAN, 30.0);
            } else if self.bombing > 0.0 {
                self.ebullets.remove(i);
                self.burst(b.x, b.y, 2, palette::YELLOW, 40.0);
            } else if !self.agents.is_empty() && self.try_agent_eat(b.x, b.y) {
                self.ebullets.remove(i);
            } else if p.alive {
                let d2 = dist2(b.x, b.y, p.x, p.y);
                // Grazing: points for letting one past close enough to feel it.
                if d2 < 64.0 && !b.grazed {
                    self.ebullets[i].grazed = true;
                    self.graze += 1;
                    self.add_score(10.0);
                    self.cues.sfx(Sfx::Graze);
                    self.burst(p.x, p.y, 2, palette::WHITE, 40.0);
                }
                // The hitbox is the one cyan pixel, not the ship.
                let reach = 2.0 + b.r;
                if d2 < reach * reach {
                    self.ebullets.remove(i);
                    self.hurt_player();
                }
            }
        }
    }

    fn body_collisions(&mut self) {
        let p = self.player;
        if !p.alive || p.inv > 0.0 {
            return;
        }
        let touched = self.enemies.iter().any(|e| {
            if e.dead || !e.visible {
                return false;
            }
            let reach = e.r * 0.55 + 2.0;
            dist2(p.x, p.y, e.x, e.y) < reach * reach
        });
        if touched {
            self.hurt_player();
        }
    }

    fn update_pickups(&mut self, dt: f32) {
        let p = self.player;
        for i in (0..self.pickups.len()).rev() {
            let u = {
                let pull = if self.bombing > 0.0 { 8.0 } else { 3.2 };
                let u = &mut self.pickups[i];
                u.t += dt;
                u.y += u.vy * dt;
                u.vy = (u.vy + 14.0 * dt).min(42.0);
                u.x += (u.t * 4.0).sin() * 8.0 * dt;
                if p.alive {
                    // Capsules drift toward the ship, and a bomb hoovers them.
                    u.x += (p.x - u.x) * dt * pull;
                    u.y += (p.y - u.y) * dt * pull;
                }
                *u
            };
            if p.alive && dist2(u.x, u.y, p.x, p.y) < 26.0 * 26.0 {
                self.collect(u);
                self.pickups.remove(i);
            } else if u.y > 270.0 {
                self.pickups.remove(i);
            }
        }
    }

    fn collect(&mut self, u: Pickup) {
        match u.kind {
            Item::Power => {
                if self.power < 4 {
                    self.power += 1;
                    let name = SKILLS[(self.power - 1) as usize];
                    self.floater(u.x, u.y, name, palette::CYAN);
                } else {
                    self.add_score(1000.0);
                    self.floater(u.x, u.y, "1000", palette::YELLOW);
                }
                self.cues.sfx(Sfx::Power);
            }
            Item::Bomb => {
                self.bombs = (self.bombs + 1).min(6);
                self.floater(u.x, u.y, "OWN BOMB", palette::RUST);
                self.cues.sfx(Sfx::Power);
            }
            Item::Score => {
                self.add_score(2000.0);
                self.floater(u.x, u.y, "2000", palette::YELLOW);
                self.cues.sfx(Sfx::Coin);
            }
            Item::OneUp => {
                self.lives = (self.lives + 1).min(9);
                self.floater(u.x, u.y, "1UP", palette::YELLOW);
                self.cues.sfx(Sfx::OneUp);
            }
            Item::Unwrap => {
                self.unwrap_t = 8.0;
                self.floater(u.x, u.y, "UNWRAP", palette::YELLOW);
                self.cues.sfx(Sfx::Power);
            }
            Item::Tokio => {
                self.tokio_t = 10.0;
                self.floater(u.x, u.y, "TOKIO", palette::RUST);
                self.cues.sfx(Sfx::Power);
            }
            Item::Unsafe => {
                self.unsafe_t = 6.0;
                self.floater(u.x, u.y, "UNSAFE", palette::LRED);
                self.cues.sfx(Sfx::Power);
            }
            Item::Claude | Item::Grok | Item::Codex | Item::Agent => {
                let id = u.kind.agent();
                self.recruit_agent(id, false);
            }
        }
    }

    fn update_token_items(&mut self, dt: f32) {
        let p = self.player;
        for i in (0..self.token_items.len()).rev() {
            let u = {
                let u = &mut self.token_items[i];
                u.t += dt;
                u.y += u.vy * dt;
                *u
            };
            if self.rng.chance(dt * 28.0) {
                let vx = (self.rng.f() - 0.5) * 30.0;
                self.spark(u.x, u.y, vx, -40.0, u.id.def().col, 0.28, 2.0, true);
            }
            if p.alive && dist2(u.x, u.y, p.x, p.y) < 26.0 * 26.0 {
                self.token_items.remove(i);
                self.recruit_agent(Some(u.id), false);
            } else if u.y > 280.0 {
                self.token_items.remove(i);
            }
        }
    }

    fn update_particles(&mut self, dt: f32) {
        for q in self.particles.iter_mut() {
            q.t += dt;
            q.x += q.vx * dt;
            q.y += q.vy * dt;
            q.vy += 40.0 * dt;
        }
        self.particles.retain(|q| q.t < q.life);

        for q in self.booms.iter_mut() {
            q.t += dt;
        }
        self.booms.retain(|q| q.t <= 0.35);

        for q in self.floaters.iter_mut() {
            q.t -= dt;
            q.y -= 18.0 * dt;
        }
        self.floaters.retain(|q| q.t > 0.0);
    }

    // ------------------------------------------------------------------- draw

    pub fn draw(&self, p: &mut Painter) {
        self.draw_scenery(p);
        self.draw_token_items(p);
        self.draw_pickups(p);
        self.draw_enemies(p);
        self.draw_beams(p);
        self.draw_pbullets(p);
        self.draw_ship(p);
        self.draw_ebullets(p);
        self.draw_effects(p);
        self.draw_overlays(p);
    }

    fn draw_scenery(&self, p: &mut Painter) {
        for s in &self.streaks {
            p.rect(s.x.floor(), s.y.floor(), 1.0, s.len, palette::a(palette::WHITE, s.a));
        }
        for s in &self.scenery {
            let bob = (self.time * 3.2 + s.x * 0.04).sin() * 1.4;
            p.sprite(
                s.art,
                s.x,
                s.y + bob,
                0.0,
                s.sc,
                s.sc,
                s.w / 2.0,
                s.h / 2.0,
                palette::a(palette::WHITE, s.a),
            );
        }
    }

    /// A falling agent badge: a shaft of light, a halo, spokes, and the name.
    fn draw_token_items(&self, p: &mut Painter) {
        for u in &self.token_items {
            let def = u.id.def();
            let col = def.col;
            let pulse = 0.5 + 0.5 * ease_cos((u.t * 6.0).sin() * 0.5 + 0.5);
            let y = u.y + (u.t * 3.4).sin() * 3.0;
            p.rect(u.x - 5.0, y - 56.0, 10.0, 62.0, palette::a(col, 0.16 + 0.18 * pulse));
            p.rect(u.x - 1.0, y - 60.0, 2.0, 66.0, palette::a(palette::WHITE, 0.28 + 0.4 * pulse));
            p.circle(u.x, y + 8.0, 14.0 + pulse * 6.0, palette::a(col, 0.35 * pulse));
            p.ring(
                u.x,
                y,
                16.0 + (u.t * 8.0).sin() * 3.0,
                1.0,
                palette::a(palette::WHITE, 0.22 * pulse),
            );
            p.ring(u.x, y, 22.0 + pulse * 4.0, 1.0, palette::a(palette::WHITE, 0.22 * pulse));
            for i in 0..6 {
                let a = u.t * 2.2 + i as f32 * std::f32::consts::FRAC_PI_3;
                let len = 10.0 + pulse * 6.0;
                p.rect(
                    u.x + a.cos() * 8.0 - 1.0,
                    y + a.sin() * 8.0 - 1.0,
                    2.0,
                    len,
                    palette::a(col, 0.55 * pulse),
                );
            }
            let sc = 1.45 + pulse * 0.25;
            p.sprite(
                def.art,
                u.x.floor() + 1.0,
                y.floor() + 2.0,
                0.0,
                sc,
                sc,
                10.0,
                10.0,
                palette::a(col, 0.55),
            );
            p.sprite(def.art, u.x.floor(), y.floor(), 0.0, sc, sc, 10.0, 10.0, palette::WHITE);
            let tw = font::text_width(def.name, 1.0);
            p.text(
                def.name,
                (u.x - tw / 2.0).floor(),
                (y - 28.0).floor(),
                palette::a(col, 0.9),
                1.0,
            );
        }
    }

    fn draw_pickups(&self, p: &mut Painter) {
        for u in &self.pickups {
            p.sprite(Sprite::Pickup, u.x, u.y, u.t * 2.0, 1.0, 1.0, 9.0, 9.0, palette::WHITE);
            let label = u.kind.label();
            let w = font::text_width(label, 1.0);
            p.text(label, u.x - w / 2.0, u.y - 3.0, u.kind.col(), 1.0);
        }
    }

    fn draw_enemies(&self, p: &mut Painter) {
        for e in &self.enemies {
            if e.dead {
                continue;
            }
            let (ox, oy) = (e.w / 2.0, e.h / 2.0);
            // A little motion in the pose, per species.
            let rot = match e.kind {
                Kind::Moth | Kind::Heisen => (e.t * 8.0).sin() * 0.15,
                Kind::Worm => (e.t * 5.0).sin() * 0.2,
                Kind::Leak => (e.t * 2.0).sin() * 0.08,
                Kind::Infloop => e.t * 5.0,
                Kind::Clippy => (e.t * 3.0).sin() * 0.25,
                Kind::Offby1 => 0.12,
                _ => 0.0,
            };
            p.sprite(
                e.art,
                e.x.floor() + 1.0,
                e.y.floor() + 2.0,
                rot,
                1.0,
                1.0,
                ox,
                oy,
                palette::a(palette::BLACK, 0.35),
            );
            let tint = if e.flash > 0.0 { palette::WHITE } else { e.tint };
            p.sprite(e.art, e.x.floor(), e.y.floor(), rot, 1.0, 1.0, ox, oy, tint);

            if (e.boss || e.midboss) && e.maxhp > 0.0 {
                let bw = if e.boss { 56.0 } else { 36.0 };
                let bh = if e.boss { 5.0 } else { 3.0 };
                let bx = (e.x - bw / 2.0).floor();
                let by = (e.y - oy - 8.0).floor();
                let u = (e.hp / e.maxhp).clamp(0.0, 1.0);
                p.rect(bx - 1.0, by - 1.0, bw + 2.0, bh + 2.0, palette::a(palette::BLACK, 0.8));
                p.rect(bx, by, bw * u, bh, palette::DRED);
                if e.hp_flash > 0.0 {
                    p.rect(bx, by, bw * u, bh, palette::a(palette::WHITE, 0.75));
                } else {
                    p.rect(bx, by, bw * u, 2.0, palette::YELLOW);
                }
            }
        }
    }

    fn draw_beams(&self, p: &mut Painter) {
        if let Some(x) = self.pbeam {
            let py = self.player.y - 16.0;
            p.rect(
                x - 3.0,
                0.0,
                6.0,
                py,
                [0.35, 0.95, 1.0, 0.22 + 0.08 * (self.time * 28.0).sin()],
            );
            p.rect(x - 1.0, 0.0, 2.0, py, [0.85, 1.0, 1.0, 0.85]);
        }
        if let Some(l) = self.laser {
            match l.state {
                LaserState::Aim => {
                    let a = 0.35 + 0.25 * (self.time * 20.0).sin();
                    p.rect(l.x - 5.0, 0.0, 10.0, GH, [1.0, 0.85, 0.3, a]);
                }
                LaserState::Fire => {
                    p.rect(l.x - 8.0, 0.0, 16.0, GH, [1.0, 0.3, 0.2, 0.75]);
                    p.rect(l.x - 2.0, 0.0, 4.0, GH, [1.0, 1.0, 0.8, 0.95]);
                }
            }
        }
    }

    fn draw_pbullets(&self, p: &mut Painter) {
        for b in &self.pbullets {
            if b.missile {
                let ang = b.vy.atan2(b.vx) + std::f32::consts::FRAC_PI_2;
                p.sprite(Sprite::Tokio, b.x, b.y, ang, 1.0, 1.0, 6.0, 9.0, palette::WHITE);
                continue;
            }
            let tint = match (b.col, b.pierce) {
                (Some(c), _) => {
                    p.rect(b.x.floor() - 3.0, b.y.floor() - 1.0, 6.0, 13.0, palette::a(c, 0.55));
                    c
                }
                (None, true) => [1.0, 0.95, 0.4, 1.0],
                (None, false) => palette::WHITE,
            };
            let sc = if b.pierce { (1.25, 1.4) } else { (1.12, 1.12) };
            p.sprite(Sprite::Pbullet, b.x, b.y, 0.0, sc.0, sc.1, 5.0, 7.0, tint);
            p.rect(b.x.floor() - 1.0, b.y.floor() - 3.0, 3.0, 7.0, palette::WHITE);
        }
    }

    fn draw_ship(&self, p: &mut Painter) {
        for g in &self.ghosts {
            let a = (g.t / 0.16).max(0.0) * 0.35;
            p.sprite(
                Sprite::Player,
                g.x.floor(),
                g.y.floor(),
                0.0,
                1.0,
                1.0,
                20.0,
                20.0,
                [0.4, 0.9, 1.0, a],
            );
        }
        for ag in &self.agents {
            let pop = ag.pop;
            let (sc, rot) = if pop < 1.0 {
                // Arriving: it spins up out of nothing.
                (0.18 + ease_cos(pop) * 1.22, (1.0 - ease_cos(pop)) * 6.4)
            } else {
                let settle = (1.35 - pop).max(0.0);
                (1.0 + ((pop - 1.0) * 14.0).sin() * 0.1 * settle, 0.0)
            };
            let sc = if ag.chomp > 0.0 { sc * (1.12 + ag.chomp * 0.8) } else { sc };
            let token_u = (ag.tokens / ag.def.token_max).clamp(0.0, 1.0);
            // Nearly out of budget: the agent starts to flicker.
            let flicker = if token_u < 0.25 && (self.time * 14.0).floor() as i32 % 2 != 0 {
                0.35
            } else {
                1.0
            };
            let spin = if ag.id == AgentId::Grok { 16.0 } else { 8.0 };
            let mut aura = 7.0 + (self.time * spin).sin() * 2.0;
            if self.agent_boost > 0.0 {
                aura += 4.0;
            }
            let col = ag.def.col;
            p.circle(ag.x, ag.y, aura, palette::a(col, (0.28 + token_u * 0.25) * flicker));
            p.sprite(
                ag.def.art,
                ag.x.floor(),
                ag.y.floor(),
                rot,
                sc,
                sc,
                10.0,
                10.0,
                palette::a(col, flicker),
            );
            p.rect(ag.x - 7.0, ag.y + 10.0, 14.0, 3.0, palette::a(palette::BLACK, 0.65 * flicker));
            p.rect(ag.x - 7.0, ag.y + 10.0, 14.0 * token_u, 3.0, palette::a(col, 0.95 * flicker));
        }

        let p0 = self.player;
        if !p0.alive {
            return;
        }
        let show = !(p0.inv > 0.0 && (p0.flash * 16.0).floor() as i32 % 2 == 0);
        if show {
            let tint = if self.unsafe_t > 0.0 {
                [1.0, 0.55, 0.45, 1.0]
            } else if self.unwrap_t > 0.0 {
                [1.0, 1.0, 0.72, 1.0]
            } else if self.agent_boost > 0.0 {
                let pulse = 0.55 + 0.45 * ease_cos((self.time * 8.0).sin() * 0.5 + 0.5);
                [0.75 + 0.25 * pulse, 0.95, 1.0, 1.0]
            } else {
                palette::WHITE
            };
            p.sprite(Sprite::Player, p0.x.floor(), p0.y.floor(), 0.0, 1.0, 1.0, 20.0, 20.0, tint);
            if self.agent_boost > 0.0 {
                let pulse = 0.2 + 0.25 * ease_cos((self.time * 12.0).sin() * 0.5 + 0.5);
                p.ring(
                    p0.x,
                    p0.y,
                    16.0 + (self.time * 9.0).sin() * 3.0,
                    1.0,
                    [0.45, 0.95, 1.0, pulse],
                );
                p.ring(p0.x, p0.y, 11.0, 1.0, palette::a(palette::WHITE, pulse * 0.7));
            }
            if self.muzzle > 0.0 {
                let a = self.muzzle / 0.08;
                p.rect(p0.x - 3.0, p0.y - 22.0, 6.0, 8.0, [1.0, 1.0, 0.7, a]);
                p.rect(p0.x - 1.0, p0.y - 26.0, 2.0, 6.0, [1.0, 0.55, 0.25, a]);
            }
            let fl = 4.0 + ((self.time * 30.0).sin() * 2.0).floor();
            p.rect(p0.x - 1.0, p0.y + 16.0, 2.0, fl, palette::CYAN);
            p.rect(p0.x - 1.0, p0.y + 16.0, 2.0, (fl - 2.0).max(1.0), palette::RUST);
        }
        // The hitbox, always visible: the ship is decoration, this pixel is you.
        p.rect(p0.x.floor() - 1.0, p0.y.floor() - 1.0, 3.0, 3.0, palette::a(palette::WHITE, 0.95));
        p.rect(p0.x.floor(), p0.y.floor(), 1.0, 1.0, palette::CYAN);
    }

    fn draw_ebullets(&self, p: &mut Painter) {
        for b in &self.ebullets {
            let c = b.col.unwrap_or(palette::MAGENTA);
            p.rect(b.x.floor() - 3.0, b.y.floor() - 3.0, 6.0, 6.0, c);
            p.rect(b.x.floor() - 1.0, b.y.floor() - 1.0, 3.0, 3.0, palette::WHITE);
            let dim = [c[0], c[1] * 0.5, c[2] * 0.5, 1.0];
            p.rect(b.x.floor() - 4.0, b.y.floor(), 1.0, 1.0, dim);
            p.rect(b.x.floor() + 3.0, b.y.floor(), 1.0, 1.0, dim);
        }
    }

    fn draw_effects(&self, p: &mut Painter) {
        for q in &self.booms {
            let sc = q.scale * (0.6 + q.t * 2.2);
            let a = 1.0 - q.t / 0.35;
            p.sprite(
                Sprite::Boom,
                q.x,
                q.y,
                q.t * 4.0,
                sc,
                sc,
                24.0,
                24.0,
                palette::a(palette::WHITE, a),
            );
        }
        for q in &self.rings {
            let a = ease_cos(1.0 - q.t / q.life);
            p.ring(q.x, q.y, q.r, 1.0, palette::a(q.col, 0.85 * a));
            p.ring(q.x, q.y, q.r * 0.62, 1.0, palette::a(palette::WHITE, 0.45 * a));
        }
        for q in &self.particles {
            let a = ease_cos((1.0 - q.t / q.life).max(0.0));
            if q.glint {
                let s = (q.r * (0.45 + a)).max(1.0);
                p.rect(q.x.floor() - s, q.y.floor(), s * 2.0 + 1.0, 1.0, palette::a(q.col, a));
                p.rect(q.x.floor(), q.y.floor() - s, 1.0, s * 2.0 + 1.0, palette::a(q.col, a));
                p.rect(q.x.floor(), q.y.floor(), 1.0, 1.0, palette::a(palette::WHITE, a));
            } else {
                let s = q.r.max(1.0);
                p.rect(q.x.floor(), q.y.floor(), s, s, palette::a(q.col, a));
            }
        }
        for q in &self.floaters {
            let w = font::text_width(&q.text, 1.0);
            p.text(&q.text, (q.x - w / 2.0).floor(), q.y.floor(), q.col, 1.0);
        }

        if self.bomb_flash > 0.0 {
            let (x, w) = p.view_span();
            p.rect(x, 0.0, w, GH, [1.0, 0.55, 0.25, self.bomb_flash * 0.32]);
        }
        if let Some(r) = self.own_ring {
            let a = 1.0 - r.t / r.life;
            p.ring(r.x, r.y, r.r, 1.0, [0.86, 0.40, 0.27, 0.55 * a]);
            p.ring(r.x, r.y, r.r * 0.72, 1.0, [1.0, 0.85, 0.35, 0.8 * a]);
            p.ring(r.x, r.y, r.r * 1.15, 1.0, [0.4, 0.9, 1.0, 0.35 * a]);
        }
    }

    /// In-playfield callouts: power state, READY, the boss klaxon, HP bars.
    fn draw_overlays(&self, p: &mut Painter) {
        let vx = p.view_left().floor();
        let vw = (p.view_right() - vx).max(80.0).floor();
        let boss_up = self.boss().is_some();

        if self.unwrap_t > 0.0 {
            p.text("UNWRAP", vx + 2.0, 20.0, palette::YELLOW, 1.0);
        } else if self.tokio_t > 0.0 {
            p.text("TOKIO", vx + 2.0, 20.0, palette::RUST, 1.0);
        } else if self.unsafe_t > 0.0 {
            p.text("UNSAFE", vx + 2.0, 20.0, palette::LRED, 1.0);
        } else if self.agent_boost > 0.0 {
            p.text("AI BOOST", vx + 2.0, 20.0, palette::CYAN, 1.0);
        }

        if self.ready_t > 0.0 && !self.over {
            p.center("READY", 104.0, palette::YELLOW, 1.0);
            p.center(&format!("STAGE {}", self.stage), 118.0, palette::CYAN, 1.0);
            let name =
                stage::NAMES.get((self.stage - 1).max(0) as usize).copied().unwrap_or("STAGE");
            p.center(name, 132.0, palette::WHITE, 1.0);
            p.center("Z SHOT   X BOMB", 148.0, palette::CYAN, 1.0);
        }

        if self.warning_t > 0.0 {
            let flash = (self.time * 8.0).floor() as i32 % 2 == 0;
            let hot = if flash { palette::LRED } else { palette::YELLOW };
            let cold = if flash { palette::YELLOW } else { palette::LRED };
            p.center(self.boss_warn[0], 100.0, hot, 1.0);
            p.center(self.boss_warn[1], 114.0, palette::WHITE, 1.0);
            p.center(self.boss_warn[2], 128.0, cold, 1.0);
        }

        if self.clear && self.clear_t > 0.0 {
            p.center("STAGE CLEAR", 104.0, palette::YELLOW, 1.0);
            let next = if self.stage < 3 {
                stage::NAMES.get(self.stage as usize).copied().unwrap_or("NEXT")
            } else {
                "PRINCESS PITCH"
            };
            p.center(next, 118.0, palette::CYAN, 1.0);
            p.center("BONUS 20000", 132.0, palette::WHITE, 1.0);
        }

        if self.combo >= 8 && !boss_up {
            p.text(&format!("COMBO {}", self.combo), vx + 2.0, 30.0, palette::YELLOW, 1.0);
        }

        let bar_x = vx + 8.0;
        let bar_w = vw - 16.0;
        if !boss_up {
            if let Some(m) = self.midboss() {
                let u = (m.hp / m.maxhp.max(1.0)).max(0.0);
                p.rect(bar_x, 16.0, bar_w, 6.0, palette::BLACK);
                p.rect(bar_x, 16.0, bar_w * u, 6.0, palette::RUST);
                p.rect(bar_x, 16.0, bar_w * u, 2.0, palette::YELLOW);
                p.text("OVERFLOW", bar_x, 24.0, palette::YELLOW, 1.0);
                let hp = format!("{}/{}", m.hp.ceil() as i64, m.maxhp.ceil() as i64);
                p.text_right(&hp, bar_x + bar_w, 24.0, palette::WHITE, 1.0);
            }
        }

        if let Some(b) = self.boss() {
            let maxhp = b.maxhp.max(1.0);
            let hp = b.hp.max(0.0);
            let u = hp / maxhp;
            p.rect(bar_x, 16.0, bar_w, 8.0, palette::BLACK);
            let fill =
                if b.hp_flash > 0.0 { palette::a(palette::WHITE, 0.9) } else { palette::DRED };
            p.rect(bar_x, 16.0, bar_w * u, 8.0, fill);
            p.rect(bar_x, 16.0, bar_w * u, 2.0, palette::YELLOW);
            let title: String = b.title.chars().take(12).collect();
            p.text(&title, bar_x, 26.0, palette::LRED, 1.0);
            let hp_txt = format!("{}/{}", hp.ceil() as i64, maxhp.ceil() as i64);
            p.text_right(&hp_txt, bar_x + bar_w, 26.0, palette::WHITE, 1.0);
            if !self.phase_name.is_empty() {
                p.text(self.phase_name, bar_x, 36.0, palette::YELLOW, 1.0);
            }
        }

        if let Some(fx) = &self.recruit_fx {
            let t = fx.t / fx.life;
            let fade = if t < 0.2 {
                ease_cos(t / 0.2)
            } else if t > 0.58 {
                1.0 - ease_cos((t - 0.58) / 0.42)
            } else {
                1.0
            };
            let rise = ((1.0 - ease_cos((t / 0.25).min(1.0))) * -10.0).floor();
            p.rect(fx.x - 6.0, 0.0, 12.0, GH, palette::a(fx.col, 0.28 * fade));
            p.rect(fx.x - 1.0, 0.0, 2.0, GH, palette::a(palette::WHITE, 0.18 * fade));
            let (bx0, bw) = p.view_span();
            p.rect(bx0, 86.0 + rise, bw, 38.0, palette::a(palette::BLACK, 0.55 * fade));
            let scale = if fx.name.len() <= 6 { 2.0 } else { 1.0 };
            let w = font::text_width(fx.name, scale);
            let x = (96.0 - w / 2.0).floor();
            p.text(fx.name, x + 1.0, 93.0 + rise, [0.0, 0.0, 0.0, fade], scale);
            p.text(fx.name, x, 92.0 + rise, palette::a(fx.col, fade), scale);
            let sw = font::text_width("ONLINE", 1.0);
            let sx = (96.0 - sw / 2.0).floor();
            p.text("ONLINE", sx + 1.0, 115.0 + rise, [0.0, 0.0, 0.0, fade], 1.0);
            p.text("ONLINE", sx, 114.0 + rise, palette::a(palette::WHITE, fade * 0.9), 1.0);
        }
    }
}

/// The optional half of a player shot.
#[derive(Clone, Copy)]
pub struct ShotOpts {
    pub r: f32,
    pub life: f32,
    pub pierce: bool,
    pub col: Option<Col>,
    pub agent: bool,
    pub trail: bool,
    pub homing: bool,
    pub missile: bool,
}

impl ShotOpts {
    pub const DEFAULT: ShotOpts = ShotOpts {
        r: 4.0,
        life: 1.2,
        pierce: false,
        col: None,
        agent: false,
        trail: true,
        homing: false,
        missile: false,
    };
}
