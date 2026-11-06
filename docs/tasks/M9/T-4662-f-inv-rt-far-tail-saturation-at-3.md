---
id: T-4662
title: "F.INV.RT far-tail saturation at 3.696066238512854E+150/df1: find the search's mechanism (T-2808 split)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-4675]
defer: phase-1b
---

## Goal
F.INV.RT gives Excel's saturated value in the far tail at a df2 below 2, from a model of the search that
produces it, never from the observed constant.

## Context
Split out of T-2808 (2026-11-06), which found no structural source for the constant. The rule and the
evidence are in docs/excel-semantics/statistics.md "F.INV.RT saturates at 3.696066238512854E+150/df1 in the
far tail (T-2435)", which also records what T-2808 ruled out. Owner rule: "do not overfit any function
values". The cap may be implemented only from a derivation that reproduces 3.696066238512854E+150 exactly
(and 1.8480349672907807E+150 at df1 = 2, which is not exactly half: the ratio to C/2 is
1.0000010000006678), not by copying it (reviewer-excel-fidelity check 7, decisions 0056 and 0188 R4).
What T-2808 found: an AS 109 replay (start clamped to [1E-4, 0.9999], `acu` = 1E-300) on the beta variable
of the upper tail stops where its Newton correction u/b falls below sqrt(acu) = 1E-150, which gives the
right order (2E150 to 5E150 over df1) and the 1/df1 scaling, but its stop depends on df2 through the
geometric path from the start (0.64 to 1.36 times the constant), while Excel's value is the same to the
last digit for df2 1.5, 1.9, 1.99 and 1.999 and p from 1E-200 to 1E-300. A probe varying df1 (1, 2, 3, 4,
10, 0.5) and df2 (0.5, 1.2, 1.9) at p = 1E-200 would show what the constant depends on.
Code: crates/rustsheet-engine/src/functions/statistical/fisher.rs `f_quantile`.

Cells (known failures):
- probes/t2435-dist-extremes S!A77, A79, A115:A118 and their column-B companions B77, B79, B115:B118
- gen-m9/abc2939ebd1e2365 Calc!A11 `=CHIDIST(F.INV.RT(1E-200,1,1.999),10)`: Excel 0, ours #NUM!
- gen-m9/a612e89321a6df9c Calc!A23 `=GAMMALN.PRECISE(F.INV.RT(1E-20,3,9))`: Excel 1087085.7102783846, ours ...853
- gen-m9/01523d247594952e Calc!A377 `=SYD(15023,(AVEDEV(Data!A11:A11,{1;2;3;4;5})*F.INV.RT(1E-20,9,3)),33,1)`
- probes/t2435-dist-extremes A78, A80, A81 (df2 = 2, df1 = df2 = 1, F.INV at 1 - 1E-16) are numerically close
  but not bit-exact.

Needs an oracle sweep (gap triage 2026-11-06): the cached cells cannot settle the mechanism, so this task is not
attempted blind. It waits for T-4675, which fills `probes-numeric/t4675-beta-search-saturation`
with predictions committed before the fill. Read that case and its statistics.md table first.

## Acceptance criteria
- [ ] A derivation of the saturation from the search's structure, reproducing both constants exactly, recorded
      in statistics.md; or the cells stay known failures with the negative result recorded.
- [ ] The cells above pass, or are named with evidence in the statistics.md entry.
- [ ] R4 guards (reviewer-excel-fidelity check 7, decision 0188), for any rule chosen by scoring candidates against
      oracle cells: train and hold-out counts side by side (bit-exact, within 4 ulp, within 15 digits) for the chosen
      rule and the correctly rounded one, on a split of about 30% stated before scoring; a fresh sweep oracle-filled
      after the rule is frozen, with its result recorded here; the Gate A envelope count on that sweep, listing every
      point outside it; a `FITTED` tag and its fitted range on any fitted constant (none for a structural model);
      the reproducibility floor met; no branch that keeps matched cells passing without the rule (no keep-matched
      branch).
- [ ] `make check` green; reviewers approve.

## Out of scope
- BETA.INV's 0.5 at a tiny shape (T-4663).
- F.INV.RT far from the math at a large df2 (T-4103 entry, T-4568).
