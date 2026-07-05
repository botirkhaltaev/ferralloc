# runic-test-support

`runic-test-support` holds reusable test machinery for Runic.

This crate is internal to the workspace and is not published to crates.io.

## Intended Scope

- Deterministic allocation traces.
- Workload distributions shared by tests.
- Subprocess helpers for abort behavior.

The crate is intentionally small; test helpers should not obscure allocator invariants.
