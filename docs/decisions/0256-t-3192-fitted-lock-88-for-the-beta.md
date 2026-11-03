# 0256 — T-3192: fitted lock 88 for the BETA.DIST a = b = 1/2 reflection cut

- **Status:** accepted (2026-11-03, T-3192). It applies decision 0188 R4.3 and adds no rule.
- **Context:** T-3192 finds that BETA.DIST's cdf at a = b = 1/2 is Excel's arctangent closed form
  2*ATAN(SQRT(x)/SQRT(1-x))/PI, reflected as (0.5 - w(y)) + 0.5 above a cut (`incomplete_beta::half_half_cdf`;
  docs/excel-semantics/statistics.md, "BETA.DIST at a = b = 1/2 is an arctangent closed form, reflected above
  x = 0.9"). The cut `HALF_HALF_REFLECT_ABOVE` = 0.9 was placed by scoring the first fresh sweep
  (probes-numeric/t3192-beta-half-half: the direct form wins up to x = 0.888, the reflection from 0.900), so it is a
  fit (decision 0188 R4) and carries a `FITTED(T-3192, ...)` tag.
  - Hold-out: frozen before the second sweep (t3192-beta-half-half-2) was filled; there 287 of 341 bit-exact against
    228 for no reflection and 253 for reflection at 0.5, every cell within 4 ulp (train 281 of 321).
  - Reproducibility (decision 0112): a second excel-graph fill of both sweeps matched the first on all 1,304 cells.
- **Decision:** quality/fitted.json rises by one for incomplete_beta.rs, for this one new fit
  (`scripts/check_fitted.py --update`); the same update records gamma.rs's count falling from 7 to 6, the tag count on
  main, which the ratchet only lets fall.
- **Consequences:** the fitted ratchet stays one-way for new fits, each with its own record. T-3192's R4.2 Gate A batch
  (`r4-T-3192`) is still owed before the PR merges (task file, Landing notes); T-3886 owns the form's residual misses.
