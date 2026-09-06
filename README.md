# Pycelium

Minimalist **CLI mycelium simulator** — a fungal head grows toward food on a terminal grid, trails the path, and branches when it feeds.

Part of the [Initial Visuals](https://github.com/initialvisuals) toolkit / game lab. Spiritual cousin to later Mycelium / engine experiments.

### Status

Small learning toy. ASCII animation in the terminal. Windows-oriented clear (`cls`); on other platforms you may want `clear` instead.

### Quick start

```bash
python pycelium.py
```

### Legend

| Glyph | Meaning |
|-------|---------|
| `●` | Fungal growth head |
| `◍` | Food source |

### GPU mesocosm (`pycelium-win`)

Windows / wgpu 3D pedon. Left telemetry HUD uses **white sans-serif English** plus a color readout box per row (rich by default; **Tab** cycles rich / sparse / off). Adjustable meters (**SLICE Z**, **THICK**, **ZOOM**, and the eight **PARAM** knobs) have framed sliders; tweak keys show a short toast with the name, value, and binds. **H** toggles a corner control-scheme panel; hover a meter for a teach callout. A top-center **species strip** (eight squares) arms specimen inoculate or field paint: [docs/PAINT.md](docs/PAINT.md). Small 3×5 digits stay on the right. Legend: [docs/HUD_LEGEND.md](docs/HUD_LEGEND.md). Face-aligned **slice export** (`E` capture, `S` settings, `Enter` write PNG/JSON/SVG) defaults to a **512×512** power-of-two square (presets 8…8192): [docs/SLICE_EXPORT.md](docs/SLICE_EXPORT.md). Heightmap write is **M**.

```bash
cargo run -p pycelium-win --release -- --preset demo
# Tab: label density    H: control scheme    hover a meter for a teach callout
# T: specimen/paint    click a species square    1-8 slot    9/0 brush    Alt erase
# E: capture slice    S: export size (8…8K square)    C: 2D/rich    M: heightmap    Enter: export
# --bench N is headless and does not open the HUD
```

### Experiment log

Logged trials (presets, `--drift` / `--bench`, HUD bake metrics) live in
[`docs/EXPERIMENT_LOG.md`](docs/EXPERIMENT_LOG.md). Append runs to
[`docs/experiments/runs.csv`](docs/experiments/runs.csv). Example rows are
labeled **EXAMPLE** and are not measurements.

### Files

| Path | What |
|------|------|
| `pycelium.py` | CLI toy |
| `pycelium-win/` | GPU mesocosm + telemetry HUD |
| `docs/HUD_LEGEND.md` | Left HUD map (glyphs + English) |
| `docs/PAINT.md` | Species presets, inoculate, field paint |
| `docs/SLICE_EXPORT.md` | Capture cutter + export formats |
| `docs/EXPERIMENT_LOG.md` | How to record a logged trial |
| `docs/experiments/` | CSV / JSONL schema + EXAMPLE rows |
| `README.rst` | Original short readme |
| `LICENSE` | License |

### License

See [LICENSE](LICENSE).

---

**Initial Visuals** — tools, sims, games, and experiments.
