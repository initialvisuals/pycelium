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
- **H** toggles a compact **control-scheme** panel (bottom-right; slides left of the export card when that card is open)
- Hover a HUD row (or a scheme row) for a white elbow string and a teach popup near the cursor. Boxes and bars stay hit-testable when Tab density is `off`.

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
| 0.56 | warm | `SLICE Z` | 4 | orthogonal slab depth (voxels) + slider |
| 0.60 | warm | `THICK` | 3 | slab thickness + slider |
| 0.64 | warm | `ZOOM` | 3 | XY field as percent (`zoom * 100`) + slider |

Keys: `[ ]` depth, `; '` thickness, `, .` zoom, arrows pan. Shift = coarse. Drag the framed track, or use the keys; a near-HUD toast shows the value and the keys while you tweak.

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
| 0.90 | ice | `PARAM` | 1 + 4 | slot `1–8` and value × 100 + slider |

Slots: chemotropism, nitrotropism, autotropism, persistence, maintenance, enzyme_k, branch_cost, extension. Keys `1–8` select, `-` / `=` nudge, or drag the framed track. While adjusting, a toast shows **slot / name / old→new** and `1–8 select  -/= nudge`, then fades after a short idle.

Slice **export** (face-aligned 2D squash / rich box) is a separate capture mode (`E`). It does not change these HUD rows. The export settings card (`S`) lives on the **right**, under the slab inset. See [SLICE_EXPORT.md](SLICE_EXPORT.md).

## Control scheme + hover teach

**H** opens or hides the corner keybind panel (global, including during capture). Hover any left-HUD meter — FPS through PARAM — or a row on that panel for a verbose plain-English callout (sim meaning, plus terms such as anastomosis, chemotropism, soluble C/N, exoenzyme, cord). The popup sits near the mouse and is tied to the control with a thin white elbow / string (1–2 px shadow, no blur).

Tab `off` still teaches if the cursor is on a color box, bar, or slider. Sliders stay drawn in every density mode (same as bars). If a future chrome piece has no hit box, teach cannot fire for it.

Heightmap PNG write in capture is **M**, not H. **Y** still cycles the heightmap axis and turns that write on.

## Inset (right)

The orthogonal slab inset is **not** part of the left HUD. It shows hypha + soluble C in the current Z slab, framed, with zoom/pan.

## Controls that do not change presets

`--preset performant|beast|demo`, `--bench`, and host RAM planning are independent of `--labels`. English captions are present-pass only.
