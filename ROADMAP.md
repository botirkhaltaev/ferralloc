# Ferralloc Roadmap

## Thesis

Ferralloc exists because Rust should have a serious Rust-native hosted allocator with a small auditable unsafe core, out-of-line metadata, explicit span invariants, and a clean path toward thread-local heaps, remote frees, hardening, and hugepage-aware allocation.

Ferralloc is not a line-for-line port of mimalloc, jemalloc, TCMalloc, snmalloc, or another allocator. It should learn from existing allocators while keeping Rust-native invariants explicit and testable.

The useful claim is not:

```text
Ferralloc is safe because it is written in Rust.
```

The useful claim is:

```text
Ferralloc reduces and audits the unsafe core, encodes allocator invariants explicitly, and makes allocator correctness testable before adding performance layers.
```

## Current Milestone

```text
A global-lock Rust allocator that can run real Rust programs and survive randomized allocation traces.
```

Correctness comes before speed.

## v0.1 Scope

Build only:

```text
Linux x86_64
Rust stable
GlobalAlloc
one global lock
mmap-backed spans
small allocations via size classes
large allocations via direct mmap
out-of-line metadata
page-indexed pointer-to-span lookup
block boundary checks
basic realloc
basic alloc_zeroed
randomized tests
```

Do not build yet:

```text
profiles
thread-local heaps
remote frees
quarantine
canaries
hugepages
NUMA
C ABI
LD_PRELOAD
per-CPU caches
ML/lifetime placement
stats dashboard
```

## Core Invariant

```text
Every pointer returned by ferralloc belongs to exactly one span,
every span owns blocks of exactly one size class,
and every free must map back to a known span and block boundary.
```

If this invariant is wrong, later features like thread-local heaps and remote frees will hide bugs. If it is correct, the allocator can be made fast later.

## Architecture

```text
GlobalAlloc
  -> Ferralloc
      -> Allocator
          -> State
              -> SpanMap
              -> SpanTable
              -> Span
                  -> FreeList
              -> OsMemory
```

Use one global lock around `State`.

## Entity Responsibilities

```text
Ferralloc    owns the Rust GlobalAlloc boundary
Allocator    owns the core public allocator API
State        owns allocation policy and global lock-protected state
LayoutSpec   owns normalized layout semantics
SizeClasses  owns size-class selection
OsMemory     owns mmap and munmap
Span         owns small-block and large-allocation metadata
FreeList     owns the intrusive free-block chain
SpanTable    owns out-of-line span metadata storage
SpanMap      owns page-indexed pointer-to-span lookup
```

## Workspace

```text
crates/ferralloc-core
  allocator mechanics and global state

crates/ferralloc
  public GlobalAlloc wrapper

crates/ferralloc-test-support
  reusable future test machinery

crates/ferralloc-bench
  future benchmark harness
```

## Reference Lessons

Use `allocator-refs/` as read-only inspiration:

- linked-list-allocator: minimal Rust `GlobalAlloc` shape, size/alignment matrix tests, free-order tests.
- talc: Rust-native allocator structure and high-alignment regression testing.
- ferroc: randomized allocation traces, fuzz-style action sequences, zeroed allocation checks, cookie validation.
- mimalloc: future span/page-local free-list design and locality lessons.
- TCMalloc: future frontend/middle/backend layering and size-class/span invariant tests.
- snmalloc: future remote-free/message-passing design.
- PartitionAlloc, Scudo, hardened_malloc: later out-of-line metadata and hardening work.
- mimalloc-bench: later workload and benchmark ideas.

Do not copy reference implementation code.

## Current Test Shape

Default tests should cover:

```text
layout normalization and overflow checks
size-class alignment invariants
free-list LIFO behavior
mmap mapping and writability
span block uniqueness and boundary checks
span table reservation, insertion, mutation, removal
span map lookup, removal, overlap rejection, L2 boundary crossing
small and large allocation paths
alignment matrices
alloc_zeroed
realloc prefix preservation
subprocess abort cases
Box, Vec, String, HashMap, Arc smoke tests
deterministic randomized allocation traces
```

Abort tests must run in subprocesses, not inside the test harness process.

## Known Follow-Ups

Track these as GitHub issues instead of expanding v0.1 scope:

```text
Improve SpanMap metadata allocation.
Add block-state tracking for double-free detection.
Revisit SpanTable test/production capacity differences.
```

## Later Milestones

```text
v0.2 block-state tracking
  Detect double frees and make block state explicit.

v0.3 empty span reuse and release policy
  Reuse empty spans first, release later after a simple threshold.

v0.4 thread-local heaps
  Add per-thread fast paths only after span invariants are stable.

v0.5 remote frees
  Add owner heap IDs and snmalloc/mimalloc-inspired remote handling.

v0.6 hardening
  Encoded freelists, canaries, quarantine, and guard-page ideas.

v0.7 hugepage-aware backend
  Explore 2 MiB segment packing and hugepage coverage later.
```
