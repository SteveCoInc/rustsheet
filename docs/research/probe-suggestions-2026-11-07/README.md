# Excel probe suggestions for the 2026-11-07 Excel-parity findings

Companion to docs/research/excel-parity-findings-2026-11-07.md. Predictions come from a prototype build of those fixes on main `2e192ce48` (not merged). This is a list of Excel questions for agents with the
`excel-graph` oracle. Each group checks a rule that the recent fixes **inferred** (from cached cells, Newton
trajectories or arithmetic replay) rather than measured, or a boundary the docs mark UNVERIFIED / open / "not established".

- **P1**: the result could falsify a shipped fix.
- **P2**: pins an open boundary.
- **P3**: nice to have.

## Files

| Path (under `probes/`) | What |
|---|---|
| `cases/val-*.case.json` (20) | Draft `rustsheet-case/v1` probe cases. One sheet `P`, formulas only, `"expected": {}`, tags `M9 statistical probe` (IRR: `M9 financial irr probe`). Copy into `eval/suites/probes/cases/`, rename to a task id (`tNNNN-...`) and run `uv run rustsheet-oracle fill -b excel-graph --suite probes`. |
| `predicted/val-*.predicted.json` | The engine's value for every cell (`{"t","v","formula"}` per cell, same addressing as the case). This is not an oracle sidecar and must not be committed as one. |
| `gen_cases.py` | Regenerates every case (stdlib only). |
| `merged/merged-ops.procedure.json` | The merged-range operation scripts (groups M1 to M6) with the engine's expected outcome. |
| `merged/build_merged_fixture.py` | Builds the start workbook `merged-covered-ops.xlsx` . |
| `merged/engine-predictions.txt` | The prototype's output for M1 to M6. |

**How the predictions were computed.** A release build of the prototype was run on the draft cases with `rustsheet-eval show probes/<id>.case.json --all` (each case mounted with a placeholder sidecar), and the `actual` lines were parsed into `predicted/`. The merged-range outcomes come from a small program linking `rustsheet_engine` and `rustsheet_xlsx` that loads the start workbook and calls `unmerge_cells`, `range_clear_contents`, `copy_to_clipboard` / `paste_from_clipboard`, `auto_fill_rows/columns`, `on_navigate_to_edge_in_direction` and `set_user_input`. The predictions are the prototype's values, not Excel's; a probe that disagrees falsifies the corresponding rule.

