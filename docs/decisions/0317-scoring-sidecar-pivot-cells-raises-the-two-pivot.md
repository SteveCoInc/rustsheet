# 0317 — Scoring sidecar pivot cells raises the two pivot probes' visible-parity lines

- **Status:** accepted (2026-11-06, T-4124, M16; decision 0274). It applies `--allow-regressions`
  (decision 0219) once, to two lines of `eval/baseline/visible-parity.txt`.
- **Context:**
  - T-4123 made oracle sidecars capture every cell of each pivot Excel saved. T-4124 scores those cells,
    as the M16 plan requires.
  - Two existing probes hold such cells, probes/t4123-pivot-capture and probes/t4128-pivot-spike. Their
    pivot cells become scored Excel cells, and the engine does not render pivots yet (M16 PV1), so they
    fail. That is 241 cells visibly off, in two cases whose lock lines were written before the cells were
    scored.
  - Decision 0219 refuses a rise in an existing case's line without `--allow-regressions` and a decision
    record. This rise is new coverage of the same Excel evidence, not a behaviour regression: no cell
    that passed before fails now. The known-failure lists show the same: the new `F` lines are all
    pivot-area cells, and no previously checked cell changes verdict.
- **Decision:** T-4124 records these two cases' lines at their new counts with
  `rustsheet-eval update-baseline --allow-regressions`. No other line may rise in the same run; the PR
  shows only these two lines changing in `visible-parity.txt`. Probe PRs that add new pivot cases need
  nothing like this, because a new case always gets its line (decision 0219).
- **Consequences:**
  - The two lines fall again as PV1 makes the engine render pivots, through the steward's daily lowering
    (decision 0093).
  - Any later task that starts scoring cells in existing cases needs its own record.
