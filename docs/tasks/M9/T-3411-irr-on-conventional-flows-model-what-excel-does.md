---
id: T-3411
title: "IRR on conventional flows: model what Excel does after its secant fails (Newton from v = 1 and the T-3406 candidates are refuted)"
milestone: M9
lane: functions-financial
status: done
priority: P0
size: M
tier: opus
depends_on: [T-3406]
campaign: semantics
---

## Goal
IRR returns Excel's type (number, `#NUM!` or `#DIV/0!`) on conventional cash flows where the fitted secant fails,
from a model of what Excel does there that holds on a fresh sweep. It replaces or reworks T-3347's `irr_conventional`
stage (PR #2529, blocked on its hold-out) and unblocks T-3347.

## Context
- Evidence: probes/t3406-irr-num-boundary (144 cells; financial.md "IRR on conventional flows where the secant fails:
  which return is path-sensitive"), plus T-3347's probe, sweep and hold-out (on its branch). The engine on main fails
  76 of the 144: level annuities `IRR({-P,1,...,1})` from r = -0.10 (n = 20), -0.04 (n = 30) and -0.02 (n = 40, 60)
  are `#NUM!` where Excel returns the root. That is an ordinary input, visibly wrong.
- Refuted (T-3406, type agreement on 450 cells): Newton on P(v) from v = 1 under any cap; the secant from 0.1 under
  a 50, 100 or 200-step cap (295, 301, 306); the secant carried past a flat step by a Newton step (304, 311, 317); a
  retry from any of 17 fixed guesses. The second family of the probe (an outlay scaled by 0.97 to 1.03) alternates
  between a number and `#NUM!`, so the decision depends on the exact path of an iteration: a model has to reproduce
  Excel's arithmetic step by step, as the secant already does with the engine's twice-rounded operators.
- Ideas not yet tried: run the T-3406 candidates through the engine's own `irr_npv` (extended operators) rather than
  a double replica, since a flat step in double may not be flat there; a secant in r rather than d after the first
  failure; a bisection or bracket walk like XIRR's (financial.md "XIRR is a bracketing bisection"); Excel's NPV
  overflow surfacing as `#DIV/0!` (T-3407) as a sign of which iteration runs.
- Type cells of T-3347's hold-out (Excel returns the root, the engine is `#NUM!`; gap triage 2026-11-01, `make eval` on
  origin/main 408444f20):
- probes-numeric/t3347-irr-conventional-holdout.case.json (30): P!A1, A3, A7, A13, A18, A21, A23, A39, A44, A45, A46, A47, A49, A50, A53, A54, A56, A57, A61, A66, A67, A69, A71, A90, A95, A103, A105, A107, A110, A119
- Decision 0188 R4: state the hold-out split and margin before scoring; fill a fresh sweep after the model is frozen.

## Acceptance criteria
- [ ] (Partly met; the rest moved to T-4027.) A model that gives Excel's type on every cell of probes/t3406-irr-num-boundary and T-3347's three cases
      (T-3407's `#DIV/0!` cells may stay `#NUM!` if T-3407 has not landed), with the R4 hold-out and a fresh
      probes-numeric sweep scored per stratum: 0 type mismatches, Gate A envelope.
- [x] The engine implements it, for the residual cut (in T-3347's PR, or a new one replacing it), with the rule in
      docs/excel-semantics/financial.md.
- [x] `make check` green; reviewers approve.

## Out of scope
- The last digits (T-3408) and `#DIV/0!` (T-3407).

## R4 plan (fixed 2026-11-02 before the fresh sweep is generated or scored)
- Frozen model (commit "T-3411: freeze"): `irr_secant` runs up to `IRR_SECANT_STEPS` = 220 steps (was 50) and a
  stalled run (a zero step, or a flat secant after a step under 1e-7) returns its iterate when |NPV| < 1e-4 absolute
  (`IRR_STALL_RESIDUAL`; was 1e-7 of the largest flow, zero steps only). Both constants are bracketed by named
  cells, not chosen by maximizing a score: cap 213..=228 (t3347 holdout A90 stops exactly at step 213; t3478 r4-wide
  A240 would stop at 229 on digits Excel does not return), residual 8.4E-5..2.9E-4 (t3406 A123 returns; r4-wide A52
  is `#NUM!`).
- No hold-out split was fixed before the bracketing: every IRR case in eval was used. The fresh sweep below is the
  out-of-sample test.
- Fresh sweep: probes-numeric/t3411-irr-r4-fresh, 150 cells from docs/research/t3411/gen_sweep.py (seed
  "T-3411/r4-fresh"), generated after the freeze, oracle-filled from excel-graph, 50-digit roots in its math.tsv.
- Margin, stated now: on the fresh sweep the frozen model must (1) agree with Excel on the type (number vs error) on
  at least as many cells as main's model (cap 50) and (2) have a bit-exact rate within 10 points of its rate on
  t3478-irr-r4-wide (257/300 = 85.7%), the sweep that is generated the same way. Otherwise the model is not shipped.

## R4 round 1 result (fresh sweep t3411-irr-r4-fresh, scored once on the frozen model)
- Main's model 80/150, frozen model 93/150 (15 fixed, 2 new digit misses). Criterion (1) passes (type mismatches
  56 -> 44); criterion (2) fails: 62% is not within 10 points of 85.7%. Per the plan, the round-1 model is not shipped.
- The sweep refutes cap 220: A55's run from guess -0.99235 would stop at its 215th step on digits Excel does not
  return (Excel's are the retry's), so Excel's cap is 213 or 214.
- It refutes the stall rule as Excel's mechanism: A129 goes flat at |NPV| 2.3E-5 and Excel's digits are those of
  `irr_fallback` (Newton on the NPV polynomial from v = 0), not of the flat iterate. That fallback, uncapped, gives
  Excel's exact digits on 15 of the 22 stalled cells (up to 179 steps), but no fallback cap separates Excel's
  returns from its errors (returns and `#NUM!` interleave from 0 to 680 steps over the 285 cells where both
  secant runs fail).

## R4 plan, round 2 (fixed 2026-11-02 before the second sweep is generated or scored)
- Frozen model: main's IRR with the secant cap 50 -> `IRR_SECANT_STEPS` = 213 and nothing else (no stall rule).
- Fresh sweep 2: probes-numeric/t3411-irr-r4-fresh-2, 150 cells, gen_sweep.py with seed "T-3411/r4-fresh-2".
- Margin: on sweep 2 the frozen model (1) gets no cell wrong that main's model gets right (type or digits) and
  (2) has a pass rate within 10 points of its own pass rate on t3411-irr-r4-fresh (sweeps generated the same way).
  Otherwise nothing ships and the task is blocked with these notes.

## R4 plan, round 3 (fixed 2026-11-03 before the third sweep is generated or scored)
- Frozen model (commit "T-3411: freeze round 3"): main's IRR plus one check in `irr_fallback`: a root it settles
  on is `#NUM!` when |`irr_npv`| there is `IRR_ROOT_RESIDUAL` = 1E-4 or more. Nothing else changes (secant cap 50,
  fallback cap 50). The constant is bracketed by named cells over every IRR cell in eval whose secant runs fail and
  whose uncapped Newton settles: Excel's largest returning residual is 9.54E-5 (probes-numeric/t3478-irr-r4-wide A83,
  t3411-irr-r4-fresh A128), its smallest error residual above that 1.106E-4 (probes/t3406-irr-num-boundary A131).
- No hold-out split: every IRR case in eval was used to find the bracket. The fresh sweep is the out-of-sample test.
- Fresh sweep 3: probes-numeric/t3411-irr-r4-fresh-3, 150 cells, gen_sweep.py with seed "T-3411/r4-fresh-3",
  generated after the freeze, oracle-filled from excel-graph.
- Margin: on sweep 3, cell by cell against main's model, the frozen model (1) loses no cell main gets right (type or
  digits) and (2) has no more type mismatches than main. Otherwise nothing ships and these notes record why.

