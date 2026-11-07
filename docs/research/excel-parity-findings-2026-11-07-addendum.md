# Addendum to the 2026-11-07 Excel-parity findings: inverse beta, F and Student-t inverse

Status: research hand-off, **docs only**. This supersedes the "Held" section of
`docs/research/excel-parity-findings-2026-11-07.md`. The inverse-beta / F / Student-t inverse prototype now passes every
gate: in a prototype on main `2e192ce48`, together with items 1–10 of the findings, the full eval reached Gate B 493/506,
with Gate A at **0 unlisted failing points and 0 stale exemptions**. `check` showed only the three pre-existing probe cases.
The engine (5742) and xlsx (583) tests, workspace clippy `-D warnings` and fmt were clean.

Sections A–D: functions newly at Gate B rate: **BETA.INV, BETAINV, T.INV.2T, TINV, CONFIDENCE.T, FINV, F.INV.RT, F.TEST**. F.INV goes
from 224 to 226/227 but is still one cell short. As before, re-implement through the work loop and re-measure on current
main.

## A. BETA.INV / BETAINV: bounded AS 109 Newton stopping points

**Excel.** Excel's inverse beta returns *AS 109 Newton iterates*, not the refined CDF crossing. Distinguishing cells:
- ironcalc BETADIST_BETAINV.xlsx H5: `BETA.INV(0.6734553797493481,3,600,2000,5489582)` = 33587.000000000226. The crossing
  gives …6.99999999992.
- H26 = 0.2339999999999406.
- H16 = −2.220446049250313E-16, an affine cancellation residual.
- gen-m9/4e55500bb8ad3cd8 A360: `BETAINV(1E-200,3.14159,TIME(4,2,1))` = 5.115610690476489E-64. This is the initial
  power-law estimate, i.e. the first best residual on a log-quantized CDF plateau.
- probes/t3862-beta-inv-sqrt-dbl-max H1/H3 (shapes 1E10, 1E14): the *smaller* shape controls the accuracy.
- BETA_GAMMA.xlsx BETA_DIST!J3 = 1.9999999999999998: finalize in the original coordinate. Complementing across the binade at
  ½ loses an ulp.

**Search profile** (one bounded loop in `inverse_beta.rs`, shared by callers through a profile enum):
- start from the shared *accurate* normal quantile;
- replace only *endpoint* estimates, by 1E-6 or 1−1E-6;
- residual divided by Excel's beta-density kernel;
- absolute correction accuracy `10^(−5/min(a,b)² − p^(−0.2) − 13)`, with no integer truncation and a 1E-300 floor on
  squared-step comparisons;
- on a genuine early stop, return the *pre-step* iterate;
- on plateaus, keep the first best residual.

**Convergence contract.** These rules are required to keep Gate A clean, and they are the ones a first attempt got wrong:
1. **Do not flush BETA.INV's residual.** A subnormal probability residual can imply a large coordinate correction. Flushing
   it returned the initial 1E-6 seed as a false root: accuracy/BETA.INV/s1 A111,
   `BETA.INV(2.22507385850721E-308,16.0270968311551,5.57477023741975)`. Only the F caller's residual flush is evidenced, and
   it must be rejected when the *unflushed* coordinate correction exceeds the R3 conditioning limit.
2. **Do not report an exhausted search as a root.** Distinguish a bracketed plateau or a representable endpoint saturation
   from an exhausted *unbracketed* search. Failure goes to the existing endpoint-aware crossing fallback, which yields
   Excel's `#NUM!`, not to "the nearest interior double". Gate A cells: accuracy/BETA.INV/s1 A89, A94, A184, and
   `BETA.INV(.1,5,1E-14)`.
   - *Separating number from error.* The cumulative one-third contraction count separates Excel's numeric endpoint results
     from its errors: 11..597 contractions for numbers against 672..1667 for errors. Its scale underflows at MIN_POSITIVE,
     which is a mechanism, not an input cutoff. It is still a hypothesis, so probe it (probe group G21).
