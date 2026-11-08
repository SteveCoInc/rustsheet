# Desktop Excel probe requests for the remaining Gate B misses (2026-11-08)

Status: research hand-off, **docs only**. This is a ranked request for Excel values, not Excel evidence. It targets the six
functions still below the Gate B rate (NORM.S.INV, NORM.INV, LOGINV, F.INV, HYPGEOMDIST, FVSCHEDULE) and the error-only
classifier IRR keeps after addendum 3. **Desktop Excel is the oracle wanted.** Excel for the web (`excel-graph`) is
acceptable for every probe except FVS-01's live and seeded steps and the isolated variants of FVS-02 (desktop only), and
FVS-01's web carrier is web-only. Whichever is used, run CAL-01 in the same build first. **Capture:** recalculate fully,
save as `.xlsx`, and read every cached value from the file at 17 significant digits or as raw binary64 bits, errors as
error codes. Values copied from the grid, `TEXT()`, `.Text` or a 15-digit CSV are not usable. Record build, bitness and OS.

## Ranked list

| ID | function | priority | cells | what it decides |
|---|---|---|---:|---|
| CAL-01 | all | P0 | 114 | Desktop versus web on cached anchors; repeatability; that low-bit residuals survive capture |
| FVS-01 | FVSCHEDULE | P0 | 44 per variant | Whether the overflowing call is a live zero, a lost non-finite value, a stale value or an error |
| HYP-01 | HYPGEOMDIST | P0 | 1,188 | Whether the conditional-binomial identity with Excel's own BINOM.DIST factors holds at non-dyadic p and large N |
| NSI-01 | NORM.S.INV, NORM.INV, LOGINV | P0 | 575 | That NORM.INV(p,0,1/SQRT(2)) exposes the inverse-erf double; the leading constant; LOGINV's association |
| NSI-02 | NORM.S.INV, NORM.INV | P0 | 414 | Where the central noise reset is, and which predicate selects it |
| IRR-01 | IRR (NPV) | P0 | 554 | Whether the error kind is invariant under 2^k scaling, negation and trailing zeros; the two overflow rules |
| FIN-01 | F.INV | P0 | 754 | Repeatability of the target; plateau versus re-hash response to p; BETA.INV twins |
| FIN-02 | F.INV (BETA.DIST, BINOM.DIST) | P0 | 765 | Which derivative-exponent storage the search uses, from worksheet kernels at the exact iterates |
| HYP-02 | HYPGEOMDIST | P0 | 1,200 | What Excel computes when a binomial factor underflows |
| FVS-02 | FVSCHEDULE | P0 | 357 | Overflow cut, retained exponent range, zero/unit factors, underflow |
| NSI-03 | NORM.S.INV, NORM.INV | P1 | 585 | Adjacent-input jump spectra of the central kernel (pilot) |
| FIN-03 | F.INV | P1 | 783 | Iteration cap behind `#NUM!`, the flush-stop class boundary, one-step searches |
| IRR-02 | IRR | P1 | 820 | The seed of the attempt that decides the error kind, read in Excel's own arithmetic |
| HYP-03 | HYPGEOMDIST (BINOM.DIST) | P1 | 595 | Endpoint, plateau, mode-normalization and bd0-cut behaviour of the factor kernel |
| FVS-03 | FVSCHEDULE | P1 | 193 | Plain versus twice-rounded product in range |

Run order if time is short: CAL-01, FVS-01, HYP-01, NSI-01, NSI-02, IRR-01. Every P0 sheet is self-contained.

Conventions. Each probe has a draft case `probe-cases-2026-11-08/val2-<id>.case.json` (lowercase id, for example `val2-cal-01`) (`rustsheet-case/v1`, sheet `P`
unless stated, no oracle values). Hypotheses are labelled H1, H2, ... per function. Every number in a "model" column is a
**prediction from a model of the arithmetic, not an Excel value**; every number called "cached" is an existing Excel for
the web value from the corpus. "ulp" is one unit in the last place of the cell's own value. Exact doubles are built from
integer limbs and powers of two, never from long decimal literals. Rows listed below are the case's rows.

## Shared: CAL-01 (P0, all functions)

**Question.** The cached corpus is Excel for the web. Does the desktop build reproduce it, is it repeatable, and does the
capture keep low bits?

**Layout** (sheet `P`).
- A1:E8 canary. Row 1 headers; A2:A8 = -3..3; row 2 then fill down to row 8:
  `B2 =1+A2*2^-52`, `C2 =SUM(B2,-1)*2^52`, `D2 =(B2-1)*2^52`, `E2 =SUM(1,-B2)*2^52`.
- G1:H37 anchors, one per row: label in G, formula in H. Cached web values:

| H | formula | cached (web) | H | formula | cached (web) |
|---|---|---|---|---|---|
| 2 | `=F.INV(1E-300,9,3)` | 5.814110643623313e-68 | 20 | `=IRR({0.3,-2.5,1.005})` | 6.909808543634389 |
| 3 | `=F.INV(1E-300,8,3)` | 2.994029821162069e-76 | 21 | `=IRR({-0.3,2.5,-1.005})` | 6.909808543634354 |
| 4 | `=F.INV(1E-300,11,3)` | #NUM! | 22 | `=IRR({42,1,1,-1})` | -0.7467044766537371 |
| 5 | `=F.INV(1E-250,11,3)` | #NUM! | 23 | `=IRR({-500,1,1,1,1,1})` | #NUM! |
| 6 | `=F.INV(1E-300,9,2)` | 4.787632644515261e-68 | 24 | `=IRR({132632163387.293,-132632163322.948})` | -4.851388180071581e-10 |
| 7 | `=F.INV(1E-300,9.9,3.9)=F.INV(1E-300,9,3)` | (TRUE expected: df truncation) | 25 | `=IRR({-1,1E+15},0.05)` | #DIV/0! |
| 8 | `=NORM.S.INV(0.5+0.706/2)` | 1.0493870846841074 | 26 | `=IRR({-1,1E+7})` | #NUM! |
| 9 | `=NORM.S.INV(0.5+0.71/2)` | 1.058121617684777 | 27 | `=IRR({-218328.5,` 60 ones `})` | #NUM! |
| 10 | `=NORM.S.INV(0.5+1*2^-30)` | 2.3344794983332983e-09 | 28 | `=IRR({-823935.9,` 60 ones `})` | #DIV/0! |
| 11 | `=NORM.S.INV(0.5+20*2^-12)` | 0.012239701470896465 | 29 | `=FVSCHEDULE((-1E+300),{100,200,300})` | -6.110601e+306 |
| 12 | `=HYPGEOMDIST(2,4,3,19)` | 0.09287925696594419 | 30 | `=FVSCHEDULE(0,{-1E+300,1E+300})` | 0 |
| 13 | `=HYPGEOM.DIST(2,4,3,19,FALSE)` | 0.09287925696594419 | 31 | `=LN(0.612354100000274)` | known run-to-run near-tie |
| 14 | `=HYPGEOMDIST(0,27,0,35)` | 0.9999999999999999 | 32 | `=2^-1022*2^1022=1` | TRUE |
| 15 | `=HYPGEOMDIST(3,4,7,15)` | 0.20512820512820512 | 33 | `=2^-997*2^997=1` | TRUE |
| 16 | `=HYPGEOMDIST(41966,46221,65797,74726)` | 0 | 34 | `=INFO("release")` | record |
| 17 | `=HYPGEOMDIST(1,2,3,1000000)` | 5.999987999988011e-06 | 35 | `=INFO("osversion")` | record |
| 18 | `=BINOM.DIST(1,2,0.5,FALSE)` | 0.49999999999999994 | 36 | `=INFO("system")` | record |
| 19 | `=BINOM.DIST(3,10,0.3,FALSE)` | 0.26682793200000005 | 37 | `=BINOM.DIST(2,10,0.3,FALSE)` | 0.23347444050000005 |

Sources: t4974 G09, G16, G20 (excel-graph), HYPGEOMDIST/BINOM.DIST Gate A anchors, IRR cells of
probes-numeric/t2927, acc/IRR/r4-T-3411, irr-guess-edges, irr-fallback-roots, probes/t3406-irr-num-boundary A80/A81,
gen-m9/67a4c03afd63f988 A318 and gen-m9/81bcdc0bec0c8e6b A276.

**Hypotheses.** H1: desktop equals web bit for bit. H2: a stable backend difference (for example operations rounded once
instead of the twice-rounded x87 style the IRR kernel needs; a model of that reading changes 45 of 2,734 cached IRR numbers
and 0 of 882 error kinds, and is most visible in update-200 returns such as H24). H3: run-to-run variation.

**Protocol and thresholds.** Three full recalculations (Ctrl+Alt+F9), one after a cold reopen; keep each saved file.
Canary: C must equal A and E must equal -A exactly; if not, the in-sheet `SUM(...)*2^52` residuals used by HYP and
NSI probes are unusable and raw bits are required. Any anchor that differs from its cached value under H2 means desktop
values must not be pooled with cached web cells in one fit; the later sheets are then read only against themselves.
H31 is expected to vary at one ulp (FYL2X near-tie); use it only as the noise calibration.

---

## NORM.S.INV, NORM.INV, LOGINV

**Open question.** NORM.S.INV (5 cells), NORM.INV (4) and LOGINV (1) remain below rate. NORMINV passes. Every central
Excel output is fl(sqrt(2)*u) for some double u, and outputs mirror exactly for bit-identical |p - 1/2| (addendum 2,
"Still open"). The central inverse-erf approximation that produces u is unidentified: published AS 241, Blair, Cephes,
Acklam, Boost and Giles variants, and converged inversion of the public ERF, have been excluded at the observed error
level. The remaining misses are about 22 ulp worst.

