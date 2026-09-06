# Pycelium

A **game-like simulator / idea lab** with a data-science mindset. Not a game. The goal is to push dense mycelium-style growth simulations on target PC hardware — GPU utilization, RAM utilization, threading, and slice views — then iterate on the numbers.

Gamification can come later. This repo is **independent of Mycelium Engine**; do not integrate Engine code here.

Part of the [Initial Visuals](https://github.com/initialvisuals) toolkit.

## Hardware target

| Resource | Target |
|----------|--------|
| GPU | RTX 3060 12GB VRAM |
| System RAM | 64GB |

v1 is **CPU-first**: a dense 2D grid that can be scaled up to soak RAM, with a harness that reports occupancy and throughput. GPU compute is a later step, not this scaffold.

## Lanes

| Lane | Status | Notes |
|------|--------|-------|
| **PC-2D** | Primary | Dense grid sim, experiment harness, ASCII / PGM slices |
| **PC-3D** | Path | Same core ideas, volumetric / stacked slices later |
| **Termux-2D** | Secondary | Keep the Python toy runnable; not the focus tonight |

## Quick start (PC sim lab)

Requires [Rust](https://rustup.rs/) (stable, edition 2021).

```bash
# compile + unit tests
cargo test

# measurable run (default 512², 200 steps)
cargo run -p pycelium-lab --release -- bench

# ASCII slice (hypha #, tip @, nutrient .)
cargo run -p pycelium-lab --release -- view --width 256 --height 128 --steps 80 --seed 11

# soak RAM: two bytes/cell (occupancy + nutrient). 8192² ≈ 128 MiB; 32768² ≈ 2 GiB
cargo run -p pycelium-lab --release -- bench --width 4096 --height 4096 --steps 50 --seed 1

# machine-readable metrics + occupancy image
cargo run -p pycelium-lab --release -- bench --json --export-pgm occupancy.pgm
```

The harness prints active tip count, hypha cells, occupied cells, nutrient sum, occupancy %, elapsed time, steps/sec, and cells/sec (`width * height * steps / time`). Bump `--width` / `--height` when you want the grids to dominate RAM.

ASCII legend: `.` nutrient, `#` hypha, `@` tip. PGM is occupancy (black empty, grey hypha, white tip) and opens in any image viewer.

## Legacy Python prototype

The original CLI toy is unchanged under [`legacy/`](legacy/README.md). From the repo root:

```bash
python pycelium.py
# or
python legacy/pycelium.py
```

Windows-oriented clear (`cls`); on other platforms you may want `clear`.

## Layout

| Path | What |
|------|------|
| `crates/pycelium-core` | Sim only: grids, tips, stepper, metrics |
| `crates/pycelium-lab` | CLI experiment harness + 2D views |
| `legacy/pycelium.py` | Original Python mycelium toy |
| `pycelium.py` | Launcher for that toy |
| `CONTRIBUTING.md` | How to work here; credits policy |
| `README.rst` | Original short readme |

## License

See [LICENSE](LICENSE).

---

**Initial Visuals** — tools, sims, and experiments.
