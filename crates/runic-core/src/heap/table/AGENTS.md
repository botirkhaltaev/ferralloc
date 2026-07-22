# AGENTS.md

Scope: `crates/runic-core/src/heap/table/`.

- Keep TLS heap-entry owner-local frontend state and run caches here.
- `Inbox` is a movable Treiber-style head of intrusive `RemoteList` batches; construct with `Inbox::new()`.
- `RemoteList.first`/`.last` are plain `NonNull<u8>`, not `Option`: a list is only ever built from a non-empty batch, so construction (`RemoteList::from_ends`) and `Inbox::push_batch` never need to check or `expect` non-emptiness.
- Remote frees use batched transport: `RemoteBatch` on `ThreadHeap` coalesces; `Inbox::push_batch` / `drain` move `RemoteList`s. Do not add single-node push/pop façades.
- Create heaps with `Heap::new` + `Arena::claim` / `insert` (inbox is movable; no placement-only install).
- Keep `Heap` responsible for Free/Active/Draining mode and owner-local lifecycle helpers; `Heap::mode()` returns the `HeapMode` snapshot directly (Free/Active/Draining) for callers that must branch on lifecycle state, instead of separate `is_active`/`is_draining` bools composed ad hoc into a route struct.
- Keep `HeapTable` responsible for slot identity, `generations[]` ABA checks, `push_remote_batch`, and reclaim generation bumps.
- `HeapTable::push_remote_batch` is mode-aware under the table lock: `Active` enqueues to the inbox; `Draining` enqueues then `flush`es (completing claimed nodes) and may reclaim; `Free`/stale generation fails. Retained TLS batches must stay publishable after owner exit.
- Do not put `HeapTable` on steady-state owner-local allocation hot paths.
- Clear or validate owner-local caches whenever a heap is abandoned or reactivated.
- Preserve explicit separation between owner-local frees and remote-free claim→batch→inbox→drain behavior. There is exactly one remote-free protocol (`Allocator`'s slow dealloc path claims, coalesces onto the TLS `RemoteBatch`, and calls `push_remote_batch`; direct `AllocatorState::dealloc_*_draining` only handles frees whose mode snapshot is already Draining). Do not add a second, unbatched remote-free implementation for `realloc` or any other caller — route all cross-heap frees (including from `realloc`) through the same `Allocator::dealloc` path.
- Do not introduce passive forwarding wrappers for heap table behavior; prefer methods on `HeapTable`, `Heap`, or the TLS heap entry that owns the state.
