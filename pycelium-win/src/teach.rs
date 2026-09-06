//! Control-scheme panel + hover teach callouts for the present-pass HUD.
//!
//! Hit-test boxes match `shaders/present.wgsl` and `export_settings.rs`.
//! Color swatches / bars stay drawable when Tab density is `off`, so those
//! rows remain hoverable. The right slab inset is also hit-tested. If a
//! control has no box (none today on the left HUD), teach cannot fire.

use crate::cutter::{SLIDER_X0, SLIDER_X1, SLIDER_Y0, SLIDER_Y1};
use crate::export_settings::{
    ASPECT_Y0, ASPECT_Y1, CHIP_Y0, CHIP_Y1, FIT_Y0, FIT_Y1, FORMAT_Y0, FORMAT_Y1, HT_Y0, HT_Y1,
    PANEL_X0, PANEL_X1, PANEL_Y1, PRESET_ROW, PRESET_ROWS, PRESET_Y0, SQUARE_PRESETS,
};
use crate::hud_font;

pub const HELP_COLS: usize = 26;
pub const HELP_ROWS: usize = 22;
pub const TIP_COLS: usize = 36;
pub const TIP_ROWS: usize = 8;
pub const NUDGE_COLS: usize = 28;
pub const NUDGE_ROWS: usize = 3;
pub const HELP_BASE: usize = 0;
pub const TIP_BASE: usize = HELP_COLS * HELP_ROWS;
pub const NUDGE_BASE: usize = TIP_BASE + TIP_COLS * TIP_ROWS;
pub const OVERLAY_WORDS: usize = 1024;

pub const HUD_X0: f32 = 0.008;
pub const HUD_X1: f32 = 0.250;
/// Thin framed tracks on SLICE Z / THICK / ZOOM / the eight PARAM rows. Match `present.wgsl`.
pub const SLIDER_TRACK_X0: f32 = 0.128;
pub const SLIDER_TRACK_X1: f32 = 0.246;
/// Compressed tip block so eight compact param sliders fit without a scroll.
pub const TIP_BLOCK_Y0: f32 = 0.688;
pub const TIP_BLOCK_Y1: f32 = 0.792;
pub const TIP_ROW_H: f32 = 0.026;
/// Shared param group. Eight rows; smaller tracks than the slice knobs.
pub const PARAM_BLOCK_Y0: f32 = 0.800;
pub const PARAM_BLOCK_Y1: f32 = 0.984;
pub const PARAM_ROW_H: f32 = 0.023;
const INSET: [f32; 4] = [0.72, 0.06, 0.98, 0.34];

