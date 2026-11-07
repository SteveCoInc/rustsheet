---
id: T-4338
title: "Owners for three 4-ulp IRR and MIRR cells no task names: t3411-irr-r4-fresh-2 A138 and the web side of MIRR's desktop root (Phase 1b)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
The three cells below are bit-exact, moved to a narrower open task, or listed in the Outcome with Excel's value,
ours, the 50-digit math and why the engine keeps its value.

## Context
- Gap triage 2026-11-05 (`make eval` on origin/main 4e3993c9e). Every visibly-off cell has an open owner
  (`make unowned-off ALL=1` prints 0). Checking every failing cell of report.json against the open tasks (the
  `scripts/exit_owners.py` ownership rule) leaves these three financial cells with no open owner. Each is 4 ulp off
  and within 15 significant digits, so decisions 0114 item 3 and 0188 R5 put the task under `defer: phase-1b`.
- **IRR, 1 cell.** probes-numeric/t3411-irr-r4-fresh-2 A138: a 14-flow series with a positive first flow and 13
  negative ones, guess 46.8743397434183. Excel 0.3838361826002541, ours 0.3838361826002543
  (`make eval-show CASE=probes-numeric/t3411-irr-r4-fresh-2.case.json`). T-3411 (done) drew this sweep. The open
  IRR tasks that list other cells of it (T-3408, T-3548, T-3407, T-3900, T-4213) do not name A138; T-3548 is blocked
  as time-boxed, and decision 0267 accepts the cells it lists. Move A138 to T-3408 if its secant stop is the shape
  T-3408 matches.
- **MIRR, 2 cells.** probes-numeric/power-mirr-root A1 (`MIRR({40,-5,-5,-5,-5,-5},0.1,0.2)`) and A2 (the same
  flows from C1:H1): Excel for the web 0.3933280241536674, ours 0.3933280241536672. docs/excel-semantics/financial.md
  (MIRR, "Web vs desktop (T-0959)") records this as a desktop/web conflict: Mac desktop Excel 16.0300 gives ...672
  (ironcalc-calc MIRR.xlsx Sheet2!N16, cached), the engine keeps the pre-T-0959 root (`power_with_libm_exp`) that
  matches it and all 467 fit cells, and decisions 0030 and 0039 keep passing desktop evidence. The Windows desktop
  value was T-1075's question (M14), and T-1075 is dropped. This task does not change MIRR unless new evidence
  settles the conflict; its Outcome records that the two cells stay as the web side.

## Cells
- probes-numeric/t3411-irr-r4-fresh-2.case.json (1): Calc!A138
- probes-numeric/power-mirr-root.case.json (2): Calc!A1, A2

## Acceptance criteria
- [ ] A138 is bit-exact, or moved to the IRR task of its path (T-3408, named in both files), or listed in the Outcome
      with Excel, ours, the 50-digit root and the secant step it stops on.
- [ ] power-mirr-root A1 and A2 stay as recorded in financial.md, unless Windows desktop evidence is in eval/ by then;
      a change to MIRR's root keeps ironcalc-calc MIRR.xlsx and the 467 fit cells bit-exact.
- [ ] Any IRR change that moves A138 meets the R4 guards of reviewer-excel-fidelity check 7 (decision 0188):
      hold-out counts, a fresh sweep, Gate A envelope result, a `FITTED` tag and range, the reproducibility floor and
      no keep-matched branch.
- [ ] Fixed cells' lines disappear from eval/baseline/probes-numeric.txt with no new ones.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other cells of t3411-irr-r4-fresh-2 (their owners above) and MIRR's fit (T-1687).
