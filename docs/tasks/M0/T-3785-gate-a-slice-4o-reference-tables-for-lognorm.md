---
id: T-3785
title: "Gate A slice 4o: reference tables for LOGNORM.DIST, LOGNORM.INV, EXPON.DIST, WEIBULL.DIST, FISHER and FISHERINV"
milestone: M0
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: [T-3783]
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
LOGNORM.DIST, LOGNORM.INV, EXPON.DIST, WEIBULL.DIST, FISHER and FISHERINV each get a committed, oracle-filled `s1`
Gate A table with a cited mpmath definition, and `make check` runs them. Every failing point has an owner.

## Context
- Gate A (decision 0188 R1(a), SPEC-0029 "Gate A holds"): on main (fafb6c6f) these six M9 statistical functions are
  missing tables, and no open task or pending branch names them. They are the closed-form continuous distributions
  and transforms around LN and EXP.
- It depends on T-3783 because LOGNORM.DIST and LOGNORM.INV reuse its normal cdf and quantile definitions
  (`defs/normal.py`): one definition per concept (CLAUDE.md directive 3).
- The math, at the double of each written literal: the lognormal as the normal of ln x; the exponential and Weibull
  cdf and density in closed form (1 - e^-t through expm1 at working precision); FISHER as atanh and FISHERINV as tanh.
- Semantics the domain and sampler must read (statistics.md):
  - "LOGNORM.INV is EXP of NORM.INV's quantile (T-0925)", and "One incomplete-gamma kernel ... (T-1778)":
    `LOGNORM.DIST(x,m,s,TRUE)` = `NORM.DIST(LN(x),m,s,TRUE)`.
  - "FISHER, FISHERINV, EXPON.DIST and HARMEAN are Excel operators around LN and EXP (T-2437)": FISHER near 0 is
    Excel's cancellation, not atanh. `FISHER(1.49172367472348E-11)` is 5 digits of 15 off atanh, and
    `FISHER(1.02759470998055E-16)` is 0. That is a relative error past decision 0188 R3's 1e-4 against the math, so
    those points are R3 candidates. List them for an owner ruling, or leave the region out of the table and say so
    in the definition. Either way, do not tune the definition to Excel's bits.
  - "WEIBULL.DIST sweep (T-1046)", "WEIBULL.DIST's cdf switches from expm1 to 1-EXP(-t) near t = 1 (T-2239)" and
    "WEIBULL's density divides by l^k: #DIV/0! when it underflows, #NUM! when it overflows (T-2239)": these give the
    error domain. The density's `#DIV/0!` where its limit is 0 is an Excel error that the table matches; it is not a
    math value.
  - "Normal densities are #NUM! once the square overflows (T-2435)" applies to LOGNORM.DIST's density.
- Owners for failing points: T-0925 (LOGNORM.INV central), T-0968 (LOGNORM.DIST cdf residue, blocked), T-0975
  (LOGNORM.DIST density), T-0978 (WEIBULL.DIST last digits), or a new task.
- Tools: as in T-3549 and T-3586. The definitions go in a new `defs/continuous.py`; FISHER and FISHERINV can live there
  or in their own module.
- Why opus: FISHER's evidenced cancellation is an R3 or out-of-table call, and three error domains (Weibull's l^k,
  the overflowing square, LN of x at 0) have to be read from the evidence.

## Acceptance criteria
- [x] One definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, each citing its sources, with unit
      tests against Microsoft's published examples or the semantics entries' values. The lognormal definitions call
      `defs/normal.py` and do not copy it.
- [x] `eval/accuracy/tables/<FUNCTION>/s1.{case,math,oracle}.json` for each of the six functions, generated, filled
      (excel-graph) and committed. `rustsheet-eval accuracy` reports `missing tables` lower by 6, with 0 failing points
      and 0 table defects.
- [x] Every point outside the envelope is a `known-failures.tsv` line with an open owner task. The FISHER near-zero
      choice is stated, and any R3 candidate is listed in the Outcome for an owner ruling and not marked in this PR.
- [x] The probes-numeric math sidecars are regenerated with the new definitions and committed.
- [x] `make check` green; reviewers approve.

## Out of scope
- The legacy names LOGNORMDIST, LOGINV, EXPONDIST and WEIBULL (T-3786 rules on legacy names).
- HARMEAN and the other sample-array statistics (T-3786).
- Changing any engine kernel.

## Outcome
- `tools/src/rustsheet_tools/accuracy/defs/continuous.py`: LOGNORM.DIST, LOGNORM.INV, EXPON.DIST, WEIBULL.DIST, FISHER
  and FISHERINV, each citing its statistics.md entries. The lognormal definitions call normal.py's `cdf`, `density`
  and `standard_quantile` (and its probability draw, now public as `inverse_probability`); they copy none of it.
  EXPON.DIST and WEIBULL.DIST are -expm1(-t) and the closed-form densities at the working precision; FISHER is atanh
  and FISHERINV tanh.
