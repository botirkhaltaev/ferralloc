# Small Allocation Cache Design Notes

Issue: #16

Runic should eventually have a small-allocation front-end cache, but the first implementation must improve hot workloads without weakening run block-state invariants.

## Rejected Designs

### Cache Allocated Blocks Outside User Visibility

One attempted shape refilled a per-size-class cache by allocating blocks from runs and holding them as already allocated until a future user allocation popped them.

This is not correct. If user code keeps a stale pointer and frees it while the cache owns that block, run block-state would still report the block as allocated and the stale free could be accepted instead of detected as a double free.

### Cache Freed Blocks On Deallocation

Another attempted shape moved validated frees into a small cache instead of returning directly to run state. This preserves double-free detection only if cached blocks remain represented by exactly one owner.

Measured locally, that added state transitions and cache management on every free and regressed the target workloads:

- `explicit/single_size_churn/runic/64`: about `15.7 us`, slower than the current `13-14 us` range.
- `explicit/small_biased_random/runic`: about `118.5 us`, slower than the available-run-list branch result around `112 us`.

### Refill Cache With Cached Block State

A safer version added a cached block state so cached blocks were not user-visible allocations. This preserved stale-free detection, but the extra state transitions and run lookups were still slower:

- `explicit/single_size_churn/runic/64`: about `19.7 us`.
- `explicit/small_biased_random/runic`: about `145 us`.

## Required Invariants

Any future cache must preserve these rules:

- A block held by an internal cache must not be reported as a live user allocation.
- A stale free of a cached block must still fail as a double free or invalid free.
- Cached blocks must not also be available in the run bitmap.
- A run should be linked as available only when its owned reusable state can satisfy an allocation, not merely because cached blocks exist elsewhere.
- The cache must not allocate internally.

## Direction

The next viable design should reduce both allocation and deallocation hot-path work. A cache that only shifts blocks between run metadata and a small array is not enough.

Promising directions:

- Add a block-state representation that models user-allocated, run-reusable, and cache-owned blocks without extra bitmap passes.
- Pair the cache with a validated fast deallocation path, or defer this work until thread-local heap ownership can make cache hits avoid global metadata work.
- Re-benchmark against `single_size_churn_64`, `small_biased_random`, and abort tests before accepting any implementation.

Until then, Runic should keep the simpler available-run-list design from #6 rather than merging a cache that regresses the target workloads.
