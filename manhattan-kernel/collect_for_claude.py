#!/usr/bin/env python3
"""Összegyűjti a Manhattan Kernel forráskódját és egy promptot generál Claude-nak."""

import os
import glob

# A gyűjtendő fájlok listája (súlyozva a fontosság szerint)
FILES_TO_COLLECT = [
    # Core abstractions
    "src/abstraction/program.rs",
    "src/abstraction/transform.rs",
    "src/abstraction/hypothesis.rs",
    "src/abstraction/goal_decomposer.rs",
    "src/abstraction/representation.rs",
    # Structure (KSG)
    "src/structure/graph.rs",
    "src/structure/topology.rs",
    # Sandbox (operators)
    "src/sandbox/operators.rs",
    # Concept learning
    "src/concept/mod.rs",
    "src/concept/detectors.rs",
    "src/concept_learner.rs",
    # Agent & Meta-learner
    "src/agent/agent_loop.rs",
    "src/agent/explorer.rs",
    "src/meta_learner.rs",
    # Policy
    "src/policy/mod.rs",
    # ARC adapter
    "src/adapter/arc/adapter.rs",
    # Hypothesis bus
    "src/hypothesis_bus.rs",
    # Cargo.toml a függőségekhez
    "Cargo.toml",
]

OUTPUT_FILE = "manhattan_kernel_for_claude.txt"

def collect_files():
    """Összegyűjti a fájlokat egyetlen kimeneti fájlba."""
    with open(OUTPUT_FILE, 'w', encoding='utf-8') as out:
        out.write("=" * 80 + "\n")
        out.write("MANHATTAN KERNEL - TELJES FORRÁSKÓD\n")
        out.write("=" * 80 + "\n\n")
        
        for filepath in FILES_TO_COLLECT:
            if os.path.exists(filepath):
                out.write(f"\n{'='*80}\n")
                out.write(f"FÁJL: {filepath}\n")
                out.write(f"{'='*80}\n\n")
                try:
                    with open(filepath, 'r', encoding='utf-8') as f:
                        content = f.read()
                        out.write(content)
                        out.write("\n\n")
                except Exception as e:
                    out.write(f"[HIBA OLVASÁSKOR: {e}]\n\n")
            else:
                out.write(f"\n{'='*80}\n")
                out.write(f"FÁJL: {filepath} - NEM TALÁLHATÓ\n")
                out.write(f"{'='*80}\n\n")
    
    file_size = os.path.getsize(OUTPUT_FILE)
    print(f"Kimeneti fájl létrehozva: {OUTPUT_FILE}")
    print(f"Méret: {file_size:,} bájt")
    print(f"Tartalmazott fájlok száma: {len([f for f in FILES_TO_COLLECT if os.path.exists(f)])}/{len(FILES_TO_COLLECT)}")

if __name__ == "__main__":
    collect_files()
