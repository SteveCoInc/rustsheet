---
id: T-3924
title: "Math sidecars: arithmetic over exact-input cells inside a call's arguments, and an exact outer power-of-two or -0 wrapper: about 930 counted sweep cells"
milestone: M0
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: []
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
`rustsheet-accuracy math` gives a math value to two more kinds of Gate A cell:
- an argument that is arithmetic over number literals and exact-input cells (`F.DIST(A112*A112,1,6,FALSE)`,
  `BETA.DIST(1/(1+20/(4*A1249)),2,10,FALSE)`). The argument is evaluated with Excel's operator model, as the fit lab
  evaluates an exact-input cell, and the math is the function at that double.
- a one-call formula wrapped in an exact scale (`=F.DIST(A1020,1,10,FALSE)*2^600`, `=BINOM.DIST(3000,10000,0.3,TRUE)-0`).
  The math value is the scaled math, when the scale is exact.

Decision 0188 R1(c) then reads about 930 counted cells against the math. It counts them today only because they have
no math value.

## Context
- Gap triage 2026-11-03 (`make eval`, report.json): probes-numeric has 4,129 R1(c)-counted cells, and
  `scripts/exit_owners.py --all-off` lists 2,161 counted cells that only `defer:` tasks own. `scripts/phase1_exit.py`'s
  `owners` criterion reads those as unowned. Among them are 472 BETA.DIST cells of t2513-r3-fdensity-a and most of
  the 1,291 F.DIST cells of the t2586 sweeps.
- tools/src/rustsheet_tools/accuracy/eval_math.py, module doc: a candidate gets no math value when "an argument reads a
  formula cell ..., unless that cell's whole formula is arithmetic over number literals" or when the argument is
  anything the fit lab's sweep reader leaves unparsed. `fitlab.sweep.parse_arg` reads a bare reference or
  `arithmetic(text)` over literals, but not arithmetic that mixes literals and references.
  `eval_math`'s `_CALL` also needs the whole formula to be one call, so an outer `*2^600` drops the cell.
- `fitlab.sweep.arithmetic` is Excel's operator model, from numbers.md "Division rounds twice": each of + - * / rounds
  its exact result to 64 bits and then to a double, a subnormal product, quotient or power is 0, and `^` must be exact.
  Reuse it (directive 3) by letting it resolve a reference to an exact-input cell. Do not write a second evaluator.
- An outer scale is exact when it is `*2^k`, `/2^k` or `-0` and the scaled math stays a normal double. The t2586
  companions exist so that the tiny densities are visible above the double range's floor (statistics.md, F density
  entry: "their `*2^600` companions such as C1296 and C1301 show the 0 is Excel's, not Graph's"). Where the unscaled
  value is subnormal or 0 in Excel, the math of the product is not Excel's operator result, and the cell keeps its skip.
- Counted cells without a math value, by form:
  - arithmetic argument, 511:
    - probes-numeric/t2513-r3-fdensity-a 472 (Calc!C1249 `BETA.DIST(1/(1+20/(4*A1249)),2,10,FALSE)`: Excel
      0.0007647686159537181, ours 0.0007647686159537187);
    - t2513-r3-tdensity-hold 31, t2513-r3-tdensity-a1 7, t2513-r3-tdensity-a2 1 (`F.DIST(A112*A112,1,6,FALSE)`).
  - outer scale, 423: probes-numeric/t2586-r5-f-edges-a 206 (Calc!C1020 `F.DIST(A1020,1,10,FALSE)*2^600`),
    t2586-r5-f-edges-hold 119, t2586-verify-r5-f-edges-hold 96; probes/t3444-binom-range-inv-large-n Range!B4, B10
    (`BINOM.DIST(3000,10000,0.3,TRUE)-0`).
- Ownership:
  - Their value owners are `defer:` tasks (T-3158, T-3173, T-3174, T-3244 and others). Until this task lands, it owns
    the counted cells of the cases below (decision 0245; `exit_owners` reads a line that names a case and no cell as
    owning all of it), because what keeps them counted is the missing math value, not the engine.
  - T-2647 owns the 86 subnormal-Y cells of the t2586 edge cases, and T-3923 their non-integer-df cells.
  - The Outcome names the owner of every cell of these cases that is still counted after the regeneration.
  - Cases:
    - probes-numeric/t2513-r3-fdensity-a.case.json
    - probes-numeric/t2513-r3-tdensity-hold.case.json, t2513-r3-tdensity-a1.case.json, t2513-r3-tdensity-a2.case.json
    - probes-numeric/t2586-r5-f-edges-a.case.json
    - probes-numeric/t2586-r5-f-edges-hold.case.json
    - probes-numeric/t2586-verify-r5-f-edges-hold.case.json

