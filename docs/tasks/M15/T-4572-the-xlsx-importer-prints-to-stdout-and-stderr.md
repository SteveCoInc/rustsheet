---
id: T-4572
title: "The xlsx importer prints to stdout and stderr (Unexpected feature, skipped pivot cache): report through diagnostics instead"
milestone: M15
lane: engine
status: blocked
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The xlsx importer, a library, writes nothing to stdout or stderr. What it now prints goes into the import's
diagnostics, or is dropped where it carries no information:
- "Unexpected color node", "Unexpected type", "Invalid type" (`println!`);
- "skipped pivot cache" (`eprintln!`).

## Context
Found by the T-4559 round-trip runs over Calipers and POI: `crates/rustsheet-xlsx/src/import/util.rs:101`,
`worksheets.rs:444,486,511` and `pivots.rs:174` print from library code. `import/styles.rs` also prints "Unexpected pattern" and calls `dbg!(feature)` on an unknown fill
 child (found by T-4856; decision 0335 locks it as lint debt: `clippy::dbg_macro`, `print_stdout`). A caller's output, such as
`rustsheet-eval roundtrip` piped to `head`, gets polluted, and a closed pipe made the worker panic ("failed printing
to stdout") on calipers `rich_text` and `i18n_hebrew`. Workspace lints forbid `print_stdout` in the harness, and the
importer should hold the same rule. Check whether a diagnostics channel exists (the pivot `skip` reason path) and
reuse it (directive 3).

## Acceptance criteria
- [ ] No `println!`, `eprintln!` or `dbg!` in `crates/rustsheet-xlsx/src` outside tests and bins, with clippy's
      `print_stdout` and `print_stderr` denied for the crate.
- [ ] A test loads a workbook with an unexpected colour node and a skipped pivot cache, and gets their reasons through
      the diagnostics.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing what is imported.

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275, 0306): M15 waits until Phase 1 parity exits, except for the task the
owner-directed session claims. Unblock when docs/status.md has the line `**Phase 1:** exited <date>`, or by claiming
it in that session.
