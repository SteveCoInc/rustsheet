---
id: T-3413
title: "CUMPRINC's short-range last bits that CUMIPMT's PMT·m - CUMPRINC amplifies (T-3118's regressed and still-off cells)"
milestone: M9
lane: functions-financial
status: blocked
priority: P2
size: S
tier: opus
depends_on: [T-3118]
---

## Goal
Our CUMPRINC has Excel's bits on the short ranges whose CUMIPMT is still off, so those interests are right too.

## Context
Since T-3118 CUMIPMT is `PMT·m - CUMPRINC` (financial.md "CUMIPMT and CUMPRINC over many periods"),
and our PMT has Excel's bits (T-3014). Every CUMIPMT cell still off is our CUMPRINC 1 to 4 ulps from
the x that solves `PMT·m - x = Excel's CUMIPMT`, amplified by the cancellation (an ulp of a principal
of 1,000 is hundreds of ulps of an interest of 1 to 100). The fin-cumipmt-gate-edges cells as a cell-list line
(gap triage 2026-11-01h: `exit_owners.py` reads hyphen spans by their ends only, decision 0136, so it listed the
inner ones as owned by deferred T-3216 alone):
- probes/fin-cumipmt-gate-edges.case.json (13): Sheet1!A1:A4, A6, A7, E3:E10, H3

Cells: probes/fin-cumipmt-gate-edges A1-A4,
A6, A7 (type 0, rates 6E-5 to 1.5E-4, periods 3-10; x is -3 to +1 ulps from ours), E3-E10 (type 1,
8E-6 to 1E-4; -4 to +2 ulps), H3; fin-cumipmt-overflow-and-forms A8, A12 (x one ulp below our
-1570.8656559792591), A14 (+2 ulps); fin-cumipmt-tiny-rate-structure F2 (x is Excel's own probed
CUMPRINC G2); CUMPRINC_CUMIPMT.xlsx G2, G3; gen-m9 6e18550e9fc9c1b9 A329, 79d87fc527927626 A352.
T-3118's regressed cells are listed in decision 0197. The probes that pair the two functions
(tiny-rate-structure) carry CUMPRINC's own expected values for the fit.
The rest of tiny-rate-structure (37 visibly-off CUMIPMT cells and their CUMPRINC partners, all but F2 and G2) is
T-3731's (gap triage 2026-10-31k, T-3552 until its attempt 1 closed). Both tasks change the same short-range CUMPRINC form, so the one that lands second
starts by re-measuring its cells on the first one's main.

## Cells added by gap triage (2026-11-04)
`make eval` on origin/main 3c8d8c7d3. These are two more CUMIPMT cells of the same cancellation, and no open task lists
them. Both are Mac 16.0300 cached values, and their CUMPRINC partners are in T-3216's list.
- G29 `CUMIPMT(0.0083,36,8000,3,10,0)`: Excel -459.6252739600568, ours -459.6252739600561. Against the 50-digit math
  -459.62527396005656, Excel is 4.3 ulp below and ours 7.7 ulp above. The gap of 7E-13 is past half a unit in the
  15th digit, so the cell is visibly off, but only at the 15th digit (decision 0267 Tier D). Our H29 is 3.5 ulp from
  Excel's -1604.273009770884, and `PMT·8 - CUMPRINC` carries that gap into the interest.
- G31 `CUMIPMT(0.0083,36,8000,3,10,1)`: Excel -455.8417871269037, ours -455.84178712690345 (within 15 digits).
- ironcalc-calc/FINANCIAL/CUMPRINC_CUMIPMT.xlsx (2): Sheet1!G29, G31

## Blocked because
Time-boxed (decision 0056), attempt 1 (opus run 2026-11-05): no principal form tried makes every listed CUMIPMT
cell bit-exact, and the best ones trade cells, so the kernel is unchanged. Attempt 2 should start from "Next step"
below, not from more orderings of these forms.

## Attempts
Attempt 1 (2026-11-05, docs/research/t3413/fit.py and fit2.py; float64 with glibc exp/expm1/log1p). Re-measured on
origin/main 6523a495: the visibly-off cells of fin-cumipmt-gate-edges (A1-A4, A6, A7 type 0 at 6E-5 to 1.5E-4 over
60 periods, periods 3-10; E3-E10 type 1 at 8E-6 to 1E-4 over 100) and fin-cumipmt-tiny-rate-structure (H3, the
single-period type 1 A2-A12 at 2E-5, F2) are still off by 1 ulp of the principal (128 to 4096 ulps of the
interest). Scored as Excel's `-pmt·m - CUMPRINC` with three payment forms each (`(pv/d)·r`, `pv·r/d`, and for type 1
those over `1+r` and `pv·(r/(1+r))/d`, `d = -expm1(-nper·L)`), bit-exact cells out of 7 type 0 and 17 type 1:
- the engine's per-period sum `Σ pmt·exp((k-1-nper)·L)`: 2/7 and 2/17;
- the closed form `pmt·(exp((end-nper)·L) - exp((first-1-nper)·L))/expm1(L)`: 0/7 and 0/17 (100 to 800,000 ulps);
  `pmt·exp((first-1-nper)·L)·expm1(m·L)/expm1(L)`: 3/7 and 2/17;
- type 0 balance drops `(pv - pmt/r)·(expm1((s-1)·L) - expm1(e·L))` and variants: at most 3/7;
- T-2514's geometric sum `-((pmt·((1-v^m)/r·(1+r)))·v^(nper-end+1))`: 4/7 and 7/17;
- its orderings (fit2.py, 17 each): best type 0 `(pmt·a)·((1+r)/r)·v` 5/7 (A3 and H3 still off by 256 and 384);
  best type 1 `pmt·(a/r·(1+r)·v)` and `pmt·a/r·((1+r)·v)` 9/17, which miss different cells.
So the misses are one rounding of the principal that no single operation order removes, as T-3731 found for the
tiny-rate structure.
- Next step for attempt 2: T-3731's "solve for the factor": for each rate, find in 50-digit math the interval of
  payments (and separately of discount factors `v^j`) that makes each order give Excel's interest on every cell
  of that rate (A2-A12 at 2E-5 share one payment), and test whether it contains an 80-bit or `1/(1+r)`-based
  rounding. A non-empty interval for one order across a rate's cells would name the form; an empty one for every
  order means a second rounding in the chain.
- Any form fitted here must meet the R4 guards (hold-out split before scoring, fresh sweep) and clear the
  web-against-desktop reproducibility floor, since each miss is one principal ulp.

## Acceptance criteria
- [ ] The listed CUMIPMT cells are bit-exact through a CUMPRINC change, or financial.md records why
      the remaining ones are not.
- [ ] No regressions (`make eval-baseline`); `make check` green; reviewers approve.

## Out of scope
- CUMPRINC near period 1E15 (T-3141) and the bond/depreciation bits of T-1374.
