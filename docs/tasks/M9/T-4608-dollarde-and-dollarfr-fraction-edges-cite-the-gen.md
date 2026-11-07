---
id: T-4608
title: "DOLLARDE and DOLLARFR fraction edges: probe fractions below 1, non-integer fractions and fractions from 1E9 up (10^k-1, 10^k, 10^k+1), and match"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel cells decide the DOLLARDE and DOLLARFR fraction edges that financial.md "DOLLARDE and DOLLARFR scale the fraction
by the next power of ten (T-4110)" leaves unprobed. financial.md states them with the cells, and ours matches every
cell. The edges:
- a fraction below 1 (0, between 0 and 1, negative);
- a non-integer fraction;
- fractions from 1E9 up, where a double `log10` of 10^k + 1 may round to k.

## Context
- Gap triage 2026-11-06 (`make eval` on agent/tasks/refill-sonnet-1206, cut from origin/main). No unowned visibly-off
  cells were left, so this triage read the rules the semantics docs mark unprobed.
- The entry: "Unprobed: a fraction below 1 (Microsoft documents `#NUM!` below 0 and `#DIV/0!` from 0 to 1), a non-integer
  fraction, and fractions past 1E9, where a double `log10` of 10^k + 1 may round to k". The evidence so far is the Gate A
  tables accuracy/DOLLARDE/s1 and accuracy/DOLLARFR/s1 (240 points, integer fractions) and gen-m9 cells.
- gen-m9 has these fractions only inside composite formulas, which do not isolate the rule:
  - gen-m9/762cd0e4ec9a7aa9 Calc!A141 holds `DOLLARFR(0.3,0.1)` and is `#DIV/0!`;
  - gen-m9/c7664e494e418395 Calc!A37 holds `DOLLARDE(Data!E9,0.3)` and is `#DIV/0!`;
  - gen-m9/5919660b7bb4d195 Calc!A50 holds `DOLLARFR(Data!D5,1.005)` and is a number;
  - gen-m9/c468f5046aadfbbc Calc!A77 holds `DOLLARDE(Data!A11,2.5)` and is `#NUM!` from CRITBINOM.
- Ours (release build of this branch, scratch case):
  - `DOLLARDE(1.5,0.5)` is `#DIV/0!`;
  - `DOLLARDE(1.5,999999999)` is 1.5000000005, `(1.5,1000000000)` 1.5, `(1.5,1000000001)` 5.999999995 and
    `(1.5,1E15+1)` 1.4999999999999996;
  - `DOLLARFR(1.5,1000000001)` is 1.05000000005 and `DOLLARFR(1.5,99999999999)` 1.499999999995.
- Code: `fn_dollarde` and `fn_dollarfr` in crates/rustsheet-engine/src/functions/financial/rate_conversions.rs.

## Probe
excel-probe skill, excel-graph, tag M9. Name the case probes/t4608-dollarde-dollarfr-fraction-edges.
- DOLLARDE and DOLLARFR of 1.5, -1.5 and 0.3 with each fraction below: 0, -1, -0.5, 0.3, 0.5, 0.999, 1, 1.005, 1.5,
  2.5, 15.9 and 16.1.
- The same dollars with 10^k - 1, 10^k and 10^k + 1 for k = 9, 10, 12, 15, and with 2^53 - 1, 2^53 and 1E300.

## Acceptance criteria
- [ ] probes/t4608-dollarde-dollarfr-fraction-edges is committed with its excel-graph oracle sidecar.
- [ ] Every probe cell passes, or each visibly-off cell has an owner task. A last-digit-only miss on a power-of-ten
      cut is filed as its own `defer: phase-1b` task, not fitted here.
- [ ] financial.md states the error rule for fractions below 1, the truncation (or not) of a non-integer fraction, and
      the 10^k cut, with the cells. Its "Unprobed" sentence is gone.
- [ ] If the engine changes, a unit test pins one changed cell (test_fn_financial.rs `fn_dollarde` is the place).
- [ ] No new known-failure line in eval/baseline/probes.txt, and accuracy/DOLLARDE and accuracy/DOLLARFR still pass
      (`rustsheet-eval accuracy`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Argument-kind pairs (T-3780) and the empty-text or error kinds (T-3126).
