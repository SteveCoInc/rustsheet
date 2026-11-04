---
id: T-4018
title: "FDIST(7,10,1E+8) at a huge first shape: gen-m9 7bd219ec0c0c5e22 A349, Excel 6.7E-15 off the math, the Gate B face of deferred T-3209 (visible)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-4017]
campaign: visible
---

## Goal
gen-m9/7bd219ec0c0c5e22 Calc!A349 `=FDIST(7,10,1E+8)` is within half a unit of the 15th significant digit of Excel's
value. The fix models Excel's incomplete beta at a huge first shape above the mean, where Excel is the inaccurate
side. No F.DIST, F.DIST.RT, FDIST or BETA.DIST cell that is within 15 digits today moves off.

## Context
- Gap triage 2026-11-04 (`make eval` on origin/main 30a36f59, excel-graph). A349 is visibly off in gen-m9, a Gate B
  suite. Decision 0211 counts every visibly-off cell there whatever its math, so FDIST stays below Gate B until it
  passes. FDIST has 3 visibly-off cells of 227 rate cells; T-4015 and T-4016 own the other two.
  - `make unowned-off` misses A349 only because T-3198's triage table names it. T-3198 moved it to T-2221's "Cells
    added from T-3198 (2026-11-03)" (deferred), saying none of those cells was visibly off; the report disagrees for
    this one.
  - T-3209 (deferred) owns the band and says: "A point that Gate A's tables show outside the envelope gets its own
    task without the key". A Gate B cell is the same case. Precedents: T-3952 and T-3954.
  - This task takes A349 out of T-2221's list. T-2221 and T-3198 name this task.
- The cell (`make eval-show CASE=gen-m9/7bd219ec0c0c5e22.case.json`, NumericClose):
  - Excel 4.433824524496815E-11, ours 4.433824524496776E-11.
  - 50-digit math (mpmath 1.3 `betainc`, the right tail I_v(d2/2, d1/2) at v = d2/(d2 + d1·x)):
    4.4338245244967852577E-11.
  - Excel is +6.7E-15 relative from the math, ours -2.1E-15. The correctly rounded value, 4.433824524496785E-11, is
    visibly off Excel, so the target is Excel's own arithmetic: `visible`, not `accuracy`.
  - Excel's error is far below decision 0188 R3's 1E-4, so Excel wins.
- Region: a = 5E7 and b = 5 (df1 = 10), above the mean, with a tail of 4.4E-11.
  - Excel: bratio's L140 bup + bgrat.
  - Ours: T-2528's accurate DiDonato-Morris expansion (`large_shape_beta.rs`; a >= 1E6, b < 40, b/a <= 1E-6).
  - docs/excel-semantics/statistics.md, "Incomplete beta at a huge first shape": Excel's error grows with
    z = -nu ln x (nu = a + (b - 1)/2). This is consistent with z formed with a few ulp of error in Excel's own LN or
    log1p, but not pinned down.
  - T-2528's probe covers only b = 2.5, so T-4017 sweeps df1 from 2 to 79. Needs an oracle sweep: this task waits
    for it (decision 0056, no fit blind).
- Code: crates/rustsheet-engine/src/functions/statistical/large_shape_beta.rs, `bratio/` (bgrat), `fisher.rs`. Models:
  T-2510's `bratio_excel.py` `bgrat` and `docs/research/t2528/reference.py`.
- `large_shape_beta.rs` is an accuracy stand-in that is meant to fold into bratio's bgrat (T-3209). Fit Excel's
  arithmetic inside that one definition, not as a second series (directive 3).

Cells this task owns (split with T-2221 and T-3198, which name this task):
- gen-m9/7bd219ec0c0c5e22.case.json (1): Calc!A349

## Visible-parity stage
`campaign: visible` (decisions 0083, 0114). The stage is met when:
- A349 is within half a unit of the 15th significant digit of Excel's value;
- no cell of gen-m9, ironcalc-calc, T-2528's probe, T-1530's df-cut probe, T-1538's probes or T-4017's sweep that is within
  15 digits today goes visibly off.

T-2528's 22 probe cells stay T-3209's (Phase 1b), unless the same model lands them, in which case the Outcome lists
them.

## Acceptance criteria
- [ ] gen-m9/7bd219ec0c0c5e22 Calc!A349 leaves eval/baseline/gen-m9.txt or is within 15 digits of Excel. The Outcome
      shows the `make eval-show` output.
- [ ] The R4 guards of reviewer-excel-fidelity check 7 (decision 0188 R4) for the model of Excel's z or bgrat
      arithmetic:
      - Hold-out: counts (bit-exact, within 4 ulp, within 15 digits) beside the correctly rounded candidate, on a
        split of T-4017's and T-2528's cells stated before scoring (about 30%, by cell hash, never tuned on).
      - Fresh sweep: a log-uniform sweep over a in [5E6, 5E9], b in [1, 40) and x above the mean, oracle-filled after
        the model is frozen, with its Gate A envelope result and every point outside the envelope listed.
      - Tag: a `FITTED(T-4018, n_cells, range)` tag and range on any fitted piece, with accurate math outside it (a
        structural model of kind (c) owes no tag).
      - Floor: nothing fitted below the reproducibility floor (4 ulp web against desktop, decision 0112).
      - No keep-matched branch.
- [ ] The within-15-digit counts on T-2528's probe and T-4017's cases do not fall. No new known-failure line
      appears in any eval/baseline/<suite>.txt (`make eval-baseline` refuses regressions).
- [ ] statistics.md's "Incomplete beta at a huge first shape" states the model and its scores, citing the cases.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Below the mean, tiny x, and shapes of 40 or more (T-2510, T-3479, T-3527).
- The other FDIST Gate B cells (T-4015, T-4016).

## Evidence (T-4017)
Sweep cases: probes-numeric t4017-f-huge-df2-above-mean-* (docs/excel-semantics/statistics.md, "Incomplete beta at a huge first shape").
