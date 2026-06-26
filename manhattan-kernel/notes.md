# dev notes

## 2026-06-26

this shit is solid. 23 tests green across all modules. task, structure,
adapter, memory, policy, candidate, telemetry – every single one tested.

KSG graph has fingerprint now, candidate generator does basic edge deletion,
policy engine decides correctly between algorithm/cache/local_search/llm.
compiler adapter mock validates fake candidates like a champ.

this is not an MVP. this is a fucking tank.

next: more search operators (add edge, merge nodes, swap params), real
compiler validator (cargo check integration), bootstrap script.
also need to wire adapter into policy engine for real decisions,
not just the stub. but the foundation is rock solid.
