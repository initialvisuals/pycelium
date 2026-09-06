Pycelium
========

A mycelium mesocosm. The Python file is the original ASCII plate. The Windows
build is a 3D soil column: the fungus is three-dimensional; a 2D view is a
section through it.

Python (tiny ASCII dish)
------------------------

::

    python pycelium.py

Windows GPU mesocosm
--------------------

Rust + ``wgpu`` (Vulkan / Direct3D 12). Tuned on an RTX 3060 / 64 GB machine.

::

    cargo run --release -p pycelium-win
    cargo run --release -p pycelium-win -- --preset demo
    cargo run --release -p pycelium-win -- --preset beast --ram-gb 48
    cargo run --release -p pycelium-win -- --preset demo --bench 200

The living colony sits in a GPU brick. A much larger pedon (organic C, soluble
C, soluble N, moisture) lives in system RAM. Depth is wetter; litter is at the
surface; wood is C-rich and elongated; nitrogen patches are offset so the
network has to forage.

Layers
------

1. Pedon chemistry (host RAM)
2. Exoenzymes — Michaelis–Menten cleavage, product repression, moisture gate
3. Tips — persistence + chemotropism + nitrotropism − autotropism; vesicle-limited extension; ~70° laterals
4. Network — biomass wall; anastomosis dumps cytoplasm into the cord
5. Translocation — internal C conducts only along biomass (√(ρᵢρⱼ))

Inoculum is a handful of germ tubes. Time fills the volume.

Experiment log
--------------

How to record a logged trial (presets, ``--drift`` / ``--bench``, HUD
bake metrics): ``docs/EXPERIMENT_LOG.md``. Append runs to
``docs/experiments/runs.csv``. Rows marked EXAMPLE are format samples,
not measurements.

HUD
---

Left panel: FPS, live tips, fusions, branches, C:N, bars for biomass / cord C /
soluble C / N / enzyme / organic, plus the clicked tip (id, lineage, age,
reserve) and the active parameter. English names fade in beside the 3×5
glyphs (Tab cycles rich / sparse / off). Legend: ``docs/HUD_LEGEND.md``.

Inset: orthogonal slice (the plate photograph).

Controls: drag orbit, wheel zoom, click pick, ``1-8`` parameter,
``-/=`` tune, Space pause, R reseed, F litter, D drift, Esc quit.

Slab (the plate photograph): ``[ ]`` depth, ``; '`` thickness, ``, .``
XY field, arrows pan, Shift for coarse steps. The 3D view tints the
active slab so you can see where the section sits in the column.