**Probe hygiene.** Literals carry at most 15 significant digits (the playbook's "Traps" rule). Adjacent doubles are
built as `x0 ± j*2^-k` expressions, and exact data doubles go in cells, not array constants. CHISQ.* truncate df, so
every chi-square df is an integer.

## Overview

| # | Group | Pri | Case | Cells | Question |
|---|---|---|---|---:|---|
| G01 | CHISQ.INV.RT Q-to-P switch | P2 | val-chisq-inv-rt-q-to-p-switch | 251 | Where in (.9999, .9999462] Excel stops inverting the rounded Q |
| G02 | Gamma inverse unit-x .5 boundary | P2 | val-chisq-inv-unit-half-boundary | 375 | Does the leading-factor start + Halley path extend past unit x = .5? |
| G03 | Split complement region | P1 | val-gamma-split-complement-region | 255 | Is the upper tail (0.5-P)+0.5 exactly in a>1, unit x<=1, and 1-P elsewhere? |
| G04 | Small-x prefactor boundary | P2 | val-gamma-small-x-prefactor-boundary | 360 | Edges of T-3219's stored-log prefactor (x/beta = 1, shape just above 1, shape near 171) |
| G05 | Far upper tail at other shapes | P2 | val-gamma-far-upper-tail-shapes | 110 | T-3469's reciprocal log at shapes other than 6 |
| G06 | AS 109 accuracy exponent | P1 | val-beta-inv-as109-accuracy-exponent | 624 | Is 10^(-5/min(a,b)^2 - p^-0.2 - 13) untruncated, on min(a,b) and the oriented p? |
| G07 | AS 109 endpoint replacement and saturation | P1 | val-beta-f-inv-endpoint-saturation | 280 | The 1E-6 / 1-1E-6 replacement and damped far-tail values across shape regions |
| G08 | F inverse back-transform | P1 | val-f-inv-back-transform | 384 | Compensated four-log ratio vs the direct quotient |
| G09 | F tiny-tail neighbourhood | P2 | val-f-inv-tiny-tail-neighbourhood | 90 | Dispatch near the two remaining F.INV misses |
| G10 | F AS 109 tolerance cap | P1 | val-f-inv-as109-tolerance-cap | 175 | EPSILON^2 absolute cap plus a unit-roundoff relative stop at symmetric df |
| G11 | Student-t joint stop | P1 | val-t-inv-joint-abs-rel-stop | 110 | Absolute (R19) AND relative stop, with no df window |
| G12 | Student-t q≈.25 stop | P2 | val-t-inv-quarter-stop | 112 | The unresolved ordinary stop near p = .25 |
| G13 | Tiny Student-t centre region | P1 | val-t-inv-tiny-centre-region | 147 | Small-beta reuse only where df<16 and the first series term <= EPSILON |
| G14 | Student-t deep tails | P2 | val-t-inv-small-beta-tails | 60 | The seed+Newton tail, and the df-2 underflow switch between 1E-100 and 1E-200 |
| G15 | T.DIST df 1 and large-df centre | P3 | val-t-dist-df1-and-large-df-centre | 76 | Rounded-y at df 1, and the df >= 1000 central fraction boundary |
| G16 | NORM.S.INV central fingerprint | P3 | val-norm-s-inv-central-fingerprint | 226 | Dense data for inverse-erf candidates around the x^2 = .5 split |
| G17 | LOGNORM.DIST density holdout | P1 | val-lognorm-density-holdout | 148 | Four-term two_sum chain with the positive low-part sign at non-dyadic sigma |
| G18 | Loader store chain | P1 | val-binom-loader-store-chain | 234 | bd0's x87 cutoff at exact-boundary pairs; stored vs extended chains |
| G19 | SKEW/KURT x87 | P1 | val-moments-x87-adversarial | 204 | Double-rounded stored ops and the (z*z)*(z*z) order on adversarial data |
| G20 | IRR negative discount and caps | P2 | val-irr-negative-discount-and-caps | 63 | Secant crossing d>1, final-domain #NUM!, the open fallback targets |
| G21 | BETA.INV number vs #NUM! at tiny shapes | P1 | val-beta-inv-tiny-shape-num-boundary | 272 | Where Excel switches from a number to #NUM! on unbracketed or stalled searches |
| G22 | BETA.INV at DBL_MIN probabilities | P1 | val-beta-inv-dbl-min-residual | 132 | Does Excel flush a subnormal first residual? |
| G23 | BETA.INV large-shape round trip | P1 | val-beta-inv-large-shape-round-trip | 861 | Where BETA.INV(BETA.DIST(x)) returns x vs an early iterate |
| G24 | T.DIST central tail at df 999-1001 | P1 | val-t-dist-df1000-central-endpoint | 198 | Is the finite-series domain inclusive at df 1000? |
| M1 | Unmerge with covered content | P1 | merged U1 | - | Does covered content show again? |
| M2 | Clear contents of a merged range | P1 | merged U2, U2b | - | Are covered cells cleared too? |
| M3 | Copy / cut-paste of a merged range | P1 | merged U3, U3b, U4 | - | Does covered content travel with the merge? |
| M4 | Autofill over merged ranges | P1 | merged U5, U5b, U5c | - | Tiled covered content; are progressions computed over non-covered cells only? |
| M5 | Ctrl+arrow on merged ranges | P2 | merged U6 | - | Is a merged cell as empty as its anchor? |
| M6 | API write into a covered cell | P3 | merged U7 | - | Refused or accepted? |

Totals: 30 groups (18 P1, 9 P2, 3 P3), about 5,800 formula cells plus 12 merged-range scenarios.

The groups below list each case's row and column layout in its `description`. A value quoted as "engine" comes from
`predicted/`.

---

## G01. CHISQ.INV.RT: where Excel switches from inverting Q to inverting P (P2)

**Question.** T-3146 found two things. Through p = .9999 Excel's quantile carries an ulp-scale error of the rounded Q.
By .9999462 (t3311-fresh-sweep A197) it carries an ulp-scale error of P. Where between those points does the switch
happen? Does the switch depend on df?

**Why it matters.** `closed_form::excel_halley_quantile` hard-codes `Tail::Upper if p <= 0.9999` (Halley on Q). Above
that it returns `None`, so the small-P solver runs. A switch elsewhere, or one that depends on df, moves every cell in
(.9999, switch].

**Cells.**
- Columns A:E: CHISQ.INV.RT at df 3, 5, 4, 7 and 10.
- Column F: CHIINV at df 3. Column G: CHISQ.INV(1-p, 3).
- Rows 1-33: p = .9999 + k·2E-6 (k = 0..27), .9999462, .99996, .99997, .99998 and .99999. These extend to
  .99999 because the audit notes the cut is only bracketed between .9999 and .99999.
- Rows 34-41: .9999 ± 1..4·2^-53, which tests whether the cut is `<=` or `<`.

**Engine.**
- A1 `CHISQ.INV.RT(.9999,3)` = 0.005214832329030269 (the documented Excel value).
- Above .9999 the engine is on the P solver, and its RT value differs from CHISQ.INV(1-p) in the last digits. For
  example, A2 = 0.005144995535215276 vs G2 = 0.0051449955352152795.

**Reading the results.** Fit the first p at which Excel's value equals the P-solver value. If that p is the same for
every df, the rule is a probability cut. If it moves with df, it is a residual-scale rule.

## G02. Gamma/chi-square inverse around unit x = .5 (P2)

**Question.** The leading-factor start `EXP((LN(p)+GAMMALN(a)+LN(a))/a)` followed by Halley steps is documented only
for unit x <= .5. Above that, "the start remains unresolved". What does Excel return just below and above chi-square
x = 1?

**Why it matters.** `excel_halley_quantile` rejects a start above 0.5 and falls back to the accurate crossing. If Excel
keeps Halley iterates beyond .5, the crossing is wrong there. If Excel already stops below .5, the shipped region is
too wide.

**Cells.**
- Per df 3, 5, 8, 12 and 20, three columns: forward `CHISQ.DIST.RT(x,df)`, `CHISQ.INV.RT(<that cell>,df)`, and
  `CHISQ.INV(CHISQ.DIST(x,df,TRUE),df)`.
- Rows: x = .80 .. 1.20 in steps of .02, then .996, .998, 1.002 and 1.004.

**Engine.**
- Round trips land within a few ulps of x: B10 (df 3, x = .98) = 0.9800000000000001, and E10 (df 5) =
  0.9799999999999994.
- Recording the forward cell lets you check the forward CDF against Excel independently.

## G03. Split complement (0.5-P)+0.5: where it applies (P1)

**Question.** Is Excel's upper gamma tail `(0.5-P)+0.5` exactly where the engine uses it (shape a = df/2 > 1, unit
x <= 1)? Is it `1-P` elsewhere?

