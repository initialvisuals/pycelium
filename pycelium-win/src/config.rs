use clap::{Parser, ValueEnum};

use crate::types::{SoilVoxel, Tip};

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Preset {
    /// Small pedon, fast startup.
    Demo,
    /// Default for an RTX-class card.
    Performant,
    /// Push host RAM toward the machine limit.
    Beast,
}

#[derive(Parser, Debug)]
#[command(
    name = "pycelium-win",
    about = "Windows GPU mycelium: 3D mesocosm, exoenzymes, translocation, telemetry HUD"
)]
pub struct Args {
    #[arg(long, value_enum, default_value_t = Preset::Performant)]
    pub preset: Preset,

    /// Host memory to commit, in GiB.
    #[arg(long)]
    pub ram_gb: Option<f32>,

    /// GPU brick XY edge (depth is derived unless --depth is set).
    #[arg(long)]
    pub resolution: Option<u32>,

    /// GPU brick depth in voxels.
    #[arg(long)]
    pub depth: Option<u32>,

    /// Hyphal tip slots on the GPU (most start dormant as a branch pool).
    #[arg(long)]
    pub agents: Option<u32>,

    /// Resource bodies stamped into the pedon.
    #[arg(long, default_value_t = 28)]
    pub food: u32,

    #[arg(long, default_value_t = 1)]
    pub steps: u32,

    #[arg(long, default_value_t = false)]
    pub vsync: bool,

    /// Slide the GPU brick through the host pedon.
    #[arg(long, default_value_t = false)]
    pub drift: bool,

    #[arg(long)]
    pub bench: Option<u32>,

    /// English HUD label density. Tab cycles rich → sparse → off at runtime.
    #[arg(long, value_enum, default_value_t = LabelDensity::Rich)]
    pub labels: LabelDensity,

    /// Directory for slice-export files (created on first capture).
    #[arg(long, default_value = "exports")]
    pub export_dir: String,
}

/// How loudly the left telemetry HUD speaks English.
/// Cryptic 3×5 glyphs stay visible in every mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum LabelDensity {
    /// All meter names at low opacity; hover / pick fades them up.
    #[default]
    Rich,
    /// Names only on hover, pick, or the focused param row.
    Sparse,
    /// Glyphs and bars only.
    Off,
}

impl LabelDensity {
    pub fn cycle(self) -> Self {
        match self {
            Self::Rich => Self::Sparse,
            Self::Sparse => Self::Off,
            Self::Off => Self::Rich,
        }
    }

    pub fn as_f32(self) -> f32 {
        match self {
            Self::Off => 0.0,
            Self::Sparse => 1.0,
            Self::Rich => 2.0,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Rich => "rich",
            Self::Sparse => "sparse",
            Self::Off => "off",
        }
    }
}

#[derive(Clone, Debug)]
pub struct MemoryPlan {
    pub gpu_width: u32,
    pub gpu_height: u32,
    pub gpu_depth: u32,
    pub gpu_agents: u32,
    pub host_spores: usize,
    pub soil_width: u32,
    pub soil_height: u32,
    pub soil_layers: u32,
    pub food_count: u32,
    pub target_bytes: usize,
    pub estimated_host_bytes: usize,
    pub steps_per_frame: u32,
    pub vsync: bool,
    pub drift: bool,
}

impl MemoryPlan {
    pub fn from_args(args: &Args) -> Self {
        let (default_ram_gb, default_xy, default_depth, default_agents, default_soil_z, default_drift) =
            match args.preset {
                Preset::Demo => (0.85_f32, 96_u32, 64_u32, 48_000_u32, 96_u32, false),
                Preset::Performant => (4.0, 192, 96, 400_000, 192, false),
                Preset::Beast => (40.0, 256, 160, 1_500_000, 384, true),
            };

        let ram_gb = args.ram_gb.unwrap_or(default_ram_gb).max(0.25);
        let gpu_xy = args.resolution.unwrap_or(default_xy).clamp(64, 512);
        let gpu_depth = args.depth.unwrap_or(default_depth).clamp(32, 384);
        let gpu_agents = args.agents.unwrap_or(default_agents).clamp(1_024, 8_000_000);
        let drift = args.drift || default_drift;

        let target_bytes = (ram_gb as f64 * 1024.0 * 1024.0 * 1024.0) as usize;
        let staging = gpu_cells(gpu_xy, gpu_xy, gpu_depth) * std::mem::size_of::<SoilVoxel>();
        let spore_budget = match args.preset {
            Preset::Demo => (gpu_agents as usize).saturating_mul(2),
            Preset::Performant => (gpu_agents as usize).saturating_mul(3),
            Preset::Beast => (gpu_agents as usize).saturating_mul(6).max(16_000_000),
        };
        let spore_bytes = spore_budget.saturating_mul(std::mem::size_of::<Tip>());
        let soil_budget = target_bytes.saturating_sub(staging.saturating_add(spore_bytes));

        let mut soil_z = default_soil_z.max(gpu_depth);
        let voxel = std::mem::size_of::<SoilVoxel>();
        let mut side = ((soil_budget / voxel.saturating_mul(soil_z as usize)) as f64)
            .sqrt() as u32;
        side = (side / 32) * 32;
        side = side.max(gpu_xy);

        let mut soil_width = side;
        let mut soil_height = side;
        let mut bytes = soil_bytes(soil_width, soil_height, soil_z) + spore_bytes + staging;
        while bytes > target_bytes && (soil_width > gpu_xy || soil_height > gpu_xy || soil_z > gpu_depth)
        {
            if soil_width > gpu_xy {
                soil_width = (soil_width - 32).max(gpu_xy);
            }
            if soil_height > gpu_xy {
                soil_height = (soil_height - 32).max(gpu_xy);
            }
            if soil_z > gpu_depth {
                soil_z = (soil_z - 8).max(gpu_depth);
            }
            bytes = soil_bytes(soil_width, soil_height, soil_z) + spore_bytes + staging;
        }

        Self {
            gpu_width: gpu_xy,
            gpu_height: gpu_xy,
            gpu_depth,
            gpu_agents,
            host_spores: spore_budget,
            soil_width,
            soil_height,
            soil_layers: soil_z,
            food_count: args.food.max(1),
            target_bytes,
            estimated_host_bytes: bytes,
            steps_per_frame: args.steps.max(1),
            vsync: args.vsync,
            drift,
        }
    }

