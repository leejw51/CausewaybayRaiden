//! Playing the game with no browser attached.
//!
//! The whole point of putting the simulation in Rust is that it can be run
//! like this: a fixed seed, a fixed frame time, and assertions about what
//! actually happens over a minute of play. LÖVE could only test this inside a
//! window, on a machine with a GPU.

use raiden_core::app::{App, State, B_SHOOT, H_BACK, H_CONFIRM, H_DOWN, H_RIGHT};
use raiden_core::balance::Rank;
use raiden_core::defs::{AgentId, Item, Kind};
use raiden_core::draw::{
    TAG_CIRCLE, TAG_ELLIPSE_FILL, TAG_ELLIPSE_RING, TAG_RECT, TAG_RING, TAG_SPRITE, TAG_TEXT,
    TAG_TILED,
};
use raiden_core::sprites::SPRITES;
use raiden_core::world::{Pad, World};

const STEP: f32 = 1.0 / 60.0;

fn idle() -> Pad {
    Pad::default()
}

fn shooting() -> Pad {
    Pad { shoot: true, ..Pad::default() }
}

/// Run `secs` of game time, holding `pad`.
fn run(w: &mut World, secs: f32, pad: Pad) {
    let frames = (secs / STEP) as i32;
    for _ in 0..frames {
        w.update(STEP, pad);
    }
}

fn world(stage: i32, rank: Rank) -> World {
    let mut w = World::new(20260904, 50_000, stage, rank);
    w.set_view(0.0, 192.0);
    w
}

// ------------------------------------------------------------------ the fight

#[test]
fn a_stage_spawns_bugs_and_then_its_boss() {
    let mut w = world(1, Rank::Normal);
    run(&mut w, 12.0, shooting());
    assert!(!w.enemies.is_empty(), "nothing showed up in the first 12s");
    assert!(w.boss().is_none(), "the boss arrived far too early");

    run(&mut w, 26.0, shooting());
    let boss = w.boss().expect("no boss by 38s");
    assert_eq!(boss.kind, Kind::BossOverflow);
    assert!(boss.hp > 0.0);
    assert!(!w.clear);
}

#[test]
fn killing_the_boss_clears_the_stage() {
    let mut w = world(1, Rank::Normal);
    run(&mut w, 38.0, shooting());
    let id = w.boss().expect("no boss").id;
    // Skip the long fight: take its health away and let the frame notice.
    for e in w.enemies.iter_mut() {
        if e.id == id {
            e.hp = 0.5;
        }
    }
    run(&mut w, 0.2, shooting());
    assert!(w.clear, "the boss died and the stage did not clear");
    assert!(w.clear_t > 0.0);
    assert!(w.boss_dead);
    assert!(w.ebullets.is_empty(), "the screen should be swept on clear");
    assert!(w.score > 20_000, "no clear bonus: {}", w.score);
}

#[test]
fn each_stage_ends_on_its_own_boss() {
    for (stage, want) in [(1, Kind::BossOverflow), (2, Kind::BossDeadlock), (3, Kind::Boss)] {
        let mut w = world(stage, Rank::Normal);
        run(&mut w, 45.0, shooting());
        let boss = w.boss().unwrap_or_else(|| panic!("stage {stage} has no boss"));
        assert_eq!(boss.kind, want, "wrong boss on stage {stage}");
    }
}

#[test]
fn holding_the_shot_fires_and_the_bullets_leave_the_screen() {
    let mut w = world(1, Rank::Normal);
    w.ready_t = 0.0;
    run(&mut w, 0.5, shooting());
    assert!(!w.pbullets.is_empty(), "holding shot fired nothing");

    // Ten seconds of continuous fire must not accumulate: bullets that fly off
    // the top are culled, not kept.
    run(&mut w, 10.0, shooting());
    assert!(w.pbullets.len() < 60, "player bullets are leaking: {}", w.pbullets.len());
}

#[test]
fn a_bullet_that_reaches_the_hitbox_costs_a_life() {
    let mut w = world(1, Rank::Normal);
    w.ready_t = 0.0;
    w.player.inv = 0.0;
    let lives = w.lives;
    // Parked right on the ship, moving slowly: it cannot be missed.
    w.ebullets.push(raiden_core::world::EBullet {
        x: w.player.x,
        y: w.player.y - 1.0,
        vx: 0.0,
        vy: 4.0,
        r: 3.0,
        life: 4.0,
        grazed: false,
        col: None,
    });
    run(&mut w, 0.1, idle());
    assert!(w.player.dying, "the hit did not open the death-bomb window");
    run(&mut w, 0.5, idle());
    assert_eq!(w.lives, lives - 1, "the ship survived a direct hit");
}

