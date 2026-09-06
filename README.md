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

Windows / wgpu 3D pedon. Left telemetry HUD keeps the cryptic **3×5 bitmap digits** and fades in short English names (rich density by default; **Tab** cycles rich / sparse / off). Legend: [docs/HUD_LEGEND.md](docs/HUD_LEGEND.md).

```bash
cargo run -p pycelium-win --release -- --preset demo
# Tab: label density    hover a meter to focus    --labels sparse|off
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
| `docs/EXPERIMENT_LOG.md` | How to record a logged trial |
| `docs/experiments/` | CSV / JSONL schema + EXAMPLE rows |
| `README.rst` | Original short readme |
| `LICENSE` | License |

### License

See [LICENSE](LICENSE).

---

**Initial Visuals** — tools, sims, games, and experiments.
