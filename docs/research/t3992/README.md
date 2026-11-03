# T-3992: excel-graph fill-to-fill bit stability on the R4 fit kernels

**Question.** Does Excel for the web return different bits when it recalculates the same input bytes again? Decision 0188
R4.4 forbids fitting below the oracle's reproducibility floor without a reproducibility check. The only fill-to-fill
evidence on file was T-2143's LN probe, which decision 0112 read as "the last bit of an FYL2X near-tie varies between
runs". This note measures that on the kernels the open last-digit fits target.

**Method.** Each case below was refilled twice on 2026-11-03 (round 1 22:16-22:18 UTC, round 2 22:18-22:20 UTC) with `rustsheet-oracle fill -b excel-graph
--force`, with the local oracle cache (`eval/.cache/oracle/excel-graph`) emptied before each round. Every refill was a
fresh upload and recalculation: each sidecar carries a new `produced_at` and a new uploaded `workbook_name`, with the same
`input_sha256` as the committed fill. After each fill its sidecar was copied aside and the committed one restored, so no
sidecar under `eval/` changes. The committed fill is the first of the three, and it dates from 2026-10-22 to 2026-11-03 (column
"committed fill"). [`compare_fills.py`](compare_fills.py) compares every cell's stored value across the fills.
[`diffs.json`](diffs.json) holds, per case, the fill timestamps and every cell whose value differs in any fill, with all
values and their spread in ulps. The copied refill sidecars were not committed: they would be about 1 MB per round, and
`diffs.json` already holds every value that differed.

## Results

| kernel family | case | committed fill (UTC) | fills | cells | cells whose bits differ | largest spread |
|---|---|---|---|---|---|---|
| Student t quantile | probes-numeric/t3349-t-inv-2t-sweep | 2026-10-31 06:14 | 3 | 300 | 0 | 0 |
| IRR | probes-numeric/t3478-irr-r4-wide | 2026-10-31 23:27 | 3 | 300 (231 numbers) | 0 | 0 |
| RATE | probes/t3850-rate-same-state | 2026-11-03 15:30 | 3 | 240 (235 numbers) | 0 | 0 |
| CUMPRINC/CUMIPMT | probes/fin-cumipmt-tiny-rate-structure | 2026-10-29 06:17 | 3 | 226 | 0 | 0 |
| gamma quantile | probes-numeric/t3311-gamma-inv-lower-tail-cdf | 2026-10-31 07:47 | 3 | 899 | 0 | 0 |
| lnGamma/GAMMA | probes-numeric/t3570-r5-bpser-lg-aux | 2026-11-01 20:09 | 3 | 2,520 | 0 | 0 |
| incomplete beta, bfrac | probes-numeric/t3953-f-tail-bfrac-tiny | 2026-11-03 15:43 | 3 | 4,800 | 0 | 0 |
| incomplete beta, bpser | probes-numeric/t2464-r2-bpser-small-gl | 2026-10-23 12:36 | 3 | 5,843 | 0 | 0 |
| LN (control) | probes-numeric/t2143-ln-near-ties-fill1 | 2026-10-22 12:57 | 3 | 380 | 0 | 0 |
| LN (control) | probes-numeric/t2143-ln-near-ties-fill2 | 2026-10-22 12:58 | 2 | 380 | 0 | 0 |
| LN (control) | probes-numeric/t2143-ln-near-ties-fill3 | 2026-10-22 12:58 | 2 | 380 | **23** | 1 ulp |

Across the eight fit-target kernels, 15,128 cells were compared over three fills each, and not one bit differed. The two
fills of a case ran two minutes apart; the committed fill ran between about 6.5 hours and 12 days earlier.

**The control.** On 2026-10-22, T-2143 filled three workbooks that differ only in a text label in D1 (added so that the oracle
cache would not return fill 1), each with the same 380 LN formulas. Fill 3 differed from fills 1 and 2 at 23 near-tie cells
(18 of the 30 FYL2X near-ties in B1:B30, and B172, B187, B205, B206 and B217 among the 300 new near-ties), each by one ulp. Fills 1 and 2 agree on every cell. Refilling all three
workbooks today gives fill 1's bits every time: fill 1's bytes twice and fill 2's and fill 3's bytes once each, with no
difference at any cell. So fill 3's 2026-10-22 bits were one deviating *run*, not a property of its bytes. Today the 23
cells come back as fills 1 and 2 had them. On this probe, then, the variation seen on 2026-10-22 belonged to that run: the
same bytes did not vary in four later runs. That run changed 23 cells, all inside decision 0112's window (d 2.19e-4 to
3.86e-4 from a midpoint) and all by one ulp. It did not change every point in the window. Of the window's 69 points, 34
returned the same bits in all fills, and 12 of the 30 old near-ties were unchanged.

