//! What a bug is: the stat tables from `src/world.lua`, as data.
//!
//! Kept apart from the simulation so the stage scripts can name enemies and
//! drops without reaching into the world.

use crate::palette::{self, Col};
use crate::sprites::Sprite;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Beetle,
    Moth,
    Spider,
    Worm,
    Nullptr,
    Leak,
    Overflow,
    Deadlock,
    Heisen,
    Offby1,
    Clippy,
    Lifetime,
    Infloop,
    Panic,
    BossOverflow,
    BossDeadlock,
    Boss,
}

/// How an enemy moves. `Boss` is inert here: bosses steer from their own
/// phase machine instead.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Path {
    #[default]
    Down,
    Sine,
    Zigzag,
    Swoop,
    Hover,
    Dive,
    Side,
    Teleport,
    Orbit,
    Boss,
}

/// A pickup. `Agent` is a wildcard the world resolves into `C`, `G` or `X` in
/// rotation, so no run showers you with the same assistant twice.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Item {
    Power,
    Bomb,
    Score,
    OneUp,
    Claude,
    Grok,
    Codex,
    Unwrap,
    Tokio,
    Unsafe,
    Agent,
}

impl Item {
    /// The letter stamped on the capsule.
    pub fn label(self) -> &'static str {
        match self {
            Item::Power => "P",
            Item::Bomb => "B",
            Item::Score => "S",
            Item::OneUp => "1",
            Item::Claude => "C",
            Item::Grok => "G",
            Item::Codex => "X",
            Item::Unwrap => "U",
            Item::Tokio => "T",
            Item::Unsafe => "N",
            Item::Agent => "A",
        }
    }

    pub fn col(self) -> Col {
        match self {
            Item::Power => palette::CYAN,
            Item::Bomb | Item::Tokio => palette::RUST,
            Item::OneUp => palette::LGREEN,
            Item::Claude => palette::CLAUDE,
            Item::Grok => palette::GROK,
            Item::Codex => palette::CODEX,
            Item::Unsafe => palette::LRED,
            _ => palette::YELLOW,
        }
    }

    /// The agent this capsule recruits, if it is one.
    pub fn agent(self) -> Option<AgentId> {
        match self {
            Item::Claude => Some(AgentId::Claude),
            Item::Grok => Some(AgentId::Grok),
            Item::Codex => Some(AgentId::Codex),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AgentId {
    Claude,
    Grok,
    Codex,
}

pub const AGENTS: [AgentId; 3] = [AgentId::Claude, AgentId::Grok, AgentId::Codex];

/// Claude aims, Grok sprays wide, Codex runs two clean streams.
///
/// They are shields before they are guns: a wide `eat_r`, slow weak shots.
/// Tokens burn like a vibe-coding budget and the agent leaves at zero.
#[derive(Clone, Copy, Debug)]
pub struct AgentDef {
    pub name: &'static str,
    pub art: Sprite,
    pub item: Item,
    pub rate: f32,
    pub col: Col,
    pub token_max: f32,
    pub drain: f32,
    pub shot_cost: f32,
    pub orbit: f32,
    pub spin: f32,
    pub eat_r: f32,
}

impl AgentId {
    pub fn def(self) -> AgentDef {
        match self {
            AgentId::Claude => AgentDef {
                name: "CLAUDE",
                art: Sprite::Claude,
                item: Item::Claude,
                rate: 1.25,
                col: palette::CLAUDE,
                token_max: 22.0,
                drain: 0.40,
                shot_cost: 0.04,
                orbit: 22.0,
                spin: 2.1,
                eat_r: 30.0,
            },
            AgentId::Grok => AgentDef {
                name: "GROK",
                art: Sprite::Grok,
                item: Item::Grok,
                rate: 1.05,
                col: palette::GROK,
                token_max: 18.0,
                drain: 0.45,
                shot_cost: 0.03,
                orbit: 36.0,
                spin: 4.4,
                eat_r: 38.0,
            },
            AgentId::Codex => AgentDef {
                name: "CODEX",
                art: Sprite::Codex,
                item: Item::Codex,
                rate: 1.15,
                col: palette::CODEX,
                token_max: 20.0,
                drain: 0.42,
                shot_cost: 0.03,
                orbit: 14.0,
                spin: 1.5,
                eat_r: 28.0,
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EnemyDef {
    pub hp: f32,
    pub r: f32,
    pub score: f32,
    pub speed: f32,
    pub fire: f32,
    pub w: f32,
    pub h: f32,
    pub art: Sprite,
    pub boss: bool,
    pub title: &'static str,
}

/// One row of the stat table. Every bug needs all of these, so the argument
/// list is the table's columns rather than a design to be tidied away.
#[allow(clippy::too_many_arguments)]
const fn bug(
    hp: f32,
    r: f32,
    score: f32,
    speed: f32,
    fire: f32,
    w: f32,
    h: f32,
    art: Sprite,
) -> EnemyDef {
    EnemyDef { hp, r, score, speed, fire, w, h, art, boss: false, title: "" }
}

impl Kind {
    pub fn def(self) -> EnemyDef {
        match self {
            Kind::Beetle => bug(1.0, 8.0, 100.0, 44.0, 0.0, 26.0, 26.0, Sprite::Beetle),
            Kind::Moth => bug(2.0, 10.0, 300.0, 38.0, 2.1, 32.0, 28.0, Sprite::Moth),
            Kind::Spider => bug(5.0, 11.0, 500.0, 24.0, 1.55, 30.0, 30.0, Sprite::Spider),
            Kind::Worm => bug(4.0, 10.0, 400.0, 34.0, 0.0, 28.0, 32.0, Sprite::Worm),
            Kind::Nullptr => bug(2.0, 9.0, 250.0, 56.0, 0.0, 24.0, 24.0, Sprite::Nullptr),
            Kind::Leak => bug(6.0, 12.0, 600.0, 20.0, 1.05, 28.0, 28.0, Sprite::Leak),
            Kind::Overflow => bug(8.0, 13.0, 800.0, 22.0, 1.55, 30.0, 32.0, Sprite::Overflow),
            Kind::Deadlock => bug(9.0, 13.0, 700.0, 18.0, 1.25, 32.0, 24.0, Sprite::Deadlock),
            // Heisenbug wears the moth's art and blinks in and out of it.
            Kind::Heisen => bug(3.0, 10.0, 450.0, 42.0, 1.7, 32.0, 28.0, Sprite::Moth),
            Kind::Offby1 => bug(3.0, 10.0, 350.0, 44.0, 1.55, 28.0, 26.0, Sprite::Offby1),
            Kind::Clippy => bug(6.0, 12.0, 650.0, 22.0, 1.8, 30.0, 30.0, Sprite::Clippy),
            Kind::Lifetime => bug(3.0, 10.0, 500.0, 32.0, 1.55, 28.0, 32.0, Sprite::Lifetime),
            Kind::Infloop => bug(8.0, 13.0, 750.0, 26.0, 1.2, 32.0, 32.0, Sprite::Infloop),
            Kind::Panic => bug(1.0, 11.0, 400.0, 62.0, 0.0, 28.0, 28.0, Sprite::Panic),
            Kind::BossOverflow => EnemyDef {
                hp: 420.0,
                r: 32.0,
                score: 40000.0,
                speed: 14.0,
                fire: 0.9,
                w: 88.0,
                h: 88.0,
                art: Sprite::BossOverflow,
                boss: true,
                title: "STACK OVERFLOW",
            },
            Kind::BossDeadlock => EnemyDef {
                hp: 560.0,
                r: 34.0,
                score: 55000.0,
                speed: 12.0,
                fire: 1.0,
                w: 90.0,
                h: 80.0,
                art: Sprite::BossDeadlock,
                boss: true,
                title: "THREAD DEADLOCK",
            },
            Kind::Boss => EnemyDef {
                hp: 720.0,
                r: 36.0,
                score: 80000.0,
                speed: 14.0,
                fire: 0.85,
                w: 104.0,
                h: 92.0,
                art: Sprite::King,
                boss: true,
                title: "SEGFAULT",
            },
        }
    }

    /// The three attack names a boss announces as its health drops.
    pub fn phase_names(self) -> [&'static str; 3] {
        match self {
            Kind::BossOverflow => ["STACK WALK", "FRAME SMASH", "OVERFLOW"],
            Kind::BossDeadlock => ["WAIT LOCK", "JOIN HANG", "POISON PILL"],
            _ => ["SIGSEGV", "NULL DEREF", "KERNEL PANIC"],
        }
    }

    /// How often this bug leaves something behind when it dies.
    pub fn drop_chance(self) -> f32 {
        match self {
            Kind::Spider => 0.55,
            Kind::Clippy | Kind::Overflow => 0.50,
            Kind::Infloop | Kind::Leak => 0.45,
            Kind::Deadlock => 0.40,
            Kind::Worm => 0.35,
            _ => 0.10,
        }
    }
}

/// The four shot ranks, collected with `P`.
pub const SKILLS: [&str; 4] = ["PRINTLN", "TRAIT", "ASYNC", "TOKIO"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bosses_are_flagged_and_named() {
        for k in [Kind::Boss, Kind::BossOverflow, Kind::BossDeadlock] {
            let d = k.def();
            assert!(d.boss);
            assert!(!d.title.is_empty());
        }
        assert!(!Kind::Beetle.def().boss);
        assert!(Kind::Beetle.def().title.is_empty());
    }

    #[test]
    fn a_boss_outlasts_any_bug() {
        let toughest_bug = [Kind::Deadlock, Kind::Overflow, Kind::Infloop]
            .iter()
            .map(|k| k.def().hp)
            .fold(0.0f32, f32::max);
        assert!(Kind::BossOverflow.def().hp > toughest_bug * 10.0);
    }

    #[test]
    fn only_the_three_capsules_recruit() {
        let recruiting: Vec<Item> = [
            Item::Power,
            Item::Bomb,
            Item::Score,
            Item::OneUp,
            Item::Claude,
            Item::Grok,
            Item::Codex,
            Item::Unwrap,
            Item::Tokio,
            Item::Unsafe,
            Item::Agent,
        ]
        .into_iter()
        .filter(|i| i.agent().is_some())
        .collect();
        assert_eq!(recruiting, vec![Item::Claude, Item::Grok, Item::Codex]);
    }

    #[test]
    fn every_agent_capsule_points_back_at_its_agent() {
        for id in AGENTS {
            assert_eq!(id.def().item.agent(), Some(id));
        }
    }

    #[test]
    fn drop_chances_are_probabilities() {
        for k in [Kind::Beetle, Kind::Spider, Kind::Boss, Kind::Panic] {
            let c = k.drop_chance();
            assert!((0.0..=1.0).contains(&c));
        }
    }
}
