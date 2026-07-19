# runic-core/src

Allocator core organized around entities and invariants.

## Modules

- `allocator`: public core facade and abort boundary used by the global wrapper.
- `config`: allocator and extent retention/reuse configuration.
- `heap`: owner-local heaps, run/extent heaps and arenas, heap table, and thread binding.
- `layout`: normalized layout semantics and mapping sizing.
- `memory`: address ranges, mmap ownership, and page-indexed owner lookup.
- `size_class`: size-class selection.
- `slot_store`: fixed-capacity slot storage for arenas and related tables.

## Invariant

Every returned pointer must map to exactly one page-map entry. Runs accept only valid block-boundary frees; extents accept only the exact returned pointer.

## Tests

Unit tests live with the owning module when possible. Cross-entity allocator traces live in `../tests/`.
