//! Capture-mode cutter: face-aligned plane / box and the two-ended thickness slider.
//!
//! The mesocosm is a unit cube. Hovering a face center snaps the cutter to that
//! face's normal (horizontal or vertical). Hovering a face mid-edge rotates 90°
//! onto the free axis so vertical structures can be captured, not only Z slabs.

use std::fmt;

/// World-space slider (UV) for the two-ended thickness control.
pub const SLIDER_X0: f32 = 0.30;
pub const SLIDER_X1: f32 = 0.70;
pub const SLIDER_Y0: f32 = 0.900;
pub const SLIDER_Y1: f32 = 0.958;
const HANDLE_PAD: f32 = 0.018;
const FACE_CENTER_SNAP: f32 = 0.20;
const FACE_EDGE_SNAP: f32 = 0.14;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    X = 0,
    Y = 1,
    Z = 2,
}

impl Axis {
    pub fn as_f32(self) -> f32 {
        self as u8 as f32
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::X => "x",
            Self::Y => "y",
            Self::Z => "z",
        }
    }

    pub fn cycle(self) -> Self {
        match self {
            Self::X => Self::Y,
            Self::Y => Self::Z,
            Self::Z => Self::X,
        }
    }

    pub fn size(self, w: u32, h: u32, d: u32) -> u32 {
        match self {
            Self::X => w,
            Self::Y => h,
            Self::Z => d,
        }
    }

    /// Plane axes (u, v) for a slice whose normal is `self`.
    pub fn plane_uv(self) -> (Self, Self) {
        match self {
            Self::X => (Self::Y, Self::Z),
            Self::Y => (Self::X, Self::Z),
            Self::Z => (Self::X, Self::Y),
        }
    }
}

impl fmt::Display for Axis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureMode {
    /// Max-project N layers onto one plane (starts at 1 layer).
    Squash2D,
    /// Thin box between two parallel planes; JSON keeps the volume slab.
    RichBox,
}

