//! Slice bake: PNG density view, optional heightmap, density mask, SVG contours, JSON meta.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::cutter::{Axis, CaptureMode, Cutter};
use crate::export_settings::{ExportAspect, ExportSettings, FitMode};

#[derive(Clone, Debug)]
pub struct VolumeFields {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub biomass: Vec<f32>,
    pub soluble_c: Vec<f32>,
}

impl VolumeFields {
    pub fn idx(&self, x: u32, y: u32, z: u32) -> usize {
        let w = self.width as usize;
        let h = self.height as usize;
        (z as usize * h + y as usize) * w + x as usize
    }

    pub fn sample(&self, x: i32, y: i32, z: i32) -> (f32, f32) {
        let w = self.width as i32;
        let h = self.height as i32;
        let d = self.depth as i32;
        if w <= 0 || h <= 0 || d <= 0 {
            return (0.0, 0.0);
        }
        let x = ((x % w) + w) % w;
        let y = ((y % h) + h) % h;
        let z = z.clamp(0, d - 1);
        let i = self.idx(x as u32, y as u32, z as u32);
        (self.biomass[i], self.soluble_c[i])
    }
}

#[derive(Clone, Debug)]
pub struct SlicePlane {
    pub axis: Axis,
    pub u_axis: Axis,
    pub v_axis: Axis,
    pub width: u32,
    pub height: u32,
    pub center: f32,
    pub lo: i32,
    pub hi: i32,
    pub biomass: Vec<f32>,
    pub soluble: Vec<f32>,
    /// Coordinate of the strongest sample along the heightmap axis, normalized 0..1.
    pub height_along: Vec<f32>,
    pub volume_samples: Vec<OccupiedSample>,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct OccupiedSample {
    pub u: u32,
    pub v: u32,
    pub t: i32,
    pub d: f32,
}

#[derive(Serialize)]
struct SliceJson {
    format: &'static str,
    timestamp: String,
    mode: &'static str,
    axis: &'static str,
    center_voxel: f32,
    thickness_voxels: f32,
    lo_voxel: i32,
    hi_voxel: i32,
    grid: GridMeta,
    plane: PlaneMeta,
    heightmap_axis: &'static str,
    heightmap: bool,
    export_aspect: &'static str,
    export_fit: &'static str,
    export_preset: u32,
    source_width: u32,
    source_height: u32,
    stats: SliceStats,
    density_encoding: &'static str,
    density_u8: Vec<u8>,
    samples: Vec<OccupiedSample>,
}

#[derive(Serialize)]
struct GridMeta {
    width: u32,
    height: u32,
    depth: u32,
}

#[derive(Serialize)]
struct PlaneMeta {
    u: &'static str,
    v: &'static str,
    width: u32,
    height: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct SliceStats {
    pub min: f32,
    pub max: f32,
    pub mean: f32,
    pub occupied: u32,
    pub threshold: f32,
}

#[derive(Clone, Debug)]
pub struct ExportPaths {
    pub png: PathBuf,
    pub json: PathBuf,
    pub svg: PathBuf,
    pub mask: PathBuf,
    pub height: Option<PathBuf>,
}

pub fn extract_slice(
    vol: &VolumeFields,
    cutter: &Cutter,
    include_volume: bool,
) -> SlicePlane {
    let axis = cutter.axis;
    let (u_axis, v_axis) = axis.plane_uv();
    let width = u_axis.size(vol.width, vol.height, vol.depth);
    let height = v_axis.size(vol.width, vol.height, vol.depth);
    let axis_n = axis.size(vol.width, vol.height, vol.depth) as f32;
    let center = cutter.pos * axis_n;
    let n_layers = cutter.thickness_voxels().round().max(1.0) as i32;
    let mid = center.round() as i32;
    let lo = (mid - (n_layers - 1) / 2).max(0);
    let hi = (lo + n_layers - 1).min(axis_n as i32 - 1).max(lo);

    let n = (width as usize).saturating_mul(height as usize);
    let mut biomass = vec![0.0f32; n];
    let mut soluble = vec![0.0f32; n];
    let mut height_along = vec![0.0f32; n];
    let mut volume_samples = Vec::new();
    let keep_volume = include_volume || cutter.mode == CaptureMode::RichBox;
    const MAX_SAMPLES: usize = 80_000;

    for v in 0..height {
        for u in 0..width {
            let i = (v * width + u) as usize;
            let mut best = 0.0f32;
            let mut best_sol = 0.0f32;
            let mut best_h = 0.0f32;
            for t in lo..=hi {
                let (x, y, z) = voxel_at(axis, u, v, t);
                let (b, s) = vol.sample(x, y, z);
                if keep_volume && b > 0.02 && volume_samples.len() < MAX_SAMPLES {
                    volume_samples.push(OccupiedSample {
                        u,
                        v,
                        t,
                        d: (b * 1000.0).round() / 1000.0,
                    });
                }
                if b >= best {
                    best = b;
                    best_sol = s;
                    let p = [
                        x as f32 / vol.width.max(1) as f32,
                        y as f32 / vol.height.max(1) as f32,
                        z as f32 / vol.depth.max(1) as f32,
                    ];
                    best_h = match cutter.heightmap_axis {
                        Axis::X => p[0],
                        Axis::Y => p[1],
                        Axis::Z => p[2],
                    };
                }
            }
            biomass[i] = best;
            soluble[i] = best_sol;
            height_along[i] = best_h;
        }
    }

    SlicePlane {
        axis,
        u_axis,
        v_axis,
        width,
        height,
        center,
        lo,
        hi,
        biomass,
        soluble,
        height_along,
        volume_samples,
    }
}

fn voxel_at(axis: Axis, u: u32, v: u32, t: i32) -> (i32, i32, i32) {
    match axis {
        Axis::X => (t, u as i32, v as i32),
        Axis::Y => (u as i32, t, v as i32),
        Axis::Z => (u as i32, v as i32, t),
    }
}

pub fn plane_stats(plane: &SlicePlane) -> SliceStats {
    let mut min = f32::INFINITY;
    let mut max = 0.0f32;
    let mut sum = 0.0f32;
    let mut occupied = 0u32;
    for &b in &plane.biomass {
        min = min.min(b);
        max = max.max(b);
        sum += b;
        if b > 0.02 {
            occupied += 1;
        }
    }
    if plane.biomass.is_empty() {
        min = 0.0;
    }
    let n = plane.biomass.len().max(1) as f32;
    let threshold = (max * 0.15).max(0.02);
    SliceStats {
        min,
        max,
        mean: sum / n,
        occupied,
        threshold,
    }
}

/// Crop to the occupied density AABB and/or letterbox into a square POT canvas.
pub fn apply_export_layout(plane: &SlicePlane, settings: &ExportSettings) -> SlicePlane {
    let work = match settings.fit {
        FitMode::CropAabb => crop_to_dense_aabb(plane),
        FitMode::Pad => plane.clone(),
    };
    match settings.aspect {
        ExportAspect::Square => letterbox_to(&work, settings.square_size(), settings.square_size()),
        ExportAspect::Native => work,
    }
}

fn dense_threshold(plane: &SlicePlane) -> f32 {
    let max_b = plane.biomass.iter().copied().fold(0.0f32, f32::max);
    (max_b * 0.15).max(0.02)
}

fn crop_to_dense_aabb(plane: &SlicePlane) -> SlicePlane {
    let t = dense_threshold(plane);
    let mut x0 = plane.width;
    let mut y0 = plane.height;
    let mut x1 = 0u32;
    let mut y1 = 0u32;
    for v in 0..plane.height {
        for u in 0..plane.width {
            let i = (v * plane.width + u) as usize;
            if plane.biomass[i] > t {
                x0 = x0.min(u);
                y0 = y0.min(v);
                x1 = x1.max(u);
                y1 = y1.max(v);
            }
        }
    }
    if x0 > x1 {
        return plane.clone();
    }
    let x0 = x0.saturating_sub(1);
    let y0 = y0.saturating_sub(1);
    let x1 = (x1 + 1).min(plane.width.saturating_sub(1));
    let y1 = (y1 + 1).min(plane.height.saturating_sub(1));
    crop_rect(plane, x0, y0, x1, y1)
}

fn crop_rect(plane: &SlicePlane, x0: u32, y0: u32, x1: u32, y1: u32) -> SlicePlane {
    let width = x1.saturating_sub(x0) + 1;
    let height = y1.saturating_sub(y0) + 1;
    let n = (width as usize).saturating_mul(height as usize);
    let mut biomass = vec![0.0f32; n];
    let mut soluble = vec![0.0f32; n];
    let mut height_along = vec![0.0f32; n];
    for v in 0..height {
        for u in 0..width {
            let src = ((y0 + v) * plane.width + (x0 + u)) as usize;
            let dst = (v * width + u) as usize;
            biomass[dst] = plane.biomass[src];
            soluble[dst] = plane.soluble[src];
            height_along[dst] = plane.height_along[src];
        }
    }
    let mut out = plane.clone();
    out.width = width;
    out.height = height;
    out.biomass = biomass;
    out.soluble = soluble;
    out.height_along = height_along;
    out
}

fn letterbox_to(plane: &SlicePlane, dst_w: u32, dst_h: u32) -> SlicePlane {
    if dst_w == 0 || dst_h == 0 {
        return plane.clone();
    }
    if plane.width == dst_w && plane.height == dst_h {
        return plane.clone();
    }
    let sw = plane.width.max(1) as f32;
    let sh = plane.height.max(1) as f32;
    let scale = (dst_w as f32 / sw).min(dst_h as f32 / sh);
    let nw = sw * scale;
    let nh = sh * scale;
    let ox = (dst_w as f32 - nw) * 0.5;
    let oy = (dst_h as f32 - nh) * 0.5;
    let n = (dst_w as usize).saturating_mul(dst_h as usize);
    let mut biomass = vec![0.0f32; n];
    let mut soluble = vec![0.0f32; n];
    let mut height_along = vec![0.0f32; n];
    for v in 0..dst_h {
        for u in 0..dst_w {
            let sx = (u as f32 + 0.5 - ox) / scale - 0.5;
            let sy = (v as f32 + 0.5 - oy) / scale - 0.5;
            let i = (v * dst_w + u) as usize;
            if sx < -0.5 || sy < -0.5 || sx > sw - 0.5 || sy > sh - 0.5 {
                continue;
            }
            biomass[i] = sample_bilinear(&plane.biomass, plane.width, plane.height, sx, sy);
            soluble[i] = sample_bilinear(&plane.soluble, plane.width, plane.height, sx, sy);
            height_along[i] =
                sample_bilinear(&plane.height_along, plane.width, plane.height, sx, sy);
        }
    }
    let mut out = plane.clone();
    out.width = dst_w;
    out.height = dst_h;
    out.biomass = biomass;
    out.soluble = soluble;
    out.height_along = height_along;
    out
}

fn sample_bilinear(field: &[f32], w: u32, h: u32, x: f32, y: f32) -> f32 {
    if w == 0 || h == 0 || field.is_empty() {
        return 0.0;
    }
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let tx = (x - x0 as f32).clamp(0.0, 1.0);
    let ty = (y - y0 as f32).clamp(0.0, 1.0);
    let at = |ix: i32, iy: i32| -> f32 {
        if ix < 0 || iy < 0 || ix >= w as i32 || iy >= h as i32 {
            return 0.0;
        }
        field[(iy as u32 * w + ix as u32) as usize]
    };
    let a = at(x0, y0);
    let b = at(x0 + 1, y0);
    let c = at(x0, y0 + 1);
    let d = at(x0 + 1, y0 + 1);
    let top = a * (1.0 - tx) + b * tx;
    let bot = c * (1.0 - tx) + d * tx;
    top * (1.0 - ty) + bot * ty
}

pub fn write_exports(
    dir: &Path,
    plane: &SlicePlane,
    cutter: &Cutter,
    grid: (u32, u32, u32),
    stamp: &str,
) -> Result<ExportPaths> {
    write_exports_with_meta(dir, plane, cutter, grid, stamp, None, (plane.width, plane.height))
}

pub fn write_exports_with_meta(
    dir: &Path,
    plane: &SlicePlane,
    cutter: &Cutter,
    grid: (u32, u32, u32),
    stamp: &str,
    settings: Option<&ExportSettings>,
    source: (u32, u32),
) -> Result<ExportPaths> {
    fs::create_dir_all(dir).with_context(|| format!("create {}", dir.display()))?;
    let stem = format!(
        "pycelium_{}_{}_{}",
        stamp,
        cutter.axis.as_str(),
        cutter.mode.as_str()
    );
    let png = dir.join(format!("{stem}.png"));
    let json = dir.join(format!("{stem}.json"));
    let svg = dir.join(format!("{stem}.svg"));
    let mask = dir.join(format!("{stem}_mask.png"));
    let height = if cutter.export_heightmap {
        Some(dir.join(format!("{stem}_height.png")))
    } else {
        None
    };

    write_density_png(&png, plane)?;
    write_mask_png(&mask, plane)?;
    if let Some(ref hpath) = height {
        write_height_png(hpath, plane)?;
    }
    write_svg(&svg, plane)?;
    write_json(&json, plane, cutter, grid, stamp, settings, source)?;

    Ok(ExportPaths {
        png,
        json,
        svg,
        mask,
        height,
    })
}

pub fn utc_stamp(now: SystemTime) -> String {
    let secs = now.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let (y, m, d, hh, mm, ss) = civil_from_unix(secs);
    format!("{y:04}{m:02}{d:02}_{hh:02}{mm:02}{ss:02}")
}

/// Howard Hinnant civil-from-days (UTC).
fn civil_from_unix(secs: u64) -> (i32, u32, u32, u32, u32, u32) {
    let z = (secs / 86400) as i64 + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = y + if m <= 2 { 1 } else { 0 };
    let rem = secs % 86400;
    (
        y as i32,
        m as u32,
        d as u32,
        (rem / 3600) as u32,
        ((rem % 3600) / 60) as u32,
        (rem % 60) as u32,
    )
}

fn write_density_png(path: &Path, plane: &SlicePlane) -> Result<()> {
    let mut rgba = vec![0u8; (plane.width * plane.height * 4) as usize];
    for v in 0..plane.height {
        for u in 0..plane.width {
            let i = (v * plane.width + u) as usize;
            let hy = 1.0 - (-plane.biomass[i] * 3.0).exp();
            let food = 1.0 - (-plane.soluble[i] * 3.2).exp();
            let o = i * 4;
            rgba[o] = ((0.04 + 0.58 * hy + 0.70 * food) * 255.0).clamp(0.0, 255.0) as u8;
            rgba[o + 1] = ((0.045 + 0.90 * hy + 0.28 * food) * 255.0).clamp(0.0, 255.0) as u8;
            rgba[o + 2] = ((0.05 + 0.80 * hy + 0.06 * food) * 255.0).clamp(0.0, 255.0) as u8;
            rgba[o + 3] = 255;
        }
    }
    write_png(path, plane.width, plane.height, png::ColorType::Rgba, &rgba)
}

fn write_mask_png(path: &Path, plane: &SlicePlane) -> Result<()> {
    let max_b = plane.biomass.iter().copied().fold(0.0f32, f32::max).max(1e-6);
    let mut gray = vec![0u8; (plane.width * plane.height) as usize];
    for (i, &b) in plane.biomass.iter().enumerate() {
        gray[i] = ((b / max_b) * 255.0).clamp(0.0, 255.0) as u8;
    }
    write_png(path, plane.width, plane.height, png::ColorType::Grayscale, &gray)
}

fn write_height_png(path: &Path, plane: &SlicePlane) -> Result<()> {
    let stats = plane_stats(plane);
    let mut gray = vec![0u8; (plane.width * plane.height) as usize];
    for (i, &b) in plane.biomass.iter().enumerate() {
        gray[i] = if b > stats.threshold {
            (plane.height_along[i] * 255.0).clamp(0.0, 255.0) as u8
        } else {
            0
        };
    }
    write_png(path, plane.width, plane.height, png::ColorType::Grayscale, &gray)
}

fn write_png(
    path: &Path,
    width: u32,
    height: u32,
    color: png::ColorType,
    data: &[u8],
) -> Result<()> {
    let file = fs::File::create(path).with_context(|| format!("write {}", path.display()))?;
    let mut encoder = png::Encoder::new(file, width, height);
    encoder.set_color(color);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().context("png header")?;
    writer.write_image_data(data).context("png pixels")?;
    Ok(())
}

fn png_size(path: &Path) -> Result<(u32, u32)> {
    let file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let decoder = png::Decoder::new(file);
    let reader = decoder.read_info().context("png info")?;
    let info = reader.info();
    Ok((info.width, info.height))
}

fn write_json(
    path: &Path,
    plane: &SlicePlane,
    cutter: &Cutter,
    grid: (u32, u32, u32),
    stamp: &str,
    settings: Option<&ExportSettings>,
    source: (u32, u32),
) -> Result<()> {
    let stats = plane_stats(plane);
    let max_b = stats.max.max(1e-6);
    let density_u8 = plane
        .biomass
        .iter()
        .map(|b| ((b / max_b) * 255.0).clamp(0.0, 255.0) as u8)
        .collect();
    let samples = if cutter.mode == CaptureMode::RichBox {
        plane.volume_samples.clone()
    } else {
        Vec::new()
    };
    let (export_aspect, export_fit, export_preset) = match settings {
        Some(s) => (s.aspect.as_str(), s.fit.as_str(), s.square_size()),
        None => ("native", "pad", 0),
    };
    let doc = SliceJson {
        format: "pycelium-slice-v1",
        timestamp: stamp.to_string(),
        mode: cutter.mode.as_str(),
        axis: cutter.axis.as_str(),
        center_voxel: plane.center,
        thickness_voxels: cutter.thickness_voxels(),
        lo_voxel: plane.lo,
        hi_voxel: plane.hi,
        grid: GridMeta {
            width: grid.0,
            height: grid.1,
            depth: grid.2,
        },
        plane: PlaneMeta {
            u: plane.u_axis.as_str(),
            v: plane.v_axis.as_str(),
            width: plane.width,
            height: plane.height,
        },
        heightmap_axis: cutter.heightmap_axis.as_str(),
        heightmap: cutter.export_heightmap,
        export_aspect,
        export_fit,
        export_preset,
        source_width: source.0,
        source_height: source.1,
        stats,
        density_encoding: "u8_row_major",
        density_u8,
        samples,
    };
    let text = serde_json::to_string_pretty(&doc).context("json")?;
    fs::write(path, text).with_context(|| format!("write {}", path.display()))
}

fn write_svg(path: &Path, plane: &SlicePlane) -> Result<()> {
    let stats = plane_stats(plane);
    let lines = marching_segments(
        plane.width,
        plane.height,
        &plane.biomass,
        stats.threshold,
    );
    let mut out = String::new();
    out.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {} {}\" width=\"{}\" height=\"{}\" fill=\"none\">\n",
        plane.width, plane.height, plane.width, plane.height
    ));
    out.push_str("  <rect width=\"100%\" height=\"100%\" fill=\"#0a0c0d\"/>\n");
    for (a, b) in lines {
        out.push_str(&format!(
            "  <line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"#9ef0d8\" stroke-width=\"0.8\"/>\n",
            a.0, a.1, b.0, b.1
        ));
    }
    out.push_str("</svg>\n");
    let mut f = fs::File::create(path).with_context(|| format!("write {}", path.display()))?;
    f.write_all(out.as_bytes())?;
    Ok(())
}

