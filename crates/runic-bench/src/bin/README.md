# runic-bench/src/bin

Benchmark support binaries.

`rss` runs isolated allocator/workload combinations and reports resident-set size. It uses fresh subprocesses so one allocator's cached memory does not affect another row.

## Run

```sh
cargo run -p runic-bench --bin rss
```
