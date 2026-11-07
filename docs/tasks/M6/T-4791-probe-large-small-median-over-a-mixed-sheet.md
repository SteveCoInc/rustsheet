---
id: T-4791
title: "Probe LARGE/SMALL/MEDIAN over a mixed-sheet grid whose first element is a value or an error: {own; error; off} and {value; own; off}"
milestone: M6
lane: functions-lookup
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
LARGE, SMALL and MEDIAN over a reference grid on both sheets give the Excel result when the first element is a value or an error, and `RefGrid::order_statistic_error` follows the evidence.

## Context
Review MINORs on T-4714's PR (excel-fidelity). T-4714 made an own-sheet first reference give the value view (`#VALUE!`)
and left every other first element on the any-off-sheet rule. Two grids are unprobed (probes/t4697-large-small-median-
mixed-sheet-refs has neither): `{own ref; error element; off-sheet ref}` (before T-4714 the error element came first;
now `#VALUE!`) and `{value; own ref; off-sheet ref}` (now `#N/A`; "the first *reference* element" would give `#VALUE!`).
Code: `RefGrid::order_statistic_error` (crates/rustsheet-engine/src/reference_operators/areas.rs); notes: lookup.md
"A grid with references on both sheets", SPEC-0014 rule 5. Use the `excel-probe` skill.

## Acceptance criteria
- [ ] A probe case holds both grids (LARGE, SMALL, MEDIAN) with excel-graph values, and lookup.md records the result.
- [ ] `order_statistic_error` and its doc comment (which cites H14/H15 for an error-first grid) match the evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other readers (rule 3, rule 4).

## Outcome
Probe `t4791-large-small-median-mixed-first-element` (12 cells, excel-graph). {own; error; off} is `#VALUE!` and a
value hit first is `#VALUE!` in both orders: all 9 cells already matched, so `order_statistic_error` is right for them.
An INDIRECT `#REF!` element first beside an off-sheet reference is `#N/A` (H4:H6, ours `#REF!`); a cell error value
still propagates (t4704), so the code change is T-4862. Recorded in lookup.md. Caveat: the value-first grids lift
if_not_found from INDIRECT, which may not pair by position, so the value-first rule stays unresolved beyond
"an own-sheet reference gives `#VALUE!`". Doc comment of `order_statistic_error` left as is (its error-first text is
still true for cell errors).