## R4 plan, round 4 (fixed 2026-11-04 before the fourth sweep and the Gate A batch are generated or scored)
- Frozen model (commit "T-3411: freeze round 4"): round 3's check moved one Newton step: `irr_fallback` takes one
  more step from its stop iterate and is `#NUM!` when |`irr_npv`| there is `IRR_ROOT_RESIDUAL` = 1E-4 or more; it
  still returns the stop iterate. Caps unchanged (secant 50, fallback 50).
- Bracket, over every IRR cell in eval (169 cases, scored cell by cell with `rustsheet-eval show --all`): no cell
  moves for any cut in 9.16E-5..1.22E-4. Excel's largest returning residual is 9.155E-5 (t3411-irr-r4-fresh A37,
  A140), its smallest `#NUM!` 1.2207E-4 (t3478-irr-r4-wide A90; next t3411-irr-r4-fresh-3 A49 at 1.526E-4).
- R4.1 split (fit lab `holdout.is_holdout`, case ids without `.case.json`): 41,288 train and 17,904 test cells.
  A37, A140 and fresh-3 A49 are train; wide A90 is test. On train alone the bracket is 9.155E-5..1.526E-4, which
  holds 1E-4, and the test cell A90 falls on Excel's side. Bit-exact against main: train 39,997 -> 40,008 (+11, 0
  lost), test 17,342 -> 17,346 (+4, 0 lost; A275 goes from a type to an error-code mismatch). Gap 96.90% train vs
  96.88% test, within the 5-point default margin.
