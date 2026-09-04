//! The frame, as a flat list of numbers.
//!
//! Rust owns every pixel decision and the browser owns none: a frame is a
//! `Vec<f32>` of tagged commands plus a pool of strings the text commands
//! point into. Both cross the wasm boundary as one typed-array view each,
//! so a busy frame costs two copies rather than thousands of calls.
//!
//! Commands are self-describing by tag; the reader switches on the first
//! number and skips by that command's fixed length.
//!
//! | tag | command      | payload                                            |
//! |-----|--------------|----------------------------------------------------|
//! | 1   | sprite       | art, x, y, rot, sx, sy, ox, oy, r, g, b, a         |
//! | 2   | rect         | x, y, w, h, r, g, b, a                             |
//! | 3   | circle       | x, y, radius, r, g, b, a                           |
//! | 4   | ring         | x, y, radius, line width, r, g, b, a               |
//! | 5   | text         | string index, x, y, scale, r, g, b, a              |
//! | 6   | tiled layer  | art, y offset, r, g, b, a                          |
//! | 7   | ellipse ring | x, y, rx, ry, rotation, line width, r, g, b, a     |
//! | 8   | ellipse fill | x, y, rx, ry, rotation, r, g, b, a                 |
//!
//! Coordinates are playfield pixels: `0..192` across and `0..256` down, the
//! same space `love2d/` draws in. On a wide window the visible span reaches
//! past both edges, which is what [`Painter::view_left`] and
//! [`Painter::view_right`] report.

use crate::font;
use crate::palette::Col;
use crate::sprites::Sprite;

pub const TAG_SPRITE: f32 = 1.0;
pub const TAG_RECT: f32 = 2.0;
pub const TAG_CIRCLE: f32 = 3.0;
pub const TAG_RING: f32 = 4.0;
pub const TAG_TEXT: f32 = 5.0;
pub const TAG_TILED: f32 = 6.0;
pub const TAG_ELLIPSE_RING: f32 = 7.0;
pub const TAG_ELLIPSE_FILL: f32 = 8.0;

/// The playfield: the safe area every stage is authored against.
pub const GW: f32 = 192.0;
pub const GH: f32 = 256.0;

pub struct Painter {
    buf: Vec<f32>,
    strings: Vec<String>,
    /// Visible span in playfield coordinates. Negative on the left and past
    /// `GW` on the right when the window is wider than 3:4.
    left: f32,
    right: f32,
}

impl Default for Painter {
    fn default() -> Self {
        Painter {
            buf: Vec::with_capacity(8192),
            strings: Vec::with_capacity(64),
            left: 0.0,
            right: GW,
        }
    }
}

impl Painter {
    pub fn set_view(&mut self, left: f32, right: f32) {
        // Never narrower than a sliver: a zero-width view would divide by zero
        // in every centring calculation downstream.
        self.left = left.min(GW - 8.0);
        self.right = right.max(self.left + 8.0);
    }

    pub fn view_left(&self) -> f32 {
        self.left
    }

    pub fn view_right(&self) -> f32 {
        self.right
    }

    /// The span anything full-width should cover: a few pixels of bleed past
    /// each visible edge, so a dim overlay has no seam at the border.
    pub fn view_span(&self) -> (f32, f32) {
        let x0 = (self.left - 4.0).floor();
        let x1 = (self.right + 4.0).ceil();
        (x0, (x1 - x0).max(1.0))
    }

    /// Where the ship may fly: the whole visible width, not just the playfield.
    pub fn play_bounds(&self) -> (f32, f32) {
        (self.left + 10.0, self.right - 10.0)
    }

    /// The wider of the playfield and the window, for scenery that should not
    /// stop at the playfield edge on a wide screen.
    pub fn ambient_span(&self) -> (f32, f32) {
        (self.left.min(0.0), self.right.max(GW))
    }

    pub fn begin(&mut self) {
        self.buf.clear();
        self.strings.clear();
    }

    pub fn commands(&self) -> &[f32] {
        &self.buf
    }

    /// The frame's strings, newline joined; text commands index into the
    /// lines of this. Newlines cannot appear in game text.
    pub fn text_pool(&self) -> String {
        self.strings.join("\n")
    }

