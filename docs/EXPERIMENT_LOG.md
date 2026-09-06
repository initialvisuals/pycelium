# Experiment log

Pycelium is a **sim lab**, not a wet-lab notebook. This log is for recording
parameter combinations and the **bakeable emergent metrics** they produce in
the Windows GPU mesocosm (`pycelium-win`).

Use it to decide whether a combo is worth locking (baking) into a preset,
a demo, or a later house copy. Do **not** treat HUD numbers as fungal
discovery, field ecology, or claims about real mycelium.

Target box for house runs: **RTX 3060 12 GB + 64 GB RAM**.

InitialVisualsAdmin may later park a canonical house copy of this book.
Until then, append here. Engine is out of scope — do not change it for
logging work.

## Who appends

| Operator | Typical use |
|----------|-------------|
| Evan | Interactive HUD sessions, qualitative notes, bake calls |
| Engine | Leave the engine alone; do not log from it |
| CE / agents | Scripted `--bench` smokes and documented HUD snapshots |

Each row should name the operator in `operator` so a later house copy can
sort human vs agent runs.

## Living files

| Path | Role |
|------|------|
| [`experiments/runs.csv`](experiments/runs.csv) | Append real runs here (header only to start) |
| [`experiments/examples.csv`](experiments/examples.csv) | Format samples only — **not data** |
| [`experiments/examples.jsonl`](experiments/examples.jsonl) | Same samples as JSONL |

Copy the CSV header (or one JSONL object shape) and append. Keep example
rows out of `runs.csv`. If a row is a format check, set `row_kind=EXAMPLE`.

## How to run a logged trial

From the repo root:

```bash
# RTX-class default (performant): 4 GiB host, 192²×96 brick, 400k tip slots
cargo run --release -p pycelium-win

# Small pedon, fast startup
cargo run --release -p pycelium-win -- --preset demo

# Headless throughput (no window). Prints steps/s and M tip-slots/s.
cargo run --release -p pycelium-win -- --preset demo --bench 200

# Slide the GPU brick through the host pedon
cargo run --release -p pycelium-win -- --preset performant --drift

# Push host RAM (beast enables drift by default)
cargo run --release -p pycelium-win -- --preset beast --ram-gb 48
```

`--preset beast --ram-gb 48` is only for a 64 GB-class machine. On the
3060 box, start with `demo` or `performant` and raise `--ram-gb` only if
the pedon commit succeeds.

### Flags that belong in the row

| Flag | Column | Notes |
|------|--------|--------|
| `--preset demo\|performant\|beast` | `preset` | Defaults below if a size flag is omitted |
| `--ram-gb` | `ram_gb` | Host commit target in GiB |
| `--resolution` | `resolution` | GPU brick XY edge (depth is separate) |
| `--depth` | `depth` | GPU brick Z |
| `--agents` | `agents` | Tip slots on the GPU (most start dormant) |
| `--food` | `food` | Resource bodies stamped into the pedon (default 28) |
| `--steps` | `steps` | Sim steps per displayed frame (default 1) |
| `--drift` | `drift` | `y` if the brick slides through the pedon |
| `--vsync` | `notes` | Mention if you forced vsync; default is off |
| `--bench N` | `bench` | Headless; record `N` and the printed steps/s |

`D` in the window toggles drift at runtime. If you flip it, say so in
`notes` — the CLI flag is only the starting state.

There is **no CLI `--seed`**. `SimUniforms::seed` starts at `0xA31C5EED`
and increments every step. `R` reseeds tips and clears biomass. Put the
reseed story in `seed_notes` (for example `default seed; R once at ~2 min`).

### Preset defaults

From `pycelium-win/src/config.rs`. Overrides win; beast forces drift on
even without `--drift`.

| Preset | `ram_gb` | `resolution` | `depth` | `agents` | `drift` | Intent |
|--------|----------:|-------------:|--------:|---------:|:-------:|--------|
| `demo` | 0.85 | 96 | 64 | 48000 | n | Small pedon, fast startup |
| `performant` | 4.0 | 192 | 96 | 400000 | n | Default RTX-class card |
| `beast` | 40.0 | 256 | 160 | 1500000 | y | Push host RAM toward the machine limit |

Startup prints the resolved plan:

```text
GPU brick WxHxD | N tip slots | pedon WxHxD | spores N | host ~X GiB (target Y GiB)
```