- Against main, cell by cell over the 169 cases: 16 cells better (15 pass, 1 closer), 0 worse; round 3's three
  losses (fresh-2 A43, wide A79, A292) are gone.
- Fresh sweep 4: probes-numeric/t3411-irr-r4-fresh-4, 150 cells, gen_sweep.py with seed "T-3411/r4-fresh-4", and
  the Gate A batch accuracy/IRR/r4-T-3411 (`rustsheet-accuracy generate IRR --batch r4-T-3411`), both generated
  after the freeze and oracle-filled from excel-graph.
- Margin: on sweep 4, cell by cell against main's model, the frozen model (1) makes no cell worse (type, error code
  or closeness bucket) and (2) has no more type mismatches than main; on r4-T-3411 every point is inside the Gate A
  envelope or fails the same way on main. Otherwise nothing ships and these notes record why.
- Result (scored once on the frozen model): sweep 4 has 5 cells gained (A29, A46, A48, A49, A144), 0 worse, type
  mismatches 67 -> 62; r4-T-3411 has 31 points outside the envelope against 32 on main, the same points and kinds
  (A296 fixed). The margin holds.
- Review round 1 (excel-fidelity, REQUEST_CHANGES): (a) the FITTED range was not enforced (R4.3); (b) an absolute
  cut and one relative to the flows' size fit the bracket cells alike, since all sit near 1E+11; (c) R4.1 lacked the
  4-ulp and 15-digit columns; (d) the bracket was read off all cells before the split was applied (train-only
  bracket 9.155E-5..1.526E-4 also holds 1E-4). Answers:
  - (b) probes-numeric/t3411-irr-residual-scale (docs/research/t3411/gen_scale.py, excel-graph): the bracket cells
    (fresh A37, A140, wide A79, A292, A90, A69, fresh-3 A49, fresh-4 A49) with every flow times 10^k, k = -6..4.
    Excel itself turns `#NUM!` as the flows grow: fresh A37, A140, wide A79 and A292 return up to x1 and are `#NUM!`
    from x10; wide A90 is `#NUM!` at x1, returns at x10 and is `#NUM!` from x100. The model gives Excel's type on all
    72 cells (65 bit-exact, 7 digit misses of T-3408's kind), so the cut is absolute, as implemented. The reviewer's
    scaled cells (A79 x100 and up, A292 x1000 and up) are `#NUM!` in Excel too.
  - (a) The check applies only within `IRR_ROOT_RESIDUAL_FLOWS`: at most 148 flows (the longest scored series) and
    the largest |flow| 6.4E-12..2.2E+82 (the scored IRR sweeps' extremes); outside, the root is returned as on main.
    No cell of eval moves (cell by cell over the 169 cases against round 4: 0 changed).
  - (c) R4.1 columns, train (41,288 cells) / test (17,904), main -> model: bit-exact 39,997 -> 40,008 / 17,342 ->
    17,346; within 4 ulp 40,570 -> 40,581 / 17,601 -> 17,605; within 15 digits 40,639 -> 40,650 / 17,640 -> 17,644.
    Main's model ("always return the root") is the math reference for a type cut.
  - The cut separates 3 from 4 rounding steps (2^-15) of the engine's own `irr_npv` near 1E+11; that NPV is
    bit-identical to Excel's there is not shown, so the rule stays UNVERIFIED in financial.md.

## Outcome
Done as the residual-cut step of the type rule (round 4 of the R4 plan above); the rest of the type rule, the level
annuities that need the fallback's cap lifted, moved to T-4027.
- Engine: `irr_fallback` takes one more Newton step from its stop iterate and is `#NUM!` when |`irr_npv`| there is
  `IRR_ROOT_RESIDUAL` = 1E-4 or more; it still returns the stop iterate. FITTED(T-3411, bracket 9.155E-5..1.2207E-4),
  decision 0261 (fitted lock 9 -> 10). Applied only within `IRR_ROOT_RESIDUAL_FLOWS` (2..148 flows, largest |c|
  6.4E-12..2.2E+82), per R4.3. Unit test `irr_fallback_tests_the_residual_one_newton_step_past_its_stop`.
- Scale probe probes-numeric/t3411-irr-residual-scale (72 cells): Excel's type on all 72, 65 bit-exact; it shows the
  cut is absolute (Excel turns `#NUM!` as the same flows are scaled up).
- R4 guards (round-4 plan): bracket over all 169 IRR cases; R4.1 split train +11 / test +4 bit-exact, 0 lost, the one
  test cell inside the train bracket (wide A90) on Excel's side; fresh sweep probes-numeric/t3411-irr-r4-fresh-4: 5
  gained, 0 worse, type mismatches 67 -> 62; Gate A batch accuracy/IRR/r4-T-3411 (300 points): 31 outside the
  envelope against 32 on main, the same points and kinds, carried in known-failures.tsv by T-3548 (27), T-3407 (3)
  and T-3408 (1); `rustsheet-eval accuracy` 0 failing.
- Eval delta, cell by cell against main over every IRR case: 16 cells better (15 now pass: t3411-irr-r4-fresh A31,
  A39, A45, A144, A148; fresh-2 A50, A146; fresh-3 A49, A127, A128; t3478-irr-r4-wide A69, A70, A90, A258, A262;
  wide A275 from a type to an error-code mismatch), 0 worse; plus the new fresh-4 case (69/150). Round 3's three
  losses (fresh-2 A43, wide A79, A292) are gone.
- financial.md "IRR's fallback is `#NUM!` when NPV at its root is 1E-4 or more" updated.

## Progress notes
2026-11-04, opus run (round 3, shipped as a partial step; the task stays open): the type split on the outlay
families is the residual at the fallback's root. Inside t3406 A97-A132 (shared secant and Newton paths), Excel
returns exactly where |`irr_npv`| at the Newton root is under 1E-4 (returns up to 8.4E-5, `#NUM!` from 1.106E-4);
over all 710 secant-failing IRR cells in eval no return is above 9.54E-5. Shipped per the round-3 R4 plan above:
`IRR_ROOT_RESIDUAL` = 1E-4 in `irr_fallback`, caps unchanged. Fresh sweep 3 (t3411-irr-r4-fresh-3, scored once):
0 lost, +3, type mismatches 52 -> 49; also +5 fresh, +2 fresh-2, +5 t3478-wide, no other cell changes
(`make eval-check`: 0 regressions). financial.md "IRR's fallback is `#NUM!` when NPV at its root is 1E-4 or more".
- Still open (why the task is not done): with the Newton search uncapped plus the residual check, 219 cells would be
  fixed but 71 that main gets right would break, 39 of them the t3406 level annuities A12-A24, A34-A48, A56-A65 and
  A79 (residual under 3E-7, Excel `#NUM!`). Their cutoff in Newton steps (n = 20: 125 returns, 133 `#NUM!`; n = 30:
  223 / 241; n = 40: 313 / 344; n = 60: 577 / 649) is no fixed cap and no fixed multiple of n (6.25..6.65 per flow at
  n = 20, 9.6..10.8 at n = 60), and most returning annuities (A1-A10, A25-A30) are not Newton iterates from v = 0.
  Measured and rejected as separators on the annuities: |NPV| at the secant's far first step (1.3E+20 returns vs
  4.3E+23 `#NUM!` at n = 20, but 1.3E+131 returns at n = 60), the far step's d, and v0^n of the first Newton iterate.
- Next step: find the annuities' path. Since their returns are not v = 0 Newton iterates, try Newton started at the
  secant's far point or at v = 1/(1 + guess), each with the residual check, scoring type and digits on t3406 A1-A96
  only; then raise the fallback cap only if the 71-cell regression list empties.

2026-11-02, opus run (resumed from the landing notes below): the secant's cap moved to its own task, T-3806. The R4
sections above record the two rounds. T-3806 was dropped in its review: round 2's cap of 213 loses
probes-numeric/t3478-irr-r4-wide A227 (Excel `#NUM!`, the guess run converges at step 189), and A201/A221 need a cap
of 195/200 or more, so no step cap alone is Excel's rule (financial.md "IRR's secant runs longer than 50 steps, but no
step cap is Excel's rule"). Round 2's own criterion 1 was judged on per-case counts, which hid A227; compare cell by
cell, probes-numeric included. What is still
open, from an engine-faithful replica (Python, the engine's twice-rounded operators; scratch, not committed):
- After the cap, the failing returns fall in two classes. (a) The secant's first step jumps far (d from -2 to -1.7E+7,
  |NPV| up to 1E+290), the next step comes back to within a few ulp of d1 and the secant goes exactly flat: 38 cells
  where Excel returns, 71 `#NUM!`, 7 `#DIV/0!` of t3406 and the t3347 holdout. Every step formula tried (11 orderings
  of the secant update, plain or twice-rounded) goes flat on the same cells, so the flat is not a rounding accident
  of the update. Inside one outlay family (t3406 A122-A132) Excel alternates number / `#NUM!` with nearly identical
  first steps. `#DIV/0!` is Excel's answer exactly where log10 |NPV(d2)| is above 228 (226.2 `#NUM!`, 230.8
  `#DIV/0!`; t3406 A83 at 303.8 is `#NUM!`), as if a later trial point 35..72 times farther out in v had its
  discount leave the double range. (b) Stalls at the root with |NPV| above 1e-7: Excel returns, with the fallback's
  digits (financial.md "IRR's secant runs up to 214 steps").
