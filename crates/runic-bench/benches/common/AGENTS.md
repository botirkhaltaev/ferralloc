# AGENTS.md

Scope: `crates/runic-bench/benches/common/`.

- Keep helpers specific to benchmark entry-point wiring.
- Put reusable workload logic in `crates/runic-bench/src/` instead.
- Avoid hidden global state that changes benchmark comparability.