**Hypotheses.**
- H1: a private, cancellation-prone rational inverse-erf in stored binary64 Horner steps, then a stored sqrt(2) multiply.
- H2: the same rational evaluated with extended-precision intermediates (no 53-bit product-rounding jumps).
- H3: converged inversion of an accurate ERF (control; already inconsistent with tens-of-ulp errors).
- H4: the central kernel switches pieces at x = |2(p - 1/2)| <= fl(sqrt(0.5)) (direct predicate).
- H5: it switches at fl(x*x) <= 0.5 (squared predicate); at x = fl(sqrt(0.5)) itself H4 and H5 pick different branches.
- H6: no branch at sqrt(0.5); only a binade change of x*x at 0.5 alters the rounding spectrum.
- H7 (wrapper): NORM.INV computes mean + (sd*sqrt(2))*u, so NORM.INV(p,0,1/SQRT(2)) returns u exactly; LOGNORM.INV and
  LOGINV compute EXP(mean + sd*z) with z the stored standard quantile (addendum 2, section F).
- H8 (wrapper): the wrapper rounds z first and then scales, so NORM.INV(p,0,1/SQRT(2)) is not u.

**What t4974 G16 already answers.** Recomputing G16's cached values (excel-graph) against a 60-digit inverse erf of the
exact binary64 p gives a sharp change: the 30 cells with x in [0.700, 0.70700] have |z error| up to 16.1 ulp (median 6.6;
3 of 30 within 1.5 ulp), and the 32 cells with x in [0.70750, 0.715] stay within 3.6 ulp (median 0.6). So the reset lies in
(0.70700, 0.70750], much tighter than the bracket (0.70671, 0.70994) from cached LOGNORM.INV callers. The earlier proposal of
a 0.7065..0.7103 reset map is therefore dropped; NSI-02 maps only the remaining bracket. G16's dyadic low-x columns
(p = .5 + k*2^-30 and .5 + k*2^-12) also already cover odd symmetry at small x, so no new symmetry sweep is requested.

### NSI-01 (P0): wrapper exposure, leading constant, LOGINV association

**Layout.**
- **Block A, A1:L13** (row 1 headers). Paste row 2 and fill B:L down to row 13; then enter column A:
  ```
  A2 =(0*67108864+1)/2^1    B2 =(1-A2)/2    C2 =2*(0.5-B2)    D2 =-NORM.INV(B2,0,1/SQRT(2))
  E2 =-NORMINV(B2,0,1/SQRT(2))    F2 =-NORM.S.INV(B2)    G2 =-NORM.INV(B2,0,1)
  H2 =-NORM.INV(B2,0,2^20/SQRT(2))/2^20    I2 =-NORM.INV(B2,0,2^-20/SQRT(2))*2^20
  J2 =GAMMA.INV(A2,0.5,1)    K2 =SQRT(J2)    L2 =D2*D2
  A3:A13 =(H*67108864+L)/2^E with (H,L,E) = (80530636,53687091,53) (0,5,3) (42863322,64372217,52)
         (90191589,16682567,53) (22907258,9464255,51) (46976204,53687091,52) (47256970,8820197,52)
         (94906265,41892813,53) (11911823,24159191,50) (0,3,2) (112742891,34896609,53)
  ```
  Plus `B14 =(1/SQRT(2))*SQRT(2)` and `B15 =SQRT(0.5)*SQRT(2)` (labels in A14:A15).
- **Block B, A17:E49** (row 17 headers). Row r = 18..49: `A =n/2^e`, `B =(1-Ar)/2`, `C =-NORM.INV(Br,0,1/SQRT(2))`,
  `D =Ar*((118947364*67108864+28634987)/2^53)`, `E =Ar*((59473682*67108864+14317493)/2^52)`, with (n,e) for each base
  e0 = 45, 40, 35, 30: (1,e0) then (2^(53-e0)+1, 53), (2^(52-e0)+1, 52), (2^(53-e0)+3, 53), (2^(51-e0)+1, 51),
  (2^(53-e0)+5, 53), (2^(52-e0)+3, 52), (2^(53-e0)+7, 53). Columns D and E are worksheet replays of the two candidate
  multipliers r0 = 0.886226925452758 (0x3fec5bf891b4ef6b) and its predecessor r- (0x3fec5bf891b4ef6a).
- **Block C, A52:J76** (row 52 headers): 24 consecutive p doubles around fl(0.2). Row r = 53..76: `A =(H*67108864+L)/2^E`,
  `B =2*(0.5-Ar)`, `C =NORM.S.INV(Ar)`, `D =NORM.INV(Ar,0,1/SQRT(2))`, `E =LOGINV(Ar,50,30)`, `F =LOGNORM.INV(Ar,50,30)`,
  `G =NORM.INV(Ar,50,30)`, `H =LOGINV(Ar,0,32)`, `I =EXP(50+30*Cr)` (replay of H7's LOGINV association),
  `J =EXP(50+(30*SQRT(2))*Dr)` (replay of the NORM.INV association). (H,L,E) per row are in the case; consecutive rows
  step p by one ulp (the limb exponents differ only because each fraction is reduced). Rows with equal B are intended.

**Predictions and thresholds.**

| check | H7 | H8 / other |
|---|---|---|
| Block A: D = E = H = I | bit for bit | D differs from H or I by >= 1 ulp |
| F = G = fl(sqrt(2)*D) (offline, exact) | every row | fails in some row |
| J = L (half-shape GAMMA.INV shares u) | if the gamma path shares the kernel | J != L: separate kernel |
| B14 | 1 exactly (model: fl(fl(1/fl(sqrt 2))*fl(sqrt 2)) = 1) | B15 = 1.0000000000000002 (model): SQRT(0.5) is not a cancelling scale |
| Block B: C = D in every row | multiplier r0 | C = E in every row: r-; neither: another association or a raw-x approximation |
| Block C: E = F; equal B give equal C:H | shared central kernel | E != F: legacy and modern lognormal differ |
| Block C: E vs I | within 2 ulp (EXP and affine rounding only) | E vs J within 2 ulp instead: NORM.INV association |

One cached ulp is decisive for every equality (Excel is deterministic once CAL-01 passes), but invisible in a 15-digit
export. In block C a 1-ulp change of z moves E by about 25 output ulps (sd = 30), so a disagreement above 5 ulp between E
and its replay is an association or kernel difference, not EXP rounding. Column H (sd = 32) has an exact scaling and
isolates EXP. Block B does not identify the rational's P0 and Q0 separately.

### NSI-02 (P0): the reset inside (0.70700, 0.70750]

**Layout.** Columns for every data row r: `A x`, `B =(1-Ar)/2`, `C =-NORM.INV(Br,0,1/SQRT(2))`, `D =-NORM.S.INV(Br)`,
`E =ERF.PRECISE(Cr)`, `F =Ar*Ar`. Headers in rows 1 and 38.
- Rows 2-36, exact x = k/2^16 for k = 46333..46367 (0.7069855 to 0.7075043): `A2 =46333/2^16`, `A3 =A2+2^-16`, fill to A36.
- Rows 39-70, 32 consecutive doubles around fl(sqrt(0.5)): `A39 =(94906265*67108864+41892797)/2^53`, `A40 =A39+2^-53`,
  fill to A70. A55 is fl(sqrt(0.5)) = 0x3fe6a09e667f3bcd; A54 is its predecessor. No `SQRT(.5)` builds an input.
- p = (1-x)/2 is exact for every x here, so all x bits reach the function.

**Predictions and thresholds.** Classify each row by |z error| against the exact quantile of its p (offline, from the raw
C/D values; E is returned raw and subtracted offline, never in the sheet).

| outcome | reading |
|---|---|
| Rows 2-36: noisy (> 4 ulp typical) up to some k*, quiet (<= 3.6 ulp) after | the reset is at k*/2^16; H6 if it is not at sqrt(0.5) |
| Rows 39-70: noisy through A55, quiet from A56 | H4 (x <= fl(sqrt .5)) |
| Rows 39-70: noisy through A54, quiet from A55 | H5 (fl(x*x) <= .5; F55 > 0.5) |
| No change in rows 39-70 | the cut is elsewhere in the bracket (read it from rows 2-36) |

Decide on the envelope of at least eight consecutive rows each side, never on one row: in G16's noisy band 3 of 30 cells
were within 1.5 ulp by chance, so the classification of A55 alone carries about a 10 % error rate. Exclude the old 0.005
central grid from any coefficient fit.

### NSI-03 (P1): adjacent-input ladders in the noisy band

**Layout.** Three 64-row runs side by side, headers in row 1, data rows 2-65. Run b uses columns (A,B,C), (E,F,G), (I,J,K):
```
A2 =(41119938*67108864+7989739)/2^52     E2 =(44111950*67108864+32588731)/2^52     I2 =(47256970*67108864+8820197)/2^52
A3 =A2+2^-53 (fill to A65)                E3 =E2+2^-53                               I3 =I2+2^-53
B2 =-NORM.INV((1-A2)/2,0,1/SQRT(2))       F2 =-NORM.INV((1-E2)/2,0,1/SQRT(2))        J2 =-NORM.INV((1-I2)/2,0,1/SQRT(2))
C2 =-NORM.S.INV((1-A2)/2)                 G2 =-NORM.S.INV((1-E2)/2)                  K2 =-NORM.S.INV((1-I2)/2)
```
Starts are 0.6127348261931, 0.6573192847611 and 0.7041837294613; all lie below the reset.

**Predictions.** The true u grows by about 1.3 to 1.5 ulp per input ulp here, so a correctly rounded kernel gives first
differences of 1 or 2 ulp. H1 predicts multi-ulp jumps and occasional decreases (negative first differences); H2 predicts
no negative differences beyond 1 ulp; H3 predicts |error| <= 1 ulp everywhere (control). Stored 6/6 and 7/7 Horner trees,
even/odd evaluation and common-factor forms produce different jump sequences with nearly identical real functions; one
64-point run cannot identify a degree. These are pilots. If they separate H1 from H2 but not the structure, extend each
anchor contiguously (32 consecutive 64-point blocks) before asking for unrelated inputs.

**Existing cells to exclude (normal family).**
- calipers Statistical!B46 `NORMINV(B45,D41,E41)`: already passes; dominated by affine cancellation.
- gen-m9/8fec7b207f4dd753 Calc!A55 `LOGINV(0.2,...,30)` on its own: EXP and affine rounding blur the inner error; use
  NSI-01 block C instead.
- Nominal complement pairs whose |p - 1/2| are not bit-identical (decimal .2 and .8): they cannot test odd symmetry.
- accuracy/NORM.INV/s1 Calc!A126 `=NORM.INV(0.075,-23.8500052252671,25.888212855209)`: the x = .85 tail/affine conflict;
  keep it as evidence but out of central fitting.
- Any single quiet cell in the noisy band as evidence of a branch.

---

## F.INV

**Open question.** One cell: `F.INV(1E-300,9,3)`, cached 5.814110643623313e-68, the engine one LN bin (about 200 ulp,
2.8e-14 relative) higher. The search stops when the residual `cdf - p` becomes subnormal (a flush stop), so the result
is a raw AS 109 iterate whose final LN bin depends on the last steps' rounding. t4974 G09 showed the target is not
isolated (7 of 40 neighbours visibly off) and that `F.INV(1E-300,11,3)` and `F.INV(1E-250,11,3)` are `#NUM!` (T-4995).

**Hypotheses.**
- H1: the current profile (addendum section C): Newton derivative = worksheet beta density with the Loader exponent
  retained unrounded; stored a*LN(x) in direct bpser; compensated four-log back-transform.
- H2: the Loader exponent is accumulated in stored doubles in the order
  `s(n) - s(k) - bd0(k,np) - s(n-k) - bd0(n-k,nq)`, then `EXP(acc - lf/2 [+ LN(n+1)])`; H2a keeps lf extended, H2b stores
  it once. One investigation reports this order reproduces the exponent of 238/238 far-tail BINOM.DIST and 369/369 BETA.DIST
  cached point probabilities.
- H3: the stored scaled log-binomial return and a stored total derivative exponent before EXP. A second investigation
  reports that this makes both old target cells exact but loses protected siblings, and rejected it globally. H2 and H3
  are kept separate because the two reports disagree on whether stored-exponent storage is the mechanism.
- H4: the inverse's bpser keeps the internal LN to 64 bits before forming a*ln(x) (rejected globally as a fix; kept as a
  CDF contrast).
