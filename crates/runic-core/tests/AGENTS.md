# AGENTS.md

Scope: `crates/runic-core/tests/`.

- Tests here should cover cross-entity behavior rather than one module's private invariant.
- Prefer deterministic traces and explicit layout/alignment cases.
- Do not test invalid frees in-process if they abort; use subprocess tests in `crates/runic/tests/`.
- Keep tests allocation-aware; avoid hiding allocator recursion risks behind complex test fixtures.
