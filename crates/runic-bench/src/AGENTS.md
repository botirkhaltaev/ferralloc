# AGENTS.md

Scope: `crates/runic-bench/src/`.

- Keep benchmark workloads deterministic and documented.
- Touch allocated memory when measuring allocation paths so work is not optimized away.
- Preserve alignment and sampled realloc validation unless a benchmark explicitly documents otherwise.
- Keep allocator-target selection centralized in `allocator_target`.
