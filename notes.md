# dev notes

## 2026-06-26

25 tests green. finally. the heredoc bullshit almost killed me but whatever.

the thing actually works now. takes broken rust code, builds a graph from
compiler errors, tries a bunch of deterministic fixes (type replacement,
import adding, line deletion, renaming, full main replacement), validates
each one against rustc, and only falls back to llm if nothing works.

the candidate generator has operator stats now - it tracks which operators
succeed and ranks them. that's the learning part. not gradient descent, just
counting. but it works.

memory is three tiers. episodic logs everything, semantic stores schemas,
procedural caches proven fixes. when the same error comes in twice, second
time is instant cache hit. that's satisfying.

still don't know if this is actually new or just a well-engineered router.
the schema composition thing might be where the real innovation is, but
I'm not touching that until the base is rock solid. graph merging is a
fucking nightmare if you do it wrong.

the grant thing - I don't care. this is my project. if someone wants to
throw money at it, fine. but I'm not writing any bullshit marketing copy.
the code speaks for itself.

next: probably real llm integration. ollama or something. but honestly
the stub is fine for now. the interesting part is the search, not the llm.