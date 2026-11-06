---
id: T-4467
title: "BESSELY order 4 next to its tiny-x overflow cut is 4 ulp below ours: find the rounding of the recurrence that reproduces probes/t4458-bessely-tiny-x-cuts Calc!D32"
milestone: M9
lane: functions-engineering
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
BESSELY matches Excel's last digits next to its tiny-x overflow cut, starting with `BESSELY(2.036E-77,4)`, without
losing a cell that passes today.

## Context
- From T-4458 (probes/t4458-bessely-tiny-x-cuts, excel-graph, 2026-11-06; engineering.md "BESSELY at a tiny x and a
  small order"). The cut of every order 2 to 10 is the double model's. The one miss is Calc!D32:
  - `BESSELY(2.036E-77,4)` is -1.7783214437962194E+308 in both fills of the case;
  - ours, the double model `b * (2j * (1/x)) - a`, gives -1.7783214437962202E+308, 4 ulp away;
  - its neighbours 2.035E-77 and 2.037E-77 match, as do every other cell of the case and the 263 cells of
    probes/t4458-bessely-tiny-x-bisect.
- Reassociating the step does not reproduce it: `(2j/x)*b`, `(2j*b)/x` and exact steps all give
  -1.77832144379622E+308 or ours. Candidates to test, in order:
  - `y1`'s `1/x` and `ln x` terms kept in x87 extended precision;
  - the step's product kept extended before the subtraction;
  - an extended `inv_x`.
- That choice is a rounding model, so it is kind (c) of reviewer-excel-fidelity check 7 and owes decision 0188 R4's
  guards. The hold-out split must be fixed before scoring, and the fresh sweep must sit under eval/suites/probes.
  The miss is exactly 4 ulp, which is the edge of R4.4's web-vs-desktop reproducibility floor
  (docs/research/oracle-calibration.md). So first show that it reproduces, on a third fill and on desktop if one is
  available. If it varies, it is below the floor and should be recorded, not fitted.
- In the same fill, probe order 1 over the normal band [2.2250738585072014E-308, 3E-308), for example 2.5E-308,
  2.3E-308 and 2.2250738585072014E-308. T-4458 stopped at 3E-308, so "order 1 answers at every normal x" is open
  there.
- Owned cells: probes/t4458-bessely-tiny-x-cuts Calc!D32.

## Acceptance criteria
- [ ] probes/t4458-bessely-tiny-x-cuts Calc!D32 passes, or engineering.md records why it is no target, with no other
      BESSELY cell changing verdict (probes/t4458-*, probes/m9-bessel-order-limits, probes-numeric/m9-bessel,
      ironcalc-calc BESSEL.xlsx, gen-m9).
- [ ] If a rounding model is chosen by scoring, the Outcome shows the R4 guards: the hold-out, a fresh sweep, the Gate
      A result, the reproducibility floor, and no keep-matched branch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- BESSELY at a subnormal x (engineering.md: unreliable on excel-graph, no target).
