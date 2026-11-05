# 0292 — Lock the FITTED 1.3a cut of POISSON's upper-window cdf

- **Status:** accepted (2026-11-05, T-4322)
- **Context:**
  - T-4322 found the form Excel uses for the POISSON.DIST cdf from 1.3a to 1.5a (a = k + 1, integer shapes 31 to 301):
    sqrt(a / 2 pi) exp(-B - c(a)) times the fraction, where B is T-4308's window exponent (excel_tail.rs
    `excel_window_upper`; docs/research/t4322/score.py; docs/excel-semantics/statistics.md, the T-4322 paragraph).
    The form itself is structural, since the exponent comes from T-4308. Its lower end, `WINDOW_MIN_RATIO` = 1.3, is a
    fit: it was picked inside the bracket (1.2957a, 1.3068a) that the seen T-4302 probes give.
  - The R4 guards of decision 0188:
    - The cut was frozen in commit ff265dc32 before the fresh sweep was written and filled. The sweep is
      probes-numeric/t4322-poisson-window-r4-fresh: 48 points from seed 43220 at 1.3a to 1.5a, plus a ladder across
      the cut at k = 100, 200 and 300 (excel-graph, 2026-11-05).
    - On the fresh sweep, 51 of 57 points are within 4 ulp of Excel, against 10 of 57 for the correctly rounded Q.
      Gate A's envelope holds all 57 points, and every point of the Gate A batch accuracy/POISSON.DIST/r4-T-4322
      (120 points, drawn after the freeze).
    - The ladder puts Excel's cut in (1.298a, 1.3a]: at 1.3a the form is 2, 0 and -5 ulp off.
    - The tag is `FITTED(T-4322, 24 cells, x/a 1.26..1.5, shape 31..301)`, and the form applies only in that range.
  - `scripts/check_fitted.py --update` also lowers annuity.rs's count from 4 to 3. That is the file's actual
    count on main: an earlier PR removed a tag without updating the lock.
- **Decision:** add the new tag to `quality/fitted.json`, so excel_tail.rs goes from 3 to 4 tags, and take
  annuity.rs's tighten-only correction in the same `--update`.
- **Consequences:** the lock holds 91 tags in total. T-4345 owns the region below the cut, where Excel uses neither
  this form nor the correctly rounded Q. If T-4345 finds the cut's mechanism, for example Cephes' `SMALLRATIO`
  switch to the asymptotic series, the constant stops being a fit and its tag can go.
