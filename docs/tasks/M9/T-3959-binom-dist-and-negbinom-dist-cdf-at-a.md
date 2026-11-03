---
id: T-3959
title: "BINOM.DIST and NEGBINOM.DIST cdf at a large n: Excel's cdf arithmetic on T-3774's never-attempted cdf cells, near the mean and at moderate p (34 visibly off)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
The cumulative (`TRUE`) BINOM.DIST and NEGBINOM.DIST cells of the two t3352 probes listed below are within 15
significant digits of Excel, from a model of the arithmetic Excel's bratio branches use at these points. Any cell left
off is recorded as UNVERIFIED in statistics.md with the hypotheses ruled out and their counts.

## Context
- T-3992 (excel-graph fill-to-fill stability, 2026-11-03): its kernel was not measured; see docs/research/t3992/README.md.
- Gap triage 2026-11-03b (`make eval` on origin/main c2062e5c8, excel-graph evidence; `scripts/exit_owners.py` per-cell
  ownership against every open task). The M9 exit and the Phase 1 rate (rate:BINOM.DIST, rate:NEGBINOM.DIST) count these
  cells. Their only owner is T-3774, which is `blocked` (time-boxed, decision 0056) after two attempts at the **pmf**
  term. T-3774 lists "the cdf cells' basym arithmetic (not attempted)" among its untried hypotheses, and its Context
  says the cdf cells "were not attempted". Decision 0056's time-box therefore does not cover them. This task splits
  them out; T-3774 keeps the pmf cells. T-3961 takes the p < 0.01 tail cells, where Excel is hundreds to tens of
  thousands of ulp off the math (another regime: see there).
