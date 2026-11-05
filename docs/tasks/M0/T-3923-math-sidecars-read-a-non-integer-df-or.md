---
id: T-3923
title: "Math sidecars: read the argument forms Excel accepts and the Gate A definitions refuse (a truncated df, BETA.DIST/BETA.INV's bounds, RATE's guess): about 1,050 counted sweep cells"
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
`rustsheet-accuracy math` gives a math value to a one-call cell of a Gate A function whose arguments are literals, or
exact-input cells, in a form Excel accepts but the definition refuses today. The forms are a non-integer df that Excel
truncates, the optional bounds A and B of BETA.DIST and BETA.INV, and RATE's `guess`. Decision 0188 R1(c) can then
read about 1,050 counted probes and probes-numeric cells against the math. It counts them today only because they have
no math value. Most should turn out to be Phase 1b evidence, and the ones that do not show up as true envelope failures.

## Context
- Gap triage 2026-11-03 (`make eval` on agent/functions/triage-opus-2026-11-03, report.json):
  - probes-numeric has 4,129 R1(c)-counted cells. Many count only because they have no math value: "until a cell has a
    math reference it counts as failing the envelope", crates/rustsheet-eval/src/r1c.rs.
  - `scripts/exit_owners.py --all-off` lists 2,161 counted cells that only `defer:` tasks own. `scripts/phase1_exit.py`'s
    `owners` criterion reads those cells as unowned. 1,291 of them are F.DIST cells of the t2586 sweeps.
- tools/src/rustsheet_tools/accuracy/eval_math.py `fit_arg` returns "non-integer for an integer parameter" for any
  non-integer value of an `int` parameter. Its module doc says why: "whether Excel truncates is per function, so the
  cell is skipped rather than guessed". The evidence now settles it for these functions:
  - **F.DIST**: statistics.md, "F density ..." entry: "A non-integer df is truncated first (`F.DIST(2,150.7,3,FALSE)`
    0.17202585847929508 is the density at df 150)". Also "F-distribution df limit (T-1530)" and the legacy entry
    (`FDIST(1,2.9,5)` = `FDIST(1,2,5.9)` = `FDIST(1,2,5)`).
  - **T.DIST**: statistics.md, "Student t sweep: df truncation, df limits, T.INV.2T above 1 (T-1043)": df is
    truncated, not rounded.
  - Any other function: only with a cited entry. Otherwise keep the skip.
- `Definition.params_for` refuses an argument count the definition has no params for:
  - BETA.DIST with A and B (5 or 6 arguments);
  - BETA.INV with A and B (4 or 5 arguments);
  - RATE with `guess` (6 arguments).
  BETA.DIST and BETA.INV with bounds are the standard beta on (x-A)/(B-A), as in Microsoft's documentation and
  statistics.md. Read how Excel forms (x-A)/(B-A) from the evidence: the engine's beta.rs and the T-3443 and T-3566
  entries. A definition that rounds where Excel does not would be a fit, which Gate A forbids, so where the evidence
  does not say, compute the exact (x-A)/(B-A) and say so in the definition.
  RATE's `guess` does not change the root when `annuity.unique_rate` proves it unique (`defs/annuity.py`, the module
  doc). A guess Excel cannot converge from is an Excel error, which Gate A already reads as error against number.
