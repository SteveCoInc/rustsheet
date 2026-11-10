# Probe hand-off: fill the 15 `val3-*` probes with the Excel oracle (2026-11-09)

**For:** Steve's Excel-for-the-web oracle agent, or the Excel oracle workflow's operator.
**Why:** decide the next numerical questions for NORM.S.INV, NORM.INV, LOGINV, F.INV,
HYPGEOMDIST and FVSCHEDULE. HYPGEOM.DIST/BINOM.DIST are shared-kernel controls;
IRR needs only a small confirmation, not another broad search.

## What this change adds

Exactly **15 pending workbooks, 8,857 checked formula cells**, in
`eval/suites/probes/cases/val3-*.case.json`. They use the same `rustsheet-case/v1`,
`sheets[].cells` and qualified `check` layout as val2. Every formula, including
canaries and observers, is checked. There are **no expected values or oracle sidecars**.
The engine, evaluator, existing tests, baselines and gates are unchanged.

Each case's description freezes its competing numerical or categorical predictions.
[val3-numerical-predictions.json](val3-numerical-predictions.json) supplies the longer
NSI/HYP prediction tables and high-precision FIN alternatives. This is research metadata,
**not an oracle or math sidecar**; never copy it into expected values.

| Case | Checked formulas | What it decides |
|---|---:|---|
| `val3-cal-01` | 11 | Raw capture precision and continuity with five cached val2 anchors |
| `val3-nsi-01` | 774 | Shared hidden inverse-erf/gamma core versus accurate independent gamma; core preimages |
| `val3-nsi-02` | 132 | Direct z at cancellation points; stored-z versus scale-first affine/LOGINV association |
| `val3-fin-01` | 7,433 | Public-kernel AS 109 trace pilot, stored/nested steps, private beta/F returns and all 17 p bins |
| `val3-fin-02` | 227 | Six one-step private/public identities and seven first-seed re-entries |
| `val3-hyp-01` | 150 | Common rounded PMF versus caller-specific normalization/assembly, with exact dyadic scaling |
| `val3-irr-01` | 42 | Fresh zero-root residue, array/range and scaling; two fourth-attempt confirmations |
| `val3-fvs-01` | 11 | Isolated `0/(2^1023)` |
| `val3-fvs-02` | 11 | Isolated `2^1023` |
| `val3-fvs-03` | 11 | Isolated zero divided by a loaded denominator |
| `val3-fvs-04` | 11 | Isolated zero-principal FVSCHEDULE |
| `val3-fvs-05` | 11 | Isolated overflowing schedule followed by rate zero |
| `val3-fvs-06` | 11 | Same stored factors, but tiny nonzero second rate |
| `val3-fvs-07` | 11 | Zero factor before any product overflow |
| `val3-fvs-08` | 11 | Overflow followed by a recovering factor |

**Size choice.** The r7 FIN request for all 17 neighboring trajectories plus six
other full trajectories is deliberately reduced to **one complete bounded pilot**,
all 17 raw inverse-return anchors, and the six short-search controls. One primary
trace mismatch already rejects the proposed public path; matching it does **not**
confirm the untraced trajectories. The largest workbook builds locally to about
115 KB. The NSI pilot retains all 192 ladder points; redundant square/root columns
are computed offline instead of consuming oracle cells. No wide exploratory grids.

## How to run

1. **Run CAL-01 first.** From the repository's `tools` directory:

   ```bash
   uv run rustsheet-oracle fill -b excel-graph --suite probes --case probes/val3-cal-01.case.json
   ```

   Inspect the raw numeric sidecar, not displayed text. In every workbook,
   `Precision!B2:B4` must be **0.9999999999999998, 1, 1.0000000000000002** and
   `Precision!C2:C4` must be **-1, 0, 1**. The C column uses `(B-1)*2^52`, not SUM.
   If B has been rounded to 15 decimal digits, C can still look right: **both**
   columns must pass. CAL-01's five raw anchors are specified in its description.
   A capture failure means stop; an anchor change means save and report that
   snapshot before interpreting any conditional predictions.

