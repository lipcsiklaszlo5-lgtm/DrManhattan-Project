#!/usr/bin/env python3
"""Save current results as new golden baseline (manual step)."""
import json, sys
from pathlib import Path
PROJECT_ROOT = Path(__file__).resolve().parent.parent
# Dummy: user must run manually with the JSON from arc_blackbox_regression.py
print("Run arc_blackbox_regression.py first and copy its output to golden/baseline.json manually.")
