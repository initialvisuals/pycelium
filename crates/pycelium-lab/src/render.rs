use pycelium_core::Grid;

/// Downsample occupancy + nutrient into a terminal slice.
///
/// ` ` empty, `.` nutrient, `#` hypha, `@` tip.
pub fn render_ascii(occupancy: &Grid, nutrient: &Grid, max_w: u32, max_h: u32) -> String {
    let src_w = occupancy.width().max(1);
    let src_h = occupancy.height().max(1);
    let view_w = max_w.min(src_w);
    let view_h = max_h.min(src_h);

    let mut out = String::with_capacity((view_w as usize + 1) * view_h as usize + 64);
    out.push_str(&format!(
        "slice {view_w}x{view_h} of {src_w}x{src_h}  (.=nutrient  #=hypha  @=tip)\n"
    ));

    for vy in 0..view_h {
        let y0 = (vy as u64 * src_h as u64 / view_h as u64) as u32;
        let y1 = (((vy as u64 + 1) * src_h as u64 / view_h as u64) as u32).max(y0 + 1);
        for vx in 0..view_w {
            let x0 = (vx as u64 * src_w as u64 / view_w as u64) as u32;
            let x1 = (((vx as u64 + 1) * src_w as u64 / view_w as u64) as u32).max(x0 + 1);
            out.push(sample_glyph(occupancy, nutrient, x0, x1, y0, y1));
        }
        out.push('\n');
    }
    out
}

fn sample_glyph(occupancy: &Grid, nutrient: &Grid, x0: u32, x1: u32, y0: u32, y1: u32) -> char {
    let mut best_occ = 0u8;
    let mut best_nut = 0u8;
    for y in y0..y1.min(occupancy.height()) {
        for x in x0..x1.min(occupancy.width()) {
            best_occ = best_occ.max(occupancy.get(x, y));
            best_nut = best_nut.max(nutrient.get(x, y));
        }
    }
    if best_occ == 255 {
        '@'
    } else if best_occ > 0 {
        '#'
    } else if best_nut > 0 {
        '.'
    } else {
        ' '
    }
}

/// Binary PGM (P5) of occupancy. Tips are white, hypha mid-grey, empty black.
pub fn write_occupancy_pgm(occupancy: &Grid) -> Vec<u8> {
    let w = occupancy.width();
    let h = occupancy.height();
    let header = format!("P5\n{w} {h}\n255\n");
    let mut out = Vec::with_capacity(header.len() + occupancy.len());
    out.extend_from_slice(header.as_bytes());
    for &cell in occupancy.as_slice() {
        let v = if cell == 255 {
            255
        } else if cell > 0 {
            160
        } else {
            0
        };
        out.push(v);
    }
    out
}

#[cfg(test)]
mod tests {
    use pycelium_core::{SimConfig, World};

    use super::{render_ascii, write_occupancy_pgm};

    #[test]
    fn ascii_slice_has_expected_rows() {
        let cfg = SimConfig {
            width: 32,
            height: 16,
            seed: 1,
            initial_tips: 2,
            nutrient_patches: 4,
            patch_radius: 3,
            ..SimConfig::default()
        };
        let mut world = World::new(cfg).unwrap();
        world.run_experiment(12);
        let text = render_ascii(world.occupancy(), world.nutrient(), 16, 8);
        let rows = text.lines().count();
        // caption + 8 slice rows
        assert_eq!(rows, 9);
        assert!(text.contains('#') || text.contains('@'));
    }

    #[test]
    fn pgm_header_and_payload_match_grid() {
        let cfg = SimConfig {
            width: 10,
            height: 6,
            seed: 4,
            ..SimConfig::default()
        };
        let world = World::new(cfg).unwrap();
        let bytes = write_occupancy_pgm(world.occupancy());
        let header = b"P5\n10 6\n255\n";
        assert!(bytes.starts_with(header));
        assert_eq!(bytes.len(), header.len() + 10 * 6);
    }
}
