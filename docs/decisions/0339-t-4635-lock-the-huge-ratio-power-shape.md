# 0339 — T-4635: lock the huge-ratio power-shape cut in the FITTED ratchet

- **Status:** accepted (2026-11-07, T-4635)
- **Context:**
  - T-4635 found Excel's rule R for GAMMA.DIST and the chi-squared functions at a huge finite y = x / beta
    (docs/excel-semantics/statistics.md "GAMMA.DIST at a huge finite x / beta (T-4635)"; docs/research/t4635). Above
    shape 1 the result is `#NUM!` where y / a * 4 overflows. Up to some shape between 1.1 and 1.11 it is also `#NUM!`
    where y^a overflows.
  - The engine needs that shape cut as a constant, `HUGE_RATIO_POWER_SHAPE_MAX` = 1.1 in
    functions/statistical/gamma.rs. Every value in [1.1, 1.11) scores every probed cell. 1.1 was taken because it is
    the largest evidenced `#NUM!` shape (probes/t4635-gamma-dist-huge-ratio-cut Lo, shape 1.1 from y = 1E282). Shape
    1.11 is a number at y = 1E306 (probes/t4635-gamma-dist-huge-ratio-fresh Cut D1). So the constant is a fit
    (decision 0188 R4, reviewer-excel-fidelity check 7 (a)).
  - R4 guards. The outcome is categorical (`#NUM!` or 1 / 0), so the bit-exact, 4-ulp and 15-digit counts equal the
    cell counts. The y^a branch was proposed from the first fill and placed on the second fill's Lo sheet (shapes
    1.01 to 1.3). The fresh check came after the freeze: the Cut sheet of probes/t4635-gamma-dist-huge-ratio-fresh
    (shapes 1.11 to 1.14, y 1E285 to 1E306), whose 16 cells are all numbers as the cut predicts. The correctly
    rounded candidate (no `#NUM!` at a finite y) misses every `#NUM!` cell of the regime. A 30% hold-out split is not
    meaningful for a single cut on a categorical outcome, so the fresh check stands in for it. The structural 4 / a
    quotient is a model (check 7 (c)), not a tag; its fresh sweep scores 84 of 84 cut-straddling cells (T-4635 Outcome).
  - The tag is `FITTED(T-4635, 2 cells, shape 1.1..1.11)`.
- **Decision:** add the new tag to `quality/fitted.json` (`scripts/check_fitted.py --update`), so
  functions/statistical/gamma.rs goes from 6 to 7 tags.
- **Consequences:**
  - The ratchet now counts this fit. Removing it, or moving the cut outside [1.1, 1.11), needs new evidence.
  - Gate A gives no value to shapes strictly between 1.1 and 1.11 where y^a overflows (gamma_dist.py
    `_huge_ratio_domain`). T-4877 owns the remaining unprobed edge (below shape 1.01) and the Rust unit test.
