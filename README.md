# manhattan-kernel

CPU-only, deterministic runtime for isolating and constraining LLM outputs.

## What it does

The kernel takes a task (e.g., broken Rust code), builds a structured graph
representation (KSG) from compiler errors, searches for a valid fix using
deterministic operators, and only calls an LLM as a last resort.

The LLM never touches system state. It receives a reduced abstract schema,
returns a hypothesis, and the kernel validates it against the compiler before
accepting it. Failed hypotheses are discarded.

## Why

LLMs are non-deterministic black boxes. Most agent frameworks give them
direct access to tools, memory, and state. This is unsafe for anything
that needs to be reliable.

This kernel inverts that: algorithms are the default, LLM is the exception.
The kernel decides, routes, validates, and learns. The LLM only generates
candidates when all deterministic options are exhausted.

## Status

- 25 integration tests passing
- Compiler adapter working (rustc --emit=metadata)
- Candidate generator with 5 operator types
- Three-tier memory (episodic, semantic, procedural)
- Learning: operator success rates tracked, search ranked by confidence
- LLM executor stub (mock responses for testing)
- Bootstrap script for batch schema extraction

## Build

Requires Rust toolchain.

    cargo build
    cargo test

## Run

    cargo run -- "fn main() { let x: i32 = \"hello\"; }"

## Structure

    src/
      task/         - Task struct, builder, types
      structure/    - Kernel Structure Graph (KSG)
      adapter/      - DomainAdapter trait, CompilerAdapter
      executor/     - Executor trait, LLM executor stub
      memory/       - Episodic, semantic, procedural memory
      policy/       - Policy engine, cost model, decision logic
      candidate/    - Candidate generator, local search operators
      telemetry/    - LLM call tracking, avoidance rate

## License

MIT