**Why it matters.** The rule rests on 51 cells of one flat band (t3148). `small_x_factors` limits it to
`a > 1 && x <= 1`. The same double rounding could just as well apply to every upper tail computed from P, or not at
small shapes.

**Cells.**
- For df 2, 3, 4, 5, 9, 17, 39, 40 and 41, three columns: CHISQ.DIST.RT, `1-CHISQ.DIST(x,df,TRUE)`, and
  `(0.5-CHISQ.DIST(x,df,TRUE))+0.5`.
- Every row is an x where the two complements of the engine's P differ, found by a dense x = .01..6 scan, so each
  cell discriminates on its own.
- Controls outside the engine's region:
  - df 2, where a = 1 and the engine uses the direct form.
  - df 9 and 17 at x > 2.
  - df 39, 40 and 41 at x > 2.
- x lists per df:
  - 2: .04 .08 .09 .11 .15
  - 3: .07 .08 .1 .12 .14 .34 .38 .64
  - 4: .01 .06 .12 .17 .27 .49 .54 .71
  - 5: .06 .18 .3 .47 .71 .89 1.1 1.33
  - 9: .03 .25 .39 .54 .83 1.13 1.46 1.62 | 2.07 2.32 2.58 2.69 2.93 3.31 3.7 3.87
  - 17: .1 .35 .5 .75 .88 1.16 1.44 1.71 | 2.03 2.48 2.9 3.52 3.93 4.22 4.58 5.11
  - 39, 40, 41: eight x values each in 2.5..5.8

**Engine.** RT equals the split column inside the region and the direct column outside it.
- In the region: D1 `CHISQ.DIST.RT(.07,3)` = 0.9951764815745945 (split), where 1-P gives ...944.
- Outside: A1 `CHISQ.DIST.RT(.04,2)` = 0.9801986733067553 (direct), where split gives ...554. Also M9
  `CHISQ.DIST.RT(2.07,9)` = 0.9903116098418613 (direct).

**Falsified if** Excel's RT matches the split form at a = 1, at x > 2 or at df >= 40, or matches the direct form
inside the region. The forward P must match first: check `CHISQ.DIST` in the B/C operands.

## G04. Small-x stored-log prefactor boundary (P2)

**Question.** T-3219 uses `a*LN(x/beta)` and `EXP(GAMMALN(a+1))` for shape > 1 and x/beta <= 1. Do the edges agree with
Excel?
- x/beta = 1 ± ulps.
- Shape 1+2^-52 and 1.0001.
- Shapes near 170-172, where EXP(GAMMALN(a+1)) overflows and the stable prefactor takes over.
- beta = 2, which tests whether x/beta is stored before LN.

**Cells.** Per shape (1+2^-52, 1.0001, 2, 8, 30, 100, 170, 170.5, 171, 172), one CDF column and one density column.
Rows 1-9 use beta = 1 at x = 1-4u, 1-u, 1, 1+2u, 1+8u, .999, 1.001, .5 and 1E-3. Rows 10-18 repeat them with beta = 2.

**Engine.**
- `GAMMA.DIST(1,1+2^-52,1,TRUE)` = 0.6321205588285578 and at 1+2^-52 it is ...576. Watch this step across x = 1.
- Shapes >= 171 give 0 (subnormal underflow) at x = 1. Excel's flush or subnormal output is itself informative.

## G05. Far-upper-tail reciprocal log at other shapes (P2)

**Question.** Does `B = a*LN(a/x)+(x-a)` (T-3469) hold at shapes 1.5, 2.5, 6, 15, 40 and 100, for x/a from 3 to 30?

**Why it matters.** The evidence was mostly shape 6 and x/a = 10. The switch domain's own boundaries are open (T-3640).

**Cells.** `CHISQ.DIST.RT(2·a·r, 2a)` and, at integer a, `POISSON.DIST(a-1, a·r, TRUE)`, the same Q.

**Engine.**
- E6 `CHISQ.DIST.RT(120,12)` = F6 `POISSON.DIST(5,60,TRUE)` = 6.180223580811598E-20.
- K6 `CHISQ.DIST.RT(2000,200)` = 6.0358275296307356E-294, while its Poisson twin L6 = 6.035827529631422E-294. The two
  paths diverge at a = 100, so an Excel answer settles which path is right there.

## G06. BETA.INV AS 109 accuracy exponent (P1)

**Question.** Is the absolute correction accuracy `10^(-5/min(a,b)^2 - p^(-0.2) - 13)`, untruncated, with min(a,b),
and with p the oriented probability (<= .5)?

**Why it matters.** This profile is "inferred from Newton trajectories". AS 109 truncates the exponent to an integer
and uses the first shape, so each variant stops at a different iterate exactly where the exponent crosses an integer.