3. **Keep the R3 math-over-Excel policy.** In the ordinary tiny-tail F.INV/BETA.INV region, compare the early iterate with
   the converged crossing and apply decision 0188's R3 limit (1E-4 relative discrepancy). The five R3-marked F.INV points
   (accuracy/F.INV/s1 A112, A126, A146, A160, A171) must follow the math, as `eval/accuracy/math-over-excel.tsv` requires.
   The separately evidenced F right-tail and near-one gamma-limit profiles are unaffected.
4. **Do not apply the early-iterate stop to every region.** Large-shape round trips (probes-numeric/t3443-basym-sweep
   A101/A102: `BETA.INV(BETA.DIST(x,a,b,TRUE),a,b)` returns x exactly in Excel) contradict a universal early stop. Probe group
   G23 maps the region.

## B. T.INV.2T / TINV / CONFIDENCE.T: joint AS 109 stop plus the small-beta centre

Built on findings item 8 (T.INV's `beta_small_quantile`).
- **Ordinary quantile.** It uses the shared AS 109 loop with a Student-t profile that stops only when *both* the absolute
  (R19) and the relative convergence tests hold. A sign-crossed last step counts as a crossing, not an early stop.
  Controls:
  - Exact as the beta iterate: (df, p) → Excel (5, .5) 0.7266868438004216; (7, .5) 0.7111417780817859; (8, .5)
    0.7063866126448375; (9, .5) 0.7027221467513249; (28, .5) 0.6833528429885039; (123456789, .5) 0.6744897431508498.
  - Must take one more step: df 6 and 123456791.
  - Must be preserved: gen-m9/0bbab620 A67, probes-numeric/m9-t-inv A120 and t3349-t-inv-2t-sweep A105.
- **Tiny centre.** The tiny-centre case reuses the small-beta seed and iterate, i.e. the grouped logarithmic seed, under
  the same omitted-term applicability rule.
- **CONFIDENCE.T** goes through T.INV.2T at df = size − 1.
- **Test expectations corrected to existing Excel oracles** (tolerances unchanged):
  - df 5, p = 1−1E-10 → 1.3171528710518458E-10 (gen-m9 b9a32ce9a1fd0d71 A58; probes-numeric/m9-t-inv-2t A65).
  - df 5, p = 1−1E-16 → 1.4623333233988838E-16 (gen-m9 afc00c8b70e78baf A64).

## C. FINV / F.INV.RT / F.INV: F caller arithmetic on top of the beta iterates

- **Back-transform.** A compensated four-log ratio `EXP(LN(x) − LN(1−x) + LN(df2) − LN(df1))`, keeping the small beta tail.
  For the complement use the shared log1p, `log1p(−x)`, not `LN(fl(1−x))`. Combined with a numerator-first compensated sum
  and a split EXP, this raised exact replay matches from 758 to 859/995.
- **Inverse-specific beta-series storage.** In the F inverse the series normalization is stored and the EXP argument split.
  The worksheet CDF keeps its historical extended normalization. That calls for a series profile selector
  (worksheet vs inverse-F) in `bratio/bpser.rs`.
- **Elementary tails and gamma-ratio normalization.** The elementary beta tails, and a *retained*
  `GAMMALN(a) + algdiv(a,b)` normalization through the density exponent and extended EXP, decide the AS 109 trajectory. Storing
  algdiv early loses a log. One example is accuracy/F.INV/s1 A167, `F.INV(4.36982683607437E-189,1,1698856)` =
  3.8539086753642276E-145; the mathematical root underflows, so returning zero would contradict Excel.
- **Half-half seed.** `F.INV(1E-200,1,1)` = 2.705579217114786E-151 needs the ASIN/power-law seed with its caller-specific
  square-root store and the stored 2/π multiplier.
- **Small shapes above one.** Inverse log-beta orders the shapes and stores the GAMMALN differences, as in the existing
  below-one branch. At (2.5, 4.5) this moves log-beta by one ulp and makes `F.INV(1-1E-16,9,5)` = 4311298.620581582.
- **Remaining.** F.INV has one cell left (226/227).

## D. F.TEST: paired normalizer and exponential storage

**Root cause.** For the beta-series region with min(a,b) > 1 and max(a,b) > 8, Excel's caller uses a balanced gamma-ratio
log-beta, not an accurate half-integer log-beta.

**Change.** With a = min, b = max, c = a+b and w = del(b) − del(c) (the existing TOMS algdiv correction):
- `h = store(a + w)`
- `h = store(ext(h) − ext(a)·LN(c))`
- `v = store((b − ½)·log1p(−a/c))`
- `L = store(ext(h) + v + GAMMALN(a))`
- Pair it with the series' split-exponent finish `(EXP(high)·series)·(1+low)`. The low correction comes *after* the
  multiplication by the series.

Changing only log-beta was the regression trap; the pairing keeps gen-m9/a790a4d4a6a82756 A289 and the
normalization-grid controls exact.

**Evidence.**
- gen-m9/b6593d7dd13f2dac A306: Excel 1.23966888990022E-69, now inside the 15-digit window.
- 581 direct F.TEST/FTEST calls: 329 → 393 exact, with no exact match lost.
- An independent 184-cell accuracy-table holdout: 107 → 110 exact.

No new coefficients, value-keyed tables or fitted cutoffs.

## E. Alternative for the Student-t inverses: an AS 109-shaped search on BETA.DIST's own CDF

This is an independent prototype, built *directly on main* `2e192ce48` without A–D. It is self-contained and on its own
passes all gates: Gate B 462 → 466 (T.INV, T.INV.2T, TINV, CONFIDENCE.T), Gate A exit 0, `check` showing only the three
known probes, 5717 engine tests, and clean clippy and fmt. It is an alternative to item 8 of the findings plus section B
here; pick one so there is a single definition.

**Excel.** From df 2, these functions return an iterate of an inverse incomplete-beta search shaped like AS 109, run on
Excel's *own* incomplete beta (BETA.DIST's CDF). They return neither the exact quantile nor the double nearest the
crossing. The current engine is 5–220 ulp off in three regimes:
- *Power-law tail:* the result is AS 109's first estimate `EXP((LN(P)+(LN(a)+lnB))/a)`, or a few Newton hops from it. The
  kernel's `EXP(a·LN x − lnB)` prefactor makes the CDF a staircase in x, and the first iterate with the smallest |cdf − P|
  is returned.