- Error domain, from the evidence: WEIBULL.DIST is `#NUM!` once t = EXP(alpha*LN(x/beta)) overflows, and its density
  `#DIV/0!` where beta^alpha underflows to 0 and `#NUM!` where it overflows, x = 0 included (T-2239, T-3312); at
  x = 0 the value is 0 for every shape (t1048-weibull). LOGNORM.DIST takes x > 0 and sd > 0, LOGNORM.INV p in (0, 1),
  EXPON.DIST x >= 0 and lambda > 0, FISHER (-1, 1). Excel agrees on every error point drawn: 60 `#NUM!` and 8
  `#DIV/0!` cells across the six tables, each a domain error of the math too.
- **FISHER near zero:** left out of the tables, not marked R3. FISHER and FISHERINV points with 0 < |x| < 1E-6 are
  redrawn (0 itself stays). Excel's 0.5*LN((1+x)/(1-x)) and (EXP(2y)-1)/(EXP(2y)+1) cancel there, about 1E-16/|x|
  relative; past R3's 1e-4 below about 1E-12 (`FISHER(1.49172367472348E-11)`, `FISHER(1.02759470998055E-16)` = 0,
  probes/t2437-fit-separators A1, A4). The cut is 1E-6, not 1E-8 as GAUSS's, because at 1E-8 the doubles' error is
  still 5E-9, above the gate's 1e-9: Excel's `FISHER(1E-08)` is 9.999999950247593E-9 (t2437-fit-separators A19).
- Also left out, as unprobed or ill-conditioned (stated in the module doc): points whose relative condition number
  times 2^-52 passes 1e-10 (LOGNORM.DIST's ln x - mean against a small sd, LOGNORM.INV's mean + sd*z, WEIBULL's
  alpha*LN(x/beta) and its density's powers); EXP-overflow neighbourhoods; LOGNORM.INV past the double range;
  LOGNORM.DIST densities whose exp(-z^2/2) underflows before the division by x*sd lifts it back (the engine gives 0,
  no cell says what Excel does); EXPON.DIST with lambda*x past 1E+300; WEIBULL with x/beta subnormal, a beta^alpha
  between EXP's underflow and 1E-300, or a density intermediate outside the normal range.
- **R3 candidates, for an owner ruling (not marked; T-4317):** FISHER and FISHERINV near 0. In
  probes/t2437-fit-separators, Excel is more than 1e-4 off the math at FISHER A2, A4, A5, A7, A8, A10, A11 (|x| from
  1E-16 to 1.2E-14, up to 1.0: `FISHER(1.02759470998055E-16)` is 0) and FISHERINV A21 to A32 and A34 to A36 (|y| from
  1E-16 to 8E-15, 2.2E-3 to 0.20). The engine matches Excel there (T-2437). No other cell of the six functions in the
  regenerated sidecars, outside these and t2437-fit-separators's other near-0 FISHER and FISHERINV cells (inside
  R3, listed in T-4317), is more than 1e-9 off the math.
- In the tables, the worst Excel-vs-math gap is 4.0E-11 relative (`FISHER(1.40857451285715E-06)`, at the cut);
  otherwise WEIBULL.DIST 2.1E-13, LOGNORM.DIST 3.0E-13, EXPON.DIST 2.1E-14, LOGNORM.INV 6.4E-14, FISHERINV 1.4E-11.
- Tables: `eval/accuracy/tables/{LOGNORM.DIST,LOGNORM.INV,EXPON.DIST,WEIBULL.DIST,FISHER,FISHERINV}/s1`, 120 or 200
  points each (960 in all), filled from excel-graph. `rustsheet-eval accuracy`: 18,032 points in 113 tables, 0 failing,
  0 table defects, no new known failures; missing tables 144 -> 138. 53 LOGNORM.DIST and 6 LOGNORM.INV points are in
  the envelope by the math only (the engine is not within 15 digits of Excel there; T-0975 and T-0925 own those last
  digits), every other point by both.
- Sidecars: the probes and probes-numeric math sidecars of the cases with one-call cells of the six functions are
  regenerated (only the cases `make eval-math-check` reported stale).
- Unit tests: tools/tests/test_accuracy_continuous.py (Microsoft's examples, the T-1040, T-1046, T-0925 and T-2239
  values, the WEIBULL error domain, and the FISHER draws avoiding the cancellation region).
- Follow-up: T-4317 (owner ruling on R3 for FISHER and FISHERINV near 0, blocked on the owner).
- Eval delta: none in the gated suites (tools, tables and sidecars only).
