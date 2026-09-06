#!/usr/bin/env python3
"""Compatibility launcher for the original CLI mycelium toy."""

from pathlib import Path
import runpy

_LEGACY = Path(__file__).resolve().parent / "legacy" / "pycelium.py"
runpy.run_path(str(_LEGACY), run_name="__main__")
