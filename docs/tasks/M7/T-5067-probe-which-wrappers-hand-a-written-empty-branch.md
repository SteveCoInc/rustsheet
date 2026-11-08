---
id: T-5067
title: "Probe which wrappers hand a written-empty branch on as an omission (IFNA, IFS, SWITCH, nesting, LET, arrays) for T-4912"
milestone: M7
lane: functions-math
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Establish which wrappers besides IF, CHOOSE and IFERROR hand a written-empty branch on as an omission, and whether
the chosen branch or the wrapper's shape decides, so T-4912 can be designed on evidence (its AC 2).

## Context
- T-4901's probe showed IF, CHOOSE and IFERROR doing it (probes/t4901-mround-blank-let-lambda-roundings). T-4912's
  AC 2 asks for the other wrappers. The design question is whether a blank value on the chosen branch is told
  apart from a written-empty one. Our `CalcResult::EmptyArg` covers both: IFERROR and VLOOKUP return it for a blank.

## Acceptance criteria
- [x] A probe of IFNA, IFS, SWITCH, nesting, parentheses, LET, an operator, an array condition, blank and lookup
      branches beside a written-empty one, and the other readers (SQRTPI, FACTDOUBLE, EDATE, SERIESSUM, QUOTIENT,
      FLOOR.PRECISE, CEILING.MATH), filled on excel-graph.
- [x] The rule is recorded in numbers.md, and every visibly-off cell has an owner (T-4912).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The engine change (T-4912).

## Outcome
- New case probes/t5067-written-empty-branch-wrappers (32 cells, excel-graph, 2026-11-08). 15 pass and 17 are off.
  T-4912 owns the 17 off cells, and they are recorded in eval/baseline with `make eval-baseline NEW_CASES=...`.
- numbers.md has a new entry, "Which wrappers hand a written-empty branch on as an omission (T-5067)":
  - IF, CHOOSE, IFERROR and IFNA hand it on, nested, in parentheses or through a LET name.
  - IFS, SWITCH and an operator read it as 0.
  - The chosen branch decides: a blank there is 0 even beside a written-empty branch.
  - An array IF reads it as 0 per element.
- So the omission is a value of its own in Excel. T-4912's design is amended: it needs a marker that is not
  `EmptyArg`, not a node-only look-through.
- Eval delta: 0 fixed, 1 new case with 17 owned visibly-off cells, 0 regressions.
- Also fixed a stale "below" in numbers.md's 2010+ roundings entry (a design-review MINOR on #3963).