#[test]
fn a_bomb_inside_the_death_window_saves_the_ship() {
    let mut w = world(1, Rank::Normal);
    w.ready_t = 0.0;
    w.player.inv = 0.0;
    let (lives, bombs) = (w.lives, w.bombs);
    w.ebullets.push(raiden_core::world::EBullet {
        x: w.player.x,
        y: w.player.y,
        vx: 0.0,
        vy: 0.0,
        r: 3.0,
        life: 4.0,
        grazed: false,
        col: None,
    });
    w.update(STEP, idle());
    assert!(w.player.dying);
    w.update(STEP, Pad { bomb: true, ..Pad::default() });
    assert!(!w.player.dying, "the death bomb did not land");
    assert_eq!(w.lives, lives, "a saved ship still lost a life");
    assert_eq!(w.bombs, bombs - 1);
}

#[test]
fn the_bomb_takes_ownership_of_every_bullet_on_screen() {
    let mut w = world(1, Rank::Normal);
    w.ready_t = 0.0;
    for i in 0..20 {
        w.ebullets.push(raiden_core::world::EBullet {
            x: 40.0 + i as f32 * 5.0,
            y: 80.0,
            vx: 0.0,
            vy: 30.0,
            r: 3.0,
            life: 4.0,
            grazed: false,
            col: None,
        });
    }
    w.update(STEP, Pad { bomb: true, ..Pad::default() });
    assert!(w.ebullets.is_empty(), "bullets survived the bomb");
    assert!(w.score > 0, "clearing bullets should pay");
}

#[test]
fn running_out_of_lives_ends_the_run() {
    let mut w = world(1, Rank::Hard);
    w.ready_t = 0.0;
    w.bombs = 0;
    for _ in 0..(w.lives + 1) {
        w.player.inv = 0.0;
        w.player.dying = true;
        w.player.die_t = 0.0;
        run(&mut w, 0.1, idle());
    }
    assert!(w.over, "the run should be over");
}

// --------------------------------------------------------------- power and AI

#[test]
fn power_capsules_climb_the_shot_ranks_and_then_pay_out() {
    let mut w = world(1, Rank::Normal);
    w.ready_t = 0.0;
    w.power = 1;
    for _ in 0..6 {
        w.pickups.push(raiden_core::world::Pickup {
            x: w.player.x,
            y: w.player.y,
            vy: 0.0,
            t: 0.0,
            kind: Item::Power,
        });
        w.update(STEP, idle());
    }
    assert_eq!(w.power, 4, "power should cap at TOKIO");
    assert!(w.score > 0, "capsules past the cap should score instead");
}

#[test]
fn an_agent_arrives_burns_its_tokens_and_leaves_its_badge_behind() {
    let mut w = world(1, Rank::Normal);
    w.ready_t = 0.0;
    // Parked off to the side, clear of the 52/96/140 lanes the game rains
    // spare badges down, so the only badge in play is the one being watched.
    w.player.x = 178.0;
    w.player.y = 240.0;
    w.pickups.push(raiden_core::world::Pickup {
        x: w.player.x,
        y: w.player.y,
        vy: 0.0,
        t: 0.0,
        kind: Item::Claude,
    });
    w.update(STEP, idle());
    assert_eq!(w.agents.len(), 1);
    assert_eq!(w.agents[0].id, AgentId::Claude);

    // Tokens drain while the ship is flying; Claude carries 22 at 0.40/s, so
    // it should run dry inside a minute. The badge it leaves is checked on the
    // very frame it goes, before anything can pick the badge up again.
    let mut expired_at = None;
    for f in 0..((90.0 / STEP) as i32) {
        // Kept invulnerable: dying also costs an agent, and this test is about
        // the budget running out rather than about the ship being shot down.
        w.player.inv = 1.0;
        let before = w.token_items.len();
        let had = w.agents.iter().any(|a| a.id == AgentId::Claude);
        w.update(STEP, idle());
        if had && !w.agents.iter().any(|a| a.id == AgentId::Claude) {
            assert!(
                w.token_items.len() > before,
                "an expiring agent should leave its badge on the map"
            );
            expired_at = Some(f as f32 * STEP);
            break;
        }
    }
    let secs = expired_at.expect("the agent never ran out of budget");
    assert!((30.0..70.0).contains(&secs), "a full token budget lasted {secs}s");
}