- Counted cells that have no math value today, by reason (the case's sidecar has no entry and no `skipped` line):
  - non-integer df, 827:
    - F.DIST: t2586-r5-f-final-hold 396, t2586-verify-r5-f-final-hold 336, t3567-r5-fdf1-hold 41 (B1307
      `F.DIST(6.11179254226e-05,377,1.123,TRUE)`),
      t2586-r5-f-edges-a 32, t2586-verify-r5-f-edges-hold 19, t2586-r5-f-edges-hold 1 (e.g. t2586-r5-f-final-hold
      Calc!B1000 `F.DIST(A1000,67,89.1,FALSE)`: Excel 1.5710419373836891e-97, ours 1.5710419373836956e-97);
    - T.DIST: m9-t-nonint-df Calc!A44 `T.DIST(2,3.7,FALSE)`.
    - CHISQ.INV.RT: t3336-chisq-df1-right-tail Calc!A290 `CHISQ.INV.RT(1E-9,1.5)`, only if an entry shows that its
      df truncates. statistics.md shows it for CHISQ.DIST.RT and CHIDIST (`CHIDIST(3,2.9)` = `CHIDIST(3,2)`), not yet
      for the inverse.
  - BETA.DIST and BETA.INV with bounds, 58: probes-numeric t3443-basym-a 9, t3443-basym-sweep 9, t3443-basym-hold 3
    (e.g. Calc!A9 `BETA.DIST(1.49003654331704,6199.511,41408,TRUE,0.95,5.02)`), t3566-beta-inv-tiny-shape-and-range 37
    (Calc!A101 `BETA.INV(0.4998,0.01,0.01,0.15,1.2)`, A11 `BETA.INV(0.5,1E-5,1E-5,0.15,1.2)`).
  - RATE with a guess, 163: probes/t3906-rate-restart-oos 137 (Sheet1!A1
    `RATE(1,-1568519397.05575,7403597.01937745,1561086885.99697,0,0.000328820719779769)`), t3698-rate-flat-restart-fresh
    25, t3698-rate-flat-restart 21 (and probes-numeric t3698-rate-r4-fresh A201, t3478-rate-r4-wide A285, if they are
    the same form).
- Ownership:
  - Their value owners are `defer:` tasks (T-3158, T-3173, T-3174) or live ones (T-3850 for RATE). Until this task
    lands, it owns the counted cells of the cases below (decision 0245; `exit_owners` reads a line that names a case
    and no cell as owning all of it), because what keeps them counted is the missing math value, not the engine.
  - T-2647 owns the 86 subnormal-Y cells of the t2586 edge cases, and T-3924 the scaled and arithmetic-argument cells.
  - The Outcome names the owner of every cell of these cases that is still counted after the regeneration.
  - Cases:
    - probes-numeric/t2586-r5-f-final-hold.case.json
    - probes-numeric/t2586-verify-r5-f-final-hold.case.json
    - probes-numeric/t3567-r5-fdf1-hold.case.json
    - probes-numeric/t3443-basym-a.case.json, t3443-basym-sweep.case.json, t3443-basym-hold.case.json
    - probes-numeric/t3566-beta-inv-tiny-shape-and-range.case.json
- Code: `eval_math.py` (`fit_arg`, `fit_args`), `model.py` (`Param`: add an evidenced truncation flag rather than
  a new kind if that is simpler), `defs/fisher.py` (`_df`), `defs/student_t.py` (`bounded_df`, `_DF`), `defs/beta.py`,
  `defs/annuity.py`. Keep one truncation helper (`defs/_common.py` `trunc`).

## Acceptance criteria
- [ ] An `int` parameter reads a non-integer by truncation only where its definition cites the semantics entry that
      shows Excel truncates. F.DIST's and T.DIST's df do (CHISQ.INV.RT's if an entry shows it), each with a unit test (`F.DIST(2,150.7,3,FALSE)` gets the
      df-150 value). Every other `int` parameter keeps today's skip.
- [ ] BETA.DIST and BETA.INV accept the optional A and B. RATE accepts the optional `guess` and ignores it when the
      root is unique. Each has a unit test and a cited source.
- [ ] The probes and probes-numeric sidecars are regenerated and committed, and `make eval-math-check` is current.
- [ ] The Outcome quotes report.md's R1(c) line ("Visibly-off cells under decision 0188 R1(c)") before and after, and
      the count of `scripts/exit_owners.py --all-off` cells with `(deferred)` before and after. Any newly gained cell
      that is now outside Gate A's envelope is named there with its owner task, filed if none exists.
- [ ] `make eval-baseline` records nothing: no case's pass/fail changes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Arithmetic inside an argument and outer scale wrappers such as `F.DIST(A1,3,5,FALSE)*2^600` (T-3924).
- Arguments that read formula results other than exact-input arithmetic (t3444's `BINOM.DIST(A1,...)` over
  `BINOM.INV`).
- New Gate A definitions (T-3798, T-3925).
- Any engine change.

## Outcome
- `Param.truncates` (model.py) cites the semantics entry showing Excel truncates a non-integer `int` argument;
  `eval_math.fit_int` then truncates toward zero, and every other `int` parameter keeps the skip. Set for F.DIST's
  df (statistics.md T-2586 round 5 and T-1864: `F.DIST(2,150.7,3,FALSE)` is the df-150 density), T.DIST's df
  (T-1043) and CHISQ.INV.RT's df (T-1543: `CHISQ.INV.RT(1E-9,1.5)` is the df-1 value). F.INV, T.INV, CHISQ.INV and
  the rest still skip.
- `Definition.call_params`: trailing optional parameters an eval call may pass but the sampler never draws, so no
  Gate A table changes. BETA.DIST takes [A, B] at the exact t = (x - A)/(B - A), the density divided by B - A
  (`rescaled_param` "x": the domain function checks A <= x <= B instead of x's [0, 1]); a density at an end of
  [A, B] with a shape of 1 or less, where Excel gives 0, is no math value. BETA.INV takes [A, B] as A + (B - A) q
  (T-3566: Excel's scaled calls equal that bit for bit at 40 of 40 points). RATE takes `guess` and ignores it: its
  math only answers a unique root.
- Unit tests: tools/tests/test_accuracy_call_forms.py; test_accuracy_eval_math.py's skip examples moved to T.INV.
- Sidecars: `math --check` named 45 stale probes and probes-numeric cases (more than the task listed: every RATE
  call with a guess, and F.DIST/T.DIST/CHISQ.INV.RT cells in other cases); each was regenerated.
- R1(c) line ("Visibly-off cells under decision 0188 R1(c)"), before then after (local `make eval`; ironcalc-calc
  26 counted both times):
  - probes 1875 counted, 99 Phase 1b evidence -> 1643 counted, 337 Phase 1b evidence;
  - probes-numeric 2008 counted, 2891 Phase 1b evidence -> 1954 counted, 2945 Phase 1b evidence.
- `scripts/exit_owners.py --all-off`: no `(deferred)` lines before or after. With this task `done`, no cell of
  the listed cases is left without an open owner; their remaining counted cells keep their owners (T-2647,
  T-3924, T-3850 and the defer: tasks).
- Eval delta: none (no pass/fail changes; sidecars only feed R1(c)).
