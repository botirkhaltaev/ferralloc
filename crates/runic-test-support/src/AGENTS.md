# AGENTS.md

Scope: `crates/runic-test-support/src/`.

- Keep APIs deterministic and composable for tests.
- Avoid global mutable state unless a subprocess boundary requires it.
- Prefer explicit trace entities over callback-heavy helpers.
- Keep dependencies minimal; this crate should not become a general utility crate.
