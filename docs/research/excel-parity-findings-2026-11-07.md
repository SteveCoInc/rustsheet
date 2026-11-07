# Excel-parity findings: root causes and implementation notes (2026-11-07)

Status: research hand-off. **Docs only, no code.** Each section is a self-contained description of a root cause and
of the change that fixed it in an independent prototype. Agents should re-implement it through the normal work loop
(task, tests first, `make check-fast`, review). The prototypes were measured against main `2e192ce48`. Upstream has
moved since then, so re-measure before claiming anything.

How the prototypes were checked: a full `rustsheet-eval run`; `rustsheet-eval check` (the only REGRESSION lines were the
three that also fail on unmodified main: probes/eng-impower-angle-cut, eng-impower-past-cut, lambda-call-depth);
Gate A (`rustsheet-eval accuracy`); engine and xlsx crate tests with the full output checked; workspace clippy
`-D warnings`; and `cargo fmt --check`. Gate B counts below are for main + the prototypes up to and including that
item, in the order listed.

| # | Task | Functions newly at Gate B rate | Gate B (on 2e192ce48) | Gate A |
|---|---|---|---|---|
| 1 | T-4722 | COUNT, COUNTA, COUNTBLANK, ISBLANK, ISTEXT, ISFORMULA, LEN | 462 → 469 | clean |
| 2 | T-4328 | KURT, SKEW | +2 | clean |
| 3 | T-2146 | BINOM.DIST.RANGE | +1 | clean |
| 4 | T-3469 | POISSON | +1 | clean |
| 5 | T-3219 | GAMMA.DIST, GAMMADIST, CHISQ.DIST, CHISQ.DIST.RT | +4 | clean |
| 6 | T-3146 | CHIINV, CHISQ.INV, CHISQ.INV.RT | +3 | clean |
| 7 | T-3730 | T.TEST, TTEST | +2 | clean |
| 8 | T-3363 | T.INV | +1 | clean |
| 9 | T-2930 | T.DIST.2T (see the off-by-one note) | +1 | clean after the note |
| 10 | T-4321 | LOGNORM.DIST | +1 | clean |
| 11 | T-2928 | (IRR improves 297 → 298/302; no function newly passes) | 0 | clean |
| 12 | T-2690 | LINEST, LOGEST | +2 | clean (resolves known failures) |
| 13 | T-3731 | CUMIPMT | +1 | clean (resolves known failures) |
| H | held | BETA.INV, BETAINV, T.INV.2T, TINV, CONFIDENCE.T, FINV, F.INV.RT | (+7) | **fails**: see "Held" |

Items 1–13 together took Gate B from 462 to about 488 on the old base, with Gate A unchanged.

---

## 1. T-4722: covered cells of a merged range keep their content

**Symptom.** fixtures/merged-covered-values A4:N4 and fixtures/merged-covered-formula B1, A3:D3 were visibly off, and
COUNT, COUNTA, COUNTBLANK, ISBLANK, ISTEXT, ISFORMULA and LEN each sat just under the rate (ISBLANK 4/3590). Round trips
also lost covered values (poi/bug66675.xlsx 3,604 cells, poi/60512.xlsm 12; docs/research/xlsx-round-trip-2026-11-04.md §2.2).

