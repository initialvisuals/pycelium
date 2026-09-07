//! Species presets, paint channels, and the top-strip tool chrome.
//!
//! Selecting a preset writes the eight global PARAM knobs and arms the
//! specimen brush with that lineage id. Per-tip genetics / antagonism are
//! a later hook (`Tip.flags == 2` marks a painted inoculum).

use crate::types::SimUniforms;

pub const SPECIES_COUNT: usize = 8;

/// Top-center strip: left of the slab inset, above the left HUD.
pub const STRIP_X0: f32 = 0.268;
pub const STRIP_Y0: f32 = 0.010;
pub const STRIP_Y1: f32 = 0.078;
pub const CELL: f32 = 0.042;
pub const MODE_X0: f32 = STRIP_X0 + CELL * SPECIES_COUNT as f32 + 0.010;
pub const MODE_W: f32 = 0.046;
pub const STRIP_X1: f32 = MODE_X0 + MODE_W * 2.0 + 0.004;

pub const RADIUS_MIN: f32 = 1.5;
pub const RADIUS_MAX: f32 = 24.0;
pub const RADIUS_DEFAULT: f32 = 5.0;
pub const STRENGTH_DEFAULT: f32 = 0.55;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolMode {
    View,
    Specimen,
    Paint,
}

impl ToolMode {
    pub fn cycle(self) -> Self {
        match self {
            Self::View => Self::Specimen,
            Self::Specimen => Self::Paint,
            Self::Paint => Self::View,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::View => "VIEW",
            Self::Specimen => "SPECIMEN",
            Self::Paint => "PAINT",
        }
    }