2. **Fill the other 14 in one pass**, selecting only this batch, still from `tools`:

   ```bash
   args=()
   for path in ../eval/suites/probes/cases/val3-*.case.json; do
     [[ "$path" == *val3-cal-01.case.json ]] && continue
     args+=(--case "probes/$(basename "$path")")
   done
   uv run rustsheet-oracle fill -b excel-graph --suite probes "${args[@]}"
   ```

   Each case becomes a **separate workbook/session**. Do not combine the eight FVS
   cases into one workbook, even on different sheets. Do not add companion powers,
   overflowing schedules, reciprocal observers, MOD fingerprints or `2^-1023`
   scaling to them. Ordinary precision canaries are already included. Excel for
   the web is sufficient for this batch; **no desktop runner is required**.

3. **For a browser-driven agent:** use the existing spec-to-xlsx builder for each
   case, open each workbook separately, let Excel recalculate, and download the
   saved workbook. Capture its values with the repository's existing snapshot/
   oracle pipeline, preserving the formula cells, types, exact error spellings and
   full binary64 values. Do not transcribe formatted UI numbers or paste decimal
   replacements for integer-limb formulas. Keep precision-as-displayed disabled.
   Return the 15 `val3-*.oracle.json` sidecars, saved recalculated workbooks, backend/
   build identifiers, calculation order and any refusal/canary diagnostics.

4. **Repeatability is separate evidence.** One fill does not establish cold-reopen
   or process-state invariance. If cheap, retain independent fresh-session repeats
   of FIN-01 and FVS-05:08, including a cold reopen, as separately named artifacts;
   do not replace the first result. `fill --force` alone is **not** an independent
   calculation: the existing oracle client can reuse its content-addressed recalc
   cache. No repeat is required to complete this one-pass request. Web sessions do
   not give control of the server process or floating-point exception state.

**Capture rules.** SUM near-zero snapping made some val2 residuals unreadable;
subnormal POWER and large MOD fingerprints also lost information. This batch
uses raw return values, normal-range dyadic scaling and nested direct subtraction.
Compare binary64 bits offline. Worksheet equality/inequality flags may use different
comparison semantics from a native numerical loop; the FIN trace therefore records
their raw operands too. An error or a surprising number is a result, not a reason
to omit a cell or substitute a prediction.

## Per-family questions and predictions

### NORM.S.INV / NORM.INV / LOGINV

**Known from cached values re-read for this request.** `val2-nsi-03!P!C11` is
0.8645876128073491. Its noisy third ladder falls from 0.7392428978657416 to
0.7392428978657366 at J3:J4. In `val2-nsi-01`, the gamma square root K2 is
0.47693627620446993 whereas the exposed proxy D2 is 0.47693627620447: the proxy
cannot simply be assumed to be the internal core. The LOGINV row C65/E65 is
-0.8416212335729145 / 56153957292.82028; E=I (EXP of stored-z association) on
all 24 rows 53:76. None of this recovers the central approximation's coefficients.

**NSI-01.** `P!A:D`, `G:J`, `M:P`, rows 2:65, repeat the three val2 x/proxy/z
ladders and add `GAMMA.INV(x,0.5,1)`. The frozen shared-core hypothesis requires a
binary64 w such that `RN53(RN64(w*RN53(sqrt(2))))` equals repeated z and gamma is
`RN53(RN64(w*w))`. RN means nearest-even significand rounding, not a tolerance.
The predictions file enumerates the admissible gamma values per row, conditional
on those z anchors repeating. An accurate separate gamma inversion instead predicts
`RN53(erfinv(exact captured x)^2)`.

At D11 these predict **0.3737558701099552** versus **0.37375587010995437**.
The strict exposed-proxy-square model is a subset of the shared-core predictions
on these cached ladders, not a fully independent alternative: accepting it does not
reject sharing. A gamma outside the shared set rejects that conditional mechanism;
one inside can select a previously ambiguous core. Agreement does not establish a
unique central approximation or prove a particular processor architecture.

