# AGENTS.md

Scope: `crates/runic-bench/`.

- This crate is internal and `publish = false`.
- Benchmarks should compare allocator behavior without weakening correctness checks.
- Keep workloads deterministic unless randomness is explicitly seeded and reported.
- Do not optimize benchmark code in ways that change allocator semantics being measured.
- Run `cargo bench -p runic-bench --no-run` after changes.