    pub fn gpu_cells(&self) -> usize {
        gpu_cells(self.gpu_width, self.gpu_height, self.gpu_depth)
    }

    pub fn describe(&self) -> String {
        format!(
            "GPU brick {}x{}x{} | {} tip slots | pedon {}x{}x{} | spores {} | host ~{:.2} GiB (target {:.2} GiB)",
            self.gpu_width,
            self.gpu_height,
            self.gpu_depth,
            self.gpu_agents,
            self.soil_width,
            self.soil_height,
            self.soil_layers,
            self.host_spores,
            self.estimated_host_bytes as f64 / 1024.0 / 1024.0 / 1024.0,
            self.target_bytes as f64 / 1024.0 / 1024.0 / 1024.0,
        )
    }
}

fn gpu_cells(w: u32, h: u32, d: u32) -> usize {
    w as usize * h as usize * d as usize
}

fn soil_bytes(width: u32, height: u32, layers: u32) -> usize {
    width as usize * height as usize * layers as usize * std::mem::size_of::<SoilVoxel>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn beast_plan_is_large() {
        let args = Args {
            preset: Preset::Beast,
            ram_gb: Some(8.0),
            resolution: Some(96),
            depth: Some(64),
            agents: Some(4096),
            food: 8,
            steps: 1,
            vsync: false,
            drift: false,
            bench: None,
            labels: LabelDensity::Rich,
            export_dir: "exports".into(),
        };
        let plan = MemoryPlan::from_args(&args);
        assert!(plan.estimated_host_bytes > 1024 * 1024);
        assert!(plan.soil_width >= plan.gpu_width);
        assert!(plan.gpu_depth >= 32);
        assert_eq!(args.labels, LabelDensity::Rich);
    }

    #[test]
    fn labels_flag_does_not_change_performant_plan() {
        let off = Args::try_parse_from(["pycelium-win", "--labels", "off"]).expect("parse");
        let def = Args::try_parse_from(["pycelium-win"]).expect("parse");
        let a = MemoryPlan::from_args(&off);
        let b = MemoryPlan::from_args(&def);
        assert_eq!(a.gpu_width, b.gpu_width);
        assert_eq!(a.gpu_agents, b.gpu_agents);
        assert_eq!(a.gpu_depth, b.gpu_depth);
        assert_eq!(off.labels, LabelDensity::Off);
        assert_eq!(def.labels, LabelDensity::Rich);
        assert!(matches!(def.preset, Preset::Performant));
        assert_eq!(def.export_dir, "exports");
    }

    #[test]
    fn export_dir_flag_does_not_change_plan() {
        let custom =
            Args::try_parse_from(["pycelium-win", "--export-dir", "out/slices"]).expect("parse");
        let def = Args::try_parse_from(["pycelium-win"]).expect("parse");
        let a = MemoryPlan::from_args(&custom);
        let b = MemoryPlan::from_args(&def);
        assert_eq!(a.gpu_width, b.gpu_width);
        assert_eq!(a.gpu_agents, b.gpu_agents);
        assert_eq!(custom.export_dir, "out/slices");
    }

    #[test]
    fn label_density_cycles_and_keeps_presets_untouched() {
        assert_eq!(LabelDensity::Rich.cycle(), LabelDensity::Sparse);
        assert_eq!(LabelDensity::Sparse.cycle(), LabelDensity::Off);
        assert_eq!(LabelDensity::Off.cycle(), LabelDensity::Rich);
        assert_eq!(LabelDensity::Off.as_f32(), 0.0);
        assert_eq!(LabelDensity::Sparse.as_f32(), 1.0);
        assert_eq!(LabelDensity::Rich.as_f32(), 2.0);
        let beast = Args {
            preset: Preset::Beast,
            ram_gb: None,
            resolution: None,
            depth: None,
            agents: None,
            food: 28,
            steps: 1,
            vsync: false,
            drift: false,
            bench: None,
            labels: LabelDensity::Off,
            export_dir: "exports".into(),
        };
        let plan = MemoryPlan::from_args(&beast);
        assert!(plan.gpu_agents >= 1_000);
        assert_eq!(beast.labels, LabelDensity::Off);
    }
}