**Excel.** A value or formula stored in a covered (non-anchor) cell of a `<mergeCell>` is kept. Formulas read it, a covered
formula recalculates, ISFORMULA is TRUE, and save writes it back (xlsx-storage.md "A merged range keeps the values of its
covered cells"; excel-graph evidence in the two fixtures).

**Root cause.** `load_sheet` (rustsheet-xlsx `import/worksheets.rs`) calls `merged_cells::clear_covered_cells`, which
enforces the engine invariant "covered cells hold no content".

**Change.**
- **Import:** stop clearing covered cells. Delete `clear_covered_cells`, and move its unit-test shapes into xlsx
  integration tests that assert the content is kept.
- **Evaluation and export:** no change needed.
- **Paste:** `merge_cells_keep_styles(range, kept: Option<&Area>)` and `covered_cells_clear_diffs(range, kept)` take the
  block a paste just wrote. Covered cells inside it keep their content and get no clear diff; only cells that a recreated
  merge swallows *outside* the pasted block are cleared. Factor the duplicated copy-paste and cut-paste merge code into
  one `recreate_pasted_merge`. Before this change, cut-paste silently dropped covered content.
- **Autofill:** `fill_covered_cell` copies a non-empty covered source via `Model::fill_cell` (formulas shifted, with a
  `SetCellValue` undo diff). It is never treated as a progression term, and an empty source still clears.
- **Unchanged:** merge itself still clears what it covers, edits of covered cells are still refused, and unmerge needs no
  change (covered content simply shows again).
- **Spec:** write docs/specs/SPEC-0036 with a row per operation. Unmerge, clear, paste, autofill and Ctrl+arrow are
  UNVERIFIED, which makes them probe candidates.

**Tests.**
- `many_whole_column_merges_import_quickly_and_clear_covered_cells` becomes `..._keep_covered_cells`, with the same 10 s
  bound, now asserting the values "2"/"3".
- `test_import_sanitizes_merged_cells`: covered C2 keeps 43.
- New `covered_cells_keep_their_values_and_styles_on_import`, `covered_cells_calculate_and_survive_a_round_trip`, and
  engine tests for read-only reads, unmerge, clear, copy-paste, cut-paste and autofill, each with undo/redo.

## 2. T-4328: SKEW, SKEW.P and KURT use stored x87 operations

**Symptom.** `KURT({0.1,42,123456789.123,0.1})` = 3.9999999999979536 (gen-m9/27601819397dd56e A200);
`KURT({0.2,7,3.14159,-2.5})` = −0.7628272385808703 (gen-m9/7bd219ec0c0c5e22 **A279**); a 9-value KURT
(c2974c061384fdd9 A20) = 0.734384233823282; `SKEW({1E-15,1,2})` = 2.4980018054066022E-15 (d208bbafc3bc1997 A262).

**Root cause.** Three copy-pasted binary64 loops in `count_and_average.rs`. Excel's moments behave like stored x87
operations, each rounded to a 64-bit significand and then to binary64:
- *FSQRT double rounding:* SKEW's variance 0.999999999999999 has a twice-rounded sqrt of 0.9999999999999996, not
  0.9999999999999994.
- *Power order:* the fourth power is `(z·z)·(z·z)`, not `((z·z)·z)·z`. At z = 1.4999999999999234, z² double-rounds to
  2.24999999999977, one ulp below the once-rounded square, and the bias subtraction magnifies the difference.

**Change.** One `standardized_moment(values, divisor, fourth)` built from the existing `extended_sum` / `extended_product`
/ `extended_quotient` / `extended_sqrt` kernels:
- a two-pass mean and m2;
- `z = (x−mean)/sqrt(m2/divisor)`;
- cubes `(z·z)·z`, fourth powers `(z·z)·(z·z)`, summed in input order;
- divisor n−1 for SKEW/KURT and n for SKEW.P.

Keep the T-2432 rule that overflowing squares give `#DIV/0!`. The duplicate loops go away.

**Evidence.** A replay of 465 cached calls, split by case/cell hash into train and holdout, rejected compensated
summation, reciprocal normalization, the one-pass variance, raw moments and unstored extended accumulators. On Gate A
all 120 KURT accuracy points become exact.

**Note.** Cite A279 for the KURT case above. A386 is a HYPGEOMDIST cell.

## 3. T-2146: BINOM.DIST.RANGE (Loader's deviance store points)

**Symptom.** `BINOM.DIST.RANGE(7,1E-20,6)` (gen-m9/9cca17438a7e6f8a A360): Excel 6.999999999999857E-120, ours 154 ulp high.

**Root cause.** In `loader.rs::excel_dbinom_exponent`, when bd0 takes its logarithmic branch, Excel stores the two
sequential deviance subtractions to binary64 before adding the extended log-normalizer and calling EXP. We kept every low
bit extended. The all-series and scaled-probability callers (beta density, negative binomial) really do keep the extended
chain, and their existing exact controls show that a blanket store would be wrong.

**Change.**
- A `LoaderExponent::{Probability, ScaledProbability}` selector passed from the binomial PMF.
- `Bd0Branch::store()` reuses bd0's existing x87 branch decision. This matters because `BINOM.DIST(2,10,.3,FALSE)` sits on
  the series side of the x87 cutoff but not of the binary64 one, and must stay 0.23347444050000005.
- No new PMF and no new constants.

**Evidence.** A fixed train/holdout replay of 2,251 interior PMFs: exact matches go from 228/1117 to 469/1117 (train) and
from 242/1134 to 471/1134 (holdout). All 130 one-point range controls reach visible parity.

## 4. T-3469: POISSON far upper tail uses the reciprocal log

**Symptom.** gen-m9/43c6227e0ff368c7 A108 = Q(6,42) and gen-m9/5919660b7bb4d195 A80 = Q(10,100) were 25 and 72 ulp off.
Their CHISQ.DIST.RT twin (probes-numeric/t1049-chisqdistrt A28) and the "unexplained shape-6 spikes"
(t3269-shape6-misses) are the same issue.

**Root cause.** `incomplete_gamma/excel_tail.rs::excel_far_upper` built the Stirling exponent as `a·LN(x/a)+(a−x)`
(`excel_ratio_exponent`). Excel uses Loader's direct deviance `B = a·LN(a/x)+(x−a)`, with prefactor
`sqrt(a/2π)·EXP(−B−stirlerr(a))`. The two are equal in exact maths, but differ once the quotient is stored before LN, and
|ln Q| ≈ 30–90 amplifies the difference.

**Change.** `stirling_prefactor(a, -direct_deviance(a, x), stirlerr_exact(a).0)`. `excel_ratio_exponent` stays in use on
the density path.

## 5. T-3219: gamma small-x prefactor from separately rounded factors

**Symptom.** P(8,0.2) (ironcalc BETA_GAMMA.xlsx BETA_DIST!L3), P(9,5E-6) (GAMMA.xlsx E16), P(30,0.3) (gen-m9/68d8ffb69ff739d8
A341) and the density `CHISQ.DIST(1E-10,8,FALSE)` (984cba67bbe6972b A49) were 5–36 ulp off. Excel is the *less* accurate one
here, reproducibly.

**Root cause.** For shape a > 1 and y = x/β ≤ 1, Excel's prefactor is three separately rounded factors: `power =
EXP(a·LN(y))`, `norm = EXP(GAMMALN(a+1))` and `decay = EXP(−y)`.
- *CDF:* `P = (power·decay/norm)·series`.
- *Density:* `(power/norm)·a/x·EXP(−y)`.
- *Order is observable at 1 ulp:* `CHISQ.DIST(1.5,6,TRUE)` (35e8168aae2b00f8 A119) needs decay before normalization, and
  probes/stat-argument-modes H152 and m9-gamma-dist-a2 A21 rule out series before normalization.

**Change.**
- `LogSeriesFactors { power, gamma }` with `leading()` and `lower(decay, series)`, plus `small_x_factors(a, x)`. These are
  used by `pair_with` (Excel exponent policy) and `density_with`. The stable prefactor remains the fallback when a factor
  under- or overflows.
- The inverse's internal CDF (`excel_form_lower`) keeps normalize-before-decay: that order gives Excel's first Newton
  iterate at `CHISQ.INV.RT(1-1E-16,3)` (ecc87f80077959e0 A301).
- `excel_tiny_quantile`'s residual guard switches to `Exponent::Accurate` explicitly, so it keeps the meaning it was
  fitted with.

**Evidence.** Literal-call sidecars give 77/77 small-shape small-x CDFs within 2 ulp, and 170/170 densities within 4 ulp.

## 6. T-3146: CHIINV, CHISQ.INV and CHISQ.INV.RT (split complement plus Halley)

**Symptom.** `CHIINV(.975,5)` = .8312116134866638 (285d87cea1aaa73c A261); `CHISQ.INV.RT(.9999,3)` = .005214832329030269
(4250f880ab640c49 A371), which is 3084 ulp from the true quantile; `CHISQ.INV(1E-20,10)` = .0005210568413974334
(a7bf0d8d81d33ba6 A152).

**Root cause.**
1. In the small-x series region the upper tail is `(0.5−P)+0.5` with each operation stored, not `1−P`. That is exact on
   51/51 cells of t3148-gamma-quantile-flat-band, against 45/51 for `1−P`. It creates Q plateaus up to 0.75 ulp wide.
2. The inverse (shape 1–20, unit x ≤ 0.5) starts from `EXP((LN(p_small)+GAMMALN(a)+LN(a))/a)` and takes *Halley* steps,
   `x −= t/(1 − t·((a−1)/x − 1)/2)`, with t = (p−Q)/f for a right-tail search or (P−p)/f for a lower-tail one. It keeps the
   first smallest residual when the rounded CDF cycles.

**Change.** In `incomplete_gamma/closed_form.rs`, add the Halley update and split complement inside the supported region,
reusing the existing bounded iteration. Outside the region, keep the accurate crossing.

**Open.** The dispatch `p <= 0.9999` is inferred, not measured. Excel switches from inverting Q to inverting P somewhere in
(.9999, .9999462]; see probe G01. Label it as bracketed until probed.

## 7. T-3730: T.TEST and TTEST use the ported beta kernel at the rounded y

**Symptom.** Eight gen-m9 sample tests at df 13–16 and fractional Welch df were 5–19 ulp off: 09b47629f54b9817 A144,
928d08e3e1eecb5f A393, 9bc247733a1ca9fa A139/A254, afc00c8b70e78baf A238, c7664e494e418395 A219, ce312817dc9c7f0b A228,
f82473275380882e A308.

**Root cause.** In `student_t.rs::tail_from`, the generic fallback and the integer-series path used pre-port normalization.
Near the centre the series scaled the *tail* by its beta normalization, but that normalization belongs to the central
probability before it is subtracted from one. The sample variance and the Welch statistic are fine.

**Change.** Where the normalization is not exact (`!inside_one` or no exact `log_gamma_ratio`), keep the evidenced
elementary series. Otherwise call the ported TOMS 708 kernel at `(n/2, y, 1−y)`, the same one T.DIST uses. With this, all
eight cells are within 0–2 ulp.

## 8. T-3363: T.INV small-beta inverse tail

**Symptom.** `TINV(1E-100,5)` = 1.801609952626993E20 (the old log-tail approximation was 39 ulp high);
`TINV(1E-200,8)` = 2.405205857517846E25; `CONFIDENCE.T(1E-200,0.01,15)` (8f2ac25945af87da A289).

**Root cause.** For the small beta argument behind t inverse tails (df ≥ 2), Excel's result is a finite-precision iterate,
not the correctly rounded root.

**Change.** A shared `inverse_beta::beta_small_quantile(a, b, p)`:
- *Seed:* `ln_x = (LN(p) + (LN(a) + lnB))/a`. The grouping is observable.
- *Domain:* applies only where the omitted third series term `|a(1−b)(2−b)(3−b)/(6(a+3))·x³| ≤ EPSILON`; this domain
  rule comes from the series itself, not from a list of p/df pairs.
- *First correction:* `x −= (1−b)/(a+1)·x²`.
- *Newton:* steps against the beta CDF, keeping the best absolute residual.
- *Stopping:* stop when the log-probability is stationary (`ln_p + residual == ln_p`), on a sign flip of the relative
  residual, when the squared correction underflows, or when the residual is below MIN_POSITIVE.
- *Underflow:* when the squared correction underflows at the seed, return the logarithmic seed so the t layer can
  back-transform in log space (df 2 at 1E-200).
- *Cap:* a hard iteration limit.

**Tests.** (df, p) → Excel: (2, 2e-20), (5, 1e-100), (8, 1e-200), (9, 1e-100), (10, 1e-100), (4, 1e-300), (14, 1e-200).

## 9. T-2930: T.DIST.2T rounded df-1 argument and large-df centre

**Symptom.** gen-m9/928d08e3e1eecb5f A231: T.DIST.2T(1.005−1, 1) = 0.9968169276635932. The exact Cauchy tail gives …5881.

**Root cause and change.**
- *df 1:* at df 1 and t ≤ 1, evaluate the existing tail kernel at the *rounded* `y = df/(df+t²)` instead of the exact
  Cauchy tail. The controls at t = 0.3 and t = 1 stay exact.
- *Large df:* for t < 1 *beyond* the finite integer-series bound, use the stable Student-t central fraction. This avoids
  log cancellation in the beta prefactor: `T.DIST.2T(0.5,1E8)` goes from 6 ulp to 2 ulp.

**Important off-by-one.** The large-df guard must be `n > SERIES_MAX_DF`, not `>=`. The series domain is inclusive
through 1000, and `>=` regresses `T.DIST.RT(.5,1000)` (Excel 0.30859254041693746, probes-numeric/m9-t-dist-rt A123) and
t0969-t-tail-grid B1553. Add df 999/1000/1001 controls.

## 10. T-4321: LOGNORM.DIST density

**Symptom.** One Gate B cell, and the density sweep (e.g. gen-m9/3a25a03eb94fcfbd A220 = 7.694598626706419E-22 at sigma .1).

**Root cause.** Excel accumulates the negative log-density as a compensated (TwoSum) chain in a fixed order, keeps the
positive low-order residual after negating the head, and squares the displacement and sigma *before* standardizing.

**Change.** `lognormal_density(x, mean, sigma)`:
- `d = LN(x) − mean`, extended;
- `half = (d·d)/(sigma·sigma)·0.5`, falling back to `(d/sigma)²·0.5` when either square is not normal;
- `(h,e1) = two_sum(half, LN(sigma))`, `(h,e2) = two_sum(h, LN(√2π))`, `(h,e3) = two_sum(h, LN(x))`;
- result `EXP(−h)·(1+(e1+e2+e3))`;
- zero density when `half` overflows.

**Evidence.** All 120 density-sweep cells and 200 accuracy-table points are exact.

## 11. T-2928: IRR discount domain validated at convergence (partial)

**Change.**
- In the shared secant solver (`financial_util.rs`), allow intermediate negative discounts, but return `None` when the
  final discount v ≤ 0. Remove the per-iterate `d2 > 1` rejection.
- Detect stalls against the discounted *absolute* cash-flow scale, `irr_npv(|flows|, |1−d2|)`, instead of the largest
  distant flow.

**Effect.** IRR goes from 297 to 298/302, with 42 visible gains and no losses. IRR is still below the rate, see the audit
section.

## 12. T-2690: LINEST and LOGEST via centered pivoted Householder QR

**Symptom.** LINEST 243/245 and LOGEST 224/225 (two-point slope cancellation; one-point rank-deficient fits).

**Root cause.** `linest.rs` solved normal equations by Gauss-Jordan with an absolute pivot cutoff of 1e-12, then filtered
rank afterwards. Excel's results match a QR solve.

**Change.**
- Replace `solve_system` / `invert_matrix` / the `independent_columns` filtering with a *centered, column-pivoted
  Householder QR*.
- Sum each reflector's unchanged tail separately from its new head. Reconstructing the tail by subtracting head² loses
  Excel's exact identity-fit cancellation.
- Compute the statistics from the transformed residual and the triangular factor.
- GROWTH keeps its product of exponential factors.

**Effect.** 131 visible gains and no losses; Gate A known failures resolved.

## 13. T-3731: CUMIPMT and CUMPRINC from the grouped balance drop

**Symptom.** CUMIPMT 130/132 (a tiny-rate cell is 1 ulp of a large sum off; CUMPRINC_CUMIPMT.xlsx G29).

**Root cause.** `cumulative.rs` summed per period and selected between formulations with fitted constants
(`GEOMETRIC_MIN_LEAD`, `SHORT_GEOMETRIC_MAX_PERIODS`, `SHORT_GEOMETRIC_MIN_RATE`, `SHORT_GEOMETRIC_MAX_LEAD`). Excel uses a
constant-time discounted-balance identity.

**Change.**
- *Principal:* `principal = −balance·(drop·tail)`, where `(balance, log) = discounted_balance(rate, nper, pv, 0)`,
  `tail = EXP(−(nper−end+1)·log)` and `drop = 1−EXP(−count·log)`. Multiply by (1+rate) for end-of-period payments, and add
  the first PMT when type = 1 and start = 1.
- *Interest:* payments minus principal through the existing Excel form.
- *Remove* the fitted selectors and `discount_complement`, in favour of a shared `excel_one_minus_exp`.
- *Rounding ties:* keep the x87-to-double ties in the balance product, the tail exponent, the expm1 quotient, the
  growth-log argument and the beginning-payment reciprocal.

**Effect.** 471 visible gains and no losses. Gate A resolves 53 known failures together with item 12, so remove those
stale exemption lines as the gate instructs.

---

## Held: work in progress that currently breaks Gate A

These raised Gate B (BETA.INV, BETAINV, T.INV.2T, TINV, CONFIDENCE.T, FINV and F.INV.RT reached the rate) but introduce 23
unlisted Gate A failures. Do not ship them as-is. The useful findings:

- **Mechanism with real evidence.** Excel's BETA.INV returns *bounded AS 109 Newton stopping points*, not the refined
  crossing. Examples:
  - BETADIST_BETAINV.xlsx H5 = 33587.000000000226, where the crossing gives …6.99999999992; also H16 and H26.
  - gen-m9/4e55500bb8ad3cd8 A360 `BETAINV(1E-200,3.14159,TIME(4,2,1))` is the initial power-law estimate.
  - The profile that fits: an accurate normal start, endpoint-only replacement by 1E-6 / 1−1E-6, the residual divided by
    Excel's beta density, absolute accuracy `10^(−5/min(a,b)² − p^(−0.2) − 13)`, return of the pre-step iterate, the first
    best residual on plateaus, and finalization in the original coordinate.
  - The F inverse uses those iterates plus a compensated four-log back-transform `EXP(LN(x)−LN(1−x)+LN(df2)−LN(df1))`.
- **Why it fails Gate A.**
  1. *Stagnation is accepted as success.* Stalled or unbracketed searches return an arbitrary finite value where Excel
     returns `#NUM!` (`BETA.INV(.1,5,1E-14)` → 0.9999999999999999; eval/accuracy/tables/BETA.INV/s1 A89, A94, A184).
  2. *Flushing a subnormal Newton residual creates false convergence* near DBL_MIN (A111).
  3. *The F profile bypasses the R3 math-over-Excel points* in eval/accuracy/math-over-excel.tsv (F.INV/s1 A112, A126,
     A146, A160, A171).
  4. *The early-iterate rule was applied to every region* and contradicts t3443-basym-sweep A101/A102.
- **Lead on the error boundary.** All 22 numeric endpoint cases need 11..597 one-third contractions, and all 10 Excel
  `#NUM!` cases need 672..1667. Cumulative scale underflow at MIN_POSITIVE separates them without any input cutoff.

## Audit findings in pre-existing code (not related to the changes above)

From an independent read-only audit, highest severity first. Each was confirmed with a repro against the repo's own oracles.

1. **POISSON per-count fitted constants** (`poisson.rs` ~24–44, 72–78). Counts 2–5 use Stirling-error constants
   13–34 ulp off the true values, chosen to flip rounded exponents ("It is not Excel's stirlerr", "the one cell keeping
   it"). Counterexample: `POISSON.DIST(4,4.8,FALSE)` (t2055-poisson-pmf-sweep A48) is 1 ulp off, and `stirlerr_exact`
   gives Excel's value.
2. **GAMMA.DIST `HUGE_SHAPE_CDF_CUT = 1.85E+76`** (`gamma.rs` ~31–49). A two-cell fit, contradicted by the repo's own
   oracles: 32 CDF and 5 density error-vs-value reversals in fit-audit-t3271-cdf-cut and t3316-gamma-cdf-y-sweep.
3. **IRR training-set bounding boxes** (`financial_util.rs` ~315–455). 200 versus 50 iterations depending on a rectangle
   of flow count, peak and guess, plus a `guess == 0.1` classifier.
   - `IRR(VSTACK(-50,SEQUENCE(53,1,1,0)),-0.9)` returns a root, but appending a zero flow gives `#NUM!`.
   - t3478-irr-r4-wide A206 fails because its peak lies outside the box.
4. **IRR/XIRR on a huge blank range abort the process.** `=IRR(B1:XFC1000000)` hits an allocation failure (SIGABRT) before
   any budget check, because `financial/mod.rs` collects every coordinate before charging the lift cost. `SUM` on the same
   range is fine. This violates directive 5.
5. Incomplete-beta iteration exhaustion is returned as a successful probability. Gamma-product overflow silently zeroes
   representable beta tails.
6. Medium and low:
   - RATE's score-selected restart, despite a confirmed finite-root regression.
   - Matrix kernels that escape the work budget through cubic computation.
   - NPER's fallback overflows an avoidable intermediate.
   - An unexplained second PRODUCT accumulator in SUBTOTAL.
   - ROUND's x87 emulation ends at the sampled digit count rather than at a semantic boundary.

## Suggested probes

A set of 30 probe groups (about 5,800 cells, `rustsheet-case/v1` drafts with predicted values) covers every rule above that
was inferred rather than measured. Highlights:
- the CHISQ.INV.RT Q→P switch;
- the AS 109 accuracy exponent and endpoint saturation;
- the F back-transform;
- the Student-t joint stop and tiny centre;
- the LOGNORM density holdout and the Loader store chain;
- adversarial SKEW/KURT data;
- the BETA.INV number-versus-error boundary;
- T.DIST at df 999/1000/1001;
- SPEC-0036's merged-range operations.

Available on request as a separate PR.