- H5: desktop and web differ stably, or a whole inverse varies between recalculations.
- H6a: adjacent p give piecewise-constant LN-bin plateaus; H6b: every p is an independent re-hash of the whole path.
- H7: the inverse calls the worksheet BETA.DIST arithmetic; otherwise a private caller.
- H8: F.INV is the back-transform of the very iterate BETA.INV returns for (df1/2, df2/2).
- H9: `#NUM!` at (11,3) is an iteration cap N; the model's step counts give 481 <= N < 512 (the two `#NUM!` cells take 512
  steps in the model, every numeric G09 cell at most 481). Alternative: a shape-specific failure.
- H10: the stop is an absolute residual flush (relative error of order DBL_MIN/p, shrinking as p grows); alternatives are
  relative convergence or the converged quantile.

### FIN-01 (P0): target repeatability and neighbourhood

**Layout.**
- **A1:L8** (row 1 headers), rows 2-8 = (df1,df2) (9,3) (9,1186) (9,8) (6,3) (7,3) (5,3) (10,5) in A:B; `L2` = 1E-300.
  Row r: `C =F.INV(1E-300,Ar,Br)`, `D` the same formula, `E =F.INV($L$2,Ar,Br)`, `F =F.INV.RT(1E-300,Br,Ar)`,
  `G =FINV(1E-300,Br,Ar)`, `H =(Cr/Dr-1)*2^40`, `I =(Cr/Er-1)*2^40`, `J =(Cr*Fr-1)*2^40`, `K =(Fr/Gr-1)*2^40`.
- **A11:J28** (row 11 headers), 17 consecutive p doubles; row 20 is fl(1E-300). Row r = 12..28: `A` = offset -8..8,
  `B =((89884656*67108864+49869657+Ar)*2^-52)*2^-997`, `C =Br*2^997`, `D =NORM.S.INV(Br)`, `E =F.INV(Br,9,3)`,
  `F =F.INV(Br,9,1186)`, `G =F.INV(Br,9,8)`, `H =F.INV.RT(Br,3,9)`, `I =LN(Er)`, `J =(Er/$E$20-1)*2^40`.
- **A31:F56** (row 31 headers), re-hash draws. Row r = 32..56: `A` = k = -12..12, `B =1E-300*(1+Ar*2^-20)`,
  `C =F.INV(Br,9,3)`, `D =BETA.INV(Br,4.5,1.5)`, `E =F.INV(Br,9,1186)`, `F =F.INV(Br,7,3)`. B's cached value defines the input.
- **A59:H99** (row 59 headers), BETA.INV twins of the 40 G09 cells: A = 1E-300 (rows 60-79) or 1E-250 (rows 80-99),
  B = df1 7..11, C = df2 2..5 (df2 varies fastest). `D =BETA.INV(Ar,Br/2,Cr/2)`, `E =BETA.DIST(Dr,Br/2,Cr/2,TRUE)/Ar`,
  `F =F.INV.RT(Ar,Cr,Br)`, `G =F.INV(Ar,Br,Cr)`, `H =(Dr/(1-Dr))*(Cr/Br)`.

Run five full recalculations, two after a cold reopen, and keep each. If possible repeat once with single-threaded
calculation.

**Model predictions (not Excel values)** for block 1, column C:

| df1,df2 | H1 value | H4 - H1 | H3 - H1 | cached (web) |
|---|---:|---:|---:|---|
| 9,3 | 5.8141106436234775e-68 | -200 ulp | -200 ulp | 5.814110643623313e-68 |
| 9,1186 | 1.1502976097070488e-67 | 0 | -198 ulp | 1.1502976097070162e-67 (gen-m9/caa1eed48b1883fe A237, df2 via VAR.P) |
| 9,8 | 8.2313827761952575e-68 | +142 ulp | +142 ulp | none |
| 6,3 | 3.8517135711308428e-101 | -172 ulp | +173 ulp | none |
| 7,3 | 6.4994118524223709e-87 | 0 | 0 | 6.499411852422371e-87 (G09) |
| 5,3 | 4.5137677567954673e-121 | 0 | +298 ulp | none |
| 10,5 | 3.0556733812626199e-61 | 0 | 0 | 3.05567338126262e-61 (G09) |

**Separation.** H1-H4 differ by whole LN bins (>= 140 ulp) wherever they differ; a contrast reproduced in all five runs
is decisive, a contrast of 1-4 ulp is not. H5: any nonzero H (duplicate) or a change between runs makes these cells
noise-limited; stop fitting them. Nonzero I means literal versus reference p matters. Block 2: H6a predicts a few plateau
edges across the 17 rows and the model contrasts (H4 - H1 from -200 to 0 ulp at (9,3); H3 - H1 from -198 to 0 at
(9,1186); +142 at (9,8)) move plateau edges, not single rows; H6b predicts uncorrelated rows. Record a moving edge as an
interval. Block 3: a model wrong by 1e-13 per step still matches each draw at its natural rate (model perturbation: 0.0-0.3
for (9,3), about 0.6 for (9,1186), about 0.8 for (7,3)), so the correct model must match all 75 F cells. Block 4: under H8
D lies in G's LN bin in every row (F.INV gives about 7 bits of the 53-bit iterate; BETA.INV gives all), and the (5.5,1.5)
rows say whether BETA.INV is `#NUM!` where F.INV is; E reads the stop residual.

### FIN-02 (P0): worksheet kernels at the exact final iterates