## Acceptance criteria
- [x] An argument that is arithmetic over literals and exact-input cells gets the double that `fitlab.sweep.arithmetic`
      gives for it, through one shared evaluator. A unit test covers `A*A` and `1/(1+20/(4*A))` over an exact-input
      cell.
- [x] A formula `=CALL(...)` followed by `*2^k`, `/2^k` or `-0` gets the scaled math only where the scale is exact. The
      definition of "exact" is stated in the module doc, and a unit test covers a scaled cell and a cell left skipped.
- [x] The probes and probes-numeric sidecars are regenerated and committed, and `make eval-math-check` is current.
- [x] The Outcome quotes report.md's R1(c) line before and after, and the `(deferred)` count of
      `scripts/exit_owners.py --all-off` before and after. Any newly gained cell that is now outside Gate A's envelope
      is named with its owner task, filed if none exists.
- [x] `make eval-baseline` records nothing.
- [x] `make check` green; reviewers approve.

## Out of scope
- Non-integer df, optional bounds and RATE's guess (T-3923).
- Arguments that read formula results other than exact-input arithmetic: t3444's `BINOM.DIST(A1-1,...)` over
  `BINOM.INV`, and nested calls such as t3570's `LN(GAMMA(...))`.
- Any engine change.

## Outcome
- `fitlab.sweep.arithmetic` takes an optional `resolve`: a single-cell reference token (`A1249`, `Calc!$A$2`; never a
  function name such as `LOG10(` or a range) is an operand only through it. `parse_arg(..., cell_refs=True)` resolves
  through `cell_number`, which reads a cell holding a normal double or 0 and refuses text, logicals, empty cells and
  stored subnormals (no evidence of how Excel reads one), and the result drops a negative zero. One evaluator: the fit
  lab keeps `cell_refs` off, so its sweeps are unchanged; `eval_math` turns it on, and its `LiteralBook` still reads a
  formula cell only when its whole formula is arithmetic over literals.
- `eval_math.scaled_call` takes a call followed by `*2^k`, `/2^k` or `-0` with k from -1022 to 1023; `scale_math`
  scales the call's math by 2^k when the call's math and the scaled value are both normal doubles or 0, and otherwise
  lists the cell under `skipped` as `scaled value not exact` (rule in the module doc and SPEC-0029). `-0` and 0 keep
  the generator's text.
- Unit tests: tools/tests/test_accuracy_arith_scale.py (`A1*A1`, `1/(1+20/(4*A2))` over cells, the resolver's
  refusals, scale parsing, exact and inexact scales, a case with scaled, skipped and arithmetic cells, `check_case`).
- Sidecars: `math --check` named 28 stale probes and probes-numeric cases, more than the seven listed: any one-call
  cell whose argument does arithmetic on a cell (`EXP(-(A1*A1))`, `ATAN(B1/A1)`, `T.DIST(-A1,B1,TRUE)`) and the
  `*2^100`/`-0` companions of t2756, t2771, t3444 and subnormal-saved-as-zero. All 28 are regenerated case by case
  (t3444 with its points spread over processes; its CRITBINOM and BINOM.DIST.RANGE points at n 1E9 to 2^31-2 take
  20-60 s each). Math cells in those sidecars 24,400 -> 70,339; generator skips 424 -> 1,335 (most are t1916's 519
  not-converged points and the `scaled value not exact` companions, such as t2586's subnormal-density `*2^600`
  cells and `EXP(-745)*2^1000`). `make eval-math-check`: current.
- R1(c) line, before (main at 51a7483a) then after (local `make eval`, all sidecars but t3444's regenerated; t3444
  has no visibly-off cell either way):
  - before: "ironcalc-calc 26 counted, 0 Phase 1b evidence, 0 R3 passing, 0 M14-owned; probes 864 counted, 326 Phase
    1b evidence, 0 R3 passing, 16 M14-owned; probes-numeric 2199 counted, 2924 Phase 1b evidence, 66 R3 passing, 0
    M14-owned";
  - after: "ironcalc-calc 26 counted, ...; probes 864 counted, 326 Phase 1b evidence, ...; probes-numeric 2198
    counted, 2925 Phase 1b evidence, 66 R3 passing, 0 M14-owned".
  The ~930 cells the 2026-11-03 triage counted are no longer visibly off: engine work merged since then brought every
  cell of the seven listed cases (and of all 28 regenerated cases) within 15 digits of Excel, so the new math values
  move R1(c) by one cell only. No regenerated case has a visibly-off cell, so none gains a cell outside Gate A's
  envelope and no cell of these cases needs an owner.
- `scripts/exit_owners.py --all-off`: 0 `(deferred)` lines before and after (40 unowned FORECAST.ETS cells both times,
  unrelated).
- `make eval-baseline`: records nothing. Eval delta: none (sidecars feed R1(c) only).
