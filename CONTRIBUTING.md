# Contributing

Pycelium is a **simulator / idea lab**, not a game. Prefer measurable changes: grid scale, occupancy, tip/hypha counts, elapsed time, cells/sec. Keep the core stepper (`crates/pycelium-core`) free of UI so later GPU / threading work does not have to unwind a renderer.

This repo is independent of Mycelium Engine. Do not import, submodule, or otherwise integrate Engine code here.

## Layout

| Path | Role |
|------|------|
| `crates/pycelium-core` | Dense 2D growth sim + experiment report |
| `crates/pycelium-lab` | CLI harness, ASCII slice, PGM export |
| `legacy/` | Original Python CLI toy |

## Checks

```bash
cargo test
python -c "import ast; ast.parse(open('legacy/pycelium.py').read())"
```

## Credits

Credits and AI credits will follow the [Concrete Echo](https://github.com/initialvisuals/_CONCRETE_ECHO_) pattern later: **name + role + shipped work**.

Do not invent credits. Only record people or agents that actually landed work, and describe what shipped.
