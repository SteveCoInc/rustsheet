# 0317 — Scoring sidecar pivot cells gives the two pivot probes visible-parity lines

- **Status:** accepted (2026-11-06, T-4124, M16; decision 0274). It applies `--allow-regressions`
  (decision 0219) once, to add two lines to `eval/baseline/visible-parity.txt`.
- **Context:**
  - T-4123 made oracle sidecars capture every cell of each pivot Excel saved. T-4124 scores those cells,
    as the M16 plan requires.
  - Two existing probes hold such cells, probes/t4123-pivot-capture and probes/t4128-pivot-spike. Their
    pivot cells become scored Excel cells, and the engine does not render pivots yet (M16 PV1), so they
    fail. That is 241 cells visibly off, in two existing cases that had no lock line, because before
    T-4124 none of their scored cells was visibly off. Their checked-cell counts go from 1 and 2 to 226 and 74.
  - Decision 0219 refuses a new line, or a rise, for a case already in the baseline, without `--allow-regressions` and a decision
    record. This rise is new coverage of the same Excel evidence, not a behaviour regression: no cell
    that passed before fails now. The known-failure lists show the same: the new `F` lines are all
    pivot-area cells, and no previously checked cell changes verdict.
- **Decision:** T-4124 adds these two cases' lines (181 and 60 cells) with
  `rustsheet-eval update-baseline --allow-regressions`. No other line may rise in the same run; the PR
  shows only these two lines changing in `visible-parity.txt`. Probe PRs that add new pivot cases need
  nothing like this, because a new case always gets its line (decision 0219).
- **Consequences:**
  - The two lines fall again as PV1 makes the engine render pivots, through the steward's daily lowering
    (decision 0093).
  - Any later task that starts scoring cells in existing cases needs its own record.
