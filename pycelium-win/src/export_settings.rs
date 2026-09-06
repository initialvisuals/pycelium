//! Export settings window: square power-of-two presets, pad vs crop, native aspect,
//! and optional format toggles.
//!
//! Drawn as an in-engine card on the right (below the slab inset) while capturing.
//! Geometry here must match `shaders/present.wgsl`.
//!
//! The previous 11-row preset stack plus FIT / ASPECT / format labels overflowed:
//! FIT sat on the 8K row, and SVG painted past the card / window. Presets are now
//! two columns so the full card stays above the thickness slider (`y=0.90`).

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
pub const CHIP_Y1: f32 = 0.412;
pub const PANEL_Y1: f32 = 0.848;
pub const PRESET_Y0: f32 = 0.440;
pub const PRESET_ROW: f32 = 0.032;
pub const PRESET_COLS: usize = 2;
pub const PRESET_ROWS: usize = 6;
pub const FIT_Y0: f32 = 0.644;
pub const FIT_Y1: f32 = 0.684;
pub const ASPECT_Y0: f32 = 0.692;
pub const ASPECT_Y1: f32 = 0.732;
pub const FORMAT_Y0: f32 = 0.748;
pub const FORMAT_Y1: f32 = 0.788;
pub const HT_Y0: f32 = 0.796;
pub const HT_Y1: f32 = 0.836;

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
pub enum ExportFormat {
    Png,
    Mask,
    Json,
    Svg,
    Heightmap,
}

impl ExportFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Mask => "mask",
            Self::Json => "json",
            Self::Svg => "svg",
            Self::Heightmap => "heightmap",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportHit {
    TogglePanel,
    Preset(usize),
    Fit(FitMode),
    Aspect(ExportAspect),
    Format(ExportFormat),
}

#[derive(Clone, Debug)]
pub struct ExportSettings {
    pub panel_open: bool,
    pub aspect: ExportAspect,
    pub preset_index: usize,
    pub fit: FitMode,
    pub write_png: bool,
    pub write_mask: bool,
    pub write_json: bool,
    pub write_svg: bool,
    pub write_heightmap: bool,
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            panel_open: false,
            aspect: ExportAspect::Square,
            preset_index: DEFAULT_PRESET_INDEX,
            fit: FitMode::Pad,
            write_png: true,
            write_mask: true,
            write_json: true,
            write_svg: true,
            write_heightmap: false,
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

    pub fn toggle_format(&mut self, format: ExportFormat) {
        match format {
            ExportFormat::Png => self.write_png = !self.write_png,
            ExportFormat::Mask => self.write_mask = !self.write_mask,
            ExportFormat::Json => self.write_json = !self.write_json,
            ExportFormat::Svg => self.write_svg = !self.write_svg,
            ExportFormat::Heightmap => self.write_heightmap = !self.write_heightmap,
        }
        // Enter should still write something if the user turns every file off.
        if !self.write_png
            && !self.write_mask
            && !self.write_json
            && !self.write_svg
            && !self.write_heightmap
        {
            self.write_png = true;
        }
    }

    pub fn format_on(&self, format: ExportFormat) -> bool {
        match format {
            ExportFormat::Png => self.write_png,
            ExportFormat::Mask => self.write_mask,
            ExportFormat::Json => self.write_json,
            ExportFormat::Svg => self.write_svg,
            ExportFormat::Heightmap => self.write_heightmap,
        }
    }

    pub fn describe(&self) -> String {
        let mut formats = String::new();
        for (on, name) in [
            (self.write_png, "png"),
            (self.write_mask, "mask"),
            (self.write_json, "json"),
            (self.write_svg, "svg"),
            (self.write_heightmap, "height"),
        ] {
            if on {
                if !formats.is_empty() {
                    formats.push(' ');
                }
                formats.push_str(name);
            }
        }
        if self.aspect == ExportAspect::Native {
            format!(
                "native {}  {}  {}",
                self.fit.as_str(),
                self.square_size(),
                formats
            )
        } else {
            format!(
                "{}x{} {}  {}  {}",
                self.square_size(),
                self.square_size(),
                self.aspect.as_str(),
                self.fit.as_str(),
                formats
            )
        }
    }

