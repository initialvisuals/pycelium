use anyhow::{Context, Result};
use rand::Rng;
use rayon::prelude::*;

use crate::config::MemoryPlan;
use crate::model::{GERM_TUBES_PER_SPORE, INOCULUM_SITES};
use crate::types::{SoilVoxel, Tip};

pub struct HostWorld {
    pub soil: Vec<SoilVoxel>,
    pub spores: Vec<Tip>,
    pub soil_width: u32,
    pub soil_height: u32,
    pub soil_layers: u32,
    pub brick_x: u32,
    pub brick_y: u32,
    pub brick_z: u32,
    pub foods: Vec<ResourceBody>,
}

#[derive(Clone, Copy, Debug)]
pub struct ResourceBody {
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub rx: f32,
    pub ry: f32,
    pub rz: f32,
    pub carbon: f32,
    pub nitrogen: f32,
}

pub struct BrickUpload {
    pub organic: Vec<f32>,
    pub soluble_c: Vec<f32>,
    pub soluble_n: Vec<f32>,
    pub moisture: Vec<f32>,
}

impl HostWorld {
    pub fn commit(plan: &MemoryPlan) -> Result<Self> {
        println!("committing 3D pedon...");
        let soil_len =
            plan.soil_width as usize * plan.soil_height as usize * plan.soil_layers as usize;
        let w = plan.soil_width;
        let h = plan.soil_height;
        let d = plan.soil_layers;
        let mut soil = try_commit_vec(soil_len, |i| {
            let x = (i % w as usize) as u32;
            let y = ((i / w as usize) % h as usize) as u32;
            let z = (i / (w as usize * h as usize)) as u32;
            let zn = z as f32 / (d.max(2) - 1) as f32;
            let n0 = hash01(x.wrapping_mul(374761393) ^ y.wrapping_mul(668265263) ^ z);
            let n1 = hash01(x / 6 ^ (y / 6).wrapping_mul(17) ^ z.wrapping_mul(9));
            SoilVoxel {
                organic: (0.10 * (-(z as f32) / 9.0).exp() * (0.35 + 0.85 * n1)).max(0.004),
                soluble_c: 0.008 + 0.02 * n0 * (1.0 - zn),
                soluble_n: 0.004 + 0.01 * hash01(x.wrapping_add(y << 3) ^ 0x51ed),
                moisture: (0.20 + 0.68 * zn + 0.08 * n0).clamp(0.08, 0.96),
            }
        })
        .context("pedon reservation failed — try a lower --ram-gb")?;

        let mut rng = rand::thread_rng();
        let mut foods = Vec::with_capacity(plan.food_count as usize);
        for k in 0..plan.food_count {
            let wood = k % 3 != 1;
            foods.push(ResourceBody {
                x: rng.gen_range(0..plan.soil_width),
                y: rng.gen_range(0..plan.soil_height),
                z: if wood {
                    rng.gen_range(0..(plan.soil_layers / 3).max(1))
                } else {
                    rng.gen_range(0..plan.soil_layers)
                },
                rx: rng.gen_range(18.0..70.0),
                ry: rng.gen_range(12.0..40.0),
                rz: if wood {
                    rng.gen_range(6.0..14.0)
                } else {
                    rng.gen_range(10.0..28.0)
                },
                carbon: if wood {
                    rng.gen_range(1.6..2.8)
                } else {
                    rng.gen_range(0.4..0.9)
                },
                nitrogen: if wood {
                    rng.gen_range(0.08..0.22)
                } else {
                    rng.gen_range(0.8..1.8)
                },
            });
        }
        for food in &foods {
            stamp_body(&mut soil, w, h, d, *food);
        }

        let gw = plan.gpu_width as f32;
        let gh = plan.gpu_height as f32;
        let gd = plan.gpu_depth as f32;
        let live = (INOCULUM_SITES * GERM_TUBES_PER_SPORE) as usize;
        let spores = try_commit_vec(plan.host_spores, |i| {
            if i >= live {
                return Tip::DORMANT;
            }
            let site = (i as u32) % INOCULUM_SITES;
            let cx = gw * (0.22 + 0.14 * site as f32);
            let cy = gh * (0.28 + 0.12 * ((site + 2) as f32 * 0.31).fract());
            let cz = gd * 0.22;
            let t = (i as f32) * 0.41;
            let p = t.sin();
            let q = (t * 1.3).cos();
            let r = (t * 0.7).sin();
            let mut dir = [p, q, r * 0.45];
            let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt().max(1e-5);
            dir[0] /= len;
            dir[1] /= len;
            dir[2] /= len;
            Tip {
                pos: [cx + dir[0] * 3.0, cy + dir[1] * 3.0, cz + dir[2] * 2.0],
                age: 0.0,
                dir,
                reserve: 1.1,
                state: 1.0,
                lineage: site + 1,
                parent: 0,
                flags: 0,
            }
        })
        .context("spore bank reservation failed — try a lower --ram-gb")?;

        println!(
            "host committed: {:.2} GiB pedon + {:.2} GiB spores | {} germ tubes",
            (soil_len * std::mem::size_of::<SoilVoxel>()) as f64 / 1024.0 / 1024.0 / 1024.0,
            (plan.host_spores * std::mem::size_of::<Tip>()) as f64 / 1024.0 / 1024.0 / 1024.0,
            live
        );

        Ok(Self {
            soil,
            spores,
            soil_width: w,
            soil_height: h,
            soil_layers: d,
            brick_x: w.saturating_sub(plan.gpu_width) / 2,
            brick_y: h.saturating_sub(plan.gpu_height) / 2,
            brick_z: 0,
            foods,
        })
    }

