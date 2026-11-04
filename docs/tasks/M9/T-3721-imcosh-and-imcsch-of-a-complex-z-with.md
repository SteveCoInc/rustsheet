---
id: T-3721
title: "IMCOSH and IMCSCH of a complex z with |x| <= 0.01: the 3 t3589 texts one 15-digit tie off (B15, D15, D29)"
milestone: M9
lane: functions-engineering
status: done
priority: P3
size: S
tier: opus
depends_on: [T-3720]
campaign: visible
---

## Goal
IMCOSH and IMCSCH of a complex z = x+yi with 0 < |x| <= 0.01 give Excel's text for the three t3589 cells that
T-3609 left UNVERIFIED. The kernel is chosen from ranked candidates on T-3720's sweep, not on these 3 cells alone.

## Context
- Gap triage 2026-11-01h (`make eval` on main b3389661, excel-graph). `scripts/exit_owners.py --milestone M9` lists
  these cells under `rate:IMCOSH` and `rate:IMCSCH`, both functions being below the Phase 1 rate. No open task owns
  them: T-3609 (done) recorded them as UNVERIFIED. They are text cells, so decision 0188 R1(c) counts them as visibly
  off. Excel vs ours (`make eval-show` on the case in the cell list below):
  - S!B15 `IMCOSH("-0.00908725305283191-0.897991007302082i")`: Excel 0.623208141637039+0.00710702505540606i, ours
    ...+0.00710702505540607i. The imaginary part is sinh x · sin y.
  - S!D15 `IMCSCH("-0.00908725305283191-0.897991007302082i")`: Excel -0.00925756264826299+1.27852743654627i, ours
    -0.009257562648263+1.27852743654627i.
  - S!D29 `IMCSCH("-0.00994266897763484-0.217145289213716i")`: Excel -0.208738033086954+4.63196641538457i, ours
    -0.208738033086955+4.63196641538457i.
- What is known (docs/excel-semantics/engineering.md, "IMSECH and IMCSCH of a small real number", T-3609): every
  pairing of the series, libm and `1/u`-form sinh x with the form, libm and series cosh x scores the same 105/111
  complex IMCOSH and 71/75 complex IMCSCH texts, and none fits these three. The other misses there are notation only.
  So the x factor is not the lead. The candidates to rank are:
  - y's factor: Excel's worksheet SIN/COS (`trig_sin_cos`, already Excel's for IMSIN/IMCOS since T-2629) against
    libm `sin`/`cos`;
  - the order of the complex product (sinh x · sin y against sin y · sinh x is the same double, but a fused or x87
    product is not);
  - for IMCSCH, the reciprocal of the complex sinh: `conj(w)/|w|²` with |w|² as `re² + im²`, `hypot²`, the
    Smith-style division IMDIV uses, or 1/(sinh² x + sin² y) from the closed form
    (sinh² x cos² y + cosh² x sin² y = sinh² x + sin² y).
  T-3720's companions (Excel's SIN(y), COS(y), SINH(x), COSH(x), EXP(-x)) give each factor's double directly. Reuse
  the existing kernels (directive 3). Method: docs/research/excel-kernel-playbook.md. Fit lab: `im-hyperbolic`,
  `im-hyperbolic-t3589-fresh`.
- Needs an oracle sweep: T-3720. Do not pick a kernel from the 3 cells above.
- Overlap: T-3593/T-3594 (unmerged branch agent/functions/tasks-gap-triage-opus-2026-11-01, PR #2676) cover IMSINH
  and IMCOSH at |x| <= 0.01. T-3589 has since fixed T-3594's 5 owned cells, and neither task names these cells. If
  T-3594 lands first, start by re-measuring B15 on its main.
- Code: `inverse_exp_sinh_cosh`, `imcsch` in crates/rustsheet-engine/src/functions/engineering/complex/trig.rs.
- Cells owned:
  - probes-numeric/t3589-im-hyperbolic-small-x-sweep.case.json (3): S!B15, D15, D29

## Acceptance criteria
- [x] The 3 owned cells match Excel's text. Otherwise the Outcome names each one left over, with the candidate
      scores showing that no candidate within the guards below reaches it.
- [ ] R4 guards (reviewer-excel-fidelity check 7, decision 0188), shown in the Outcome:
  - [x] Hold-out (R4.1): train and hold-out counts side by side (bit-exact by text, within 15 digits) for the chosen
        kernel and the correctly rounded one, on T-3720's sweep split by the fit lab (T-3360). The margin is stated
        before scoring. The chosen kernel beats the correctly rounded one on hold-out.
  - [x] Fresh sweep (R4.2): after the choice is frozen, a new probes-numeric case with complex z at 0 < |x| <= 0.01
        is oracle-filled and its counts are recorded.
  - [x] Gate A envelope (R2): the fresh sweep's count inside the envelope (within 15 digits of Excel, or within 1e-9
        relative of 50-digit mpmath, absolute floor 1e-300). Every point outside is listed. The expected answer is 0
        points outside and 0 error-against-number cells.
  - [x] `FITTED` tag (R4.3): any constant or cut chosen by scoring carries `FITTED(T-3721, n_cells, range)` and applies
        only inside that range. A structural choice among candidates (kind (c)) has no tag, but the Outcome says it is
        kind (c).
  - [x] Reproducibility floor (R4.4): no step targets differences below the oracle's floor (decision 0112, web
        against desktop up to 4 ulp).
  - [x] No keep-matched branch: no band or special case whose domain is defined by the cells that already match
        (decisions 0056, 0082).