**Cells.**
- The small shape m takes the values 1, 1.1, 1.118, 1.12, 1.25, 1.29, 1.3, 1.58, 1.59, 2.23, 2.24 and 3. These
  straddle the points where -5/m^2 crosses an integer: m = 1, 1.118, 1.291, 1.581, 2.236.
- The other shape is 3 (first 24 columns) or 10 (next 24). Column pairs are (m, other) then (other, m), so both
  orientations are covered.
- p = .001, .0041, .0042, .01, .03, .03125, .032, .1, .3, .5, .7, .96875, .99. This straddles the points where
  p^-0.2 = 2 and 3, and their mirrors.

**Engine.**
- `BETA.INV(.03125,1,3)` = 0.01052709757989863 and `(.03125,3,1)` = 0.3149802624737183.
- `BETA.INV(.03125,1.25,3)` = 0.022605778221640397.

**Reading the results.** A truncated or first-shape variant would disagree with the engine in whole bands of rows or
columns (for example all m in (1.118, 1.29) at one p), not in isolated cells.

## G07. AS 109 endpoint replacement and far-tail saturation (P1)

**Question.** Is only an out-of-range start replaced, by 1E-6 or 1-1E-6? Does the damped trajectory give Excel's
far-tail values across shape regions?

**Cells.**
- BETA.INV at shapes (.5,.5), (1,1), (1,3), (3,1), (.1,10), (10,.1), (5,5), (50,50), (.5,50), (50,.5) and (2,5E9).
- F.INV and F.INV.RT pairs at df1 in {1, 2} and df2 in {1, 2, 3, 5, 10, 30}.
- p = 1E-20, 1E-50, 1E-100, 1E-200, 1E-300, 1-1E-12, 1-1E-15 and 1-2^-52.

**Engine.** It returns `#NUM!` in 17 cells, wherever the root underflows or rounds to 1:
- BETA.INV(1E-200 or 1E-300, .5, .5) and (1-1E-12 .. 1-2^-52, .5, .5).
- (.1,10) at p <= 1E-50, and the mirrored (10,.1) at p >= 1-1E-12.
- (.5,50) at p <= 1E-200, and the mirrored (50,.5) at p >= 1-1E-12.

Two numeric values to note:
- `F.INV(1E-200,1,1)` = `F.INV(1E-300,1,1)` = 2.7055792171152472E-151. This is the documented half-half saturation.
- `BETA.INV(1E-300,1,1)` = 1.0000000000000237E-300.

**Falsified if** Excel returns 0 or a number where the engine gives #NUM!, or if the saturation constant differs at a
shape other than (1,1).

## G08. F inverse back-transform (P1)

**Question.** Is F.INV built from the beta quantile x as the compensated four-log
`LN(df2)+LN(x)-LN(df1)-LN(1-x)` (TwoSum, then `EXP(s)*(1+err)`)? Or is it the direct quotient
`df2*x/(df1*(1-x))`?

**Cells.**
- Per df pair (1,1), (2,3), (5,5), (4,10), (10,4), (30,7), (1,100) and (100,1), four columns:
  1. F.INV in rows 1-6, F.INV.RT in rows 7-12.
  2. BETA.INV of the same tail.
  3. The direct quotient of that beta cell.
  4. The worksheet four-log EXP form.
- p = 1E-10, .01, .3, .5, .9 and .999.

**Engine.**
- `F.INV(1E-10,1,1)` = 2.4674011002723316E-20 = the four-log column. BETA.INV = 2.4674011002723397E-20 = the
  quotient. This is the documented pair.
- `F.INV(.3,2,3)` = 0.40265143230557304, while the quotient gives ...293 and the worksheet four-log gives ...29.
  Excel matching ...304 supports the compensated form over both naive forms.
- Six BETA.INV(1-1E-10, ·) cells are #NUM! in the engine. Excel returning a number there is a separate finding.

## G09. F.INV tiny-tail neighbourhood (P2)

**Question.** The two remaining F.INV misses are:
- `F.INV(1-1E-16,9,5)`: Excel 4311298.620581582, engine 4311298.62058159.
- `F.INV(1E-300,9,3)`: Excel 5.814110643623313E-68, engine ...4775E-68.

What dispatch rule (small-tail beta series; stop on a subnormal residual) covers their neighbours?

**Cells.**
- Columns: df1 = 7..11.
- Rows 1-5: F.INV(1-1E-16, df1, 3..7).
- Rows 6-9: F.INV(1E-300, df1, 2..5).
- Rows 10-13: F.INV(1E-250, df1, 2..5).
- Rows 14-18: F.INV.RT(1E-16, df1, 3..7).

**Engine.** C3 and C7 reproduce the two known misses, so the Excel values above confirm the oracle fill. The other 88
cells map out how far the miss pattern extends.

## G10. F AS 109 tolerance cap and relative stop (P1)

**Question.** Does F's search cap the absolute tolerance at EPSILON^2 and also stop on a unit-roundoff relative step?
`F.INV(.4,5,5)` is the only single control for this.

**Cells.**
- Columns A:K: (k,k) for k = 2..12. Rows 1-7 are F.INV and rows 8-14 FINV, at p = .3, .35, .4, .45, .55, .6 and .7.
- Columns L:N: F.INV at (3,6), (5,8) and (8,11).

**Engine.** `F.INV(.4,5,5)` = 0.7878805312664828 and `FINV(.4,5,5)` = 1.2692279607322503.

