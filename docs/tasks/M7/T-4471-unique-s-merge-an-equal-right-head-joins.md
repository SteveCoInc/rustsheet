---
id: T-4471
title: "UNIQUE's merge: an equal right head joins the left head, which stays; no coalescing with the last group out (T-4393's model, 60/60 and hold-out 60/60)"
milestone: M7
lane: engine
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
UNIQUE (and the GROUPBY/PIVOTBY key lists built on its kept rows) keeps the rows Excel keeps on intransitive runs,
by T-4393's merge model.

## Context
- T-4393 (docs/research/t4393/model.py `unique_kept`; docs/excel-semantics/coercion.md "UNIQUE's merge: equal heads
  join, the output never coalesces"): a natural merge sort over groups. A row `=` the last group of its run joins
  it; at a merge, an equal right head joins the left head's group, which stays the head and meets the next right
  head too; a group taken out is never compared with the last group out. Chosen on t4379 and t4380 (60 of 60 runs),
  confirmed on probes/t4393-unique-merge-holdout (60 of 60 UNIQUE predictions committed before the fill).
- The engine's `coalesce_equal_lines` (crates/rustsheet-engine/src/functions/util/pairwise_order.rs) is T-4200's
  rule: equal heads leave together and every push coalesces with the last group out. The change is to its merge
  step only; the run building stays.
- The model was checked on two-field runs (t4379, t4380, t4393) and on docs/research/t4241/model.py's 234 GROUPBY
  blocks (t4195, t4241; unchanged). Run the full probes suite: one-field and combining-mark cases (t4200, t4237,
  the `{a͂;a̧;a͂͂}` rows of pairwise_order.rs's doc comment) must not regress; if one does, record it and stop.
- probes/t4393-unique-merge-holdout Calc!B1:B60 (UNIQUE; the engine misses 54 of them).
- Owned cells (from T-4393): probes/t4379-pivotby-repeated-unreached-leaf (20, moved from T-4380): Calc!B1, Calc!C1, Calc!D1,
  Calc!E1, Calc!B2, Calc!C2, Calc!D2, Calc!E2, Calc!B8, Calc!C8, Calc!D8, Calc!E8, Calc!B9, Calc!C9, Calc!D9, Calc!E9,
  Calc!B10, Calc!C10, Calc!D10, Calc!E10; probes/t4358-pivotby-unreached-column-leaf (2): Calc!B6, Calc!D6;
  probes/t4380-unique-keep-head-holdout (50): Calc!B1, Calc!B2, Calc!B6, Calc!B7, Calc!B8, Calc!B9, Calc!B10, Calc!B11, Calc!B12, Calc!B13, Calc!B15, Calc!B16, Calc!B17, Calc!B18, Calc!B20, Calc!B22, Calc!B23, Calc!B24, Calc!B25, Calc!B26, Calc!B27, Calc!B28, Calc!B29, Calc!B30, Calc!B31, Calc!B32, Calc!B33, Calc!B37, Calc!C1, Calc!C2, Calc!C6, Calc!C7, Calc!C8, Calc!C9, Calc!C11, Calc!C12, Calc!C13, Calc!C15, Calc!C16, Calc!C18, Calc!C20, Calc!C22, Calc!C23, Calc!C24, Calc!C27, Calc!C29, Calc!C30, Calc!C31, Calc!C32, Calc!C37.

## Acceptance criteria
- [ ] `coalesce_equal_lines` follows T-4393's model; unit tests on Cy Cx Ay Bx Ay (Cy, Cx, Bx, Ay) and Cy Cx Ay Bx Cy
      (Cy, Cx, Bx).
- [ ] probes/t4393-unique-merge-holdout column B passes; t4379's and t4380's UNIQUE column B passes; no regression.
- [ ] GROUPBY/PIVOTBY cells that change are recorded. Owned cells still off afterwards pass to T-4472 (GROUPBY's
      tree): move them into its Owned cells line in this PR.
- [ ] `make check` green; reviewers approve.

## Out of scope
- GROUPBY's tree and descent (T-4472).
