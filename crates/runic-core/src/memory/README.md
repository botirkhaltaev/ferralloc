# memory

Memory modules own address ranges, OS mappings, and page-indexed pointer lookup.

## Files

- `address.rs`: address ranges and pointer offset checks.
- `os.rs`: mmap/munmap ownership and page-size helpers.
- `page_map.rs`: page-indexed lookup from user pointers to `PageOwner` metadata pointers.
- `mod.rs`: module exports.

## Invariants

- Every returned pointer maps to exactly one `PageOwner` while allocated.
- `PageOwner` pointers must refer to live arena entries until their page-map range is removed.
- Page-map insertion rejects overlapping ownership.
- Page-map removal validates the expected owner before clearing entries.
