//! Export settings window: square power-of-two presets, pad vs crop, native aspect.
//!
//! Drawn as an in-engine card on the right (below the slab inset) while capturing.
//! Geometry here must match `shaders/present.wgsl`.

/// Square power-of-two export sizes, 8 through 8K.
pub const SQUARE_PRESETS: [u32; 11] = [
    8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192,
];

/// Mid-list default: 512², not the native slab aspect.
pub const DEFAULT_PRESET_INDEX: usize = 6;

/// Right-column card. Sits under the inset (`x=0.72..0.98`, `y=0.06..0.34`)
/// and above the thickness slider (`y=0.90`).
pub const PANEL_X0: f32 = 0.735;
pub const PANEL_X1: f32 = 0.985;
pub const CHIP_Y0: f32 = 0.368;
pub const CHIP_Y1: f32 = 0.418;
pub const PANEL_Y1: f32 = 0.882;
pub const PRESET_Y0: f32 = 0.448;
pub const PRESET_ROW: f32 = 0.028;
pub const FIT_Y0: f32 = 0.768;
pub const FIT_Y1: f32 = 0.808;
pub const ASPECT_Y0: f32 = 0.816;
pub const ASPECT_Y1: f32 = 0.856;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportAspect {
    /// Power-of-two square (default).
    Square,
    /// Keep the extracted slab aspect (advanced / old behavior).
    Native,
}

impl ExportAspect {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Square => "square",
            Self::Native => "native",
        }
    }

    pub fn cycle(self) -> Self {
        match self {
            Self::Square => Self::Native,
            Self::Native => Self::Square,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FitMode {
    /// Letterbox / pillarbox the full plane into the square.
    Pad,
    /// Crop to the occupied density AABB, then letterbox that into the square.
    CropAabb,
}

impl FitMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pad => "pad",
            Self::CropAabb => "crop",
        }
    }

    pub fn cycle(self) -> Self {
        match self {
            Self::Pad => Self::CropAabb,
            Self::CropAabb => Self::Pad,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportHit {
    TogglePanel,
    Preset(usize),
    Fit(FitMode),
    Aspect(ExportAspect),
}

#[derive(Clone, Debug)]
pub struct ExportSettings {
    pub panel_open: bool,
    pub aspect: ExportAspect,
    pub preset_index: usize,
    pub fit: FitMode,
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            panel_open: false,
            aspect: ExportAspect::Square,
            preset_index: DEFAULT_PRESET_INDEX,
            fit: FitMode::Pad,
        }
    }
}

impl ExportSettings {
    pub fn square_size(&self) -> u32 {
        SQUARE_PRESETS[self.preset_index.min(SQUARE_PRESETS.len() - 1)]
    }

    pub fn cycle_preset(&mut self, dir: i32) {
        let n = SQUARE_PRESETS.len() as i32;
        let next = (self.preset_index as i32 + dir).rem_euclid(n);
        self.preset_index = next as usize;
    }

    pub fn set_preset(&mut self, index: usize) {
        if index < SQUARE_PRESETS.len() {
            self.preset_index = index;
        }
    }

    pub fn toggle_panel(&mut self) {
        self.panel_open = !self.panel_open;
    }

    pub fn close_panel(&mut self) {
        self.panel_open = false;
    }

    pub fn describe(&self) -> String {
        if self.aspect == ExportAspect::Native {
            format!(
                "native {}  {}",
                self.fit.as_str(),
                self.square_size()
            )
        } else {
            format!(
                "{}x{} {}  {}",
                self.square_size(),
                self.square_size(),
                self.aspect.as_str(),
                self.fit.as_str()
            )
        }
    }

    /// Packed present uniform: x = open, y = preset index, z = aspect, w = fit.
    pub fn present_vec(&self) -> [f32; 4] {
        [
            if self.panel_open { 1.0 } else { 0.0 },
            self.preset_index as f32,
            if self.aspect == ExportAspect::Native {
                1.0
            } else {
                0.0
            },
            if self.fit == FitMode::CropAabb {
                1.0
            } else {
                0.0
            },
        ]
    }

    pub fn contains_cursor(&self, uv: (f32, f32)) -> bool {
        if uv.0 < PANEL_X0 || uv.0 > PANEL_X1 {
            return false;
        }
        if self.panel_open {
            uv.1 >= CHIP_Y0 && uv.1 <= PANEL_Y1
        } else {
            uv.1 >= CHIP_Y0 && uv.1 <= CHIP_Y1
        }
    }

