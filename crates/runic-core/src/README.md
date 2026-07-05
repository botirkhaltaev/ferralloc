# runic-core/src

This directory contains the allocator core. Modules are intentionally organized around allocator entities and invariants.

## Modules

- `address`: address ranges and pointer offset helpers.
- `allocator`: public core allocator facade and abort boundary used by the global wrapper.
- `extent`: metadata for one dedicated allocation mapping.
- `extent_table`: out-of-line extent metadata storage and reservations.
- `free_list`: intrusive free-block stack used by runs.
- `heap`: allocation policy and lock-protected allocator state.
- `layout`: normalized allocation layout semantics and mapping sizing.
- `os_memory`: mmap-backed memory ownership and page rounding.
- `page_map`: page-indexed owner lookup for returned pointers.
- `run`: size-classed fixed-block allocation metadata.
- `run_table`: out-of-line run metadata storage and reservations.
- `size_class`: size-class selection.

## Invariant

Every returned pointer must map to exactly one page-map entry. Runs accept only valid block-boundary frees; extents accept only the exact returned pointer.

## Tests

Unit tests live with the owning module when possible. Cross-entity allocator traces live in `../tests/`.
