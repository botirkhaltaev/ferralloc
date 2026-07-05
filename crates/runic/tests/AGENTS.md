# AGENTS.md

Scope: `crates/runic/tests/`.

- Use subprocesses for tests that expect aborts.
- Keep smoke tests focused on public `RunicAlloc` behavior.
- Avoid assumptions about exact pointer reuse in the shared global heap.
