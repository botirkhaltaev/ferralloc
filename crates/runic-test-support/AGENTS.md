# AGENTS.md

Scope: `crates/runic-test-support/`.

- This crate is internal and `publish = false`.
- Keep helpers deterministic and allocation-conscious.
- Prefer reusable trace/distribution entities over broad test utility bags.
- Do not move allocator logic into test support.
