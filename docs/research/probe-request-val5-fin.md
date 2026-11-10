# PROBE-REQUEST val5 — the beta density kernel, fresh cells (r8fin, 2026-11-10)

Two unfilled spec cases in `eval/suites/probes/cases/`, same `rustsheet-case/v1` layout as val3 and val4 (every
formula cell is in `check`; no expected values; each workbook carries the `Precision` canary sheet, whose B2:B4 must
read 0.9999999999999998, 1, 1.0000000000000002 and C2:C4 -1, 0, 1). Fill each as its own workbook with the
excel-graph backend; read raw values, not displayed text. Frozen engine values:
`docs/research/val5-fin-predictions.json` (hypotheses, never oracle values).

Background (T-5107, docs/research/t5107-fin-val4.md): the beta density for both shapes above 1 was identified on the
literal `BETA.DIST(x,a,b,FALSE)` cells already in the suites, where the engine is now bit-exact on 10,609 of 10,609.
The last piece, Excel's Stirling error at a count that is not whole, was read off those same cells. These two cases
are the fresh evidence decision 0188 R4.2 asks for: drawn after the kernel was frozen, none of their cells was seen
while it was identified.

| Case | Checked formulas | What it pins |
|---|---:|---|
| `val5-fin-02` | 377 | The Stirling error at 53 counts that are not whole, one at a time: the density at x = k/n, where both of Loader's deviances are exactly 0 |
| `val5-fin-03` | 840 | A log-uniform sweep of the density over shapes from 1.03 to 300 and x from 1E-9 to 1 - 1E-9, 300 cells (seed "r4-T-5107"), with the cdf beside it as an observer; and 117 densities at shapes where a count lands on a whole number during the Stirling walk (sheet W) |

## val5-fin-02 — the Stirling error, read directly
Sheet `S`: n = 1, 2, 4, 8, 16 (column A) and a count k below n that is not whole (B); the shapes a = k + 1 and
b = n - k + 1 (C, D); x = k/n (E, checked raw; n is a power of two, so x, n·x and n·(1 - x) are exact, and G checks
n·x); F is `BETA.DIST(E,C,D,FALSE)`. At that x the two deviances vanish and Loader's lc is
stirlerr(n) - stirlerr(k) - stirlerr(n - k) with nothing else in it, so F reads the two walked-up Stirling errors to
about 1E-17. H and I are GAMMALN observers.
- **Expect:** F equals the frozen value on all 53 rows.
- **If a row misses:** its counts k and n - k name the step of the walk that is off (statistics.md "Stirling error at
  a count that is not whole: a walk up to the series"): a count above 2.67 takes the series side of ln(1 + 1/m), a
  count below it LN of the stored 1 + 1/m; rows with a quarter or an eighth count test steps no half-integer cell
  pinned.

## val5-fin-03 — the fresh sweep
Sheet `P`: a, b (A, B), x (C), the density (D) and the cdf (E). Rows 2-101: both shapes 1 + log-uniform(0.05, 299),
x = 10^u with u uniform on (-9, -0.02), 40% of the rows mirrored to 1 - x. Rows 102-201: one count (shape - 1)
log-uniform on (0.03, 6) and not whole, the other shape as above. Rows 202-301: both counts below 6, every other row
a pair of half-integers, x uniform on (0.02, 0.98). Every argument has at most 12 significant digits and is a cell value, so
the cell holds the nearest double (a formula literal would not always: the engine's literal reader, which models Excel's, reads 0.999989119353 as
0.9999891193529999; the frozen values are computed from the cells).
- **Expect:** P!D equals the frozen value on all 300 rows (17 of them are 0 in the far tail and say
  little). E carries no prediction: the cdf's kernels are not identified in every region (bfrac's continued fraction,
  bpser's ln B above 8).
- **If D misses:** a count below 6 that is not whole points at the Stirling walk-up; a deviance near the series cut
  (|k - n·x| against 0.2·(k + n·x)) at bd0; anything else at the stored lc or the finish.

Sheet `W` (added after the first internal review): a, b (A, B; a shape that is not a short decimal is a
checked raw cell, so the fill shows the double Excel holds), x (C: 0.3, 0.5, 0.7) and the density (D). 39 shapes: 33
where a count (shape - 1, or n - k) is an ulp or three from 2, 3 or 4, or is j·2^-52, so that the count rounds onto
a whole number (2, 3, 4 or 5) while the Stirling walk adds 1; and 6 controls an even number of ulps from a whole
shape, where no count lands. The rule under test: wholeness is tested once, on the count given; a count that lands
walks on to 6.0 and takes the series there. It was found on 64 cells with expression arguments that the
identification never read (probes-numeric/t2513-r3-loader-const-sweep1 A8209:A8320; internal review).
- **Expect:** D equals the frozen value on all 117 rows. On 84 of them the frozen value differs from the value of
  the rule the review refuted (the walk stopping at the table's entry of the count it landed on); the 33 others are
  the controls and rows where the two rules give the same double (the nine rows at 1 + j·2^-52 among them: not
  evidenced by any cell so far).
- **If D misses on a landing row:** the walk does something else at a landed count; the count and the whole number
  it lands on (k or n - k; 2, 3, 4 or 5) say which case.
