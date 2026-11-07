---
id: T-4675
title: "Oracle sweep: where Excel's beta quantile search saturates at a times 5.41E-151 (BETA.INV, F.INV.RT, F.INV, FINV; for T-4662)"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
An oracle-filled probes-numeric case gives T-4662 enough Excel points to tell what Excel's far-tail saturation of
the beta quantile search depends on. T-4662 then derives the mechanism from these points instead of fitting the
single constant it has now.

## Context
- Gap triage 2026-11-06 (`make eval` on origin/main ebe81baf6): `make unowned-off ALL=1` lists exactly two
  visibly-off cells, and only the deferred T-4662 owns them. T-4662 owns these cells:
  - gen-m9/abc2939ebd1e2365 Calc!A11 `=CHIDIST(F.INV.RT(1E-200,1,1.999),10)`: Excel 0, ours `#NUM!` (wrong type).
  - gen-m9/01523d247594952e Calc!A377 `=SYD(15023,(AVEDEV(Data!A11:A11,{1;2;3;4;5})*F.INV.RT(1E-20,9,3)),33,1)`:
    Excel -1987062826671.328, ours -1987062826671.3352.
  T-4662's own text says that a probe varying df1 and df2 at p = 1E-200 would show what the constant depends on.
  No open task files that probe, and T-4662 has no sweep dependency. The triager rule is: "a numeric-fitting group
  that needs more Excel points than the cached workbooks give is not attempted blind".
- Evidence on file (docs/excel-semantics/statistics.md "F.INV.RT saturates at 3.696066238512854E+150/df1 in the far
  tail (T-2435)"):
  - probes/t2435-dist-extremes S!A77, A79, A115, A116, A118: F.INV.RT(p, 1, df2) is 3.696066238512854E+150 for df2
    1.5 to 1.999 and p 1E-200 to 1E-300. A117 `F.INV.RT(1E-200,2,1.999)` is 1.8480349672907807E+150.
  - probes-numeric/t1543-inverse-cdf-tiny-p Calc!A91, A92: `BETA.INV(1E-300,0.5,100)` and `BETA.INV(1E-200,0.5,100)`
    are both 2.7050408442680296E-151. The true quantile is about 7.9E-603. A93 at p = 1E-100 is the true
    7.873641070203053E-203. T-4665 names A91 and A92 as exceptions to BETA.INV's underflow rule (docs only); T-4665's
    Out of scope puts their saturation in T-4663's family; the lead below suggests it is T-4662's mechanism, and
    this sweep measures both.
  - accuracy/BETA.INV/s1 A131 `BETA.INV(2.22507385850721E-308,1.12049449852741,2.10175380918621)` is
    9.192367895421942E-151, also far above the true quantile.
- Lead found in triage. This is arithmetic on the cells above, not new evidence. The upper F tail is the beta
  quantile of x = df2/(df2 + df1*F) at shape a = df2/2 and b = df1/2. The saturated x divided by a is the same to
  16 digits for df2 = 1.5, 1.9 and 1.999: 5.411158434229572E-151, at b = 0.5. At b = 1 (A117) it is
  5.4111530230729346E-151. BETA.INV A91 (a = 0.5, b = 100) gives 5.4100816885360592E-151. A131 (a = 1.12, b = 2.10)
  gives 8.2E-151. So the stop looks linear in a, independent of p, and weakly dependent on b. That fits a Newton
  correction near x/a falling below a tolerance of about 1E-150 (T-2808's AS 109 replay). T-2808's replay landed on
  the geometric path, but Excel's value is exactly proportional to a. A131's ratio does not fit, so the rule is not
  settled. This sweep is there to test it.
- Prediction, committed in the case `description` before the fill (excel-probe skill; owner rule "do not overfit"):
  H1 holds where Excel saturates. H1: x/a lies within 1E-3 relative of 5.411E-151, and does not change with p.
  F.INV.RT(p, df1, df2) * df1 is then the same for every df2 below 2. The sweep also records where the
  saturation starts: in p, at the BETA.INV(10^-k, 0.5, 100) and F.INV.RT(10^-k, 1, 1.999) rows.
- Grid (one cell per formula, about 220 cells, `probes-numeric/t4675-beta-search-saturation.case.json`, tag M9):
  - `BETA.INV(p,a,b)`: p in {1E-200, 1E-300, 2.22507385850721E-308}; a in {0.25, 0.5, 0.75, 0.9, 0.9995,
    1.12049449852741, 1.5, 2}; b in {0.5, 1, 2.10175380918621, 10, 100}.
  - `BETAINV(1E-200,a,b)` for a in {0.5, 0.9995} and b in {0.5, 100} (the alias).
  - `F.INV.RT(p,df1,df2)`: p in {1E-200, 1E-300}; df1 in {0.5, 1, 2, 3, 4, 10}; df2 in {0.5, 1.2, 1.5, 1.9, 1.999,
    2.5}.
  - `F.INV(1E-200,df1,df2)`: df1 in {0.5, 1, 1.5, 1.999}, df2 in {1, 10}. `FINV(1E-200,1,1.999)`,
    `FINV(1E-200,2,1.999)`.
  - Onset: `BETA.INV(10^-k,0.5,100)` for k = 100, 110, ..., 200 and `F.INV.RT(10^-k,1,1.999)` for k = 100, 120,
    140, 150, 160, 180.
- Code the result informs, not changed here: crates/rustsheet-engine/src/functions/statistical/fisher.rs
  `f_quantile`, beta.rs `beta_quantile`.

## Acceptance criteria
- [ ] `eval/suites/probes-numeric/cases/t4675-beta-search-saturation.case.json` has the grid above and the H1
      prediction in its `description`, written before the fill. Its `.oracle.json` comes from
      `rustsheet-oracle fill -b excel-graph` with no pending cell, and its `.math.json` comes from
      `rustsheet-accuracy math --suite probes-numeric`.
- [ ] statistics.md's "F.INV.RT saturates at 3.696066238512854E+150/df1" entry cites the case and tabulates x/a for
      every saturated cell. It states whether H1 held (cells for and against), and gives the onset p for each onset
      row. Where Excel gives an error or the true quantile instead, it says which cells do so.
- [ ] T-4662 names the new case (`- probes-numeric/t4675-beta-search-saturation.case.json`) in its cell list, so
      every visibly-off cell of the case has an owner (`scripts/new_case_owners.py` passes).
- [ ] Fixtures hold only formulas and Excel values. If the oracle is unavailable, this task is `blocked` with the
      reason.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Deriving or implementing the stop rule (T-4662).
- BETA.INV's 0.5 at a DBL_MIN shape (T-4663, T-4676) and its 1-2^-53 at p near 1 (T-3351).
- F.INV.RT far from the math at a large df2 or at p = DBL_MIN with large shapes (T-4568).
