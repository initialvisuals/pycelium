# Species presets, inoculate, and field paint

Lab tools for the GPU mesocosm (`pycelium-win`). Not a game brush. Stamps write the **same storage buffers** the sim already marches: tips, soluble C/N, moisture, organic, enzyme, biomass.

House chrome: top-center strip of eight square species buttons (colored glyph + short name), plus **SPEC** / **PAINT** chips. Hover a square for a teach callout. A toast (same family as the PARAM nudge) shows tool, species or channel, and brush radius.

See also [HUD_LEGEND.md](HUD_LEGEND.md).

## Tools

| Mode | Arm | Left-drag in the box | 1–8 |
|------|-----|----------------------|-----|
| **VIEW** | `T` until VIEW, or start | Orbit; release picks the nearest live tip | PARAM knobs (unchanged) |
| **SPECIMEN** | Click a species square, SPEC chip, or `T` | Inoculate germ tubes of that lineage | Select species + write its eight knobs |
| **PAINT** | PAINT chip or `T` | Deposit / subtract a field channel | Select channel |

- **Right-drag** always orbits (so the left button can stamp).
- **9 / 0** shrink / grow brush radius (Shift = coarse). Wheel does the same while a tool is armed, or while the cursor is on the strip. In VIEW the wheel still dollies.
- **Alt** subtracts: kill tips of the selected lineage (specimen), or remove the armed channel (paint).
- Capture (`E`) keeps the mouse for the cutter; stamps do not fire.

The stamp lands at the **ray–cube midpoint**, the same depth rule as the capture cutter — inside the volume, not only on the face.

## Eight starter species

Selecting a square **arms the specimen brush** and writes the eight global PARAM knobs. Those knobs are still shared by every live tip (no per-lineage genome yet). Painted inocula set `Tip.lineage` to 1…8 and `Tip.flags = 2` so a later genetics pass can tell them from the startup handful (`flags = 0`) and `grow()` branches (`flags = 1`).

| # | Name | Short | Color | Lineage | Bias |
|---|------|-------|-------|---------|------|
| 1 | Pioneer | PION | ice teal | 1 | High chemo + extension, cheap forks, twitchy |
| 2 | Cord-former | CORD | amber | 2 | High persist + extension, expensive branches |
| 3 | Scavenger | SCAV | rust | 3 | High enzyme_k + chemo; litter miner |
| 4 | Nitrophile | NITR | violet | 4 | High nitrotropism, ignores C plumes |
| 5 | Mat-former | MAT | green | 5 | Low auto / persist, cheap forks, dense fill |
| 6 | Thrifty | THRF | gold | 6 | Very low maintenance, cautious steps |
| 7 | Ranger | RANG | coral | 7 | Long hunting arcs: chemo + persist + auto + extend |
| 8 | Miner | MINE | ice | 8 | Costly aggressive metabolizer (antagonism hook) |

Startup germ tubes still use inoculum sites 1–5 (`model.rs`). They are not these named presets.

**Extension hook:** antagonism / per-tip genes should read `lineage` and `flags` rather than inventing a second id. Do not treat Miner as chemical warfare — it is flavor plus a costly enzyme/maintenance mix.

## Paint channels

`1–8` in PAINT mode. All writes are GPU compute on a small AABB around the stamp (radius-bounded), not a full-volume pass.

| # | Channel | Buffer(s) |
|---|---------|-----------|
| 1 | SOL C | `soluble_c` |
| 2 | SOL N | `soluble_n` |
| 3 | H2O | `moisture` (gates enzyme + diffusion) |
| 4 | ORG | `organic` polymer |
| 5 | ENZ | `enzyme` |
| 6 | WOOD | organic + a little soluble C (woody litter) |
| 7 | LITTER | organic + soluble N (+ a touch of C) |
| 8 | VOID | cutout: scales down biomass, organic, soluble C/N, enzyme, internal C, and kills tips in the brush |

There is **no dedicated poison field**. VOID is a subtractive stamp on existing buffers. Moisture is real and already used in `kinetics` / `diffuse`; it is not drawn as a left-HUD bar yet.

## Performance

- Field stamps dispatch `(2r+2)³` threads in 8×8×4 groups. Default radius is 5 voxels; max 24.
- Inoculate walks tip slots once, claims at most six dormant **parent-half** slots (`id < tip_count/2`) so `grow()` keeps its branch pool.
- Strokes skip samples closer than `0.4 × radius`.

Target box remains an RTX 3060-class card. `--bench` never opens a window and does not exercise the strip.

## Keys (additive)

| Key | Action |
|-----|--------|
| `T` | Cycle VIEW → SPECIMEN → PAINT |
| Click square | Arm that species + specimen tool |
| `1–8` | VIEW: param. SPECIMEN: species. PAINT: channel |
| `9` `0` | Brush radius |
| Alt | Erase / subtract |
| Right-drag | Orbit |
| Wheel | Dolly in VIEW; radius when a tool is armed |
