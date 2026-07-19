# AGENTS.md

Scope: `crates/runic-core/src/heap/extent/`.

- Keep exact-pointer validation on `Extent`.
- Keep dedicated allocation policy, mapping reuse, and cache-vs-fresh zeroing (`ExtentInit`) on `ExtentHeap`.
- Keep metadata storage and reservation behavior on `ExtentArena`.
- Remove page-map ownership before removing extent metadata.
- Avoid unbounded mapping retention; preserve `ExtentCache` slot and byte limits.
- Add tests beside the extent entity that owns the changed invariant.
