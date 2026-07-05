# AGENTS.md

Scope: `crates/runic-bench/src/bin/`.

- Binaries here are benchmark tools, not public product CLIs.
- Keep output stable enough for comparison across allocator changes.
- Prefer explicit arguments over environment-dependent behavior.
