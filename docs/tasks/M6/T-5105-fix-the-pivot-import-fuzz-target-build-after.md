---
id: T-5105
title: "Fix the pivot_import fuzz target build after render took num_fmts"
milestone: M6
lane: engine
status: in-progress
priority: P0
size: S
tier: sonnet
depends_on: []
issue: 4106
issue_author: github-actions[bot]
---

## Goal
The `pivot_import` fuzz target builds and runs again, so the nightly fuzz run covers pivot
compute, layout and render on untrusted pivot parts (directive 5).

## Context
Issue https://github.com/SteveCoInc/rustsheet/issues/4106 (decision 0302 fuzz finding): the
nightly run 42472677616 failed at the build step of `pivot_import`, saving no input. The job log
shows `error[E0061]: this function takes 5 arguments but 4 arguments were supplied` at
`fuzz/fuzz_targets/pivot_import.rs:102`: T-4717 (#4066) added a `num_fmts: &[NumFmt]` parameter
to `rustsheet_engine::pivot::render`, and the fuzz target was not updated (CI does not build the
fuzz crate). The engine's own caller (`pivot/grid.rs`) passes `&workbook.styles.num_fmts`.

The same run's `formula_eval` failure (a panic in `model.rs`) is a different cause and target and
is not this task.

## Acceptance criteria
- [ ] `fuzz/fuzz_targets/pivot_import.rs` passes the imported workbook's `styles.num_fmts` to
      `render`, like the engine's own caller, so custom number formats reach the fuzzed path.
- [ ] `cargo check --manifest-path fuzz/Cargo.toml --bin pivot_import` builds.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The `formula_eval` crash in the same run.
- Building the fuzz crate in PR CI (a protected-path change).
