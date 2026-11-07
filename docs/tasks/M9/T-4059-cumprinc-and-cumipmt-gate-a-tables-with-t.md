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
  result is in T-3141's Outcome (on its branch, PR #3105, which lands after this PR and cites these batches).
- [x] probes/fin-cumprinc-underflow-type1 C7:C9 and G7:G9 and fin-cumprinc-underflow-fresh I10 and K10 pass, or the
  sweep shows the geometric sum is Excel's there.
- [x] `make check` green; reviewers approve.

## Out of scope
- Ranges where `v^nper` does not underflow (T-3216, T-3413, T-3776).

## Outcome
- **The aimed draw.** The strata of `annuity_cumulative.py` never reach T-3141's region. Adding a shape among its
  edges would move every `s1` point.
  - A definition can now carry `aimed`, a whole-point draw of one region. Only `rustsheet-accuracy generate --stratum
    aimed` reaches it, and it is never in the default split.
  - Sidecars record such points as stratum `aimed`: SPEC-0029 "Strata", `Stratum::Aimed` in rustsheet-eval.
  - The spec limits aimed points to `e-*` evidence batches. An R4.2 sweep stays a full-domain `r4-*` batch.
  - CUMIPMT's and CUMPRINC's aimed draw is `_underflowed`. It draws a short range (at most 32 periods) where
    `1 - v^nper` is 1:
    - fractional rates from 1E-4 to 3;
    - the leading exponent below 0.3 one time in three, else log-uniform up to 700;
    - m·L across PMT's expm1 switch;
    - type 1 from period 2;
    - nper up to 1E15.
  - `s1` regenerates byte-identically for both functions, and `verify` reports 0 differences.
- **Fit data `e-T-3141`.** These are the first aimed draw, 200 points per function. Their history:
  - They were generated as `generate --batch r4-T-3141 --stratum aimed` and filled from excel-graph at 2026-11-06
    21:53 UTC. That was after T-3141's kernel froze on 2026-11-04.
  - They are kept byte for byte, so their description still names that batch. Their seed is in the math sidecar,
    and `verify` reports 0 differences.
  - With T-3141's kernel they are exact on CUMPRINC 52/52 below a leading exponent of 0.3, but on only 63/148 from 0.3
    up, where the geometric sum takes over. With the core at every lead they are exact on 200/200, and on CUMIPMT
    199/200.
  - T-3141 then dropped its 0.3 cut on this result together with the probe cells type1 C7:C9 and G7:G9, fresh I10 and
    K10, and three gen-m9 cells. So these points are fit data, not a sweep, and they are named after T-3141.
  - The new form froze at T-3141's commit 0a1f01a (2026-11-06 22:20:15 UTC).
- **Aimed hold-out `e-T-4059`.** 200 aimed points per function, drawn after the refreeze and filled from 22:20:43 UTC
  on. T-3141's form is exact on CUMPRINC 199/200 and CUMIPMT 200/200; main gets 81 and 151. From 0.3 up, the
  geometric sum gets 70 of 170 CUMPRINC principals to the bit. The finding is in financial.md "CUMPRINC where
  `v^nper` underflows, at deep leading exponents (T-4059)". Its open misses go to T-4710: e-T-4059 A119 and e-T-3141
  CUMIPMT A41.
- **R4.2 sweep `r4-T-3141`.** 200 points each, drawn and filled after T-3141's freeze. Each is 100% log-uniform over
  the full domain (`--stratum log-uniform`), as SPEC-0029 asks.
  - Main and T-3141's frozen form score the same on every point except 4 CUMPRINC principals from a leading exponent
    of 0.3 up. T-3141 gets all 4 to the bit: 24 of those 39 against main's 20, and it loses none.
  - A full-domain draw seldom lands in T-3141's region: 7 of the 200 CUMPRINC points are short ranges where `v^nper`
    underflows. T-3141 gets all 7 to the bit and main 3. The aimed hold-out above covers the region itself.
  - Envelope: 28 CUMIPMT points fail on main and on T-3141 alike. All are tiny rates, 1E-10 to 4E-8, where the
    interest `PMT·m - CUMPRINC` cancels. Those are T-4390's class, so their known-failure lines name T-4390.
  - Every other point is in the envelope on both.
- **Eval delta.** Gate A gains 1,200 points: 1,172 in the envelope and 28 known failures (T-4390). The eval suites do not
  change.
- **Follow-up:** T-4710.
