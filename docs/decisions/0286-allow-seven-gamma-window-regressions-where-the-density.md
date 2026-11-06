# 0286 — Allow seven gamma-window regressions where the density takes Excel's mode chain

- **Status:** accepted (2026-11-05, T-4296)
- **Context:**
  - T-4296 probed Excel's large-shape gamma density at x = a, where the exponent is 0, so the cells read out the
    rest of the chain (probes/t4296-gamma-density-at-mode, excel-graph 2026-11-05; docs/research/t4296/score.py).
    Excel there is exp(-c(a)) / sqrt(2 pi a) times (a / x), with c the exact Stirling error. That chain is bit-exact
    at all 40 fit shapes (A), all 40 hold-out shapes (B, drawn from a fixed seed before scoring) and all 40 scale-3
    cells (C). The engine's sqrt(a / 2 pi) exp(-c) / x is bit-exact at 22, 22 and 18 of them, and the correctly
    rounded density at 21, 33 and 21. This is a structural model, an operation order chosen from ranked candidates
    (reviewer-excel-fidelity check 7 (c)). The engine's density now takes it inside the near-mode window
    (density.rs `mode_density`), on the engine's own exponent -a rlog(y/a).
  - Fresh sweep probes-numeric/t4296-gamma-density-r4-fresh: 160 log-uniform cells, drawn after the code was frozen,
    since GAMMA.DIST has no Gate A table yet (T-3925). The counts are bit-exact / within 4 ulp / within 15 digits.
    - At x = a (rows 1 to 40): new 40/40/40, main 20/40/40, correctly rounded 18/40/40.
    - Elsewhere in the window (rows 41 to 160; 34 of them underflow to 0 in Excel and in both engines): new
      38/51/64, main 38/51/63, correctly rounded 39/52/57.
    - Gate A's envelope (decision 0188 R2) holds every point for both main and the new chain.
  - Gated suites: probes fixes 49 cells. Seven gated cells that matched now miss by 1 or 2 ulp. All are inside the
    window and away from the mode, where Excel's exponent is still unknown: per unit of shape it is about half an ulp
    of ln(x/a) from every tested form (T-4290).
    - probes/t4255-gamma-window-exact-ratio C29 and C31.
    - probes/t4290-gamma-window-large-pow2 C90, C123, D92, D93 and D95.

    The old chain hit these bits by luck. Across all 1,441 probed window densities the window chain is bit-exact at
    371 against 224 (both on the correctly rounded exponent; score.py part 2), and no case loses.
- **Options considered:**
  - *Keep the old chain away from x = a, or near the seven cells.* Either would be a branch chosen to keep the
    previously matching cells, which decisions 0056 and 0082 forbid. It also loses where the at-mode cells and the
    fresh sweep say the chain is Excel's.
  - *Wait until the window exponent is known.* That is open research (T-4290, and the follow-up this task files). The
    prefactor result stands on its own.
- **Decision:** run `rustsheet-eval update-baseline --allow-regressions` once in the T-4296 PR, for exactly these
  seven cells.
- **Consequences:** `eval/baseline/probes.txt` gains these seven lines and loses the 49 fixed ones. Reviewers
  confirm that the baseline diff adds no other line. The cells come back once Excel's window exponent is modelled.