    /// Packed present uniform: x = open, y = preset index,
    /// z = flags (bit0 native, bit1 crop, bit2 png, bit3 mask, bit4 json,
    /// bit5 svg, bit6 heightmap).
    pub fn present_vec(&self) -> [f32; 4] {
        let mut flags = 0u32;
        if self.aspect == ExportAspect::Native {
            flags |= 1;
        }
        if self.fit == FitMode::CropAabb {
            flags |= 2;
        }
        if self.write_png {
            flags |= 4;
        }
        if self.write_mask {
            flags |= 8;
        }
        if self.write_json {
            flags |= 16;
        }
        if self.write_svg {
            flags |= 32;
        }
        if self.write_heightmap {
            flags |= 64;
        }
        [
            if self.panel_open { 1.0 } else { 0.0 },
            self.preset_index as f32,
            flags as f32,
            0.0,
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

    fn preset_index_at(uv: (f32, f32)) -> Option<usize> {
        let rows = PRESET_ROWS as f32;
        if uv.1 < PRESET_Y0 || uv.1 >= PRESET_Y0 + PRESET_ROW * rows {
            return None;
        }
        let row = ((uv.1 - PRESET_Y0) / PRESET_ROW).floor() as i32;
        if !(0..PRESET_ROWS as i32).contains(&row) {
            return None;
        }
        let mid = 0.5 * (PANEL_X0 + PANEL_X1);
        let col = if uv.0 < mid { 0 } else { 1 };
        let i = row as usize * PRESET_COLS + col;
        if i < SQUARE_PRESETS.len() {
            Some(i)
        } else {
            None
        }
    }

    fn format_at(uv: (f32, f32)) -> Option<ExportFormat> {
        if uv.1 >= FORMAT_Y0 && uv.1 <= FORMAT_Y1 {
            let t = ((uv.0 - PANEL_X0) / (PANEL_X1 - PANEL_X0)).clamp(0.0, 0.999);
            return Some(match (t * 4.0).floor() as i32 {
                0 => ExportFormat::Png,
                1 => ExportFormat::Mask,
                2 => ExportFormat::Json,
                _ => ExportFormat::Svg,
            });
        }
        if uv.1 >= HT_Y0 && uv.1 <= HT_Y1 {
            return Some(ExportFormat::Heightmap);
        }
        None
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
        if let Some(i) = Self::preset_index_at(uv) {
            return Some(ExportHit::Preset(i));
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
        if let Some(fmt) = Self::format_at(uv) {
            return Some(ExportHit::Format(fmt));
        }
        None
    }

    pub fn apply_hit(&mut self, hit: ExportHit) {
        match hit {
            ExportHit::TogglePanel => self.toggle_panel(),
            ExportHit::Preset(i) => self.set_preset(i),
            ExportHit::Fit(fit) => self.fit = fit,
            ExportHit::Aspect(aspect) => self.aspect = aspect,
            ExportHit::Format(fmt) => self.toggle_format(fmt),
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
        assert!(s.write_png && s.write_mask && s.write_json && s.write_svg);
        assert!(!s.write_heightmap);
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
        assert_eq!(s.hit((0.86, 0.39)), Some(ExportHit::TogglePanel));
        assert!(s.hit((0.86, 0.50)).is_none());
        assert!(s.contains_cursor((0.86, 0.39)));
        assert!(!s.contains_cursor((0.86, 0.50)));
        assert!(!s.contains_cursor((0.40, 0.39)));
    }

    #[test]
    fn open_panel_hits_two_column_presets_fit_aspect_and_formats() {
        let mut s = ExportSettings::default();
        s.panel_open = true;
        // 8 is left cell of row 0; 16 is right cell.
        assert_eq!(
            s.hit((0.76, PRESET_Y0 + 0.01)),
            Some(ExportHit::Preset(0))
        );
        assert_eq!(
            s.hit((0.92, PRESET_Y0 + 0.01)),
            Some(ExportHit::Preset(1))
        );
        // 512 is left cell of row 3 (index 6).
        let y512 = PRESET_Y0 + PRESET_ROW * 3.0 + 0.01;
        assert_eq!(s.hit((0.76, y512)), Some(ExportHit::Preset(6)));
        // 8192 is left cell of row 5 (index 10); right cell is empty.
        let y8k = PRESET_Y0 + PRESET_ROW * 5.0 + 0.01;
        assert_eq!(s.hit((0.76, y8k)), Some(ExportHit::Preset(10)));
        assert!(s.hit((0.92, y8k)).is_none());
        assert_eq!(s.hit((0.76, 0.664)), Some(ExportHit::Fit(FitMode::Pad)));
        assert_eq!(
            s.hit((0.92, 0.664)),
            Some(ExportHit::Fit(FitMode::CropAabb))
        );
        assert_eq!(
            s.hit((0.76, 0.712)),
            Some(ExportHit::Aspect(ExportAspect::Square))
        );
        assert_eq!(
            s.hit((0.92, 0.712)),
            Some(ExportHit::Aspect(ExportAspect::Native))
        );
        let span = PANEL_X1 - PANEL_X0;
        assert_eq!(
            s.hit((PANEL_X0 + span * 0.12, 0.768)),
            Some(ExportHit::Format(ExportFormat::Png))
        );
        assert_eq!(
            s.hit((PANEL_X0 + span * 0.38, 0.768)),
            Some(ExportHit::Format(ExportFormat::Mask))
        );
        assert_eq!(
            s.hit((PANEL_X0 + span * 0.62, 0.768)),
            Some(ExportHit::Format(ExportFormat::Json))
        );
        assert_eq!(
            s.hit((PANEL_X0 + span * 0.88, 0.768)),
            Some(ExportHit::Format(ExportFormat::Svg))
        );
        assert_eq!(
            s.hit((0.86, 0.816)),
            Some(ExportHit::Format(ExportFormat::Heightmap))
        );
        assert!(s.contains_cursor((0.86, 0.70)));
        assert!(PANEL_Y1 < 0.90, "card must sit above the thickness slider");
        assert!(FIT_Y0 >= PRESET_Y0 + PRESET_ROW * PRESET_ROWS as f32);
        assert!(FORMAT_Y1 <= HT_Y0);
        assert!(HT_Y1 <= PANEL_Y1);
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
        s.apply_hit(ExportHit::Format(ExportFormat::Svg));
        assert!(!s.write_svg);
        s.apply_hit(ExportHit::Format(ExportFormat::Heightmap));
        assert!(s.write_heightmap);
    }

    #[test]
    fn last_format_stays_on() {
        let mut s = ExportSettings::default();
        s.write_mask = false;
        s.write_json = false;
        s.write_svg = false;
        s.toggle_format(ExportFormat::Png);
        assert!(s.write_png, "PNG should snap back on if it was the last file");
    }

    #[test]
    fn present_vec_packs_flags() {
        let mut s = ExportSettings::default();
        // default: square, pad, png+mask+json+svg = bits 2+3+4+5 = 4+8+16+32 = 60
        assert_eq!(s.present_vec(), [0.0, 6.0, 60.0, 0.0]);
        s.panel_open = true;
        s.aspect = ExportAspect::Native;
        s.fit = FitMode::CropAabb;
        s.preset_index = 3;
        s.write_heightmap = true;
        s.write_svg = false;
        // native+crop+png+mask+json+height = 1+2+4+8+16+64 = 95
        assert_eq!(s.present_vec(), [1.0, 3.0, 95.0, 0.0]);
        assert!(s.format_on(ExportFormat::Png));
        assert!(!s.format_on(ExportFormat::Svg));
        assert_eq!(ExportFormat::Heightmap.as_str(), "heightmap");
    }
}