#[test]
fn agents_eat_the_bullets_that_come_at_you() {
    let mut w = world(1, Rank::Normal);
    w.ready_t = 0.0;
    w.pickups.push(raiden_core::world::Pickup {
        x: w.player.x,
        y: w.player.y,
        vy: 0.0,
        t: 0.0,
        kind: Item::Codex,
    });
    w.update(STEP, idle());
    let ag = w.agents[0];
    w.ebullets.push(raiden_core::world::EBullet {
        x: ag.x,
        y: ag.y,
        vx: 0.0,
        vy: 0.0,
        r: 3.0,
        life: 4.0,
        grazed: false,
        col: None,
    });
    w.update(STEP, idle());
    assert!(w.ebullets.is_empty(), "the agent let it through");
}

#[test]
fn flying_alone_makes_the_game_offer_help() {
    // The token rain is what keeps a stranded run playable. With no agents it
    // comes more often than with one already flying.
    let empty = Rank::Normal.kit().token_rain_empty;
    let busy = Rank::Normal.kit().token_rain_busy;
    assert!(empty < busy);

    let mut w = world(1, Rank::Normal);
    w.ready_t = 0.0;
    run(&mut w, empty + 1.0, idle());
    assert!(!w.token_items.is_empty(), "no help arrived while flying alone");
}

// -------------------------------------------------------------- the whole run

#[test]
fn a_minute_of_play_stays_bounded() {
    let mut w = world(2, Rank::Hard);
    run(&mut w, 60.0, shooting());
    // Nothing here should grow without limit over a stage.
    assert!(w.ebullets.len() <= 130, "enemy bullets: {}", w.ebullets.len());
    assert!(w.particles.len() <= 400, "particles: {}", w.particles.len());
    assert!(w.enemies.len() < 60, "enemies: {}", w.enemies.len());
    assert!(w.scenery.len() <= 6, "scenery: {}", w.scenery.len());
    assert!(w.token_items.len() <= 3, "badges: {}", w.token_items.len());
}

#[test]
fn nothing_in_a_long_run_goes_to_nan() {
    let mut w = world(3, Rank::Normal);
    run(&mut w, 50.0, shooting());
    assert!(w.player.x.is_finite() && w.player.y.is_finite());
    for e in &w.enemies {
        assert!(e.x.is_finite() && e.y.is_finite() && e.hp.is_finite());
    }
    for b in &w.ebullets {
        assert!(b.x.is_finite() && b.y.is_finite());
    }
    for b in &w.pbullets {
        assert!(b.x.is_finite() && b.y.is_finite());
    }
}

#[test]
fn easy_really_is_easier_than_hard() {
    let mut easy = world(1, Rank::Easy);
    let mut hard = world(1, Rank::Hard);
    run(&mut easy, 25.0, idle());
    run(&mut hard, 25.0, idle());
    // Same script, same seed, same non-input: the difference is the rank.
    assert!(
        hard.ebullets.len() >= easy.ebullets.len(),
        "hard fired {} and easy fired {}",
        hard.ebullets.len(),
        easy.ebullets.len()
    );
    assert!(easy.lives > hard.lives);
}

// ------------------------------------------------------------- the draw list

/// Walk a frame's command list the way the renderer does, and prove it can.
fn check_draw_list(cmds: &[f32], pool_lines: usize) {
    let mut i = 0usize;
    let mut seen = 0usize;
    while i < cmds.len() {
        let tag = cmds[i];
        let len = match tag {
            t if t == TAG_SPRITE => 13,
            t if t == TAG_RECT => 9,
            t if t == TAG_CIRCLE => 8,
            t if t == TAG_RING => 9,
            t if t == TAG_TEXT => 9,
            t if t == TAG_TILED => 7,
            t if t == TAG_ELLIPSE_RING => 11,
            t if t == TAG_ELLIPSE_FILL => 10,
            other => panic!("unknown draw tag {other} at {i}"),
        };
        assert!(i + len <= cmds.len(), "command at {i} runs past the buffer");
        for (k, v) in cmds[i..i + len].iter().enumerate() {
            assert!(v.is_finite(), "non-finite field {k} in command at {i}");
        }
        if tag == TAG_SPRITE || tag == TAG_TILED {
            let art = cmds[i + 1];
            assert!(art >= 0.0 && (art as usize) < SPRITES.len(), "art index {art} out of range");
        }
        if tag == TAG_TEXT {
            let idx = cmds[i + 1];
            assert!(
                idx >= 0.0 && (idx as usize) < pool_lines,
                "text index {idx} past a pool of {pool_lines}"
            );
        }
        // Alpha is the last field of every command.
        let alpha = cmds[i + len - 1];
        assert!((0.0..=1.0).contains(&alpha), "alpha {alpha} at {i}");
        i += len;
        seen += 1;
    }
    assert_eq!(i, cmds.len(), "the buffer did not end on a command boundary");
    assert!(seen > 0, "the frame drew nothing at all");
}