/// Marching-squares line segments at `threshold` on a row-major field.
pub fn marching_segments(
    w: u32,
    h: u32,
    field: &[f32],
    threshold: f32,
) -> Vec<((f32, f32), (f32, f32))> {
    let mut lines = Vec::new();
    if w < 2 || h < 2 {
        return lines;
    }
    let at = |x: u32, y: u32| field[(y * w + x) as usize];
    for y in 0..h - 1 {
        for x in 0..w - 1 {
            let v0 = at(x, y);
            let v1 = at(x + 1, y);
            let v2 = at(x + 1, y + 1);
            let v3 = at(x, y + 1);
            let mut code = 0u8;
            if v0 >= threshold {
                code |= 1;
            }
            if v1 >= threshold {
                code |= 2;
            }
            if v2 >= threshold {
                code |= 4;
            }
            if v3 >= threshold {
                code |= 8;
            }
            if code == 0 || code == 15 {
                continue;
            }
            let fx = x as f32;
            let fy = y as f32;
            let lerp = |a: f32, b: f32| {
                let d = b - a;
                if d.abs() < 1e-6 {
                    0.5
                } else {
                    ((threshold - a) / d).clamp(0.0, 1.0)
                }
            };
            let top = (fx + lerp(v0, v1), fy);
            let right = (fx + 1.0, fy + lerp(v1, v2));
            let bottom = (fx + lerp(v3, v2), fy + 1.0);
            let left = (fx, fy + lerp(v0, v3));
            let pair = match code {
                1 | 14 => Some((left, top)),
                2 | 13 => Some((top, right)),
                3 | 12 => Some((left, right)),
                4 | 11 => Some((right, bottom)),
                6 | 9 => Some((top, bottom)),
                7 | 8 => Some((left, bottom)),
                5 => {
                    lines.push((left, top));
                    lines.push((right, bottom));
                    None
                }
                10 => {
                    lines.push((top, right));
                    lines.push((left, bottom));
                    None
                }
                _ => None,
            };
            if let Some(seg) = pair {
                lines.push(seg);
            }
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cutter::Cutter;

    fn brick(w: u32, h: u32, d: u32, mark: (u32, u32, u32), val: f32) -> VolumeFields {
        let n = (w * h * d) as usize;
        let mut vol = VolumeFields {
            width: w,
            height: h,
            depth: d,
            biomass: vec![0.0f32; n],
            soluble_c: vec![0.0f32; n],
        };
        let i = vol.idx(mark.0, mark.1, mark.2);
        vol.biomass[i] = val;
        vol
    }

    #[test]
    fn z_squash_finds_marked_voxel() {
        let vol = brick(8, 8, 8, (3, 4, 5), 1.5);
        let mut c = Cutter::new(8);
        c.enter(5.0, 8);
        c.axis = Axis::Z;
        c.pos = 5.0 / 8.0;
        c.half_vox = 0.5;
        let plane = extract_slice(&vol, &c, false);
        assert_eq!(plane.width, 8);
        assert_eq!(plane.height, 8);
        assert_eq!(plane.axis, Axis::Z);
        assert_eq!(plane.u_axis, Axis::X);
        assert_eq!(plane.v_axis, Axis::Y);
        let i = (4 * 8 + 3) as usize;
        assert!((plane.biomass[i] - 1.5).abs() < 1e-5);
        assert!(plane.biomass.iter().filter(|b| **b > 0.0).count() == 1);
    }

    #[test]
    fn x_axis_vertical_slice_maps_yz() {
        let vol = brick(8, 6, 4, (2, 1, 3), 0.8);
        let mut c = Cutter::new(4);
        c.enter(2.0, 4);
        c.axis = Axis::X;
        c.pos = 2.0 / 8.0;
        c.half_vox = 0.5;
        let plane = extract_slice(&vol, &c, false);
        assert_eq!(plane.width, 6);
        assert_eq!(plane.height, 4);
        let i = (3 * 6 + 1) as usize;
        assert!((plane.biomass[i] - 0.8).abs() < 1e-5);
    }

    #[test]
    fn squash_thickens_across_nearby_layers() {
        let mut vol = brick(4, 4, 8, (1, 1, 2), 0.4);
        let i = vol.idx(1, 1, 4);
        vol.biomass[i] = 1.2;
        let mut c = Cutter::new(8);
        c.enter(3.0, 8);
        c.pos = 3.0 / 8.0;
        c.half_vox = 0.5;
        let thin = extract_slice(&vol, &c, false);
        c.half_vox = 2.0;
        let thick = extract_slice(&vol, &c, false);
        let p = (1 * 4 + 1) as usize;
        assert!(thin.biomass[p] < 0.01);
        assert!((thick.biomass[p] - 1.2).abs() < 1e-5);
    }

    #[test]
    fn rich_mode_keeps_occupied_volume_samples() {
        let vol = brick(4, 4, 4, (1, 2, 2), 0.5);
        let mut c = Cutter::new(4);
        c.enter(2.0, 4);
        c.mode = CaptureMode::RichBox;
        c.pos = 0.5;
        c.half_vox = 1.0;
        let plane = extract_slice(&vol, &c, true);
        assert!(plane.volume_samples.iter().any(|s| s.u == 1 && s.v == 2));
    }

    #[test]
    fn heightmap_follows_chosen_axis() {
        let vol = brick(4, 4, 8, (1, 1, 6), 1.0);
        let mut c = Cutter::new(8);
        c.enter(6.0, 8);
        c.pos = 6.0 / 8.0;
        c.export_heightmap = true;
        c.heightmap_axis = Axis::Z;
        let plane = extract_slice(&vol, &c, false);
        let i = (1 * 4 + 1) as usize;
        assert!((plane.height_along[i] - 6.0 / 8.0).abs() < 1e-5);
    }

    #[test]
    fn marching_squares_boxes_a_block() {
        let mut field = vec![0.0f32; 16];
        field[5] = 1.0;
        field[6] = 1.0;
        field[9] = 1.0;
        field[10] = 1.0;
        let segs = marching_segments(4, 4, &field, 0.5);
        assert!(segs.len() >= 4);
    }

    #[test]
    fn write_bundle_creates_png_json_svg_mask() {
        let dir = std::env::temp_dir().join(format!(
            "pycelium-slice-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        let vol = brick(8, 8, 4, (2, 3, 1), 1.0);
        let mut c = Cutter::new(4);
        c.enter(1.0, 4);
        c.pos = 0.25;
        c.export_heightmap = true;
        let plane = extract_slice(&vol, &c, false);
        let paths = write_exports(&dir, &plane, &c, (8, 8, 4), "20260102_030405").unwrap();
        assert!(paths.png.exists());
        assert!(paths.json.exists());
        assert!(paths.svg.exists());
        assert!(paths.mask.exists());
        assert!(paths.height.unwrap().exists());
        let json = fs::read_to_string(&paths.json).unwrap();
        assert!(json.contains("pycelium-slice-v1"));
        assert!(json.contains("\"axis\": \"z\""));
        let svg = fs::read_to_string(&paths.svg).unwrap();
        assert!(svg.contains("<svg"));
        let (pw, ph) = png_size(&paths.png).unwrap();
        assert_eq!((pw, ph), (8, 8));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn utc_stamp_is_compact() {
        let t = UNIX_EPOCH + std::time::Duration::from_secs(1_704_067_200); // 2024-01-01 00:00:00 UTC
        let s = utc_stamp(t);
        assert_eq!(s, "20240101_000000");
    }

    #[test]
    fn stats_count_occupied() {
        let vol = brick(4, 4, 4, (0, 0, 0), 0.3);
        let mut c = Cutter::new(4);
        c.enter(0.0, 4);
        c.pos = 0.0;
        let plane = extract_slice(&vol, &c, false);
        let s = plane_stats(&plane);
        assert_eq!(s.occupied, 1);
        assert!(s.max >= 0.3);
    }

    #[test]
    fn default_layout_is_512_square() {
        let vol = brick(8, 6, 4, (2, 3, 1), 1.0);
        let mut c = Cutter::new(4);
        c.enter(1.0, 4);
        c.axis = Axis::Z;
        c.pos = 0.25;
        let plane = extract_slice(&vol, &c, false);
        assert_eq!((plane.width, plane.height), (8, 6));
        let out = apply_export_layout(&plane, &ExportSettings::default());
        assert_eq!((out.width, out.height), (512, 512));
        let i = (256 * 512 + 256) as usize;
        // Marked voxel (2,3) maps near the center of an 8×6 letterbox.
        assert!(out.biomass.iter().any(|&b| b > 0.2), "hypha should survive upsample");
        assert!(out.biomass[i] < 1.5);
    }

    #[test]
    fn native_aspect_keeps_slab_size() {
        let vol = brick(8, 6, 4, (2, 1, 1), 1.0);
        let mut c = Cutter::new(4);
        c.enter(1.0, 4);
        c.axis = Axis::Z;
        c.pos = 0.25;
        let plane = extract_slice(&vol, &c, false);
        let mut settings = ExportSettings::default();
        settings.aspect = ExportAspect::Native;
        let out = apply_export_layout(&plane, &settings);
        assert_eq!((out.width, out.height), (8, 6));
    }

    #[test]
    fn crop_aabb_then_square_pads_occupied_region() {
        let vol = brick(16, 16, 4, (2, 2, 1), 1.0);
        let mut c = Cutter::new(4);
        c.enter(1.0, 4);
        c.axis = Axis::Z;
        c.pos = 0.25;
        let plane = extract_slice(&vol, &c, false);
        let mut settings = ExportSettings::default();
        settings.fit = FitMode::CropAabb;
        settings.preset_index = 2; // 32×32
        let out = apply_export_layout(&plane, &settings);
        assert_eq!((out.width, out.height), (32, 32));
        let occupied = out.biomass.iter().filter(|b| **b > 0.2).count();
        assert!(occupied > 0);
        // Crop should fill more of the canvas than padding the whole 16×16.
        settings.fit = FitMode::Pad;
        let padded = apply_export_layout(&plane, &settings);
        let pad_occ = padded.biomass.iter().filter(|b| **b > 0.2).count();
        assert!(occupied >= pad_occ);
    }

    #[test]
    fn write_bundle_records_export_meta() {
        let dir = std::env::temp_dir().join(format!(
            "pycelium-slice-meta-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        let vol = brick(8, 8, 4, (2, 3, 1), 1.0);
        let mut c = Cutter::new(4);
        c.enter(1.0, 4);
        c.pos = 0.25;
        let plane = extract_slice(&vol, &c, false);
        let settings = ExportSettings::default();
        let composed = apply_export_layout(&plane, &settings);
        let paths = write_exports_with_meta(
            &dir,
            &composed,
            &c,
            (8, 8, 4),
            "20260102_030405",
            Some(&settings),
            (plane.width, plane.height),
        )
        .unwrap();
        let (pw, ph) = png_size(&paths.png).unwrap();
        assert_eq!((pw, ph), (512, 512));
        let json = fs::read_to_string(&paths.json).unwrap();
        assert!(json.contains("\"export_aspect\": \"square\""));
        assert!(json.contains("\"export_fit\": \"pad\""));
        assert!(json.contains("\"export_preset\": 512"));
        assert!(json.contains("\"source_width\": 8"));
        let _ = fs::remove_dir_all(&dir);
    }
}
