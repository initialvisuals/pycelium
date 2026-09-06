//! Geometric grotesque HUD captions — stroked, y=0 at the top (not a 5×5 bitmap).
//!
//! The previous 5×5 pack was sampled with a Y flip (`1.0 - cell.y`) while bits were
//! documented as top-left, so letters read inverted / noisy. This atlas is built
//! upright and sampled with cell.y = 0 at the top.

pub const GLYPH: u32 = 32;
pub const ATLAS_COLS: u32 = 16;
pub const ATLAS_ROWS: u32 = 3;

type Pt = (f32, f32);
type Seg = (Pt, Pt);

pub struct FontAtlas {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

/// 0 = space, 1–26 = A–Z, 27 = `:`.
pub fn rasterize_atlas() -> FontAtlas {
    let width = ATLAS_COLS * GLYPH;
    let height = ATLAS_ROWS * GLYPH;
    let mut pixels = vec![0u8; (width * height) as usize];
    for ch in 0..=27 {
        let col = (ch as u32) % ATLAS_COLS;
        let row = (ch as u32) / ATLAS_COLS;
        let glyph = raster_glyph(ch);
        for y in 0..GLYPH {
            for x in 0..GLYPH {
                let dx = col * GLYPH + x;
                let dy = row * GLYPH + y;
                pixels[(dy * width + dx) as usize] = glyph[(y * GLYPH + x) as usize];
            }
        }
    }
    FontAtlas {
        width,
        height,
        pixels,
    }
}

pub fn raster_glyph(ch: i32) -> Vec<u8> {
    let strokes = glyph_strokes(ch);
    let thick = 0.078;
    let mut out = vec![0u8; (GLYPH * GLYPH) as usize];
    if strokes.is_empty() {
        return out;
    }
    let n = GLYPH as f32;
    for y in 0..GLYPH {
        for x in 0..GLYPH {
            let p = ((x as f32 + 0.5) / n, (y as f32 + 0.5) / n);
            let mut d = 1.0e9_f32;
            for &(a, b) in strokes {
                d = d.min(sd_segment(p, a, b));
            }
            let fade = 1.35 / n;
            let cover = (1.0 - (d - thick) / fade).clamp(0.0, 1.0);
            out[(y * GLYPH + x) as usize] = (cover * 255.0) as u8;
        }
    }
    out
}

fn sd_segment(p: Pt, a: Pt, b: Pt) -> f32 {
    let pax = p.0 - a.0;
    let pay = p.1 - a.1;
    let bax = b.0 - a.0;
    let bay = b.1 - a.1;
    let denom = (bax * bax + bay * bay).max(1.0e-8);
    let h = (pax * bax + pay * bay) / denom;
    let h = h.clamp(0.0, 1.0);
    let dx = pax - bax * h;
    let dy = pay - bay * h;
    (dx * dx + dy * dy).sqrt()
}

fn glyph_strokes(ch: i32) -> &'static [Seg] {
    match ch {
        1 => A,
        2 => B,
        3 => C,
        4 => D,
        5 => E,
        6 => F,
        7 => G,
        8 => H,
        9 => I,
        10 => J,
        11 => K,
        12 => L,
        13 => M,
        14 => N,
        15 => O,
        16 => P,
        17 => Q,
        18 => R,
        19 => S,
        20 => T,
        21 => U,
        22 => V,
        23 => W,
        24 => X,
        25 => Y,
        26 => Z,
        27 => COLON,
        _ => &[],
    }
}