- `irr_fallback` (Newton on the NPV polynomial from v = 0) run without a cap returns on 138 of 186 cells where Excel
  returns and on 99 of 103 where Excel is `#NUM!`; returns and `#NUM!` interleave at every step count, so a larger
  fallback cap is refuted as the model, though its digits are Excel's on 15 of 22 stalls.
- Next step: model what Excel does at a flat secant (class a): the evidence says a flat secant is not the end of the
  run in Excel; test a restart from the flat point (offset 1E-3 or the guess offset) or the fallback from v = 0 at
  that point, scored on class (a) only, before touching class (b).

2026-11-02 22:30 UTC, opus run (time-boxed experiment; no engine change kept): tried the "next step" above, a
restart of the secant at a flat step (class a), as a temporary switch in `irr_secant`. Scored cell by cell with
`rustsheet-eval show` (passed cells of t3406 / t3347 holdout / t3478 wide / t3411 fresh / fresh-2):
- main (cap 50): 68 / 101 / 229 / 80 / 76. Cap 213, no restart: 70 / 106 / 251 / 88 / 83.
- At a flat secant (f1 == f0), restart with d0 = the flat point and d1 = d0 -+ 1E-3 (the start rule), up to
  1 or 3 times, steps counted on: cap 50 gives 68..70 / 101 / 229 / 80 / 76; cap 213 gives 70..72 / 106 / 250..251 /
  88 / 83. The same restart with the step count reset (fresh 50 steps per restart, 1 or 5 restarts): 68..70 / 101 /
  229..230 / 80 / 76.
