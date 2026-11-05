# 0290 — Allow the one gamma-density regression at x/a = 1.5078125 where the density takes Loader's chain

- **Status:** accepted (2026-11-05, T-4308)
- **Context:**
  - T-4308 read Excel's large-shape GAMMA.DIST density exponent as an exact double on the T-4296 chain
    exp(-c(a) - B) / sqrt(2 pi a) * (a / x) (docs/research/t4308/read_out.py, parts 1 and 2). Inside the window
    |y - a| <= 0.2 (y + a) it is Loader's bd0 split B = (y - a) v - (2a atanh(v) - 2a v), with v = (y - a)/(y + a) and
    atanh correctly rounded. The fit is probes/t4290-gamma-window-large-pow2 A:C (551 of 551 cells) and the hold-out
    is column D (193 of 195). The split and an R4.1 margin were fixed and pushed before scoring. This is a structural
    model, an operation order chosen from ranked candidates (reviewer-excel-fidelity check 7 (c)), with no fitted
    constant. Outside the window the same chain on Loader's direct deviance a ln(a/y) + (y - a) is Excel's at 156 of
    160 probed densities above it and 119 of 155 below it. The ratio prefactor the engine took there is at 43 and 65
    (read_out.py part 4). The second half is T-4319's finding (T-4302), and this PR takes it in too.
  - Fresh sweep probes-numeric/t4308-gamma-density-r4-fresh: 154 cells drawn after the model was frozen (seed
    43081). Of the 54 numeric cells with a nonzero Excel value, the engine is bit-exact at 52 and the correctly
    rounded density at 3. Gate A's envelope (decision 0188 R2) holds at every numeric point. A152:A154 are outside it:
    Excel's #DIV/0! past the 2 pi a overflow against our number, unchanged by this PR (T-4340). On the older
    probes-numeric/t4296-gamma-density-r4-fresh the engine is bit-exact at 125 of 126 (read_out.py part 3).
  - Gated suites: probes fixes 1,168 cells, including the seven of decision 0286, and gen-m9 fixes 1. One gated cell
    that matched now misses: probes/t4290-gamma-window-large-pow2 B223, `GAMMA.DIST(1544,1024,1,FALSE)`, at
    x/a = 1.5078125, just above the window. On that row (A223, B223, C223) Excel is the ratio prefactor to 0 to
    2 ulp, and the direct deviance is 75 to 2,021 ulp off. At its neighbours x/a = 1.5039, 1.5098 and 1.5117 (A222,
    D223, A224, B224) the direct deviance is bit-exact and the ratio prefactor up to 1,160 ulp off. A223 and C223
    were already failing.
- **Options considered:**
  - *Keep the ratio prefactor at x/a = 1.5078125, or just above the window.* That is a branch chosen to keep a
    previously matching cell. Decisions 0056 and 0082 forbid it, and it would lose the neighbours, where the chain
    is bit-exact.
  - *Leave the region above the window to T-4319.* Below and above the window it is the same chain, and the A/B
    shows that splitting it would leave over a hundred probed cells off for one cell.
- **Decision:** run `rustsheet-eval update-baseline --allow-regressions` once in the T-4308 PR, for exactly this cell.
- **Consequences:** `eval/baseline/probes.txt` gains the one line B223 and loses the 1,168 fixed ones. Reviewers
  confirm that the baseline diff adds no other line. Row 223 is recorded in statistics.md as an open point.
  Follow-up T-4339 probes the region above 1.5a more densely.
