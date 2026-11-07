# IRR solver lineage table (T-4835, Phase 1 of plan.md)

This page compares how other solvers handle each part of an IRR iteration, set against what the Excel oracle
evidence shows. It is desk research: no oracle run, no code change. Its purpose is to rank, for each of the three
events the engine cannot model yet (a **flat secant**, an **iterate past the pole**, the **step cap**), the mechanisms
that T-4836's offline hypothesis bank should score first.

## Sources

All files were read on 2026-11-07 from the default branch of each project. Line numbers refer to that revision.

| Key | Source | Path |
|---|---|---|
| **XL** | Excel's IRR, as the oracle shows it | docs/excel-semantics/financial.md (sections named in the table) |
| **XR** | Excel's RATE, fitted bit-exact on 782/782 | financial.md "RATE is a secant on the FV residual" and its amendments |
| **VB-I** | .NET `Microsoft.VisualBasic.Financial.IRR` (a port of the VB runtime's C code) | dotnet/runtime `src/libraries/Microsoft.VisualBasic.Core/src/Microsoft/VisualBasic/Financial.vb` lines 236-358, `OptPV2` 903-924 |
| **VB-R** | the same file's `Rate` | Financial.vb lines 692-746 |
| **LO** | LibreOffice Calc `ScInterpreter::ScIRR` | LibreOffice/core `sc/source/core/tool/interpr2.cxx` lines 1437-1535 (`SCdEpsilon` 1.0E-7, line 58) |
| **GN** | Gnumeric `gnumeric_irr` and goal-seek | GNOME/gnumeric `plugins/fn-financial/functions.c` lines 1296-1398; `src/tools/goal-seek.c` (`goal_seek_newton` 318-, `goal_seek_initialize` 154-162) |
| **POI** | Apache POI `Irr` | apache/poi `poi/src/main/java/org/apache/poi/ss/formula/functions/Irr.java` |
| **FJS** | formulajs `IRR` ("inspired by Apache OpenOffice") | formulajs/formulajs `src/financial.js` lines 712- |
| **IC** | IronCalc upstream `compute_irr` (the fork's origin) | ironcalc/IronCalc `base/src/functions/financial/financial_util.rs` lines 49-147 |
| **DOC** | Microsoft documentation | the IRR help page (quoted in plan.md section 2); "What's New: Changes made to Excel functions" (support.microsoft.com, 355d08c8-8358-4ecb-b6eb-e2e443e98aac) |

Not reached: the VBA `IRR` of Office (VBE7) has no oracle (plan.md, T-1075). Lotus 1-2-3 and Multiplan `@IRR` manuals
did not turn up in a web search. Microsoft KB Q70976 ("IRR and RATE Give Different Results in Version 3.00") and a
MrExcel thread on IRR differences between Excel 2007 and 2010 were found, but their hosts are not reachable from this
session, so neither is cited for any value.

## Marks

Each source cell in the table carries one mark against Excel's IRR:

- **E**: Excel-evidenced. Excel does the same, per the cited financial.md section.
- **C**: contradicted. Excel's evidence rules this out for IRR, at least where the evidence reaches. The cell gives the
  reason.
- **?**: unknown. Excel's behaviour at this mechanism is not observed, so the source's choice is an open candidate.

The **XL** column itself is marked E (evidenced) or ? (unknown).

## The table

| Mechanism | XL (Excel IRR) | XR (Excel RATE) | VB-I | VB-R | LO | GN | POI | FJS | IC |
|---|---|---|---|---|---|---|---|---|---|
| **Method** | E: secant ("IRR is a secant in d", 263/263 bit-exact) | E: secant | E: secant | E: secant | C: Newton, analytic derivative | C: Newton, then a net of test points and bisection | C: Newton, then a bracketed Newton/bisection hybrid | C: Newton with a forward-difference slope, then bisection | C: Newton, then bisection |
| **Variable** | E: d = 1 - v = r/(1 + r) (T-3120) | r | C: r (VB verbatim is 3/263 bit-exact, plan.md section 6) | r | C: r | C: r | C: r | C: r | C: r |
| **Start** | E: d0 = g/(1 + g), d1 = d0 -+ 0.001, toward larger \|NPV\| | E (same family): r1 = r0 - 0.001·sign(f(r0)) | C: r1 = r0 +- 1e-5, by the sign of NPV(r0) | r1 = r0/2 or 2·r0 | n/a (Newton) | n/a | n/a | n/a | n/a |
| **Step anchor** | E: on the newer iterate, d2 = d1 - f1(d1 - d0)/(f1 - f0) | E: the same | E: the same, in r | E: the same | n/a | n/a | n/a | n/a | n/a |
| **NPV evaluation** | E: worksheet NPV's running product of x = 1/(1 - d), twice-rounded operators (T-3205) | E: closed-form FV residual | ?: Horner from the last flow, `total/(1 + r) + c_i`. The plan's H9 candidate for long series, where Excel's arithmetic is unknown; on short series the fit set prefers the running product | n/a | ?: `c_i/pow(1 + r, i)` in a Kahan sum (also H9-like) | running product of 1/(1 + r) | running product of 1 + r | running product of 1 + r | `c_i/(1 + r)^(i+1)`, plain sum |
| **Leading zeros** | E: dropped (T-3120) | n/a | E: `OptPV2` skips them | n/a | ? (harmless in Newton) | not dropped | not dropped | not dropped | not dropped |
| **Step stop** | E: \|dd\| < 1e-7, absolute | E: \|dr\| < 1e-7 | E: \|dr\| < 1e-7 | C: no step test | E: \|dx\| < 1e-7 (`SCdEpsilon`) | C: relative step < 5e-11 | E: \|dx\| <= 1e-7 | C: 1e-9, then its bisection stage | C: 1e-8 |
| **NPV stop** | E on short series: \|NPV\| < 1e-7, absolute; ? on 21+ flows (plan.md section 6, the residual) | E: \|f\| < 1e-7 | ?: \|NPV\| < max\|c\|·1e-9 (scaled by the flows). Contradicted on the fit set, where the stop is absolute; open on long series (H8) | E: \|f\| < 1e-7 alone | C: none | only f = 0 exactly | C: none | 1e-10 | C: none |
| **Zero-step stop** | E: d2 = d1 with \|NPV\| < 1e-7·max\|c\| returns (T-3120); a stall returns up to step 41 and is `#NUM!` from step 70 (T-3408) | E: zero step, then flat, holds (T-3348) | none | none | n/a | n/a | n/a | n/a | n/a |
| **Flat secant** (f1 = f0) | ?: the engine fails; Excel returns a root on many such runs | E: hold at r1, judged by \|f\| < 1e-4 and 1 + r > 1e-4 (T-3348); restart once from r0 at steps 3..16 (T-3698) | ?: nudge the older iterate one offset away and re-evaluate; still flat is an error | ?: the same nudge; still flat is `CalcDivByZero` | n/a (f' = 0 gives a 0 step through `div_allow_zero`, then the 1e-7 stop) | ?: halve the last nonzero derivative and step on | f' = 0 gives NaN, then the bracket stage | f' too small, then bisection | n/a |
| **Pole** (d >= 1, r <= -1) | E: an iterate at d = 1 exactly, or an NPV that cannot be evaluated, is `#DIV/0!` and ends the call with no retry (T-2802, T-3407); ? past d = 1 | E: runs on below -1, judges only the result (T-3348) | ?: clamp r0 = (r1 - 1)/2, halfway from r1 to -1 (H5); a start at r1 <= -1 is an error | ?: no clamp (LEvalRate at r <= -1) | ?: no clamp; 1 + x = 0 divides by zero | C: leaves [xmin, xmax] = [-1, ...], so the run is an error. Excel's d = 1 is `#DIV/0!`, not `#NUM!` | ?: 1 + x0 = 0 gives NaN, then the bracket stage | ?: clamps to -0.99999999 | ?: no clamp |
| **Cap** | E: runs of 51..200 steps return their own iterate; past 200 the step count does not decide (T-3408). C: the documented "20 tries" | E: 200 steps; the 200th iterate returns under \|dr\|, \|f\| < 1e-4 and 1 + r > 1e-4 (T-2689) | C: 40 steps, then an error (Excel returns runs of 51..200) | C: 40 | C: 20 | ?: 100 Newton steps, then bisection | ?: 1000 per stage | ?: 1000 per stage | C: 50 Newton, 50 bisection |
| **Retry / second stage** | E: one retry of the secant from 0.1; past it, the value does not depend on the guess (T-4844), and is often a Newton iterate on P(v) from v = 0 (partial) | E: one restart (above) | C: none | C: none | C: none | ?: a net of 10 points around the guess (x2 and /2), Newton from -0.99, a point at 1 - eps, then bisection. Its net reads the guess, unlike Excel's second stage (T-4844) | ?: a bracket on a fixed grid (-1 + 10^-9 .. 10,000) plus the guess, then Newton/bisection. The bracket nearest the guess is chosen, so it reads the guess too | ?: bisection from the Newton end point | ?: bisection on [-0.99999, 100], then Newton from 200 and from -2 |
| **Far-root bound** | E: `#NUM!` at 1 + r >= about 1E+7 | n/a | none | none | none | ?: xmax = (DBL_MAX/1e10)^(1/n) - 1, a bound that depends on n | ?: bracket grid up to 10,000 | ?: clamps to 1000 | ?: bisection up to 100 |
| **Guess range** | E: `#VALUE!` at <= -1 or >= 2^53 ("IRR's guess range") | | E: guess <= -1 is an error | n/a | C: guess -1 silently becomes 0.1 | C: no check | C: no check | C: no check | E: guess <= -1 is `#VALUE!` |
| **Zero root** | E: exact 0 only where the arithmetic gives it ({-4,4}; {-100,50,50} with guess 0 or at scale 1E6, not by default) | | none | none | ?: snaps \|x\| < 1e-7 to 0 when guess = 0, which matches Excel's guess-0 cells | none | none | none | none |
| **Error codes** | E: `#NUM!` (no sign change, no convergence, far root); `#DIV/0!` (an NPV that cannot be evaluated; some flat runs, unexplained, T-3900) | E: `#NUM!` | one exception for every failure | ?: flat twice is `CalcDivByZero`, the cap `CannotCalculateRate`: one code per event (H10) | Err:523 (no convergence) | `#NUM!` | NaN | `#NUM!` | `#NUM!` |

Notes on the table:

- **One family.** VB-I, VB-R and Excel's IRR and RATE share a secant anchored on the newer iterate, a 1e-7 step test
  and an offset start. LO, GN, POI, FJS and IC are all Newton and are a different family. No Newton candidate
  reproduces the fit set: the best scores 74/263 (financial.md "IRR is a secant in d"). They matter here only as
  examples of what a second stage can look like.
- **Excel 2010 did touch IRR.** The "What's New: Changes made to Excel functions" page lists IRR and XIRR (with
  CUMIPMT, CUMPRINC, IPMT, PMT and PPMT) among the functions whose algorithm changed to improve accuracy. It does not
  list RATE. So today's IRR may not share RATE's code, and the RATE transplant (plan.md section 6) is evidence of a
  shared design, not of shared source. The VB runtime's C code presumably predates that change, which would fit its sibling, non-copy
  status (3/263 bit-exact).
- **The documented constants are pre-2010.** "0.00001 percent" is the 1e-7 step test (evidenced). "20 tries" matches
  only LO's cap and is contradicted for Excel (runs of 51 to 200 steps return, T-3408). VB's 40 is contradicted as well.

## Ranked mechanisms for T-4836

Ranked by how much evidence backs each mechanism in Excel's own solver family, with the plan's hypothesis ids. The
feasibility deltas are those of plan.md section 6.

### Flat secant

1. **H1, hold and judge** (XR, T-3348): hold at the newer iterate and return it under |f| < 1e-4 and 1 + r > 1e-4.
   It is Excel's own rule for RATE, and the largest single gain in feasibility.py (+109 engine misses).
2. **H3, restart once from the older iterate** (XR, T-3698), early in the run. It is RATE's refinement of H1 and
   needs H1 underneath.
3. **H2, nudge and retry** (VB-I, VB-R): move the older iterate one offset (Excel's 0.001 in d, not VB's 1e-5) away
   from the newer one and step again. Flat again is an error. With H10, VB-R suggests that error is `#DIV/0!`, which
   fits financial.md "IRR is #DIV/0! on a flat secant after a far step".
4. GN's halved derivative. It is a Newton idea with no secant analogue in the family. Score it last, or not at all.

### Pole (an iterate at d >= 1)

1. **H4, run on past d = 1** (XR "runs on below -1"): NPV is evaluated at the negative v, the run continues, and only
   the result is judged. It is evidenced for RATE (50/50 bit-exact on t3348-rate-below-minus-one) and gives +65 over
   H1 at cap 50. It has to keep d = 1 exactly as `#DIV/0!` (v = 0, T-2802) and the overflow `#DIV/0!` (T-3407).
2. **H5, the r-half clamp** (VB-I): 1 + r <- (1 + r1)/2, so v doubles. VB's actual IRR code.
3. **H6, the d-half clamp**: the same idea in Excel's variable. It walks v toward 0 and is the plan's candidate for
   the "Newton from v = 0" iterates (financial.md "IRR's returns where the secant fails are Newton iterates on P(v)").
4. FJS's fixed clamp to -0.99999999 in r. No family member does this; keep it only as a control.

### Cap

1. **H7, cap 200 and return the last iterate under the 1e-4 tests** (XR, T-2689). It is consistent with T-3408's
   "up to 200 always finish". The 181 step-cap cells of the residual (plan.md section 6) are its test.
2. **No fixed cap, a stall test only.** T-3408 found that past 200 steps the step count does not decide. Score it
   against H7 on the 201..300-step runs of docs/research/t3408/long_runs.tsv.
3. Caps of 40 (VB) and 20 (LO, DOC) are contradicted. Do not score them.

### Across events

- **H10, one error code per event** (VB-R maps flat-after-nudge to divide-by-zero and the cap to "cannot calculate").
  Score it with each flat-handling candidate on the 44 + 11 `#DIV/0!`-type cells.
- **H8, an NPV stop scaled by max|c|** (VB-I). It is contradicted on short series. Score it only on 21+ flows, where
  the residual lies, and through the x2^k ladders of T-4839.
- **H9, another NPV arithmetic on long series**: VB-I's Horner from the last flow and LO's pow-and-Kahan sum are the
  two concrete forms to score against the running product (Phase 4, T-4844 and T-4845).
- **Second stage.** Excel's second stage ignores the guess (T-4844). That rules out GN's net and POI's bracket as
  written, since both read the guess. A guess-free stage (H6's walk to v = 0, or IC's fixed bracket) stays in.
