/// Point-in-time counters for data-science iteration.
#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub step: u64,
    pub width: u32,
    pub height: u32,
    pub grid_cells: usize,
    pub memory_bytes: usize,
    pub active_tips: u32,
    pub hypha_cells: u64,
    pub occupied_cells: u64,
    pub nutrient_sum: u64,
    pub occupancy_ratio: f64,
}

impl Snapshot {
    pub fn occupancy_percent(&self) -> f64 {
        self.occupancy_ratio * 100.0
    }
}
