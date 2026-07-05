# AGENTS.md

Scope: `crates/runic/src/`.

- Keep wrapper code thin and explicit.
- Do not duplicate allocator policy from `runic-core`.
- Any unsafe call in `GlobalAlloc` should remain a direct delegation to the core boundary.
- Keep exports intentional; avoid expanding the public API casually.
