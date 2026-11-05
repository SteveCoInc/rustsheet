---
id: T-3413
title: "CUMPRINC's short-range last bits that CUMIPMT's PMT·m - CUMPRINC amplifies (T-3118's regressed and still-off cells)"
milestone: M9
lane: functions-financial
status: in-progress
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

## Acceptance criteria
- [ ] The listed CUMIPMT cells are bit-exact through a CUMPRINC change, or financial.md records why
      the remaining ones are not.
- [ ] No regressions (`make eval-baseline`); `make check` green; reviewers approve.

## Out of scope
- CUMPRINC near period 1E15 (T-3141) and the bond/depreciation bits of T-1374.
