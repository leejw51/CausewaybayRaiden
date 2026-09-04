//! The MSX1 palette from `src/gfx.lua`, plus the hero's rust.
//!
//! Colours are RGBA in 0..1, the way LÖVE takes them, and travel to the
//! browser as four floats in the draw list.

pub type Col = [f32; 4];

pub const fn rgb(r: f32, g: f32, b: f32) -> Col {
    [r, g, b, 1.0]
}

/// The same colour at a different opacity.
pub const fn a(c: Col, alpha: f32) -> Col {
    [c[0], c[1], c[2], alpha]
}

pub const BLACK: Col = rgb(0.00, 0.00, 0.00);
pub const NAVY: Col = rgb(0.10, 0.10, 0.35);
pub const CYAN: Col = rgb(0.40, 0.86, 0.94);
pub const LGREEN: Col = rgb(0.45, 0.82, 0.49);
pub const DRED: Col = rgb(0.73, 0.37, 0.32);
pub const RUST: Col = rgb(0.86, 0.40, 0.27);
pub const LRED: Col = rgb(1.00, 0.54, 0.49);
pub const YELLOW: Col = rgb(0.87, 0.82, 0.53);
pub const MAGENTA: Col = rgb(0.72, 0.40, 0.71);
pub const GRAY: Col = rgb(0.80, 0.80, 0.80);
pub const WHITE: Col = rgb(1.00, 1.00, 1.00);

/// Claude yellow, Grok red, Codex green — the three agent tints.
pub const CLAUDE: Col = rgb(0.95, 0.84, 0.18);
pub const GROK: Col = rgb(0.95, 0.22, 0.22);
pub const CODEX: Col = rgb(0.22, 0.88, 0.34);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alpha_only_changes_alpha() {
        let faded = a(RUST, 0.25);
        assert_eq!(&faded[..3], &RUST[..3]);
        assert_eq!(faded[3], 0.25);
    }
}
