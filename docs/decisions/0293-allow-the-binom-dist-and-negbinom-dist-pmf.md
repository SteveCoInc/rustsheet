# 0293 — Allow the BINOM.DIST and NEGBINOM.DIST pmf regressions where Loader's kernel replaces the exact walk

- **Status:** accepted (2026-11-05, T-2146)
- **Context:**
  - T-2513 established that Excel's interior BINOM.DIST pmf is Loader's `dbinom_raw`, the kernel BETA.DIST's density
    shares, and that NEGBINOM.DIST's pmf is p times the same kernel shifted by one (statistics.md "BINOM.DIST pmf:
    closed forms at k = 0 and k = n, and one kernel with BETA.DIST", "NEGBINOM.DIST pmf sits on the shifted binomial
    kernel"). The engine still answered both pmfs with T-0927's exact double-double walk. It used Loader's kernel
    (`loader.rs` `excel_dbinom_exponent`) only past the walk's budget.
  - T-2146 ranked the two as structural candidates (reviewer-excel-fidelity check 7 (c)). Loader's kernel goes
    everywhere in the interior: k between 1 and n - 1, every f >= 1 with r >= 2, and q^f at r = 1. The f = 0
    NEGBINOM pmf goes to EXP(fl(r·LN(p))), the gen-m9 `NEGBINOMDIST(0,1,1E-200)` = 9.99999999999978E-201 form. A
    tail-only switch (Loader below e^-T) lost to "everywhere" at every T from 2 to 60, on both train and hold-out.
    Counts are failing / visibly-off BINOM-family cells; lower is better:
    - train (the T-2513 -a and -n probes, T-2145's sweeps, T-3568, T-3960 and the other ungated BINOM probes):
      walk 11,612 / 4,303, Loader 10,575 / 3,392;
    - hold-out (t2513-r3-binom-pmf-hold, t2513-verify-r3-binom-pmf-h, the -hold T-3568 and T-3960 cases): walk
      6,892 / 2,065, Loader 6,163 / 1,442;
    - fresh sweep, probes-numeric/t2146-binom-negbinom-pmf-fresh, 400 cells drawn after the freeze: bit-exact
      walk 96, Loader 135; visibly off walk 168, Loader 131.
  - Gated suites. Fixed: gen-m9 26 cells (12 of them visibly off), ironcalc-calc 7 (1 visibly off), probes 2
    (2 visibly off). Fifteen cells that matched now miss, because the kernel's stored chain is not identified
    (T-2513: the model is bit-exact on about a third of cells):
    - within 4 ulp: gen-m9 17c110a53924beb8 A197 (1 ulp), 329b177b5d5165ad A391 (1), 3a25a03eb94fcfbd A73 (2),
      b9a32ce9a1fd0d71 A309 (1), d37313406b9f6e55 A141 (1), ede0323c389fc703 A249 (1) and A97 (2); ironcalc-calc
      BINOM.xlsx BINOM.DIST!E51, E53, E54 (1 each); probes/t3352-binom-negbinom-large-n A169 (2);
    - beyond 4 ulp, within 15 digits: gen-m9 941c409532603d32 A223 `NEGBINOM.DIST(6,6,0.05,FALSE)` (5 ulp);
      probes/t3352-binom-negbinom-large-n A15 `BINOM.DIST(16,10000,0.001,FALSE)` (7) and A157 (5);
    - visibly off: gen-m9 1db18890916ee643 A10 `NEGBINOMDIST(9,3,0.001)`, Excel 5.450697538692308E-08, Loader
      5.4506975386923137E-08 (9 ulp).
    Decision 0083's standing allowance covers only the first group.
- **Options considered:**
  - *Loader only in the tails (pmf below e^-20), where the four larger regressions disappear.* That threshold is
    chosen to keep cells that already matched. It loses to Loader everywhere on train, hold-out and the fresh
    sweep, and decisions 0056 and 0082 forbid such a branch.
  - *Keep the walk until the stored chain is identified.* That keeps about 600 more visibly-off cells on the
    hold-out alone, and the exact value is not Excel's (statistics.md "Discrete distributions").
- **Decision:** run `rustsheet-eval update-baseline --allow-regressions` once in the T-2146 PR, for exactly these
  fifteen cells.
- **Consequences:** eval/baseline/gen-m9.txt, ironcalc-calc.txt and probes.txt gain these fifteen lines and lose the
  fixed ones. Reviewers confirm that the baseline diff adds no other line from an existing case. The follow-up task
  T-2146 files (Loader's stored chain) names the fifteen cells, and they come back when that chain is identified.