    pub fn committed_bytes(&self) -> usize {
        self.soil.len() * std::mem::size_of::<SoilVoxel>()
            + self.spores.len() * std::mem::size_of::<Tip>()
    }

    pub fn extract_brick(&self, pw: u32, ph: u32, pd: u32) -> BrickUpload {
        let mut brick = BrickUpload {
            organic: vec![0.0; (pw * ph * pd) as usize],
            soluble_c: vec![0.0; (pw * ph * pd) as usize],
            soluble_n: vec![0.0; (pw * ph * pd) as usize],
            moisture: vec![0.0; (pw * ph * pd) as usize],
        };
        let ox = self.brick_x;
        let oy = self.brick_y;
        let oz = self.brick_z;
        let sw = self.soil_width;
        let sh = self.soil_height;
        let sd = self.soil_layers;
        let soil = self.soil.as_slice();
        brick
            .organic
            .par_chunks_mut(pw as usize)
            .zip(brick.soluble_c.par_chunks_mut(pw as usize))
            .zip(brick.soluble_n.par_chunks_mut(pw as usize))
            .zip(brick.moisture.par_chunks_mut(pw as usize))
            .enumerate()
            .for_each(|(row, (((org, sc), sn), m))| {
                let layer = row as u32 / ph;
                let y = row as u32 % ph;
                let z = (oz + layer) % sd;
                let yy = (oy + y) % sh;
                for x in 0..pw {
                    let xx = (ox + x) % sw;
                    let idx = (z as usize * sh as usize + yy as usize) * sw as usize + xx as usize;
                    let v = soil[idx];
                    org[x as usize] = v.organic;
                    sc[x as usize] = v.soluble_c;
                    sn[x as usize] = v.soluble_n;
                    m[x as usize] = v.moisture;
                }
            });
        brick
    }

    pub fn drift(&mut self, gpu_w: u32, gpu_h: u32, _gpu_d: u32) {
        let max_x = self.soil_width.saturating_sub(gpu_w);
        let max_y = self.soil_height.saturating_sub(gpu_h);
        if max_x == 0 && max_y == 0 {
            return;
        }
        self.brick_x = if max_x == 0 {
            0
        } else {
            (self.brick_x + 1) % (max_x + 1)
        };
        if self.brick_x == 0 && max_y > 0 {
            self.brick_y = (self.brick_y + 1) % (max_y + 1);
        }
    }

    pub fn spawn_food(&mut self) {
        let mut rng = rand::thread_rng();
        let food = ResourceBody {
            x: self.brick_x + rng.gen_range(0..self.soil_width.min(64) + 8),
            y: self.brick_y + rng.gen_range(0..self.soil_height.min(64) + 8),
            z: rng.gen_range(0..(self.soil_layers / 2).max(1)),
            rx: rng.gen_range(16.0..40.0),
            ry: rng.gen_range(12.0..28.0),
            rz: rng.gen_range(8.0..18.0),
            carbon: 1.8,
            nitrogen: 0.35,
        };
        stamp_body(
            &mut self.soil,
            self.soil_width,
            self.soil_height,
            self.soil_layers,
            food,
        );
        self.foods.push(food);
    }
}

fn stamp_body(soil: &mut [SoilVoxel], w: u32, h: u32, d: u32, body: ResourceBody) {
    let rx = body.rx.ceil() as i32;
    let ry = body.ry.ceil() as i32;
    let rz = body.rz.ceil() as i32;
    for dz in -rz..=rz {
        for dy in -ry..=ry {
            for dx in -rx..=rx {
                let nx = dx as f32 / body.rx.max(1.0);
                let ny = dy as f32 / body.ry.max(1.0);
                let nz = dz as f32 / body.rz.max(1.0);
                let q = nx * nx + ny * ny + nz * nz;
                if q > 1.0 {
                    continue;
                }
                let falloff = (1.0 - q).powf(1.4);
                let x = wrap_signed(body.x as i32 + dx, w as i32) as u32;
                let y = wrap_signed(body.y as i32 + dy, h as i32) as u32;
                let z = (body.z as i32 + dz).clamp(0, d as i32 - 1) as u32;
                let idx = (z as usize * h as usize + y as usize) * w as usize + x as usize;
                soil[idx].organic += body.carbon * falloff;
                soil[idx].soluble_c += body.carbon * 0.08 * falloff;
                soil[idx].soluble_n += body.nitrogen * falloff;
            }
        }
    }
}

fn wrap_signed(v: i32, m: i32) -> i32 {
    ((v % m) + m) % m
}

fn hash01(mut x: u32) -> f32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^= x >> 16;
    (x as f32) * (1.0 / u32::MAX as f32)
}

fn try_commit_vec<T, F>(len: usize, fill: F) -> Result<Vec<T>>
where
    T: Copy + Send,
    F: Fn(usize) -> T + Sync,
{
    let mut v = Vec::<T>::new();
    v.try_reserve_exact(len)
        .with_context(|| format!("try_reserve_exact({len}) failed"))?;
    unsafe {
        v.set_len(len);
    }
    v.par_iter_mut()
        .enumerate()
        .for_each(|(i, slot)| *slot = fill(i));
    Ok(v)
}
