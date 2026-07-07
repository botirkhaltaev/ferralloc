# runic-core/src

This directory contains the allocator core. Modules are intentionally organized around allocator entities and invariants.

## Modules

- `address`: address ranges and pointer offset helpers.
- `allocator`: public core allocator facade and abort boundary used by the global wrapper.
- `extent`: dedicated allocation metadata, extent arena, extent heap, and mapping reuse.
- `heap`: allocation policy and lock-protected allocator state.
- `layout`: normalized allocation layout semantics and mapping sizing.
- `memory`: address ranges, mmap-backed memory ownership, and page-indexed owner lookup.
- `run`: size-classed fixed-block allocation metadata, run arena, and run heap.
- `size_class`: size-class selection.

## Invariant

Every returned pointer must map to exactly one page-map entry. Runs accept only valid block-boundary frees; extents accept only the exact returned pointer.

## Tests

Unit tests live with the owning module when possible. Cross-entity allocator traces live in `../tests/`.
