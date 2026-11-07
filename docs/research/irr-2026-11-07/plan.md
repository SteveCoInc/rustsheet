# IRR deep research plan (2026-11-07)

A Fit Lab research plan for IRR, the lowest-accuracy function in the engine. It covers what IRR is, what its arguments
can be, where the engine stands and why the earlier fit work plateaued. Then it gives a phased plan to reverse engineer
the parts of Excel's computation that are still unknown. The numbers come from a full `rustsheet-eval run` at
`d13a3dec` on 2026-11-07 and from `feasibility.py` in this directory, which reproduces them in about 5 seconds:

    cd tools && uv run python -I ../docs/research/irr-2026-11-07/feasibility.py

## 1. Bottom line

- IRR is 79.28% within 15 digits over 3,129 attributed cells (docs/progress/functions.md). The next-worst financial
  function, RATE, is 93.57%.
- **Most of the gap is not digits. It is the answer's type.** Of the 727 IRR cells off Excel, 575 return the wrong
  kind of result: Excel returns a root where the engine is `#NUM!` (491) or `#DIV/0!` (11), Excel is `#DIV/0!` where
  the engine is `#NUM!` (44), and so on. 112 are visibly-off numbers and 38 are last-digit misses.
- Almost all of it is in the adversarial probe sweeps earlier tasks built on purpose, not in real workbooks:
  gen-m9 and ironcalc-calc have 8 and 2 off cells. The fetched suites (calipers, corpus-bench, poi) were not in this
  run. They hold about 62 IRR cells (3,551 on the progress page against 3,489 here), so they cannot hold much of the
  gap.
- The core is solved. Excel's secant in d = 1 - v is bit-exact on all 263 fit-set cells (financial.md "IRR is a secant
  in d"). What is unknown is what Excel does when that secant hits one of three events the engine treats as dead ends:
  a **flat secant**, an **iterate past the pole** (d >= 1), and the **step cap**. The engine's answers there are
  invented. It has a Newton fallback from v = 0 and six range-limited FITTED cuts layered on top.
- Two new leads, both checked here without the oracle (section 6):
  1. **RATE's driver.** Excel's RATE solver, already fitted bit-exact on 782/782, has fully worked-out failure
     handling (hold on a flat secant, a 1e-4 return test, run on past -1, a 200-step cap). Put on IRR's secant
     unchanged, it gives Excel's type on 1,990 of 2,555 IRR probe cells. The engine gives 2,000, with its fallback and
     six cuts fitted on these same cells. The transplant has no IRR-specific constants and gets 118 cells the engine
     misses.
  2. **VB runtime lineage.** Microsoft's open-source `Microsoft.VisualBasic.Financial.IRR` (dotnet/runtime) is a
     sibling of Excel's IRR, not a copy: verbatim it is bit-exact on only 3/263. Its failure handlers (nudge on a flat
     secant, halve the distance to the pole, an NPV tolerance scaled by the largest flow) are the obvious candidates
     for the events we have not observed.
- The bullseye is **long series**. 399 of the 434 cells that neither the engine nor any candidate gets right are series
  of 21+ flows where Excel returns a root and the replica stalls (a zero step, then a flat secant: 195) or runs out of
  steps (181). The secant was fitted on 2 to 41 flows. Something about Excel's long-series arithmetic or stop test
  differs.
- The plan replaces sweep-and-score with **discriminating probes**: inputs built so that the surviving hypotheses
  disagree, triggering one chosen event at one chosen step, plus path-invariant transforms (power-of-two scaling,
  negation, zero padding) that isolate one mechanism at a time.
- Targets (section 9): IRR type agreement on the probe cells from 78% to at least 95%, IRR within 15 digits from
  79.3% to at least 97% (stretch 99%), and the six range-limited cuts down to at most two.

## 2. What IRR is

IRR(values, [guess]) is the **internal rate of return** of a series of periodic cash flows: the rate r at which their
net present value is zero,

    NPV(r) = sum over i = 0..n-1 of  c_i / (1 + r)^i  = 0.

