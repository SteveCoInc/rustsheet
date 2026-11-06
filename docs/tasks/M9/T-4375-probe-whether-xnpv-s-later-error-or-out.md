---
id: T-4375
title: "Probe whether XNPV's later error or out-of-range date beats a refused first value's #VALUE!"
milestone: M9
lane: functions-financial
status: done
priority: P3
size: S
tier: sonnet
depends_on: [T-3815]
---

## Goal
Evidence for which error XNPV gives when its first value is no number (empty, text, boolean) and a later element is
an error value, an error date, or a later date that is negative or past 2958465, and the engine matching it.

## Context
- Follow-up of T-3815 (Excel-fidelity review). T-3815's element walk (`Model::xnpv_walk_error`,
  crates/rustsheet-engine/src/functions/financial/flows.rs) runs only when the first value is a number: every row of
  probes/t3815-xnpv-element-reading has one. Behind a refused first value the engine keeps T-2603's `#VALUE!`
  (`Model::xnpv_first_value_refused`, financial/mod.rs), which is evidenced only for later empty, 0 and
  before-the-first dates (probes/t2603-xnpv-no-number-values C1, C7) and later text or boolean dates (`#NUM!`).
- financial.md, "XNPV is #VALUE! when the first values cell is no number (T-2603)" and "XNPV reads its values and
  dates element by element (T-3815)", list the combinations as Unprobed.
- Probe (excel-probe skill, excel-graph): values `<"x",30,#N/A,50>`, `<,30,40,50>` and `<TRUE,30,40,50>` against
  good dates; the same values against dates with a `#DIV/0!`, a -1 and a 2958466 at index 1 and at index 3; each as
  a range, an array constant and a computed array (extend docs/research/t3815/gen_case.py or write a new case).
- Also probe (T-3815 fidelity review round 2, MINOR), so the T-3815 entry's remaining inferred lines get cells:
  - a number-first walk against a later 0, empty or before-the-first date together with a later error value or
    non-number value (`XNPV(0.1,{-100;30;#N/A;50},{39448;0;39600;39700})`: the engine gives `#N/A`; T-2603 C1/C7
    only evidence those dates behind a refused first value);
  - `xnpv_one_cell` (flows.rs) through a one-cell range (`D!A2:A2`), `OFFSET`/`INDEX` returning one cell and a
    defined name for one cell, holding an error, numeric text and date text (every D-column cell of
    probes/t3815-xnpv-element-reading is a plain reference).

## Acceptance criteria
- [x] The probe case is filled on excel-graph and passes, the cases above included; financial.md states the rule and
      drops the Unprobed lines.
- [x] If Excel lets any of these beat the `#VALUE!`, the walk follows it (one walk, no second reader).
- [x] `make check` green; reviewers approve.

## Out of scope
- XIRR (T-3816); a first value that is a number (T-3815, evidenced).

## Outcome
Probe probes/t4375-xnpv-refused-first-and-one-cell filled on excel-graph (114 cells). Behind a refused first value
(text, empty, TRUE) the `#VALUE!` beats every later error value, error date and bad date: already the engine's
reading, now evidenced. Two engine differences found and fixed in `flows.rs`: with a number first, a later date of 0,
empty or before the first is `#NUM!` at its own index (before a later error value; the walk visits it), and a
one-cell `INDEX` or defined name holding numeric or date text reads as its number (`xnpv_one_cell`). The probe passes
114/114. financial.md states the rules and drops the Unprobed lines (a literal text argument stays unprobed).
