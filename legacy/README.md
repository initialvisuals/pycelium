# Legacy Python prototype

This is the original 2017-era CLI mycelium toy: a single fungal head walks a 40×20 terminal grid toward food and leaves a trail.

It is kept so nothing is lost. It is **not** the PC sim-lab path.

## Run

From the repository root (compatibility launcher):

```bash
python pycelium.py
```

Or directly:

```bash
python legacy/pycelium.py
```

The original script calls `cls` to clear the screen (Windows). On Linux / macOS / Termux you may want to swap that for `clear`, or just let the frames scroll.

## Legend

| Glyph | Meaning |
|-------|---------|
| `●` | Fungal growth head |
| `◍` | Food source |

`README.rst` at the repo root is the original short readme for this toy.