**Other fill-to-fill variations on file** (not refilled here):
- LN: probes/ln-fyl2x-kernel (2026-10-20) and the T-1941 research run (2026-10-19) differ at 3 of the 30 near-ties and agree
  on the other 250 cells. That is a partial difference between two runs ("LN is FLDLN2 then FYL2X" in numbers.md).
- ERFC/EXP: the T-1916 re-fill changed 9 of 20 ERFC near-tie cells (T-1940's Context).
- AMORDEGRC: gen-m9 d912671baa1364aa Calc!A76 is 1 in one fill and 0 in two later fills of the same input sha256 (T-3610).

**What this shows and what it does not.**
- On the eight probes measured, three runs of the same bytes returned the same bits at every cell. On LN, four runs today
  of T-2143's bytes agreed with each other and with fills 1 and 2 of 2026-10-22.
- No run today deviated, so this session had no positive control. Two runs a few minutes apart say little about how
  often deviating runs occur. The variations on file are rare, change some near-tie cells and not others, and are each one
  ulp (AMORDEGRC's is a step in an integer result). Nothing here shows whether the measured probes contain cells close
  enough to a rounding tie to vary.
- Different days: six cases have a committed fill from an earlier day that agrees with today's refills. RATE
  (t3850-rate-same-state, committed 2026-11-03 15:30) and incomplete beta bfrac (t3953-f-tail-bfrac-tiny, committed
  2026-11-03 15:43) have all three runs on 2026-11-03, so they do not yet have two days that agree.
- None of this compares web against desktop. R4.4's 4-ulp web-against-desktop floor (oracle-calibration.md) is a separate
  question (T-1075, M14).

## Recommendation for T-3610 (input, not a decision)

- One possible criterion: a cell is a fitting target once at least two excel-graph runs on different days agree on its
  bits. Six of the measured probes meet it for every cell: t3349, t3478, fin-cumipmt-tiny-rate-structure, t3311, t3570
  and t2464. RATE's t3850 and bfrac's t3953 need a refill on a later day.
- Where a refill differs, the cells that differ are the ones to drop (decision 0112). The LN control suggests refilling
  once more: there, the deviating run was alone, and later runs agreed with the earlier majority. One case is too little
  to make that a rule.
- This covers only the probes and dates measured. It does not cover the cells of "family only" tasks, which were not
  refilled, and it does not settle R4.4's web-against-desktop floor, which still needs a desktop check or an owner ruling.

## Open tasks that cite R4.4 or the reproducibility floor

"Measured" means the task's own probe was refilled. "Family only" means a probe of the same kernel family was refilled but
the task's own cells were not. The unstable count applies only to a task's own probe. Every one of these tasks got a
one-line note under its Context that links this page.

| task | title | measured | own cells fill-unstable |
|---|---|---|---|
| T-2647 | F.DIST pdf at a tiny x: Excel's last digits where Y = density·x is subnormal (probes-numer | no | - |
| T-3415 | T.INV.2T, TINV and CONFIDENCE.T centre at p near 1, df 2 up: Excel is 1E-15 to 4E-15 off t | yes, own probe `t3349-t-inv-2t-sweep` | 0 of 300 |
| T-3421 | T.INV.2T, TINV, T.INV and CONFIDENCE.T at a tiny central probability, df 2 up: Excel's cen | family only (`t3349-t-inv-2t-sweep`) | not refilled |
| T-3434 | COMPLEXs.xlsx IMARGUMENT H2, H3, H10: Mac-cached ATAN2 digits the web does not give; name  | no | - |
| T-3435 | BETA.DIST cdf at shapes 2.5 to 100 on the BETA.INV round trip: 13 cells where Excel is 4 t | family only (`t2464-r2-bpser-small-gl`) | not refilled |
| T-3474 | F.DIST and F.DIST.RT last digits on the t2220 d1-1, d2-1 and rt sweeps: 111 visibly-off ce | family only (`t3953-f-tail-bfrac-tiny`) | not refilled |
| T-3489 | PMT 1 to 4 ulp low in magnitude at rates 1.2E-16 to 2.1E-16 (pmt-tiny-rate row 2, pmt-tiny | no | - |
| T-3527 | F.DIST, F.DIST.RT and BETA.DIST at a tiny x or a huge second df or shape: the bit-level re | family only (`t3953-f-tail-bfrac-tiny`) | not refilled |
| T-3528 | BETA.DIST density at b < 1 < 8 < a: the bit-level residue on probe t2512-r3-bgrat-bupdens  | no | - |
| T-3538 | BETA.DIST cdf last digits on the m9-beta-dist a = 0.5, 5 and 10 grids, and NEGBINOM.DIST(5 | family only (`t2464-r2-bpser-small-gl`) | not refilled |
| T-3562 | BETA.INV on BETADIST_BETAINV H5 and H26: Excel's landing point off its own crossing (H26 a | no | - |
| T-3563 | BETA.DIST density at min < 1 < 8 < max: the 132 visibly-off t3383-r4-algdiv cells, from Ex | no | - |
| T-3564 | BETA.DIST density at min < 1 < 8 < max: the 4,164 bit-level t3383-r4-algdiv cells within 1 | no | - |
| T-3565 | CHISQ.INV.RT deep right tail: the 419 T-2561 edge-probe values 1 to 4 ulp off Excel (Excel | family only (`t3311-gamma-inv-lower-tail-cdf`) | not refilled |
| T-3591 | RATE in the noise floor on T-3348's fresh sweep: 4 digit cells beyond 15 digits, inside Ga | family only (`t3850-rate-same-state`) | not refilled |
| T-3592 | Owners for 76 unowned last-digit stat cells inside Gate A: BETA.DIST (39), F.DIST.RT (36)  | family only (`t2464-r2-bpser-small-gl`) | not refilled |
| T-3595 | F.DIST density past M-final's fit set: the 48 visibly-off t3478-f-density-r4-wide cells (m | no | - |
| T-3597 | BETA.DIST and F.DIST at basym points: Excel's arithmetic inside basym (the t3443 probes' v | family only (`t3953-f-tail-bfrac-tiny`) | not refilled |
| T-3607 | RATE last digits on T-3348's training sweep: 5 t3348-rate-gate-a-sweep cells 1 to 3 ulp fr | family only (`t3850-rate-same-state`) | not refilled |
| T-3618 | CHISQ.DIST.RT, CHIDIST and CHISQ.DIST cdf last digits on m9-chisq-dist-rt, m9-legacy-chidi | no | - |
| T-3619 | GAMMA.DIST density on t2753-gamma-density-switch and m9-gamma-dist-a1 A28: 64 cells 1 to 7 | no | - |
| T-3620 | YIELD one-step bonds: yield-one-step-start A162, B19 and B24 are 1 to 4 ulp off Excel afte | no | - |
| T-3640 | Excel's far-tail Q above shape 11 and past x = 600: the 37 t3269-fresh-sweep cells T-3269' | no | - |
| T-3659 | GAMMA.INV and CHISQ.INV deep lower tail outside T-3311's cut: which Newton iterate or sear | yes, own probe `t3311-gamma-inv-lower-tail-cdf` | 0 of 899 |
| T-3681 | POISSON cdf deep upper tail at shapes 2 to 11: the 13 gated gen-m9 cells T-3269 left 1 to  | no | - |
| T-3696 | BETA.INV at a shape of 1: Excel's last bit where the closed form is 1-3 ulp off (T-3664 re | no | - |
| T-3735 | BETA.DIST cdf with a shape within 16 ulp of 1 (t2464 w2f unit-shape): 127 bit-level cells  | family only (`t2464-r2-bpser-small-gl`) | not refilled |
| T-3740 | BINOM.DIST.RANGE past the mean: Excel's exact upper-tail form (21 probe rows within 6E-14) | no | - |
| T-3757 | Owners for 177 unowned last-digit cells of the F density r4-wide sweep (Phase 1b) | no | - |
| T-3767 | BETA.DIST density on the t3568 bup-terms probes: 7,026 unowned visibly-off cells at b from | no | - |
| T-3768 | BETA.DIST density on the t3568 bup-terms probes: the 4,281 bit-level cells within 15 digit | no | - |
| T-3772 | BETA.DIST density at shapes c+j against a0 > 15: owner for the 7,026 last-digit t3568 term | no | - |
| T-3774 | BINOM.DIST and NEGBINOM.DIST at a large n: Excel's unidentified |d| 2^-53 bd0 term and the | no | - |
| T-3776 | CUMPRINC at a tiny rate over short ranges, attempt 3 (decision 0056): solve for Excel's pe | yes, own probe `fin-cumipmt-tiny-rate-structure` | 0 of 226 |
| T-3779 | gen-m9 arg-pairs batch: oracle-filled cells for the unseen argument-class pairs of the M9  | no | - |
| T-3780 | gen-m9 arg-pairs batch: oracle-filled cells for the unseen argument-class pairs of the M9  | no | - |
| T-3782 | gen-m9 arg-pairs batch: oracle-filled cells for the unseen argument-class pairs of the M9  | no | - |
| T-3792 | BETA.DIST density on the t3568 bup-terms probes: Loader residue at a + b - 2 up to 3,000 ( | no | - |
| T-3803 | BETA.DIST density on the t3568-r5-bup-terms probes: owners for the 11,307 failing density  | no | - |
| T-3804 | Gamma quantiles in T-3505's budget band (shapes 2.9E5 to 1E8): the last ulps of 292 cells  | family only (`t3311-gamma-inv-lower-tail-cdf`) | not refilled |
| T-3813 | Incomplete beta: bfrac's last unmodelled rounding per term past 10 terms | family only (`t3953-f-tail-bfrac-tiny`) | not refilled |
| T-3818 | BETA.DIST density at one large shape on the T-3568 bup-terms probes: 7,026 unowned visibly | no | - |
| T-3886 | Incomplete beta bpser: T-3192's remaining items (b0 >= 8 L diff, T-2505's 987 moved cells, | family only (`t2464-r2-bpser-small-gl`) | not refilled |
| T-3933 | BETA.DIST cdf on bratio's direct bgrat branch at 0 < a0 < 1, a0 != 1/2: take bgrat_increme | family only (`t2464-r2-bpser-small-gl`) | not refilled |
| T-3935 | BETA.DIST density with both shapes large past n = 256: the 1,168 unowned t3571-r5-bfrac-fr | no | - |
| T-3936 | BETA.DIST cdf on the bfrac Lentz region once the density matches: the 1,157 unowned t3571- | family only (`t3953-f-tail-bfrac-tiny`) | not refilled |
| T-3937 | GAMMALN, GAMMA and LN(GAMMA) last digits on T-3570's lg-aux and lngamma-gl probes once T-2 | yes, own probe `t3570-r5-bpser-lg-aux` | 0 of 2520 |
| T-3938 | CUMPRINC one-period last digits on T-3552's evidence probes (387 gated probes cells, Phase | yes, own probe `fin-cumipmt-tiny-rate-structure` | 0 of 226 |
| T-3939 | GAMMA.DIST and GAMMA.INV last digits on the fit-audit and t3316 probes (58 cells within 15 | family only (`t3311-gamma-inv-lower-tail-cdf`) | not refilled |
| T-3940 | BETA.DIST at a = b = 1/2: the 61 one-ulp cells of T-3192's fresh sweep t3192-beta-half-hal | family only (`t2464-r2-bpser-small-gl`) | not refilled |
| T-3941 | RATE one ulp off at t3698-rate-r4-fresh A233 (nper 795, guess 0.124; Phase 1b) | family only (`t3850-rate-same-state`) | not refilled |
| T-3952 | F.DIST density at x = 123456789.123, df (8, 8): Excel 2.2E-15 and ours 6.6E-15 off the mat | no | - |
| T-3954 | FDIST(1E+15,4490,6) on bfrac at x0 = 1.3E-18: Excel 1.2E-14 off the math, ours 4.6E-15 (ge | yes, own probe `t3953-f-tail-bfrac-tiny` | 0 of 4800 |
| T-3956 | bpser last digits T-3879 left to GAMMALN: 1,093 unowned cells of t3879-bpser-split-fresh,  | family only (`t3570-r5-bpser-lg-aux`) | not refilled |
| T-3959 | BINOM.DIST and NEGBINOM.DIST cdf at a large n: Excel's cdf arithmetic on T-3774's never-at | no | - |
| T-3961 | BINOM.DIST and NEGBINOM.DIST cdf at p below 0.01 and a huge n: Excel is 269 to 64,789 ulp  | no | - |
| T-3963 | RATE flat secants: model Excel's noise-floor iterates, since no rule on the model's flat-s | yes, own probe `t3850-rate-same-state` | 0 of 240 |
| T-3982 | IMSECH's two T-3930 leftover texts (t3721 C386 complex, C390 real): find Excel's cosh wher | no | - |