    fn intern(&mut self, s: &str) -> f32 {
        // Frames repeat their labels ("SCORE", a stage name); a linear scan
        // over a few dozen short strings is cheaper than the duplicates.
        if let Some(i) = self.strings.iter().position(|t| t == s) {
            return i as f32;
        }
        self.strings.push(s.to_string());
        (self.strings.len() - 1) as f32
    }

    fn col(&mut self, c: Col) {
        self.buf.extend_from_slice(&c);
    }

    // ------------------------------------------------------------- primitives

    #[allow(clippy::too_many_arguments)]
    pub fn sprite(
        &mut self,
        art: Sprite,
        x: f32,
        y: f32,
        rot: f32,
        sx: f32,
        sy: f32,
        ox: f32,
        oy: f32,
        c: Col,
    ) {
        if c[3] <= 0.0 {
            return;
        }
        self.buf.extend_from_slice(&[TAG_SPRITE, art.id(), x, y, rot, sx, sy, ox, oy]);
        self.col(c);
    }

    /// A sprite drawn from its top-left at 1:1, the common case.
    pub fn blit(&mut self, art: Sprite, x: f32, y: f32, c: Col) {
        self.sprite(art, x, y, 0.0, 1.0, 1.0, 0.0, 0.0, c);
    }

    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, c: Col) {
        if c[3] <= 0.0 || w <= 0.0 || h <= 0.0 {
            return;
        }
        self.buf.extend_from_slice(&[TAG_RECT, x, y, w, h]);
        self.col(c);
    }

    pub fn circle(&mut self, x: f32, y: f32, r: f32, c: Col) {
        if c[3] <= 0.0 || r <= 0.0 {
            return;
        }
        self.buf.extend_from_slice(&[TAG_CIRCLE, x, y, r]);
        self.col(c);
    }

    pub fn ring(&mut self, x: f32, y: f32, r: f32, lw: f32, c: Col) {
        if c[3] <= 0.0 || r <= 0.0 {
            return;
        }
        self.buf.extend_from_slice(&[TAG_RING, x, y, r, lw]);
        self.col(c);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ellipse_ring(&mut self, x: f32, y: f32, rx: f32, ry: f32, rot: f32, lw: f32, c: Col) {
        if c[3] <= 0.0 {
            return;
        }
        self.buf.extend_from_slice(&[TAG_ELLIPSE_RING, x, y, rx, ry, rot, lw]);
        self.col(c);
    }

    pub fn ellipse_fill(&mut self, x: f32, y: f32, rx: f32, ry: f32, rot: f32, c: Col) {
        if c[3] <= 0.0 {
            return;
        }
        self.buf.extend_from_slice(&[TAG_ELLIPSE_FILL, x, y, rx, ry, rot]);
        self.col(c);
    }

    /// A background plate, repeated to fill the canvas, scrolled to `y_off`.
    pub fn tiled(&mut self, art: Sprite, y_off: f32, c: Col) {
        if c[3] <= 0.0 {
            return;
        }
        self.buf.extend_from_slice(&[TAG_TILED, art.id(), y_off]);
        self.col(c);
    }

    // ------------------------------------------------------------------- text

    pub fn text(&mut self, s: &str, x: f32, y: f32, c: Col, scale: f32) {
        if c[3] <= 0.0 || s.is_empty() {
            return;
        }
        let idx = self.intern(s);
        self.buf.extend_from_slice(&[TAG_TEXT, idx, x, y, scale]);
        self.col(c);
    }

    /// Text over its own black offset copy, so it reads against any backdrop.
    pub fn text_shadow(&mut self, s: &str, x: f32, y: f32, c: Col, scale: f32) {
        let shadow = [0.0, 0.0, 0.0, c[3]];
        self.text(s, x + scale, y + scale, shadow, scale);
        self.text(s, x, y, c, scale);
    }

    /// Centred across the *visible* width, shrinking to fit rather than
    /// running off the edge of a narrow window.
    pub fn center(&mut self, s: &str, y: f32, c: Col, scale: f32) {
        let (x0, x1) = (self.left, self.right);
        let max_w = (x1 - x0).max(8.0);
        let mut scale = scale;
        let mut w = font::text_width(s, scale);
        if w > max_w {
            scale = max_w / (s.chars().count().max(1) as f32 * 8.0);
            w = font::text_width(s, scale);
        }
        self.text_shadow(s, (x0 + (max_w - w) / 2.0).floor(), y, c, scale);
    }

    /// Right-aligned at `x`.
    pub fn text_right(&mut self, s: &str, x: f32, y: f32, c: Col, scale: f32) {
        self.text(s, x - font::text_width(s, scale), y, c, scale);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette;

    #[test]
    fn a_frame_starts_empty_and_records_commands() {
        let mut p = Painter::default();
        p.begin();
        assert!(p.commands().is_empty());
        p.rect(1.0, 2.0, 3.0, 4.0, palette::WHITE);
        assert_eq!(p.commands(), &[TAG_RECT, 1.0, 2.0, 3.0, 4.0, 1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn fully_transparent_and_empty_shapes_cost_nothing() {
        let mut p = Painter::default();
        p.begin();
        p.rect(0.0, 0.0, 10.0, 10.0, palette::a(palette::WHITE, 0.0));
        p.rect(0.0, 0.0, 0.0, 10.0, palette::WHITE);
        p.circle(0.0, 0.0, 0.0, palette::WHITE);
        p.text("", 0.0, 0.0, palette::WHITE, 1.0);
        assert!(p.commands().is_empty());
    }

    #[test]
    fn repeated_strings_are_pooled_once() {
        let mut p = Painter::default();
        p.begin();
        p.text("READY", 0.0, 0.0, palette::WHITE, 1.0);
        p.text("READY", 0.0, 8.0, palette::WHITE, 1.0);
        p.text("GO", 0.0, 16.0, palette::WHITE, 1.0);
        assert_eq!(p.text_pool(), "READY\nGO");
        // Both READY commands point at line 0, GO at line 1.
        let idx: Vec<f32> = p.commands().chunks(9).map(|c| c[1]).collect();
        assert_eq!(idx, vec![0.0, 0.0, 1.0]);
    }

    #[test]
    fn text_pool_survives_a_reset() {
        let mut p = Painter::default();
        p.begin();
        p.text("ONE", 0.0, 0.0, palette::WHITE, 1.0);
        p.begin();
        p.text("TWO", 0.0, 0.0, palette::WHITE, 1.0);
        assert_eq!(p.text_pool(), "TWO");
    }

    #[test]
    fn centring_uses_the_visible_span_not_the_playfield() {
        let mut p = Painter::default();
        p.set_view(-64.0, 256.0);
        p.begin();
        p.center("AB", 0.0, palette::WHITE, 1.0);
        // Shadow first, then the glyph itself: check the second command's x.
        let x = p.commands()[2 + 9];
        assert_eq!(x, (-64.0f32 + (320.0 - 16.0) / 2.0).floor());
    }

    #[test]
    fn centring_shrinks_rather_than_overflowing() {
        let mut p = Painter::default();
        p.set_view(0.0, 40.0);
        p.begin();
        p.center("AAAAAAAAAA", 0.0, palette::WHITE, 1.0);
        let scale = p.commands()[4];
        assert!(scale < 1.0, "expected the text to shrink, got {scale}");
        assert!(font::text_width("AAAAAAAAAA", scale) <= 40.0);
    }

    #[test]
    fn the_view_can_never_collapse() {
        let mut p = Painter::default();
        p.set_view(100.0, 100.0);
        assert!(p.view_right() > p.view_left());
        let (_, w) = p.view_span();
        assert!(w >= 1.0);
    }

    #[test]
    fn ambient_span_always_covers_the_playfield() {
        let mut p = Painter::default();
        p.set_view(20.0, 150.0);
        let (a, b) = p.ambient_span();
        assert!(a <= 0.0 && b >= GW);
    }

    #[test]
    fn right_aligned_text_ends_where_asked() {
        let mut p = Painter::default();
        p.begin();
        p.text_right("HI", 100.0, 0.0, palette::WHITE, 1.0);
        assert_eq!(p.commands()[2], 100.0 - 16.0);
    }
}