- At a flat secant, restart from d = 0 (v = 1, r = 0) once: cap 50 68 / 101 / 227 / 80 / 76; cap 213 70 / 106 / 248 /
  88 / 84.
- So a restart at the flat point, from the flat point or from r = 0, does not reproduce Excel's returns on class
  (a): at most +2 cells on t3406, and it loses cells on t3478. Refuted. Do not repeat.
- Next step, not yet tried: class (a)'s first step jumps d from about -2 to -1.7E+7 (|NPV| up to 1E+290). Test
  whether Excel clamps or rejects that far first step, for example by treating |NPV| above a bound or a
  non-representable discount power (T-3407's `#DIV/0!` sign) as a failed trial and halving the step toward d1.
  Score that on class (a) only, cell by cell, at cap 213.
- Then tried that next step (22:40 UTC): a damped secant step, halving d2 toward d1 (up to 60 halvings) when the
  trial (1) fails or has |NPV| above |NPV(d1)|, (2) fails or has d2 >= 1, or (3) fails or has |NPV| above 1E+100.
  Passed cells, t3406 / t3347 holdout / t3478 / fresh / fresh-2 / t2565 / irr-fallback-roots / m9-fin-irr-rate /
  t2927 / guess-edges / guess-edges-2 (main at cap 50: 68 / 101 / 229 / 80 / 76 / 180 / 40 / 691 / 40 / 42 / 40):
  (1) 29 / 84 / 177..192 / 68..76 / 67..71 / 173 / 36 / 655 / 52 / 40 / 26, which breaks the fit set;
  (2) and (3) at cap 50: 68 / 101 / 224 / 80 / 76 / 180 / 36 / 691 / 52 / 41..42 / 40; at cap 213: 70 / 106 / 238 /
  88 / 83 / 180 / 36 / 691 / 55 / 41..42 / 40. No damping rule moves t3406, and all lose t3478 cells and
  irr-fallback-roots cells (-4). Refuted: Excel does not shorten a far first step. Do not repeat.
- What is left: the far first step stands in Excel too, so what tells Excel's returns apart from its `#NUM!` inside
  one outlay family (t3406 A122-A132) has to come later in the run. Next, log the exact iterate sequence of a
  returning cell and its `#NUM!` neighbour with the engine's operators (a throwaway test printing d, NPV per step at
  cap 213) and compare where they part.
