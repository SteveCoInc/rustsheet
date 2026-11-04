---
id: T-3985
title: "GROUPBY lifted options beside a spent filter: probe a position holding text or an out-of-range number and an error, or two errors"
milestone: M7
lane: functions-lookup
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
GROUPBY's spent-filter rule for a position where two lifted options (field_headers and total depth) both hold
something unusual is evidenced, and `Model::lift_options`' `settled` closure follows the cells.

## Context
- Follow-up of T-3964 (PR #3036, reviewer MINORs). T-3964 showed one error element of a lifted field_headers or total
  depth stays its error after another element spent the filter (probes/t3964-groupby-filter-lift-errors G!B71, B83,
  B89, B101). With two lifted options the engine models (unprobed) that a position's first error, in argument order,
  wins only when no text element comes before it; any text (numeric text such as "1" included) and an out-of-range
  number before the error leave the spent `#VALUE!`.
- Probe beside `(Data!C1:C12<0.2)`, with field_headers and total depth both lifted (ranges, since an array constant
  cannot hold `NA()`), the error in the second element so the first spends the filter: {numeric text "1", `#N/A`},
  {other text "x", `#N/A`}, {5 (out of range), `#N/A`}, {`#DIV/0!`, `#N/A`}, each in both argument orders, plus
  no-filter controls.
- Also from the T-3964 review: arrays.md's T-3840 bullet says "a `=1/0` element is `#DIV/0!` in either place" (it means
  field_headers or total depth; reword), the T-3778 bullet's new sentence needs re-wrapping, and the closure's
  `find(..).filter(..)` reads better as one `match`.

## Acceptance criteria
- [ ] An excel-graph probe answers each combination; ours matches every cell (change the `settled` closure if not).
- [ ] arrays.md "A lifted GROUPBY's filter serves one element" cites the cells and its Status line no longer lists the
      combinations as unprobed; the wording and wrapping fixes above are made.
- [ ] `make check` green; reviewers approve.

## Out of scope
- PIVOTBY's lift beside a filter column (T-3892).
