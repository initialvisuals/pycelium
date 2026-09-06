# Slice export + cutter

Capture a face-aligned plane (or a thin box) from the GPU mesocosm and write **PNG + JSON + SVG** into `exports/` (override with `--export-dir`). No mesh or voxel conversion — density images, a mask, optional heightmap, and a small JSON record.

Orbit, tip pick, HUD glyphs/labels, and **Tab** label density are unchanged. Capture is a separate mode.

## Enter / leave

| Key | Action |
|-----|--------|
| `E` | Toggle capture mode |
| `C` | Cycle **2D squash** ↔ **rich box** (while capturing) |
| `S` | Toggle the **export settings** card (while capturing) |
| `P` | Cycle **pad** ↔ **crop** to dense AABB |
| `A` | Cycle **square POT** ↔ **native** aspect (advanced) |
| `Enter` | Write files (timestamped stem) |
| `Esc` | Close settings if open; otherwise leave capture (quit only when capture is off) |

Mouse move positions the cutter through the cube. Left-drag still orbits unless you grab a slider handle or click the settings card. Click-to-pick is disabled while capturing.

## Two capture modes

1. **2D squash** (default) — same idea as the orange view slab: stack **N** nearby layers and keep the max biomass / soluble C on one plane. Starts at **1 layer** at the cursor. Drag the two-ended slider handles outward to thicken the bake.
2. **Rich (3D box)** — two parallel planes forming a thin box you stretch (same slider). The PNG is still a density projection of that box; JSON also stores occupied volume samples `{u,v,t,d}` between the planes.

## Face snap (Blender-like)

The world is the unit cube. Hover near a **face center** (halfway) and the cutter axis snaps to that face’s normal:

- Top / bottom (`±Z`) → horizontal slice (XY)
- Side faces (`±X` / `±Y`) → **vertical** slice, so upright structures can be captured

Hover a **face mid-edge** to rotate **90° on the free axis** (top-face left/right mid-edge → vertical X; top-face front/back mid-edge → vertical Y). The orange preview plane/box and a wash on the snapped faces show the alignment.

`X` cycles axis by hand if snap misses. `Y` cycles the optional heightmap axis (and turns heightmap export on). `M` toggles writing the heightmap PNG. **H** is reserved for the global control-scheme panel (see [HUD_LEGEND.md](HUD_LEGEND.md)); it never writes a heightmap.

## Export settings (square POT)

Default bake is a **power-of-two square**, not the ultra-wide native slab. The in-engine settings card sits on the right under the slab inset while capturing, above the thickness slider. Click the **EXPORT** chip (or press `S`) to open it. Size presets are a **two-column** grid so the full card stays on-screen and every control is inside the hit rect.

| Control | Default | Notes |
|---------|---------|--------|
| Size preset | **512×512** | 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, **8192 (8K)**. Click a cell, or wheel over the card. |
| Fit | **Pad** | Letterbox / pillarbox the full plane into the square. **Crop** trims to the occupied density AABB first, then pads. |
| Aspect | **Square** | Advanced: **Native** keeps the extracted slab width×height (old behavior). Size presets apply only to square. |
| Formats | PNG MASK JSON SVG on | Click a chip to include or skip that file. At least one file stays armed. |
| Heightmap | **off** | Click the HEIGHT row, or press `M`. `Y` still picks the axis and turns this on. |

PNG, density mask, JSON, SVG, and optional heightmap (`M` / `Y`) are the same files as before; the card toggles are live, not labels.

PNG / mask / SVG / `density_u8` are written at the chosen output size. JSON also records `export_aspect`, `export_fit`, `export_preset`, `source_width`, and `source_height`.

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
| `_height.png` | Optional. Greyscale along the chosen domain axis (black↔white). `M` / `Y`. |
| `.json` | `pycelium-slice-v1`: grid meta, axis, thickness, stats, `density_u8`, rich `samples`, plus `export_aspect` / `export_fit` / `export_preset` / source size |
| `.svg` | Marching-squares contours of high-density regions |

## CLI

```bash
cargo run -p pycelium-win --release -- --preset demo
# E enter capture   hover a face   C rich/2D   S settings   Enter write
cargo run -p pycelium-win --release -- --export-dir D:\captures
```

`--export-dir` does not change RAM / GPU presets. `--bench` stays headless and does not export. Export size is chosen in the capture settings card (default 512×512 square), not from the CLI.
