# 0288 — Allow two POISSON cdf regressions where the far-above-the-mean cdf takes Loader's deviance

- **Status:** accepted (2026-11-05, T-4302)
- **Context:**
  - T-4302 put the pmf, the gamma density and CHISQ.DIST.RT next to each far-above-the-mean POISSON.DIST cdf
    (probes/t4302-poisson-cdf-beside-pmf, excel-graph 2026-11-05; docs/research/t4302/score.py). Above 1.5a, Excel's
    cdf is its own gamma density times x times the fraction, within 4 ulp at 112 of 120 points. That density is
    exp(-c - B) / sqrt(2 pi a) times (a / x), with Loader's direct deviance B = a ln(a/x) + (x - a), bit for bit at
    all 120. This is a structural model (reviewer-excel-fidelity check 7 (c)). The engine's POISSON cdf now takes
    sqrt(a / 2 pi) exp(-B - c) times the accurate fraction at integer shapes 31 to 301 and 1.5a < x <= 3a
    (excel_tail.rs `excel_far_above_mean`).
  - The counts are bit-exact / within 4 ulp / within 15 digits:
    - train: new 6/32/30, main 3/5/8;
    - hold-out: new 13/36/36, main 2/5/9;
    - fresh sweep probes-numeric/t4302-poisson-cdf-r4-fresh, drawn after the freeze: new 8/45/44, main 2/15/15.
    Gate A's envelope holds all 48 fresh points.
  - Gated suites: probes fixes 8 cells. Two gated cells that matched now miss by 1 ulp:
    probes/t4291-poisson-far-above-mean A3 `POISSON.DIST(30,52.5,TRUE)` and A5 `POISSON.DIST(30,75,TRUE)`. At these
    two cells the accurate kernel was within 3 ulp of the true Q, and Excel was too. The rest of Excel's value there is
    its own continued fraction, which is up to 4 ulp from the accurate one on the train cells and is not modelled.
- **Options considered:**
  - *Start the form at shape 41, or keep the accurate kernel near these two cells.* Either would be a branch chosen
    to keep the cells that already matched, which decisions 0056 and 0082 forbid. At k = 30 the form is within 1
    ulp of Excel at all four above-window cells (A3 to A6), where main misses A4 and A6 by 1 ulp too.
  - *Model Excel's fraction first.* That is an open question below the reproducibility floor of 4 ulp between web
    and desktop (decision 0188 R4.4). The prefactor result stands on its own.
- **Decision:** run `rustsheet-eval update-baseline --allow-regressions` once in the T-4302 PR, for exactly these
  two cells.
- **Consequences:** `eval/baseline/probes.txt` gains these two lines, loses the 8 fixed ones, and records the new
  cases. Reviewers confirm that the baseline diff adds no other line from an existing case. The two cells come back
  once Excel's continued fraction is modelled.