// y = 0 at the top. Geometric grotesque (system-UI / Futura-ish).
const A: &[Seg] = &[
    ((0.18, 0.88), (0.50, 0.14)),
    ((0.82, 0.88), (0.50, 0.14)),
    ((0.30, 0.58), (0.70, 0.58)),
];
const B: &[Seg] = &[
    ((0.22, 0.14), (0.22, 0.88)),
    ((0.22, 0.14), (0.62, 0.14)),
    ((0.62, 0.14), (0.76, 0.26)),
    ((0.76, 0.26), (0.62, 0.42)),
    ((0.62, 0.42), (0.22, 0.42)),
    ((0.22, 0.42), (0.66, 0.42)),
    ((0.66, 0.42), (0.80, 0.64)),
    ((0.80, 0.64), (0.66, 0.88)),
    ((0.66, 0.88), (0.22, 0.88)),
];
const C: &[Seg] = &[
    ((0.76, 0.26), (0.60, 0.14)),
    ((0.60, 0.14), (0.36, 0.14)),
    ((0.36, 0.14), (0.20, 0.30)),
    ((0.20, 0.30), (0.20, 0.72)),
    ((0.20, 0.72), (0.36, 0.88)),
    ((0.36, 0.88), (0.60, 0.88)),
    ((0.60, 0.88), (0.76, 0.76)),
];
const D: &[Seg] = &[
    ((0.22, 0.14), (0.22, 0.88)),
    ((0.22, 0.14), (0.58, 0.14)),
    ((0.58, 0.14), (0.80, 0.32)),
    ((0.80, 0.32), (0.80, 0.70)),
    ((0.80, 0.70), (0.58, 0.88)),
    ((0.58, 0.88), (0.22, 0.88)),
];
const E: &[Seg] = &[
    ((0.24, 0.14), (0.24, 0.88)),
    ((0.24, 0.14), (0.78, 0.14)),
    ((0.24, 0.50), (0.68, 0.50)),
    ((0.24, 0.88), (0.78, 0.88)),
];
const F: &[Seg] = &[
    ((0.24, 0.14), (0.24, 0.88)),
    ((0.24, 0.14), (0.78, 0.14)),
    ((0.24, 0.50), (0.66, 0.50)),
];
const G: &[Seg] = &[
    ((0.76, 0.26), (0.60, 0.14)),
    ((0.60, 0.14), (0.36, 0.14)),
    ((0.36, 0.14), (0.20, 0.30)),
    ((0.20, 0.30), (0.20, 0.72)),
    ((0.20, 0.72), (0.36, 0.88)),
    ((0.36, 0.88), (0.62, 0.88)),
    ((0.62, 0.88), (0.78, 0.72)),
    ((0.78, 0.72), (0.78, 0.56)),
    ((0.78, 0.56), (0.52, 0.56)),
];
const H: &[Seg] = &[
    ((0.22, 0.14), (0.22, 0.88)),
    ((0.78, 0.14), (0.78, 0.88)),
    ((0.22, 0.50), (0.78, 0.50)),
];
const I: &[Seg] = &[
    ((0.28, 0.14), (0.72, 0.14)),
    ((0.50, 0.14), (0.50, 0.88)),
    ((0.28, 0.88), (0.72, 0.88)),
];
const J: &[Seg] = &[
    ((0.30, 0.14), (0.78, 0.14)),
    ((0.62, 0.14), (0.62, 0.74)),
    ((0.62, 0.74), (0.48, 0.88)),
    ((0.48, 0.88), (0.28, 0.88)),
    ((0.28, 0.88), (0.20, 0.76)),
];
const K: &[Seg] = &[
    ((0.22, 0.14), (0.22, 0.88)),
    ((0.74, 0.14), (0.22, 0.52)),
    ((0.22, 0.52), (0.76, 0.88)),
];
const L: &[Seg] = &[
    ((0.24, 0.14), (0.24, 0.88)),
    ((0.24, 0.88), (0.78, 0.88)),
];
const M: &[Seg] = &[
    ((0.16, 0.88), (0.16, 0.14)),
    ((0.16, 0.14), (0.50, 0.58)),
    ((0.84, 0.14), (0.50, 0.58)),
    ((0.84, 0.14), (0.84, 0.88)),
];
const N: &[Seg] = &[
    ((0.22, 0.88), (0.22, 0.14)),
    ((0.22, 0.14), (0.78, 0.88)),
    ((0.78, 0.88), (0.78, 0.14)),
];
const O: &[Seg] = &[
    ((0.36, 0.14), (0.64, 0.14)),
    ((0.64, 0.14), (0.80, 0.30)),
    ((0.80, 0.30), (0.80, 0.72)),
    ((0.80, 0.72), (0.64, 0.88)),
    ((0.64, 0.88), (0.36, 0.88)),
    ((0.36, 0.88), (0.20, 0.72)),
    ((0.20, 0.72), (0.20, 0.30)),
    ((0.20, 0.30), (0.36, 0.14)),
];
const P: &[Seg] = &[
    ((0.22, 0.14), (0.22, 0.88)),
    ((0.22, 0.14), (0.64, 0.14)),
    ((0.64, 0.14), (0.78, 0.28)),
    ((0.78, 0.28), (0.64, 0.46)),
    ((0.64, 0.46), (0.22, 0.46)),
];
const Q: &[Seg] = &[
    ((0.36, 0.14), (0.64, 0.14)),
    ((0.64, 0.14), (0.80, 0.30)),
    ((0.80, 0.30), (0.80, 0.68)),
    ((0.80, 0.68), (0.64, 0.84)),
    ((0.64, 0.84), (0.36, 0.84)),
    ((0.36, 0.84), (0.20, 0.68)),
    ((0.20, 0.68), (0.20, 0.30)),
    ((0.20, 0.30), (0.36, 0.14)),
    ((0.52, 0.64), (0.80, 0.90)),
];
const R: &[Seg] = &[
    ((0.22, 0.14), (0.22, 0.88)),
    ((0.22, 0.14), (0.64, 0.14)),
    ((0.64, 0.14), (0.78, 0.28)),
    ((0.78, 0.28), (0.64, 0.46)),
    ((0.64, 0.46), (0.22, 0.46)),
    ((0.46, 0.46), (0.78, 0.88)),
];
const S: &[Seg] = &[
    ((0.74, 0.24), (0.58, 0.14)),
    ((0.58, 0.14), (0.34, 0.14)),
    ((0.34, 0.14), (0.22, 0.26)),
    ((0.22, 0.26), (0.34, 0.40)),
    ((0.34, 0.40), (0.66, 0.54)),
    ((0.66, 0.54), (0.78, 0.70)),
    ((0.78, 0.70), (0.64, 0.88)),
    ((0.64, 0.88), (0.34, 0.88)),
    ((0.34, 0.88), (0.22, 0.76)),
];
const T: &[Seg] = &[
    ((0.16, 0.14), (0.84, 0.14)),
    ((0.50, 0.14), (0.50, 0.88)),
];
const U: &[Seg] = &[
    ((0.22, 0.14), (0.22, 0.70)),
    ((0.22, 0.70), (0.36, 0.88)),
    ((0.36, 0.88), (0.64, 0.88)),
    ((0.64, 0.88), (0.78, 0.70)),
    ((0.78, 0.70), (0.78, 0.14)),
];
const V: &[Seg] = &[
    ((0.16, 0.14), (0.50, 0.88)),
    ((0.84, 0.14), (0.50, 0.88)),
];
const W: &[Seg] = &[
    ((0.10, 0.14), (0.28, 0.88)),
    ((0.28, 0.88), (0.50, 0.40)),
    ((0.50, 0.40), (0.72, 0.88)),
    ((0.72, 0.88), (0.90, 0.14)),
];
const X: &[Seg] = &[
    ((0.20, 0.14), (0.80, 0.88)),
    ((0.80, 0.14), (0.20, 0.88)),
];
const Y: &[Seg] = &[
    ((0.18, 0.14), (0.50, 0.50)),
    ((0.82, 0.14), (0.50, 0.50)),
    ((0.50, 0.50), (0.50, 0.88)),
];
const Z: &[Seg] = &[
    ((0.20, 0.14), (0.80, 0.14)),
    ((0.80, 0.14), (0.20, 0.88)),
    ((0.20, 0.88), (0.80, 0.88)),
];
const COLON: &[Seg] = &[((0.50, 0.32), (0.50, 0.32)), ((0.50, 0.70), (0.50, 0.70))];

