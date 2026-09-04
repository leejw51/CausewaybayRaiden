//! The one list of art the game knows about.
//!
//! Three consumers read this table and they must agree on the order, because
//! the wasm draw list refers to art by index:
//!
//!   * `Sprite` below, used by the simulation;
//!   * `mkassets`, which shrinks `love2d/assets/*.png` down to the sizes here
//!     and writes `public/art/manifest.json` in this order;
//!   * the TypeScript loader, which reads that manifest.
//!
//! `mkassets` pulls this file in with `#[path]` rather than a dependency, so
//! the build tool does not drag wasm-bindgen along and there is still exactly
//! one place to add a sprite.

/// How a source image is turned into a game asset.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// Chroma-keyed to transparency; drawn as a sprite.
    Sprite,
    /// Opaque, tiled; a background plate.
    Bg,
    /// Chroma-keyed *and* tiled; a parallax layer over the plate.
    Layer,
}

/// `(name, source png, width, height, kind)`, in draw-list index order.
pub const SPRITES: &[(&str, &str, u32, u32, Kind)] = &[
    ("player", "player.png", 40, 40, Kind::Sprite),
    ("beetle", "enemy_beetle.png", 26, 26, Kind::Sprite),
    ("moth", "enemy_moth.png", 32, 28, Kind::Sprite),
    ("spider", "enemy_spider.png", 30, 30, Kind::Sprite),
    ("worm", "enemy_worm.png", 28, 32, Kind::Sprite),
    ("nullptr", "enemy_null.png", 24, 24, Kind::Sprite),
    ("leak", "enemy_leak.png", 28, 28, Kind::Sprite),
    ("overflow", "enemy_overflow.png", 30, 32, Kind::Sprite),
    ("deadlock", "enemy_deadlock.png", 32, 24, Kind::Sprite),
    ("offby1", "enemy_offby1.png", 28, 26, Kind::Sprite),
    ("clippy", "enemy_clippy.png", 30, 30, Kind::Sprite),
    ("lifetime", "enemy_lifetime.png", 28, 32, Kind::Sprite),
    ("infloop", "enemy_loop.png", 32, 32, Kind::Sprite),
    ("panic", "enemy_panic.png", 28, 28, Kind::Sprite),
    ("boss", "boss.png", 96, 88, Kind::Sprite),
    ("king", "king.png", 104, 92, Kind::Sprite),
    ("bossOverflow", "boss_overflow.png", 88, 88, Kind::Sprite),
    ("bossDeadlock", "boss_deadlock.png", 90, 80, Kind::Sprite),
    ("princess", "princess_pitch.png", 32, 32, Kind::Sprite),
    ("tokio", "tokio_missile.png", 12, 18, Kind::Sprite),
    ("claude", "agent_claude.png", 22, 20, Kind::Sprite),
    ("codex", "agent_codex.png", 22, 20, Kind::Sprite),
    ("grok", "agent_grok.png", 22, 20, Kind::Sprite),
    ("gemini", "agent_gemini.png", 20, 20, Kind::Sprite),
    ("pickup", "pickup.png", 18, 18, Kind::Sprite),
    ("boom", "explosion.png", 48, 48, Kind::Sprite),
    ("pbullet", "bullet_player.png", 10, 14, Kind::Sprite),
    ("cloud", "cloud.png", 48, 22, Kind::Sprite),
    ("shopHysan", "shop_hysan.png", 52, 52, Kind::Sprite),
    ("shopMarket", "shop_market.png", 48, 48, Kind::Sprite),
    ("shopRamen", "shop_ramen.png", 48, 48, Kind::Sprite),
    ("shopDimsum", "shop_dimsum.png", 48, 48, Kind::Sprite),
    ("shopBakery", "shop_bakery.png", 48, 48, Kind::Sprite),
    ("shopCoffee", "shop_coffee.png", 46, 46, Kind::Sprite),
    ("shopCase", "shop_case.png", 46, 46, Kind::Sprite),
    ("bg", "bg_city.png", 192, 256, Kind::Bg),
    ("bgFar", "bg_far.png", 192, 256, Kind::Bg),
    ("bgMid", "bg_mid.png", 192, 256, Kind::Bg),
    ("bgNear", "bg_near.png", 192, 256, Kind::Bg),
    ("bgApt", "bg_apt.png", 192, 256, Kind::Bg),
    ("bgMtr", "bg_mtr.png", 192, 256, Kind::Bg),
    ("bgHku", "bg_hku.png", 192, 256, Kind::Bg),
    ("bgAptFar", "bg_apt_far.png", 192, 256, Kind::Bg),
    ("bgMtrFar", "bg_mtr_far.png", 192, 256, Kind::Bg),
    ("bgHkuFar", "bg_hku_far.png", 192, 256, Kind::Bg),
    ("haze", "parallax_haze.png", 192, 256, Kind::Layer),
    ("clouds", "parallax_clouds.png", 192, 256, Kind::Layer),
    ("storyApt", "story_apt.png", 192, 144, Kind::Bg),
    ("storyMtr", "story_mtr.png", 192, 144, Kind::Bg),
    ("storyHku", "story_hku.png", 192, 144, Kind::Bg),
    ("storyEnd", "story_end.png", 192, 144, Kind::Bg),
    ("title", "title_hero.png", 192, 144, Kind::Bg),
    ("worldMap", "world_map.png", 192, 256, Kind::Bg),
    ("bezel", "bezel.png", 80, 256, Kind::Bg),
];