impl CaptureMode {
    pub fn cycle(self) -> Self {
        match self {
            Self::Squash2D => Self::RichBox,
            Self::RichBox => Self::Squash2D,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Squash2D => "squash",
            Self::RichBox => "rich",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Handle {
    Neg,
    Pos,
}

#[derive(Clone, Debug)]
pub struct Cutter {
    pub active: bool,
    pub mode: CaptureMode,
    pub axis: Axis,
    /// Normalized 0..1 position along `axis`.
    pub pos: f32,
    /// Half-thickness in voxels. 0.5 = one layer at the cursor.
    pub half_vox: f32,
    pub snapped: bool,
    pub dragging: Option<Handle>,
    pub export_heightmap: bool,
    pub heightmap_axis: Axis,
}

impl Cutter {
    pub fn new(depth: u32) -> Self {
        let z0 = (depth as f32 * 0.35).clamp(1.0, (depth as f32 - 2.0).max(1.0));
        let pos = if depth == 0 {
            0.35
        } else {
            z0 / depth as f32
        };
        Self {
            active: false,
            mode: CaptureMode::Squash2D,
            axis: Axis::Z,
            pos,
            half_vox: 0.5,
            snapped: false,
            dragging: None,
            export_heightmap: false,
            heightmap_axis: Axis::Z,
        }
    }

    pub fn enter(&mut self, slice_z: f32, depth: u32) {
        self.active = true;
        self.dragging = None;
        self.axis = Axis::Z;
        self.snapped = false;
        self.half_vox = 0.5;
        if depth > 0 {
            self.pos = (slice_z / depth as f32).clamp(0.0, 1.0);
        }
        self.heightmap_axis = self.axis;
    }

    pub fn leave(&mut self) {
        self.active = false;
        self.dragging = None;
        self.snapped = false;
    }

    pub fn toggle(&mut self, slice_z: f32, depth: u32) {
        if self.active {
            self.leave();
        } else {
            self.enter(slice_z, depth);
        }
    }

    pub fn cycle_mode(&mut self) {
        self.mode = self.mode.cycle();
    }

    pub fn cycle_axis(&mut self) {
        self.axis = self.axis.cycle();
        self.snapped = false;
        self.heightmap_axis = self.axis;
    }

    pub fn cycle_heightmap_axis(&mut self) {
        self.heightmap_axis = self.heightmap_axis.cycle();
        self.export_heightmap = true;
    }

    pub fn thickness_voxels(&self) -> f32 {
        (self.half_vox * 2.0).max(1.0)
    }

    pub fn half_norm(&self, axis_size: f32) -> f32 {
        if axis_size <= 0.0 {
            return 0.0;
        }
        (self.half_vox / axis_size).clamp(0.0, 0.5)
    }

    pub fn clamp_half(&mut self, axis_size: f32) {
        let max_half = (axis_size * 0.5).max(0.5);
        self.half_vox = self.half_vox.clamp(0.5, max_half);
        let hn = self.half_norm(axis_size);
        self.pos = self.pos.clamp(hn, 1.0 - hn);
    }

    pub fn nudge_half(&mut self, delta_vox: f32, axis_size: f32) {
        self.half_vox += delta_vox;
        self.clamp_half(axis_size);
    }

    /// Packed present uniform: axis, pos, half_vox, mode(+0.5 if snapped).
    pub fn present_vec(&self) -> [f32; 4] {
        if !self.active {
            return [2.0, self.pos, 0.5, 0.0];
        }
        let mode = match self.mode {
            CaptureMode::Squash2D => 1.0,
            CaptureMode::RichBox => 2.0,
        };
        let mode = if self.snapped { mode + 0.5 } else { mode };
        [self.axis.as_f32(), self.pos, self.half_vox, mode]
    }

    pub fn describe(&self) -> String {
        format!(
            "CAPTURE {} {} @ {:.0}%  thick {:.0}  heightmap {}/{}  Enter write  Esc leave",
            self.mode.as_str(),
            self.axis,
            self.pos * 100.0,
            self.thickness_voxels(),
            if self.export_heightmap { "on" } else { "off" },
            self.heightmap_axis
        )
    }

    pub fn hit_handle(&self, uv: (f32, f32), axis_size: f32) -> Option<Handle> {
        if !self.active || !in_slider_band(uv) {
            return None;
        }
        let hn = self.half_norm(axis_size);
        let neg = self.pos - hn;
        let pos = self.pos + hn;
        let t = slider_t(uv.0);
        if (t - neg).abs() <= HANDLE_PAD {
            return Some(Handle::Neg);
        }
        if (t - pos).abs() <= HANDLE_PAD {
            return Some(Handle::Pos);
        }
        None
    }

    pub fn drag_handle(&mut self, uv: (f32, f32), axis_size: f32) {
        let Some(handle) = self.dragging else {
            return;
        };
        let t = slider_t(uv.0);
        let half_norm = match handle {
            Handle::Neg => (self.pos - t).abs(),
            Handle::Pos => (t - self.pos).abs(),
        };
        self.half_vox = (half_norm * axis_size).max(0.5);
        self.clamp_half(axis_size);
    }

    /// Mouse-move: snap axis from cube-face hover and place the plane at ray depth.
    pub fn track_pointer(&mut self, eye: [f32; 3], dir: [f32; 3], axis_size: f32) {
        if !self.active || self.dragging.is_some() {
            return;
        }
        let Some(seg) = ray_cube_segment(eye, dir) else {
            self.snapped = false;
            return;
        };
        if let Some(axis) = snap_axis_from_segment(eye, dir, seg) {
            self.axis = axis;
            self.snapped = true;
            self.heightmap_axis = axis;
        } else {
            self.snapped = false;
        }
        let mid = segment_midpoint(eye, dir, seg);
        self.pos = axis_component(mid, self.axis).clamp(0.0, 1.0);
        self.clamp_half(axis_size);
    }
}

fn in_slider_band(uv: (f32, f32)) -> bool {
    uv.0 >= SLIDER_X0 - 0.02
        && uv.0 <= SLIDER_X1 + 0.02
        && uv.1 >= SLIDER_Y0 - 0.01
        && uv.1 <= SLIDER_Y1 + 0.01
}

fn slider_t(x: f32) -> f32 {
    ((x - SLIDER_X0) / (SLIDER_X1 - SLIDER_X0)).clamp(0.0, 1.0)
}

fn axis_component(p: [f32; 3], axis: Axis) -> f32 {
    match axis {
        Axis::X => p[0],
        Axis::Y => p[1],
        Axis::Z => p[2],
    }
}

/// Midpoint of the ray–cube segment, in unit-cube space.
pub fn ray_cube_midpoint(ro: [f32; 3], rd: [f32; 3]) -> Option<[f32; 3]> {
    let seg = ray_cube_segment(ro, rd)?;
    Some(segment_midpoint(ro, rd, seg))
}

fn segment_midpoint(ro: [f32; 3], rd: [f32; 3], seg: (f32, f32)) -> [f32; 3] {
    let t = 0.5 * (seg.0.max(0.0) + seg.1);
    [
        ro[0] + rd[0] * t,
        ro[1] + rd[1] * t,
        ro[2] + rd[2] * t,
    ]
}

/// Ray vs unit cube. Returns (tmin, tmax) when the segment intersects.
pub fn ray_cube_segment(ro: [f32; 3], rd: [f32; 3]) -> Option<(f32, f32)> {
    let mut tmin = f32::NEG_INFINITY;
    let mut tmax = f32::INFINITY;
    for i in 0..3 {
        if rd[i].abs() < 1e-8 {
            if ro[i] < 0.0 || ro[i] > 1.0 {
                return None;
            }
            continue;
        }
        let mut t0 = (0.0 - ro[i]) / rd[i];
        let mut t1 = (1.0 - ro[i]) / rd[i];
        if t0 > t1 {
            std::mem::swap(&mut t0, &mut t1);
        }
        tmin = tmin.max(t0);
        tmax = tmax.min(t1);
        if tmax < tmin {
            return None;
        }
    }
    if tmax < 0.0 {
        return None;
    }
    Some((tmin, tmax))
}

fn point_on(ro: [f32; 3], rd: [f32; 3], t: f32) -> [f32; 3] {
    [
        ro[0] + rd[0] * t,
        ro[1] + rd[1] * t,
        ro[2] + rd[2] * t,
    ]
}

fn face_from_point(p: [f32; 3]) -> Option<(Axis, bool)> {
    const EPS: f32 = 2e-3;
    let mut best = None;
    let mut best_d = EPS;
    let faces = [
        (Axis::X, false, p[0]),
        (Axis::X, true, 1.0 - p[0]),
        (Axis::Y, false, p[1]),
        (Axis::Y, true, 1.0 - p[1]),
        (Axis::Z, false, p[2]),
        (Axis::Z, true, 1.0 - p[2]),
    ];
    for (axis, pos, dist) in faces {
        if dist <= best_d {
            best_d = dist;
            best = Some((axis, pos));
        }
    }
    best
}

fn face_uv(p: [f32; 3], axis: Axis) -> (f32, f32) {
    match axis {
        Axis::X => (p[1].clamp(0.0, 1.0), p[2].clamp(0.0, 1.0)),
        Axis::Y => (p[0].clamp(0.0, 1.0), p[2].clamp(0.0, 1.0)),
        Axis::Z => (p[0].clamp(0.0, 1.0), p[1].clamp(0.0, 1.0)),
    }
}

/// Snap: face center → face normal; mid-edge → 90° onto the free axis.
pub fn snap_axis_on_face(p: [f32; 3], face: Axis) -> Option<Axis> {
    let (u, v) = face_uv(p, face);
    let du = (u - 0.5).abs();
    let dv = (v - 0.5).abs();
    let center = (du * du + dv * dv).sqrt();
    if center <= FACE_CENTER_SNAP {
        return Some(face);
    }
    let (ua, va) = face.plane_uv();
    // Mid of an edge parallel to U (v ≈ 0 or 1, u ≈ 0.5) → snap to V.
    let edge_v = (dv - 0.5).abs() <= FACE_EDGE_SNAP && du <= FACE_EDGE_SNAP + 0.06;
    if edge_v {
        return Some(va);
    }
    // Mid of an edge parallel to V (u ≈ 0 or 1, v ≈ 0.5) → snap to U.
    let edge_u = (du - 0.5).abs() <= FACE_EDGE_SNAP && dv <= FACE_EDGE_SNAP + 0.06;
    if edge_u {
        return Some(ua);
    }
    None
}

fn snap_axis_from_segment(ro: [f32; 3], rd: [f32; 3], seg: (f32, f32)) -> Option<Axis> {
    let mut best: Option<(f32, Axis)> = None;
    for t in [seg.0, seg.1] {
        if t < -1e-4 {
            continue;
        }
        let p = point_on(ro, rd, t);
        let Some((face, _)) = face_from_point(p) else {
            continue;
        };
        if let Some(axis) = snap_axis_on_face(p, face) {
            let (u, v) = face_uv(p, face);
            let score = (u - 0.5).abs() + (v - 0.5).abs();
            if best.map(|(s, _)| score < s).unwrap_or(true) {
                best = Some((score, axis));
            }
        }
    }
    best.map(|(_, a)| a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_cutter_starts_idle_one_layer_on_z() {
        let c = Cutter::new(64);
        assert!(!c.active);
        assert_eq!(c.mode, CaptureMode::Squash2D);
        assert_eq!(c.axis, Axis::Z);
        assert!((c.half_vox - 0.5).abs() < 1e-5);
        assert!((c.thickness_voxels() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn enter_follows_view_slice_depth() {
        let mut c = Cutter::new(96);
        c.enter(48.0, 96);
        assert!(c.active);
        assert!((c.pos - 0.5).abs() < 1e-5);
        assert_eq!(c.axis, Axis::Z);
    }

    #[test]
    fn toggle_and_esc_leave() {
        let mut c = Cutter::new(64);
        c.toggle(20.0, 64);
        assert!(c.active);
        c.toggle(20.0, 64);
        assert!(!c.active);
    }

    #[test]
    fn mode_and_axis_cycle() {
        let mut c = Cutter::new(32);
        c.cycle_mode();
        assert_eq!(c.mode, CaptureMode::RichBox);
        c.cycle_mode();
        assert_eq!(c.mode, CaptureMode::Squash2D);
        c.cycle_axis();
        assert_eq!(c.axis, Axis::X);
        c.cycle_axis();
        assert_eq!(c.axis, Axis::Y);
    }

    #[test]
    fn ray_hits_unit_cube() {
        let seg = ray_cube_segment([-1.0, 0.5, 0.5], [1.0, 0.0, 0.0]).unwrap();
        assert!((seg.0 - 1.0).abs() < 1e-4);
        assert!((seg.1 - 2.0).abs() < 1e-4);
        assert!(ray_cube_segment([-2.0, 2.0, 0.5], [1.0, 0.0, 0.0]).is_none());
        let mid = ray_cube_midpoint([-1.0, 0.5, 0.5], [1.0, 0.0, 0.0]).unwrap();
        assert!((mid[0] - 0.5).abs() < 1e-4);
        assert!((mid[1] - 0.5).abs() < 1e-4);
    }

    #[test]
    fn face_center_snaps_to_face_normal() {
        assert_eq!(snap_axis_on_face([1.0, 0.5, 0.5], Axis::X), Some(Axis::X));
        assert_eq!(snap_axis_on_face([0.5, 0.5, 1.0], Axis::Z), Some(Axis::Z));
        assert_eq!(snap_axis_on_face([0.5, 0.0, 0.5], Axis::Y), Some(Axis::Y));
    }

    #[test]
    fn face_mid_edge_snaps_ninety_degrees() {
        // Top face (+Z): mid of X-aligned edge → Y (vertical).
        assert_eq!(snap_axis_on_face([0.5, 0.0, 1.0], Axis::Z), Some(Axis::Y));
        // Top face: mid of Y-aligned edge → X (vertical).
        assert_eq!(snap_axis_on_face([0.0, 0.5, 1.0], Axis::Z), Some(Axis::X));
        // +X face: mid of horizontal edge (z=0.5, y=0) → Y.
        assert_eq!(snap_axis_on_face([1.0, 0.0, 0.5], Axis::X), Some(Axis::Y));
        // +X face: mid of vertical edge (y=0.5, z=0) → Z (horizontal).
        assert_eq!(snap_axis_on_face([1.0, 0.5, 0.0], Axis::X), Some(Axis::Z));
    }

    #[test]
    fn face_corner_does_not_snap() {
        assert_eq!(snap_axis_on_face([0.02, 0.02, 1.0], Axis::Z), None);
    }

    #[test]
    fn hover_plus_x_face_center_snaps_vertical() {
        let mut c = Cutter::new(64);
        c.enter(32.0, 64);
        c.track_pointer([-1.0, 0.5, 0.5], [1.0, 0.0, 0.0], 64.0);
        assert_eq!(c.axis, Axis::X);
        assert!(c.snapped);
        assert!((c.pos - 0.5).abs() < 0.05);
    }

    #[test]
    fn hover_top_face_center_snaps_horizontal() {
        let mut c = Cutter::new(64);
        c.enter(10.0, 64);
        c.track_pointer([0.5, 0.5, 2.0], [0.0, 0.0, -1.0], 64.0);
        assert_eq!(c.axis, Axis::Z);
        assert!(c.snapped);
    }

    #[test]
    fn slider_handles_thicken_from_one_layer() {
        let mut c = Cutter::new(100);
        c.enter(50.0, 100);
        // One-layer handles sit on the cursor; grabbing either starts the thicken.
        let hn = c.half_norm(100.0);
        let x_neg = SLIDER_X0 + (c.pos - hn) * (SLIDER_X1 - SLIDER_X0);
        assert_eq!(c.hit_handle((x_neg, 0.93), 100.0), Some(Handle::Neg));
        c.dragging = Some(Handle::Pos);
        let x_wide = SLIDER_X0 + (c.pos + 0.10) * (SLIDER_X1 - SLIDER_X0);
        c.drag_handle((x_wide, 0.93), 100.0);
        assert!((c.half_vox - 10.0).abs() < 0.6);
        assert!(c.thickness_voxels() > 8.0);
        let x_far = SLIDER_X0 + 0.02 * (SLIDER_X1 - SLIDER_X0);
        assert_eq!(c.hit_handle((x_far, 0.93), 100.0), None);
    }

    #[test]
    fn present_vec_idle_is_mode_zero() {
        let c = Cutter::new(32);
        assert_eq!(c.present_vec()[3], 0.0);
        let mut c = c;
        c.enter(8.0, 32);
        assert!((c.present_vec()[3] - 1.0).abs() < 1e-5);
        c.snapped = true;
        assert!((c.present_vec()[3] - 1.5).abs() < 1e-5);
        c.mode = CaptureMode::RichBox;
        assert!((c.present_vec()[3] - 2.5).abs() < 1e-5);
    }

    #[test]
    fn thickness_never_drops_below_one_layer() {
        let mut c = Cutter::new(40);
        c.nudge_half(-8.0, 40.0);
        assert!((c.half_vox - 0.5).abs() < 1e-5);
    }

    #[test]
    fn heightmap_flag_turns_on_when_axis_cycled() {
        let mut c = Cutter::new(16);
        assert!(!c.export_heightmap);
        c.cycle_heightmap_axis();
        assert!(c.export_heightmap);
        assert_eq!(c.heightmap_axis, Axis::X);
    }
}