**Layout.** Row 1 headers; data rows 2-52, row r: `A` label, `B` x, `C =LN(Br)`, `D =4.5*Cr`,
`E =BETA.DIST(Br,4.5,1.5,TRUE)`, `F =BETA.DIST(Br,4.5,1.5,FALSE)`, `G =(Er/1E-300-1)*2^40`, `H =Br-(Er-1E-300)/Fr`,
`I =LN(Hr)`. Let x358 = ((78894216*67108864+64806208)*2^-52)*2^-222 = 1.7442400146889503e-67 (the model's iterate 358).
- Rows 2-15: the model's iterates 346-359, `B =(H*2^27+L)*2^-52*2^-E` with (H,L,E) = (49231138,11420598,219)
  (38291233,80226081,219) (59565820,59186356,220) (46333018,4685790,220) (36046540,111142328,220) (56119341,58667650,221)
  (43761179,38663803,221) (34305875,34736678,221) (54627840,46356535,222) (45293101,88899061,222) (40632223,124065554,222)
  (39506005,117544472,222) (39447108,64806208,222) (39446954,27908446,222). These are inputs, not claims about Excel's path.
- Rows 16-32: x358 + d for d = -47..-31, `B =((78894216*67108864+(64806208+d))*2^-52)*2^-222`.
- Rows 33-52: x358 + d for d = -160..-16 and +16..+160, step 16, same formula.
- **Rows 54-81** (row 54 headers): density rescaling. Row r: `A` anchor, `B` k = -4..4,
  `C =((H*67108864+L)*2^-52)*2^-(E0-k)`, `D =LN(Cr)`, `E =BETA.DIST(Cr,a,b,FALSE)`, `F =Er*2^N(k)`,
  `G =BETA.DIST(Cr,a,b,TRUE)`; anchors (H,L,E0; a,b) = primary (78894216,64806208,222; 4.5,1.5), compound
  (101076099,33250835,230; 4.5,593), sibling (83771161,10735829,223; 4.5,4). N(k) (in the case) brings F to order one.
- **Rows 83-183** (row 83 header): `A =BINOM.DIST(k,n,2^-j,FALSE)` for (k,n,j) =
  (1,2,30) (1,2,36) (1,2,42) (1,2,48) (1,3,96) (1,3,102) (1,3,108) (1,3,114) (1,6,54) (1,6,60) (1,6,66) (1,6,72) (2,3,30)
  (2,3,36) (2,3,42) (2,3,48) (2,4,192) (2,4,198) (2,4,204) (2,4,210) (2,6,54) (2,6,60) (2,6,66) (2,6,72) (2,10,54)
  (2,10,60) (2,10,66) (2,10,72) (3,4,36) (3,4,42) (3,4,48) (3,4,54) (3,5,30) (3,7,30) (3,7,36) (3,7,42) (3,7,48) (3,9,66)
  (3,9,72) (3,9,78) (3,9,84) (3,14,66) (3,14,72) (3,14,78) (3,14,84) (4,5,30) (4,5,36) (4,5,42) (4,5,48) (4,6,30) (4,6,36)
  (4,6,42) (4,6,54) (4,8,96) (4,8,102) (4,8,108) (4,8,114) (4,12,30) (4,12,36) (4,12,42) (4,12,48) (4,18,54) (4,18,60)
  (4,18,66) (4,18,72) (5,6,30) (5,6,36) (5,7,42) (5,7,48) (5,7,54) (5,7,60) (5,10,30) (5,10,36) (5,10,42) (5,10,48)
  (5,11,42) (5,11,48) (5,11,54) (5,11,60) (5,15,78) | controls: (1,2,54) (1,4,162) (2,3,264) (2,5,180) (2,6,300) (3,4,234)
  (3,6,42) (3,7,156) (3,14,204) (4,7,120) (4,9,138) (4,18,222) (5,8,66) (5,15,42) (6,8,42) (6,18,132) (7,10,114) (8,10,96)
  (9,10,36) (9,38,66). Rows 84-163 discriminate, rows 164-183 are controls.

**Predictions and thresholds.**
- CDF (E): H1, H2 and H3 share the direct-bpser CDF, so E is constant inside one LN bin (about 150 doubles) and jumps by
  about 1.28e-13 relative (about 570 CDF ulps) between bins. H4 moves the plateau edges; the model's edge is at d = -39 with a
  1.13687e-13 relative jump.
- Density (F) at rows 2-52: H1 and H2 differ by up to one unit of 2^-43 in the exponent (up to 1.1e-13 relative).
- Rescaling block at k = 0, model values: primary H1 2.5799655780425726e-233, H3 2.5799655780426173e-233 (+100 ulp);
  compound H1 5.1552518749575278e-231, H3 5.155251874957736e-231 (+182 ulp); sibling H1 4.8594623900995559e-233, H3
  4.8594623900996551e-233 (+111 ulp). A shape-normalizer error is constant in k; exponent storage gives a k-dependent signed
  pattern.
- H7: if H in the x358 row (row 14) lands in the LN bin of Excel's F.INV result (FIN-01 C2) while the model's next iterate
  does not, the inverse runs the worksheet kernels and the model's private CDF profile is the defect.
- Binomial block: production's stored order `fl(fl(S - bd0_p) - bd0_q)` and H2's order give exponents that differ by
  3.6e-15 to 5.7e-14 relative in rows 84-163 and agree in the controls. Cached far-tail cells already lean to H2 (27 cells
  fit only H2, 1 only production, 15 neither, of 295).
All contrasts here are >= 100 ulp except the density H1/H2 split; Excel's only known run-to-run noise (an LN near-tie) is
one ulp. Worksheet agreement alone does not prove the private caller shares the profile; combine with FIN-03 block 3.

### FIN-03 (P1): search length, stop class and one-step searches

**Layout.**
- **Rows 1-33** (row 1 headers): k = 150, 155, ..., 305 in A; B:G = `F.INV(1E-k,11,3)`, `BETA.INV(1E-k,5.5,1.5)`,
  `F.INV(1E-k,13,3)`, `F.INV(1E-k,12,3)`, `F.INV(1E-k,11,4)`, `F.INV.RT(1E-k,3,11)`, written as literals (for example
  `=F.INV(1E-150,11,3)`). Paste pattern if literals are inconvenient: `=F.INV(VALUE("1E-"&$A2),11,3)`, then confirm the
  k = 250 and 300 rows equal the literal G09 cells.
- **Rows 35-73** (row 35 headers): j = -1, 0..16, 18..56 step 2 in A; row r: `B =2^-1022*2^Ar`, `C =F.INV(Br,9,3)`,
  `D =BETA.INV(Br,4.5,1.5)`, `E =F.INV.RT(Br,3,9)`, `F =BETA.DIST(Dr,4.5,1.5,TRUE)/Br`, `G =F.INV(Br,17,50)`,
  `H =FINV(Br,7,9)`, `I =LN(Gr)-LN(Br)/8.5`. Row 36 (j = -1) is a subnormal input that may flush: read B before anything else.
- **Rows 75-81** (row 75 headers): six two-evaluation searches at p = `2.22507385850721E-308`, the literal embedded in each
  call; (df1,df2) = (33,12) (36,13) (60,21) (63,22) (77,27) (85,30). Columns C:U: echo, `-NORM.S.INV(p)`, the worksheet AS 109
  seed J (`s=1/(df1-1)`, `t=1/(df2-1)`, `h=2/(s+t)`, `r=(q*q-3)/6`, `w=q*SQRT(h+r)/h-(t-s)*(r+5/6-2/(3*h))`,
  `J=df1/(df1+df2*EXP(2*w))`), K = the model's exact start value (an input; limbs in the case), CDF and density at K, one
  worksheet Newton step N, `F.INV`, the beta coordinate from F, their gap, `BETA.DIST(N)/p`, `FINV`, `BETA.INV` and
  `(J/K-1)*2^40`.
- **Rows 83-91** (row 83 headers): (df1,df2) = (100,36) (200,80) and the six above; `C =F.INV(2^-1022,Ar,Br)`,
  `D =BETA.INV(2^-1022,Ar/2,Br/2)`, `E =F.INV(2^-1020,Ar,Br)`, `F =F.INV.RT(2^-1022,Br,Ar)`.

**Predictions.**
- Block 1: the model's step counts move with k (B 318 -> 512, D 337 -> 1, E 330 -> 512, F 137 -> 251). H9 predicts one
  step-count threshold for every column; a shape-specific failure predicts `#NUM!` in B at every k and numbers elsewhere.
  C says whether BETA.INV shares the rule, G whether the right-tail entry does. Number versus `#NUM!` has no noise.
- Block 2: small j can reach a subnormal residual (flush stop); from about j = 52 the stop is ordinary, so the class
  boundary shows as a change in F. Under H10 the relative error band is of order DBL_MIN/p and halves per step of j, and I
  (constant for an accurate tiny-tail (17,50) quantile) jumps discretely. Cached anchor: `F.INV(2.22507385850721E-308,17,50)`
  = 2.698957993077865e-37 (web) against the R3 mathematical value 2.59580221092205219e-37.
- Block 3, model F results and offsets from the converged quantile: (33,12) 4.524960641104397e-20, +1.19 %;
  (36,13) 1.6237415311537908e-18, +3.11 %; (60,21) 1.0209716001731779e-11, +1.80 %; (63,22) 3.1041983633621045e-11, +1.11 %;
  (77,27) 1.8272033639978865e-09, +1.20 %; (85,30) 1.0329889061892471e-08, +1.64 %. Percent-scale separation from a converged
  inverse is decisive. H1 versus H3/H4 differ only at (60,21) (H4 -45, H3 -23 ulp): use it only above the repeat spread.
  If J = K but N and P disagree reproducibly, private forward arithmetic is favoured over a seed error.
- Block 4: model step counts (100,36) 4, (200,80) 3. A wrong step count shows as a percent-level difference.

**Existing cells to exclude (F.INV).**
- LN at FYL2X near-ties (for example `LN(0.612354100000274)`; about 30 of 2.9 million cached LN cells).
- Single flush-stop cells (p <= about 1E-292, both shapes > 1): deterministic in Excel but hash-limited for a model, which
  matches each by chance with probability 0.4-1.0. Judge by counts over G09 and FIN-01, never by one cell.
- gen-m9/caa1eed48b1883fe A237 (df2 = 1186 via VAR.P): matched by about 58 % of all variants tried.
- gen-m9/589f42e67f4b4b8d A319 (`F.INV(1E-200,PERCENTOF(..),5)`): R3 math-over-Excel; attribution check only.
- The R3-marked accuracy/F.INV/s1 A112, A126, A146, A160, A171: they must follow the math. Research readouts at DBL_MIN
  are not permission to remove the R3 safeguard.
- Point probabilities with one deviance on bd0's series branch and M >> k on the log side (55-65 % reproduced by any order).

---

## HYPGEOMDIST

**Open question.** One cell: `HYPGEOMDIST(2,4,3,19)`, cached 0.09287925696594419; the engine's exact kernel gives rounded
30/323 = 0.09287925696594428, six ulps higher. With Excel's own cached BINOM.DIST factors, the identity
`HYPGEOMDIST(x,n,K,N) = (B(x;K,p)/B(n;N,p))*B(n-x;N-K,p)`, p = max(x,n-x)/N, each operation rounded to 64 bits and then to
binary64, reproduces 379/379 cached tuples (N <= 20, p in {.1,.125,.2,.25,.3,.5,.7,.9}). It matches the target with
Excel's factors, but the engine's factors are off (at (3,4,7,15) B(3,7,.2) and B(1,8,.2) are +2 and +1 ulp, leaving the
identity 4 ulps off), so the identity alone is not a gate-clean fix. Untested: non-dyadic p, N > 20 with Excel's own
factors, and what happens when a factor underflows.

**Hypotheses.**
- H1: the identity above with the worksheet BINOM.DIST factors, everywhere (so fixing BINOM.DIST fixes HYPGEOMDIST).
- H2: the same structure with p = n/N (R's `dhyper` choice): 11/106 on cached tables without a zero cell.
- H3: a private factor path that agrees with worksheet BINOM.DIST only at the dyadic p of the cached controls.
- H4: the factor kernel agrees with BINOM.DIST for factors with k = n and differs for factors with m - k = N - n
  (addendum 2, "Still open", from 579/1442 direct-division fits). This predates the 379/379 result and conflicts with it,
  so it is kept as a separate hypothesis.
- H5: a separate large-population or count-width path (for example a 32-bit count above 2^31).
- Underflow (HYP-02): U1 no guard (error once the denominator factor is 0); U2 switch to p = n/N; U3 switch to transposed
  sizes p = K/N; U4 log-space or other fallback; U5 return 0 when the numerator factor underflows (as in the one cached zero).

### HYP-01 (P0): factor bridge at non-dyadic p and large N

**Layout.** Row 1 headers; rows 2-66 one tuple each, A:D = x, n, K, N. Paste row 2, fill E:R down:
```
E =HYPGEOMDIST(A2,B2,C2,D2)      F =HYPGEOM.DIST(A2,B2,C2,D2,FALSE)
G =MAX(A2,B2-A2)/D2   H =BINOM.DIST(A2,C2,G2,FALSE)   I =BINOM.DIST(B2-A2,D2-C2,G2,FALSE)   J =BINOM.DIST(B2,D2,G2,FALSE)
K =(H2/J2)*I2         L =IFERROR(SUM(E2,-K2)/E2*2^52,"err")
M =B2/D2              N =BINOM.DIST(A2,C2,M2,FALSE)   O =BINOM.DIST(B2-A2,D2-C2,M2,FALSE)   P =BINOM.DIST(B2,D2,M2,FALSE)
Q =(N2/P2)*O2         R =IFERROR(SUM(E2,-Q2)/E2*2^52,"err")
```
Tuples (x,n,K,N), rows 2-66: (2,4,3,19) (3,4,7,15) (3,4,4,11) (10,12,17,26) (20,29,21,30) (5,10,6,32) (13,47,25,66)
(14,33,48,72) (3,27,43,75) (40,74,43,87) (39,47,78,89) (16,75,16,99) (64,92,70,100) (1,3,3,13) (4,5,4,13) (3,5,5,13)
(5,5,9,14) (0,2,10,15) (2,3,8,15) (2,4,3,15) (1,4,4,15) (1,5,9,15) (2,5,10,15) (2,4,9,16) (1,1,8,17) (3,3,6,17) (4,4,6,17)
(2,5,5,17) (1,5,6,17) (2,5,7,17) (2,5,4,18) (5,5,5,18) (0,1,5,19) (0,1,9,19) (1,2,4,19) (2,2,9,19) (0,1,5,6) (0,4,1,7)
(0,2,3,100) (2,4,3,23) (6,6,90,102) (69,184,115,230) (10,50,200,1000) (523,1726,649,2456) (4083,4221,8278,9730)
(1116,11227,1645,16501) (21847,24776,51317,58526) (100,1000,5000,100000) (436213,437898,1361108,1367085)
(2190,2733817,2702,3504782), and (1,2,3,N) for N = 202770, 1E6, 1E7, 1E8, 1E9, 1048575, 1048576, 1048577, 2147483646,
2147483647, 2147483648, 2147483649, 1099511627775, 1099511627776, 1099511627777.

**Predictions and thresholds.**

| outcome | reading |
|---|---|
| L = 0 in every row (at most |L| <= 1 in the rare 64-bit-intermediate case, 1 of 379 cached) | H1 |
| R = 0, L != 0 | H2 |
| L != 0 where p is non-dyadic, L = 0 where p is dyadic | H3; H4 if the failing factor is always the m - k = N - n one |
| L = 0 below a size and != 0 or errors above it, on both neighbours of 2^20, 2^31 or 2^40 | H5 |

L and R are relative residuals scaled by 2^52 computed inside Excel, so they survive 15-digit transport; |L| >= 2 is
decisive. Model values: the engine's own factors put the identity 4 ulps off at (3,4,7,15) and 10 to 16 relative units of
2^-53 below cached Excel on the cached (1,2,3,N) cells. H:J add 195 BINOM.DIST values, mostly at non-dyadic p, for the
kernel work (HYP-03).

### HYP-02 (P0): factor underflow

**Layout.** As HYP-01 (A:R) plus, per row r: `S =C/D`, `T =BINOM.DIST(Ar,Br,Sr,FALSE)`, `U =BINOM.DIST(Cr-Ar,Dr-Br,Sr,FALSE)`,
`V =BINOM.DIST(Cr,Dr,Sr,FALSE)`, `W =(Tr/Vr)*Ur`, `X =IF(ISERROR(Er),ERROR.TYPE(Er),0)`. Rows:
- 2-14: (x, n, 10000, 20000) with n = 3258 + 24i and x = n/2 + 25, i = 0..12. The true value stays near 0.01 while the
  denominator factor B(n;N,x/N) falls from about 1E-296 through 2.2E-308 (n = 3378) to below 4.9E-324 (n = 3522).
- 15-22: (n/2, n, n, 2n), n = 2048, 2400, 2440, 2444, 2446, 2448, 2450, 4096. True value about 2/sqrt(pi*n), about .02;
  the model puts the denominator's normal underflow at n = 2446-2448.
- 23-33: the 11 cached Gate A cells that hit the underflow: (32779,103344,55462,182949) (86981,185471,151580,336802)
  (32015,153806,69820,347800) (104652,248905,153240,357994) (81139,899806,104873,1158742) (382970,760906,966072,1922132)
  (475258,5199470,490334,5359677) (812734,911284,5166198,5793205) (24231,139791,993978,5970034)
  (114521,901933,802783,6219712) (2966973,7428902,3274608,8195813).
- 34-50: the cached zero cell (41966,46221,65797,74726) with its table scaled by s = .86, .88, .89, .90, .905, .91, .915,
  .92, .93, .94, .95, .955, .96, .965, .97, .98, 1 (rounded tuples in the case): the numerator factor crosses the underflow
  while the true value (1E-160 to 1E-185) stays representable.

**Predictions.** U1: X nonzero from the first row where J = 0. U2: R = 0 from that row on (the R factors sit near their
modes, so E is then within a few units of 2^-53 of exact). U3: E = W bit for bit. U4: E matches no group. U5 (rows 34-50):
E = 0 exactly from the row where H = 0, although the true value is representable; a smooth positive E throughout
contradicts U5. J also shows whether BINOM.DIST returns subnormals (switch at 2.2E-308) or flushes (switch near 4.9E-324).
Number versus error versus zero is far above any rounding; before the boundary the candidates differ by hundreds of ulps.

### HYP-03 (P1): the factor kernel at the target's p

**Layout.**
- **A1:L9** (row 1 headers), rows 2-9 = (a,b) (1,6) (4,7) (2,19) (1,7) (2,7) (1,8) (1,10) (2,100); row r: `C =Ar/Br`,
  `D =BINOM.DIST(1,1,Cr,FALSE)`, `E =EXP(LN(Cr))`, `F =BINOM.DIST(0,1,Cr,FALSE)`, `G =1-Cr`, `H =EXP(LN(Gr))`,
  `I =SUM(Dr,-Er)/Dr*2^52`, `J =SUM(Dr,-Cr)/Dr*2^52`, `K =SUM(Fr,-Hr)/Fr*2^52`, `L =SUM(Fr,-Gr)/Fr*2^52`.
- **A11:I28** (row 11 headers), j = -8..8 in A12:A28: `B =0.2+Ar*2^-55`, `C =BINOM.DIST(3,7,Br,FALSE)`,
  `D =2/19+Ar*2^-56`, `E =BINOM.DIST(2,16,Dr,FALSE)`, `F =BINOM.DIST(4,19,Dr,FALSE)`, `G =BINOM.DIST(2,3,Dr,FALSE)`,
  `H =0.75+Ar*2^-53`, `I =BINOM.DIST(8,12,Hr,FALSE)`. Each step is one ulp of p; E, F and G are the target's three factors.
- **A30:N42** (row 30 headers), (k,n) = (1,2) (1,4) (1,5) (1,6) (1,7) (2,7) (3,8) (4,8) (5,10) (2,8) (2,10) (6,12); row r:
  `C =A/B`, `D =B*C`, `E =B*(1-C)`, `F =BINOM.DIST(A,B,C,FALSE)`, `G =BETA.DIST(C,A+1,B-A+1,FALSE)`,
  `H =NEGBINOM.DIST(B-A,A+1,C,FALSE)`, `I =F*(B+1)`, `J =C*F`, `K =SUM(G,-I)/G*2^52`, `L =SUM(H,-J)/H*2^52`,
  `M =EXP(LN(F))`, `N =SUM(F,-M)/F*2^52` (all with row r).
- **A44:M54** (row 44 headers): (k,n,p0,step) = (2,10,3/10,2^-54) offsets -1,+1; (256,1280,3/10,2^-54) offsets -1,0,+1;
  (3,10,1/5,2^-55) offsets -1,+1; (384,1280,1/5,2^-55) offsets -1,0,+1. Row r: `D =p0`, `E =Dr+Cr*step`, `F =B*E`,
  `G =B*(1-E)`, `H =(A-F)/(A+F)`, `I =(B-A-G)/(B-A+G)`, `J =BINOM.DIST(A,B,E,FALSE)`, `K =BETA.DIST(E,A+1,B-A+1,FALSE)`,
  `L =ABS(H)<1/5`, `M =ABS(A-F)<(1/5)*(A+F)`.

**Predictions.** Endpoints: direct p and q give J = 0 and L = 0; an EXP(LN(p)) success term gives I = 0; a rounded-q
logarithm gives K = 0; a log1p(-p) complement can leave K != 0. Cached controls at p = 1/8 and 1/10 already have D one ulp
above p, so direct p is wrong there. Consecutive p: a Loader kernel that first stores n*p and n*q gives plateaus wherever
adjacent rows share both products; a kernel using p directly moves every row (by about (k - np)/(pq) ulp(p), a few units
of 2^-53). Mode block (both deviances zero): the current kernel matches only 27/50 cached mode PMFs (B(1,2,.5) .5 against
cached .49999999999999994; B(4,8,.5) .2734375 against .27343750000000006; B(5,10,.5) .24609375000000003 against
.24609375000000008; B(2,8,.25) .31146240234374994 against .31146240234375), so a bd0-only repair leaves F unchanged; a
value-scaled beta implementation predicts K = 0, the negative-binomial value product L = 0. bd0 cut: at (2,10,3/10) the
stored v is exactly -0.2; the current path gives .23347444050000005 and an inclusive split-deviance cut .23347444050000002
(model). A real 0.2 branch must move with L/M at the 128x-scaled location; the 128x rows magnify exponent rounding to tens
or hundreds of PMF ulps.

**Existing cells to exclude (HYPGEOMDIST).**
- The 11 cached underflow cells (HYP-02 rows 23-33) until HYP-02 fixes the rule: their candidates differ by less than the
  engine's binomial error there.
- Cells with N > 10,000 used without their factor cells: Excel's value carries factor errors of 1E2 to 2E4 units of 2^-53.
- HYPGEOM.DIST(..,TRUE) cells, for PMF rounding: they add summation rounding.
- Cells that are exactly 0 or 1 (outside the support, n = 0, K = N, n = N), except as entry-condition evidence.
- gen-m9 cells with non-literal arguments (FORECAST.ETS.CONFINT, COUNTA expressions) until their scalar arguments are fixed.
- Standalone 15-digit mode PMFs and small complement pairs (one-ulp differences); use the in-sheet residuals instead.
- Endpoint denominators such as "B(1,6,1/6) = .40187757201646096" inferred from modelled factors: conditional, not oracles.

---

## FVSCHEDULE

**Open question.** One cell: gen-m9/89f0aee953a55a45 Calc!A349, `=FVSCHEDULE(5916,{-1E+300,1E+300})`, saved by Excel for
the web as a literal 0 (two fills); every arithmetic order gives -inf in doubles. Smaller workbooks with an overflowing
call fail to open on excel-graph (T-3312). Finite products already match (130 replays: principal first, rates row-major,
stored `1+rate`, one rounding per step). The engine on main returns `#DIV/0!` for an infinite result, from an inherited
`is_infinite()` check with no Excel evidence (addendum 2); a NaN result (inf times a zero factor) falls through as a number.
Nothing yet shows what a formula that reads the overflowed cell sees.

**Hypotheses.**
- H1: a true numeric zero is returned (per-step and final replacement are observationally equivalent here).
- H2: the live value is non-finite (or nonzero) and only the saved file shows 0.
- H3: the calculation or final store fails and the cell keeps its previous value (an initialized or stale 0).
- H4: an error is returned on desktop; the exact code is to be measured, not assumed to be `#DIV/0!`.
- H5: an exception left pending by the overflow surfaces in a later add-in call (not requested here; see Deferred).
- H6: the accumulator keeps an extended exponent range until return, so an out-of-range intermediate can come back.
- K1: each step is a plain binary64 product; K2: each step is rounded to 64 bits and then to 53 (twice-rounded).

### FVS-01 (P0, desktop; web carrier on Excel for the web): what the overflowing call is

**Layout** (sheet `Probe`, label in A, formula in B, rows 1-22):
```
target                     =FVSCHEDULE(5916,{-1E+300,1E+300})
formula_present            =ISFORMULA(B1)
formula_text               =FORMULATEXT(B1)
value_type                 =TYPE(B1)
is_number                  =ISNUMBER(B1)
is_error                   =ISERROR(B1)
equals_zero                =B1=0
positive                   =B1>0
negative                   =B1<0
sign                       =SIGN(B1)
reciprocal                 =1/B1
self_difference            =B1-B1
if_error_sentinel          =IFERROR(B1,-17)
error_code_or_zero         =IF(ISERROR(B1),ERROR.TYPE(B1),0)
is_blank                   =ISBLANK(B1)
absolute_value             =ABS(B1)
scaled_by_2_to_1023        =B1/(2^1023)
plus_one                   =B1+1
below_minus_1E307          =B1<-1E+307
above_1E307                =B1>1E+307
text_wrapper               ="["&B1&"]"
finite_calculation_control =7*1.25*1.125
```
**Protocol.** One fresh workbook and Excel process per variant; disable "Precision as displayed"; enter the target first
and read its live `.Value2` and VARIANT kind before entering the observers; then save, reopen and read again. Repeat the
original in three processes. A refusal, a failed save or a process error is a result, not a zero. Variants (replace B1):
`=FVSCHEDULE(1,{1E+300,1E+300})` (positive overflow), `=FVSCHEDULE(5916,{1E+300,-1E+300})` (reversed), and
`=FVSCHEDULE(1E+300,1E+10)` (scalar schedule).
- **Seeded update** (sheet `Seed`, separate workbook): `B1 7`, `B2 0`, `B3 0`, `B4 =FVSCHEDULE(B1,B2:B3)`, and the observer
  rows above pointed at B4 plus `=B4=B1`. Calculate (B4 = 7), save a copy, switch to manual, type `-1E+300` in B2 and `1E+300`
  in B3, calculate B4 only, read it live, then calculate the observers and save/reopen. Repeat with B1 = 19 and 5916, and
  once with the rate signs reversed.
- **Inline** (separate workbook, run only after the above shows no carried state): `=FVSCHEDULE(5916,{-1E+300,1E+300})+1`,
  `=ISNUMBER(FVSCHEDULE(5916,{-1E+300,1E+300}))`, `=(FVSCHEDULE(5916,{-1E+300,1E+300}))=0`.
- **Web carrier** (Excel for the web only, the oracle of record): copy eval/suites/gen-m9/cases/89f0aee953a55a45.case.json
  (the only workbook with this call that opens) and add on sheet `Calc`, whose column C is empty: C349 `=(A349)=0`,
  C350 `=A349+1`, C351 `=ISNUMBER(A349)`, C352 `=ISERROR(A349)`, C353 `=1/A349`, C354 `=A349&""`, C355 `=A349>1E+307`,
  C356 `=A349<-1E+307`, C357 `=A349*0`, C358 `=TYPE(A349)`, C359 `=ABS(A349)`. Variant b also adds the three inline cells
  at C360:C362. Variant c is variant a with the uploaded cached `<v>` of A349 set to 12345. If b is refused where a opens,
  record that too.

**Predictions** (all categorical: type, boolean or error, no last-bit question):

| observer | H1 zero | H2 live non-finite | H3 stale value s | H4 error |
|---|---|---|---|---|
| value_type, is_number, is_error | 1, TRUE, FALSE | 1, TRUE, FALSE | 1, TRUE, FALSE | 16, FALSE, TRUE |
| equals_zero, sign | TRUE, 0 | FALSE, -1 for -inf | s = 0, SIGN(s) | error |
| reciprocal, self_difference | #DIV/0!, 0 | 0 or error, error | 1/s, 0 | error |
| plus_one, below_minus_1E307 | 1, FALSE | #NUM!, TRUE | s+1, FALSE | error |
| if_error_sentinel, error_code_or_zero | 0, 0 | value, 0 | s, 0 | -17, the code (2 DIV, 3 VALUE, 6 NUM, 7 N/A) |
| Seed sheet B4 after the edit | 0 | live value differs from saved 0 | 7 / 19 / 5916 retained | error |
| web carrier C356, C350 | FALSE, 1 | TRUE, #NUM! | FALSE, 1 (variant c: saved 12345) | error |

H1 and H3 agree whenever the stale value is 0; the seeded update and web variant c separate them.

### FVS-02 (P0): the edges of the double range

**Layout.** Row 1 headers; rows 2-43 one target each. Paste row 2 and fill C:I down; each row's B, F and H are listed:
```
A label   B target   C =ISNUMBER(B2)   D =IF(ISERROR(B2),ERROR.TYPE(B2),0)   E =IFERROR(B2=0,"err")
F scale k   G =IFERROR(B2*2^F2,"err")   H control (where given)   I =IF(ISERROR(H2),ERROR.TYPE(H2),0)
```
| rows | B (k in F) | purpose |
|---|---|---|
| 2-9 | `=FVSCHEDULE(P,1)`, P = (2^1023-2^971), (2^1023-2^970), (2^1023), (2^1023+2^971), then the four negated; k = -1023; H `=P*2` | first overflow, both signs |
| 10-22 | `=FVSCHEDULE((1+m*2^-52)*2^1023,1-m*2^-51)`, m = 1048576, 30000000, 33554431, 33554432, 33554433, 33554434, 33558528, 33562622, 33562623, 33562624, 33562625, 33562626, 40000000; k = -1023; H `=((1+m*2^-52)*2^1023)*(1+(1-m*2^-51))` | K1 versus K2 at the cut |
| 23-28 | `=FVSCHEDULE(2^1023,{1,-0.5})`, `=FVSCHEDULE(-(2^1023),{1,-0.5})`, `=FVSCHEDULE(2^1023,{-0.5,1})`, `=FVSCHEDULE(FVSCHEDULE(2^1023,1),-0.5)` (k = -1023); `=FVSCHEDULE(1E+300,{1E+10,-0.999999999})`, `=FVSCHEDULE(1E+300,{-0.999999999,1E+10})` (k = -997) | out of range and back |
| 29-34 | `=FVSCHEDULE(2^1023,{1,-1})`, `{-1,1}` (k = 0), `{1,0}`, `{1,1E-300}`, `{1,#N/A}`, `{1,"x"}` (k = -1023) | zero and unit factors; validation order |
| 35-37 | `=FVSCHEDULE(1,{1E+300,1E+300,-1})` (k 0), `=FVSCHEDULE(5916,{1E+300})` (k -1000), `=FVSCHEDULE(0,{-1E+300,1E+300})` (k 0) | inf times 0; in-range and zero-principal controls |
| 38-40 | `=FVSCHEDULE(1E-300,{-0.9999999999})` (k 1030), `=FVSCHEDULE(1E-300,{-0.9999999999,-0.9999999999,1E+30})` (k 964), `=FVSCHEDULE(2.5E-308,{-0.2})` (k 1022) | subnormal running products |
| 41-43 | `=FVSCHEDULE(2^-1022,J:L)` with J:L = (`=-1+2^-52`, `=-1+2^-52`, `=2^200`) k 926; (`=-1+2^-52`, 0, `=2^200`) k 874; (`=2^200`, `=-1+2^-52`, `=-1+2^-52`) k 926 | underflow with a normal final result |

Run the sheet as one workbook only if FVS-01 showed no carried state; otherwise copy each row's B (and J:L) into its own
FVS-01-style workbook.

**Predictions** (model arithmetic; G is the fingerprint B*2^k):
- Rows 2-3: finite under every hypothesis (G = 2-2^-51 and 2-2^-52). Rows 4-5 and 8-9: H1 gives 0 (E TRUE), H4 an error
  code in D, H2 a number with E FALSE. A sign-dependent split rejects sign-symmetric handling; a failure in rows 2-3
  rejects the IEEE threshold.
- Rows 10-22: K1 overflows for m <= 33554432 (rows 10-13) and gives DBL_MAX from m = 33554433 (row 14); K2 overflows
  through m = 33562623 (row 18) and gives DBL_MAX from row 19. Rows 14-18 separate K1 and K2 by DBL_MAX against zero or
  error. H shows where the `*` operator cuts on the same inputs.
- Rows 23-26: H6 gives G = 1 (row 23) and -1 (row 24); a double loop gives 0 (H1) or an error (H4); a suppressed per-step
  store that keeps the old 2^1023 gives 0.5 and -0.5; row 25 is 1 under every hypothesis; H6 in row 23 with zero or error
  in row 26 locates a function-return boundary. Row 27: H6 9.999999718180687e+300, double loop overflow; row 28 control
  9.999999718180686e+300.
- Rows 29-34: row 29 is inf times 0 under a double loop (NaN, then an error, a 0 or a NaN-number) and 0 under H6 (pair it
  with row 23: row 29 alone cannot prove H6); row 30 is 0 under all; rows 31-32 must behave as row 4; row 33 gives `#N/A`
  (D = 7) if the schedule is validated before multiplying, the overflow's error if the loop streams; row 34 likewise with
  `#VALUE!` (D = 3).
- Rows 38-40, per-step double / flush below DBL_MIN / extended exponent: 1.0000000827404e-310 / 0 / 1.0000000827404e-310;
  9.99988867182683e-291 / 0 / 1.000000165480749e-290; 2e-308 / 0 / 2e-308. A kept subnormal saves as 0 in the file, so read G.
- Rows 41-43, G under gradual underflow / per-step flush / retained range: 0 / 0 / 1; 1 / 0 / 1; 1 / 1 / 1. All nonzero
  finals are normal, so the saved-subnormal artefact cannot explain a 0.

### FVS-03 (P1): product rounding in range

**Layout.**
- **A1:G9** (row 1 headers), n = 0..7 in A2:A9: `B =FVSCHEDULE((1+An*2^-52),0.5+2^-52)`, `C =MOD(B*2^52,32)`,
  `D =FVSCHEDULE(-(1+An*2^-52),0.5+2^-52)`, `E =MOD(ABS(D)*2^52,32)`, `F =(1+An*2^-52)*(1.5+2^-52)`, `G =MOD(F*2^52,32)`.
- **A11:J23** (row 11 headers), rows 12-23, principal in A and j in B: (163155824546526, 130635076464448)
  (656542657233605, 514812003729707) (379307607009905, 373741565052584) (628105692524842, 169689079467139)
  (821300390846059, 904421884804959) (222582297394973, 215009126879211) (920289876130538, 589928720146801)
  (586229564169117, 306986357007248) (814752858009111, 649643786500197) (443331593688838, 995685211640490)
  (183033083458162, 436001276273540) (786331085903402, 299627412334170). Row r: `C =FVSCHEDULE(Ar,Br*2^-52)`,
  `D =Ar*(1+Br*2^-52)`, `E =PRODUCT(Ar,1+Br*2^-52)`, `F =(Cr-Dr)`, `G =(Cr-Er)`, `H =MOD(Cr*64,1024)`,
  `I =MOD(Dr*64,1024)`, `J =MOD(Er*64,1024)`.

**Predictions** (exact rational arithmetic, not Excel): C and E for n = 0..7 are 1, 3, 4, 6, 7, 9, 10, 12 under K1 and
1, 2, 4, 6, 7, 8, 10, 12 under K2 (n = 1 and n = 5 discriminate). H for rows 12-23 under K1: 990, 776, 580, 808, 856, 390,
248, 536, 280, 980, 750, 920; under K2: 988, 784, 576, 800, 848, 392, 256, 544, 272, 976, 752, 912. The fingerprints are exact
integers below 2^53, so they survive 15-digit transport; G (and I, J) calibrate the operator separately and must not be
assumed to share FVSCHEDULE's rounding.

**Existing cells to exclude (FVSCHEDULE).**
- gen-m9/89f0aee953a55a45 Calc!A349 as evidence of a live overflow-to-zero rule: a saved zero with no dependents; keep it as
  the Gate B observation.
- gen-m9/81bcdc0bec0c8e6b A276 and gen-m9/b72d952c5f52cb4d A324: the same schedule with a zero principal; zero controls,
  not overflow observations.
- The 120 Gate A FVSCHEDULE caches: none crosses the range; they cannot vote between K1, K2 and H6.
- gen-m9/01523d247594952e Calc!A225 (2 ulp): explained by the MEDIAN of an even count (1.4000000000000001, the
  lo+(hi-lo)/2 form), not by FVSCHEDULE. gen-m9/984cba67bbe6972b Calc!A141: a GAMMA.INV last digit (T-3239).
- docs/research/t4458/fill1 (BESSELY at a subnormal x): order-dependent; do not transfer its pending-error pattern.
- probes/accrint-zero-results: unstable ACCRINT 0/#NUM!; not an analogue.

---

## IRR (error-only classifier)

**Open question.** Addendum 3's kernel (one secant in d = 1 - v, 200 updates, retries from d(0.1) and d = -0.1) reproduces
every cached numeric return, and t4974 G20 confirmed it out of sample (63/63 in a model of the kernel). Still open: which
error, `#NUM!` or `#DIV/0!`, Excel returns when no attempt converges, and what happens on overflow inside a run.

**Hypotheses.**
- H1: decision 0316's error-only classifier as kept in the tree (a flat pair at one point after a far step with
  |NPV| > 3e226, 27-144 flows, largest flow 1.3e4..6.7e14, runs from 0.1); right on 30 of 44 cached `#DIV/0!` cells.
- H2: one more attempt after d = -0.1, a far secant step from a fixed d = s, which is `#DIV/0!` if the step lands on a zero
  or infinite discount; right on 44/44 `#DIV/0!` and 630/630 `#NUM!` cached cells for s in (0.6495, 0.6745). H2-E: the
  attempt only decides the error kind; H2-R: it is a full run that can return a root.
- H3: the same far step taken in the rate instead of d (at best 4-5 errors on cached cells).
- H4: path dependence at NPV's noise floor, with no classifier rule (addendum 3: two cells with the same modelled
  trajectory get different results).