**NSI-02.** Rows 2:4 use the r7-requested p=0.6305586598182364 and its neighbors,
mean=-1, sd=3; rows 5:7 use p=0.500001994711402 and its neighbors, mean=0, sd=1E6.
The latter cached workbook's F28 was re-read as 5.0000000001370815; its direct z
is precisely the missing measurement. Rows 8:10 replay val2 rows 58,65,69.
Columns D/E expose z/proxy; F/G/H compare native NORM.INV, stored-z affine and
scale-first affine forms. P stores the gamma argument; I/J/K test a gamma-derived
core separately, without presuming that rounded centering recovers the same input.
L:M are the two lognormal inverse names; N/O are the two EXP associations.

Stored-z predicts **F=G, L=M=N**; proxy-first predicts **F=H, L=O**. At row 8 the
cached lognormal association contrasts predict 56153957292.82128 versus
56153957292.82148; at row 9 the LOGINV anchor is 56153957292.82028. Judge the
fresh direct-z cells, not a substituted engine quantile. The broader spectral/
scale-covariance request and reset-predicate investigation are deferred.

### F.INV

**Known.** `val2-fin-02!P!F14` is 2.5799655780427067e-233. The six val2 short
searches return their worksheet proposal N in private BETA.INV T. The primary
17-adjacent-p F returns interleave two values rather than forming one monotone
plateau. A final F logarithm bin hides beta-coordinate differences, so record
both inverse returns. The r7 density-order lead is not a recovered full trajectory.

**FIN-01.** `Seed` computes the normal seed in sheet cells. `Trace` has 384 trial
rows (2:385), with a separate row for each one-third damping retry. `Adjacent`
records all 17 p inputs and private BETA.INV/F.INV returns, four-log maps, CDF/PDF
at the returned coordinate, and a normal-range scaled relative residual.

Trace columns: B x; C/D CDF/PDF; E residual; F stored step; G prior-square bound;
H damping; I adjustment; J square; K proposal; L stop code; M acceptance; N next x;
O nested-call step at **the same x**; P scaled relative residual; Q/R first best
coordinate/residual; S cumulative contractions; T distinct CDF-evaluation count.
The stop codes are 0 continue, 1 zero worksheet residual, 2 unusable slope,
3 squared-step floor, 4 unchanged proposal. After stopping, the state is frozen.
For a code-4 plateau the summary keeps the first least-residual coordinate.
384 is a **probe budget**, not an inferred Excel cap; `INCOMPLETE` is inconclusive.
This published-AS-109-style replay is a hypothesis, not proprietary implementation text.

The public-kernel hypothesis predicts `Seed!B13=B11`; its compensated F mapping
predicts B14=B12. Identical stored/nested steps before the stop support identical
updates along this traced path; any F/O difference exposes storage sensitivity
without needing a second full nested trajectory. A private caller/update alternative
can differ despite matching public kernels and has no unique numerical prediction.
Raw operands, not the worksheet's stop flags alone, must determine the offline verdict.

Numerical alternatives: accurately converged beta/F are **1.74423319304034e-67 /
5.8141106434678e-68** (100-digit calculation); cached beta/F are
**1.7442331930869875e-67 / 5.814110643623313e-68**. If val2 repeats, Adjacent D is
5.8141106436234775e-68 at offsets -6,-3,-2,+1,+4,+5,+7,+8 and
5.814110643623313e-68 at the other nine offsets. Reproducing F bins alone is not
success: the primary coordinate and raw trace must also agree. Only after this
pilot agrees should all remaining trajectories and iteration-cap boundaries be expanded.

**FIN-02.** Rows 2:7 are the six short-search shapes; rows 8:14 are all seven
numeric re-entry controls. J is a newly computed seed, K/L its CDF/PDF, M/N stored/
nested proposals, O/P private beta/F, Q/R seed/proposal F mappings, S/T the return's
CDF and residual diagnostic. Stored-public predicts O=M; nested-public predicts
O=N on rows 2:7. The first-seed slope-stop hypothesis predicts O=J and P=Q on
8:14, versus the different accurate roots in the predictions file. Proposal errors
on unusable-slope rows are intentional. At row 2 the cached seed/proposal are
1.284002864569973e-19 / 1.244364176303708e-19; P8/P13's cached re-entries are
9.062275163816585e-58 / 4.296064882817687e-64. No fitted cap or shape exception.

### HYPGEOMDIST / HYPGEOM.DIST / BINOM.DIST

