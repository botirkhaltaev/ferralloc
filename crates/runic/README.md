# runic-alloc

`runic-alloc` is Runic's public `GlobalAlloc` wrapper crate.

The package name on crates.io is `runic-alloc`; the Rust library name is `runic`.

## Install

```sh
cargo add runic-alloc
```

## Usage

```rust
use runic::RunicAlloc;

#[global_allocator]
static GLOBAL: RunicAlloc = RunicAlloc::new();
```

Use the const builder for explicit retention policy experiments. Extent policy
controls free-side retention: `Keep` retains a freed mapping while slot and byte
budgets allow it, `Drop` retains nothing. Allocation-side lookup always reuses a
retained mapping by exact length. Cache storage is fixed-size internally, so
configured slot budgets above the implementation cap are clamped.

```rust
use runic::{Budget, ExtentPolicy, RunicAlloc};

#[global_allocator]
static GLOBAL: RunicAlloc = RunicAlloc::builder()
    .extent()
    .policy(ExtentPolicy::Keep)
    .budget(Budget::new(32, 16 * 1024 * 1024))
    .done()
    .build();
```

## Crate Shape

- `src/lib.rs`: public export surface.
- `src/global.rs`: configured `RunicAlloc` implementation of `GlobalAlloc`.
- `src/bin/abort_case.rs`: subprocess binary used by abort tests.
- `tests/`: global allocator smoke and abort-case integration tests.

Most allocator mechanics live in `runic-core`.
