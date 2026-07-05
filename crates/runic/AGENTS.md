# AGENTS.md

Scope: `crates/runic/`.

- Keep this crate's public API small. The intended public item is `RunicAlloc`.
- The package is `runic-alloc`, but the library crate name is `runic`; preserve that split unless explicitly changed.
- `GlobalAlloc` methods are unsafe boundaries; delegate mechanics to `runic-core::Allocator`.
- Abort-case behavior belongs in subprocess tests, not in-process test harnesses.
- Run `cargo test -p runic` after changes here.
