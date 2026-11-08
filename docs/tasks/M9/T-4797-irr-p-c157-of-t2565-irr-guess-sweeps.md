---
id: T-4797
title: "IRR P!C157 of t2565-irr-guess-sweeps has no math sidecar entry though its guess is a constant cell (T-4778 follow-up)"
milestone: M9
lane: functions-financial
status: dropped
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`=IRR({2.5,1,-123456789.123},B157)` (probes/t2565-irr-guess-sweeps P!C157) gets a math value in its `.math.json` sidecar, or a skip reason, so decision 0188 R1(c) can sort it.

## Context
T-4778 gave IRR's definition an optional guess. After regeneration the sidecar `eval/suites/probes/cases/t2565-irr-guess-sweeps.math.json` holds only 6 cells and neither C157 nor a skip for it, although B157 is a constant (2697.5242098806502) and the flows have one sign change. `eval_math.candidates` returns 6 candidates for the case and none for C157. Find why (`tools/src/rustsheet_tools/accuracy/eval_math.py`, `candidates`). The cell is still counted visibly off (NumericClose, Excel 0.006875307760903171, engine 0.006875307761111449).

## Acceptance criteria
- [ ] `candidates` lists C157 and the sidecar has its math value or a stated skip; `rustsheet-accuracy math --check` passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Handoff (resumed)
Findings (sonnet run, no code changed):
- The task's premise is wrong. `B157` in `t2565-irr-guess-sweeps.case.json` is the JSON *string*
  `"2697.5242098806502"`, i.e. a **text cell** (`spec2xlsx._write` uses `write_string`;
  `strings_to_numbers` is False). All 175 `B` guess cells are text. So `eval_math.candidates` does list
  C157 (and every other `C` cell), each refused by `fit_arg` as "non-numeric argument"
  (`_fit_number`); `case_math` only counts such refusals (`out.skip`), it never writes them to the sidecar,
  and `check_case` ignores `str`-args candidates, so `math --check` passes as is.
- The numbers quoted in the task are not the current ones. `rustsheet-eval show
  probes/t2565-irr-guess-sweeps.case.json`: C157 expected (Excel) 7026.083663321499, engine
  7026.0836627623075 (NumericClose); 180 of 181 cells pass.
- Evidence of a quirk: Excel reads the *text* guess cell as a number (C157 = 7026.0836633 differs from
  C156/A142 = 7026.0836628, which the default guess gives), while the engine ignores the text guess
  (its C157 equals A142). This contradicts the "A text guess is #VALUE!" lines in
  `docs/excel-semantics/financial.md` (~1839, 1869) unless those are for a literal `""` only; a
  reference to a numeric-text cell seems to be coerced.
Next (opus): decide whether the real defect is the engine (IRR guess from a numeric-text reference
should coerce like Excel; probe a few text refs: "0.5", " 1", "abc", "1e-3"), record it in
`financial.md`, and rewrite this task's goal/AC accordingly; the math sidecar needs no entry for a text guess.

## Outcome
Dropped: the premise is wrong and nothing needs changing.
- B157, and every guess in column B, is a numeric-text cell (a JSON string in the case). `candidates` does
  list C157. Like every column C cell, `fit_arg` refuses it as "non-numeric argument", which the
  eval_math module doc lists as a no-math case. Only the generator's own drops go into `skipped`, so
  `rustsheet-accuracy math --check --suite probes` passes as is ("current").
- The handoff's quirk hypothesis is not borne out either. The engine already reads a numeric-text guess
  cell as its number, as Excel does: P!C154 with text B154 is 7026.083663189925 in both, unlike the no-guess
  P!A142. 180 of the 181 t2565 cells match. So no `financial.md` text-guess rule changes. The fact is
  recorded in financial.md's T-2565 bullet.
- P!C157 is still off (NumericClose, Excel 7026.083663321499, engine 7026.0836627623075). This is the IRR
  secant outlier that financial.md's T-2565 and T-3120 bullets already record ("P!C157 stays the outlier").
  It belongs to the IRR kernel campaign, not to the math sidecar.
- Eval delta: none.