    pub fn as_f32(self) -> f32 {
        match self {
            Self::View => 0.0,
            Self::Specimen => 1.0,
            Self::Paint => 2.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintChannel {
    SolC = 0,
    SolN = 1,
    Moisture = 2,
    Organic = 3,
    Enzyme = 4,
    Wood = 5,
    Litter = 6,
    Void = 7,
}

impl PaintChannel {
    pub fn from_index(i: u8) -> Self {
        match i % 8 {
            0 => Self::SolC,
            1 => Self::SolN,
            2 => Self::Moisture,
            3 => Self::Organic,
            4 => Self::Enzyme,
            5 => Self::Wood,
            6 => Self::Litter,
            _ => Self::Void,
        }
    }

    pub fn index(self) -> u8 {
        self as u8
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::SolC => "SOL C",
            Self::SolN => "SOL N",
            Self::Moisture => "H2O",
            Self::Organic => "ORG",
            Self::Enzyme => "ENZ",
            Self::Wood => "WOOD",
            Self::Litter => "LITTER",
            Self::Void => "VOID",
        }
    }

}

/// One starter species. `params` are absolute knob values for slots 0..7.
/// `id` / `color` stay on the table so present.wgsl and PAINT.md stay in lockstep.
#[derive(Clone, Copy, Debug)]
pub struct SpeciesPreset {
    #[allow(dead_code)]
    pub id: u8,
    pub lineage: u32,
    pub name: &'static str,
    pub short: &'static str,
    #[allow(dead_code)]
    pub color: [f32; 3],
    pub params: [f32; 8],
    pub teach: &'static str,
}

pub const SPECIES: [SpeciesPreset; SPECIES_COUNT] = [
    SpeciesPreset {
        id: 0,
        lineage: 1,
        name: "PIONEER",
        short: "PION",
        color: [0.55, 0.92, 0.82],
        params: [2.40, 0.50, 1.40, 0.60, 0.0008, 0.028, 0.28, 1.60],
        teach: "Pioneer: first arriver. High chemotropism and extension, cheap forks, low persistence. Hunts carbon plumes and fans into empty soil. Lineage 1.",
    },
    SpeciesPreset {
        id: 1,
        lineage: 2,
        name: "CORD",
        short: "CORD",
        color: [0.95, 0.72, 0.28],
        params: [1.20, 0.60, 0.35, 2.60, 0.0010, 0.024, 1.40, 1.80],
        teach: "Cord-former: long committed runs. High persistence and extension, expensive branches, low autotropism so it will recross its own trail. Lineage 2.",
    },
    SpeciesPreset {
        id: 2,
        lineage: 3,
        name: "SCAVENGER",
        short: "SCAV",
        color: [0.78, 0.42, 0.10],
        params: [1.80, 0.90, 0.80, 1.10, 0.0014, 0.095, 0.50, 0.90],
        teach: "Scavenger: litter miner. High enzyme_k and chemotropism so uncleaved polymer unlocks quickly. Medium steps. Lineage 3.",
    },
    SpeciesPreset {
        id: 3,
        lineage: 4,
        name: "NITROPHILE",
        short: "NITR",
        color: [0.55, 0.40, 0.85],
        params: [0.40, 2.60, 1.00, 1.20, 0.0011, 0.030, 0.45, 1.10],
        teach: "Nitrophile: nitrogen hunter. High nitrotropism, low chemotropism. Useful in an N-poor pedon; may ignore rust C plumes. Lineage 4.",
    },
    SpeciesPreset {
        id: 4,
        lineage: 5,
        name: "MAT",
        short: "MAT",
        color: [0.32, 0.70, 0.34],
        params: [1.00, 0.70, 0.15, 0.50, 0.0025, 0.040, 0.18, 0.70],
        teach: "Mat-former: dense local fill. Low autotropism and persistence, cheap branches, short steps. Piles onto itself. Higher maintenance. Lineage 5.",
    },
    SpeciesPreset {
        id: 5,
        lineage: 6,
        name: "THRIFTY",
        short: "THRF",
        color: [0.90, 0.78, 0.40],
        params: [0.80, 0.80, 1.20, 2.00, 0.0003, 0.018, 1.20, 0.60],
        teach: "Thrifty: lean-soil survivor. Very low maintenance, high persistence, expensive forks, short cautious steps. Banks reserve. Lineage 6.",
    },
    SpeciesPreset {
        id: 6,
        lineage: 7,
        name: "RANGER",
        short: "RANG",
        color: [0.95, 0.38, 0.42],
        params: [2.20, 0.40, 1.80, 2.20, 0.0010, 0.022, 0.90, 2.00],
        teach: "Ranger: long hunting arcs. High chemotropism, persistence, autotropism, and extension. Smooth committed runs that still seek C and avoid crowding. Lineage 7.",
    },
    SpeciesPreset {
        id: 7,
        lineage: 8,
        name: "MINER",
        short: "MINE",
        color: [0.78, 0.84, 0.88],
        params: [1.30, 1.60, 2.20, 1.00, 0.0120, 0.100, 0.70, 1.00],
        teach: "Miner: costly aggressive metabolizer. High enzyme_k, autotropism, and maintenance. Placeholder flavor for later antagonism — no chemical warfare yet. Lineage 8.",
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StripHit {
    Species(u8),
    SpecimenMode,
    PaintMode,
}

#[derive(Clone, Debug)]
pub struct BrushState {
    pub mode: ToolMode,
    pub species: u8,
    pub channel: PaintChannel,
    pub radius: f32,
    pub strength: f32,
    pub erase: bool,
    pub stroking: bool,
    pub last_stamp: Option<[f32; 3]>,
}

impl Default for BrushState {
    fn default() -> Self {
        Self {
            mode: ToolMode::View,
            species: 0,
            channel: PaintChannel::SolC,
            radius: RADIUS_DEFAULT,
            strength: STRENGTH_DEFAULT,
            erase: false,
            stroking: false,
            last_stamp: None,
        }
    }
}

impl BrushState {
    pub fn species_preset(&self) -> &'static SpeciesPreset {
        &SPECIES[self.species as usize % SPECIES_COUNT]
    }

    pub fn apply_species(&self, uniforms: &mut SimUniforms) {
        let p = self.species_preset();
        for slot in 0..8u32 {
            uniforms.set_param(slot, p.params[slot as usize]);
        }
    }

    pub fn cycle_mode(&mut self) {
        self.mode = self.mode.cycle();
        self.stroking = false;
        self.last_stamp = None;
    }

    pub fn select_species(&mut self, id: u8, uniforms: &mut SimUniforms) {
        self.species = id % SPECIES_COUNT as u8;
        self.mode = ToolMode::Specimen;
        self.apply_species(uniforms);
        self.stroking = false;
        self.last_stamp = None;
    }

    pub fn select_channel(&mut self, ch: PaintChannel) {
        self.channel = ch;
        self.mode = ToolMode::Paint;
        self.stroking = false;
        self.last_stamp = None;
    }

    pub fn nudge_radius(&mut self, steps: f32) {
        self.radius = (self.radius + steps).clamp(RADIUS_MIN, RADIUS_MAX);
    }

    pub fn stamp_spacing(&self) -> f32 {
        (self.radius * 0.40).max(1.2)
    }

    pub fn should_stamp(&self, pos: [f32; 3]) -> bool {
        match self.last_stamp {
            None => true,
            Some(prev) => {
                let dx = pos[0] - prev[0];
                let dy = pos[1] - prev[1];
                let dz = pos[2] - prev[2];
                (dx * dx + dy * dy + dz * dz).sqrt() >= self.stamp_spacing()
            }
        }
    }

    pub fn present_vec(&self) -> [f32; 4] {
        [
            self.mode.as_f32(),
            self.species as f32,
            self.channel.index() as f32,
            self.radius,
        ]
    }

    pub fn toast_title(&self) -> String {
        match self.mode {
            ToolMode::View => "VIEW".into(),
            ToolMode::Specimen => format!("SPEC {}", self.species_preset().short),
            ToolMode::Paint => format!("PAINT {}", self.channel.name()),
        }
    }

    pub fn toast_value(&self) -> String {
        let erase = if self.erase { "  ERASE" } else { "" };
        match self.mode {
            ToolMode::View => "ORBIT  CLICK PICK".into(),
            ToolMode::Specimen => format!(
                "LIN {}  R {:.1}{}",
                self.species_preset().lineage,
                self.radius,
                erase
            ),
            ToolMode::Paint => format!("R {:.1}  STR {:.2}{}", self.radius, self.strength, erase),
        }
    }
}

pub fn species_cell(i: u8) -> [f32; 4] {
    let i = i.min(7);
    let x0 = STRIP_X0 + CELL * i as f32;
    [x0, STRIP_Y0, x0 + CELL, STRIP_Y1]
}

pub fn specimen_mode_rect() -> [f32; 4] {
    [MODE_X0, STRIP_Y0, MODE_X0 + MODE_W, STRIP_Y1]
}

pub fn paint_mode_rect() -> [f32; 4] {
    [
        MODE_X0 + MODE_W + 0.004,
        STRIP_Y0,
        MODE_X0 + MODE_W * 2.0 + 0.004,
        STRIP_Y1,
    ]
}

pub fn strip_hit(uv: (f32, f32)) -> Option<StripHit> {
    if uv.1 < STRIP_Y0 || uv.1 > STRIP_Y1 || uv.0 < STRIP_X0 || uv.0 > STRIP_X1 {
        return None;
    }
    if in_rect(uv, specimen_mode_rect()) {
        return Some(StripHit::SpecimenMode);
    }
    if in_rect(uv, paint_mode_rect()) {
        return Some(StripHit::PaintMode);
    }
    if uv.0 < STRIP_X0 + CELL * SPECIES_COUNT as f32 {
        let i = ((uv.0 - STRIP_X0) / CELL).floor() as i32;
        if (0..8).contains(&i) {
            return Some(StripHit::Species(i as u8));
        }
    }
    None
}

fn in_rect(uv: (f32, f32), r: [f32; 4]) -> bool {
    uv.0 >= r[0] && uv.0 <= r[2] && uv.1 >= r[1] && uv.1 <= r[3]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eight_species_have_unique_lineage_and_in_range_params() {
        let mut seen = [false; 9];
        for (i, s) in SPECIES.iter().enumerate() {
            assert_eq!(s.id as usize, i);
            assert_eq!(s.lineage, i as u32 + 1);
            assert!(!seen[s.lineage as usize]);
            seen[s.lineage as usize] = true;
            assert!(s.short.len() <= 4);
            assert!(s.name.len() <= 10);
            for slot in 0..8u32 {
                let (lo, hi) = SimUniforms::param_range(slot);
                let v = s.params[slot as usize];
                assert!(
                    v >= lo - 1e-5 && v <= hi + 1e-5,
                    "{} slot {slot} {v} not in {lo}..{hi}",
                    s.name
                );
            }
        }
    }

    #[test]
    fn apply_species_writes_all_eight_knobs() {
        let mut u = SimUniforms::new(32, 32, 16, 64);
        let mut brush = BrushState::default();
        brush.select_species(2, &mut u);
        assert_eq!(brush.mode, ToolMode::Specimen);
        assert_eq!(brush.species, 2);
        for slot in 0..8u32 {
            assert!((u.param_value(slot) - SPECIES[2].params[slot as usize]).abs() < 1e-5);
        }
    }

    #[test]
    fn strip_hits_species_and_mode_chips() {
        let pion = species_cell(0);
        let mid = (0.5 * (pion[0] + pion[2]), 0.5 * (pion[1] + pion[3]));
        assert_eq!(strip_hit(mid), Some(StripHit::Species(0)));
        let last = species_cell(7);
        let m7 = (0.5 * (last[0] + last[2]), 0.5 * (last[1] + last[3]));
        assert_eq!(strip_hit(m7), Some(StripHit::Species(7)));
        let spec = specimen_mode_rect();
        assert_eq!(
            strip_hit((0.5 * (spec[0] + spec[2]), 0.044)),
            Some(StripHit::SpecimenMode)
        );
        let paint = paint_mode_rect();
        assert_eq!(
            strip_hit((0.5 * (paint[0] + paint[2]), 0.044)),
            Some(StripHit::PaintMode)
        );
        assert!(strip_hit((0.04, 0.04)).is_none());
        assert!(strip_hit((0.50, 0.50)).is_none());
        assert!(STRIP_X1 < 0.72);
        assert!(STRIP_Y1 < 0.08);
    }

    #[test]
    fn stamp_spacing_skips_tight_samples() {
        let mut b = BrushState::default();
        b.radius = 5.0;
        assert!(b.should_stamp([10.0, 10.0, 10.0]));
        b.last_stamp = Some([10.0, 10.0, 10.0]);
        assert!(!b.should_stamp([10.2, 10.0, 10.0]));
        assert!(b.should_stamp([14.0, 10.0, 10.0]));
    }

    #[test]
    fn tool_and_channel_cycle() {
        assert_eq!(ToolMode::View.cycle(), ToolMode::Specimen);
        assert_eq!(ToolMode::Specimen.cycle(), ToolMode::Paint);
        assert_eq!(ToolMode::Paint.cycle(), ToolMode::View);
        assert_eq!(PaintChannel::from_index(7), PaintChannel::Void);
        assert_eq!(PaintChannel::Wood.index(), 5);
        let mut b = BrushState::default();
        b.nudge_radius(100.0);
        assert!((b.radius - RADIUS_MAX).abs() < 1e-5);
        b.nudge_radius(-100.0);
        assert!((b.radius - RADIUS_MIN).abs() < 1e-5);
    }

    #[test]
    fn colors_are_distinct_enough() {
        for i in 0..SPECIES_COUNT {
            for j in (i + 1)..SPECIES_COUNT {
                let a = SPECIES[i].color;
                let b = SPECIES[j].color;
                let d = (a[0] - b[0]).abs() + (a[1] - b[1]).abs() + (a[2] - b[2]).abs();
                assert!(d > 0.18, "{} vs {} too close ({d})", SPECIES[i].name, SPECIES[j].name);
            }
        }
    }
}
