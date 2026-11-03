---
id: T-2647
title: "F.DIST pdf at a tiny x: Excel's last digits where Y = density·x is subnormal (probes-numeric t1864-f-pdf-tiny-x)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-1864, T-3926]
campaign: visible
---

## Goal
F.DIST's pdf at a tiny x gives Excel's value to 15 digits where Excel's stored Y = density·x is subnormal. The cells
are probes-numeric t1864-f-pdf-tiny-x A5, A6, A10, A17 and A20, and the 86 subnormal-Y cells of the t2586 edge sweeps
listed under Context.

## Context
Found by T-1864's edge probes (probes-numeric t1864-f-pdf-tiny-x, t1864-f-pdf-edges, excel-graph, 2026-10-25).
T-1864 implements round 5's range steps (docs/excel-semantics/statistics.md, the T-2513 F density entry: Excel stores
Y = density·x, takes F = fl(Y/x), and an F below 2^-1022 is 0) in `fisher.rs` `excel_density_range`, re-forming a
subnormal Y as fl(fl(density·x)/x). That gives Excel's large subnormal errors (A6
`F.DIST(1E-215,3,1000,FALSE)`: Y = 13·2^-1074, 2.1% below the exact density; A17 `F.DIST(1E-65,10,1000,FALSE)`:
Y = 3·2^-1074, 11.6% above), but Excel's value is not exactly fl(Y/x):
- A6: Excel 6.42285339593616E-108, ours (13·2^-1074)/1E-215 = 6.422853395936205E-108 (7E-15 relative).
- A17: Excel 1.4821969375236873E-258, ours 1.4821969375237397E-258 (3.5E-14).
- A10 `F.DIST(1E-160,4,1000,FALSE)`: Excel 4.007860519064344E-160, ours 4.007860519064192E-160.
- A5 `F.DIST(1E-210,3,1000,FALSE)`: Excel 2.0745191673458065E-105, ours 2.0745191673458212E-105.
Excel's F·x/2^-1074 is 12.99999999999991 at A6, so its Y/x step is not a plain double division of the stored
subnormal (an extended-precision Y, a division by a rounded 1/x, or the EXP of a log difference are candidates).
Probe cells with a known subnormal Y (x a power of two, so Y/x is exact in double) to tell them apart.

Gap triage 2026-11-03 (`make eval`, report.json; `scripts/exit_owners.py --all-off`) found the same region in the
t2586 edge sweeps:
- 86 R1(c)-counted cells with a math value where Excel is 2.6E-9 to 41% off the 50-digit density and the engine is
  within 5E-14 of Excel.
- Example: t2586-r5-f-edges-a Calc!B771 `F.DIST(A771,3,5,FALSE)`, x = 2.43E-216. Excel 4.071920683170428e-108, ours
  4.071920683170405e-108, math 3.687267596984554e-108.
- The x are exact-input cells, so Y/2^-1074 is small: about 2 at B771.
- T-3158, T-3173 and T-3174 (`defer: phase-1b`) name these cases whole. T-3173 already points this region here
  ("Y = density·x is subnormal (T-2647)"), so this task owns these cells, and the Phase 1 `owners` criterion stops
  reading them as unowned.

Owned cells:
- probes-numeric/t1864-f-pdf-tiny-x.case.json (5): Calc!A5, A6, A10, A17, A20
- probes-numeric/t2586-r5-f-edges-a.case.json (52): Calc!B771:B783, Calc!B811:B823, Calc!B851:B863, Calc!B891:B903
- probes-numeric/t2586-r5-f-edges-hold.case.json (26): Calc!B217:B229, Calc!B257:B269
- probes-numeric/t2586-verify-r5-f-edges-hold.case.json (8): Calc!B519, B520, B534, B535, B549, B550, B564, B565

T-3926's cases t3926-f-density-subnormal-y (fit) and t3926-f-density-subnormal-y-hold (hold-out, never used for fitting) are owned whole by this task.

Needs an oracle sweep: the cached cells are all at a general x, where the candidates for Excel's Y/x step give nearly
the same value. T-3926 adds power-of-two x, where Y/x is exact in double, and a hold-out.

Excel is the target here (directive 1). Excel is more than G = 1e-4 off the math at most of these cells, but decision
0188 R3 lists no such case for the F density, and only an owner ruling could add one. The work reproduces Excel's
step, which makes it a numeric fit under decision 0188 R4.

## Acceptance criteria
- [ ] The owned cells are within 15 digits of Excel (or bit-exact), and no passing F.DIST cell of any suite changes.
- [ ] R4 guards (reviewer-excel-fidelity check 7, decision 0188):
  - the hold-out counts (train against test exact and within-15-digit rates; T-3926's hold-out case is test only);
  - a fresh log-uniform sweep, filled after the model is frozen, with its result in the Outcome;
  - the Gate A envelope result on every owned cell and on the T-3926 cases;
  - every fitted constant or cut carries `/// FITTED(T-2647, <n> cells, <range>)`, and the code falls back to accurate
    math outside the measured range;
  - no fit below the oracle's reproducibility floor (4 ulp web against desktop, decision 0112);
  - no keep-matched branch that special-cases evidenced points.
- [ ] The rule is recorded in docs/excel-semantics/statistics.md ("F density at large degrees of freedom (T-1864)").
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the owned cells are visibly off, numbers more than half a unit in the 15th significant digit from
Excel's value. Excel is the inaccurate one here (up to 41% off the math), so the stage is `visible`: reproduce Excel's
step (decision 0114). The target is every owned cell within 15 digits of Excel. Bit-level criteria still unmet then
move to a follow-up filed with `defer: phase-1b`.

## Out of scope
- The bit-level last ulp at a huge df (T-2642); Excel's large-shape noise (T-2586).
- The other t2586 cells (T-3158, T-3173, T-3174).
