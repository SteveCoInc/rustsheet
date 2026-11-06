---
id: T-4393
title: "Find UNIQUE's grouping on two-field intransitive runs: the t4379 and t4380 probes no coalescing merge fits"
milestone: M7
lane: research
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A model of which rows Excel's UNIQUE keeps on two-field runs of the intransitive texts A = ab, B = ab&U+00AD,
C = U+00AD&ab (A = C, B = C, A < B), chosen on probes/t4379-pivotby-repeated-unreached-leaf and
probes/t4380-unique-keep-head-holdout and scored on a fresh hold-out probe (predictions committed before the fill),
or the negatives recorded and the cells parked with their reason.

## Context
- T-4380 (2026-11-05) chose a merge rule on t4379 (equal run heads: the right group joins the left head, which
  stays the head) and the t4380 hold-out refuted it: 12 of 45 runs miss. docs/research/t4380/candidates.py scores
  candidate rules on the 60 runs of both probes: T-4200's rule (the engine's `coalesce_equal_lines`) 33 misses, the
  refuted rule 12, `pushkeep` 5, the right group as head 37 to 39, insertion sort then grouping neighbours 37 to 44.
- docs/excel-semantics/coercion.md, "No coalescing merge explains UNIQUE's two-field runs": over Cy Cx Ay Bx Ay
  Excel keeps the second Ay apart from the first, so two identical rows end in different groups. No
  sort-then-group-neighbours model under the row-by-row field comparison does that. Leads: how Excel compares two
  rows (all fields at once, or a per-field pass, or a hashed key), and whether UNIQUE sorts at all. One-field runs
  (t4200, t4237) never tell the rules apart.
- Any model must still give every cell of t4195's, t4200's, t4237's, t4241's, t4342's, t4344's and t4358's probes
  (docs/research/t4241/model.py scores 234 blocks); GROUPBY and PIVOTBY build their key trees on UNIQUE's kept rows.
- Owned cells: probes/t4379-pivotby-repeated-unreached-leaf (20, moved from T-4380): Calc!B1, Calc!C1, Calc!D1,
  Calc!E1, Calc!B2, Calc!C2, Calc!D2, Calc!E2, Calc!B8, Calc!C8, Calc!D8, Calc!E8, Calc!B9, Calc!C9, Calc!D9, Calc!E9,
  Calc!B10, Calc!C10, Calc!D10, Calc!E10; probes/t4358-pivotby-unreached-column-leaf (2): Calc!B6, Calc!D6;
  probes/t4380-unique-keep-head-holdout (50): Calc!B1, Calc!B2, Calc!B6, Calc!B7, Calc!B8, Calc!B9, Calc!B10, Calc!B11, Calc!B12, Calc!B13, Calc!B15, Calc!B16, Calc!B17, Calc!B18, Calc!B20, Calc!B22, Calc!B23, Calc!B24, Calc!B25, Calc!B26, Calc!B27, Calc!B28, Calc!B29, Calc!B30, Calc!B31, Calc!B32, Calc!B33, Calc!B37, Calc!C1, Calc!C2, Calc!C6, Calc!C7, Calc!C8, Calc!C9, Calc!C11, Calc!C12, Calc!C13, Calc!C15, Calc!C16, Calc!C18, Calc!C20, Calc!C22, Calc!C23, Calc!C24, Calc!C27, Calc!C29, Calc!C30, Calc!C31, Calc!C32, Calc!C37.

## Acceptance criteria
- [ ] A model scored on t4379 and t4380 with a fresh hold-out probe (predictions committed before the fill), its
      result in the Outcome; or the candidates tried and their scores in coercion.md and the owned cells parked
      with that reason.
- [ ] If a model fits, an engine task to implement it is filed and takes the owned cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (the implementing task does that).
