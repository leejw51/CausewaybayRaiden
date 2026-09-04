//! Shrink the LÖVE art down to what the browser actually downloads.
//!
//! `love2d/assets` holds ~155 MB of 1024-square studio renders. The LÖVE build
//! never shows them at that size: `src/gfx.lua` redraws each one into a canvas
//! of a few dozen pixels at load time, keys the magenta backdrop out, and keeps
//! only that. A desktop game can afford to do it on every launch; a web game
//! cannot ship the originals at all.
//!
//! So the same shrink happens here, once, at build time, and `public/art` ends
//! up a few hundred kilobytes. The sizes and the chroma-key thresholds are
//! copied from `gfx.lua` so the sprites come out pixel-for-pixel as the LÖVE
//! game draws them.
//!
//!     cargo run -p mkassets -- <src dir> <out dir>

use std::path::{Path, PathBuf};

use image::imageops::FilterType;
use image::{ImageBuffer, Rgba, RgbaImage};

#[path = "../../raiden-core/src/sprites.rs"]
mod sprites;

use sprites::{Kind, SPRITES};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let src = PathBuf::from(args.first().map(String::as_str).unwrap_or("../love2d/assets"));
    let out = PathBuf::from(args.get(1).map(String::as_str).unwrap_or("public/art"));

    if let Err(err) = std::fs::create_dir_all(&out) {
        fail(&format!("cannot create {}: {err}", out.display()));
    }

    let mut missing: Vec<&str> = Vec::new();
    let mut total = 0u64;

    for (name, file, w, h, kind) in SPRITES {
        let from = src.join(file);
        let to = out.join(format!("{name}.png"));
        match convert(&from, &to, *w, *h, *kind) {
            Ok(bytes) => total += bytes,
            Err(err) => {
                eprintln!("  {file}: {err}");
                missing.push(file);
            }
        }
    }

    let manifest = manifest_json();
    if let Err(err) = std::fs::write(out.join("manifest.json"), &manifest) {
        fail(&format!("cannot write manifest: {err}"));
    }

    if !missing.is_empty() {
        fail(&format!("{} source images missing or unreadable", missing.len()));
    }
    println!("  {} assets -> {} ({:.0} KB)", SPRITES.len(), out.display(), total as f64 / 1024.0);
}

fn fail(msg: &str) -> ! {
    eprintln!("mkassets: {msg}");
    std::process::exit(1);
}

/// The load order the TypeScript side needs, so it never has to repeat the
/// list. Hand-rolled rather than pulling in serde for eight lines of JSON.
fn manifest_json() -> String {
    let names: Vec<String> = SPRITES.iter().map(|s| format!("\"{}\"", s.0)).collect();
    let tiles: Vec<String> =
        SPRITES.iter().filter(|s| s.4 != Kind::Sprite).map(|s| format!("\"{}\"", s.0)).collect();
    format!("{{\n  \"sprites\": [{}],\n  \"tiled\": [{}]\n}}\n", names.join(", "), tiles.join(", "))
}

fn convert(from: &Path, to: &Path, w: u32, h: u32, kind: Kind) -> Result<u64, String> {
    let img = image::open(from).map_err(|e| e.to_string())?;
    // Triangle, not nearest: LÖVE draws the source with a linear filter and
    // keys the *filtered* pixels, so keying after a box-ish resize is what
    // reproduces its soft sprite edges.
    let small = img.resize_exact(w, h, FilterType::Triangle).to_rgba8();
    let out: RgbaImage = match kind {
        Kind::Bg => opaque(small),
        Kind::Sprite | Kind::Layer => chroma_key(small),
    };
    out.save(to).map_err(|e| e.to_string())?;
    std::fs::metadata(to).map(|m| m.len()).map_err(|e| e.to_string())
}

/// Backgrounds are composited onto black, exactly as `G.makeBg` does, so a
/// source with an alpha channel cannot punch a hole in the sky.
fn opaque(img: RgbaImage) -> RgbaImage {
    let (w, h) = img.dimensions();
    ImageBuffer::from_fn(w, h, |x, y| {
        let p = img.get_pixel(x, y).0;
        let a = p[3] as f32 / 255.0;
        Rgba([(p[0] as f32 * a) as u8, (p[1] as f32 * a) as u8, (p[2] as f32 * a) as u8, 255])
    })
}

/// The hot magenta studio backdrop, removed.
///
/// Both of `gfx.lua`'s passes are here: the shader's "is this pink and is it
/// far enough from green" test, and the tighter sweep it runs over the result.
/// Neither alone clears the whole backdrop.
fn chroma_key(img: RgbaImage) -> RgbaImage {
    let (w, h) = img.dimensions();
    ImageBuffer::from_fn(w, h, |x, y| {
        let p = img.get_pixel(x, y).0;
        let (r, g, b) = (p[0] as f32 / 255.0, p[1] as f32 / 255.0, p[2] as f32 / 255.0);
        let shader_hit = g < 0.46 && r > 0.70 && b > 0.70 && ((r + b) * 0.5 - g) > 0.42;
        let tight_hit = g < 0.42 && r > 0.75 && b > 0.75;
        if shader_hit || tight_hit {
            Rgba([0, 0, 0, 0])
        } else {
            Rgba(p)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn px(r: u8, g: u8, b: u8) -> RgbaImage {
        ImageBuffer::from_pixel(1, 1, Rgba([r, g, b, 255]))
    }

    #[test]
    fn magenta_backdrop_is_cleared() {
        assert_eq!(chroma_key(px(255, 0, 255)).get_pixel(0, 0).0[3], 0);
        assert_eq!(chroma_key(px(230, 40, 230)).get_pixel(0, 0).0[3], 0);
    }

    #[test]
    fn art_colours_survive() {
        // The hero's rust, the HUD cyan, and a mid grey: none are backdrop.
        for c in [(219, 102, 69), (102, 219, 240), (128, 128, 128)] {
            let out = chroma_key(px(c.0, c.1, c.2));
            assert_eq!(out.get_pixel(0, 0).0[3], 255, "{c:?} was keyed out");
        }
    }

    #[test]
    fn backgrounds_come_out_opaque() {
        let mut src: RgbaImage = ImageBuffer::from_pixel(2, 2, Rgba([255, 0, 255, 0]));
        src.put_pixel(0, 0, Rgba([10, 20, 30, 255]));
        let out = opaque(src);
        for p in out.pixels() {
            assert_eq!(p.0[3], 255);
        }
        assert_eq!(out.get_pixel(1, 1).0, [0, 0, 0, 255]);
    }

    #[test]
    fn manifest_lists_every_sprite_in_order() {
        let json = manifest_json();
        assert!(json.contains("\"player\""));
        assert!(json.contains("\"bezel\""));
        let first = json.find("\"player\"").unwrap();
        let second = json.find("\"beetle\"").unwrap();
        assert!(first < second, "manifest must keep draw-list order");
    }
}
