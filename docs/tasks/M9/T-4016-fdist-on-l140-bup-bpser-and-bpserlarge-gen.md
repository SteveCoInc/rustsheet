---
id: T-4016
title: "FDIST on L140 bup+bpser and BpserLarge: gen-m9 2801c1c11c21e134 A374 and 285d87cea1aaa73c A34, the Gate B face of deferred T-3472 (accuracy)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
The two gen-m9 FDIST cells below are within half a unit of the 15th significant digit of Excel's value. Excel is
within 2E-16 of the math on both, and the engine is about 1E-15 off. No cell that matches Excel bit for bit on the
same branches today moves off.

## Context
- Gap triage 2026-11-04 (`make eval` on origin/main 30a36f59, excel-graph). Both cells are visibly off in gen-m9, a
  Gate B suite (decision 0211 counts every visibly-off cell there whatever its math). Their only working owner is
  T-3472, which its run (#3059, 2026-11-04) deferred to Phase 1b.
  - A34 is in `make unowned-off ALL=1`'s 109 deferred-only cells.
  - A374 is missing from that list only because T-3952 names its case and T-3198's triage table names the cell.
    Neither task works it: T-3198 moved its gen-m9 cells to T-2221 (deferred), and T-3952 owns only A219.
  - A374 is one of FDIST's 3 visibly-off Gate B rate cells. T-4015 and T-4018 own the other two.
  - This task takes both cells. T-3472 and T-2221 keep only the bit-level residue, and both name this task.
    Precedents: T-3952 and T-3954.
- The cells (`make eval-show CASE=gen-m9/<case>.case.json`, NumericClose). References are 50-digit mpmath 1.3
  `betainc` at the double arguments, with the right tail as I_v(d2/2, d1/2). Relative errors are given as
  Excel / ours.
  - gen-m9/2801c1c11c21e134 Calc!A374 `=FDIST(3.14159,9,6)`: Excel 0.0884857396733851, ours 0.08848573967338502, math
    0.088485739673385121636; -1.8E-16 / -1.12E-15. The correctly rounded value, 0.08848573967338512, is within
    15 digits of Excel. Branch: `L140BupBpser`, swapped, at a0 3, b0 4.5, x0 0.175. This is the same branch as
    `F.DIST.RT(3,5,10)` on T-2220's right-tail sweep (a T-3472 cell, deferred).
  - gen-m9/285d87cea1aaa73c Calc!A34 `=FDIST(40,7,MINUTE(0.2))`, which is `FDIST(40,7,48)`. Excel
    6.448979666396885E-18, ours 6.448979666396879E-18, math 6.4489796663968854534E-18; -3.1E-17 / -9.9E-16. The
    correctly rounded value is Excel's. Branch: `BpserLarge`, I_v(24, 3.5) at v = 0.146 (b0·x0 = 0.51).
- Why this is not a one-line kernel swap (T-3472's "Progress notes"):
  - Both branches sit in the residual of models that are bit-exact elsewhere:
    - T-3198's L140 power sum: 674 of 816 exact;
    - T-3879's split-z bpser: 1,139 of 1,139.
  - So moving every point of the branch toward the math costs exact cells, where Excel's own arithmetic is what we
    match.
- The work is to find which step of each model loses about 1E-15 at these two points and is accurate in Excel:
  - for A374, the bup prefix (n terms) or the bpser tail of L140;
  - for A34, BpserLarge's power factor or its series.
  Then make that step accurate without changing the steps the exact cells depend on.
- Evidence on file:
  - T-3198's L140 argument probe (414 rows, the L140 F points);
  - T-3879's fresh bpser split sweep and T-3570's round-5 bpser probes;
  - T-2220's right-tail sweep.
  (Cases are named here by their tasks, so this task does not claim their cells for `exit_owners.py`.)
- Code: crates/rustsheet-engine/src/functions/statistical/bratio/ (`dispatch`, the L140 bup+bpser path and
  `bpser`), and `fisher.rs`. Rules: docs/excel-semantics/statistics.md, "Incomplete beta L140 bup+bpser at the F
  points" and "Incomplete beta is TOMS 708 `bratio`".

Cells this task owns (split with T-3472 and T-2221, which keep only the bit-level residue):
- gen-m9/2801c1c11c21e134.case.json (1): Calc!A374
- gen-m9/285d87cea1aaa73c.case.json (1): Calc!A34

## Visible-parity stage
`campaign: accuracy` (decisions 0083, 0114). Excel is within 2E-16 of the math and ours is about 1E-15 off. The stage
is met when:
- A374 and A34 are within half a unit of the 15th significant digit of Excel's value;
- no cell of gen-m9, ironcalc-calc, T-3198's L140 probe or the bpser probes that is within 15 digits today goes visibly
  off.

Bit-level criteria still unmet then go back to T-3472 (`defer: phase-1b`).

## Acceptance criteria
- [ ] gen-m9/2801c1c11c21e134 Calc!A374 and gen-m9/285d87cea1aaa73c Calc!A34 leave eval/baseline/gen-m9.txt or are
      within 15 digits of Excel. The Outcome shows the `make eval-show` output for both and names the step changed.
- [ ] The bit-exact counts of T-3198's L140 model on its L140 argument probe, and of T-3879's split-z bpser on its
      fresh sweep, do not fall. No new known-failure line appears in any eval/baseline/<suite>.txt
      (`make eval-baseline` refuses regressions).
- [ ] If any cut, band or selector is chosen by scoring candidates against oracle cells, it is a fit
      (reviewer-excel-fidelity check 7, decision 0188 R4). The Outcome then shows every R4 guard:
      - Hold-out: counts (bit-exact, within 4 ulp, within 15 digits) beside the correctly rounded candidate.
      - Fresh sweep: a log-uniform sweep over the branch's domain, oracle-filled after the freeze, with its Gate A
        envelope result and every point outside the envelope listed.
      - Tag: a `FITTED(T-4016, n_cells, range)` tag and range, with accurate math outside it.
      - Floor: nothing fitted below the reproducibility floor (4 ulp, decision 0112).
      - No keep-matched branch.
- [ ] Unit tests pin both cells at Excel's value, naming the case and cell.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The BupBgrat a0 = 1/2 cells (T-4015), FDIST at a huge df2 (T-4018), and T-3472's probe cells (Phase 1b).