- Traced it (22:45 UTC; a throwaway test, not committed, printing d and NPV per step with `irr_npv` from guess 0.1)
  on t3406 A124 (Excel returns), A125 (`#NUM!`), A126 (returns) and A127 (`#NUM!`). All four take the same path:
  d0 = 0.0909..., d1 = 0.0919..., step 0 jumps to d2 = -7.45..-7.58 (NPV 8E+27..1.1E+28), step 1 comes back to
  0.09190909090909116 and step 2 to 0.09190909090909109 (7E-17 apart). What decides the run is whether NPV at
  those two d's rounds to the same double. In ours, A124's two values differ by 1 ulp (-7.60477028807863E+10 and
  ...629E+10), so the secant goes on and converges at d = -0.2360957 (it then stalls at |NPV| 3E-5, class b). For
  A125, A126 and A127 the two values are equal, so the secant is flat and the run fails. Excel returns A126, so its
  NPV at one of those two points differs from ours in the last ulp. The flat at step 2/3 is the whole of class
  (a): no rule on the step, cap or restart can reproduce it (consistent with the refutations above). Only an NPV
  that is bit-exact at d about 0.0919 can.
- Next step: compare `irr_npv`'s operators (the `discounted_sum` product at x = 1/v, extended quotients) with
  Excel's NPV on these exact points. Probe `NPV(1/(1-d)-1, flows)` style cells, or better IRR's own evaluations,
  for the A122..A132 family at d = 0.09190909090909116 and 0.09190909090909109. Then look for a summation order or
  a precision under which A122/A123/A124/A126/A128/A129/A132 step on and A125/A127/A130/A131 go flat. That is a
  bit-level fit (R4 applies).

2026-11-03, opus run (resumed, branch idle 23 h; merged origin/main in for the current IRR code). Findings, landed
in financial.md "IRR's returns where the secant fails are Newton iterates on P(v) from v = 0, past the 1e-7 stop"
with docs/research/t3411/fallback_iterates.py; no engine change:
- The "next step" above (a last-ulp NPV difference at d = 0.0919) is refuted: twelve NPV evaluations (plain,
  twice-rounded, 80-bit; quotients by x, products by v, Horner in v or x) all go flat at step 3 on t3406 A109-A144,
  so no NPV arithmetic separates A126 (returns) from A125 (`#NUM!`). Do not repeat.
