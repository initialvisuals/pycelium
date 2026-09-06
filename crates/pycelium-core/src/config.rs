use std::fmt;

/// Tunable experiment parameters. Bump `width` / `height` to soak RAM.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimConfig {
    pub width: u32,
    pub height: u32,
    pub seed: u64,
    /// Inoculum sites (active growth fronts) at t=0.
    pub initial_tips: u32,
    /// Scattered nutrient patches.
    pub nutrient_patches: u32,
    pub patch_radius: u32,
    /// Energy accumulated before a tip may branch.
    pub branch_energy: u16,
    /// Nutrient units removed when a tip occupies a cell.
    pub consume: u8,
    /// Soft cap so a dense run cannot explode tip counts.
    pub max_tips: u32,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            width: 512,
            height: 512,
            seed: 1,
            initial_tips: 4,
            nutrient_patches: 48,
            patch_radius: 12,
            branch_energy: 8,
            consume: 24,
            max_tips: 50_000,
        }
    }
}

impl SimConfig {
    pub fn grid_cells(&self) -> usize {
        (self.width as usize).saturating_mul(self.height as usize)
    }

    /// Occupancy + nutrient backing stores (2 bytes / cell).
    pub fn memory_bytes(&self) -> usize {
        self.grid_cells().saturating_mul(2)
    }

    pub fn validate(&self) -> Result<(), SimError> {
        if self.width == 0 || self.height == 0 {
            return Err(SimError::EmptyGrid);
        }
        if self.grid_cells() == 0 {
            return Err(SimError::TooLarge);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SimError {
    EmptyGrid,
    TooLarge,
}

impl fmt::Display for SimError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SimError::EmptyGrid => write!(f, "grid width and height must be > 0"),
            SimError::TooLarge => write!(f, "grid dimensions overflow usize"),
        }
    }
}

impl std::error::Error for SimError {}
