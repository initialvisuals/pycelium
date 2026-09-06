# Pycelium-win telemetry HUD

Left-column overlay drawn in `pycelium-win/src/shaders/present.wgsl`. The **3×5 bitmap digits stay**; English is a fade-in mono caption next to each meter.

Default density is **rich**: names sit at low opacity, then fade up on hover or after a tip pick. **Tab** cycles `rich → sparse → off → rich`. `--labels sparse|off` sets the startup mode. Bench mode (`--bench`) never opens a window and is unchanged.

## House chrome

- White 5×5 mono captions (not a replacement for the alien 3×5 glyphs)
- Thin 1-pixel frames around census / fields / slice / tip / param groups
- Optional elbow ticks on the focused row
- Tight 2-pixel drop shadow (no blur)
- Density dial: rich (default), sparse (hover/pick only), off (glyphs only)

## Census (top glyphs)

| Row (UV y) | Glyph | English | Source |
|------------|-------|---------|--------|
| 0.06 | 3 digits | `FPS` | present uniform `fps` |
| 0.12 | 6 digits | `TIPS` | `hud[0]` live apical tips |
| 0.16 | 5 digits | `FUSIONS` | `hud[1]` anastomosis events |
| 0.20 | 5 digits | `BRANCHES` | `hud[2]` branch births |
| 0.24 | 4 digits | `C:N` | `hud[6] / hud[7]` soluble carbon : nitrogen |

`hud[3]` is an internal growth accumulator and is **not** drawn.

## Field bars (teal → brown)

Fill is a strided voxel census (`hud_reduce`, every 4th cell). Color matches the volume look.

| Bar y | English | Color | Buffer / scale |
|-------|---------|-------|----------------|
| 0.32 | `HYPHA` | teal `0.55, 0.92, 0.82` | `hud[4]` biomass / 80000 |
| 0.36 | `CORD` | gold `0.95, 0.72, 0.28` | `hud[5]` internal C / 40000 |
| 0.40 | `SOL C` | rust `0.78, 0.42, 0.10` | `hud[6]` soluble C / 40000 |
| 0.44 | `SOL N` | violet `0.55, 0.40, 0.85` | `hud[7]` soluble N / 25000 |
| 0.48 | `ENZYME` | green `0.32, 0.70, 0.34` | `hud[8]` exoenzyme / 20000 |
| 0.52 | `ORGANIC` | brown `0.45, 0.32, 0.18` | `hud[9]` uncleaved polymer / 50000 |

## Slice (lower-middle glyphs)

| Row | Glyph | English | Meaning |
|-----|-------|---------|---------|
| 0.56 | 4 digits | `SLICE Z` | orthogonal slab depth (voxels) |
| 0.60 | 3 digits | `THICK` | slab thickness |
| 0.64 | 3 digits | `ZOOM` | XY field as percent (`zoom * 100`) |

Keys: `[ ]` depth, `; '` thickness, `, .` zoom, arrows pan. Shift = coarse.

## Selected tip

Click in the volume to pick the nearest live tip. `hud[10]` is the pick distance key (not drawn). `hud[11]` is the tip id.

| Row | Glyph | English | Meaning |
|-----|-------|---------|---------|
| 0.70 | 6 digits | `TIP` | selected tip slot |
| 0.74 | 3 digits | `LINEAGE` | inoculum / colony id |
| 0.78 | 5 digits | `AGE` | tip age |
| 0.82 | 4 digits | `RESERVE` | internal reserve × 100 |

After a pick, sparse mode keeps this block readable.

## Param knob

| Row | Glyph | English | Meaning |
|-----|-------|---------|---------|
| 0.90 | 1 + 4 digits | `PARAM` | slot `1–8` and value × 100 |

Slots: chemotropism, nitrotropism, autotropism, persistence, maintenance, enzyme_k, branch_cost, extension. Keys `1–8` select, `-` / `=` nudge.

## Inset (right)

The orthogonal slab inset is **not** part of the left HUD. It shows hypha + soluble C in the current Z slab, framed, with zoom/pan.

## Controls that do not change presets

`--preset performant|beast|demo`, `--bench`, and host RAM planning are independent of `--labels`. English captions are present-pass only.
