use crate::{SimConfig, SimError, World};

fn tiny(seed: u64) -> SimConfig {
    SimConfig {
        width: 48,
        height: 36,
        seed,
        initial_tips: 3,
        nutrient_patches: 10,
        patch_radius: 4,
        branch_energy: 6,
        consume: 20,
        max_tips: 2_000,
    }
}

#[test]
fn rejects_empty_grid() {
    let mut cfg = SimConfig::default();
    cfg.width = 0;
    assert_eq!(World::new(cfg).unwrap_err(), SimError::EmptyGrid);
}

#[test]
fn memory_bytes_are_two_grids() {
    let cfg = SimConfig {
        width: 128,
        height: 64,
        ..SimConfig::default()
    };
    assert_eq!(cfg.memory_bytes(), 128 * 64 * 2);
    assert_eq!(cfg.grid_cells(), 128 * 64);
}

#[test]
fn seed_places_inoculum_and_nutrient() {
    let world = World::new(tiny(7)).unwrap();
    let snap = world.snapshot();
    assert_eq!(snap.active_tips, 3);
    assert_eq!(snap.occupied_cells, 3);
    assert_eq!(snap.hypha_cells, 0);
    assert!(snap.nutrient_sum > 0);
    assert_eq!(world.occupancy().width(), 48);
    assert_eq!(world.occupancy().height(), 36);
}

#[test]
fn same_seed_is_deterministic() {
    let mut a = World::new(tiny(99)).unwrap();
    let mut b = World::new(tiny(99)).unwrap();
    a.run_experiment(40);
    b.run_experiment(40);
    assert_eq!(a.occupancy().as_slice(), b.occupancy().as_slice());
    assert_eq!(a.nutrient().as_slice(), b.nutrient().as_slice());
    assert_eq!(a.snapshot(), b.snapshot());
}

#[test]
fn different_seeds_diverge() {
    let mut a = World::new(tiny(1)).unwrap();
    let mut b = World::new(tiny(2)).unwrap();
    a.run_experiment(40);
    b.run_experiment(40);
    assert_ne!(a.occupancy().as_slice(), b.occupancy().as_slice());
}

#[test]
fn growth_increases_occupancy_and_keeps_counts_consistent() {
    let mut world = World::new(tiny(3)).unwrap();
    let before = world.snapshot();
    world.run_experiment(30);
    let after = world.snapshot();
    assert!(after.occupied_cells > before.occupied_cells);
    assert!(after.hypha_cells > 0);
    assert_eq!(
        after.occupied_cells,
        after.hypha_cells + after.active_tips as u64
    );
    assert!(after.nutrient_sum <= before.nutrient_sum);
    assert_eq!(after.step, 30);
}

#[test]
fn experiment_report_has_throughput_fields() {
    let mut world = World::new(tiny(5)).unwrap();
    let report = world.run_experiment(10);
    assert_eq!(report.steps, 10);
    assert!(report.elapsed_secs >= 0.0);
    assert!(report.steps_per_sec > 0.0);
    assert!(report.cells_per_sec > 0.0);
    let text = report.render_text();
    assert!(text.contains("active_tips"));
    assert!(text.contains("cells/sec"));
    let json = report.render_json();
    assert!(json.contains("\"hypha_cells\""));
}
