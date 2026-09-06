/// One soil voxel in the host pedon.
/// Organic polymer is not taken up until exoenzymes cleave it.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SoilVoxel {
    pub organic: f32,
    pub soluble_c: f32,
    pub soluble_n: f32,
    pub moisture: f32,
}

/// Apical hyphal tip in 3-space. Lineage is the inoculum / colony id.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Tip {
    pub pos: [f32; 3],
    pub age: f32,
    pub dir: [f32; 3],
    pub reserve: f32,
    pub state: f32,
    pub lineage: u32,
    pub parent: u32,
    pub flags: u32,
}

impl Tip {
    pub const DORMANT: Self = Self {
        pos: [0.0, 0.0, 0.0],
        age: 0.0,
        dir: [1.0, 0.0, 0.0],
        reserve: 0.0,
        state: 0.0,
        lineage: 0,
        parent: 0,
        flags: 0,
    };
}

/// Shared with every compute shader. 16-byte aligned.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SimUniforms {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub tip_count: u32,
    pub time: f32,
    pub sensor_distance: f32,
    pub branch_angle: f32,
    pub max_extension: f32,
    pub min_branch_age: f32,
    pub km_c: f32,
    pub km_n: f32,
    pub uptake_v: f32,
    pub enzyme_k: f32,
    pub maintenance: f32,
    pub yield_c: f32,
    pub auto_weight: f32,
    pub chemo_weight: f32,
    pub nitro_weight: f32,
    pub persist: f32,
    pub anastomosis_th: f32,
    pub branch_cost: f32,
    pub field_decay: f32,
    pub slice_z: f32,
    pub seed: u32,
    pub pick_ox: f32,
    pub pick_oy: f32,
    pub pick_oz: f32,
    pub pick_active: f32,
    pub pick_dx: f32,
    pub pick_dy: f32,
    pub pick_dz: f32,
    pub pad2: f32,
}

impl SimUniforms {
    pub fn new(width: u32, height: u32, depth: u32, tip_count: u32) -> Self {
        Self {
            width,
            height,
            depth,
            tip_count,
            time: 0.0,
            sensor_distance: 6.0,
            branch_angle: 1.22,
            max_extension: 0.95,
            min_branch_age: 16.0,
            km_c: 0.18,
            km_n: 0.10,
            uptake_v: 0.050,
            enzyme_k: 0.032,
            maintenance: 0.0012,
            yield_c: 0.52,
            auto_weight: 0.90,
            chemo_weight: 1.10,
            nitro_weight: 0.75,
            persist: 1.45,
            anastomosis_th: 0.30,
            branch_cost: 0.58,
            field_decay: 1.0,
            slice_z: depth as f32 * 0.35,
            seed: 0xA31C_5EED,
            pick_ox: 0.0,
            pick_oy: 0.0,
            pick_oz: 0.0,
            pick_active: 0.0,
            pick_dx: 0.0,
            pick_dy: 0.0,
            pick_dz: 1.0,
            pad2: 0.0,
        }
    }

    pub fn param_name(slot: u32) -> &'static str {
        match slot % 8 {
            0 => "chemotropism",
            1 => "nitrotropism",
            2 => "autotropism",
            3 => "persistence",
            4 => "maintenance",
            5 => "enzyme_k",
            6 => "branch_cost",
            _ => "extension",
        }
    }

    pub fn param_value(&self, slot: u32) -> f32 {
        match slot % 8 {
            0 => self.chemo_weight,
            1 => self.nitro_weight,
            2 => self.auto_weight,
            3 => self.persist,
            4 => self.maintenance,
            5 => self.enzyme_k,
            6 => self.branch_cost,
            _ => self.max_extension,
        }
    }

    pub fn adjust(&mut self, slot: u32, delta: f32) {
        let scale = |v: &mut f32, lo, hi| *v = (*v + delta).clamp(lo, hi);
        match slot % 8 {
            0 => scale(&mut self.chemo_weight, 0.0, 3.0),
            1 => scale(&mut self.nitro_weight, 0.0, 3.0),
            2 => scale(&mut self.auto_weight, 0.0, 3.0),
            3 => scale(&mut self.persist, 0.2, 3.0),
            4 => scale(&mut self.maintenance, 0.0, 0.02),
            5 => scale(&mut self.enzyme_k, 0.0, 0.12),
            6 => scale(&mut self.branch_cost, 0.1, 2.0),
            _ => scale(&mut self.max_extension, 0.2, 2.2),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PresentUniforms {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub out_w: u32,
    pub out_h: u32,
    pub _pad0: u32,
    pub slice_z: f32,
    pub fps: f32,
    pub eye: [f32; 4],
    pub target: [f32; 4],
    pub live_tips: f32,
    pub fusions: f32,
    pub branches: f32,
    pub cn_ratio: f32,
    pub biomass: f32,
    pub internal_c: f32,
    pub enzyme: f32,
    pub organic: f32,
    pub selected_id: f32,
    pub selected_lineage: f32,
    pub selected_age: f32,
    pub selected_reserve: f32,
    pub param_slot: f32,
    pub param_value: f32,
    pub soluble_c: f32,
    pub soluble_n: f32,
    pub slice_thickness: f32,
    pub slice_zoom: f32,
    pub slice_ox: f32,
    pub slice_oy: f32,
}

/// Orthogonal section through the pedon. Depth is the plane; thickness is
/// how many voxels that section integrates; zoom/pan is the XY field.
#[derive(Clone, Copy, Debug)]
pub struct SliceView {
    pub z: f32,
    pub thickness: f32,
    pub zoom: f32,
    pub ox: f32,
    pub oy: f32,
}

impl SliceView {
    pub fn new(depth: u32) -> Self {
        Self {
            z: depth as f32 * 0.35,
            thickness: 1.0,
            zoom: 1.0,
            ox: 0.0,
            oy: 0.0,
        }
    }

    pub fn nudge_depth(&mut self, delta: f32, max_z: f32) {
        self.z = (self.z + delta).clamp(1.0, (max_z - 2.0).max(1.0));
    }

    pub fn nudge_thickness(&mut self, delta: f32, max_z: f32) {
        self.thickness = (self.thickness + delta).clamp(1.0, max_z.max(1.0));
    }

    pub fn nudge_zoom(&mut self, delta: f32) {
        self.zoom = (self.zoom + delta).clamp(0.12, 1.0);
        self.clamp_pan();
    }

    pub fn nudge_pan(&mut self, dx: f32, dy: f32) {
        self.ox += dx;
        self.oy += dy;
        self.clamp_pan();
    }

    fn clamp_pan(&mut self) {
        let max_o = (1.0 - self.zoom).max(0.0);
        self.ox = self.ox.clamp(0.0, max_o);
        self.oy = self.oy.clamp(0.0, max_o);
    }
}

pub const TEL_COUNT: usize = 16;
