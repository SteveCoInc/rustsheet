---
id: T-3848
title: "criteria.md: cite the NBSP zero cells as not read by VALUE, not as #VALUE!"
milestone: M2
lane: functions-criteria
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The padded-zero paragraph of docs/excel-semantics/criteria.md ("A blank-cell or omitted criterion is the criterion 0")
claims only what its cells show about VALUE on the two CHAR(160)-padded zero texts.

## Context
- T-3807 review (excel-fidelity, MINOR): the paragraph says VALUE reads CHAR(160)&"0" and "0"&CHAR(160) "as
  `#VALUE!` (L4, L5)", but probes/t3807-blank-criterion-padded-zero L4:L5 are `IFERROR(VALUE(A4),"err")` and K4:K5
  `ISNUMBER(VALUE(A4))`, so those cells show only that VALUE does not return a number. The `#VALUE!` itself is
  evidenced for NBSP-padded text by probes/t3334-criteria-operand-forms H47, H51, H55, H59 (criteria.md, T-3404).
- Docs only: no case, engine or baseline change.

## Acceptance criteria
- [ ] The sentence reads "which VALUE does not read as a number (K4, K5, L4, L5)" or equivalent, and cites the
      t3334 cells if it keeps the word `#VALUE!`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- New probes (a bare `=VALUE(A4)` cell would settle the error code, but the t3334 cells already do).

## Outcome
- criteria.md now says the two CHAR(160)-padded zero cells are ones VALUE does not read as a number (K4, K5, L4, L5)
  and cites t3334 H47, H51, H55, H59 for the `#VALUE!` itself. Docs only; no eval delta.
