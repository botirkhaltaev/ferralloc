# runic/tests

Integration tests for the public global allocator crate.

## Files

- `global_smoke.rs`: verifies common Rust standard-library types allocate successfully under `RunicAlloc`.
- `abort_cases.rs`: spawns `abort_case` subprocesses for invalid-free and invalid-realloc cases.

## Run

```sh
cargo test -p runic-alloc
```