    pub fn hit(&self, uv: (f32, f32)) -> Option<ExportHit> {
        if uv.0 < PANEL_X0 || uv.0 > PANEL_X1 {
            return None;
        }
        if uv.1 >= CHIP_Y0 && uv.1 <= CHIP_Y1 {
            return Some(ExportHit::TogglePanel);
        }
        if !self.panel_open {
            return None;
        }
        if uv.1 >= PRESET_Y0 && uv.1 < PRESET_Y0 + PRESET_ROW * SQUARE_PRESETS.len() as f32 {
            let i = ((uv.1 - PRESET_Y0) / PRESET_ROW).floor() as i32;
            if (0..SQUARE_PRESETS.len() as i32).contains(&i) {
                return Some(ExportHit::Preset(i as usize));
            }
        }
        let mid = 0.5 * (PANEL_X0 + PANEL_X1);
        if uv.1 >= FIT_Y0 && uv.1 <= FIT_Y1 {
            return Some(if uv.0 < mid {
                ExportHit::Fit(FitMode::Pad)
            } else {
                ExportHit::Fit(FitMode::CropAabb)
            });
        }
        if uv.1 >= ASPECT_Y0 && uv.1 <= ASPECT_Y1 {
            return Some(if uv.0 < mid {
                ExportHit::Aspect(ExportAspect::Square)
            } else {
                ExportHit::Aspect(ExportAspect::Native)
            });
        }
        None
    }

    pub fn apply_hit(&mut self, hit: ExportHit) {
        match hit {
            ExportHit::TogglePanel => self.toggle_panel(),
            ExportHit::Preset(i) => self.set_preset(i),
            ExportHit::Fit(fit) => self.fit = fit,
            ExportHit::Aspect(aspect) => self.aspect = aspect,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_512_square_pad() {
        let s = ExportSettings::default();
        assert_eq!(s.square_size(), 512);
        assert_eq!(s.aspect, ExportAspect::Square);
        assert_eq!(s.fit, FitMode::Pad);
        assert!(!s.panel_open);
        assert_eq!(SQUARE_PRESETS[s.preset_index], 512);
    }

    #[test]
    fn presets_are_power_of_two_through_8k() {
        assert_eq!(SQUARE_PRESETS[0], 8);
        assert_eq!(SQUARE_PRESETS[10], 8192);
        for (i, &n) in SQUARE_PRESETS.iter().enumerate() {
            assert_eq!(n, 8u32 << i, "preset {i} should be 8<<{i}");
        }
    }

    #[test]
    fn cycle_preset_wraps() {
        let mut s = ExportSettings::default();
        s.cycle_preset(-1);
        assert_eq!(s.square_size(), 256);
        s.preset_index = 0;
        s.cycle_preset(-1);
        assert_eq!(s.square_size(), 8192);
        s.cycle_preset(1);
        assert_eq!(s.square_size(), 8);
    }

    #[test]
    fn closed_chip_toggles_panel() {
        let s = ExportSettings::default();
        assert_eq!(
            s.hit((0.86, 0.39)),
            Some(ExportHit::TogglePanel)
        );
        assert!(s.hit((0.86, 0.50)).is_none());
        assert!(s.contains_cursor((0.86, 0.39)));
        assert!(!s.contains_cursor((0.86, 0.50)));
        assert!(!s.contains_cursor((0.40, 0.39)));
    }

    #[test]
    fn open_panel_hits_presets_fit_and_aspect() {
        let mut s = ExportSettings::default();
        s.panel_open = true;
        assert_eq!(s.hit((0.86, PRESET_Y0 + 0.01)), Some(ExportHit::Preset(0)));
        let y512 = PRESET_Y0 + PRESET_ROW * 6.0 + 0.01;
        assert_eq!(s.hit((0.86, y512)), Some(ExportHit::Preset(6)));
        let y8k = PRESET_Y0 + PRESET_ROW * 10.0 + 0.01;
        assert_eq!(s.hit((0.86, y8k)), Some(ExportHit::Preset(10)));
        assert_eq!(s.hit((0.76, 0.788)), Some(ExportHit::Fit(FitMode::Pad)));
        assert_eq!(
            s.hit((0.92, 0.788)),
            Some(ExportHit::Fit(FitMode::CropAabb))
        );
        assert_eq!(
            s.hit((0.76, 0.836)),
            Some(ExportHit::Aspect(ExportAspect::Square))
        );
        assert_eq!(
            s.hit((0.92, 0.836)),
            Some(ExportHit::Aspect(ExportAspect::Native))
        );
        assert!(s.contains_cursor((0.86, 0.70)));
    }

    #[test]
    fn apply_hit_updates_state() {
        let mut s = ExportSettings::default();
        s.apply_hit(ExportHit::Preset(10));
        assert_eq!(s.square_size(), 8192);
        s.apply_hit(ExportHit::Fit(FitMode::CropAabb));
        assert_eq!(s.fit, FitMode::CropAabb);
        s.apply_hit(ExportHit::Aspect(ExportAspect::Native));
        assert_eq!(s.aspect, ExportAspect::Native);
        s.apply_hit(ExportHit::TogglePanel);
        assert!(s.panel_open);
    }

    #[test]
    fn present_vec_packs_flags() {
        let mut s = ExportSettings::default();
        assert_eq!(s.present_vec(), [0.0, 6.0, 0.0, 0.0]);
        s.panel_open = true;
        s.aspect = ExportAspect::Native;
        s.fit = FitMode::CropAabb;
        s.preset_index = 3;
        assert_eq!(s.present_vec(), [1.0, 3.0, 1.0, 1.0]);
    }
}
