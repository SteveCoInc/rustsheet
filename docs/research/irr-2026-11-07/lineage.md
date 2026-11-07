# IRR solver lineage: mechanisms by source (T-4835, 2026-11-07)

Phase 1 of the IRR plan (plan.md section 7). Each row of the table is one mechanism of an IRR solver. The first
column says what Excel's IRR does, from oracle evidence. The other columns say what each related solver does and how
that compares with Excel. The table feeds T-4836's hypothesis bank. It ends with a ranked list of the mechanisms to
test at each of the three events where the replica fails: a flat secant, an iterate past the pole and the step cap.

Each cell carries a mark:

- **E**: Excel-evidenced. Excel does this, and the cell cites the financial.md entry or probe that shows it. In a
  source column, E means the source does what Excel's evidence shows.
- **C**: contradicted. Excel's evidence rules this out for Excel's IRR. The cell says which evidence.
- **U**: unknown. No Excel evidence decides it yet. In a source column, U means the mechanism is a candidate.
- **-**: the source has no such mechanism.

"fin" is docs/excel-semantics/financial.md, and quoted titles are its entries.

## Sources read

| Column | Source | Version read |
|---|---|---|
| Excel | Excel's IRR (excel-graph, Microsoft 365), as fin records it | fin at this commit |
| RATE | Excel's RATE solver, fitted 782/782 bit-exact (fin "RATE is a secant on the FV residual" and its amendments) | fin at this commit |
| VB | dotnet/runtime `Microsoft.VisualBasic.Financial.IRR` and `OptPV2`, a port of the VB runtime's C code ([Financial.vb](https://github.com/dotnet/runtime/blob/fa5277b025010740ca3255dcf7a0a3e2cb681acb/src/libraries/Microsoft.VisualBasic.Core/src/Microsoft/VisualBasic/Financial.vb), IRR L236-358, OptPV2 about L900-925) | fa5277b0 |
| LO | LibreOffice `ScInterpreter::ScIRR` ([interpr2.cxx](https://github.com/LibreOffice/core/blob/6b2c10e7ca30a638afaf1d7fb5483d9e5fb06b0d/sc/source/core/tool/interpr2.cxx) L1437-1535, `SCdEpsilon` L58) | 6b2c10e7 |
| Gnm | Gnumeric `gnumeric_irr` ([fn-financial/functions.c](https://github.com/GNOME/gnumeric/blob/8d674e1fc3094f257c5a88a6f31358c5a7f114af/plugins/fn-financial/functions.c) L1302-1394) and its goal seek ([src/tools/goal-seek.c](https://github.com/GNOME/gnumeric/blob/8d674e1fc3094f257c5a88a6f31358c5a7f114af/src/tools/goal-seek.c)) | 8d674e1f |
| POI | Apache POI `org.apache.poi.ss.formula.functions.Irr` ([Irr.java](https://github.com/apache/poi/blob/519e0a3c3945364e8ea75c172fe1a15fb6d4a202/poi/src/main/java/org/apache/poi/ss/formula/functions/Irr.java)) | 519e0a3c |
| fjs | formulajs `IRR` ([src/financial.js](https://github.com/formulajs/formulajs/blob/42dbeb18b15f271b6e90d927cc972f0f06565dcf/src/financial.js) L712-860), "inspired by Apache OpenOffice" in its own comment | 42dbeb18 |
| IC | IronCalc upstream `compute_irr` ([financial_util.rs](https://github.com/ironcalc/IronCalc/blob/ae1fccd3dae89ab919e58ed299c525e6d6ba0ad5/base/src/functions/financial/financial_util.rs) L63-141), the engine's fork point | ae1fccd3 |
| Docs | Microsoft's [IRR function](https://support.microsoft.com/en-us/office/irr-function-64925eaa-9988-495b-b290-3ad0c163c1bc) page; [What's New: Changes made to Excel functions](https://support.microsoft.com/en-us/office/what-s-new-changes-made-to-excel-functions-355d08c8-8358-4ecb-b6eb-e2e443e98aac) (Excel 2010) | read 2026-11-07 |

Not reached:

- The Excel 2010 blog post "Function Improvements in Excel 2010" (blogs.msdn.com) is dead. Its archived index
  ([office2010 archive](https://learn.microsoft.com/en-us/archive/blogs/office2010/functions-and-solver-improvements-in-excel-2010))
  links to it without naming IRR.
- No Microsoft KB article on IRR `#NUM!` was found. KB Q75504 ("Using IRR with a Range that Includes Blank Cells":
  Lotus counts blanks and Excel does not) is known only from a search snippet. Its mirror (techshelps.github.io) is
  blocked by this environment's network policy.
- Lotus 1-2-3 and Multiplan `@IRR` documentation: none was found. The one candidate page (discmaster.textfiles.com)
  is blocked the same way.
- The VBA `IRR` of Office (VBE7) is not reachable through excel-graph (plan.md, T-1075).

## The mechanism table

| Mechanism | Excel IRR | RATE | VB | LO | Gnm | POI | fjs | IC | Docs |
|---|---|---|---|---|---|---|---|---|---|
| **Method** | E secant, not Newton: the error grows as (guess - root)^8 over four steps, with signs no quadratic method gives (fin "IRR is a secant in d", I!A1:A29) | E secant | E secant | C Newton, analytic derivative | C Newton (overshoot 1.000001, polish), then bracketing | C Newton, then rtsafe in a bracket | C Newton with a forward-difference derivative, then bisection | C Newton, then bisection | - "an iterative technique" |
| **Variable** | E d = 1 - v = r/(1+r); 263/263 bit-exact, the secant in v 189, in r fewer (fin "IRR is a secant in d") | r (RATE's own residual) | C r | C r | C r | C r, then y = 1/(1+r) in stage 2 | C r | C r | - |
| **Start** | E d0 = g/(1+g); g = 0.1 when omitted, 0 when written empty (fin "XIRR (and IRR): an empty guess argument") | E r0 = guess, default 0.1 | E r0 = guess (default 0.1) | E 0.1; C a guess of exactly -1 becomes 0.1 (Excel: `#VALUE!`, fin "IRR's guess range") | E 0.1 | E 0.1 | E 0.1 | E 0.1; C bounds (guess <= -1 is `#VALUE!` as in Excel, but no 2^53 bound) | E "assumed to be 0.1" |
| **Second point** | E d1 = d0 -+ 0.001, toward larger \|NPV\| (sign flips cost 121/263) | E r0 - 0.001·sign(f), up from f = 0 (fin "RATE steps up from a zero residual") | C r0 +- 1e-5, by the sign of NPV(guess) | - (derivative) | - | - | - | - | - |
| **Step anchor** | E on the newer iterate: d2 = d1 - f1(d1 - d0)/(f1 - f0) | E newer | E newer | - | - | - | - | - | - |
| **Pair update** | U: plain shift (d0, d1) <- (d1, d2) is the replica's. RATE keeps r0 when f2 = f1 (fin "RATE keeps its older iterate", T-3963, parked on owner ruling T-4239); IRR is untested | E (T-3963, parked) | plain shift | - | - | - | - | - | - |
| **NPV form** | E worksheet NPV's forward running product at x = 1/v, twice-rounded operators (T-3205); Horner in v is 189/263 (fin "IRR is a secant in d") | its own FV residual | C Horner from the last flow, anchored at the first nonzero flow | C power sum with `pow`, Kahan-summed | E forward running product (in r, `f *= 1/(1+r)`) | C power sum | E forward running product (in 1+r) | C power sum with `powi` | - |
| **Step stop** | E \|d2 - d1\| < 1e-7, absolute (1e-6 scores 169, 1e-8 146) | E \|dr\| < 1e-7 | E \|dr\| < 1e-7 | E \|dx\| < 1e-7 (`SCdEpsilon`) | C relative step < 5e-11 | E \|dx\| <= 1e-7 | C \|dr\| < 1e-9 | C < 1e-8 | E "within 0.00001 percent" (= 1e-7) |
| **Function stop** | E \|NPV\| < 1e-7, absolute: scaling the flows by 1e6 changes the result (m9-fin-irr-rate Calc!A1, A3; fin "IRR is a secant in d") | E \|f\| < 1e-7, absolute | C \|NPV\| < max\|c\|·1e-9, scaled by the flows | C none | C none (exact zero only) | C none | C \|NPV\| < 1e-10 | C none | - |
| **Zero-step stop** | E (fitted) d2 = d1 with \|NPV\| < 1e-7·max\|c\| returns, up to step 41, and is `#NUM!` from step 70 (T-3120, T-3408) | a zero step repeats to the cap (T-3963) | - | - | - | - | - | - | - |
| **Flat secant** (f1 = f0) | U: the replica fails, and Excel returns roots on 195 + 23 residual cells that end "flat" in the replica (plan.md section 6); `#DIV/0!` on some flats after a far step (fin "IRR is #DIV/0! on a flat secant after a far step", a FITTED proxy) | E hold at r1, returned under \|f\| < 1e-4 and 1 + r > 1e-4; one restart from r0 at steps 3..16 (FITTED, T-3698) | U nudge the older point one offset (1e-5) away and re-evaluate; still flat is an error | U `div_allow_zero` (IEEE inf/NaN, no guard) | U reuse the last derivative, halved; a step must cut \|f\| to 0.9 | U NaN, to stage 2 | U break to bisection | U none (NaN propagates) | - |
| **Pole** (d >= 1, r <= -1) | E d = 1 exactly is `#DIV/0!`, NPV's 1/v (fin "IRR's far-root bound", `IRR({-1,1E+300})`); U past d = 1: the replica fails | E runs on below -1; a result at or below -1 is `#NUM!` (fin "RATE holds on a flat secant and runs on below -1") | U clamp r <- (r1 - 1)/2, halfway to -1 | U none | U leaving [-1, xmax] fails stage 1 | U `1 + x = 0` gives NaN, to stage 2 | U clamp to -0.999999999 | U none (inf/NaN) | - |
| **Overflow in NPV** | E `#DIV/0!`, the call ends there, no retry, no fallback (fin "IRR is #DIV/0! where its secant cannot evaluate NPV", 59 cells) | E `^` on a negative base at integer nper | - | - | U a non-finite NPV is a goal-seek error (`#NUM!`) | - | - | U `#NUM!` | - |
| **Cap** | C the documented 20 (a cap of 20 scores 181/263); E runs of 51..200 steps return their stop iterate (fin "IRR's secant runs longer than 50 steps", 83 runs); U past 200: not a clean function of the step count | E 200 | C 40 | C 20 | C 100 (then stage 2) | C 1000 | C 1000 | C 50 | C "after 20 tries" |
| **Cap return** | U | E the 200th iterate under \|dr\|, \|f\| < 1e-4 and 1 + r > 1e-4, else `#NUM!` (T-2689) | U error | U `Err:523` | U to stage 2 | U NaN | U last iterate, never an error | U `#NUM!` | C `#NUM!` after 20 |
| **Retry** | E once from 0.1 when the run from the guess fails (fin "IRR is a secant in d", I!B13:B16) | E one restart (above) | - | - | U guess·2^k and guess/2^k, then Newton from -0.99 | - | - | C Newton from 200, then from -2 (fin "IRR's fallback when the secant fails": no secant retry from a fixed guess in -0.99..1000 fits) | - |
| **Second stage** | E it ignores the guess and reads only NPV's values, stopping at \|NPV\| < 1e-7 (fin "IRR's second stage ignores the guess", T-4844); E its returns are Newton iterates on P(v) from v = 0 on 181/405 (fin "IRR's returns where the secant fails"); E `#NUM!` at a residual >= 1e-4 (fin "IRR's fallback is #NUM! when NPV at its root is 1E-4 or more") | - (restart only) | - | - | C Ridder and bisection from a bracket that depends on the guess | U rtsafe on a fixed grid bracket (-1 + 10^-e, -0.9..1.0, 2..10000); the guess is added to the grid and the bracket holding it is preferred | C bisection from a bracket walked from the last iterate | C bisection on [-0.99999, 100] (fin: an XIRR-style bracket does not fit) | - |
| **Far-root bound** | E `#NUM!` at 1 + r >= about 1E+7 (fin "IRR's far-root bound") | E `#NUM!` at 1 + r <= 1e-7 (fin "RATE is #NUM! when 1 + r is 1e-7 or less") | - | - | U xmax = min(1e10, (DBL_MAX/1e10)^(1/n) - 1) | U grid to 10000 | U clamp at 1000 | U bracket to 100 | - |
| **Leading zero flows** | E dropped before solving (`IRR({0,-100,110})` = `IRR({-100,110})`, T-3120) | - | E skipped (`OptPV2` `While ValueArray(lLower) = 0`) | C kept | C kept | C kept | C kept | C kept | - |
| **Sign pre-check** | E no sign change is `#NUM!` | - | - (none) | - (none) | - | - | E required | E | - |
| **Guess range** | E guess <= -1 or >= 2^53 is `#VALUE!`, before the no-sign-change `#NUM!` (fin "IRR's guess range") | - | E guess <= -1 is an error | C -1 replaced by 0.1 | U domain [-1, ...] | - | - | E <= -1 is `#VALUE!` | - |
| **Error codes** | E `#NUM!` (no sign change, failure, far root), `#DIV/0!` (NPV not computable), `#VALUE!` (guess range, an error element) (fin "IRR's error order") | E `#NUM!` | - (all failures are VB runtime error 5) | C `Err:523` | E `#NUM!` | E `#NUM!` | C never `#NUM!` from the solver | E `#NUM!` | E `#NUM!` |
| **History** | E IRR is among the functions with "algorithm changes ... to improve function accuracy and performance" in Excel 2010 (Docs, what's new page); nothing says what changed | (RATE is listed too) | the VB runtime predates 2010 | | | | | | |

Notes on the table:

- **VB.** Verbatim, the VB runtime's IRR is bit-exact on 3/263 of Excel's fit set (plan.md section 6). Excel's IRR
  shares its secant skeleton (anchor, step test, leading-zero skip) but not its variable, offset, tolerance, NPV
  form or cap. Excel's IRR also shares the 2010 algorithm change with RATE, and RATE's handlers are the only ones
  evidenced on an Excel solver. That makes RATE's handlers the first candidates, and VB's the second.
- **LO** is the closest to Microsoft's documented contract (20 steps, 1e-7 on the step, `#NUM!`-style error at the
  cap), and so it shows the documentation describing a Newton-like method that Excel's evidence contradicts.
- **Gnm and fjs** share Excel's NPV form, a forward running product, but nothing else.
- **POI's second stage** is the only one in the table whose bracket comes from a fixed grid, so it reads the flows'
  NPV values much as T-4844 says Excel's does. POI still adds the guess to the grid, so it is not guess-free as
  written.
- **IC** is the engine's fork point. The engine's `compute_irr` replaced all of it (T-0916, T-2727, T-3120).
- Gnumeric's analytic derivative `irr_npv_df` weighs term i by (1+r)^-(i-1), not (1+r)^-(i+1). Every step is
  scaled by (1+r)^2. This is our reading of the source, not a Gnumeric statement, and it does not affect Excel.

## Ranked mechanisms to test at each event (for T-4836)

The ranking puts first a mechanism that is evidenced on an Excel sibling solver (RATE), then one that is present in
Microsoft's VB lineage, then one that is only in other engines. Within each tier, the plan's feasibility counts break
ties (plan.md section 6). H numbers are the plan's (section 7, Phase 2). H13 to H16 are new candidates from this
table.

### Flat secant (f1 = f0)

1. **H1, hold at d1** and return under RATE's tests (\|f\| < 1e-4, here with the far-root bound in place of
   1 + r > 1e-4). Evidenced on RATE; +109 engine misses gained in feasibility.py.
2. **H13, keep the older iterate when f2 = f1** (RATE's T-3963 finding). The pair then becomes (d0, d2), so a flat
   secant can arise only on the start pair. It needs T-4239's owner ruling before it can ship for RATE. For IRR it
   is a scoring candidate only, and its signature is the "zero step, then flat" residual (195 cells).
3. **H3, RATE's restart** from the older iterate at steps 3..16 (FITTED on RATE).
4. **H2, VB's nudge**: move the older point one offset (0.001 in d) away and re-evaluate, with a second flat as the
   error. Pair it with H10 (that error is `#DIV/0!`) against the flat-after-far-step proxy.
5. **H14, reuse the previous slope** (Gnumeric's halved derivative carried to the secant). Prior: low.
6. The replica's failure, then the retry and the second stage, as the baseline.

### Pole (an iterate at d >= 1)

Every candidate must keep the evidenced `#DIV/0!` at d = 1 exactly (`IRR({-1,1E+300})`).

1. **H4, run on past d = 1** (NPV at a negative v, as RATE runs on below -1). Evidenced on RATE; +65 over H1 at
   cap 50.
2. **H6, halve in d**: d <- (d1 + 1)/2. This is VB's pattern in IRR's own variable. It walks v toward 0, where the
   Newton-from-v = 0 iterates start, so it may explain them (H12).
3. **H5, VB's literal clamp in r**: 1 + r <- (1 + r1)/2.
4. **H15, clamp just inside the pole** (fjs's -0.999999999 in r, so d = 1 - 1e-9). Prior: low.
5. The replica's failure, as the baseline.

### Step cap

1. **H7, cap 200 and RATE's return test**: the 200th iterate under \|dd\| < 1e-4 and \|NPV\| < 1e-4. Evidenced on
   RATE; the runs of 51..200 steps already return in Excel (83 runs).
2. **Cap 200, then the second stage** (the cap as a failure that falls through to the stage of T-4844).
3. Refuted and not worth scoring again: any cap of 20 (Docs, LO), 40 (VB), 50 as Excel's (IC; the engine's cap is
   a DEVIATION, fin "The IRR and XIRR solver caps") and a cap that returns an error with no second stage.

### Across events

- **Stop scaling (H8).** VB's flow-scaled tolerance (max\|c\|·1e-9) is contradicted for the main stop, which is
  absolute (m9-fin-irr-rate Calc!A1, A3). It stays a candidate for the stall and the cap-return tests only, where
  the engine already scales the zero-step stop by max\|c\|. Test it with x2^k ladders (T-4839).
- **Second stage (H12).** Rank: Newton on P(v) from v = 0 (E on 181/405 cells), then H6 as its possible shadow, then
  **H16, POI's fixed-grid bracket with a safeguarded Newton**, with the guess left out of the grid so the stage is
  guess-free as T-4844 requires.
- **NPV form (H9).** Every source that shares Excel's forward running product (Gnm, fjs) evaluates it in plain
  doubles. None of them offers a lower-noise form for the long series. H9 has no lineage support and stays a Phase 4
  question.
