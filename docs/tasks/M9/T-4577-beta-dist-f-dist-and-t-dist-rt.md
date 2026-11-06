---
id: T-4577
title: "BETA.DIST, F.DIST and T.DIST.RT last digits in the t4456 deep-tail probes at a huge first shape or df: the 48 cells within the 13-digit bar that no task names (Phase 1b)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-4463]
defer: phase-1b
---

## Goal
The 48 cells below match Excel to the bit, or the Outcome records per case the bit-exact count and why the rest cannot
be derived. They are the t4456 tail cells (1E-300 to 1E-209, first shape or df 1E4 to 5E7) that are already within the
incomplete-beta family's 13-digit bar, or within 15 digits for T.DIST.RT.

## Context
- Gap triage 2026-11-06 (`make eval` on origin/main aabdd668c). Every failing cell of report.json was checked
  against open tasks with the `scripts/exit_owners.py` `owned` rule. Three groups had no owner:
  - 66 NPER `r3_math` cells, by design: decision 0188 R3, T-3367 and T-4285 done;
  - one t3568 BINOM.DIST cdf cell, now T-4576 (with its probe T-4575);
  - these 48 cells.
- T-4456 (done) salvaged both probes. T-4463 owns their cells that are beyond the 13-digit bar. T-4437 owns their
  exact-0 cells. The cells here are what is left. None is visibly off under decisions 0114 and 0267:
  - BETA.DIST: 32 `residue13` (first wrong at the 14th digit), 9 `digits15`, 3 four-ulp;
  - F.DIST: 1 `residue13`;
  - T.DIST.RT: 3 one-ulp.
  
  Decision 0267 item 4 says Tier C gets no new work and is worked only inside a task that owns Tier A or B cells.
  Hence `defer: phase-1b`, and the dependency on T-4463, which works the same kernel on the Tier B cells of these
  cases. Re-measure after it lands.
- Examples (Excel vs ours, `make eval-show CASE=probes/t4456-beta-t-zero-reach.case.json`):
  - t4456-beta-t-zero-reach A16 `BETA.DIST(0.99967337247139,2E6,5.5,TRUE)`: 1.587103267894936E-273 vs
    1.5871032678948377E-273 (residue13).
  - t4456-beta-t-zero-cut D7 `BETA.DIST(0.99985749639356,5e+06,30,TRUE)`: 2.1771843420983836E-258 vs
    2.177184342097894E-258 (residue13). n = 29 bup terms, on the fallback kernel past the n <= 9 port.
  - t4456-beta-t-zero-reach F4 `T.DIST.RT(9607.93233167153,100)`: 2.1717382813897045E-300 vs 2.171738281389704E-300
    (one ulp). The t's tail is `t_kernel_tails`, bgrat at b = 1/2.
- Kernels:
  - `bratio/bup.rs` `l140_bup_bgrat`, where MAX_BUP_TERMS is 9 and MAX_BUP_A0 is 1E7, so a fallback takes most of the
    BETA.DIST cells. statistics.md "L140 bup+bgrat with n >= 2 bup terms" leaves the n >= 10 residue open; it grows
    with the binade of (n - 1)·|ln y0|.
  - The t's bgrat at a0 = 1/2.
- **Needs an oracle sweep.** These cells are walks along one or two parameters, so they are too few and too correlated
  to fit and hold out. Decision 0267 item 5 freezes new probes-numeric cases except for tasks with a "Decision 0267
  cells" section, and these Tier C cells cannot have one. So no sweep task is filed now. When the steward lifts the
  defer at Phase 1 exit, file a research-lane sweep (training and a disjoint hold-out over a0 1E4 to 5E7, n 5 to 39 and
  ln tail -700 to -480, plus the t at df 1E4 to 1E8) under whatever probe rule is then in force, and make this task
  depend on it. Do not fit blind on these 48 cells.

## Cells owned
- probes/t4456-beta-t-zero-cut.case.json (14): Calc!C2, C4, C5, C6, C7, C9, D2, D3, D5, D6, D7, D8, D9, G13
- probes/t4456-beta-t-zero-reach.case.json (34): Calc!A1:A19, B6:B12, C1, D5, D6, D8, D9, F3, F4, F5

## Acceptance criteria
- [ ] Each cell above is bit-exact, or the Outcome lists it with Excel / ours / 50-digit values and the reason it
      stays. Those N known-failure lines disappear from eval/baseline/probes.txt with no new ones.
- [ ] Decision 0188 R4 guards (reviewer-excel-fidelity check 7), for any formulation chosen by scoring:
  - hold-out counts (exact / 4 ulp / 15 digits) on the sweep's hold-out, not on cells used to choose;
  - a fresh sweep, drawn after the formulation was frozen, is scored;
  - the Gate A envelope result is in the Outcome (within 15 digits of Excel, or within 1E-9 of the 50-digit value),
    listing every point outside;
  - the code carries a `FITTED` tag with the fitted range;
  - the reproducibility floor is met;
  - there is no branch that keeps already-matched cells by special case.
- [ ] No gated or probes-numeric cell goes from within 15 digits to visibly off (decision 0267 item 6 governs any
      family regression to the 14th digit).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The beyond-13-digit cells of these cases (T-4463) and their exact zeros (T-4437, T-4045).
- NPER's R3 cells (decision 0188 R3: the engine returns the math by design).
