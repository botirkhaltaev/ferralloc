# AGENTS.md

Scope: `crates/runic-core/src/`.

- Put behavior on the entity that owns the data or invariant.
- Keep module boundaries direct: `Heap`, `PageMap`, `RunHeap`, `ExtentHeap`, `RunArena`, `ExtentArena`, `Run`, `Extent`, `OsMemory`, and `SizeClasses` should own their responsibilities.
- Prefer `NonZero*`, `NonNull`, and named-field domain types over sentinel values or ambiguous tuple structs.
- Unsafe blocks must be narrow and adjacent to the safety reasoning.
- Avoid callback-style helpers for ordinary control flow.
- Add tests beside the module that owns the invariant being changed.
