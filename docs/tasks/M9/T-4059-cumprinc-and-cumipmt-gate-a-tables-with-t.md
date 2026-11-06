---
id: T-4059
title: "CUMPRINC and CUMIPMT Gate A r4-T-3141 batches: T-3141's sweep of the underflowed principal"
milestone: M9
lane: functions-financial
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
CUMPRINC and CUMIPMT get the R4.2 batch `r4-T-3141` of their Gate A tables (SPEC-0029), so T-3141's
underflowed-principal core gets the fresh sweep, envelope result and deep-leading-exponent evidence decision 0188 R4
requires, and T-3141 can land.

## Context
- T-3141 (PR #3105, branch agent/functions-financial/T-3141-cumprinc-underflow-bits) found the core
  `C = pv·((1-v^m)·v^(nper-end+1))` (type 1 `-C`, type 0 `-(C·(1+r))`) where `1 - v^nper` is 1. Its fidelity review
  asked for the R4.2 sweep as an `r4-T-3141` batch of a Gate A table, the Gate A envelope result, and evidence at leading
  exponents `(nper-end+1)·L` above 0.3 for fractional rates.
- T-4110 wrote the definitions (`tools/src/rustsheet_tools/accuracy/defs/annuity_cumulative.py`, the exact sums in
  closed form) and committed the filled `s1` tables (`eval/accuracy/tables/{CUMIPMT,CUMPRINC}/s1`, with their failing
  points owned in known-failures.tsv). Reuse both: no new definition module, and `s1` stays as committed.
- With T-3141's kernel frozen, `generate CUMPRINC --batch r4-T-3141` (and CUMIPMT). The batch must reach the
  underflowed short ranges, including ranges far from nper where `v^(nper-end+1)` is still normal and type 1 from
  period 2. `annuity_cumulative.py`'s draws do not aim there today: extend its draw for this batch (a shape in its
  strata, or `--stratum`) without moving the committed `s1` points (`rustsheet-accuracy verify` and a byte-identical
  regeneration of `s1`). Fill it, run `rustsheet-eval accuracy`, and record the result in T-3141.
- If the core holds above `GEOMETRIC_MIN_LEAD`, T-3141 may move it ahead of the geometric sum: it is then exact on
  probes/fin-cumprinc-underflow-type1 C7:C9 and G7:G9, fin-cumprinc-underflow-fresh I10 and K10, and gen-m9
  d0854366830dcdee A98, 67a4c03afd63f988 A208 and b73c6cc96e093f03 A111 (scored in T-3141 round 1).

## Acceptance criteria
- [ ] The `r4-T-3141` batches are filled after T-3141's freeze, their failing points are owned, and their envelope
  result is in T-3141's Outcome.
- [ ] probes/fin-cumprinc-underflow-type1 C7:C9 and G7:G9 and fin-cumprinc-underflow-fresh I10 and K10 pass, or the
  sweep shows the geometric sum is Excel's there.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Ranges where `v^nper` does not underflow (T-3216, T-3413, T-3776).
