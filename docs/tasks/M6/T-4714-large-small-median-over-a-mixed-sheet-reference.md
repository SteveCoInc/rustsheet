---
id: T-4714
title: "LARGE/SMALL/MEDIAN over a mixed-sheet reference grid: the first element's sheet decides #VALUE! vs #N/A"
milestone: M6
lane: functions-lookup
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
LARGE, SMALL and MEDIAN over an array of references on both the formula's sheet and another give `#VALUE!` when the
first reference element is on the formula's own sheet and `#N/A` when it is on another sheet.

## Context
Evidence: T-4697, probes/t4697-large-small-median-mixed-sheet-refs; SPEC-0014 rule 5; lookup.md. Today
`RefGrid::order_statistic_error` (T-4570) answers `#N/A` when any reference is on another sheet.
Owner of the visibly-off cells Local!H1, H3, H5, H9, H17 of that case.

## Acceptance criteria
- [x] The five cells match Excel (`#VALUE!`), H2, H4, H6:H8, H13:H16 keep passing, their baseline lines removed (`make eval-baseline`).
- [x] `make check` green; reviewers approve.

## Out of scope
- Error elements in the grid, and other readers (rule 3, rule 4).

## Outcome
`RefGrid::order_statistic_error` returns the value view (`#VALUE!`) when the first element is a reference on the formula's own sheet; otherwise the old rule (error element, else `#N/A` when any reference is off-sheet) stands, which H14 and H15 (first element a value) still pass. Eval delta: `probes/t4697-large-small-median-mixed-sheet-refs` 12 -> 17 of 17; five known failures (Local H1, H3, H5, H9, H17) removed from the baseline. Unit test added; SPEC-0014 and the doc comment updated.

## Landing notes
- PR #3847, head `972df1a7c` (after `make catch-up`). CI green, security, design and excel-fidelity approved on the code head `252b7f880`; the catch-up merged `origin/main` only.
- The merge queue removed it (`MERGE_CONFLICT`) at 05:46Z after it was enqueued at 05:43Z: another PR merged ahead of it and touched `eval/baseline/probes.txt` again.
- Next step: run `make catch-up PR=3847` again, wait for green CI, then `enable_pr_auto_merge` (SQUASH). No reviewer re-run is needed for a clean merge of `origin/main` or a catch-up.
- Do not enqueue before CI is green on the new head. The first enqueue silently failed on the same baseline conflict, with no `enqueued` event.
