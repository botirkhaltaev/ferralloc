# heap

Owner-local heap frontend: runs for small size classes, extents for dedicated large allocations, and the heap table / thread binding.

## Layout

- `mod.rs`: `Heap`, live allocation counts, and delegation into run/extent heaps.
- `owner.rs`: unified `Owner` for run and extent metadata.
- `arena.rs`: generic out-of-line metadata arena used by runs and extents.
- `id.rs`: `HeapId`.
- `run/`: size-classed fixed-block runs (`Run`, `RunHeap`, `RunArena`).
- `extent/`: dedicated mappings (`Extent`, `ExtentHeap`, `ExtentArena`, `ExtentCache`).
- `table/`: `HeapTable`, `HeapSlot`, and `ThreadHeap`.

## Invariants

- Small allocations are owned by a heap's runs; large allocations are owned by a heap's extents.
- Sharing across threads uses ownership transfer or remote-free coordination, not a shared small/large heap.
- `Heap` live counts track outstanding allocations for abandon/reclaim safety.
