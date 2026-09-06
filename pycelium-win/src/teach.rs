//! Control-scheme panel + hover teach callouts for the present-pass HUD.
//!
//! Hit-test boxes match `shaders/present.wgsl` and `export_settings.rs`.
//! Color swatches / bars stay drawable when Tab density is `off`, so those
//! rows remain hoverable. The right slab inset is also hit-tested. If a
//! control has no box (none today on the left HUD), teach cannot fire.

use crate::cutter::{SLIDER_X0, SLIDER_X1, SLIDER_Y0, SLIDER_Y1};
use crate::export_settings::{
    ASPECT_Y0, ASPECT_Y1, CHIP_Y0, CHIP_Y1, FIT_Y0, FIT_Y1, PANEL_X0, PANEL_X1, PANEL_Y1,
    PRESET_ROW, PRESET_Y0, SQUARE_PRESETS,
};
use crate::hud_font;

pub const HELP_COLS: usize = 26;
pub const HELP_ROWS: usize = 22;
pub const TIP_COLS: usize = 36;
pub const TIP_ROWS: usize = 8;
pub const HELP_BASE: usize = 0;
pub const TIP_BASE: usize = HELP_COLS * HELP_ROWS;
pub const OVERLAY_WORDS: usize = 1024;

const HUD_X0: f32 = 0.008;
const HUD_X1: f32 = 0.250;
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
}

#[derive(Clone, Debug)]
pub struct OverlayGpu {
    pub chars: [u32; OVERLAY_WORDS],
    pub overlay_ui: [f32; 4],
    pub help_rect: [f32; 4],
    pub tip_rect: [f32; 4],
    pub callout: [f32; 4],
}

impl Default for OverlayGpu {
    fn default() -> Self {
        Self {
            chars: [0; OVERLAY_WORDS],
            overlay_ui: [0.0; 4],
            help_rect: [0.0; 4],
            tip_rect: [0.0; 4],
            callout: [0.0; 4],
        }
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
    HudRow { id: HoverId::Tip, y0: 0.688, y1: 0.734 },
    HudRow { id: HoverId::Lineage, y0: 0.734, y1: 0.776 },
    HudRow { id: HoverId::Age, y0: 0.776, y1: 0.818 },
    HudRow { id: HoverId::Reserve, y0: 0.818, y1: 0.862 },
    HudRow { id: HoverId::Param, y0: 0.878, y1: 0.940 },
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
                && cursor.1 < PRESET_Y0 + PRESET_ROW * SQUARE_PRESETS.len() as f32
            {
                let mid = cursor.1;
                return (HoverId::ExportSize, [PANEL_X0, mid]);
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

    out.tip_rect = tip;
    out.callout = [anchor[0], anchor[1], attach[0], attach[1]];
    out.overlay_ui = [help_fade, tip_fade, 0.0, 0.0];
    out
}

fn pack_scheme(chars: &mut [u32]) {
    for (row, spec) in SCHEME_ROWS.iter().enumerate() {
        let mut line = format!("{:<8}", spec.keys);
        line.push_str(spec.action);
        blit_line(chars, HELP_BASE, HELP_COLS, row, &line);
    }
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
            "Depth of the orange view slab and the right-hand inset, in voxels. Keys [ ] move it. This is a viewing plane, not a capture export (press E for that). Shift makes the step coarse."
        }
        HoverId::Thick => {
            "How many voxels the view slab integrates. Semicolon and quote change it. Thicker slabs stack more layers in the inset so faint hyphae show up. This does not drive the capture cutter."
        }
        HoverId::Zoom => {
            "How much of the XY field the inset shows, as percent. Comma and period zoom; arrows pan. Shift makes these coarse. It is a loupe on the current Z slab, not camera orbit."
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
            "Live knob 1-8: chemotropism, nitrotropism, autotropism, persistence, maintenance, enzyme_k, branch cost, extension. Keys 1-8 select the slot. Minus and equals nudge the value. These do not change RAM presets."
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
            "Minus and equals (or underscore / plus) nudge the selected PARAM slot. Use 1-8 first to choose which rule you are tuning."
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
            "Square power-of-two export size, 8 through 8192 (8K). Click a row or wheel over the card. Applies when aspect is square. PNG, mask, SVG, and density_u8 use this canvas."
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
    }

    #[test]
    fn hud_boxes_hit_when_cursor_is_on_the_left() {
        let (id, anchor) = hit_test((0.04, 0.110), false, false, false);
        assert_eq!(id, HoverId::Tips);
        assert!((anchor[0] - HUD_X1).abs() < 1e-5);
        let (id, _) = hit_test((0.04, 0.910), false, false, false);
        assert_eq!(id, HoverId::Param);
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
        );
        assert_eq!(gpu.chars[0], hud_font::encode_char('H'));
        assert!(gpu.chars[TIP_BASE] > 0);
        assert!(gpu.overlay_ui[0] > 0.5);
        assert!(gpu.tip_rect[2] > gpu.tip_rect[0]);
    }

    #[test]
    fn export_open_moves_scheme_off_the_card() {
        let open = scheme_rect(true);
        assert!(open[2] < PANEL_X0);
        let closed = scheme_rect(false);
        assert!(closed[0] > 0.70);
    }
}