- Where we stand. Ours is the accurate one at most of these cells; Excel is not. 50-digit mpmath (the report's math
  values where the case has them, otherwise mpmath `betainc` at the double arguments, 60 digits) gives:
  - large-n A142 `NEGBINOM.DIST(10000,2500,0.2,TRUE)`: Excel 0.5035682089773266, ours 0.5035682089773399. Excel is
    -110 ulp from the math, ours +10.
  - large-n A130 `NEGBINOM.DIST(1000,250,0.2,TRUE)`: Excel 0.5112825491133768, ours ...798 (Excel -23 from the math,
    ours +4).
  - large-n A2 `BINOM.DIST(3000,10000,0.3,TRUE)`: Excel 0.5049329838192986, ours 0.5049329838192993 (ours +6 ulp on
    Excel). A12 `BINOM.DIST(4850,10000,0.5,TRUE)`: Excel 0.0013939837588563982, ours ...3644 (-156 ulp).
  - The NEGBINOM.DIST r = 10 column, A146, A170, A182, A206 (`NEGBINOM.DIST(1000000000,10,9.9999999e-09,TRUE)`):
    Excel is 6 to 9 ulp off the math and ours is correctly rounded at each.
  - sweep A107 `NEGBINOM.DIST(1057,94,0.0862540984610542,TRUE)`: Excel -35 ulp from the math, ours correctly rounded.
    sweep A16 `BINOM.DIST(244,493,0.730093121043103,TRUE)`: Excel -137, ours -67.
  So the work reproduces Excel's formulation where Excel is the inaccurate one (`visible`, decision 0114), not an
  accuracy fix. Every Excel value is within 5.2E-12 of the math (statistics.md "BINOM.DIST and NEGBINOM.DIST at a
  large n: basym and Loader, within 1E-11 of the math (T-3352)"), so decision 0188 R3 does not apply.
- Root-cause grouping. The cells cross several TOMS 708 bratio branches. Classify each cell with `bratio::dispatch`
  before fitting:
  - near the mean with both shapes above 100 (large-n A2, A8, A126, A138, A140, A142, ...): `basym`. That is the
    kernel T-3597 models for BETA.DIST and F.DIST, and T-3597 excludes BINOM.DIST and NEGBINOM.DIST from its scope.
    Use one model for both (directive 3). If T-3597 has landed a basym model, score it here first, and do not fork
    basym's arithmetic.
  - mean - 3 sd at p 0.3 and 0.5 (A6, A12, A22, ...): expected `bfrac` (|λ| above 3% of the smaller shape).
  - NEGBINOM.DIST at r = 10 (A146, A170, A182, A206): the swapped bup + bgrat path at b0 = 1. Only bgrat at b = 1/2 is
    ported (`bratio/mod.rs` `Branch::Bgrat`); elsewhere the engine falls back to the accurate kernel. T-3582 (done)
    recorded Excel's bgrat increment at b != 1/2 as white noise of sd 5.7 to 8.5 units of 2^-53. Treat this as an
    open lead, not a model.
  - the sweep's small-n cells (n up to 2631: A16, A21, A33, A58) and moderate NEGBINOM.DIST points: expected the walk,
    bfrac or bup/bpser. Classify them before you assume a branch.
- Evidence: 53 cdf cells of probes/t3352-binom-negbinom-large-n and 71 of probes-numeric/t3352-binom-negbinom-sweep
  fail today (of 104 and 147 cdf cells). Their passing cdf neighbours are positive controls. Train on the large-n probe
  and hold out the sweep, or split by a hash fixed before scoring. If a branch has fewer than about 20 cached points,
  do not fit it blind ("Needs an oracle sweep": file a research sweep task as T-3960 does for the tail). List its
  cells in the Outcome.
- Code: `crates/rustsheet-engine/src/functions/statistical/binom.rs` (the cdf entry points),
  `bratio/{mod,basym,bfrac,bup,bgrat}.rs`. Method: docs/research/excel-kernel-playbook.md. T-3774's
  `docs/research/t3447/pmf_regress.py` has `rnd`, `stirlerr` and `bd0` helpers, and so does
  docs/research/t2464/r3-loader-bd0/r3lib.py.

Cells this task owns (cdf only; T-3774 keeps the pmf cells of both cases):
- probes/t3352-binom-negbinom-large-n.case.json (53; visibly off: A2, A6, A8, A12, A126, A130, A138, A140, A142, A146, A170, A182, A206): Calc!A2, A4, A6, A8, A12, A14, A18, A22, A24, A28, A32, A40, A44, A46, A48, A50, A54, A58, A64, A66, A68, A76, A82, A94, A100, A106, A112, A118, A124, A126, A128, A130, A132, A134, A136, A138, A140, A142, A144, A146, A154, A158, A160, A162, A166, A170, A172, A174, A182, A184, A194, A206, A208
- probes-numeric/t3352-binom-negbinom-sweep.case.json (71; visibly off: A16, A21, A33, A58, A67, A69, A75, A77, A78, A103, A106, A107, A136, A168, A177, A222, A232, A238, A261, A269, A285): Calc!A5:A6, A16, A21, A33, A58, A61:A62, A67, A69:A70, A75, A77:A78, A82:A83, A99, A101, A103, A106:A107, A111, A114, A119:A121, A123, A126, A131, A136, A139, A141, A145, A149, A152, A159, A162, A167:A168, A177, A179, A182:A183, A185:A186, A193, A196:A197, A201:A202, A205:A207, A212, A214, A216, A222:A223, A225, A232, A234, A237:A238, A248, A251, A255, A261, A265, A269, A285, A291

## Acceptance criteria
- [ ] The Outcome has a branch table (bratio branch, cells, visibly off), from `bratio::dispatch` on each listed cell.
- [ ] Each of the 34 visibly-off cells above is within half a unit of the 15th significant digit of Excel, or is in
      statistics.md as UNVERIFIED with the hypotheses ruled out and their exact / within-4-ulp / within-15-digit counts.
- [ ] The probes/t3352-binom-negbinom-large-n cdf lines that pass leave eval/baseline/probes.txt (`make eval-baseline`).
      No new known-failure line appears in any eval/baseline file. No gated BETA.DIST, F.DIST, POISSON.DIST,
      BINOM.DIST.RANGE or BINOM.INV cell regresses (they share bratio and `loader`).
- [ ] The Gate A floor on probes-numeric/t3352-binom-negbinom-sweep still holds (0 cells outside both envelopes).
- [ ] R4 guards (reviewer-excel-fidelity check 7, decision 0188), in the Outcome, for every fitted piece:
  - hold-out counts: train against hold-out (bit-exact, within 4 ulp, within 15 digits) for the chosen candidate and
    for the correctly rounded value, from a split fixed before scoring, with the margin stated before scoring;
  - a fresh sweep over the owned region, generated and oracle-filled (excel-graph) after the model is frozen, with
    its result recorded;
  - the Gate A envelope result on the fresh sweep and on every owned cell, with every point outside it listed;
  - every fitted constant or cut carries `FITTED(T-3959, <n> cells, <range>)` and applies only inside that range,
    with accurate math outside it;
  - no fit below the oracle's reproducibility floor (4 ulp web against desktop, decision 0112) without a
    desktop-backed reproducibility check or an owner ruling (R4.4);
  - no keep-matched branch that special-cases evidenced points.
- [ ] statistics.md "BINOM.DIST and NEGBINOM.DIST at a large n" records what was found for the cdf, by branch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The pmf cells and Loader's bd0 term (T-3774).
- The p < 0.01 tail cells where Excel is hundreds or more ulp off (T-3961).
- BETA.DIST and F.DIST at basym points (T-3597): share its model, do not duplicate it.
- Bit-level residue on cells already within 15 digits: once the visible target is met, file it as a follow-up with
  `defer: phase-1b`.

## Visible-parity stage
Decisions 0083 and 0114 (`visible`): done when each of the 34 visibly-off cdf cells above is within half a unit of
the 15th significant digit of Excel, or is an UNVERIFIED statistics.md entry naming the ruled-out hypotheses.
