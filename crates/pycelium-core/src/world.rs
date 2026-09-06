use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::{Cell, Grid, SimConfig, SimError, Snapshot, DIRS};

const HYPHA: Cell = 1;
const TIP: Cell = 255;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tip {
    pub x: u32,
    pub y: u32,
    pub energy: u16,
    pub heading: u8,
}

#[derive(Clone, Debug)]
pub struct World {
    config: SimConfig,
    occupancy: Grid,
    nutrient: Grid,
    tips: Vec<Tip>,
    rng: StdRng,
    step: u64,
    hypha_cells: u64,
    occupied_cells: u64,
    nutrient_sum: u64,
}

impl World {
    pub fn new(config: SimConfig) -> Result<Self, SimError> {
        config.validate()?;
        let occupancy = Grid::new(config.width, config.height);
        let nutrient = Grid::new(config.width, config.height);
        let rng = StdRng::seed_from_u64(config.seed);
        let mut world = Self {
            config,
            occupancy,
            nutrient,
            tips: Vec::new(),
            rng,
            step: 0,
            hypha_cells: 0,
            occupied_cells: 0,
            nutrient_sum: 0,
        };
        world.seed_nutrient();
        world.seed_inoculum();
        Ok(world)
    }

    pub fn config(&self) -> &SimConfig {
        &self.config
    }

    pub fn step_index(&self) -> u64 {
        self.step
    }

    pub fn occupancy(&self) -> &Grid {
        &self.occupancy
    }

    pub fn nutrient(&self) -> &Grid {
        &self.nutrient
    }

    pub fn tips(&self) -> &[Tip] {
        &self.tips
    }

    pub fn snapshot(&self) -> Snapshot {
        let grid_cells = self.config.grid_cells();
        let occupied = self.occupied_cells;
        Snapshot {
            step: self.step,
            width: self.config.width,
            height: self.config.height,
            grid_cells,
            memory_bytes: self.config.memory_bytes(),
            active_tips: self.tips.len() as u32,
            hypha_cells: self.hypha_cells,
            occupied_cells: occupied,
            nutrient_sum: self.nutrient_sum,
            occupancy_ratio: if grid_cells == 0 {
                0.0
            } else {
                occupied as f64 / grid_cells as f64
            },
        }
    }

    pub fn step(&mut self) {
        self.step += 1;
        let mut next_tips = Vec::with_capacity(self.tips.len());
        let current = std::mem::take(&mut self.tips);

        for tip in current {
            if let Some((moved, maybe_branch)) = self.advance_tip(tip) {
                next_tips.push(moved);
                if let Some(branch) = maybe_branch {
                    if (next_tips.len() as u32) < self.config.max_tips {
                        next_tips.push(branch);
                    }
                }
            }
        }

        self.tips = next_tips;
    }

