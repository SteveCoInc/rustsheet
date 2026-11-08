---
id: T-5068
title: "Probe the 16th-digit tie direction of number-cell text, the scientific mantissa, comma-scaled fractions and pivot number groups (T-4848 follow-up)"
milestone: M1
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Each remaining unprobed caller of `decimal_round::cut_to_15_digits` breaks an exact 16th-digit tie the way Excel does, on oracle evidence.

## Context
T-4848 made `cut_to_15_digits(value, tie_away)` the one 15-digit cut and probed the tie direction of comparisons, EXACT, `&`,
`TEXT(,"0.0")`, IM* parts and fraction integer digits (probes/t4848-sixteenth-digit-ties, t4848-tie-complex-part,
t4848-tie-fraction-format). numbers.md "A 16th-digit tie splits by use" still lists these as UNVERIFIED:
- `to_excel_precision_str` (number_format.rs; cell text in cell.rs, formula-literal stringify, sequence detector): toward zero now.
  A computed tie such as `=1E+15+15` read back through a cell-text API, or a pasted value, could show it.
- The scientific-format mantissa (`format.rs`, after `round_scientific`): toward zero now. The T-4848 probes reach no tie there.
- The comma-scaled fraction value (`fraction.rs`, `part.comma > 0`): away from zero now, by analogy with the integer digits.
- Pivot number-group quotients (`pivot/number_groups.rs`, `quotient` and `inner_count`): toward zero now. Grouping by
  an interval whose quotient is an exact tie, such as start 0, interval 1 and a value of 1E+15+15, would show it.
- Conditional-format Equal: it shares `cmp_excel_precision` (away). It needs a CF probe.

## Acceptance criteria
- [ ] Probe cases (excel-graph) for each item above that can be reached. Each caller's tie flag matches the evidence, and numbers.md is updated; an item no probe can reach is noted as unreachable.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed probes/t5068-tie-number-text-mantissa (15 cells, excel-graph): the scientific mantissa cuts a 16th-digit tie toward
zero, as the engine already does (no code change; eval delta 0). The other four items cannot be reached by any oracle backend
(reasons in numbers.md "A 16th-digit tie splits by use"), so they are noted as unreachable and keep their current direction.
