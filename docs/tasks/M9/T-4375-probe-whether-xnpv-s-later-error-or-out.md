---
id: T-4375
title: "Probe whether XNPV's later error or out-of-range date beats a refused first value's #VALUE!"
milestone: M9
lane: functions-financial
status: todo
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

## Acceptance criteria
- [ ] The probe case is filled on excel-graph and passes; financial.md states the rule and drops the Unprobed lines.
- [ ] If Excel lets any of these beat the `#VALUE!`, the walk follows it (one walk, no second reader).
- [ ] `make check` green; reviewers approve.

## Out of scope
- XIRR (T-3816); a first value that is a number (T-3815, evidenced).
