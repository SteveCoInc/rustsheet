---
id: T-4200
title: "UNIQUE groups texts that hold a comparison cycle by adjacency in SORT's order, not by equality with a kept row (t3211 G4:I4)"
milestone: M7
lane: engine
status: todo
priority: P1
size: S
tier: opus
depends_on: [T-3418]
---


## Goal
UNIQUE (and its `exactly_once` count) gives Excel's rows on texts whose comparison is not transitive, as in
probes/t3211-collation-sort-cycles Calc!G4, H4, I4: `UNIQUE(VSTACK(a̧, a͂, a͂͂))` (Data!A5, A3, A4) keeps three rows,
where ours keeps two because a̧ = a͂͂ pairwise.

## Context
- Split from T-3418, which fitted SORT's order (`functions::util::sort_indices_pairwise`, a natural merge of
  non-descending runs) and owned these three cells.
- Candidate model, checked in T-3418 by a Python simulation over the probed comparison matrix (Calc!L1:S8): sort
  the lines with `sort_indices_pairwise`, group neighbours in that order that compare equal, keep each group's
  first line, output the kept lines in input order; `exactly_once` keeps the one-line groups. It reproduces every
  one-column UNIQUE cell of t3211 whose texts are Data!A1:A8 (G1:G5, G14, G15, H1:H5, H14, H15, I1:I4, I14, I15).
  In G4 the sorted order is a̧, a͂, a͂͂: no neighbours are equal, so three rows.
- Ours (`unique_of`, crates/rustsheet-engine/src/functions/spill_functions.rs) groups by hash bucket, then by
  pairwise equality with the first kept line of the bucket. A change must keep the hash fast path for totals
  (numbers, booleans, texts without cycles) and the work charge (`spend_work`), and must say how multi-column
  lines compare (probe it: two-column rows where one column holds the cycle).
- Probe first (excel-probe skill): larger sets (every order of {A3, A4, A6, A7} and {A4..A8}, as
  probes/t3418-sort-cycle-permutations does for SORT) to confirm the adjacency model before implementing it.

## Acceptance criteria
- [ ] probes/t3211-collation-sort-cycles Calc!G4, H4, I4 pass, with a new probe case backing the model.
- [ ] docs/excel-semantics/coercion.md "Not a total order" states UNIQUE's rule with the evidence.
- [ ] No regressions; `make check` green; reviewers approve.

## Out of scope
- SORT and SORTBY (T-3418), GROUPBY and PIVOTBY (T-4088).