fn ink_in_rect(glyph: &[u8], x0: u32, y0: u32, x1: u32, y1: u32) -> u32 {
    let mut s = 0u32;
    for y in y0..y1.min(GLYPH) {
        for x in x0..x1.min(GLYPH) {
            s += glyph[(y * GLYPH + x) as usize] as u32;
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atlas_is_512x96() {
        let atlas = rasterize_atlas();
        assert_eq!(atlas.width, 512);
        assert_eq!(atlas.height, 96);
        assert_eq!(atlas.pixels.len(), 512 * 96);
        assert!(atlas.pixels.iter().any(|&p| p > 80));
    }

    #[test]
    fn letter_a_is_upright_not_inverted() {
        let g = raster_glyph(1);
        // Pointed top / crossbar mid / two legs at the bottom.
        let top = ink_in_rect(&g, 10, 2, 22, 10);
        let mid = ink_in_rect(&g, 6, 14, 26, 20);
        let bot_l = ink_in_rect(&g, 2, 24, 12, 31);
        let bot_r = ink_in_rect(&g, 20, 24, 30, 31);
        let bot_c = ink_in_rect(&g, 13, 26, 19, 31);
        assert!(top > 800, "A should have a peak at the top, got {top}");
        assert!(mid > top, "A crossbar should be stronger than the peak");
        assert!(bot_l > 400 && bot_r > 400, "A should stand on two legs");
        assert!(bot_c < bot_l.min(bot_r), "A crotch should be open at the bottom");
    }

    #[test]
    fn letter_t_has_bar_on_top() {
        let g = raster_glyph(20);
        let top = ink_in_rect(&g, 2, 2, 30, 8);
        let bot = ink_in_rect(&g, 2, 24, 30, 31);
        let stem = ink_in_rect(&g, 13, 8, 19, 30);
        assert!(top > bot * 2, "T bar belongs at the top, not the bottom");
        assert!(stem > 500);
    }

    #[test]
    fn colon_is_two_stacked_dots() {
        let g = raster_glyph(27);
        let upper = ink_in_rect(&g, 10, 6, 22, 14);
        let lower = ink_in_rect(&g, 10, 18, 22, 26);
        let left = ink_in_rect(&g, 0, 0, 8, 32);
        assert!(upper > 200 && lower > 200);
        assert!(left < 80, "colon should not look like a sideways pair");
    }

    #[test]
    fn space_is_empty() {
        let g = raster_glyph(0);
        assert!(g.iter().all(|&p| p == 0));
    }
}
