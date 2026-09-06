# Pycelium-win telemetry HUD

Left-column overlay drawn in `pycelium-win/src/shaders/present.wgsl`. **English is the readable layer**: white geometric sans-serif captions (stroked atlas, not the old 5×5 bitmaps). Each row has a **color readout box** whose fill tracks that meter. Small 3×5 digit clusters sit on the right as optional dense telemetry.

Default density is **rich**: names are fully visible. **Tab** cycles `rich → sparse → off → rich`. `--labels sparse|off` sets the startup mode. Bench mode (`--bench`) never opens a window and is unchanged.

The previous 5×5 captions were sampled with a Y flip against top-left bit packing, so they looked inverted / noisy. The atlas in `hud_font.rs` is built upright (`cell.y = 0` is the top of the letter).

## House chrome

- White sans-serif captions (system-UI / grotesque style)
- Color box at the left of every row (always drawn, including `off`)
- Thin 1-pixel frames around census / fields / slice / tip / param groups
- Tight 1–2 pixel drop shadow on type (no blur)
- Density dial: rich (default, full opacity), sparse (hover/pick only), off (boxes + bars + small digits)

## Census (top)

| Row (UV y) | Box | English | Small digits | Source |
|------------|-----|---------|--------------|--------|
| 0.06 | ice | `FPS` | 3 | present uniform `fps` |
| 0.10 | teal | `TIPS` | 6 | `hud[0]` live apical tips |
| 0.14 | amber | `FUSIONS` | 5 | `hud[1]` anastomosis events |
| 0.18 | green | `BRANCHES` | 5 | `hud[2]` branch births |
| 0.22 | rust | `C:N` | 4 | `hud[6] / hud[7]` soluble carbon : nitrogen |

`hud[3]` is an internal growth accumulator and is **not** drawn.

## Field bars (teal → brown)

Fill is a strided voxel census (`hud_reduce`, every 4th cell). The box and bar share a color family; box brightness follows fill.

| Bar y | English | Color | Buffer / scale |
|-------|---------|-------|----------------|
| 0.31 | `HYPHA` | teal `0.55, 0.92, 0.82` | `hud[4]` biomass / 80000 |
| 0.35 | `CORD` | amber `0.95, 0.72, 0.28` | `hud[5]` internal C / 40000 |
| 0.40 | `SOL C` | rust `0.78, 0.42, 0.10` | `hud[6]` soluble C / 40000 |
| 0.44 | `SOL N` | violet `0.55, 0.40, 0.85` | `hud[7]` soluble N / 25000 |
| 0.48 | `ENZYME` | green `0.32, 0.70, 0.34` | `hud[8]` exoenzyme / 20000 |
| 0.52 | `ORGANIC` | brown `0.45, 0.32, 0.18` | `hud[9]` uncleaved polymer / 50000 |

## Slice (lower-middle)

| Row | Box | English | Small digits | Meaning |
|-----|-----|---------|--------------|---------|
| 0.56 | warm | `SLICE Z` | 4 | orthogonal slab depth (voxels) |
| 0.60 | warm | `THICK` | 3 | slab thickness |
| 0.64 | warm | `ZOOM` | 3 | XY field as percent (`zoom * 100`) |

Keys: `[ ]` depth, `; '` thickness, `, .` zoom, arrows pan. Shift = coarse.

## Selected tip

Click in the volume to pick the nearest live tip. `hud[10]` is the pick distance key (not drawn). `hud[11]` is the tip id. Boxes brighten after a pick.

| Row | Box | English | Small digits | Meaning |
|-----|-----|---------|--------------|---------|
| 0.70 | teal | `TIP` | 6 | selected tip slot |
| 0.74 | gold | `LINEAGE` | 3 | inoculum / colony id |
| 0.78 | ice | `AGE` | 5 | tip age |
| 0.82 | amber | `RESERVE` | 4 | internal reserve × 100 |

After a pick, sparse mode keeps this block readable.

## Param knob

| Row | Box | English | Small digits | Meaning |
|-----|-----|---------|--------------|---------|
| 0.90 | ice | `PARAM` | 1 + 4 | slot `1–8` and value × 100 |

Slots: chemotropism, nitrotropism, autotropism, persistence, maintenance, enzyme_k, branch_cost, extension. Keys `1–8` select, `-` / `=` nudge.

Slice **export** (face-aligned 2D squash / rich box) is a separate capture mode (`E`). It does not change these HUD rows. The export settings card (`S`) lives on the **right**, under the slab inset. See [SLICE_EXPORT.md](SLICE_EXPORT.md).

## Inset (right)

The orthogonal slab inset is **not** part of the left HUD. It shows hypha + soluble C in the current Z slab, framed, with zoom/pan.

## Controls that do not change presets

`--preset performant|beast|demo`, `--bench`, and host RAM planning are independent of `--labels`. English captions are present-pass only.
