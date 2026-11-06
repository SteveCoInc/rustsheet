---
id: T-4059
title: "CUMPRINC and CUMIPMT Gate A r4-T-3141 batches: T-3141's sweep of the underflowed principal"
milestone: M9
lane: functions-financial
status: done
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
- [x] The `r4-T-3141` batches are filled after T-3141's freeze, their failing points are owned, and their envelope
  result is in T-3141's Outcome.
- [x] probes/fin-cumprinc-underflow-type1 C7:C9 and G7:G9 and fin-cumprinc-underflow-fresh I10 and K10 pass, or the
  sweep shows the geometric sum is Excel's there.
- [x] `make check` green; reviewers approve.

## Out of scope
- Ranges where `v^nper` does not underflow (T-3216, T-3413, T-3776).

## Outcome
- **The aimed draw.** `annuity_cumulative.py`'s strata never reach T-3141's region, and a new shape among its edges
  would move every `s1` point. So a definition can now carry `aimed`, a whole-point draw of one region that only
  `rustsheet-accuracy generate --stratum aimed` reaches. It is never in the default split, and the sidecar records it
  as stratum `aimed` (SPEC-0029 "Strata"; `Stratum::Aimed` in rustsheet-eval). CUMIPMT's and CUMPRINC's aimed draw is
  `_underflowed`:
  - at most 32 periods;
  - `1 - v^nper` is 1 in doubles (nper·L above 40);
  - fractional rates from 1E-4 to 3;
  - the leading exponent `(nper-end+1)·L` is below 0.3 one time in three, else log-uniform from 0.3 to 700, where
    the leading power is still normal;
  - m·L is log-uniform from 0.01 to 5, across PMT's expm1 switch;
  - type 1 starts from period 2;
  - nper runs up to 1E15.
  `s1` regenerates byte-identically for both functions, and `verify` reports 0 differences.
- **Batches.** `CUMPRINC/r4-T-3141` and `CUMIPMT/r4-T-3141` have 200 points each. They were drawn and filled from
  excel-graph on 2026-11-06, after T-3141's kernel froze (its branch's last code commit is from 2026-11-04). Below a
  leading exponent of 0.3 there are 52 CUMPRINC and 44 CUMIPMT points; from 0.3 up to 679 there are 148 and 156.
- **Envelope.** On main, all 400 points are in the envelope and none fails, so no known-failure line is added. With
  T-3141's frozen kernel (branch head, scored locally), all 400 are in the envelope too.
- **Exactness against Excel's doubles**, CUMPRINC (CUMIPMT in brackets):

  | engine | lead < 0.3 | lead ≥ 0.3 |
  |---|---|---|
  | main | 23/52 (15/44) | 63/148 (126/156) |
  | T-3141 frozen (core below 0.3) | 52/52 (43/44) | 63/148 (126/156) |
  | T-3141's core at any leading exponent (its round-1 form) | 52/52 (43/44) | 148/148 (156/156) |

  The sweep shows that past 0.3 the core, not the geometric sum, is Excel's. The geometric sum is up to 4 ulps off
  there. The core is exact out to a leading exponent of 679, with m·L from 0.003 to 4.96, for both types.
  - So T-3141 takes the core at any leading exponent. That fixes fin-cumprinc-underflow-type1 C7:C9 and G7:G9 and
    fin-cumprinc-underflow-fresh I10 and K10 (T-3141 round 1).
  - The one CUMIPMT miss is the same in every row, so the principal does not cause it:
    `CUMIPMT(0.103414751928719,17312013078,200894.020467694,17312013077,17312013078,1)`, A41, is 4 ulps off.
- **Where the record goes.** The envelope result and these counts go into T-3141's Outcome on its branch (PR #3105),
  together with the cut change. That keeps this PR off the T-3141 task file, which the branch rewrites.
- **Eval delta.** Gate A gains 400 points, 0 failing; the eval suites do not change.
