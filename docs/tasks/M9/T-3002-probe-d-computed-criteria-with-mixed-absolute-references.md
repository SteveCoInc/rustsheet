---
id: T-3002
title: "Probe D* computed criteria with mixed absolute references and evaluation-time references (T-2617 leftovers)"
milestone: M9
lane: research
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Excel evidence decides the D* computed-criterion cases T-2617 left unprobed, so `reads_records` and
`retarget_unqualified_references` rest on evidence everywhere they decide a match.

## Context
- T-2617 (probes/db-computed-criteria-cross-sheet; criteria.md "D* computed criteria and an unknown header in the
  gen-m9 database sweep (T-2258)") showed that a computed criterion's unprefixed references read the database's
  sheet, and that a record matches only when every moved relative reference onto that sheet lies in the database's
  record area (a range must overlap it). Absolute `$B$5` is exempt.
- Unprobed, ours (`reads_records` in functions/database.rs) exempts a reference with an absolute row and a relative
  column (`B$5`), and checks a range with one absolute row by its moved other corner (`SUM(B$5:B6)`, `SUM($B5:B6)`).
- Unprobed, only parsed references are retargeted: `INDIRECT("B2")`, `OFFSET(B2,0,0)`, `ROW()`/`COLUMN()` with no
  argument and a structured `[@Col]` reference in a criterion on another sheet keep the criteria cell's sheet.
- Use T-2617's design: field cells holding 2^(row-1) so a DSUM is the set of matched records, over Same!A5:C12 and a
  Data/Db pair. Implementation detail to revisit when touching the code: the cross-sheet path clones the criterion's
  AST once per record (`Arc::make_mut`); retargeting once per criteria cell would avoid it.

## Acceptance criteria
- [x] A probe case with oracle values for `B$5>0`, `$B5>0`, `SUM(B$5:B6)>0`, `SUM($B5:B6)>0` (same sheet) and
      `INDIRECT("B2")`, `OFFSET(B2,0,0)`, `ROW()`, `[@Col]` conditions (criteria on another sheet) is committed under
      eval/suites/probes.
- [x] criteria.md records the result; ours is changed to match, or a functions-criteria task is filed for it.
- [x] `make check` green; reviewers approve.

## Out of scope
- The rules T-2617 verified.

## Outcome
New probe probes/t3002-db-computed-criteria-absolute-mixed (excel-graph, 2026-11-07; T-2617's bitmask design), recorded in criteria.md under "D* computed criteria and an unknown header in the gen-m9 database sweep (T-2258)":
- Same sheet (Same!A5:C12): an absolute row alone exempts a reference, wherever it points (`B$5`, `B$13`, `B$2`, `E$6` match every record). An absolute column alone does not (`$B5` behaves like `B5`). A range with one absolute corner is checked by where the moved range lies (`SUM(B$5:B6)`, `SUM($B5:B6)` all, `SUM(B$13:B14)` none). Ours already matches every same-sheet cell.
- Criteria on another sheet:
  - `OFFSET(B2,0,0)` is retargeted and follows the record-area rule. `INDIRECT("B2")` reads the criteria sheet. Both match.
  - `ROW()`/`COLUMN()` with no argument are the criteria cell's, and `DbT[@Col]` reads its own table row. Ours differs on both: 16 cells, owned by the new task T-4858 (functions-criteria).
- The T-4742 review follow-up T-4863 (functions-lookup, sonnet) is filed in this PR.

Eval delta: probes +1 case, 95 checked cells, 79 pass; 16 new known failures (Calc B7:E9, B13:E13; `make eval-baseline`, 0 regressions, visible-parity lock +16 on the new case), all owned by T-4858.