/// Art, by index into [`SPRITES`]. `as u32` is what goes into the draw list.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u32)]
#[allow(dead_code)]
pub enum Sprite {
    Player = 0,
    Beetle,
    Moth,
    Spider,
    Worm,
    Nullptr,
    Leak,
    Overflow,
    Deadlock,
    Offby1,
    Clippy,
    Lifetime,
    Infloop,
    Panic,
    Boss,
    King,
    BossOverflow,
    BossDeadlock,
    Princess,
    Tokio,
    Claude,
    Codex,
    Grok,
    Gemini,
    Pickup,
    Boom,
    Pbullet,
    Cloud,
    ShopHysan,
    ShopMarket,
    ShopRamen,
    ShopDimsum,
    ShopBakery,
    ShopCoffee,
    ShopCase,
    Bg,
    BgFar,
    BgMid,
    BgNear,
    BgApt,
    BgMtr,
    BgHku,
    BgAptFar,
    BgMtrFar,
    BgHkuFar,
    Haze,
    Clouds,
    StoryApt,
    StoryMtr,
    StoryHku,
    StoryEnd,
    Title,
    WorldMap,
    Bezel,
}

impl Sprite {
    /// Unused by `mkassets`, which pulls this file in for the table alone.
    #[allow(dead_code)]
    pub fn id(self) -> f32 {
        self as u32 as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The enum is written out by hand; the table is the source of truth.
    /// A sprite added to one and not the other would silently draw the wrong
    /// picture, so check the two line up.
    #[test]
    fn enum_matches_table() {
        assert_eq!(SPRITES.len(), Sprite::Bezel as usize + 1);
        assert_eq!(SPRITES[Sprite::Player as usize].0, "player");
        assert_eq!(SPRITES[Sprite::King as usize].0, "king");
        assert_eq!(SPRITES[Sprite::Clouds as usize].0, "clouds");
        assert_eq!(SPRITES[Sprite::Bezel as usize].0, "bezel");
    }

    #[test]
    fn names_are_unique() {
        let mut names: Vec<&str> = SPRITES.iter().map(|s| s.0).collect();
        names.sort_unstable();
        let before = names.len();
        names.dedup();
        assert_eq!(before, names.len(), "duplicate sprite name");
    }
}