It is the standard yardstick for comparing investments: a project whose IRR exceeds the cost of capital creates
value. The conventions:

- c_0 is at the end of the first period, like worksheet NPV, though the root does not depend on that. Flows are
  equally spaced. XIRR takes dates instead. MIRR is the closed-form cousin with explicit finance and reinvestment
  rates.
- A **conventional** series (one outlay, then inflows: one sign change) has exactly one root in (-1, inf). A series
  with several sign changes can have several roots (Descartes' rule) or none, and then which one IRR returns depends on
  the guess and on the solver's path.
- NPV in v = 1/(1+r) is a polynomial, P(v) = sum c_i v^i. IRR asks for a positive root v of P. No closed form exists
  past degree 4, so Excel iterates, and the details of that iteration (start, variable, step, stop, cap, failure
  handling) decide the answer's last digits and whether there is an answer at all.

Microsoft documents the function as: "Excel uses an iterative technique ... Starting with guess, IRR cycles through
the calculation until the result is accurate within 0.00001 percent. If IRR can't find a result that works after 20
tries, the #NUM! error value is returned." Only the 0.00001 percent (the 1e-7 step test) is true. The cap is not 20:
runs of 200 steps return (T-3408).

## 3. The arguments and their possible values

Everything below is evidenced unless marked documented. "financial.md" is docs/excel-semantics/financial.md.

### values

| Form | Behaviour | Evidence |
|---|---|---|
| Array constant `{-100,60,60}` | the flows, in order | fit set |
| Range `A1:F1`, 2-D range | row-major flows | gen-m9, IRR.xlsx |
| Computed array (`A1:A3+1`, `UNIQUE(..)`) | the elements | fin-irr-error-array |
| Single number (`IRR(5)`) | `#NUM!` (no sign change) | irr-guess-edges |
| Text or TRUE in an array or range | skipped: `IRR({-100,"a",60})` = `IRR({-100,TRUE,60})` = -0.40000000000000013 | "IRR over an error element" |
| Empty cells in a range | ignored (documented) | Microsoft docs |
| An error element (array or range) | `#VALUE!`, whatever the error | "IRR over an error element" |
| The argument itself an error (`IRR(1/0)`) | that error | "IRR's error order" |
| CELL("row"), ERROR.TYPE results (the number mark) | `#VALUE!` where `IRR(ROW(..))` is `#NUM!` | "IRR's error order" |
| Leading zero flows | dropped before solving (`IRR({0,-100,110})` = `IRR({-100,110})`) | T-3120 |
| All one sign, all zero, a single flow | `#NUM!` | edge-pool smoke check |
| Very many flows | fine in Excel; the engine charges a work cap (DEVIATION) | "NPV, IRR and MIRR cash-flow work cap" |

### guess

| Form | Behaviour | Evidence |
|---|---|---|
| Omitted | 0.1 (bit-identical to an explicit 0.1) | "IRR, RATE and XIRR stop early" |
| Written empty `IRR(v,)` or an empty cell | **0**, not 0.1 (XIRR differs) | "XIRR (and IRR): an empty guess argument" |
| `""` | `#VALUE!` | same |
| <= -1 or >= 2^53 | `#VALUE!`, before the no-sign-change `#NUM!` | "IRR's guess range" |
| An error | that error, after an error in values itself | "IRR's error order" |
| An array or multi-cell range | lifts: one result per element (spills) | "IRR's error order" |
| Any other number | the secant's start: changes the digits (beyond 1e-12) and sometimes which root | guess sweeps |

### Results

A number (an un-converged secant iterate, typically 1e-12 relative from the root), `#NUM!` (no sign change, no
convergence, a root past 1 + r = 1E+7, the unknown cases), `#DIV/0!` (an NPV the secant cannot evaluate, and
unexplained cases), `#VALUE!` (the argument errors above).

### The input space for research

The axes along which IRR's behaviour has been seen to change, and which every new probe should stratify:

1. **n**, the number of flows (2..200 seen). The residual sits at 21+ flows.
2. **Sign pattern**: conventional, sign changes in the middle, a positive first flow.
3. **Absolute scale** of the flows (1E-15..1E+82 seen). The stop tests are absolute, so scale changes the type.
4. **Dynamic range** between flows (a 1E+15 outlay against flows of 1).
5. **Root location**: near -1, near 0, near 0.1, far (up to 1 + r = 1E+7).
6. **Guess** relative to the root, and whether it is 0.1 (the retry's start).
7. **Structure**: level annuities, zero runs, leading or trailing zeros.

## 4. Where the engine stands

### The failures by kind (full eval, 2026-11-07)

| Excel | Engine | Cells |
|---|---|---:|
| number | `#NUM!` | 491 |
| number | number, visibly off | 113 |
| `#DIV/0!` | `#NUM!` | 44 |
| number | number, 1 to 4 ulp | 34 |
| `#NUM!` | number | 27 |
| number | `#DIV/0!` | 11 |
| other | | 7 |
| **total** | | **727** |

The largest cases: t2927-irr-fallback-sweep 122 (101 of them digits), t3411-irr-r4-fresh-4 63, t3347-irr-r4-fresh 60,
t3411-irr-r4-fresh-2 57, t3406-irr-num-boundary 54, t3411-irr-r4-fresh-3 51, t3411-irr-r4-fresh 51, t4213-irr-r4-fresh
42, t3478-irr-r4-wide 37, t3408-irr-r4-fresh 35, t4027-irr-annuity-scale-guess 33. Eight of these eleven are fresh
hold-out sweeps (decision 0188 R4.2). Those sweeps are exactly where an overfit model shows.

### What is known

- **Bit-exact (VERIFIED):** the secant in d = 1 - v from d0 = g/(1+g), d1 = d0 -+ 0.001 toward larger |NPV|. NPV is
  worksheet NPV's running product at x = 1/v with twice-rounded operators. The step is anchored on the newer iterate.
  It stops at |dd| < 1e-7 and |NPV| < 1e-7, drops leading zeros, and retries once from 0.1. This is 263/263 on the fit
  set and 180/181 on t2565.
- **Partly known:** the far-root bound (1 + r < 1E+7); `#DIV/0!` where NPV overflows (T-3407); runs of up to 200
  steps finish on their own digits (T-3408); a zero-step stop at |NPV| < 1e-7 of the largest flow (T-3120); the 1e-4
  residual cut on fallback roots (T-3411).
- **Unknown:** what Excel does at a flat secant, at an iterate past d = 1, and past about 200 steps. That decides
  number versus `#NUM!` versus `#DIV/0!`, and which digits come back.

### The model the engine ships

`compute_irr` (crates/rustsheet-engine/src/functions/financial/financial_util.rs) is the secant, the retry, then
`irr_fallback` (Newton on P(v) from v = 0). On top of these sit six FITTED, range-limited constants, each patching
one symptom:

| Constant | What it patches |
|---|---|
| `IRR_FALLBACK_STEPS` = 50 | the fallback's cap (Excel's lies between 29 and 119, or is no cap at all) |
| `IRR_ROOT_RESIDUAL` = 1E-4, `IRR_ROOT_RESIDUAL_FLOWS` | fallback roots Excel rejects |
| `IRR_FLAT_DIV_ZERO_NPV` = 3E+226, `IRR_FLAT_DIV_ZERO_FLOWS` | flat secants Excel reports as `#DIV/0!` ("a proxy, not Excel's mechanism") |
| `IRR_LONG_RUN_STEPS` = 200, `IRR_LONG_RUN_RANGE` | long runs ("no Excel cap") |
| the zero-step stall bound (1e-7 of the largest flow, first 50 steps) | stalled runs Excel returns |
| `IRR_FAR_ROOT_BOUND` = 1E+7 | far roots (this one looks like a real Excel rule) |

## 5. Why progress stalled

Earlier tasks were careful and well evidenced. Six patterns explain why they plateaued:

1. **The dead ends were invented.** The replica fails at a flat secant, at d >= 1 and at its cap, because the first
   fit had no evidence there. Every later task patched what came out of those invented failures (the Newton fallback,
   the 1e-4 cut, the 3E+226 proxy, the 200-step range) instead of observing Excel at the events themselves.
2. **Accretion is the symptom of a wrong model.** Each new cut is fitted, range-limited (R4.3), wins its own sweep
   and then loses part of the next fresh one. The fresh sweeps keep failing at 20 to 40% on exactly these cells.
3. **Sweep-and-score is inefficient.** Random or stratified sweeps mostly land where every candidate agrees. A
   300-cell sweep may hold ten cells that separate two hypotheses. T-3698 (RATE's restart) shows the better method:
   choose cells because two rules disagree on them.
4. **Some evidence was weaker than it looked.** "181 of 405 fallback returns are Newton iterates from v = 0", but on
   123 of the 181 the iterate is the converged double, which any method that reaches the root also returns. Only
   about 58 cells actually discriminate.
5. **The fit set does not cover the residual.** The secant was fitted on 2 to 41 flows. 478 of the unresolved cells
   have more than 20 flows, and most have far more.
6. **Sibling solvers were fitted alone.** RATE's failure handling was solved (T-2689, T-3348, T-3698) and never tried
   on IRR. Both share the 0.001 start offset, the 1e-7 tests and a 1e-4 secondary cut, which suggests one shared secant
   driver inside Excel.

## 6. The new leads, measured

`feasibility.py`: the outcome type on all 2,555 literal IRR cells of the 26 IRR probe cases, NPV in plain doubles. The
engine on main gives Excel's type on **2,000**.

| Candidate (all on the secant in d, with the retry) | Excel's type | Engine misses gained | Engine hits lost |
|---|---:|---:|---:|
| base: the three events as failures, no fallback, cap 50 | 1,709 | 24 | 315 |
| base, cap 200 | 1,754 | 25 | 271 |
| VB: r-half clamp at the pole, cap 200 | 1,862 | 25 | 163 |
| VB: d-half clamp, cap 200 | 1,796 | 26 | 230 |
| VB: flat nudge + r-half clamp, cap 200 | 1,874 | 38 | 164 |
| RATE: hold on flat, cap 50 | 1,868 | 109 | 241 |
| RATE: hold on flat + run on past d = 1, cap 50 | 1,933 | 109 | 176 |
| **RATE: hold on flat + run on past d = 1, cap 200** | **1,990** | **118** | **128** |

How to read it:

- The RATE transplant uses only constants fitted on RATE. Against it, the engine is scored in sample: its cuts were
  chosen on these very cases. So 1,990 against 2,000 is a strong result for a model with no IRR-specific failure rule.
- The VB clamp mostly recovers the cells the engine's Newton fallback recovers: with the flat nudge added, both
  are right on 1,836 cells, and the VB variant adds 38 the engine misses. So the "Newton from v = 0" fallback may
  be the shadow of a simpler mechanism at the pole. One candidate is H6: halving the distance to d = 1 halves v at
  each pole crossing, walking v toward 0, where that Newton search starts.
- VB verbatim (secant in r, offset 1e-5, 40 steps, NPV tolerance max|c|·1e-9): 3/263 bit-exact on the fit set, 197
  within 1e-12. A sibling, not Excel's code.

**The residual.** 434 cells are typed wrong by every candidate and the engine:

| Excel | How the replica's run from 0.1 ends | Flows | Cells |
|---|---|---|---:|
| number | zero step, then flat | 21+ | 195 |
| number | step cap (200) | 21+ | 181 |
| number | flat | 21+ | 23 |
| `#DIV/0!` | flat, or flat after a zero step | 21+ | 14 |
| `#DIV/0!` | iterate at d >= 1 | 2-20 | 7 |
| other | | | 14 |

A "zero step, then flat" run means NPV's rounding noise is above the 1e-7 stop where the secant can no longer move.
Excel returns a root there, so on long series either its NPV is less noisy, its stop test is not absolute, or its
steps differ. That is Phase 4's question.

## 7. The plan

Eight phases. Phases 1 and 2 need no oracle. Each phase ends at an explicit go or no-go. Every fit follows the fit lab's
hold-out rule (decision 0188 R4.1) and is frozen before a fresh sweep (R4.2). Every Excel finding goes into
financial.md in the same PR (CLAUDE.md directive 6).

### Phase 0: Tooling (size M, no oracle)

1. **`irr-type` fit.** Move feasibility.py into the fit lab (`kernels/irr_driver.py`). It runs on the exact replica
   (twice-rounded `X87` operators, as `irr_secant_d`) and scores type and bits on every IRR probe case, errors
   included, with the hold-out split. The fit lab today scores IRR's secant and its fallback on separate small sets.
2. **Event tracer.** Every candidate exposes `trace` (fitlab "Iterate traces") and also reports the event that ended
   each run (converged, flat, zero step, pole crossing, NPV overflow, cap) and its step. This gives per-cell answers to
   "Excel returns, the replica ends on event E at step k".
3. **Discriminating-probe designer** (`fitlab/irr_discriminate.py`). Given candidates A and B and a generator over
   the axes of section 3, it searches flows and guesses (random draws, then a local hill-climb on the flows) for inputs
   where A and B differ in type, or in value by more than the 15th digit. It writes a spec case with the inputs and
   each candidate's prediction committed before the fill, as docs/research/t4027/predictions.md did.
4. **Path-invariant transforms** for any flow set:
   - **x2^k scaling** keeps every iterate of a scale-invariant method bit-identical, while absolute tests (|NPV| <
     1e-7, 1e-4) see a 2^k-times larger residual. A type change along a 2^k ladder isolates an absolute threshold
     (T-3117 asked for this; it is still open).
   - **Negation**: is the path symmetric in the sign of NPV? T-2927 found it is not.
   - **Leading-zero padding** must be invisible.
   - **Trailing-zero padding** is not: it changes n and the noise floor without changing the root, the cleanest test
     of an n-dependent regime.
   - **Guess injection**: `IRR(flows, g)` with g a replica iterate tests whether Excel's path passes through that
     state.
   - **The fixed-point test** `IRR(flows, Excel's own result)` (below).

Exit: the `irr-type` fit reproduces section 6's table on exact arithmetic, and the designer finds separating inputs
for a known pair (the base model against the RATE transplant).

### Phase 1: Lineage desk research (size S, no oracle, parallel with Phase 0)

Build a **mechanism table**: rows are the solver's mechanisms (variable, start, offset, step anchor, stop tests and
their scaling, flat handling, pole handling, cap and cap-return, error codes, leading zeros), columns are the sources.
Mark each cell as Excel-evidenced, contradicted or unknown.

| Source | What to extract | Status |
|---|---|---|
| dotnet/runtime `Microsoft.VisualBasic.Core/.../Financial.vb` (IRR, Rate, OptPV2, LEvalRate) | flat nudge, the (r1-1)/2 pole clamp, the epsilon scaled by max\|c\|, leading-zero skip in OptPV2, Horner from the last flow | read (section 6); Rate's skeleton matches Excel's RATE |
| The VBA `IRR` of Office (VBE7) | the "C code" Financial.vb was ported from; if a desktop oracle is ever available, VBA's IRR is a second, cheaper oracle for the lineage (T-1075) | not reachable via excel-graph |
| Excel's fitted RATE (financial.md) | hold on flat, restart rule (T-3698), run on past -1, cap 200 with a 1e-4 return | fitted 782/782 |
| Excel's fitted XIRR (bisection) and YIELD (Newton, 100-step cap) | the other solvers' caps and error codes | fitted |
| LibreOffice `ScInterpreter::ScIRR` (sc/source/core/tool/interpr2.cxx) | Newton in r, 20 steps: a non-Excel baseline | read: lineage.md (T-4835) |
| Gnumeric `irr` (plugins/fn-financial, goal_seek) | Newton, then bracketing and "trawl" fallbacks: what a second stage can look like | read: lineage.md (T-4835) |
| Apache POI `Irr`, formulajs, IronCalc upstream | what other engines replicate (all Newton) | read: lineage.md (T-4835) |
| Lotus 1-2-3 / Multiplan `@IRR` docs, Microsoft KB articles on IRR `#NUM!`, "Function Improvements in Excel 2010" | historical constants (20 iterations, 0.00001%) and whether 2010 touched IRR | searched: lineage.md (T-4835) |

Exit: the table, with a ranked list of mechanisms to test at each of the three events.

### Phase 2: The hypothesis bank, offline (size M, no oracle)

Score every hypothesis and every sensible combination with the `irr-type` fit: type, bits, hold-out verdict, cells
gained and lost against main. Most of the work stays offline: one run of the whole bank takes seconds.

| ID | Hypothesis | Source | First signature to look for |
|---|---|---|---|
| H1 | Flat secant: hold at d1, return under RATE's tests (\|f\| < 1e-4, 1 + r > 1e-4) | RATE T-3348 | +109 in feasibility |
| H2 | Flat secant: nudge the older iterate one offset (0.001) away and retry the step once | VB, RATE T-3698 restart | cells where hold and nudge differ |
| H3 | Flat secant: restart a fresh run from the older iterate (RATE's restart window, steps 3..16) | RATE T-3698 | |
| H4 | Pole: run on past d = 1 (NPV at a negative v) | RATE "runs on below -1" | +65 over H1 at cap 50 |
| H5 | Pole: halve the distance in r, 1 + r <- (1 + r1)/2 (v doubles) | VB | |
| H6 | Pole: halve in d, d <- (d1 + 1)/2 (v halves; walks to v = 0, Newton-like) | VB pattern in IRR's variable | does it explain the "Newton from v = 0" iterates? |
| H7 | Cap: a fixed cap (200, as RATE) and return the last iterate under the 1e-4 tests | RATE T-2689 | the 189 step-cap residual |
| H8 | NPV tolerance scaled by the flows (VB: max\|c\|·1e-9) instead of the absolute 1e-7 | VB | x2^k ladders flip the type at a scale that tracks max\|c\| |
| H9 | Long series: another NPV evaluation (Horner in v, x87-extended accumulation, the sum in another order) that lowers the noise floor | the long-series residual | the zero-step stalls disappear on 21+ flows |
| H10 | `#DIV/0!` is the error code of one event (flat after a nudge, or an NPV overflow), `#NUM!` of another (cap) | VB Rate's `CalcDivByZero`; T-3407 | one event per code on the 44 + 11 cells |
| H11 | No retry from 0.1 after some failures (Excel's retry is conditional) | T-4027's guess cells | |
| H12 | The Newton-from-v=0 stage is real, but entered only from one event | T-3411 | the 58 discriminating iterate cells |

Exit: a ranked shortlist of at most four complete models, each with its predicted type and value on every probe cell
and the list of cells where they disagree.

### Phase 3: Engineered-event probes (size M per round, oracle: about 2 to 4 cases of 100 to 200 cells)

For each event, build flows that hit it on a chosen step in the replica, where the shortlisted models disagree.

1. **Flat at step k** (k = 1, 2, 3..16, 20+). Choose flows where NPV(d0) and NPV(d1) are bit-equal (symmetric flows
   about the start, or flows scaled until the two NPVs round alike). Compare hold, nudge, restart and fail.
2. **Zero step, then flat**, at chosen residuals |f| from 1e-8 to 1e-2, crossed with x2^k ladders: is the acceptance
   absolute (H1), or scaled by the flows (H8)?
3. **Pole crossing at step 1 or 2**: flows whose first secant step lands at d slightly above 1, exactly 1, and far
   above 1. Compare run on, the r-half and d-half clamps, and fail.
4. **NPV overflow** at a chosen step, against a pole crossing at the same step, for H10's error-code mapping.
5. **Cap**: runs that need 190 to 260 steps (between 200 and 216 the outcome was unclear, T-3408), with x2^k ladders
   for the 1e-4 return tests.

Each case carries its predictions per model, committed before the fill. A round eliminates models. Two rounds should
leave one.

### Phase 4: The long-series regime (size M to L, oracle: about 2 cases)

The 399-cell bullseye. The questions, in order of cost:

1. **Is there a regime boundary in n?** Take 10 residual series and the same shape (root, scale, dynamic range) at
   n = 10, 15, 20, 25, 30, 40, 60, 100 and 150, by trailing-zero padding and by stretching the annuity. A type change
   at a fixed n points to an algorithm switch; a smooth change points to noise.
2. **The fixed-point test.** For each residual cell, `IRR(flows, Excel's result)`. If Excel returns the same bits, its
   result passes its own stop test there. The replica's |NPV| at that rate then tells whether Excel's NPV is less
   noisy (H9) or its test looser (H8).
3. **NPV's bits on long series.** Excel's worksheet NPV shares IRR's arithmetic on short series (T-3205). Probe
   `NPV(r, flows)` at the replica's stall points on 21+ flows. If worksheet NPV's bits there are the replica's, H9 is
   refuted for NPV and the difference is in the solver.
4. **Guess injection along the replica's path**: at which step do Excel and the replica part?

Exit: a model that types at least 80% of the 399 right on the hold-out side, or a documented refutation of H8 and H9
with a narrowed next hypothesis.

### Phase 5: The error codes (size S)

With the path known, map each terminal event to Excel's code (H10). This retires `IRR_FLAT_DIV_ZERO_NPV` (its own doc
calls it "a proxy, not Excel's mechanism") and explains `IRR({-1,1E+15})` (T-3900) and t2657-pv-irr-div0's 22 cells.

### Phase 6: The last digits (size M)

Only after the type is right, because the digits of a failed path are noise:

- the 113 visibly-off and 34 ulp cells, re-scored under the new model (t2927's 101 digit cells first);
- near-zero roots (T-4213, T-4232) and the fallback digits (T-2928), which may simply disappear if the fallback is
  replaced;
- the existing arithmetic levers (twice-rounded operators, NPV order) on what is left.

### Phase 7: Consolidate (size M)

1. Freeze the model and draw a fresh sweep (decision 0188 R4.2) with a new seed of docs/research/t3411/gen_sweep.py,
   stratified on section 3's axes with a long-series quota.
2. Gate A: accuracy/IRR tables, holdout only.
3. Delete each FITTED constant the model subsumes and tag what stays, with its range.
4. Rewrite financial.md's IRR sections into one rule ("IRR's secant and its failure handling") and mark the old
   sections superseded.
5. A decision record if the change moves cells both ways (decision 0083's allowance).

## 8. Method rules for this campaign

- **Predictions before fills.** Every probe case commits each model's predicted value per cell before the oracle run.
  A model that is right only after the fact is not evidence.
- **Choose cells for disagreement**, never for coverage alone. A sweep is for the final freeze test.
- **One mechanism per probe.** A probe varies one axis, with every other axis held where the models agree.
- **Use path invariance.** Prefer x2^k over x10 (decimal scaling perturbs every iterate). Prefer trailing-zero padding
  over new flows.
- **Hold-out discipline.** Fit on train, report test. The fresh sweep comes after the freeze.
- **Time box.** Per decision 0056, each phase has its budget. A phase that misses its exit records the refutations in
  financial.md and stops, rather than adding a cut.
- **No new range-limited cuts** unless a phase exit forces one. A cut needs a decision record naming the mechanism it
  stands in for.

## 9. Targets

| Metric | Today | Target | Stretch |
|---|---:|---:|---:|
| IRR within 15 digits (functions.md) | 79.28% | 97% | 99% |
| Type agreement, the 2,555 IRR probe cells | 2,000 (78.3%) | 2,430 (95%) | 2,530 (99%) |
| IRR cells off Excel (eval) | 727 | under 150 | under 40 |
| FITTED range-limited IRR constants | 6 | 2 | 1 (the far-root bound) |
| Regressions elsewhere | | 0 | 0 |

The realistic risk: the long-series regime may come down to Excel's exact NPV noise at the last bit on 100+ flows, which
the oracle can see only through outcomes. If Phase 4 ends there, the plan still lands Phases 2, 3 and 5, about 150 to
250 cells from the feasibility numbers, and records the long-series behaviour as the remaining open quirk.

## 10. Tasks

Filed in docs/tasks/M9 (lane functions-financial, T-4835 research). The long-series phase is split in two.

| Task | What | Size | Depends on | Absorbs or supersedes |
|---|---|---|---|---|
| T-4834 | Fit lab: `irr-type` fit, event tracer and path-invariant transforms | M | | the feasibility script |
| T-4835 | IRR solver lineage table (Phase 1) | S | | |
| T-4836 | IRR hypothesis bank, offline (Phase 2) | M | T-4834, T-4835 | T-3117's analysis part |
| T-4837 | Discriminating-probe designer | M | T-4834 | |
| T-4838 | Engineered-event probes, round 1 (flat, pole) | M | T-4836, T-4837 | T-1686 (IRR side), T-4027 |
| T-4839 | Engineered-event probes, round 2 (cap, scale ladders) | M | T-4838 | T-3117, T-4038 |
| T-4844, T-4845 | The long-series regime (Phase 4) | M each | T-4838 | T-3548, T-3347 |
| T-4840 | Implement the chosen driver in `compute_irr`, retire subsumed cuts | M | T-4838, T-4839 | T-3548, T-4027 |
| T-4841 | IRR error codes from terminal events (Phase 5) | S | T-4840 | T-3900, T-3407's proxy |
| T-4842 | IRR last digits under the new model (Phase 6) | M | T-4840 | T-2131, T-2928, T-4213, T-4232, T-4338 |
| T-4843 | Freeze, fresh sweep, Gate A, financial.md rewrite (Phase 7) | M | T-4840, T-4841 | |

T-4834, T-4835 and T-4837 can run in parallel. The first oracle spend is T-4838.

## Appendix A: Microsoft.VisualBasic.Financial.IRR in brief

From dotnet/runtime `src/libraries/Microsoft.VisualBasic.Core/src/Microsoft/VisualBasic/Financial.vb`, a port of the VB
runtime's C code (its comments say so):

- NPV (`OptPV2`): skip leading zero flows, then Horner from the last flow, `total = total / (1 + r) + c_i`.
- Tolerance: `eps = max|c_i| * 1e-7 * 0.01` (relative to the largest flow).
- Start: r0 = guess, r1 = r0 + 1e-5 if NPV(r0) > 0, else r0 - 1e-5.
- Up to 40 secant steps, `r0 = r1 - (r1 - r0) * f1 / (f1 - f0)`:
  - **flat** (f1 = f0): move r0 one step (1e-5) away from r1 and re-evaluate; still flat is an error;
  - **pole** (r0 <= -1): `r0 = (r1 - 1) * 0.5`, halfway from r1 to -1;
  - **stop**: |NPV| < eps and |dr| < 1e-7.
- After 40 steps: an error.

`Rate` has the same skeleton, with r1 = r0/2 or 2·r0 and a stop on |f| < 1e-7 alone. Excel's fitted RATE shares the
skeleton with other constants (0.001 offset, 200 steps). Excel's IRR shares the leading-zero skip and the 1e-7 step
test, but runs in d with a 0.001 offset.

## Appendix B: Reproducing the numbers

- The failure table of section 4: `rustsheet-eval run --examples 2000`, then count report.json's cells whose formula
  calls IRR, by verdict and by expected and actual type.
- Section 6: feasibility.py (header for the command). It needs the tools venv and the release harness binary for the
  engine column.
- VB verbatim on the fit set: the algorithm of Appendix A on the literal IRR cells of probes-numeric/m9-fin-irr-rate
  and probes/solver-iteration-traces gives 3 bit-exact cells of 263 (a scratch run, not committed).