    fn seed_nutrient(&mut self) {
        let patches = self.config.nutrient_patches.max(1);
        let radius = self.config.patch_radius.max(1) as i32;
        let w = self.config.width as i32;
        let h = self.config.height as i32;

        for _ in 0..patches {
            let cx = self.rng.gen_range(0..self.config.width) as i32;
            let cy = self.rng.gen_range(0..self.config.height) as i32;
            let strength = self.rng.gen_range(96..=220);
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    if dx * dx + dy * dy > radius * radius {
                        continue;
                    }
                    let x = cx + dx;
                    let y = cy + dy;
                    if x < 0 || y < 0 || x >= w || y >= h {
                        continue;
                    }
                    let ux = x as u32;
                    let uy = y as u32;
                    let old = self.nutrient.get(ux, uy);
                    let new = old.saturating_add(strength / 2).max(strength);
                    self.nutrient_sum += (new as u64).saturating_sub(old as u64);
                    self.nutrient.set(ux, uy, new);
                }
            }
        }
    }

    fn seed_inoculum(&mut self) {
        let count = self.config.initial_tips.max(1);
        let cx = self.config.width / 2;
        let cy = self.config.height / 2;

        for i in 0..count {
            let (x, y) = if i == 0 {
                (cx, cy)
            } else {
                (
                    self.rng.gen_range(0..self.config.width),
                    self.rng.gen_range(0..self.config.height),
                )
            };
            self.place_tip(x, y, (i % 8) as u8);
        }
    }

    fn place_tip(&mut self, x: u32, y: u32, heading: u8) {
        if (self.tips.len() as u32) >= self.config.max_tips {
            return;
        }
        if self.occupancy.get(x, y) != 0 {
            return;
        }
        self.consume_at(x, y);
        self.occupancy.set(x, y, TIP);
        self.occupied_cells += 1;
        self.tips.push(Tip {
            x,
            y,
            energy: 0,
            heading,
        });
    }

    fn consume_at(&mut self, x: u32, y: u32) -> u8 {
        let n = self.nutrient.get(x, y);
        if n == 0 {
            return 0;
        }
        let take = n.min(self.config.consume);
        self.nutrient.set(x, y, n - take);
        self.nutrient_sum -= take as u64;
        take
    }

    fn advance_tip(&mut self, mut tip: Tip) -> Option<(Tip, Option<Tip>)> {
        let mut best: Option<(u32, u32, i32, u8)> = None;

        for (dir_i, &(dx, dy)) in DIRS.iter().enumerate() {
            let nx = tip.x as i32 + dx;
            let ny = tip.y as i32 + dy;
            if !self.occupancy.in_bounds(nx, ny) {
                continue;
            }
            let ux = nx as u32;
            let uy = ny as u32;
            if self.occupancy.get(ux, uy) != 0 {
                continue;
            }
            let food = self.nutrient.get(ux, uy) as i32;
            let persist = if dir_i as u8 == tip.heading { 18 } else { 0 };
            let jitter = self.rng.gen_range(0..8);
            let score = food * 4 + persist + jitter;
            match best {
                None => best = Some((ux, uy, score, dir_i as u8)),
                Some((_, _, best_score, _)) if score > best_score => {
                    best = Some((ux, uy, score, dir_i as u8));
                }
                _ => {}
            }
        }

        let (nx, ny, heading) = match best {
            Some((x, y, _, heading)) => (x, y, heading),
            None => {
                self.occupancy.set(tip.x, tip.y, HYPHA);
                self.hypha_cells += 1;
                return None;
            }
        };

        self.occupancy.set(tip.x, tip.y, HYPHA);
        self.hypha_cells += 1;

        let gained = self.consume_at(nx, ny);
        self.occupancy.set(nx, ny, TIP);
        self.occupied_cells += 1;

        tip.x = nx;
        tip.y = ny;
        tip.heading = heading;
        tip.energy = tip.energy.saturating_add(1).saturating_add(gained as u16 / 8);

        let branch = if tip.energy >= self.config.branch_energy {
            tip.energy = 0;
            self.try_branch(&tip)
        } else {
            None
        };

        Some((tip, branch))
    }

    fn try_branch(&mut self, parent: &Tip) -> Option<Tip> {
        let start = self.rng.gen_range(0..DIRS.len());
        for k in 0..DIRS.len() {
            let dir_i = (start + k) % DIRS.len();
            let (dx, dy) = DIRS[dir_i];
            let nx = parent.x as i32 + dx;
            let ny = parent.y as i32 + dy;
            if !self.occupancy.in_bounds(nx, ny) {
                continue;
            }
            let ux = nx as u32;
            let uy = ny as u32;
            if self.occupancy.get(ux, uy) != 0 {
                continue;
            }
            self.consume_at(ux, uy);
            self.occupancy.set(ux, uy, TIP);
            self.occupied_cells += 1;
            return Some(Tip {
                x: ux,
                y: uy,
                energy: 0,
                heading: dir_i as u8,
            });
        }
        None
    }
}
