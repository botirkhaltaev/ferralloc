# AGENTS.md

Scope: `crates/runic-bench/benches/`.

- Keep benchmark entry points thin; shared behavior belongs in `src/` or `common/`.
- Add a separate `global_*` target when measuring a process-global allocator.
- Keep Criterion settings developer-sized unless documenting a longer benchmark profile.