Copy those resolved numbers into the row when they differ from the
preset table (clamps: XY 64–512, depth 32–384, agents 1024–8e6).

## What to record

### Interactive HUD (windowed run)

Left panel, top numbers (see `present.wgsl`):

| HUD | Column | What it is |
|-----|--------|------------|
| FPS | `fps` | Window title also shows FPS, brick, committed GiB |
| live tips | `live_tips` | Active tips this step (`tel[0]`) |
| fusions | `fusions` | Anastomosis events **this step** (`tel[1]`) — not a career total |
| branches | `branches` | New laterals **this step** (`tel[2]`) |
| C:N | `cn_ratio` | Strided soluble-C / soluble-N census, not a lab assay |

Telemetry is cleared every sim step. Treat `fusions` / `branches` as a
**rate snapshot**, not a cumulative fusion count. The C:N figure samples
every 4th voxel (`hud_reduce`); it is a forage-path hint, not stoichiometry.

Pause (`Space`) when the colony looks settled, read the HUD, then write
the row. Note roughly how long you waited (`notes`: `HUD at ~3 min`).

Bakeable questions to answer in `qualitative_notes` (sim language only):

- **Fusion rate** — tips vanishing into cords vs staying apical
- **Cord vs explorative tips** — thick persistent biomass vs wandering fronts
- **C:N hunting paths** — does the front lean toward N patches or C-rich wood?

Set `bake_candidate` to `y` only when those three are repeatable enough
to lock. `n` means “interesting, not stable” or “perf smoke only”.

### Headless `--bench`

No HUD. Record throughput from the printed line:

```text
bench: N steps in T s | X steps/s | Y M tip-slots/s | host Z GiB | brick WxHxD
```

Put steps/s in `fps` only if you say `bench steps/s` in `notes` — it is
not a frame rate. Leave HUD columns blank. `bake_candidate` is usually
`n` (capacity check, not a colony bake).

## Columns

| Column | Required | Values |
|--------|:--------:|--------|
| `row_kind` | yes | `RUN` or `EXAMPLE` |
| `timestamp` | yes | UTC ISO-8601 (`2026-09-06T12:00:00Z`) |
| `operator` | yes | `Evan`, `CE`, `agent`, … — never invent a house result as Evan’s |
| `preset` | yes | `demo` / `performant` / `beast` |
| `ram_gb` | yes | Number or blank if you truly used the preset default and said so |
| `resolution` | yes | GPU XY |
| `depth` | yes | GPU Z |
| `agents` | yes | Tip slots |
| `drift` | yes | `y` / `n` (resolved start state) |
| `food` | no | Default 28 |
| `steps` | no | Default 1 |
| `bench` | no | Frame/step count for `--bench`, else blank |
| `seed_notes` | no | Default seed, `R` reseeds, litter (`F`) |
| `fps` | HUD | Window FPS, or bench steps/s if labeled |
| `live_tips` | HUD | Integer snapshot |
| `fusions` | HUD | Per-step count |
| `branches` | HUD | Per-step count |
| `cn_ratio` | HUD | HUD C:N (one decimal is enough) |
| `qualitative_notes` | yes | Cord / explorative / C:N path notes; no wet-lab claims |
| `bake_candidate` | yes | `y` / `n` |
| `notes` | no | Hardware, vsync, crashes, “EXAMPLE format only” |

Quote CSV fields that contain commas. JSONL keys match these names.

## Suggested commands by intent

```bash
# 1. Does it boot?  (log as RUN, bake_candidate=n)
cargo run --release -p pycelium-win -- --preset demo --bench 200

# 2. 3060 interactive bake watch
cargo run --release -p pycelium-win -- --preset performant

# 3. Drift / forage-path watch
cargo run --release -p pycelium-win -- --preset performant --drift

# 4. RAM ceiling (64 GB box only)
cargo run --release -p pycelium-win -- --preset beast --ram-gb 48 --bench 60
```

## Rules

1. Do not invent measurements. If you did not run it, do not add a `RUN`.
2. Example rows stay in `examples.*` and keep `row_kind=EXAMPLE`.
3. No wet-lab, medical, or “we discovered” language.
4. Do not change Engine to add logging.
5. Do not change sim shaders or presets just to fill this book.
