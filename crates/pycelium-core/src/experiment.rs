use std::time::Instant;

use crate::{SimConfig, Snapshot, World};

/// Result of running a fixed number of steps. The harness prints this;
/// later notebooks / CSV writers can consume the same struct.
#[derive(Clone, Debug)]
pub struct ExperimentReport {
    pub config: SimConfig,
    pub steps: u32,
    pub elapsed_secs: f64,
    pub snapshot: Snapshot,
    pub steps_per_sec: f64,
    /// `width * height * steps / elapsed` — useful when soaking large grids.
    pub cells_per_sec: f64,
}

impl World {
    pub fn run_experiment(&mut self, steps: u32) -> ExperimentReport {
        let started = Instant::now();
        for _ in 0..steps {
            self.step();
        }
        let elapsed = started.elapsed();
        let elapsed_secs = elapsed.as_secs_f64();
        let snapshot = self.snapshot();
        let steps_per_sec = if elapsed_secs > 0.0 {
            steps as f64 / elapsed_secs
        } else {
            f64::INFINITY
        };
        let cells_touched = (snapshot.grid_cells as f64) * (steps as f64);
        let cells_per_sec = if elapsed_secs > 0.0 {
            cells_touched / elapsed_secs
        } else {
            f64::INFINITY
        };

        ExperimentReport {
            config: self.config().clone(),
            steps,
            elapsed_secs,
            snapshot,
            steps_per_sec,
            cells_per_sec,
        }
    }
}

impl ExperimentReport {
    pub fn render_text(&self) -> String {
        let snap = &self.snapshot;
        format!(
            "=== pycelium bench ===\n\
             grid:           {w} x {h} ({cells} cells)\n\
             memory:         {mem:.2} MiB occupancy+nutrient\n\
             seed:           {seed}\n\
             steps:          {steps}\n\
             elapsed:        {elapsed:.3} ms\n\
             steps/sec:      {sps:.1}\n\
             cells/sec:      {cps:.3e}\n\
             active_tips:    {tips}\n\
             hypha_cells:    {hypha}\n\
             occupied:       {occ}\n\
             nutrient_sum:   {nut}\n\
             occupancy:      {pct:.4}%\n",
            w = snap.width,
            h = snap.height,
            cells = snap.grid_cells,
            mem = snap.memory_bytes as f64 / (1024.0 * 1024.0),
            seed = self.config.seed,
            steps = self.steps,
            elapsed = self.elapsed_secs * 1000.0,
            sps = self.steps_per_sec,
            cps = self.cells_per_sec,
            tips = snap.active_tips,
            hypha = snap.hypha_cells,
            occ = snap.occupied_cells,
            nut = snap.nutrient_sum,
            pct = snap.occupancy_percent(),
        )
    }

    pub fn render_json(&self) -> String {
        let snap = &self.snapshot;
        format!(
            "{{\n\
             \t\"width\": {w},\n\
             \t\"height\": {h},\n\
             \t\"grid_cells\": {cells},\n\
             \t\"memory_bytes\": {mem},\n\
             \t\"seed\": {seed},\n\
             \t\"steps\": {steps},\n\
             \t\"elapsed_secs\": {elapsed:.9},\n\
             \t\"steps_per_sec\": {sps:.6},\n\
             \t\"cells_per_sec\": {cps:.6},\n\
             \t\"active_tips\": {tips},\n\
             \t\"hypha_cells\": {hypha},\n\
             \t\"occupied_cells\": {occ},\n\
             \t\"nutrient_sum\": {nut},\n\
             \t\"occupancy_ratio\": {ratio:.8}\n\
             }}",
            w = snap.width,
            h = snap.height,
            cells = snap.grid_cells,
            mem = snap.memory_bytes,
            seed = self.config.seed,
            steps = self.steps,
            elapsed = self.elapsed_secs,
            sps = self.steps_per_sec,
            cps = self.cells_per_sec,
            tips = snap.active_tips,
            hypha = snap.hypha_cells,
            occ = snap.occupied_cells,
            nut = snap.nutrient_sum,
            ratio = snap.occupancy_ratio,
        )
    }
}
