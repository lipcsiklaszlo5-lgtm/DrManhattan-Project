# Architecture

## Core principle

The kernel is deterministic. The LLM is not.

Every task is converted into a Kernel Structure Graph (KSG) – a typed,
attributed graph of nodes (errors, functions, variables) and edges (relations).
All operations on this graph are deterministic and verifiable.

The LLM is isolated. It never sees the full system state. It receives only
a reduced abstract schema (goal, inputs, outputs, constraints, unknown edge)
and returns a hypothesis. The kernel validates this hypothesis through an
objective, domain-specific validator (e.g., rustc --emit=metadata).

## Pipeline

    Task (raw input)
      │
      ▼
    Domain Adapter ──► KSG
      │
      ▼
    Policy Engine ──► decide path
      │
      ├─ cache hit ──► return stored solution
      ├─ local search ──► generate graph variants, validate each
      └─ exhausted ──► call LLM, validate response, store if valid
      │
      ▼
    Memory update (episodic, semantic, procedural)

## Memory tiers

- **Episodic**: what we tried, did it work, when
- **Semantic**: abstract schemas with algebra (requires/provides predicates)
- **Procedural**: proven solutions, cached by graph fingerprint

## Safety properties

1. **LLM output isolation**: LLM responses are validated before they affect state
2. **Deterministic fallback**: if validation fails, the system falls back to known-safe algorithms
3. **CPU-only**: runs on edge hardware, offline, sandboxable
4. **No prompt engineering**: the LLM receives structured schemas, not crafted text prompts
5. **Auditable**: every decision path is logged in episodic memory

## Current limitations

- Only Rust compiler errors are supported (extensible via DomainAdapter trait)
- LLM executor is a stub (mock responses for testing)
- Schema composition (A+B->C) is planned but not yet implemented
- No real LLM integration yet (ollama/API)

## Why Rust

- Zero-cost abstractions for graph operations
- Strong type system prevents entire classes of bugs
- No garbage collection -> predictable latency on edge hardware
- Native WebAssembly support for future browser-based sandboxing