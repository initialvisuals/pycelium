//! CPU-first 2D mycelium / growth simulation.
//!
//! The core is a pair of dense grids (occupancy + nutrient) plus a list of
//! active tips. UI, ASCII, and file export live outside this crate so the
//! stepper can later sit behind a GPU or threaded backend without rewriting
//! the experiment surface.

mod config;
mod experiment;
mod grid;
mod metrics;
mod world;

pub use config::{SimConfig, SimError};
pub use experiment::ExperimentReport;
pub use grid::{Cell, Grid};
pub use metrics::Snapshot;
pub use world::{Tip, World};

#[cfg(test)]
mod tests;

/// Eight-neighborhood offsets (N, NE, E, SE, S, SW, W, NW).
pub const DIRS: [(i32, i32); 8] = [
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
];