- *Two-tailed P near ½:* the search stops at a Newton step of about one relative epsilon and returns that iterate, up to
  3 ulp of y above the crossing.
- *P above 0.9:* the search runs on the swapped shapes (½, df/2) with t = SQRT(df·z/(1−z)). That explains why Excel's
  centre is 5–24 ulp off the exact quantile (T-3421).

The first estimate uses an accurate normal quantile where AS 109 has Hastings' approximation; at huge df and tiny p, Excel
returns that estimate unrefined, matching bit for bit on 41 of 50 points. The square root of the back-transform is SQRT's
twice-rounded one.

**Change.**
- An `inverse_beta::ExcelBetaSearch` running on `beta::beta_dist_cumulative`, BETA.DIST's CDF factored out of the worksheet
  function, with bpser's own ln B.
- `t_dist.rs` uses it in three places: the tail (`t_tail_search`), near ½ (`T_SEARCH_MIN_ACCURACY`) and the centre below
  df 16 (`t_centre_search`). Everything else keeps its existing path.
- The two fitted centre cuts (0.075 and 0.1 on |1 − p|) become a single cut at a two-tailed 0.9 on either sign
  (`T_CENTRE_MIN_P`).

**Evidence.** Literal-argument Excel cells of gen-m9, probes and probes-numeric, df 2 and up:
- tail: 87 → 212 of 213 bit-exact;
- P near ½: 43 → 60 of 61;
- centre below df 16: 16 → 114 of 115.
`check` shows 142 gen-m9, 10 ironcalc-calc and 36 probe cells fixed, with no new regressions.

## Gate A bookkeeping

With these changes Gate A reports stale exemptions for points that are now inside the envelope. Remove those lines from
`eval/accuracy/known-failures.tsv` as the gate instructs; this only tightens the gate.

The suggested probe groups G06, G07, G10, G11, G13, G21, G22 and G23 (docs/research/probe-suggestions-2026-11-07) target
exactly the inferred parts of A–C.
