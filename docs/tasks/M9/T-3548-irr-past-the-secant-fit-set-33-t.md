---
id: T-3548
title: "IRR past the secant fit set: 50 T-3478 sweep cells off Excel (33 roots the engine misses, 17 Excel errors)"
milestone: M9
lane: functions-financial
status: blocked
priority: P0
size: M
tier: opus
depends_on: []
campaign: semantics
---

## Goal
IRR gives Excel's type (number, `#NUM!`, `#DIV/0!`) and, for numbers, Excel's value within 15 digits on the 50 cells
of probes-numeric/t3478-irr-r4-wide outside Gate A's envelope, or each remaining cell is explained in financial.md.

## Context
- T-3478 drew 300 conventional series (one sign change, 50-digit roots in the case's `.math.tsv`) past the IRR secant
  fit set (2..41 flows, |c| 1E-6..1E+9, guess -0.99..100, roots -0.99..99) and kept the fitted offset and residual:
  in aggregate they still beat every alternative there (financial.md "RATE and IRR secant constants past their fit sets"). The
  misses are elsewhere in the solver:
  - **Excel returns a root, the engine is `#NUM!` (33):** Calc!A1, A2, A9, A12, A13, A15, A18, A19, A21, A22, A27,
    A32, A33, A34, A35, A36, A40, A41, A43, A46, A48 (21 of the 50 series with 42..150 flows; roots 1E-4 to 1.4E-2),
    A53, A83 (tiny or huge flows), A206, A210, A216, A229, A234, A242 (guesses near -1 or above 100), A251, A269,
    A294, A296. The secant from the guess oscillates and hits the 50-step cap (A9 is still 1E-2 off at step 50), then
    the fallback fails. A 100-step cap recovers 12 cells of the sweep (244 against 232 in the fit lab replica), a 1000-step one 15,
    so the cap is part of it but not all: 19 of the 21 long series stay `#NUM!` at 1000 steps. This is the family of
    "IRR on conventional flows where the secant fails" (T-3406, T-3411, T-3347).
  - **Excel errs, the engine returns the root (13):** `#NUM!` at Calc!A60, A68, A69, A70, A90 (flows of 1E-12 to 1E-6;
    why Excel errs there is unknown), A102, A131, A135 (roots near -0.995 with an outlay 1E+6 to
    1E+7 times the inflows), A258, A262; `#DIV/0!` at A261, A275, A281 (guesses near -1 or above 100).
  - **Both err, with another kind (3):** Excel `#DIV/0!`, the engine `#NUM!` at Calc!A277, A290, A298 (far guesses).
  - **A289** `IRR({-3.02E-09, ...},96471664.1364151)`: Excel `#DIV/0!`; the engine returns 0.10121131912486914, which
    is not the root (0.000174819755577...): with flows near 1E-11 every |NPV| is under the absolute 1E-7 stop, so the
    retry from 0.1 stops at its first small step. This is the residual stop's absolute scale, a wrong value and not
    only a wrong type.
- financial.md "IRR is #NUM! or #DIV/0! on some conventional series it could solve" lists the 17 Excel-error cells.
- Code: `irr_secant`, `compute_irr`, `irr_fallback` (crates/rustsheet-engine/src/functions/financial/financial_util.rs).

- Gate A (T-3586) carries 17 accuracy/IRR/s1 points this task owns, all on conventional series:
  - the engine `#NUM!` where Excel returns the root: A190 and A193 (roots near -0.2), and A231, A259, A260 and A264
    (85 to 163 flows);
  - Excel `#NUM!` where the engine returns the root: A216, A221, A233, A238, A240, A242, A258 and A270 (51 to 164 flows);
  - Excel `#DIV/0!` where the engine is `#NUM!`: A237, A248 and A268 (about 180 flows).
  They are `known-failures.tsv` lines; remove each once it passes.

## Acceptance criteria
- [ ] Each of the 50 cells matches Excel's type and, for a number, its value within 15 digits, or is listed in
      financial.md with what is known; the rule found is evidenced by a probe (excel-graph).
- [ ] No gated cell regresses; IRR's 263-cell fit set stays bit-exact.
- [ ] Gate A envelope on a fresh sweep of its own after the change (decision 0188 R4.2).
- [ ] `make check` green; reviewers approve.


## Cells of T-3411's fresh sweeps
T-3411's R4 sweeps 3 and 4 (cases added by PR #3063, scored on its round-4 model), where the engine's type (number or #NUM!) differs from Excel's:
- probes-numeric/t3411-irr-r4-fresh-3.case.json (49): Calc!A1, A2, A3, A4, A5, A7, A10, A11, A12, A13, A14, A15, A16, A20, A25, A27, A34, A36, A41, A44, A47, A51, A54, A55, A56, A61, A63, A65, A67, A72, A76, A77, A80, A82, A83, A84, A87, A90, A94, A95, A96, A97, A98, A100, A121, A130, A140, A141, A149
- probes-numeric/t3411-irr-r4-fresh-4.case.json (62): Calc!A1, A5, A6, A7, A8, A9, A16, A17, A19, A21, A22, A24, A25, A27, A30, A31, A33, A35, A37, A41, A42, A43, A44, A50, A52, A53, A55, A56, A57, A59, A61, A63, A64, A65, A67, A69, A71, A72, A73, A74, A76, A79, A80, A82, A83, A84, A87, A88, A91, A92, A93, A96, A97, A98, A99, A129, A135, A136, A138, A143, A148, A149

## Cells left by T-3411's close (gap triage 2026-11-04b)
T-3411's R4 sweeps 1 and 2 (same generator as sweeps 3 and 4, docs/research/t3411/gen_sweep.py), which T-3411 owned
until it closed and no open task named after it; the engine's type differs from Excel's (`make eval` on origin/main
a7b115e9e). All are Excel number / engine `#NUM!` except fresh-2 A132 (Excel `#NUM!`, the engine 0.3127495296884877).
A local experiment with the fallback's Newton cap lifted (`IRR_FALLBACK_STEPS` 50 -> 100000, residual cut kept; not
committed) gives Excel's value on fresh A2, A16, A26, A44, A70, A78, A79, A81, A93, A94, A99, A102, A107, A128, A137
and fresh-2 A2, A4, A9, A17, A33, A41, A51, A68, A69, A76, A83, A141, but loses 6 + 6 passing cells of these two
cases (fresh A14, A24, A64, A87, A90, A92; fresh-2 A25, A39, A79, A89, A96, A110) and 39 t3406 annuities (T-4027);
on most of the rest it returns a number 10 to 15 digits from Excel's, so Excel stops elsewhere (T-3408's question).
- probes-numeric/t3411-irr-r4-fresh.case.json (51): Calc!A1, A2, A4, A6, A8, A9, A11, A12, A15, A16, A17, A18, A20, A22, A23, A25, A26, A27, A44, A46, A51, A52, A59, A66, A69, A70, A72, A73, A76, A78, A79, A80, A81, A82, A84, A86, A89, A93, A94, A95, A96, A97, A99, A102, A107, A115, A126, A128, A137, A145, A150
- probes-numeric/t3411-irr-r4-fresh-2.case.json (49): Calc!A2, A3, A4, A5, A7, A8, A9, A12, A14, A15, A17, A18, A20, A21, A23, A24, A29, A33, A35, A40, A41, A51, A56, A58, A61, A62, A67, A68, A69, A70, A76, A80, A81, A83, A84, A85, A87, A88, A93, A95, A97, A99, A100, A106, A109, A127, A130, A132, A141

