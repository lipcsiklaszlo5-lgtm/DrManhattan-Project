# Manhattan Kernel

## One sentence
A CPU-only, edge-capable AI runtime that executes algorithmic tasks
deterministically and calls a small language model only for genuinely
open-ended problems.

## Core principle
LLM = hypothesis generator  
Kernel = consequence validator  

The kernel never evaluates whether the LLM is "right".
It only checks whether the consequence passes an objective test.

## Architecture
Task (raw input)
→ Domain Adapter → Kernel Structure Graph (KSG)
→ Policy Engine (CostModel, confidence)
├─ Algorithm Registry (deterministic solvers)
├─ Candidate Generator (local search on schemas)
│ ├─ edge add/delete, node merge/split, param swap, order swap
│ └─ guided by procedural memory confidence
└─ LLM Hypothesis Generator (only when search exhausted)
→ Unified Validator (always runs, objective)
→ Memory (episodic / semantic / procedural)

text

## Memory tiers
- episodic: what we tried, did it work
- semantic: abstract schemas (KSG snippets) with confidence
- procedural: proven algorithms, validators, routing rules

## Candidate Generator – local search
Operators: edge deletion, edge addition, node merge, node split, attribute change, order swap.
Guided by historical success rates stored in procedural memory.
Bounded by depth (2-3) and beam width (top-k) to avoid combinatorial explosion.

## Bootstrapping
Can be pre-filled with millions of validated schemas from a cloud batch run.
On edge devices, only the index is loaded; full schema DB stays on SSD.

## Current status
All core modules implemented and tested.
