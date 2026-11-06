---
id: T-4124
title: "Score rendered pivot areas and rowItems against the oracle in the eval"
milestone: M16
lane: harness
status: done
priority: P1
size: M
tier: opus
depends_on: [T-4123]
---

## Goal
The eval scores every captured pivot cell (value, then displayed text) and compares the engine's location and rowItems/colItems with Excel's, reporting both per case and in report.md; failures enter the baseline ratchet like any cell.

## Context
Depends on the `pivots` sidecar section. Until the engine renders pivots, these cells are known failures, recorded in eval/baseline through `make eval-baseline` (never hand-edited). The structural comparison localizes layout bugs before cell values. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [x] Pivot-area cells count as scored cells of their suite; a missing render is a failure, not a skip.
- [x] A structural mismatch is listed per case in report.md (`pivot layout` section).
- [x] `make check` green; reviewers approve.

## Out of scope
- Making the engine render (PV1 tasks).

## Outcome
- **Scoring:** a spec case's sidecar pivot cells are scored Excel cells (`pivot_scoring::add_expectations`
  from `prepare_spec`), blanks as `empty`. An unrendered pivot fails cell by cell. The cells of
  probes/t4123-pivot-capture and t4128-pivot-spike are now known failures in eval/baseline/probes.txt,
  recorded with `make eval-baseline`.
- **Layout:** each sidecar pivot is compared with the engine's, through a doc-hidden harness accessor,
  `rustsheet::Workbook::pivot_places`. The verdicts are `Missing`, `LocationDiffers` and `LinesNotEmitted`.
  `rowItems`/`colItems` are compared once T-4147 makes the engine emit them. Results go to report.json
  (`pivot_layout` per case) and report.md ("Pivot layout").
- **Refactor:** the cell scorer moved from runner.rs to runner/scoring.rs to keep runner.rs under the
  file-size limit, and `spec_formulas` was extracted.
- **Limits:**
  - xlsx cases' sidecar pivots are not scored yet; T-4131 covers them.
  - Displayed text of pivot cells is not scored either: sidecars do not capture it yet, which is
    T-4169's (T-4123 Outcome).