## G11. Student-t ordinary inverse joint stop (P1)

**Question.** Does Excel keep the AS 109 iterate exactly when the last step meets both
`10^(-13-2.5/a^2-0.5/p^2)` and `|step| <= EPSILON*x`, with no df window?

**Cells.**
- Column A: T.INV.2T(.5, df) for df 2..30 and 123456785..123456795. The documented controls are df 5 vs 6 and
  123456789 vs 123456791.
- Columns B:H: T.INV.2T(p, df) at p = .2, .3, .4, .6, .8, 1.2 and 1.5, for df 3..12.

**Engine.**
- df 5 = 0.7266868438004216, df 6 = 0.7175581964914122.
- df 123456793 = 0.6744897337563592, df 123456794 = 0.674489756809267, df 123456795 = 0.6744897595409505.

**Falsified if** any df in a dense run flips between "kept" and "refined" against the engine's prediction.

## G12. Student-t ordinary stop near q = .25 (P2)

**Question.** The q≈.25 stopping rule is "unresolved". AS 109/R19 reproduces df 5, 7, 8, 9 and large df, but not
every control.

**Cells.**
- Columns: T.INV at df 5, 7, 8, 9, 6, 10 and 1E6.
- Rows: p = .24, .245, .249, .2499, .25 ± 1 and 4 ulps (2^-54), .2501, .251, .255, .26, .75, .125 and .375.

**Engine.**
- `T.INV(.25,5)` = -0.7266868438004216. One ulp below .25 gives -0.726686843800423, and one ulp above gives
  -0.7266868438004223.
- The step across p = .25 is the shape to compare with Excel.

## G13. Tiny Student-t centre: region of the small-beta reuse (P1)

**Question.** Is the small-beta inverse reused exactly in bpser's split-prefactor branch (min(a,b) < 1,
max(a,b) < 8, so df < 16) and where `|a(1-b)z/(a+1)| <= EPSILON`? Is the direct-centre search right outside it?

**Cells.**
- Columns A:R: TINV(1-c, df) for df = 1..18.
- Columns S, T: T.INV.2T at df 15 and 16.
- Column U: CONFIDENCE.T(1-c, 1, 16).
- Rows: c = 2^-52, 1E-15, 1E-14, 1E-12, 1E-10, 1E-8 and 1E-6.

**Engine.**
- `TINV(1-2^-52,2)` = 3.140184917367558E-16 (a documented Excel value).
- df 15, 16, 17 at c = 2^-52: 2.8296522872116646E-16, 2.826712359822621E-16, 2.824120214887697E-16.
- `T.INV.2T(1-1E-14,15)` = 1.2733435292452466E-14 and at df 16 = 1.2720205619201792E-14.

