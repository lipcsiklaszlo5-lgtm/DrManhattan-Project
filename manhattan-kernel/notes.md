# dev notes

## 2026-06-26

got the whole fucking foundation up. task module with builder, 7 tests green.
structure module with KSG graph – nodes, edges, attributes, fingerprint placeholder.
adapter trait defined, compiler adapter as example with mock validation.
memory split into episodic, semantic, procedural. clean, no bullshit.
policy engine with cost model and decision logic (algorithm/cache/local_search/llm).
candidate generator with basic local search (delete edge for now, more operators later).
telemetry module tracking llm calls, avoidance rate, tokens.

cargo check clean, zero warnings. cargo test 7/7 green. this is a solid base.

next: more search operators, real compiler validator, bootstrap script idea.
also need to wire the adapter into the policy engine properly.
but the skeleton is here, and it's clean as fuck.
