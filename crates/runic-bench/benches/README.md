# runic-bench/benches

Criterion benchmark entry points.

## Targets

- `explicit`: calls each allocator through `GlobalAlloc` directly in one Criterion report.
- `threaded`: exercises global-lock contention and cross-thread frees.
- `global_runic`: process-global Runic allocator workloads.
- `global_system`: process-global system allocator workloads.
- `global_mimalloc`: process-global mimalloc workloads.
- `global_jemalloc`: process-global jemalloc workloads.
- `global_snmalloc`: process-global snmalloc workloads.
- `common`: shared benchmark target setup.

## Run

```sh
cargo bench -p runic-bench
cargo bench -p runic-bench --bench global_runic
```