- Overflow inside a run: S-end (a non-finite secant step ends the call as `#NUM!`, 3 cached cells) or S-run (only that run
  fails); V-num (an overflowing NPV sum with no zero discount ends the call as `#NUM!`, 1 cached cell), V-div, or V-run.
- Worksheet NPV, an overflowing term before a zero discount: N-div or N-num.

### IRR-01 (P0): invariances and overflow kinds

**Layout.** Column A one formula per row, column B `=IF(ISERROR(Ar),ERROR.TYPE(Ar),0)`. The case holds the canonical
array-constant formulas. Pasteable equivalent for the annuity rows: put P (or q*c) in a helper column and the flows in a
row range, for example `D2 =-P`, `E2:BL2 =c` (60 ones scaled by c), `A2 =IRR(D2:BL2)`; if the range form is used, also
enter CAL-01 H27/H28 in range form, which must stay `#NUM!` and `#DIV/0!`.
- Rows 1-114: `=IRR({-q*c, c x n})` for n = 60, q = 196608, 262144, 327680, 393216, 524288, 655360, 786432, 1048576 and
  n = 40, q = 50331648, 67108864, 83886080, 100663296, 134217728, 167772160; c in row order 2^-10, 2^-3, 1, 1 negated
  (`{q,-1,...}`), 2^10, 2^20, 2^20 negated, 2^26, 2^30. Combinations needing more than 15 digits are omitted (9 rows per
  n = 60 group except 8 for q = 1048576; 7 per n = 40 group except 8 for q = 83886080).