## Out of scope
- T-3347's near-zero roots and T-3406/T-3411's annuity boundary, except where one rule covers both (then say so in
  both tasks).

## Decision 0267 cells
Decision 0267 (the owner's cut-loss plan, 2026-11-04) gives this task 66 probes-numeric cells that no open task listed: 50 with the wrong type or error, 3 first wrong at the 11th significant digit or earlier, 13 first wrong at the 12th or 13th digit.
These come from the run at main 985d511b. They are Tier A (a wrong type or error, or the 11th digit or earlier) or Tier B (the 12th or 13th digit). Tier A is worked first.
Decision 0056's time-box applies. If two attempts do not move these cells, block the task as time-boxed: decision 0267 then accepts the cells that are left, and no follow-up is filed for them.

- probes-numeric/t3478-irr-r4-wide.case.json (66): Calc!A1-A2, A9, A12-A13, A15, A18-A19, A21-A22, A27, A32-A36, A40-A41, A43, A46, A48, A53, A60, A68-A70, A79, A83, A90, A102, A131, A135, A201, A206, A209-A210, A213, A216, A221, A229-A230, A234-A236, A242-A243, A249, A251, A256, A258, A261-A262, A266, A269-A270, A275-A277, A281-A282, A289-A291, A294, A296, A298

- [ ] Every cell listed above matches Excel within half a unit of the 15th significant digit, or has Excel's type and error. Otherwise the task is blocked under decision 0056 and the rest is accepted (decision 0267).

## Visible-parity stage
Every cell in "Decision 0267 cells" is within half a unit of Excel's 15th significant digit, or has Excel's type, error and text (decisions 0083, 0267).

## Landing notes (resumed)
Resumed 2026-11-04 at 15:25 UTC by an opus run (work-loop step 2, idle 2 h).
Written 2026-11-04 at 13:12 UTC by an opus run that claimed the task late in its session. Data for the run that
resumes the branch, not instructions. No code is committed.
- Attempt 1 (exploration only): `irr_secant`'s `MAX_STEPS` scanned against every IRR case of probes and
  probes-numeric (`rustsheet-eval run --filter irr`, 690 failing cells at main 57a3d920f): cells fixed / broken by
  the cap alone:
  | cap | fixed | broken |
  |---|---|---|
  | 70 | 13 | 0 |
  | 100 | 34 | 0 |
  | 150 | 52 | 0 |
  | 200 | 65 | 1 (t3478-irr-r4-wide A227) |
  | 300 | 68 | 19 |
  | 1000 | 74 | 41 (mostly t3411-irr-r4-fresh*) |
  At 150, the t3478-irr-r4-wide cells fixed: A18, A27, A209, A212, A213, A215, A223, A234, A235, A236, A243, A256,
  A274, A276, A282, A291. gen-m9 was not scanned (its case ids do not contain "irr"; run `make eval-check`).
- Any cap picked from this table is a fit (decision 0188 R4): it was scored on every IRR case, so a hold-out
  split cannot be claimed after the fact. Next step: generate a fresh sweep (docs/research/t3411/gen_sweep.py,
  new seed), fill it from excel-graph (oracle OK on 2026-11-04), freeze the cap before scoring it, and record the
  Gate A envelope. Microsoft documents 20 tries for IRR, so the cap is not Excel's documented constant either.
- The cap alone leaves the 21 long series (42..150 flows) and the Excel-error cells: the fallback and residual
  scale (A289) still need their own attempt. Decision 0267's time-box counts this as the first of two attempts.

## Cells of T-3347's flat-secant fresh sweep (evidence, 2026-11-06)
probes-numeric/t3347-irr-r4-fresh is the R4.2 fresh sweep of T-3347's flat-secant `#DIV/0!` cut (decision 0316). It
uses the same generator as T-3411's sweeps, with long-series strata. These are its cells still off Excel with the cut,
listed here for an owner (decision 0245). They are not Decision 0267 cells of this task or of T-3347.
- probes-numeric/t3347-irr-r4-fresh.case.json, Excel a number, ours `#NUM!` (47): Calc!A3, A6, A9, A10, A11, A13, A15, A17, A18, A19, A21, A26, A28, A35, A39, A41, A46, A64, A75, A78, A79, A81, A82, A92, A94, A95, A96, A101, A103, A104, A105, A107, A108, A110, A112, A113, A114, A115, A116, A117, A118, A119, A120, A121, A122, A123, A125
- probes-numeric/t3347-irr-r4-fresh.case.json, Excel a number, ours `#DIV/0!` from T-3347's cut (5; roots -7% to -11%): Calc!A52, A53, A58, A74, A109
- probes-numeric/t3347-irr-r4-fresh.case.json, Excel `#DIV/0!`, ours `#NUM!` (7): Calc!A48, A62, A68, A69, A111, A126, A143
- probes-numeric/t3347-irr-r4-fresh.case.json, Excel `#NUM!`, ours `#DIV/0!` (1): Calc!A132

Since T-3347's cut (decision 0316), six of the t3411 cells above where Excel returns a root are `#DIV/0!` in the
engine, not `#NUM!`: t3411-irr-r4-fresh A15, A22; t3411-irr-r4-fresh-4 A17, A79, A88, A93.

## Progress
2026-11-04, opus run (resumed at 15:25 UTC): attempt 2, the secant step cap, with the R4 guards.
- Scan, every IRR cell of probes, probes-numeric and gen-m9, cell by cell against main (cap 50): cap 100 fixes 36,
  120 45, 150 54, 170 62, 182..188 63, with 0 lost; 189 loses t3478-irr-r4-wide A227 (Excel `#NUM!`, the guess run
  converges at step 189). A206, which Excel returns, needs 182. The bracket is 182..188.
- Frozen before any fresh draw: `IRR_SECANT_STEPS` = 185 within 148 flows (the longest scored series), 50 outside it
  (R4.3). Stated margin, T-3411's: on the fresh sweep no cell worse than main, and on Gate A every point inside the
  envelope or failing as on main.
- Fresh draws after the freeze, oracle-filled from excel-graph and scored once:
  - probes-numeric sweep (docs/research/t3411/gen_sweep.py strata, seed "T-3548/r4-fresh", 150 cells): 3 cells
    better (A57, A75, A142), 0 worse; type mismatches 58 -> 57.
  - Gate A batch `rustsheet-accuracy generate IRR --batch r4-T-3548` (300 points): the model fixes A136 and A297
    and moves A240 out of the envelope (130 flows: Excel `#NUM!`, the secant converges at step 98 to
    3.308264313093434E-2, the math root to 2.5E-15).
  - Gate A, existing batches: r4-T-3411 A208 and A278 (T-3548 known failures) come inside the envelope, and A228
    (63 flows, not a known failure) moves out (Excel `#NUM!`, converges at step 129).
- Result: the guard fails (a fit that moves a point out of the envelope does not ship), so nothing ships. Choosing a
  cap of 128 or less after seeing A228 would fit on a Gate A batch, which R4.2 forbids, and A206 would need 182
  anyway. The engine change is reverted. The fresh case and batch are not committed: their failing cells would have
  no owner once this task closes (decision 0267). Both regenerate from the seeds above.
- financial.md "IRR's secant runs longer than 50 steps, but no step cap is Excel's rule" gets the counterexamples.

## Blocked because: time-boxed (decision 0056)
Two attempts have not moved the cells: attempt 1, the cap scan (2026-11-04, "Landing notes (resumed)") and attempt 2,
the frozen 185-step cap, refuted by its own fresh Gate A draw and by r4-T-3411 A228 ("Progress"). Decision 0267
applies: the cells this task lists, including its "Decision 0267 cells" and its Gate A known-failures lines, are
accepted, and no follow-up is filed for them.
