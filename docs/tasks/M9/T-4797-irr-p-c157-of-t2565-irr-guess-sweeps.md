---
id: T-4797
title: "IRR P!C157 of t2565-irr-guess-sweeps has no math sidecar entry though its guess is a constant cell (T-4778 follow-up)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`=IRR({2.5,1,-123456789.123},B157)` (probes/t2565-irr-guess-sweeps P!C157) gets a math value in its `.math.json` sidecar, or a skip reason, so decision 0188 R1(c) can sort it.

## Context
T-4778 gave IRR's definition an optional guess. After regeneration the sidecar `eval/suites/probes/cases/t2565-irr-guess-sweeps.math.json` holds only 6 cells and neither C157 nor a skip for it, although B157 is a constant (2697.5242098806502) and the flows have one sign change. `eval_math.candidates` returns 6 candidates for the case and none for C157. Find why (`tools/src/rustsheet_tools/accuracy/eval_math.py`, `candidates`). The cell is still counted visibly off (NumericClose, Excel 0.006875307760903171, engine 0.006875307761111449).

Root cause (2026-11-08, opus run): `candidates` on main does list C157 and every other IRR cell of the case (181 candidates). But all 175
guess cells B1:B175 are JSON *strings* (`"2697.5242098806502"`), which spec2xlsx writes as text cells. So `LiteralBook.value` returns a
str, and `fit_arg` → `_fit_number` drops each call as "non-numeric argument". That reason is only counted in `CaseMath.skipped`; it is
neither a per-cell sidecar entry nor a `dropped` entry, and `math --check` accepts it. Excel coerces the numeric-text guess: the 175 oracle
values take 59 distinct values, so the guess matters. Two ways to fix it:
(a) Read a referenced numeric-text cell as a number for a `real` parameter, cut to 15 digits by truncation as Excel parses numeric text
(numbers.md "15-digit literal parsing"). This needs evidence that each affected function coerces a numeric-text reference, and a
regenerated math sidecar for every suite it touches.
(b) Rewrite the case's guesses as JSON numbers. This changes the case's inputs, so the oracle must be refilled.
For (b), note that the 17-digit guesses read as 15-digit values in Excel either way.

## Acceptance criteria
- [ ] `candidates` lists C157 and the sidecar has its math value or a stated skip; `rustsheet-accuracy math --check` passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
