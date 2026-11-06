---
id: T-4515
title: "Placeholders after a fraction and commas after the point: one model for the T-4507 probes"
milestone: M10
lane: functions-text
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
One model for the digit placeholders after a fraction's denominator, and for commas after the decimal point, that
matches every probe of both, replacing the per-shape rules.

## Context
number-formats.md "Placeholders after a fraction" (probes/t4043-*, t3711-fixed-denominator-then-placeholder, from
T-4507) and "Commas after the decimal point" (t3598-decimal-part-commas rows 23, 24, 26, t3666-review-followups
A7:A12). The model in "Placeholders after a fraction" matched all its probes but not
probes/t4204-fraction-denominator-mixed; reconcile it with "A placeholder denominator prints its unused placeholders in
place". Subsumes T-4069. Hold-out: a fresh probe written after the model is frozen.

## Owned cases
Every visibly-off cell of these probes (decision 0245):
- probes/t4043-fixed-denominator-cut
- probes/t4043-fixed-denominator-slots
- probes/t4043-frozen-model-holdout
- probes/t4043-placeholder-denominator-slots
- probes/t4043-separated-slots
- probes/t4043-trailing-shapes-holdout
- probes/t3711-fixed-denominator-then-placeholder
- probes/t3711-placeholders-after-fraction
- probes/t3598-decimal-part-commas
- probes/t3666-review-followups

## Acceptance criteria
- [ ] The t4043-* probes, t3711-fixed-denominator-then-placeholder and the comma cells above pass, with no regression.
- [ ] A hold-out probe made after the model froze passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Locale tables (T-4508).
