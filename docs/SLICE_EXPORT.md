# Slice export + cutter

Capture a face-aligned plane (or a thin box) from the GPU mesocosm and write **PNG + JSON + SVG** into `exports/` (override with `--export-dir`). No mesh or voxel conversion — density images, a mask, optional heightmap, and a small JSON record.

Orbit, tip pick, HUD glyphs/labels, and **Tab** label density are unchanged. Capture is a separate mode.

## Enter / leave

| Key | Action |
|-----|--------|
| `E` | Toggle capture mode |
| `C` | Cycle **2D squash** ↔ **rich box** (while capturing) |
| `Enter` | Write files (timestamped stem) |
| `Esc` | Leave capture (quit only when capture is off) |

Mouse move positions the cutter through the cube. Left-drag still orbits unless you grab a slider handle. Click-to-pick is disabled while capturing.

## Two capture modes

1. **2D squash** (default) — same idea as the orange view slab: stack **N** nearby layers and keep the max biomass / soluble C on one plane. Starts at **1 layer** at the cursor. Drag the two-ended slider handles outward to thicken the bake.
2. **Rich (3D box)** — two parallel planes forming a thin box you stretch (same slider). The PNG is still a density projection of that box; JSON also stores occupied volume samples `{u,v,t,d}` between the planes.

## Face snap (Blender-like)

The world is the unit cube. Hover near a **face center** (halfway) and the cutter axis snaps to that face’s normal:

- Top / bottom (`±Z`) → horizontal slice (XY)
- Side faces (`±X` / `±Y`) → **vertical** slice, so upright structures can be captured

Hover a **face mid-edge** to rotate **90° on the free axis** (top-face left/right mid-edge → vertical X; top-face front/back mid-edge → vertical Y). The orange preview plane/box and a wash on the snapped faces show the alignment.

`X` cycles axis by hand if snap misses. `Y` cycles the optional heightmap axis (and turns heightmap export on). `H` toggles writing the heightmap PNG.

## Thickness

- Bottom **two-ended slider**: center follows cursor depth along the snapped axis; drag either handle to grow N / box thickness (always symmetric).
- **Shift + wheel** nudges thickness in voxels.

View-slice keys (`[ ]` depth, `; '` thickness, `, .` zoom, arrows pan) still drive the existing Z slab / inset, not the cutter.

## Files

Stem: `exports/pycelium_<YYYYMMDD_HHMMSS>_<axis>_<squash|rich>.*`

| File | Contents |
|------|----------|
| `.png` | Density / biomass view (teal hypha + rust soluble C, same look as the inset) |
| `_mask.png` | Greyscale density mask (max-normalized) for later texture / material use |
| `_height.png` | Optional. Greyscale along the chosen domain axis (black↔white). `H` / `Y`. |
| `.json` | `pycelium-slice-v1`: grid meta, axis, thickness, stats, `density_u8`, rich `samples` |
| `.svg` | Marching-squares contours of high-density regions |

## CLI

```bash
cargo run -p pycelium-win --release -- --preset demo
# E enter capture   hover a face   C rich/2D   Enter write
cargo run -p pycelium-win --release -- --export-dir D:\captures
```

`--export-dir` does not change RAM / GPU presets. `--bench` stays headless and does not export.