fn pool_len(app: &App) -> usize {
    let pool = app.text_pool();
    if pool.is_empty() {
        0
    } else {
        pool.lines().count()
    }
}

#[test]
fn every_screen_produces_a_readable_frame() {
    let mut app = App::new(4242);
    // Boot, title, story, map, rank, play, pause, game over: walk through the
    // lot and validate the frame each one lays out.
    let script: &[(f32, u32)] = &[
        (7.0, 0),          // boot runs out into the title
        (0.5, 0),          // title
        (0.02, H_CONFIRM), // -> story
        (1.0, 0),
        (0.02, H_CONFIRM), // -> map
        (0.5, 0),
        (0.02, H_RIGHT),
        (0.02, H_CONFIRM), // -> rank
        (0.5, 0),
        (0.02, H_DOWN),
        (0.02, H_CONFIRM), // -> play
        (3.0, B_SHOOT),
        (0.02, H_BACK), // -> pause
        (0.3, 0),
    ];
    for (secs, bits) in script {
        let frames = ((secs / STEP) as i32).max(1);
        for f in 0..frames {
            // Edges are one frame long, exactly as the shell delivers them.
            app.update(STEP, if f == 0 { *bits } else { 0 });
            check_draw_list(app.commands(), pool_len(&app));
        }
    }
    assert_eq!(app.state, State::Pause, "the walk ended somewhere else");
}

#[test]
fn a_wide_window_widens_the_frame_rather_than_stretching_it() {
    let mut app = App::new(7);
    app.set_view(-80.0, 272.0);
    app.update(STEP, 0);
    check_draw_list(app.commands(), pool_len(&app));

    // Something must actually be drawn out in the new space, or the extra
    // width would just be empty margin. The full-width dim band is the
    // give-away: it starts left of the playfield and is wider than it.
    let wide_band =
        app.commands().windows(9).any(|c| c[0] == TAG_RECT && c[1] < -40.0 && c[3] > 300.0);
    assert!(wide_band, "nothing was drawn in the widened view");
}

#[test]
fn the_title_screen_starts_attracting_on_its_own() {
    let mut app = App::new(9);
    // Boot, then title, then the demo takes over if nobody presses anything.
    for _ in 0..((30.0 / STEP) as i32) {
        app.update(STEP, 0);
    }
    assert_eq!(app.state, State::Play, "the cabinet never went into attract");
    assert!(app.world.is_some());
}

#[test]
fn a_credit_is_spent_starting_a_game_and_not_before() {
    let mut app = App::new(11);
    let credits = app.credits;
    // Boot out, then straight through the menus.
    for _ in 0..((7.0 / STEP) as i32) {
        app.update(STEP, 0);
    }
    app.update(STEP, H_CONFIRM); // story
    assert_eq!(app.credits, credits, "the story screen took a coin");
    app.update(STEP, H_CONFIRM); // map
    app.update(STEP, H_CONFIRM); // rank
    assert_eq!(app.credits, credits, "the rank screen took a coin early");
    app.update(STEP, H_CONFIRM); // play
    assert_eq!(app.state, State::Play);
    assert_eq!(app.credits, credits - 1, "starting a game is not free");
}

#[test]
fn clearing_a_stage_is_remembered() {
    let mut app = App::new(13);
    app.set_progress(90_000, 2, 0b011);
    assert_eq!(app.hiscore, 90_000);
    assert_eq!(app.map_cursor, 2);
    assert_eq!(app.cleared, 0b011);

    // A saved score below what is already banked must not lower it.
    app.set_progress(10, 0, 0);
    assert_eq!(app.hiscore, 90_000);
}