**Falsified if** Excel keeps the seed (the engine's region behaviour) at df >= 16, or refines inside the region.
Compare against the ulp pattern between rows 2 and 3, where the series condition flips.

## G14. Student-t deep tails (P2)

**Question.**
- Does the grouped log seed + Newton hold for p from 1E-20 to 1E-300 at df 1, 2, 3, 5, 8, 15 and 30?
- At df 2, where between 1E-100 and 1E-200 does the squared Newton correction underflow so the log seed is kept?

**Cells.** Columns A:G: the df grid. Column H: T.INV(1E-k, 2) for k = 100..200 in steps of 10.

**Engine.**
- The df-2 column alternates in its last digits from 1E-150 onward: -7.071067811865475E+64 at 1E-130, then
  -7.071067811865421E+74 at 1E-150, -7.071067811865485E+79, -7.071067811865549E+84 and so on.
- The first k where Excel's digits leave the Newton-refined value pins the switch.

## G15. T.DIST at df 1 and the large-df centre (P3)

**Cells.**
- Columns A, B: T.DIST.2T and T.DIST.RT at df 1, for t = 1E-10 .. 1.5, including 1-2^-53, 1, 1+2^-52 and 1.005-1.
- Columns C:H: T.DIST.2T(t, df) for t = .1, .5, .9, .999999, 1 and 1.000001, at df = 500, 998, 999, 1000, 1001, 1E4,
  1E6, 1E8 and 1E10.

**Engine.**
- `T.DIST.2T(1.005-1,1)` = 0.9968169276635932 (documented).
- `T.DIST.2T(1,1)` = 0.5000000000000001. Excel showing exactly .5 would point at the t = 1 boundary kernel.
- df 999 / 1000 / 1001 at t = .5: 0.6171851909302892 / 0.6171850808338748 / 0.617184970957399.

## G16. NORM.S.INV central fingerprint data (P3)

**Question.** This is input for `docs/research/t0925/lattice.py`, not a pass/fail probe. Is the branch change at
x^2 = .5 (multiply image below, divide image above) sharp? Is the low-x region odd-symmetric?

**Cells.**
- Column A: p = .5 + x/2. Column B: p = .5 - x/2.
- x runs over .700..715 in steps of .0005, .84..86 in steps of .001, and .49..51 in steps of .001.
- Columns C and D: p = .5 + k·2^-30 and .5 + k·2^-12 (k = 1..40), where x is exact.

**Engine.** The engine uses accurate AS241, so it is known not to match bit for bit in this region. Its values are
reference points, not a claim.

## G17. LOGNORM.DIST density holdout (P1)

**Question.** Does the T-4321 chain hold away from the 120-cell dyadic grid?
- Terms: `((d*d)/(s*s))/2`, `LN(s)`, `LN(2.5066282746310007)`, `LN(x)`.
- Arithmetic: two_sum per addition, then `EXP(-h)*(1+e)` with the positive low-part sign.

**Cells.**
- sigma = .1, .3, .7, 1.3, 2.9 (outer) × mean = -2, 0, 1, 3.5 (inner).
- Rows: x = EXP(mean + k·sigma) as 15-digit literals, for k = -3, -1.5, -.5, 0, .5, 1.5 and 3.
- Last column: dyadic controls at (x, 1, .5).

**Engine.** `LOGNORM.DIST(1,1,.5,FALSE)` = 0.10798193302637615 (documented), and
`LOGNORM.DIST(40,1,.5,FALSE)` = 1.0469325416527326E-08.

**Falsified if** more than a handful of non-dyadic cells miss. A miss pattern that tracks sigma would point at the
`(d*d)/(s*s)` order. One that tracks the binades of h would point at the sign.

## G18. Loader store chain (P1)

**Questions.**
- (a) Is bd0's x87 cutoff `|k-M| < RN64(.2*(k+M))` right at exact-boundary integer pairs, where binary64 and x87
  disagree? `BINOM.DIST(2,10,.3,FALSE)` is the single control.
- (b) Does the unscaled binomial store its log-deviance chain while NEGBINOM (scaled) keeps the extended exponent?

**Cells.**
- Column A: BINOM.DIST(k, n, p, FALSE). Column B: BINOM.DIST.RANGE one-point. Column C: NEGBINOM.DIST(n-k, k, p, FALSE).
- These cover every (n, p, k) with n in {5, 10, 20, 30, 50, 100} and p in {.1 .. .9} where |k-np| or |(n-k)-n(1-p)|
  equals .2× their sum exactly (74 rows).
- Column D: far-tail log-branch controls.

**Engine.**
- `BINOM.DIST.RANGE(7,1E-20,6)` = `BINOM.DIST(6,7,1E-20,FALSE)` = 6.999999999999857E-120.
- `BINOM.DIST(2,10,.3,FALSE)` = 0.23347444050000005.

**Falsified if** column A misses at boundary rows where the engine's x87 cutoff and a binary64 cutoff disagree, or if
NEGBINOM misses where it shares the Loader terms.

## G19. SKEW / SKEW.P / KURT x87 double rounding (P1)

**Question.** Is every stored op rounded to a 64-bit significand and then to binary64? Are fourth powers `(z*z)*(z*z)`?
This is checked on data built to expose double rounding.

**Cells.** Row i holds A SKEW, B SKEW.P and C KURT of F{i}:Z{i}. The data cells are formulas, so 1+2^-52 and
1E15+1 survive. The 26 data sets are:
- {k·1E-15, 1, 2} for k = 1..9.
- {.1, 42, big, .1} with big = 123456789.123, 987654321.5, 1E9+.5 and 33554432.1.
- The documented KURT controls.
- {1,1,1,1,1+2^-52}, {1E15, +1, +2, +4}, {1E8, +1, +3, +7, +15}, a run of 16 threes and a 7.
- Symmetric sets, a dyadic set, and others.

**Engine.**
- `SKEW({1E-15,1,2})` = 2.4980018054066022E-15 (documented).
- `KURT({.1,42,123456789.123,.1})` = 3.9999999999979536 (documented).
- SKEW({1,1,1,1,1+2^-52}) = 3.3333333333333335 and KURT of the same set = 12.0. A degenerate variance close to zero
  is the most sensitive cell here.

## G20. IRR negative-discount crossing, final domain, open fallback targets (P2)

**Status.** The T-2928 fix is in the build: the secant may cross d > 1, the final discount must be positive, and a
stalled step is scaled by the discounted terms.

**Cells.**
- Columns A, B, C: guess default, .5 and -.5. The rows are these 21 flow vectors:
  - {.3,-2.5,1.005} and its variants: 1.0049, 1.0051, negated, ×10, ×.1, -2.6 and -2.4.
  - {1,7,-1}, {1,20,-1}, {1,6,-1}, {1,8,-1}.
  - {42,1,1,-1}, {40,1,1,-1}, {44,1,1,-1}.
  - {-400,1,1,1,1,1}, {-300,...}, {-500,...}.
  - {-4,4}, a 13-period near-zero flow, and a long-run {-1E8, 1×7, 1E8}.

**Engine.**
- A1 = 6.909808543634389, the documented Excel value. Before the fix it was ...611, and 13 of the 63 cells changed
  (rows 1-8).
- `{42,1,1,-1}` = -0.7467044766537408 and `{-400,1,1,1,1,1}` = -0.6737366751808422, unchanged by the fix. These are the open
  fallback targets, so their Excel values are new evidence.
- `{44,1,1,-1}` = -0.7500000000000002.

## G21. BETA.INV: number vs #NUM! at tiny shapes and extreme p (P1)

**Question.** Where does Excel return #NUM! instead of a number when one or both shapes are tiny? The audit found that
the shipped AS 109 profile returns a finite value wherever its search stalls or is unbracketed:
- s1 A89: 0.9999999999999999.
- s1 A94: 1.1102230246251565E-16.
- s1 A184: the 1E-6 endpoint replacement itself.

Excel returns #NUM! at all three.

**Why it matters.** "Finite" is the current success contract. The fix needs the region where Excel fails, and this
group maps it.

**Cells.**
- Columns A:X: the other shape is .5 (A:H), 1 (I:P) or 5 (Q:X), and the tiny shape is 1E-3, 1E-6, 1E-9, 1E-12,
  1E-14, 1E-15, 1E-16 or 1E-20.
  - Rows 1-5: BETA.INV(p, other, tiny). Rows 6-10: BETA.INV(p, tiny, other).
  - p = .001, .1, .5, .9 and .999.
- Column Y:
  - The three audit cells and `BETA.INV(.1,5,1E-14)`.
  - BETAINV twins.
  - `BETA.DIST(1-2^-53,5,1E-14,TRUE)`, which shows that the returned 1-2^-53 is no root.
- Column Z: both shapes tiny, (a, b) in {1E-3, 7E-4, 1E-6} × {1E-100, 1E-279}, at p = 1E-100, 2.3E-261, 1E-300
  and .5.

**Engine.**
- 172 of 272 cells are #NUM!. In columns A:X, 68 of the 86 finite cells are the suspect values: 0.9999999999999999,
  1.1102230246251565E-16, 1E-6 or 0.999999.
- Examples: `BETA.INV(.1,.5,1E-3)` = 0.9999999999999999 and `BETA.INV(.9,1E-3,.5)` = 1.1102230246251565E-16.
- Y1:Y3 reproduce the audit's values. `BETA.DIST(1-2^-53,5,1E-14,TRUE)` = 3.2462921240039577E-13.

**Reading the results.** For each shape pair, the first p at which Excel goes #NUM! is the boundary the fix needs. A
finite Excel value in a cell that predicts a suspect value is equally informative.

## G22. BETA.INV at probabilities around DBL_MIN (P1)

**Question.** When the first CDF residual is subnormal, does Excel flush it (accepting the start) or keep it? Accuracy
table s1 A111, `BETA.INV(2.22507385850721E-308,16.0270968311551,5.57477023741975)`, is 3.6855664841262965E-20 in
Excel. The engine flushes and returns 2.2394449712624932E-20.

**Cells.**
- Columns A:J: shapes (16.0270968311551, 5.57477023741975), (16, 5.5), (2, 2), (5, 5), (1, 1), (.5, .5), (3, 30),
  (30, 3), (100, 100) and (1.5, 300).
- Column K: the forward CDF of column A's quantile.
- Column L: F.INV(p, 5, 5).
- Rows: p = 1E-300, 1E-307, 4E-308, 3E-308, 2.5E-308, 2.22507385850721E-308, 2^-1022, 2.2250738585072E-308,
  2^-1022·(1+2^-52), 2^-1022·(1+2^-40) and 2^-1021.
- Row 8's literal is below DBL_MIN; the engine reads it as 0, which gives #NUM!. Literals of 1E-310 and below do not
  parse, so none are used.

**Engine.**
- The quantile collapses to the start value in rows 6, 7, 9 and 10, which all have p within about 1E-12 (relative) of
  DBL_MIN:
  - A6 = A7 = A9 = 2.2394449712624932E-20.
  - K6, K7, K9 and K10 = 0: the CDF at the returned "root" is 0, which is false convergence.
- Rows 2-5 (p = 1E-307 down to 2.5E-308) and row 11 (2^-1021) give ~3.7E-20 to 3.9E-20. For example, A4 (3E-308) =
  3.752290055216077E-20.

**Falsified if** Excel's rows 6-10 continue smoothly from rows 2-5. That would mean Excel does not flush the residual
there.

## G23. BETA.INV round trip at large shapes (P1)

**Question.** Where does `BETA.INV(BETA.DIST(x,a,b,TRUE),a,b)` give back x exactly (crossing-like), and where does it
give an early AS 109 iterate? In t3443-basym-sweep A101/A102 Excel returns x, while the engine returns an iterate
6E-16 and 2E-15 away.

**Cells.**
- 123 rows: total shape a+b in {20, 100, 200, 500, 2000, 1E4, 1E5}, crossed with a/(a+b) in {.01, .05, .3, .5, .7,
  .95}, at x = mean and mean ± 2 sd. The last two rows are the audit cells.
- Columns: A x (a 15-digit formula), B a, C b, D forward CDF, E BETA.INV(D), F E-A, G BETAINV(D).

**Engine.**
- F = 0 in 106 of 123 rows.
- The 17 non-zero rows are where the shipped early-iterate rule fires. They include both audit rows: F122 =
  6.088879400678593E-16 and F123 = 2.1163626406917047E-15.

**Reading the results.** Excel's F = 0 pattern across shape size and ratio shows which forward-kernel region keeps
early iterates. Do not just fit a threshold between the 17 cells.

## G24. T.DIST.RT / T.DIST.2T at df 999, 1000 and 1001 for 0 < t < 1 (P1)

**Question.** Is the finite integer-series domain inclusive at df 1000, so the stable central fraction starts at
df 1001? The guard `t < 1.0 && n >= SERIES_MAX_DF` switches one df early.

**Cells.**
- Per df 999, 1000 and 1001: T.DIST.RT, T.DIST.2T and T.DIST(TRUE).
- Rows: t = .05..95 in steps of .05, then .8513, 1-2^-53 and 1E-8.

**Engine.** In the prototype the bug is still present:
- D10 `T.DIST.RT(.5,1000)` = 0.3085925404169374. Excel (m9-t-dist-rt A123) is 0.30859254041693746.
- D20 `T.DIST.RT(.8513,1000)` = 0.19740326475142173. Excel (t0969 B1553) is 0.19740326475142156.

**Falsified if** Excel's df-1000 column agrees with the engine anywhere the df-999 path and the central fraction
differ. The expectation is that Excel matches the series path at df 1000 and the central fraction at df 1001.

---

## M1 to M6. Merged ranges whose covered cells hold content (the covered-cells spec (excel-parity-findings-2026-11-07.md section 1) UNVERIFIED rows)

**Format.** The case format's `sheet_ops` has no merge, unmerge, clear, paste, autofill or navigation op. These are
therefore operation scripts (`merged/merged-ops.procedure.json`) run on a start workbook.
- Office JS can do all of them: `unmerge`, `clear`, `copyFrom`, `moveTo`, `autoFill` and `getRangeEdge`.
- Graph REST covers merge, unmerge and clear. As far as I know it has no copy, autofill or edge equivalents, but check
  that before relying on it.
- Desktop Excel is the fallback (M14 Phase 4 per decision 0039).
- A useful follow-up is to extend `sheet_ops` with `unmerge`, `clear_contents`, `copy_paste`, `cut_paste`, `autofill`
  and `range_edge` so these become cases.

**Start workbook** (`merged/build_merged_fixture.py`; covered content is stored in the file, as in
fixtures/merged-covered-values):
- Sheet `M`: B2:C3 merged, with B2 = 5, covered C2 = "x", covered B3 = `=B2+1` (cached 6), C3 empty.
- Sheet `F`: B2 = 1, B3 = 2, C2 = 10, C3 = 20, and merged B4:C5 with anchor 3, covered C4 = 100, covered B5 = 200.
- Sheet `N`:
  - Row 2: A2 = 1; B2:D2 merged with an empty anchor and covered C2 = "h"; F2 = 7.
  - Row 8: the same, but with anchor B8 = "a".
  - Column H: H1 = 1; H2:H4 merged with an empty anchor and covered H3 = "h"; H6 = 7.
- Sheet `W`: B2:C2 merged, B2 = 1, covered C2 = 2.

**Engine predictions:**

| Id | Pri | Operation | Engine outcome | Falsified if Excel… |
|---|---|---|---|---|
| U1 (M1) | P1 | Unmerge M!B2:C3 | C2 = "x" and B3 = `=B2+1` (6) show again; no merge | empties covered cells |
| U2 (M2) | P1 | Clear contents of M!B2:C3 | all four cells empty; the merge is kept | keeps C2 / B3 |
| U2b (M2) | P2 | Clear contents of address M!B2 only | B2 empty, C2 = "x" kept, B3 recalculates to 1 | expands to the merge or refuses |
| U3 (M3) | P1 | Copy B2:C3, paste at J10 | J10 = 5, K10 = "x", J11 = `=J10+1` (6); merges B2:C3 and J10:K11; source unchanged | pastes only the anchor |
| U3b (M3) | P3 | Copy B2:C3, paste at C3 (overlap) | Ok, but nothing changes (silent no-op) | refuses with an error or pastes |
| U4 (M3) | P1 | Cut B2:C3, paste at J10 | content and merge move to J10:K11; source empty | leaves covered content behind |
| U5 (M4) | P1 | AutoFill M!B2:C3 down to row 7 | B4/B6 = 5, C4/C6 = "x", B5 = `=B4+1`, B7 = `=B6+1`; merges B4:C5 and B6:C7 | leaves tiles' covered cells empty, or gives a progression |
| U5b (M4) | P2 | AutoFill M!B2:C3 right to column G | D2/F2 = 5, E2/G2 = "x", D3 = `=D2+1`, F3 = `=F2+1`; merges D2:E3 and F2:G3 | as U5 |
| U5c (M4) | P1 | AutoFill F!B2:C5 down to row 9 | B6:B8 = 4, 5, 6; B9 = 200; C6:C7 = 30, 40; C8 = 100; merge B8:C9 | puts covered values in the trend, or blanks them |
| U6 (M5) | P2 | Ctrl+arrow | A2→ F2; B2→ F2; F2← A2; A8→ B8 (B8:D8); F8← B8; H1↓ H6; H6↑ H1 | stops at B2:D2 or H2:H4 (covered "h" counted) |
| U7 (M6) | P3 | API write to covered W!C2, then to anchor B2 | C2 refused ("Cannot edit a cell that is part of a merged cell"); B2 = 9 accepted | accepts the C2 write or unmerges |

## What could not be determined

- **Alternative-model values.** No Excel was run. For the open boundaries (G01, G02, G12, G14) the predicted values are
  the engine's current path only. The alternative hypotheses (Halley on Q above .9999, a Halley start beyond unit
  x = .5, AS 109 truncated exponents) were not implemented, so a cell's value alone does not say which hypothesis Excel
  follows. The groups are designed so that the location of the first disagreement does.
- **Clipboard and autofill reach.** Whether excel-graph can perform copy, autofill and edge navigation at all is
  unknown. If it cannot, M3 to M5 need Office JS or desktop Excel.
- **G16** has no pass/fail prediction. It is research data.