**Known.** Raw val2 HYP-01 E=F=K on all 59 numeric rows; HYP-02 E=F=K on all
27 nonzero-denominator rows and E=F=Q on all 22 zero-denominator rows. These
factor identities are why another hypergeometric grid would be redundant. The
legacy target is 0.09287925696594419 at (2,4,3,19), repeated by CAL-01.

**HYP-01.** Twenty-four rows use p=1/2, n=2^m-1 for m=3:8 and k=1,2,(n-1)/2,
(n+1)/2. D/F/H are BINOM, BETA density divided by n+1, and NEGBINOM multiplied by
2; E/G/I are their success/complement reversals. Every external scale is exactly
a power of two in the normal range. A single rounded PMF followed by exact caller
scaling predicts all six equal. Cross-caller differences therefore locate finishing/
normalization differences, not the external normalization's rounding. Reversal
differences also expose ordered accumulation; equal outputs do not uniquely identify it.

The prediction file includes deliberately simplified, frozen finishing experiments:
let ell=RN53(log(exact PMF)), then compare a common RN53(EXP(ell)) with separately
stored ell+LN(n+1) and ell+LN(1/2) before EXP and exact external scaling. LN terms
and the additions in these illustrative alternatives are binary64; EXP is rounded
from high precision. They are **not** complete r7 accumulation-kernel predictions.
At row 18, k=1,n=127, accurate common rounding is 7.464389127721526e-37; the
common stored-log triple is (7.464389127721548e-37 repeated), versus the scaled
triple (7.464389127721548e-37, 7.464389127721539e-37, 7.464389127721534e-37).
The non-dyadic deviance-boundary extension is deferred until this assembly test.

### FVSCHEDULE

**Known.** The raw val2 calls mix numeric zero and NUM despite identical stored
factors: FVS-02 B31/B32 are 0/NUM for second rates 0/1E-300. B30 is NUM even with
a zero factor first; B23/B24 are NUM/0 for positive/negative recovery. FVS-01 B17,
its zero carrier divided by `2^1023`, is NUM. These facts do **not** identify a pure
per-call error classifier or establish carried server state.

**Four isolated controls, FVS-01:04.** Their sole target cells Probe!B1 are
`0/(2^1023)`, `2^1023`, `0/8.98846567431158E307`, `FVSCHEDULE(0,-0.5)`.
Ordinary arithmetic predicts 0, 8.98846567431158e307, 0, 0. A standalone power
failure makes the first two NUM; a magnitude-related division failure can affect
the first and third; a schedule-only failure can affect the fourth. Each workbook
also records TYPE, ISNUMBER, error code and an identity observer. None contains
one of the other targets, so the original combined-sheet ambiguity is not repeated.

**FVS-05:08.** Each has computed principal `2^1023`, with schedules `{1,0}`,
`{1,1E-300}`, `{-1,1}`, `{1,-0.5}` respectively. Frozen categorical predictions:

| Explanation | 05 | 06 | 07 | 08 |
|---|---|---|---|---|
| Sequential stored factors, each overflow becomes zero | 0 | 0 | 0 | 0 |
| Sequential stored factors, stop with NUM on overflow | NUM | NUM | 0 | NUM |
| Extended exponent until final store, final overflow becomes NUM | NUM | NUM | 0 | 8.98846567431158e307 |
| Pure per-call behavior preserving the val2 combined-sheet results | 0 | NUM | NUM | NUM |

Numeric predictions imply TYPE=1, ISNUMBER=TRUE, error code=0; NUM implies
TYPE=16, ISNUMBER=FALSE, error code=6. The last row is an invariance prediction,
**not an algorithm**. If isolated 05/06 differ, stored factors alone are insufficient
for these fresh calls. If they become equal, context dependence is a lead, not
proof of pending exceptions. Web-session/process limitations still apply.

**Deferred explicitly:** isolated negative-principal B24, nested B26, B29/B33/B34,
loaded/referenced/computed identical-principal variants, raw versus stored rate
variants, seeded live updates and reversed combined-sheet order. These must not
be added to the eight canonical workbooks. The first isolated pass determines which
follow-up is necessary; strict process-state and live-update experiments need a
separately controlled execution protocol. No desktop claim follows from this fill.