- Also refuted on t3406 (secant alone, cap 213): Illinois once bracketed (types 104/144 against 92, but no Excel
  digits) and keeping the two smallest-|NPV| points (85).
- New lead: where both secant runs fail and Excel returns, Excel's double is very often exactly an iterate of
  `irr_fallback`'s Newton from v = 0 (see the financial.md entry and the script's output for the counts), at steps
  up to about 630: the fallback's 50-step cap is not Excel's, and where the cells can tell, Excel mostly returns the
  iterate one past the engine's 1e-7 stop (51 cells against 4). Neither the step size nor |P| decides that extra step, and Newton converges on most `#NUM!`
  cells too, so the type rule is still open.
- Next step: the type split, on the cells where Excel's value is a Newton iterate. Inside t3406 A122-A132 the
  returning and `#NUM!` cells share the secant path and the Newton path (126 steps); find what differs, for example
  a check Excel makes on the root before returning it (a residual of its own NPV at the root against a bound, or
  the root against the secant's last bracket), and probe it with excel-probe on a family whose outlay moves by
  1E-4 steps.

## Landing notes (2026-11-04, PR #3063; acted on by round 4 above)
- PR #3063 (draft), head 8bb93fa98 before these notes. CI was running on that head. Reviewers on 1779ea4f: security
  APPROVE, design APPROVE (MINOR: no unit test), excel-fidelity REQUEST_CHANGES. None has reviewed the new head.
- Fidelity's blocking finding is correct: the engine checks |`irr_npv`| at the fallback's stop iterate, but the bracket
  was measured one Newton step later. That makes three cells `#NUM!` that main had within ulps of Excel:
  probes-numeric/t3411-irr-r4-fresh-2 A43, t3478-irr-r4-wide A79 and A292 (residuals at stop / stop+1: 3.9E-3 /
  6.9E-5, 8.8E-4 / 1.2E-5, 1.3E-4 / 6.7E-6). So "0 lost, no other cell changes" in the progress note above is wrong
  for those sweeps: fresh-2 is +2/-1 and t3478-wide +5/-2. `make eval-check` missed it because probes-numeric is
  ungated. The round-3 margin was met only on fresh-3 itself.
- Next step (a new R4 round 4, since this changes the frozen model): take one more Newton step past the stop and test
  the residual there (keep returning the stop iterate unless digits are scored too). Then re-score every IRR case of
  probes and probes-numeric cell by cell against main's binary. Also owed, per the fidelity review:
  - the fit lab's R4.1 split (tools/src/rustsheet_tools/fitlab/holdout.py), with train and hold-out counts;
  - the fresh sweep as an `r4-T-3411` batch of the IRR Gate A table (eval/accuracy/tables/IRR, SPEC-0029 "The R4
    guards"), not an ungated probes-numeric case, with its envelope count;
  - the FITTED range enforced in code, or narrowed to what the evidence supports;
  - decision 0261's numbers updated to match.
- Do not repeat: the absolute 1E-4 cut at the stop iterate. Evaluating it there is refuted by the three cells above.

## Landing notes (2026-11-01, superseded by the progress notes above)
2026-11-01, opus run (stopped at the session's landing window; no PR opened, no code changed).
- Branch head: the claim commit plus these notes. Nothing is implemented; CI and reviewers have not run.
- Scored on the type (number, `#NUM!`, `#DIV/0!` read as an error) of 288 cells: probes/t3406-irr-num-boundary (144)
  and probes-numeric/t3347-irr-conventional-holdout (144). T-3347's probe and sweep cases are on T-3347's branch, not
  main, so they were not scored. The scratch scorer was a plain-double replica of `irr_secant` plus candidate fallbacks,
  not the engine's twice-rounded operators.
  - Plain-double secant from 0.1 (retry 0.1), no fallback: 193/288.
  - Plus Newton in r from the guess (analytic derivative, |dr| < 1e-7 stop): 212 / 224 / 225 at 20 / 50 / 100 steps.
    It does not rescue the level annuities (t3406 A5-A10, root -0.10 to -0.20, n = 20): from r = 0.1 the NPV slope is
    small, so the first step lands below r = -1. It also returns 7 cells Excel is `#NUM!` on (t3406 A99-A106, n = 60,
    outlay about 7E+10). Refuted as the fallback.
  - Plus the secant in r (start 0.1 and 0.101): no change (193). Refuted.
  - The engine's own XIRR bracket walk (financial.md "XIRR is a bracketing bisection"), run as
    `XIRR(flows, 45000 + 365k, guess)` so the exponents are exactly k: it returns a number on all 288 cells. That covers
    every one of the 84 cells where Excel returns and engine IRR is `#NUM!`, but it also returns on 80 `#NUM!` and 31
    `#DIV/0!` cells (e.g. t3406 A12-A15, roots -0.24 to -0.30, n = 20; A66-A69, `#DIV/0!` in Excel). An unbounded
    bracket walk is refuted. Excel's pattern (returns down to a root of about -0.22 at n = 20, `#NUM!` below, `#DIV/0!`
    for larger outlays at n = 40 and 60) points to a walk or search whose reach in r is bounded and which, for long
    series, tries a rate whose discount leaves the double range (NPV's `#DIV/0!`, T-3407).
- Next step: model the bound. Find the most negative root Excel returns per n (t3406 gives -0.22 / -0.18 / -0.14 /
  -0.12 for n = 20 / 30 / 40 / 60) and test whether it is a fixed trial-rate set (for example rates stepped from the guess
  by a factor, as XIRR's trial chain does) whose NPV signs bracket the root, scoring with the engine's extended operators
  (`irr_npv`), not plain doubles.
- The level-annuity cutoffs in outlay P (t3406, flows `{-P,1,...,1}`): Excel returns up to P = 649.62 and is `#NUM!`
  from 1003.96 at n = 20; 2133.94 / 4033.97 at n = 30; 2970.92 / 6672.71 at n = 40; 17850.72 / 60801.44 at n = 60.
  Neither P/n nor the NPV at the guess is constant across n at the cutoff, and v^n at the last returned root is 144,
  391, 416 and 2140, so a cap on the discount power is not it either. `#DIV/0!` starts at P = 1.57E+8 for n = 40 and
  8.2E+5 for n = 60 (A83 at 1.35E+7 is `#NUM!` again), never for n = 20 or 30. That fits a trial rate near -1 whose
  n-th discount power leaves the double range (1 + r below 10^(-308/n): 2E-8 at n = 40, 7.6E-6 at n = 60, out of reach
  at n = 20). Untested hypothesis: after the secant fails, Excel's search moves r toward -1, and the outlay decides how far.
- Excel's returned level-annuity roots are 1E-18 to 9E-12 from the exact root (roots.tsv), and the outlay-family roots
  1E-18 to 2E-16: consistent with any fast solver stopped at |step| < 1e-7 with a 1e-7 residual, so the digits do not
  separate a Newton step from a secant step. The search's reach, not its final steps, is what the model must find.
- Do not repeat: Newton in r and the secant in r as fallbacks; an unbounded XIRR-style bracket walk.

## Decision 0267 cells
Decision 0267 (the owner's cut-loss plan, 2026-11-04) gives this task 9 probes-numeric cells that no open task listed: 7 with the wrong type or error, 2 first wrong at the 12th or 13th digit.
These come from the run at main 985d511b. They are Tier A (a wrong type or error, or the 11th digit or earlier) or Tier B (the 12th or 13th digit). Tier A is worked first.
Decision 0056's time-box applies. If two attempts do not move these cells, block the task as time-boxed: decision 0267 then accepts the cells that are left, and no follow-up is filed for them.

- probes-numeric/t3411-irr-r4-fresh-2.case.json (4): Calc!A32, A43, A50, A146
- probes-numeric/t3411-irr-r4-fresh.case.json (5): Calc!A31, A39, A45, A144, A148

- [ ] Every cell listed above matches Excel within half a unit of the 15th significant digit, or has Excel's type and error. Otherwise the task is blocked under decision 0056 and the rest is accepted (decision 0267).

## Visible-parity stage
Every cell in "Decision 0267 cells" is within half a unit of Excel's 15th significant digit, or has Excel's type, error and text (decisions 0083, 0267).
