# AGENTS.md

Scope: `crates/runic-core/src/memory/`.

- Keep OS mapping lifecycle in `OsMemory` and `Mapping`.
- Keep pointer lookup and ownership publication in `PageMap`.
- Preserve `PageOwner` pointer lifetime assumptions: owners stay live until their page-map range is removed.
- Keep unsafe pointer/provenance code narrow and adjacent to safety comments.
- Add page-map tests for overlap, removal, L2 boundary, and span behavior when changing lookup logic.
