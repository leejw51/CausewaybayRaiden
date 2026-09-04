//! The three stage scripts from `src/stage.lua`, as timed data.
//!
//! LÖVE stored each beat as a closure that reached into the world. Closures
//! do not survive the trip to a static table well, so a beat is a list of
//! [`Act`]s the world interprets instead — which has the side benefit that a
//! test can read a script and count what it spawns without running the game.
//!
//! Times are seconds from the start of the stage.

use crate::defs::{Item, Kind, Path};

/// Everything a scripted beat can do.
#[derive(Clone, Copy, Debug)]
pub enum Act {
    /// Name the section, shown under the boss bar.
    Phase(&'static str),
    Spawn {
        kind: Kind,
        x: f32,
        y: f32,
        o: Opts,
    },
    /// A stack of `n` bugs on one column, the outer ones held back.
    SpawnV {
        kind: Kind,
        n: i32,
        cx: f32,
        y: f32,
        gap: f32,
    },
    /// `n` bugs spread from `x0` to `x1`, each entering a little later.
    SpawnLine {
        kind: Kind,
        n: i32,
        x0: f32,
        x1: f32,
        y: f32,
        path: Path,
    },
    Drop {
        x: f32,
        y: f32,
        item: Item,
    },
    /// The klaxon before a boss.
    Warn {
        secs: f32,
        lines: [&'static str; 3],
        /// Two tones instead of one, for the stages that want a louder alarm.
        double: bool,
    },
    BossMusic,
    Shake(f32),
}

/// Spawn overrides. `speed` of zero means "whatever the bug's stat block says".
#[derive(Clone, Copy, Debug)]
pub struct Opts {
    pub path: Path,
    pub vx: f32,
    pub speed: f32,
    pub drop: Option<Item>,
    pub midboss: bool,
}

pub const DEFAULT_OPTS: Opts =
    Opts { path: Path::Down, vx: 0.0, speed: 0.0, drop: None, midboss: false };

const fn p(path: Path) -> Opts {
    Opts { path, ..DEFAULT_OPTS }
}

const fn pd(path: Path, drop: Item) -> Opts {
    Opts { path, drop: Some(drop), ..DEFAULT_OPTS }
}

const fn pv(path: Path, vx: f32) -> Opts {
    Opts { path, vx, ..DEFAULT_OPTS }
}

const fn pvd(path: Path, vx: f32, drop: Item) -> Opts {
    Opts { path, vx, drop: Some(drop), ..DEFAULT_OPTS }
}

const fn ps(path: Path, speed: f32) -> Opts {
    Opts { path, speed, ..DEFAULT_OPTS }
}

const fn mid(path: Path, drop: Item) -> Opts {
    Opts { path, drop: Some(drop), midboss: true, ..DEFAULT_OPTS }
}

pub struct Beat {
    pub t: f32,
    pub acts: &'static [Act],
}

pub const NAMES: [&str; 3] = ["CAUSEWAYBAY", "MTR LINE", "HKU CAMPUS"];
pub const SUBS: [&str; 3] = ["HENNESSY RD", "DEADLOCK EXPRESS", "BURGER RUN"];

const WARN_1: [&str; 3] = ["WARNING", "STACK", "OVERFLOW"];
const WARN_2: [&str; 3] = ["WARNING", "THREAD", "DEADLOCK"];
const WARN_3: [&str; 3] = ["WARNING", "SEGMENTATION", "FAULT"];

/// The script for a stage. Anything out of range replays Causeway Bay rather
/// than dropping the player into an empty sky.
pub fn build(stage: i32) -> &'static [Beat] {
    match stage {
        2 => MTR,
        3 => HKU,
        _ => APT,
    }
}

/// 1-1 Causeway Bay: Hennessy Road, then Hysan Place, then a stack overflow.
#[rustfmt::skip]
static APT: &[Beat] = &[
    Beat {
        t: 1.8,
        acts: &[
            Act::Phase("HENNESSY RD"),
            Act::SpawnV { kind: Kind::Beetle, n: 3, cx: 96.0, y: -14.0, gap: 22.0 },
            Act::Drop { x: 48.0, y: -8.0, item: Item::Claude },
        ],
    },
    Beat {
        t: 4.0,
        acts: &[
            Act::SpawnLine { kind: Kind::Beetle, n: 3, x0: 28.0, x1: 28.0, y: -10.0, path: Path::Sine },
            Act::Spawn { kind: Kind::Offby1, x: 96.0, y: -16.0, o: pvd(Path::Zigzag, 36.0, Item::Power) },
        ],
    },
    Beat {
        t: 6.2,
        acts: &[Act::SpawnLine { kind: Kind::Beetle, n: 3, x0: 164.0, x1: 164.0, y: -10.0, path: Path::Sine }],
    },
    Beat {
        t: 8.0,
        acts: &[
            Act::Spawn { kind: Kind::Moth, x: 48.0, y: -18.0, o: p(Path::Swoop) },
            Act::Spawn { kind: Kind::Moth, x: 144.0, y: -18.0, o: p(Path::Swoop) },
            Act::Drop { x: 144.0, y: -8.0, item: Item::Grok },
        ],
    },
    Beat {
        t: 10.5,
        acts: &[
            Act::Spawn { kind: Kind::Nullptr, x: 24.0, y: -10.0, o: p(Path::Dive) },
            Act::Spawn { kind: Kind::Nullptr, x: 168.0, y: -10.0, o: p(Path::Dive) },
            Act::Spawn { kind: Kind::Clippy, x: 96.0, y: -28.0, o: pd(Path::Hover, Item::Agent) },
        ],
    },
    Beat {
        t: 13.5,
        acts: &[
            Act::Phase("HYSAN PLACE"),
            Act::Spawn { kind: Kind::Leak, x: 64.0, y: -18.0, o: p(Path::Hover) },
            Act::SpawnV { kind: Kind::Beetle, n: 3, cx: 96.0, y: -10.0, gap: 20.0 },
        ],
    },
    Beat {
        t: 16.5,
        acts: &[
            Act::Spawn { kind: Kind::Overflow, x: 96.0, y: -28.0, o: pd(Path::Down, Item::Tokio) },
            Act::Spawn { kind: Kind::Moth, x: 40.0, y: -16.0, o: p(Path::Swoop) },
            Act::Spawn { kind: Kind::Moth, x: 152.0, y: -16.0, o: p(Path::Swoop) },
        ],
    },
    Beat {
        t: 20.0,
        acts: &[
            Act::Spawn { kind: Kind::Beetle, x: 28.0, y: -10.0, o: ps(Path::Down, 48.0) },
            Act::Spawn { kind: Kind::Beetle, x: 64.0, y: -10.0, o: ps(Path::Down, 48.0) },
            Act::Spawn { kind: Kind::Beetle, x: 100.0, y: -10.0, o: ps(Path::Down, 48.0) },
            Act::Spawn { kind: Kind::Beetle, x: 136.0, y: -10.0, o: ps(Path::Down, 48.0) },
            Act::Spawn { kind: Kind::Lifetime, x: 96.0, y: -22.0, o: pd(Path::Teleport, Item::Unwrap) },
            Act::Drop { x: 96.0, y: -8.0, item: Item::Codex },
        ],
    },
    Beat {
        t: 23.5,
        acts: &[Act::Spawn { kind: Kind::Overflow, x: 96.0, y: -36.0, o: mid(Path::Hover, Item::Power) }],
    },
    Beat {
        t: 28.0,
        acts: &[
            Act::SpawnV { kind: Kind::Beetle, n: 3, cx: 96.0, y: -12.0, gap: 18.0 },
            Act::Spawn { kind: Kind::Panic, x: 40.0, y: -14.0, o: p(Path::Dive) },
            Act::Spawn { kind: Kind::Panic, x: 152.0, y: -14.0, o: p(Path::Dive) },
        ],
    },
    Beat {
        t: 32.0,
        acts: &[Act::Warn { secs: 3.2, lines: WARN_1, double: true }],
    },
    Beat {
        t: 35.6,
        acts: &[
            Act::BossMusic,
            Act::Spawn { kind: Kind::BossOverflow, x: 96.0, y: -70.0, o: p(Path::Boss) },
            Act::Shake(5.0),
        ],
    },
];

/// 1-2 MTR Line: the Deadlock Express.
#[rustfmt::skip]
static MTR: &[Beat] = &[
    Beat {
        t: 1.6,
        acts: &[
            Act::Phase("MTR TUNNEL"),
            Act::SpawnLine { kind: Kind::Beetle, n: 3, x0: 32.0, x1: 160.0, y: -8.0, path: Path::Down },
            Act::Drop { x: 64.0, y: -8.0, item: Item::Grok },
        ],
    },
    Beat {
        t: 4.2,
        acts: &[
            Act::Spawn { kind: Kind::Deadlock, x: -16.0, y: 8.0, o: pv(Path::Side, 42.0) },
            Act::Spawn { kind: Kind::Deadlock, x: 208.0, y: 18.0, o: pvd(Path::Side, -42.0, Item::Power) },
        ],
    },
    Beat {
        t: 7.0,
        acts: &[Act::Spawn { kind: Kind::Spider, x: 96.0, y: -18.0, o: pd(Path::Hover, Item::Agent) }],
    },
    Beat {
        t: 10.0,
        acts: &[
            Act::Spawn { kind: Kind::Nullptr, x: 48.0, y: -10.0, o: p(Path::Dive) },
            Act::Spawn { kind: Kind::Nullptr, x: 144.0, y: -10.0, o: p(Path::Dive) },
        ],
    },
    Beat {
        t: 12.8,
        acts: &[
            Act::Spawn { kind: Kind::Infloop, x: 96.0, y: -22.0, o: p(Path::Orbit) },
            Act::SpawnLine { kind: Kind::Beetle, n: 3, x0: 30.0, x1: 40.0, y: -10.0, path: Path::Sine },
        ],
    },
    Beat {
        t: 16.0,
        acts: &[
            Act::Spawn { kind: Kind::Deadlock, x: 96.0, y: -24.0, o: pd(Path::Hover, Item::Bomb) },
            Act::Spawn { kind: Kind::Moth, x: 32.0, y: -16.0, o: p(Path::Swoop) },
            Act::Spawn { kind: Kind::Moth, x: 160.0, y: -16.0, o: p(Path::Swoop) },
            Act::Drop { x: 128.0, y: -8.0, item: Item::Codex },
        ],
    },
    Beat {
        t: 19.5,
        acts: &[
            Act::Phase("LOCKED THREADS"),
            Act::Spawn { kind: Kind::Deadlock, x: 96.0, y: -18.0, o: p(Path::Hover) },
            Act::Spawn { kind: Kind::Heisen, x: 96.0, y: -14.0, o: p(Path::Teleport) },
        ],
    },
    Beat {
        t: 23.0,
        acts: &[
            Act::Spawn { kind: Kind::Worm, x: 96.0, y: -22.0, o: p(Path::Sine) },
            Act::Spawn { kind: Kind::Clippy, x: 96.0, y: -32.0, o: pd(Path::Hover, Item::Tokio) },
        ],
    },
    Beat {
        t: 27.0,
        acts: &[Act::Spawn { kind: Kind::Deadlock, x: 96.0, y: -32.0, o: mid(Path::Hover, Item::Agent) }],
    },
    Beat {
        t: 32.0,
        acts: &[
            Act::Spawn { kind: Kind::Beetle, x: 32.0, y: -8.0, o: pv(Path::Zigzag, 32.0) },
            Act::Spawn { kind: Kind::Beetle, x: 72.0, y: -18.0, o: pv(Path::Zigzag, -40.0) },
            Act::Spawn { kind: Kind::Beetle, x: 112.0, y: -8.0, o: pv(Path::Zigzag, 32.0) },
            Act::Spawn { kind: Kind::Beetle, x: 152.0, y: -18.0, o: pv(Path::Zigzag, -40.0) },
        ],
    },
    Beat {
        t: 35.5,
        acts: &[Act::Warn { secs: 3.2, lines: WARN_2, double: false }],
    },
    Beat {
        t: 39.0,
        acts: &[
            Act::BossMusic,
            Act::Spawn { kind: Kind::BossDeadlock, x: 96.0, y: -72.0, o: p(Path::Boss) },
            Act::Shake(6.0),
        ],
    },
];

/// 1-3 HKU Campus: the burger run, and the segmentation fault at the end of it.
#[rustfmt::skip]
static HKU: &[Beat] = &[
    Beat {
        t: 1.5,
        acts: &[
            Act::Phase("CAMPUS RUN"),
            Act::SpawnV { kind: Kind::Beetle, n: 3, cx: 96.0, y: -12.0, gap: 20.0 },
            Act::Spawn { kind: Kind::Panic, x: 96.0, y: -12.0, o: p(Path::Dive) },
            Act::Drop { x: 96.0, y: -8.0, item: Item::Codex },
        ],
    },
    Beat {
        t: 4.4,
        acts: &[
            Act::Spawn { kind: Kind::Overflow, x: 48.0, y: -20.0, o: pv(Path::Zigzag, 28.0) },
            Act::Spawn { kind: Kind::Overflow, x: 144.0, y: -20.0, o: pvd(Path::Zigzag, -28.0, Item::Power) },
        ],
    },
    Beat {
        t: 7.2,
        acts: &[
            Act::Spawn { kind: Kind::Deadlock, x: -14.0, y: 6.0, o: pv(Path::Side, 50.0) },
            Act::Spawn { kind: Kind::Leak, x: 96.0, y: -18.0, o: pd(Path::Hover, Item::Agent) },
        ],
    },
    Beat {
        t: 10.5,
        acts: &[
            Act::Spawn { kind: Kind::Moth, x: 48.0, y: -16.0, o: p(Path::Swoop) },
            Act::Spawn { kind: Kind::Moth, x: 144.0, y: -16.0, o: p(Path::Swoop) },
            Act::Drop { x: 48.0, y: -8.0, item: Item::Claude },
        ],
    },
    Beat {
        t: 14.0,
        acts: &[
            Act::Spawn { kind: Kind::Nullptr, x: 48.0, y: -10.0, o: p(Path::Dive) },
            Act::Spawn { kind: Kind::Nullptr, x: 144.0, y: -10.0, o: p(Path::Dive) },
        ],
    },
    Beat {
        t: 17.5,
        acts: &[
            Act::Phase("KERNEL EDGE"),
            Act::Spawn { kind: Kind::Infloop, x: 96.0, y: -20.0, o: pd(Path::Orbit, Item::Tokio) },
            Act::Spawn { kind: Kind::Clippy, x: 60.0, y: -24.0, o: p(Path::Hover) },
            Act::Spawn { kind: Kind::Lifetime, x: 132.0, y: -24.0, o: p(Path::Teleport) },
        ],
    },
    Beat {
        t: 21.5,
        acts: &[Act::Spawn { kind: Kind::Overflow, x: 96.0, y: -34.0, o: mid(Path::Hover, Item::Power) }],
    },
    Beat {
        t: 26.5,
        acts: &[
            Act::SpawnV { kind: Kind::Beetle, n: 4, cx: 96.0, y: -10.0, gap: 18.0 },
            Act::Spawn { kind: Kind::Panic, x: 96.0, y: -12.0, o: p(Path::Dive) },
        ],
    },
    Beat {
        t: 30.5,
        acts: &[
            Act::Spawn { kind: Kind::Spider, x: 96.0, y: -20.0, o: pd(Path::Hover, Item::Bomb) },
            Act::SpawnLine { kind: Kind::Beetle, n: 4, x0: 32.0, x1: 160.0, y: -8.0, path: Path::Dive },
        ],
    },
    Beat {
        t: 34.5,
        acts: &[Act::Warn { secs: 3.6, lines: WARN_3, double: true }],
    },
    Beat {
        t: 38.4,
        acts: &[
            Act::BossMusic,
            Act::Spawn { kind: Kind::Boss, x: 96.0, y: -78.0, o: p(Path::Boss) },
            Act::Shake(7.0),
        ],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    fn all() -> [&'static [Beat]; 3] {
        [build(1), build(2), build(3)]
    }

    #[test]
    fn unknown_stages_replay_the_first() {
        assert_eq!(build(0).len(), APT.len());
        assert_eq!(build(99).len(), APT.len());
        assert_eq!(build(1).len(), APT.len());
    }

    #[test]
    fn beats_are_in_time_order() {
        for script in all() {
            for pair in script.windows(2) {
                assert!(pair[0].t < pair[1].t, "beats out of order at {}", pair[0].t);
            }
            assert!(script[0].t > 0.0, "a stage must not open mid-fight");
        }
    }

    #[test]
    fn every_stage_warns_before_its_boss() {
        for script in all() {
            let warn = script
                .iter()
                .find(|b| b.acts.iter().any(|a| matches!(a, Act::Warn { .. })))
                .expect("no warning");
            let boss = script
                .iter()
                .find(|b| {
                    b.acts.iter().any(|a| matches!(a, Act::Spawn { kind, .. } if kind.def().boss))
                })
                .expect("no boss");
            assert!(warn.t < boss.t, "the boss arrives before its klaxon");
            // Long enough to read, short enough not to stall the stage.
            assert!((1.0..6.0).contains(&(boss.t - warn.t)));
        }
    }

    #[test]
    fn every_stage_ends_on_exactly_one_boss() {
        for script in all() {
            let bosses: Vec<&Act> = script
                .iter()
                .flat_map(|b| b.acts)
                .filter(|a| matches!(a, Act::Spawn { kind, .. } if kind.def().boss))
                .collect();
            assert_eq!(bosses.len(), 1);
            let last = script.last().unwrap();
            assert!(last.acts.iter().any(|a| matches!(a, Act::BossMusic)));
        }
    }

    #[test]
    fn every_stage_opens_with_a_section_name() {
        for script in all() {
            assert!(matches!(script[0].acts[0], Act::Phase(name) if !name.is_empty()));
        }
    }

    #[test]
    fn every_stage_offers_an_agent_early() {
        for script in all() {
            let first_agent = script
                .iter()
                .find(|b| {
                    b.acts.iter().any(|a| match a {
                        Act::Drop { item, .. } => item.agent().is_some() || *item == Item::Agent,
                        Act::Spawn { o, .. } => {
                            o.drop.is_some_and(|d| d.agent().is_some() || d == Item::Agent)
                        }
                        _ => false,
                    })
                })
                .expect("no agent anywhere in the stage");
            assert!(first_agent.t < 10.0, "help arrives too late");
        }
    }

    #[test]
    fn each_stage_has_exactly_one_midboss() {
        for script in all() {
            let n = script
                .iter()
                .flat_map(|b| b.acts)
                .filter(|a| matches!(a, Act::Spawn { o, .. } if o.midboss))
                .count();
            assert_eq!(n, 1);
        }
    }

    #[test]
    fn stages_are_named_and_subtitled() {
        assert_eq!(NAMES.len(), SUBS.len());
        for (n, s) in NAMES.iter().zip(SUBS.iter()) {
            assert!(!n.is_empty() && !s.is_empty());
        }
    }
}
