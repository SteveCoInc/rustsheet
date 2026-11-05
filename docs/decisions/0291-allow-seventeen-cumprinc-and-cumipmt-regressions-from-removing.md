# 0291 — Allow seventeen CUMPRINC and CUMIPMT regressions from removing the long-range geometric cut

- **Status:** accepted (2026-11-05, T-3524)
- **Context:**
  - T-3524 re-validated CUMPRINC and CUMIPMT's two geometric-principal cuts (functions/financial/cumulative.rs) under
    decision 0188 R4: two engine builds per cut, scored by the fit lab (`cumulative` on the 1,238 old cells,
    `cumulative-fresh` on the fresh sweep probes-numeric/t3524-cumprinc-lead-fresh-sweep, 1,000 cells, excel-graph
    2026-11-05; hold-out split T-3360, margin 5 points).
  - `LONG_GEOMETRIC_MIN_LEAD` (3.6, over more than 32 periods) fails R4: the geometric sum at every leading exponent
    beats it on the old held-out cells (149 vs 146 bit-exact) and on the fresh sweep (566 vs 508 of 1,000; held out
    184 vs 157; on the 500 long-range cells 242 vs 184, and 3 visibly off against 20), and on a second fresh sweep
    at rates 1E-12..1E-4 (probes-numeric/t3524-cumprinc-tiny-rate-sweep: 238 vs 71 of 500 long-range cells). The
    task's rule then replaces it by the other form.
  - On the gated suites that costs 17 probes cells that matched and fixes 12. Every regressed cell is numeric and 1
    or 2 ulp from Excel, none visibly off: probes/fin-cumipmt-period-bound A3, A4, A6, A11, A27, A30 and
    probes/fin-cumprinc-form-boundaries I2, I4, I7, J4, J5, J7, K1, K2, K3, L2, L4, the long-range cells near nper
    that T-3067 placed the cut on. Fixed: fin-cumipmt-period-bound A7, A8; fin-cumprinc-form-boundaries I1, J3, K5,
    K6, K7, L3, L5, L7, L9, L10. Decision 0083's standing allowance does not cover it: this is not a campaign PR
    and no visibly-off cell is fixed in the gated suites.
- **Options considered:**
  - *Keep the cut.* R4 forbids a selector that does not beat the form it replaces on test and on the fresh sweep;
    keeping it to hold these 17 cells is a keep-matched branch (decisions 0056, 0082).
  - *Refit the cut lower.* A new fit on the same cells, which R4.2 would need a further fresh sweep to accept; the
    fresh sweep shows the geometric sum ahead in every lead bucket below 3.6 but 0.3 to 0.398 (9 vs 10 cells), so
    there is no cut to fit.
- **Decision:** run `rustsheet-eval update-baseline --allow-regressions` once in the T-3524 PR, for exactly these 17
  cells.
- **Consequences:** `eval/baseline/probes.txt` gains the 17 lines above and loses the 12 fixed ones; reviewers confirm
  that the baseline diff adds no other line. The short-range cut `GEOMETRIC_MIN_LEAD` is unchanged (T-3524 kept it:
  removing it puts 38 old cells visibly off) and its form choice is T-4349.