### IRR

**Known.** The cached `FINANCIAL/IRR.xlsx` Two Values A4:B4 is {-4,4}; C4 is
2.220446049250313e-16. The r7 report requests a fresh literal/range/scaling check
of this remaining residue. The r7 fix is not part of this checkout's starting
commit, so this request makes no claim that the local engine already contains it.

**IRR-01.** Rows 2:5 use {-s,s} for s=1,4,16,256, as arrays and ranges, with
omitted, 0, .1 and 2 guesses. A persistent default returned-iterate residue predicts
2^-52 and scaled H/I=1; a fresh exact-zero result predicts H/I=0. An argument-mode
effect predicts array/range differences; guess dependence need not follow either
default prediction. Rows 8:9 repeat val2-irr-02 A379/A410 (numerical predictions
in the case description), with error code 0. No new broad IRR investigation.

## Parse check

`rustsheet-eval cases probes` discovers **3,239 cases, including all 15 val3 cases**.
Discovery alone is not a parser test: the successful release-build/full-eval run
also loads all 15 with status `pending`, not invalid, failed or timed out. A local
driver additionally calls the existing `runner::run_case_values` on every val3
case, checking the actual Rust parser/evaluator and all checked formula addresses.
The existing `rustsheet_tools.spec2xlsx.build_xlsx` builds all 15 workbooks; a
separate audit requires each `check` set to equal exactly its formula set, with
no duplicates, inline expectations or oracle sidecars. No evaluator code is changed.

Reproduce discovery and the full parse/load check from the repository root:

```bash
cargo build --release -p rustsheet_eval && target/release/rustsheet-eval run
target/release/rustsheet-eval cases probes
```

Pending status is expected: locally evaluated numbers are **not Excel evidence**.

## What happens next

Return the filled sidecars in one oracle-only change, with the saved workbooks and
capture diagnostics. Score raw outputs against the frozen predictions first; if
an anchor changed, report that fact instead of adjusting predictions to it. Record
inconclusive cases and the explicit deferred experiments. Do not infer a global
algorithm from one matching F bin, one gamma preimage, or an uncontrolled web state.
No engine fix, newly passing function or Gate B improvement is claimed by val3.

**Results:** pending Excel. Family follow-up lanes remain inverse normal, F.INV,
the shared HYP/BINOM kernel, FVSCHEDULE, and IRR confirmation.

## Results (2026-11-09, excel-graph)
Filled with `rustsheet-oracle fill -b excel-graph`, CAL-01 first, each case its own workbook. Not yet scored: the
per-family owner tasks (T-5107 F.INV, T-5108 FVSCHEDULE, T-5109 HYPGEOMDIST, T-5110 IRR, T-5112 inverse normal) read
these values against the predictions above.

**CAL-01 passed.** All five anchors equal the same-build predictions (P!B2:B6), and Precision!B2:B4 are
0.9999999999999998, 1, 1.0000000000000002 with C2:C4 -1, 0, 1, so raw capture keeps the low bits.

**12 of 15 filled.** probes/val3-fvs-05, probes/val3-fvs-06 and probes/val3-fvs-08 stay pending: in each, the
oracle's own canary sheet reads `SQRT(2)` as `#NUM!` (two attempts), so the fill refuses the workbook. Each of the three
holds one FVSCHEDULE whose schedule overflows (`{1,0}`, `{1,1E-300}`, `{1,-0.5}` on 2^1023), while probes/val3-fvs-07
(`{-1,1}`, zero before any overflow) filled normally. One hypothesis, not yet tested: an overflow leaves state that
breaks later calculation in the same workbook, which is what the isolated FVS design was meant to rule in or out
(T-5108). The filled FVS values (Probe!B1): val3-fvs-01 `0/(2^1023)` 0, -02 `2^1023` 8.98846567431158E+307,
-03 0, -04 `FVSCHEDULE(0,-0.5)` 0, -07 `FVSCHEDULE(2^1023,{-1,1})` 0 (each a number, no error); recorded in
financial.md, "FVSCHEDULE whose running product overflows or underflows".
