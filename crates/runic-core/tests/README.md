# runic-core/tests

Integration tests for allocator behavior that crosses core entities.

`manual_alloc.rs` exercises allocation, deallocation, alignment, zeroing, realloc preservation, size classes, run-boundary pressure, and deterministic randomized traces.

## Run

```sh
cargo test -p runic-core --test manual_alloc
```
