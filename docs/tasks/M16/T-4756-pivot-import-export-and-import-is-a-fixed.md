---
id: T-4756
title: "Pivot import, export and import is a fixed point over generated pivots"
milestone: M16
lane: quality
status: done
priority: P2
size: S
tier: opus
depends_on: [T-4155, T-4176]
---

## Goal
For generated pivots, a save and reload, and a second save and reload, give the same pivot caches, pivot
tables and rendered cells as the first reload: import, export and import is a fixed point.

## Context
- The last property of docs/research/pivot-tables-2026-11-04/plan.md §6 layer 5. T-4155 left it out because the
  pivot export (T-4176, `crates/rustsheet-xlsx/src/export/pivots.rs`) is in the xlsx crate, which the engine's
  test module (`crates/rustsheet-engine/src/test/test_pivot_properties.rs`) cannot reach.
- Reuse T-4155's generator (seeded `Rng` records, a Sum pivot over Data!A1:C(n+1)) rather than copying it. One way:
  move the generator to a `pub` test-support item the xlsx tests can call (check `scripts/check_duplication.py`).

## Acceptance criteria
- [x] A seeded property test in the xlsx crate checks, over 48 or more seeds, that import, export, import equals
      import, export, import, export, import (caches, tables, rendered cells), and runs within the CI time budget.
- [x] Any difference it finds is fixed, or filed as a task with its seed.
- [x] `make check` green; reviewers approve.

## Out of scope
- Oracle uploads (plan §6 layer 4).

## Outcome
`import_export_import_is_a_fixed_point_over_generated_pivots` (crates/rustsheet-xlsx/src/export/pivots/tests.rs)
runs over T-4155's 48 seeds. For each seed, a second save and reload gives the same pivot caches, pivot tables and
Sheet1!A1:Z60 rendered text as the first reload. A guard checks that the first reload still holds the pivot and
renders what was saved, so the test cannot pass on an empty workbook. It found no difference, and runs in about 1 s.
The T-4155 generator (seeded records, the Sum pivot builder, the refresh and the sheet text) and `Rng` moved to
`rustsheet_engine::test_support`. That module is built for the engine's tests and behind a new `test-support`
feature, which only the xlsx crate's dev-dependency turns on. The engine's property tests call it, so the generator
exists once. `Rng::next` is now `next_u64` (clippy's `should_implement_trait` in non-test code), and its two
callers were updated. Eval delta: none (tests only). Also files T-4903, a T-4873 review follow-up.