- [x] Every other IMCOSH/IMCSCH/IMSECH/IMSINH oracle cell keeps passing: the t3589 and t3609 sweeps, the fit lab's
      `im-hyperbolic*` and `im-small-imag` sets, and gen-m9. The probes-numeric failures of the t3589 small-x sweep
      drop by these 3, and `make eval-baseline` records no new known-failure line in
      any eval/baseline/<suite>.txt.
- [x] docs/excel-semantics/engineering.md's "IMSECH and IMCSCH of a small real number" entry records the complex
      rule, cites T-3720's case and drops "UNVERIFIED for B15, D15 and D29" where this settles them.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decisions 0083 and 0114 (`visible`): done when the 3 owned text cells equal Excel's text. Each leftover is recorded as
UNVERIFIED in engineering.md with the ranked candidates. Excel is the target, not the math. Bit-level residue that
the 15-digit text cannot show moves to a follow-up with `defer: phase-1b`.

## Out of scope
- Real x (T-3609, done). Notation of parts below 1 (T-3560). The 0.01 cut (T-2797). |x| above 0.01.

## Evidence (T-3720)
Dense sweep with SIN(y), COS(y), SINH(x), COSH(x), EXP(-x) companions: eval/suites/probes-numeric/cases/t3720-im-reciprocal-small-x-complex.case.json (+ .oracle.json). Counts in docs/excel-semantics/engineering.md.

## Outcome
- Kernel: `sinh_series` (complex/trig.rs) now sums exactly three terms, `x + x³/3! + x⁵/5!`, with no stopping test
  (`SINH_SERIES_TERMS = 3`). The old loop ran until a term left the sum unchanged, which is four terms above
  |x| ≈ 0.005. The fourth term `x⁷/7!` moves the last bit there, and each owned cell is a 15-digit near-tie it tipped.
  Kind (c): a structural choice among ranked candidates (the term count), with no fitted constant and no `FITTED` tag.
- Ranking on T-3720's sweep (404 rows, Excel's own SIN(y), COS(y), SINH(x), COSH(x), EXP(-x) as factors, scored part by
  part and then by text). For the IMCOSH imaginary part, 3 terms give 404/404, 4 or 5 terms 396 (the engine), 2 terms
  136, the 3 terms in reverse order 395, the series or Horner's form in x87 (64-bit) 390, Horner's form in doubles
  386-391, and the `1/u` form or the complex-EXP forms 42 or fewer. None of these moved it: Excel's SIN(y) against
  libm's or the exact sine, the product rounded at 53, 64 or exact bits, libm's or the worksheet SINH. With 3 terms
  the texts are IMCOSH 403, IMCSCH 402, IMSECH 404 and IMSINH 403 of 404, against 395, 398, 400 and 402 before. For
  cosh x the `1/u` form stays best (libm, the worksheet COSH, a 2-4 term series and `sqrt(1 + sinh²x)` all score lower).
- Hold-out (R4.1; fit lab `im-small-x-complex`, the fit lab's split, its default 5-point margin taken as stated before
  scoring). Three-term: train 1128/1131 texts, test 484/485. Correctly rounded: 1052/1131 and 448/485. The old engine:
  1116 and 479. Verdict: accept.
- Fresh sweep (R4.2): probes-numeric/t3721-im-hyperbolic-small-x-fresh-sweep, 400 rows × 4 functions, generated after
  the freeze (docs/research/t3589/gen_sweep.py, seed 3721; |x| log-uniform in [1E-6, 0.01] and uniform in
  [0.004, 0.01] on alternate rows, y = 0 on every fifth row) and filled by excel-graph on 2026-11-03. Fit lab
  `im-small-x-complex-fresh`: three-term 1596/1600 texts (train 1099, test 497 of 498), the old engine 1580, correctly
  rounded 1505.
- Gate A envelope (R2): all 1600 fresh texts are within 1e-9 relative of the 50-digit value. 0 points are outside and
  there are 0 error-against-number cells. The 4 fresh leftovers (C311, C386, C390, D169) are within 15 digits of
  Excel's text.
- Reproducibility floor (R4.4): every step is a whole term of the series, not a sub-ulp target. No keep-matched branch:
  the change applies to every |x| <= `INVERSE_EXP_LOW` alike.
- Leftovers: 8 texts, 4 on each sweep (fresh C311, C386, C390, D169; T-3720 A3, D7, B25, B148). At each of these x
  the three- and four-term series give the same double, so the misses are cosh x or division residue. They miss the
  same way on main. Filed as T-3930 (opus).
- Other cells: `rustsheet-eval run` on probes, probes-numeric and gen-m9 fixes 20 cells and adds 0 new failures: the 3
  owned t3589 cells and 17 on T-3720's sweep (IMCOSH A206, A253, A269, A343, A346, A362, A401, A402; IMCSCH B401-B404;
  IMSECH C203, C217, C256, C292; IMSINH D265). The fit lab's `im-hyperbolic-t3589-fresh` goes from 599 to 600 of 600.
  `make eval-check`: every gate suite is unchanged (0 fixed, 0 regressions). The probe suites are not baselined, so
  `eval/baseline/` does not change.
- engineering.md "IMSECH and IMCSCH of a small real number" now records the three-term rule, the candidates, the
  hold-out and fresh counts and the 8 UNVERIFIED leftovers. Unit test: `series_sinh_stops_after_three_terms`.
- Eval delta: probes-numeric +20 cells (491413 → 491433 of 798037 on main's cases); gate suites 0.
