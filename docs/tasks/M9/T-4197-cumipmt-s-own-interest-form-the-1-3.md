---
id: T-4197
title: "CUMIPMT's own interest form: the 1-4 ulp misses no per-period form explains (t3179 A1, D1; CUMPRINC_CUMIPMT.xlsx G3, G4, G15)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
CUMIPMT matches Excel to the bit on the short-range cells where `PMT·m - CUMPRINC` is 1 to 4 ulp off (t3179 A1 by 1
ulp, D1 by 4), or a recorded negative result closes them as Excel's own rounding.

## Context
- Follow-up of T-3179. probes/t3179-cumipmt-ipmt-sum shows Excel's CUMIPMT is not its IPMT sum, even over one period
  (financial.md "CUMIPMT of one period is not that period's IPMT").
- T-3179 scored four forms on the 168 evidenced CUMIPMT cells with ranges of at most 32 periods, each written as a
  formula over our PMT, FV, IPMT and PPMT. Ours (`PMT·m - CUMPRINC`) gets 92 to the bit, `PMT·m - SUM(PPMT)` 61,
  `SUM(IPMT)` 24 and the balance form `SUM(FV·r)` 9 to 11. None of them explains A1 and D1 together with the
  single-period cells Q1 (tiny-rate-structure) and H10 (gate-edges), which ours already has.
- `PMT·m - SUM(PPMT)` has 10 cells ours misses, all at rates 2E-6 to 1.5E-4 (gate-edges A4, A7, E5; tiny-rate-structure
  H2, K2, F3, D5, F6, F10, S10). That points at the principal at tiny rates rather than at the interest form.
- Owned cells: probes/t3179-cumipmt-ipmt-sum A1, D1. Their two known-failure lines are new-case coverage of formulas that already fail as fin-cumipmt-overflow-and-forms A8 and A13, not regressions.
  Also owned: CUMPRINC_CUMIPMT.xlsx G3 (= A1), G4 and G15, and fin-cumipmt-overflow-and-forms A8, A13 (= A1, D1),
  which T-3179 took over from T-2243.
- Scratch method (not committed): a case with `=CUMIPMT(..)` beside each candidate formula, scored with
  `rustsheet-eval show --root <scratch> --all`.

## Acceptance criteria
- [ ] Each candidate form scored on the same 168 cells, with a hold-out split fixed before scoring (decision 0188 R4).
      Adopt one only if it fixes cells and breaks none, with no keep-matched branch.
- [ ] Either the owned cells pass, or financial.md records the forms ruled out and the task closes as a negative
      result.
- [ ] `make check` green; reviewers approve.

## Out of scope
- CUMPRINC's own misses (CUMPRINC_CUMIPMT.xlsx H column).
- PMT's `1-Q` switch in CUMPRINC (T-3179: it moves no cell).