- Rows 115-202: `=IRR({-P, 40 ones})` alternating with the same plus 20 zeros (rows 115-160, P = 100000 x 1.3^i up to
  32118400) and plus 60 zeros (rows 161-202, P = 2000 up to 380099); the unpadded row is the control.
- Rows 203-263: `=IRR({-P, 200 x 1000})`, P = 3800000 .. 4387299 (61 values; the case lists them).
- Rows 264-277: worksheet NPV at rate -0.999999999999999 with a 1E+100 (or 1E+80, 1E+85) term at period 15, followed by 0..7
  zeros, ones or a -1E+100 term.

**Model predictions (not Excel values).** N = `#NUM!`, D = `#DIV/0!`, one letter per row, in row order:

| rows | n, q | H1 | H2 (any s in the bracket) |
|---|---|---|---|
| 1-9 | 60, 196608 | NNNNNNNNN | NNNNNNNNN |
| 10-18 | 60, 262144 | NNNNNNNNN | NNNNNNNNN |
| 19-27 | 60, 327680 | NNNNNNNNN | DDDDDDDDD |
| 28-36 | 60, 393216 | NNNNNNNNN | DDDDDDDDD |
| 37-45 | 60, 524288 | NNNNNNNDD | DDDDDDDDD |
| 46-54 | 60, 655360 | NNNNDDDDN | DDDDDDDDD |
| 55-63 | 60, 786432 | NDDDDDDDN | DDDDDDDDD |
| 64-71 | 60, 1048576 | NDDDDDDD | DDDDDDDD |
| 72-78 | 40, 50331648 | NNDDDDD | NNNNNNN |
| 79-85 | 40, 67108864 | DDDDDDD | NNNNNNN |
| 86-93 | 40, 83886080 | DDDDDDDN | NNNNNNNN |
| 94-100 | 40, 100663296 | DDDDDDD | NNNNNNN |
| 101-114 | 40, 134217728 and 167772160 | all D | all D |