/// Bottom-right, under the slab inset. When the export card is fully open
/// that column is taken, so the panel slides just right of the left HUD.
pub fn scheme_rect(export_panel_open: bool) -> [f32; 4] {
    if export_panel_open {
        [0.272, 0.548, 0.548, 0.888]
    } else {
        [0.728, 0.448, 0.992, 0.988]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HoverId {
    None,
    Fps,
    Tips,
    Fusions,
    Branches,
    Cn,
    Hypha,
    Cord,
    SolC,
    SolN,
    Enzyme,
    Organic,
    SliceZ,
    Thick,
    Zoom,
    Tip,
    Lineage,
    Age,
    Reserve,
    Param,
    ParamChemo,
    ParamNitro,
    ParamAuto,
    ParamPersist,
    ParamMaint,
    ParamEnzyme,
    ParamBranch,
    ParamExtend,
    Help,
    Orbit,
    Dolly,
    Pick,
    SliceKeys,
    ThickKeys,
    ZoomKeys,
    PanKeys,
    Shift,
    ParamKeys,
    Nudge,
    Tab,
    Capture,
    CaptureMode,
    Settings,
    PadCrop,
    Aspect,
    Axis,
    HtAxis,
    Heightmap,
    Write,
    Leave,
    Slider,
    Inset,
    ExportChip,
    ExportSize,
    ExportFit,
    ExportAspect,
    ExportFormat,
    ExportHeightmap,
}

#[derive(Clone, Debug)]
pub struct OverlayGpu {
    pub chars: [u32; OVERLAY_WORDS],
    pub overlay_ui: [f32; 4],
    pub help_rect: [f32; 4],
    pub tip_rect: [f32; 4],
    pub callout: [f32; 4],
    pub nudge_rect: [f32; 4],
}

impl Default for OverlayGpu {
    fn default() -> Self {
        Self {
            chars: [0; OVERLAY_WORDS],
            overlay_ui: [0.0; 4],
            help_rect: [0.0; 4],
            tip_rect: [0.0; 4],
            callout: [0.0; 4],
            nudge_rect: [0.0; 4],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HudSlider {
    SliceZ,
    Thick,
    Zoom,
    Param(u8),
}

pub fn param_row(slot: u32) -> (f32, f32) {
    let y0 = PARAM_BLOCK_Y0 + (slot % 8) as f32 * PARAM_ROW_H;
    (y0, y0 + PARAM_ROW_H)
}

pub fn param_row_mid(slot: u32) -> f32 {
    let (y0, y1) = param_row(slot);
    0.5 * (y0 + y1)
}

pub fn param_hover(slot: u32) -> HoverId {
    match slot % 8 {
        0 => HoverId::ParamChemo,
        1 => HoverId::ParamNitro,
        2 => HoverId::ParamAuto,
        3 => HoverId::ParamPersist,
        4 => HoverId::ParamMaint,
        5 => HoverId::ParamEnzyme,
        6 => HoverId::ParamBranch,
        _ => HoverId::ParamExtend,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NudgeKind {
    Param,
    SliceZ,
    Thick,
    Zoom,
    Pan,
}

impl NudgeKind {
    pub fn keys(self) -> &'static str {
        match self {
            Self::Param => "1-8 SELECT  -/= NUDGE",
            Self::SliceZ => "[ ] DEPTH  SHIFT COARSE",
            Self::Thick => "; ' THICK  SHIFT COARSE",
            Self::Zoom => ", . ZOOM  SHIFT COARSE",
            Self::Pan => "ARROWS PAN  SHIFT COARSE",
        }
    }

    pub fn row_y(self) -> f32 {
        match self {
            Self::Param => param_row_mid(0),
            Self::SliceZ => 0.570,
            Self::Thick => 0.612,
            Self::Zoom | Self::Pan => 0.656,
        }
    }
}

#[derive(Clone, Debug)]
pub struct NudgeToast {
    /// Which meter this toast belongs to (also drives `keys` / `row_y` at the call site).
    #[allow(dead_code)]
    pub kind: NudgeKind,
    pub title: String,
    pub value: String,
    pub keys: String,
    pub fade: f32,
    pub row_y: f32,
}

pub fn slider_t(x: f32) -> f32 {
    ((x - SLIDER_TRACK_X0) / (SLIDER_TRACK_X1 - SLIDER_TRACK_X0)).clamp(0.0, 1.0)
}

pub fn hud_slider_at(uv: (f32, f32)) -> Option<HudSlider> {
    if uv.0 < HUD_X0 || uv.0 > HUD_X1 {
        return None;
    }
    if uv.1 >= 0.548 && uv.1 < 0.592 {
        Some(HudSlider::SliceZ)
    } else if uv.1 >= 0.592 && uv.1 < 0.634 {
        Some(HudSlider::Thick)
    } else if uv.1 >= 0.634 && uv.1 < 0.682 {
        Some(HudSlider::Zoom)
    } else if uv.1 >= PARAM_BLOCK_Y0 && uv.1 < PARAM_BLOCK_Y1 {
        let i = ((uv.1 - PARAM_BLOCK_Y0) / PARAM_ROW_H).floor() as i32;
        if (0..8).contains(&i) {
            Some(HudSlider::Param(i as u8))
        } else {
            None
        }
    } else {
        None
    }
}

struct HudRow {
    id: HoverId,
    y0: f32,
    y1: f32,
}

const HUD_ROWS: &[HudRow] = &[
    HudRow { id: HoverId::Fps, y0: 0.050, y1: 0.095 },
    HudRow { id: HoverId::Tips, y0: 0.095, y1: 0.137 },
    HudRow { id: HoverId::Fusions, y0: 0.137, y1: 0.179 },
    HudRow { id: HoverId::Branches, y0: 0.179, y1: 0.221 },
    HudRow { id: HoverId::Cn, y0: 0.221, y1: 0.268 },
    HudRow { id: HoverId::Hypha, y0: 0.305, y1: 0.350 },
    HudRow { id: HoverId::Cord, y0: 0.350, y1: 0.392 },
    HudRow { id: HoverId::SolC, y0: 0.392, y1: 0.434 },
    HudRow { id: HoverId::SolN, y0: 0.434, y1: 0.476 },
    HudRow { id: HoverId::Enzyme, y0: 0.476, y1: 0.518 },
    HudRow { id: HoverId::Organic, y0: 0.518, y1: 0.555 },
    HudRow { id: HoverId::SliceZ, y0: 0.548, y1: 0.592 },
    HudRow { id: HoverId::Thick, y0: 0.592, y1: 0.634 },
    HudRow { id: HoverId::Zoom, y0: 0.634, y1: 0.682 },
    HudRow { id: HoverId::Tip, y0: 0.688, y1: 0.714 },
    HudRow { id: HoverId::Lineage, y0: 0.714, y1: 0.740 },
    HudRow { id: HoverId::Age, y0: 0.740, y1: 0.766 },
    HudRow { id: HoverId::Reserve, y0: 0.766, y1: 0.792 },
    HudRow { id: HoverId::ParamChemo, y0: 0.800, y1: 0.823 },
    HudRow { id: HoverId::ParamNitro, y0: 0.823, y1: 0.846 },
    HudRow { id: HoverId::ParamAuto, y0: 0.846, y1: 0.869 },
    HudRow { id: HoverId::ParamPersist, y0: 0.869, y1: 0.892 },
    HudRow { id: HoverId::ParamMaint, y0: 0.892, y1: 0.915 },
    HudRow { id: HoverId::ParamEnzyme, y0: 0.915, y1: 0.938 },
    HudRow { id: HoverId::ParamBranch, y0: 0.938, y1: 0.961 },
    HudRow { id: HoverId::ParamExtend, y0: 0.961, y1: 0.984 },
];

struct SchemeRow {
    keys: &'static str,
    action: &'static str,
    id: HoverId,
}

/// Live binds. Heightmap *write* is M (H is this panel, globally).
const SCHEME_ROWS: &[SchemeRow] = &[
    SchemeRow { keys: "H", action: "SCHEME", id: HoverId::Help },
    SchemeRow { keys: "DRAG", action: "ORBIT", id: HoverId::Orbit },
    SchemeRow { keys: "WHEEL", action: "DOLLY", id: HoverId::Dolly },
    SchemeRow { keys: "CLICK", action: "PICK TIP", id: HoverId::Pick },
    SchemeRow { keys: "[ ]", action: "SLICE Z", id: HoverId::SliceKeys },
    SchemeRow { keys: "; '", action: "THICK", id: HoverId::ThickKeys },
    SchemeRow { keys: ", .", action: "XY ZOOM", id: HoverId::ZoomKeys },
    SchemeRow { keys: "ARROWS", action: "PAN SLAB", id: HoverId::PanKeys },
    SchemeRow { keys: "SHIFT", action: "COARSE", id: HoverId::Shift },
    SchemeRow { keys: "1-8", action: "PARAM", id: HoverId::ParamKeys },
    SchemeRow { keys: "- =", action: "NUDGE", id: HoverId::Nudge },
    SchemeRow { keys: "TAB", action: "DENSITY", id: HoverId::Tab },
    SchemeRow { keys: "E", action: "CAPTURE", id: HoverId::Capture },
    SchemeRow { keys: "C", action: "2D / RICH", id: HoverId::CaptureMode },
    SchemeRow { keys: "S", action: "SETTINGS", id: HoverId::Settings },
    SchemeRow { keys: "P", action: "PAD / CROP", id: HoverId::PadCrop },
    SchemeRow { keys: "A", action: "SQ / NATIVE", id: HoverId::Aspect },
    SchemeRow { keys: "X", action: "FACE AXIS", id: HoverId::Axis },
    SchemeRow { keys: "Y", action: "HT AXIS", id: HoverId::HtAxis },
    SchemeRow { keys: "M", action: "HEIGHTMAP", id: HoverId::Heightmap },
    SchemeRow { keys: "ENTER", action: "WRITE", id: HoverId::Write },
    SchemeRow { keys: "ESC", action: "LEAVE", id: HoverId::Leave },
];

pub fn in_rect(uv: (f32, f32), r: [f32; 4]) -> bool {
    uv.0 >= r[0] && uv.0 <= r[2] && uv.1 >= r[1] && uv.1 <= r[3]
}

pub fn hit_test(
    cursor: (f32, f32),
    scheme_on: bool,
    capturing: bool,
    export_panel_open: bool,
) -> (HoverId, [f32; 2]) {
    if scheme_on {
        let rect = scheme_rect(export_panel_open && capturing);
        if in_rect(cursor, rect) {
            let t = ((cursor.1 - rect[1]) / (rect[3] - rect[1]).max(1e-5)).clamp(0.0, 0.999);
            let row = (t * HELP_ROWS as f32).floor() as usize;
            if let Some(s) = SCHEME_ROWS.get(row) {
                let mid = rect[1] + (row as f32 + 0.5) * (rect[3] - rect[1]) / HELP_ROWS as f32;
                return (s.id, [rect[0], mid]);
            }
        }
    }

    if capturing {
        if cursor.0 >= PANEL_X0
            && cursor.0 <= PANEL_X1
            && cursor.1 >= CHIP_Y0
            && cursor.1 <= CHIP_Y1
        {
            return (HoverId::ExportChip, [PANEL_X0, 0.5 * (CHIP_Y0 + CHIP_Y1)]);
        }
        if export_panel_open && cursor.0 >= PANEL_X0 && cursor.0 <= PANEL_X1 {
            if cursor.1 >= PRESET_Y0
                && cursor.1 < PRESET_Y0 + PRESET_ROW * PRESET_ROWS as f32
            {
                let mid = cursor.1;
                let i = {
                    let row = ((cursor.1 - PRESET_Y0) / PRESET_ROW).floor() as i32;
                    let col = if cursor.0 < 0.5 * (PANEL_X0 + PANEL_X1) {
                        0
                    } else {
                        1
                    };
                    row * 2 + col
                };
                if i >= 0 && (i as usize) < SQUARE_PRESETS.len() {
                    return (HoverId::ExportSize, [PANEL_X0, mid]);
                }
            }
            if cursor.1 >= FIT_Y0 && cursor.1 <= FIT_Y1 {
                return (HoverId::ExportFit, [PANEL_X0, 0.5 * (FIT_Y0 + FIT_Y1)]);
            }
            if cursor.1 >= ASPECT_Y0 && cursor.1 <= ASPECT_Y1 {
                return (
                    HoverId::ExportAspect,
                    [PANEL_X0, 0.5 * (ASPECT_Y0 + ASPECT_Y1)],
                );
            }
            if cursor.1 >= FORMAT_Y0 && cursor.1 <= FORMAT_Y1 {
                return (
                    HoverId::ExportFormat,
                    [PANEL_X0, 0.5 * (FORMAT_Y0 + FORMAT_Y1)],
                );
            }
            if cursor.1 >= HT_Y0 && cursor.1 <= HT_Y1 {
                return (
                    HoverId::ExportHeightmap,
                    [PANEL_X0, 0.5 * (HT_Y0 + HT_Y1)],
                );
            }
            if cursor.1 >= CHIP_Y0 && cursor.1 <= PANEL_Y1 {
                return (HoverId::Settings, [PANEL_X0, cursor.1]);
            }
        }
        if cursor.0 >= SLIDER_X0 - 0.02
            && cursor.0 <= SLIDER_X1 + 0.02
            && cursor.1 >= SLIDER_Y0 - 0.01
            && cursor.1 <= SLIDER_Y1 + 0.01
        {
            return (
                HoverId::Slider,
                [0.5 * (SLIDER_X0 + SLIDER_X1), 0.5 * (SLIDER_Y0 + SLIDER_Y1)],
            );
        }
    }

    if cursor.0 >= HUD_X0 && cursor.0 <= HUD_X1 {
        for row in HUD_ROWS {
            if cursor.1 >= row.y0 && cursor.1 < row.y1 {
                return (row.id, [HUD_X1, 0.5 * (row.y0 + row.y1)]);
            }
        }
    }

    if in_rect(cursor, INSET) {
        return (HoverId::Inset, [INSET[0], 0.5 * (INSET[1] + INSET[3])]);
    }

    (HoverId::None, [cursor.0, cursor.1])
}

pub fn pack_overlay(
    scheme_on: bool,
    export_panel_open: bool,
    capturing: bool,
    hover: HoverId,
    cursor: (f32, f32),
    anchor: [f32; 2],
    help_fade: f32,
    tip_fade: f32,
    nudge: Option<&NudgeToast>,
) -> OverlayGpu {
    let mut out = OverlayGpu::default();
    let help = scheme_rect(export_panel_open && capturing);
    out.help_rect = help;
    if scheme_on {
        pack_scheme(&mut out.chars);
    }

    let mut tip = [0.0; 4];
    let mut attach = [0.0; 2];
    if hover != HoverId::None {
        let lines = pack_tip(&mut out.chars, hover);
        tip = place_tip(cursor, lines, help, scheme_on);
        attach = tip_attach(anchor, tip);
    }

    let mut nudge_rect = [0.0; 4];
    let mut nudge_fade = 0.0;
    if let Some(toast) = nudge {
        if toast.fade > 0.004 {
            pack_nudge(&mut out.chars, toast);
            nudge_rect = place_nudge(toast, help, scheme_on);
            nudge_fade = toast.fade;
        }
    }

    out.tip_rect = tip;
    out.callout = [anchor[0], anchor[1], attach[0], attach[1]];
    out.nudge_rect = nudge_rect;
    out.overlay_ui = [help_fade, tip_fade, nudge_fade, 0.0];
    out
}

fn pack_scheme(chars: &mut [u32]) {
    for (row, spec) in SCHEME_ROWS.iter().enumerate() {
        let mut line = format!("{:<8}", spec.keys);
        line.push_str(spec.action);
        blit_line(chars, HELP_BASE, HELP_COLS, row, &line);
    }
}

fn pack_nudge(chars: &mut [u32], toast: &NudgeToast) {
    blit_line(chars, NUDGE_BASE, NUDGE_COLS, 0, &toast.title);
    blit_line(chars, NUDGE_BASE, NUDGE_COLS, 1, &toast.value);
    blit_line(chars, NUDGE_BASE, NUDGE_COLS, 2, &toast.keys);
}

fn place_nudge(toast: &NudgeToast, help: [f32; 4], scheme_on: bool) -> [f32; 4] {
    let w = 0.268;
    let h = 0.074;
    let mut x = 0.258;
    let mut y = (toast.row_y - 0.012).clamp(0.012, 0.984 - h);
    let mut rect = [x, y, x + w, y + h];
    if scheme_on && rects_overlap(rect, help) {
        x = (help[2] + 0.012).min(0.990 - w);
        rect = [x, y, x + w, y + h];
        if rects_overlap(rect, help) {
            y = (help[1] - h - 0.012).max(0.012);
            x = 0.258;
            rect = [x, y, x + w, y + h];
        }
    }
    rect
}

fn pack_tip(chars: &mut [u32], id: HoverId) -> usize {
    let lines = wrap_text(tip_text(id), TIP_COLS);
    let n = lines.len().min(TIP_ROWS);
    for (i, line) in lines.iter().take(n).enumerate() {
        blit_line(chars, TIP_BASE, TIP_COLS, i, line);
    }
    n.max(1)
}

fn blit_line(chars: &mut [u32], base: usize, cols: usize, row: usize, text: &str) {
    for (i, c) in text.chars().take(cols).enumerate() {
        let idx = base + row * cols + i;
        if idx < chars.len() {
            chars[idx] = hud_font::encode_char(c);
        }
    }
}

pub fn wrap_text(text: &str, cols: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cur = String::new();
    for word in text.split_whitespace() {
        let w = word.to_ascii_uppercase();
        if w.len() > cols {
            if !cur.is_empty() {
                lines.push(std::mem::take(&mut cur));
            }
            for chunk in w.as_bytes().chunks(cols) {
                lines.push(String::from_utf8_lossy(chunk).into_owned());
            }
            continue;
        }
        if cur.is_empty() {
            cur = w;
        } else if cur.len() + 1 + w.len() <= cols {
            cur.push(' ');
            cur.push_str(&w);
        } else {
            lines.push(std::mem::take(&mut cur));
            cur = w;
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

fn place_tip(cursor: (f32, f32), lines: usize, help: [f32; 4], scheme_on: bool) -> [f32; 4] {
    let w = 0.292;
    let h = 0.018 * lines as f32 + 0.018;
    let mut x = cursor.0 + 0.024;
    let mut y = cursor.1 + 0.022;
    if x + w > 0.990 {
        x = (cursor.0 - w - 0.020).max(0.012);
    }
    if y + h > 0.984 {
        y = (cursor.1 - h - 0.018).max(0.012);
    }
    let mut rect = [x, y, x + w, y + h];
    if scheme_on && rects_overlap(rect, help) {
        if cursor.0 < 0.5 {
            x = (help[0] - w - 0.016).max(0.012);
        } else {
            x = (help[2] + 0.012).min(0.990 - w);
        }
        rect = [x, y, x + w, y + h];
    }
    rect
}

fn rects_overlap(a: [f32; 4], b: [f32; 4]) -> bool {
    a[0] < b[2] && a[2] > b[0] && a[1] < b[3] && a[3] > b[1]
}

fn tip_attach(anchor: [f32; 2], tip: [f32; 4]) -> [f32; 2] {
    let mid_y = 0.5 * (tip[1] + tip[3]);
    let y = anchor[1].clamp(tip[1], tip[3]);
    if anchor[0] <= 0.5 * (tip[0] + tip[2]) {
        [tip[0], y.max(tip[1] + 0.006).min(mid_y.max(tip[1] + 0.006))]
    } else {
        [tip[2], y]
    }
}

fn tip_text(id: HoverId) -> &'static str {
    match id {
        HoverId::None => "",
        HoverId::Fps => {
            "How fast this window paints, in frames per second. Not a biology meter. If it drops, the GPU is spending more time marching the volume or counting tips. Bench mode never opens this HUD."
        }
        HoverId::Tips => {
            "Live apical tips: the growing points at the ends of hyphae. Each tip senses nearby food and nitrogen, then extends or waits. A baker or scientist watches this to see whether the colony is exploring or stalled."
        }
        HoverId::Fusions => {
            "Anastomosis events. When two live tips meet and the sim judges them close enough, they fuse and can share resources. In real fungi this welds a network; here it is a counted handshake. Rising fusions usually mean the mesh is knitting, not just radiating."
        }
        HoverId::Branches => {
            "New side tips born from an existing hypha when reserve and age allow. Branching costs carbon (see PARAM branch cost). More branches fill space faster and raise the chance of finding litter."
        }
        HoverId::Cn => {
            "Soluble carbon divided by soluble nitrogen in the brick. Hyphae need both; a skewed ratio means one nutrient is scarce. Enzyme work and uptake feel this mix. The readout is soil water, not the fungus body."
        }
        HoverId::Hypha => {
            "Hypha biomass: the teal field of filament walls occupying soil voxels. This is the physical network. Growth spends internal reserve to lay this material down. The bar is a strided census, not a wet-lab dry weight."
        }
        HoverId::Cord => {
            "Internal carbon stored in cords, the thicker transport routes. Think of it as the pantry moving through the network, not the soil food. High cord with low hypha can mean the fungus is banking rather than building."
        }
        HoverId::SolC => {
            "Soluble carbon in the soil water: already cleaved food the tips can drink. Distinct from ORGANIC, which is still locked as polymer. Chemotropism steers tips toward this rust field."
        }
        HoverId::SolN => {
            "Soluble nitrogen in the soil water. Same idea as SOL C but for N. Nitrotropism pulls tips toward it. Together with SOL C it sets the C:N readout a baker would use to judge whether the pedon is balanced."
        }
        HoverId::Enzyme => {
            "Exoenzyme the tips leak into the soil to cut ORGANIC into soluble C and N. Without it, litter sits uncleaved. The PARAM enzyme_k knob scales how fast this outside digestion happens."
        }
        HoverId::Organic => {
            "Uncleaved polymer: raw litter. The fungus cannot eat this until exoenzyme works. A brown bar falling while SOL C rises is the digestion story. This is the pantry still wrapped, not the meal."
        }
        HoverId::SliceZ => {
            "Depth of the orange view slab and the right-hand inset, in voxels. Keys [ ] move it, or drag the framed slider. This is a viewing plane, not a capture export (press E for that). Shift makes the step coarse."
        }
        HoverId::Thick => {
            "How many voxels the view slab integrates. Semicolon and quote change it, or drag the framed slider. Thicker slabs stack more layers in the inset so faint hyphae show up. This does not drive the capture cutter."
        }
        HoverId::Zoom => {
            "How much of the XY field the inset shows, as percent. Comma and period zoom, arrows pan, or drag the framed slider. Shift makes these coarse. It is a loupe on the current Z slab, not camera orbit."
        }
        HoverId::Tip => {
            "Which apical slot you clicked. Click in the volume to pick the nearest live tip. Capture mode disables pick. Sparse Tab mode keeps this block readable after a pick."
        }
        HoverId::Lineage => {
            "Inoculum / colony id of the picked tip. Separate inocula stay tagged so you can see whose hypha you selected. Useful when more than one starter is in the pedon."
        }
        HoverId::Age => {
            "How long that tip has been alive in sim time. Young tips cannot branch until min branch age. Old tips may have spent their reserve and sit still even near food."
        }
        HoverId::Reserve => {
            "Internal carbon the picked tip is carrying (shown times 100). Extension and branching spend this. A starved tip stops growing even if soil food is nearby, until uptake refills it."
        }
        HoverId::Param | HoverId::ParamKeys => {
            "Eight left-HUD sliders: chemotropism, nitrotropism, autotropism, persistence, maintenance, enzyme_k, branch cost, extension. Keys 1-8 select. Minus and equals nudge the focused slot. Drag any track to set it and select it."
        }
        HoverId::ParamChemo => {
            "Chemotropism steers each tip toward soluble carbon, the rust SOL C field. High: tips hunt food plumes and bend hard toward litter. Low: they ignore C gradients and wander or follow persistence, nitrogen, or autotropism instead. Key 1. Drag the track or use minus and equals."
        }
        HoverId::ParamNitro => {
            "Nitrotropism pulls tips toward soluble nitrogen (SOL N). High: tips, especially when reserve is low, chase N plumes, useful in an N-poor pedon. Low: nitrogen barely steers, so a tip may linger in C-rich, N-poor soil. The pull eases as the pantry fills. Key 2."
        }
        HoverId::ParamAuto => {
            "Autotropism turns a tip away from its own trail and nearby biomass (autocrine plus hypha). High: tips avoid crowding, fan into empty soil, and recross old mycelium less. Low: they may pile onto existing hyphae. This is keep-off-the-old-network, not food seeking. Key 3."
        }
        HoverId::ParamPersist => {
            "Persistence keeps the current heading versus turning to tropisms. High: long straight runs; the tip commits and only slowly bends. Low: twitchy steering, yanked by every nearby C, N, or self gradient. High persist plus high chemo still hunts, but in smoother arcs. Key 4."
        }
        HoverId::ParamMaint => {
            "Maintenance is the carbon living biomass burns just to stay alive. High: the network is expensive; cords and hyphae drain internal C and can thin if unfed. Low: cheap upkeep, so a colony banks reserve and survives lean soil. A tax on walls, not tip steps. Key 5."
        }
        HoverId::ParamEnzyme => {
            "Enzyme_k scales how fast leaked exoenzyme cuts ORGANIC polymer into soluble C and N. High: litter unlocks quickly; SOL C and SOL N rise while the brown ORGANIC bar falls. Low: enzyme sits on uncleaved polymer and the meal stays wrapped. This is outside digestion. Key 6."
        }
        HoverId::ParamBranch => {
            "Branch cost is how much reserve a tip must hold before it can birth a side tip. High: forks are expensive; fewer branches, longer unbranched runs. Low: cheap forks, denser trees, more tips hunting litter, and more carbon spent on new heads. Watch the BRANCHES census. Key 7."
        }
        HoverId::ParamExtend => {
            "Extension (max_extension) is how far a tip steps each tick when it has reserve. High: fast explorers that cover voxels quickly, but they can overshoot food and spend reserve faster. Low: short cautious steps; the colony creeps. Starved tips still take shorter steps. Key 8."
        }
        HoverId::Help => {
            "H toggles this corner control-scheme panel, always, including in capture. It never writes a heightmap. Hover a row here, or a left HUD meter, for a teach callout on a white string."
        }
        HoverId::Orbit => {
            "Left-drag orbits the camera around the pedon. This is the view, not the fungus. Grabbing a capture slider handle or the export card does not start an orbit."
        }
        HoverId::Dolly => {
            "Mouse wheel moves the eye in and out (dolly). Shift+wheel in capture instead thickens the cutter. Wheel over the export card cycles the square size preset."
        }
        HoverId::Pick => {
            "Click in the volume to select the nearest live tip. The TIP / LINEAGE / AGE / RESERVE block then tracks that slot. Disabled while capturing so the cutter can keep the mouse."
        }
        HoverId::SliceKeys => {
            "Open bracket and close bracket step the view-slab depth. Shift makes the step coarse (8 voxels). This drives the orange plane and the inset, not the capture cutter."
        }
        HoverId::ThickKeys => {
            "Semicolon and quote change view-slab thickness. Thicker integrates more Z layers in the inset. Capture thickness is the bottom slider (or Shift+wheel) instead."
        }
        HoverId::ZoomKeys => {
            "Comma and period zoom the inset XY field. They do not dolly the 3D camera (that is the wheel). Shift coarsens the step."
        }
        HoverId::PanKeys => {
            "Arrow keys pan the inset inside the current zoom. There is no separate camera pan: drag orbits, wheel dollies. Shift coarsens the slab pan."
        }
        HoverId::Shift => {
            "Hold Shift to coarsen slice depth, thickness, zoom, and pan. In capture, Shift+wheel also grows cutter thickness. It is a modifier, not a mode."
        }
        HoverId::Nudge => {
            "Minus and equals (or underscore / plus) nudge the selected PARAM slot. Use 1-8 first, or drag any of the eight left-HUD sliders to select that rule and set its value."
        }
        HoverId::Tab => {
            "Tab cycles label density: rich (full English names), sparse (names on hover or pick), then off (boxes, bars, and small digits only). Hover teach still works in off because the boxes stay hit-testable."
        }
        HoverId::Capture => {
            "E enters or leaves capture mode. The cutter follows the mouse through the cube and can snap to a face. Enter writes files. Esc leaves (or closes the settings card first)."
        }
        HoverId::CaptureMode => {
            "C cycles 2D squash (max-project N layers onto one plane) and rich box (a thin 3D slab whose JSON keeps volume samples). The PNG is still a density view either way."
        }
        HoverId::Settings | HoverId::ExportChip => {
            "S toggles the export settings card on the right under the inset. Click the EXPORT chip, pick a square size, or wheel the card. Default bake is 512 by 512 pad."
        }
        HoverId::PadCrop | HoverId::ExportFit => {
            "P cycles pad and crop. Pad letterboxes the full plane into the square. Crop trims to the occupied density box first, then pads. A baker uses crop when the mycelium is a small island."
        }
        HoverId::Aspect | HoverId::ExportAspect => {
            "A cycles square power-of-two versus native slab aspect. Square is the default (presets 8 through 8192). Native keeps the extracted width by height, the old behavior."
        }
        HoverId::Axis => {
            "X cycles the cutter axis by hand if face snap misses. Hover a face center to snap to that normal, or a face mid-edge to rotate 90 degrees onto the free axis."
        }
        HoverId::HtAxis => {
            "Y cycles the optional heightmap axis and turns heightmap export on. The greyscale PNG stores position along that axis. It does not toggle this help panel."
        }
        HoverId::Heightmap => {
            "M toggles writing the optional heightmap PNG. This used to be H; H is now the control-scheme panel in every mode. Y still picks the height axis and enables the write."
        }
        HoverId::Write => {
            "Enter writes PNG, mask, JSON, and SVG into the export directory. A heightmap PNG is included only when M (or Y) has armed it. Stem is timestamped."
        }
        HoverId::Leave => {
            "Esc closes the export settings card if it is open, then leaves capture. When capture is off, Esc quits the window."
        }
        HoverId::Slider => {
            "Two-ended orange bar: center follows cursor depth along the snapped axis. Drag either handle, or Shift+wheel, to grow N layers / box thickness. Always symmetric."
        }
        HoverId::Inset => {
            "Orthogonal squash of hypha (teal) plus soluble C (rust) in the current Z slab. Same depth, thickness, and zoom as the SLICE Z / THICK / ZOOM meters. Not the capture bake."
        }
        HoverId::ExportSize => {
            "Square power-of-two export size, 8 through 8192 (8K). Click a cell or wheel over the card. Applies when aspect is square. PNG, mask, SVG, and density_u8 use this canvas."
        }
        HoverId::ExportFormat => {
            "Click PNG, MASK, JSON, or SVG to include or skip that file on the next Enter write. At least one file stays armed. These used to be labels only."
        }
        HoverId::ExportHeightmap => {
            "Click to arm the optional heightmap PNG (same as M). Y still picks the height axis. H never writes a heightmap."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheme_lists_h_for_panel_and_m_for_heightmap() {
        let h = SCHEME_ROWS.iter().find(|r| r.id == HoverId::Help).unwrap();
        let m = SCHEME_ROWS
            .iter()
            .find(|r| r.id == HoverId::Heightmap)
            .unwrap();
        assert_eq!(h.keys, "H");
        assert_eq!(m.keys, "M");
        assert_eq!(SCHEME_ROWS.len(), HELP_ROWS);
        assert!(TIP_BASE + TIP_COLS * TIP_ROWS <= OVERLAY_WORDS);
        assert!(NUDGE_BASE + NUDGE_COLS * NUDGE_ROWS <= OVERLAY_WORDS);
    }

    #[test]
    fn hud_boxes_hit_when_cursor_is_on_the_left() {
        let (id, anchor) = hit_test((0.04, 0.110), false, false, false);
        assert_eq!(id, HoverId::Tips);
        assert!((anchor[0] - HUD_X1).abs() < 1e-5);
        let (id, _) = hit_test((0.04, 0.811), false, false, false);
        assert_eq!(id, HoverId::ParamChemo);
        let (id, _) = hit_test((0.04, 0.972), false, false, false);
        assert_eq!(id, HoverId::ParamExtend);
        let (id, _) = hit_test((0.40, 0.110), false, false, false);
        assert_eq!(id, HoverId::None);
    }

    #[test]
    fn scheme_and_export_hits() {
        let rect = scheme_rect(false);
        let y0 = rect[1] + 0.01;
        let (id, _) = hit_test((0.80, y0), true, false, false);
        assert_eq!(id, HoverId::Help);
        let (id, _) = hit_test((0.86, 0.39), false, true, false);
        assert_eq!(id, HoverId::ExportChip);
        let (id, _) = hit_test((0.50, 0.93), false, true, false);
        assert_eq!(id, HoverId::Slider);
        let (id, _) = hit_test((0.80, 0.18), false, false, false);
        assert_eq!(id, HoverId::Inset);
        let (id, _) = hit_test((0.76, 0.768), false, true, true);
        assert_eq!(id, HoverId::ExportFormat);
        let (id, _) = hit_test((0.86, 0.816), false, true, true);
        assert_eq!(id, HoverId::ExportHeightmap);
    }

    #[test]
    fn wrap_fits_columns_and_folds_case() {
        let lines = wrap_text("Hello there mycelium network", 10);
        assert!(lines.iter().all(|l| l.len() <= 10));
        assert_eq!(lines[0], "HELLO");
        assert!(tip_text(HoverId::Fusions).contains("Anastomosis"));
        assert!(wrap_text(tip_text(HoverId::Hypha), TIP_COLS).len() <= TIP_ROWS);
        assert!(wrap_text(tip_text(HoverId::Fusions), TIP_COLS).len() <= TIP_ROWS);
        assert!(wrap_text(tip_text(HoverId::Tab), TIP_COLS).len() <= TIP_ROWS);
        assert!(wrap_text(tip_text(HoverId::Param), TIP_COLS).len() <= TIP_ROWS);
        assert!(wrap_text(tip_text(HoverId::ExportFormat), TIP_COLS).len() <= TIP_ROWS);
        assert!(wrap_text(tip_text(HoverId::ExportHeightmap), TIP_COLS).len() <= TIP_ROWS);
        for slot in 0..8u32 {
            let id = param_hover(slot);
            let n = wrap_text(tip_text(id), TIP_COLS).len();
            assert!(n <= TIP_ROWS, "param {slot} teach wraps to {n} lines");
            let text = tip_text(id);
            assert!(
                text.contains("High:") && text.contains("Low:"),
                "param {slot} must explain high vs low"
            );
        }
    }

    #[test]
    fn pack_writes_scheme_glyphs() {
        let gpu = pack_overlay(
            true,
            false,
            false,
            HoverId::Tips,
            (0.20, 0.11),
            [0.25, 0.11],
            1.0,
            1.0,
            None,
        );
        assert_eq!(gpu.chars[0], hud_font::encode_char('H'));
        assert!(gpu.chars[TIP_BASE] > 0);
        assert!(gpu.overlay_ui[0] > 0.5);
        assert!(gpu.tip_rect[2] > gpu.tip_rect[0]);
    }

    #[test]
    fn pack_writes_nudge_toast() {
        let toast = NudgeToast {
            kind: NudgeKind::Param,
            title: "1 CHEMOTROPISM".into(),
            value: "1.06 TO 1.10".into(),
            keys: NudgeKind::Param.keys().into(),
            fade: 1.0,
            row_y: param_row_mid(0),
        };
        let gpu = pack_overlay(
            false,
            false,
            false,
            HoverId::None,
            (0.20, 0.90),
            [0.25, 0.90],
            0.0,
            0.0,
            Some(&toast),
        );
        assert_eq!(gpu.chars[NUDGE_BASE], hud_font::encode_char('1'));
        assert!(gpu.overlay_ui[2] > 0.5);
        assert!(gpu.nudge_rect[2] > gpu.nudge_rect[0]);
        assert!(gpu.nudge_rect[0] >= HUD_X1);
    }

    #[test]
    fn hud_slider_rows_match_left_panel() {
        assert_eq!(hud_slider_at((0.16, 0.570)), Some(HudSlider::SliceZ));
        assert_eq!(hud_slider_at((0.16, 0.610)), Some(HudSlider::Thick));
        assert_eq!(hud_slider_at((0.16, 0.650)), Some(HudSlider::Zoom));
        assert_eq!(hud_slider_at((0.16, 0.811)), Some(HudSlider::Param(0)));
        assert_eq!(hud_slider_at((0.16, 0.972)), Some(HudSlider::Param(7)));
        assert!(hud_slider_at((0.16, 0.110)).is_none());
        assert!(hud_slider_at((0.40, 0.811)).is_none());
        assert!((slider_t(SLIDER_TRACK_X0) - 0.0).abs() < 1e-5);
        assert!((slider_t(SLIDER_TRACK_X1) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn eight_param_rows_pack_under_tip_block() {
        assert!((TIP_BLOCK_Y1 - TIP_BLOCK_Y0 - 4.0 * TIP_ROW_H).abs() < 1e-5);
        assert!((PARAM_BLOCK_Y1 - PARAM_BLOCK_Y0 - 8.0 * PARAM_ROW_H).abs() < 1e-5);
        assert!(TIP_BLOCK_Y1 <= PARAM_BLOCK_Y0 + 1e-5);
        assert!(PARAM_BLOCK_Y1 <= 0.990);
        for slot in 0..8u32 {
            let (y0, y1) = param_row(slot);
            let mid = 0.5 * (y0 + y1);
            assert_eq!(hud_slider_at((0.16, mid)), Some(HudSlider::Param(slot as u8)));
            let (id, _) = hit_test((0.04, mid), false, false, false);
            assert_eq!(id, param_hover(slot));
        }
    }

    #[test]
    fn export_open_moves_scheme_off_the_card() {
        let open = scheme_rect(true);
        assert!(open[2] < PANEL_X0);
        let closed = scheme_rect(false);
        assert!(closed[0] > 0.70);
    }
}
