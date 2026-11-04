---
id: T-4017
title: "Oracle sweep: F.DIST.RT above the mean at df2 1E7 to 1E10 for df1 other than 5 (T-3209's huge-first-shape band at b != 2.5)"
milestone: M9
lane: research
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
T-4018 needs Excel for the web values to fit Excel's incomplete beta at a huge first shape (a = df2/2 from 5E6 to
5E9) above the mean, for second shapes b = df1/2 other than 2.5. This task provides them as an excel-graph sweep with
50-digit references.

## Context
- T-2528's probe (60 probes-numeric cells) covers only df1 = 5 (b = 2.5).
  - docs/excel-semantics/statistics.md, "Incomplete beta at a huge first shape", records Excel within about 1E-15
    of the math for tails above 1E-4, and up to 1.5E-14 off further out.
  - The suspected cause is z = -nu ln x formed in Excel's own LN/log1p, in bratio's L140 bup + bgrat.
  - Deferred T-3209 owns that probe's 22 visibly-off cells.
- gen-m9/7bd219ec0c0c5e22 Calc!A349 `=FDIST(7,10,1E+8)` is the Gate B face of the same band at df1 = 10 (b = 5).
  Excel is 4.433824524496815E-11, 6.7E-15 above the math (4.4338245244967852577E-11), so the correctly rounded value
  is visibly off Excel. T-4018 owns that cell (split from T-2221's bit-level list) and has to reproduce Excel's
  arithmetic there. One df1 is not enough to tell a z-error
  model from a b-dependent one; it needs several.
- The other huge-df2 F evidence is below the mean or at a tiny x, which are other branches:
  - T-3354's two F sweeps (one at a tiny x);
  - T-1530's df-cut probe and T-1538's probes.
  (Cases are named here by their tasks, so this task does not claim their cells for `exit_owners.py`.)

## Acceptance criteria
- [ ] New probes-numeric cases `eval/suites/probes-numeric/cases/t4017-f-huge-df2-above-mean-*.case.json`
      (`rustsheet-case/v1`, sheet `Calc`, one formula per cell, a `hypotheses` label per cell naming the inputs), at
      most about 1,000 cells each, filled with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite
      probes-numeric` (excel-probe skill). The grid:
      - `F.DIST.RT(x, d1, d2)` and `F.DIST(x, d1, d2, TRUE)` for d1 in {2, 3, 4, 6, 7, 8, 10, 12, 16, 20, 30, 40, 60,
        79}, d2 in {1E7, 3E7, 1E8, 3E8, 1E9, 1E10}, and x in {1.5, 2, 3, 5, 7, 10, 20, 50}. That is 1,344 cells, all
        above the mean.
      - gen-m9 A349's own point, `FDIST(7,10,1E+8)`, as a control.
      - Every call at valid arity. An invalid one makes Excel for the web reject the whole workbook (T-0040).
- [ ] The `.oracle.json` sidecars are committed with no pending cells.
- [ ] 50-digit references at the double arguments are committed beside the cases, from
      `docs/research/t2528/reference.py` or the math-sidecar tooling (SPEC-0029).
- [ ] statistics.md's "Incomplete beta at a huge first shape" section gains the sweep's summary, citing the cases: per
      df1, the count of cells correctly rounded, within 4 ulp and within 15 digits of the math, and the largest
      error.
- [ ] T-4018 and T-3209 link the cases.
- [ ] Fixtures hold only formulas and Excel values. No RUSTSHEET_ORACLE_* values in logs or commits.
- [ ] If the oracle is unavailable, the task is `status: blocked` with the reason and the command that proves the
      oracle is back (`cd tools && uv run rustsheet-oracle status`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting or implementing (T-4018, T-3209).
- Cells below the mean (T-2510's Cephes side), and tiny x (T-3479, T-3527).