- Padding: unpadded rows: number for P <= 5712, `#NUM!` above, under both. +20 zeros: H1 `#NUM!` to 11245500 then `#DIV/0!`;
  H2 `#NUM!` to 285610 then `#DIV/0!`. +60 zeros: H1 `#NUM!` 7426..102372 then `#DIV/0!`; H2 `#DIV/0!` from 7426. A returned
  number must be the same double padded or not.
- Overflow rows: 203-209 a number under every rule; rows 210-225 (band A, the first run's secant product overflows) and
  226-234 (band B, the NPV sum overflows, no zero discount); rows 235-263 `#DIV/0!` under every rule. (S-end,V-num): A N,
  B N. (S-run,V-div), the merged tree: A number (for example -0.02201504679613897 at P = 3852612), B D. (S-run,V-run): A and
  B numbers. (S-end,V-div): A N, B D.
- NPV rows 266, 270, 272, 274 separate the rules: N-div `#DIV/0!`, N-num `#NUM!`; the other NPV rows are controls.
- H4 predicts kind changes that follow neither table and are not covariant with exact 2^k scaling.
Every contrast is a number against an error or `#NUM!` against `#DIV/0!`; 2^k scaling is exact in binary.

### IRR-02 (P1): read the seed from Excel's own far steps

**Layout** (column A formula, column B error code; the case is canonical, the range form above also applies):
- Rows 1-196: `=IRR({-P, 60 ones}[,g])`, groups of 28 rows: guess omitted, then g = 1.85, 1.9, 1.95, 2, 2.05, 2.1;
  P = 284000, 285420, 286847, 288281, 289723, 291171, 292627, 294090, 295561, 297039, 298524, 300016, 301516, 303024, 304539,
  306062, 307592, 309130, 310676, 312229, 313790, 315359, 316936, 318521, 320113, 321714, 323323, 324939.
- Rows 197-378: the same with 40 ones, groups of 26; P = 105000000, 105525000, 106053000, 106583000, 107116000, 107651000,
  108190000, 108731000, 109274000, 109821000, 110370000, 110922000, 111476000, 112034000, 112594000, 113157000, 113722000,
  114291000, 114863000, 115437000, 116014000, 116594000, 117177000, 117763000, 118352000, 118944000.
- Rows 379-410: `=IRR(v)` for v = {0.111,0.138,-794,533,-0.623,-16} {0.253,-11.5,-98.1,6.22,37.4} {0.163,-57.6,-276,679,-426}
  {-0.103,5.4,92,-6.27,-33.4} {-0.137,0.24,585,-0.522,-239,0.1} {0.104,0.882,-472,34.3,168} {-1.18,162,135,-356,222,-36.6}
  {1.07,-0.401,-616,896,265,-781} {0.456,-3.12,-136,-9.74,542,-420} {0.146,-30.4,-10.8,373,-670}
  {-0.235,1.36,81.7,-54.7,-1.81,7.05} {0.156,-2.06,-191,190,-56.4} {0.101,2.91,-116,224,-116,-0.588}
  {-0.128,-1.79,221,-5.63,-106,15} {-0.196,2.22,126,18,-611,550} {2.26,-143,-791,821,-235,-6.98}
  {-0.125,-2.26,639,-214,-121,6.62} {-0.137,16.3,104,-6.28,-61.4,14.9} {0.18,12.2,-450,0.993,182}
  {0.54,-2.94,-660,725,-319,45.6} {-0.129,0.695,26.8,-55.3,20.5,19.4} {0.107,-2.99,-46.5,77.6,-37.5}
  {-0.232,0.883,197,-321,0.658,180} {-0.184,-0.129,274,0.286,-112} {-0.128,-15.9,170,1.26,-113,35.9}
  {-1.09,-1.36,431,-368,89.8} {0.85,0.683,-480,858,-456} {1.91,-202,71.7,457,-487,3.49} {-0.444,0.241,652,-1.19,-265}
  {-1.46,-0.165,754,-846,298,-11.9} {-52.4,-4,-167,-0.813,51.9,-10.6} {6.5,-6.71,38.3,0.531,-3.79,0.432}.

**Idea and model predictions.** A run from guess g starts at d = g/(1+g) (0.649 to 0.677 for these g), and a run that
meets a zero discount ends as `#DIV/0!` (137 cached cells), so each guess group shows Excel's own far-step outcome from that
d. First `#DIV/0!` P per group (below it `#NUM!`):

| group | H1 | H2, s = 0.6495 | 0.655 | 0.66 | 2/3 | 0.67 | 0.6745 |
|---|---|---|---|---|---|---|---|
| n=60, guess omitted (rows 1-28) | no flip in window | 318521 | 312229 | 307592 | 301516 | 298524 | 295561 |
| n=40, guess omitted (rows 197-222) | all `#DIV/0!` | 116594000 | 114863000 | 113157000 | 110370000 | 109274000 | 108190000 |

Guess groups (every model and every s agree, since these runs start at d = g/(1+g) themselves): n = 60, g = 1.85, 1.9,
1.95, 2, 2.05, 2.1 flip at 318521, 312229, 307592, 301516, 297039, 292627; n = 40 flip at 116594000, 114291000, 112594000,
110370000, 108731000, 107116000 under H2 and are all `#DIV/0!` under H1.

H2 predicts the omitted-guess flip at the P where the guess column with g/(1+g) = s flips, the same s at n = 60 and 40.
H3 reads different s at different n (from r0 = 0.66: P = 314907 and 116841000, read as s = 0.6525 and 0.6492). No flip at
n = 60 with an all-`#DIV/0!` n = 40 sheet supports H1. The grid step is 0.5 % in P, so s is read to about 0.002. Under S-end
the lower part of each guess column stays `#NUM!` whatever s is; under S-run guess columns whose start is below s flip with
the omitted column. Rows 379-410: H1, H2-E and the kernel give `#NUM!` in all 32; H2-R gives numbers whose digits depend on
s (for example row 379: 82.61681576151425 at s = 0.655, 82.61681579190338 at s = 2/3).

**Existing cells to exclude (IRR).**
- probes-numeric/t3411-irr-r4-fresh-4 Calc!A103 (`#DIV/0!`, 27 flows): flatness of the attempt's first pair changes with the
  seed by 0-1 ulp of NPV; wrong for 19 of 49 seeds in the bracket.
- accuracy/IRR/r4-T-3411 Calc!A296 (`#NUM!`, 3 flows): NPV's rounding quantum (about 6e-5) sits just under the 1e-4 test;
  noise for H2-R only. Both remain valid kernel checks.
- Cells with flow literals or guesses longer than 15 significant digits (probes-numeric/t2927-irr-fallback-sweep 37,
  t3347-irr-conventional-holdout 7, probes/t3406-irr-num-boundary 5, probes/t4027-irr-annuity-scale-guess 2, and
  probes/t2565-irr-guess-sweeps 130 guesses): exclude from any replay that does not model the 15-digit cut.
- The 121 range-form IRR cells and the 63 t4974 G20 cells: H1 and H2 agree on all of them.

---

## Overlap with T-4974 and other merged evidence

| existing result | answers | dropped or narrowed here |
|---|---|---|
| t4974 G16 NORM.S.INV fingerprint | reset in (0.70700, 0.70750]; low-x odd symmetry | reset map over 0.7065-0.7103 dropped; NSI-02 maps only the bracket; no new symmetry sweep |
| t4974 G09 F tiny-tail neighbourhood | the target is not isolated; `#NUM!` at (11,3) for both p | plain shape grid at 1E-300 and adjacent-df2 sweep at df1 = 9 dropped; FIN-01 adds BETA.INV twins of exactly these 40 cells |
| t4974 G08 F back-transform | compensated four-log transform at ordinary p | exact-coordinate transform control dropped |
| t4974 G22 BETA.INV at DBL_MIN | Excel does not flush the first BETA.INV residual | FIN-03 keeps only the F caller's stop class |
| t4974 G10 F AS 109 tolerance cap | the accuracy tolerance (confirmed) | not the iteration cap FIN-03 asks about |
| t4974 binom-loader-store-chain (G18) | bd0 x87 cutoff at exact-boundary decimal p | HYP-03 drops the offset-0 small-count bd0 rows; FIN-02's binomial block is its adversarial complement; it completes no HYPGEOMDIST factor triple and has no HYPGEOMDIST cell |
| t4974 G20 IRR | the kernel on fresh cells; `IRR({-500,1,1,1,1,1})` `#NUM!` with guesses omitted and 0.5 | kernel re-validation and the 200-update boundary sweep dropped |
| T-4974 overall | no FVSCHEDULE, IMSUB, BESSELY or MEDIAN cell | none |

## Deferred (not in this batch)

- FVSCHEDULE pending-exception checkerboard and explicit `Range.Calculate` order (H5), the BESSELY pending-error follow-up,
  the small-workbook refusal matrix on excel-graph, the sibling-overflow calibration (IMSUB, IMSQRT, EFFECT, ...), and the
  MEDIAN/AGGREGATE(12)/PERCENTILE even-count form: run only if FVS-01 shows carried state or the web carrier is refused.
- NORM.S.INV branch-endpoint probes at x = .5 (small kernel) and |q| = .425 (tail cut): lower information than NSI-01..03;
  non-boundary tails are already explained by AS 241.
- F.INV density grid at tiny x over eight shapes, the 96-shape DBL_MIN grid and the df-dispatch sweep: after FIN-02 picks a
  storage rule.
- HYPGEOMDIST success-complement/sample-swap/2^k-scaled target, degenerate entry conditions (n = N, K = 0), the factor tables
  for N = 7, 9, 11, and the direct exponent-store test (cached evidence already rejects a universal stored exponent).
- IRR wide ladders at n = 60/40/30, n = 80/100 annuities, the update-200 pole bound, and explicit small guesses: after
  IRR-01 and IRR-02.

This request changes no production code, eval case, baseline, tolerance, gate or R3 policy.
