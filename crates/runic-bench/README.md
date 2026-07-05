# Runic Benchmarks

This crate contains Runic's allocator iteration benchsuite.

## Timing Benchmarks

Run all Criterion benchmarks:

```sh
cargo bench -p runic-bench
```

Run one benchmark target:

```sh
cargo bench -p runic-bench --bench explicit
cargo bench -p runic-bench --bench threaded
cargo bench -p runic-bench --bench global_runic
```

## Benchmark Families

- `explicit`: calls each allocator through `GlobalAlloc` directly and compares Runic, System, mimalloc, jemalloc, and snmalloc in one Criterion report.
- `threaded`: exercises global-lock contention and cross-thread frees.
- `global_*`: uses one process-global allocator per binary for real Rust collection workloads.

The default Criterion settings are intentionally developer-sized. They are meant to make allocator changes easy to compare during iteration, not to replace a long dedicated benchmarking run.

## RSS Report

Run the footprint smoke report:

```sh
cargo run -p runic-bench --bin rss
```

The RSS runner spawns a fresh subprocess per allocator/workload pair so rows do not inherit cached memory from earlier allocator runs.

## Validation

Benchmark workloads touch allocated memory, validate returned alignment, and check sampled realloc prefix preservation. Full byte-for-byte randomized correctness remains covered by the allocator test suite; benchmark validation is sampled so timing is not dominated by memory scanning.
