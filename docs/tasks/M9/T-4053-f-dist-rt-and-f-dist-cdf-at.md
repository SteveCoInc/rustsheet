---
id: T-4053
title: "F.DIST.RT at df1 = 2 and a huge df2: Excel is up to 2E-8 off the closed-form tail (5 t4017 cells, decision 0267 Tier A)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
At df1 = 2 and a huge df2, the 5 listed F.DIST.RT cells match Excel's value. There Excel is the inaccurate side: it is
up to 2E-8 off the true tail, and its error grows with x. The task finds which arithmetic Excel uses and reproduces it.

## Context
- docs/excel-semantics/statistics.md, "Incomplete beta at a huge first shape", has the T-4017 sweep paragraph at df1 2.
  The 50-digit references are in docs/research/t4017/references.json.
- At b = 1 the true tail is the closed form (1 + 2x/df2)^(-df2/2). For example, `F.DIST.RT(20,2,1E8)` is
  2.061161907818132E-9 in Excel, against the true 2.0611618658819E-9. Ours returns the math.
- Negative result (decision 0267 analysis, 2026-11-04): Excel matches neither form below.
  - `POWER(1+2x/df2, -df2/2)` on the rounded base: 2.061161865881937E-9.
  - `EXP(-df2/2*LN(1+2x/df2))`.
  - `EXP(-df2/2*LN1P(2x/df2))` comes within 1E-13 at x = 10, but is 2E-9 off at x = 20.
  Excel's error grows with x, which points to the branch TOMS 708 takes at b = 1 with a huge a (the a0 = 1 `w1`
  complement or `bup`, compare T-3764), not to the closed form.
- Excel is about 2E-8 relative off the math, below decision 0188 R3's G = 1E-4, so Excel stays the target.
- The other t4017 quirks have their own tasks: the exact zeros (T-4045) and the `#NUM!` at df1 79 (T-4044, T-4030).
  The engine's own accuracy at df1 >= 3 belongs to T-3754.
- Code: `f_tail` and `kernel_tails` in crates/rustsheet-engine/src/functions/statistical/fisher.rs, and
  `large_shape_beta.rs`.

## Acceptance criteria
- [ ] statistics.md records Excel's df1 = 2 arithmetic and its evidence, and the engine applies it with a comment
      that cites the evidence.
- [ ] The listed cells pass. Or, per decision 0056, two attempts that do not move them block the task as time-boxed,
      and decision 0267 accepts the rest.
- [ ] No t4017, gen-m9 or probes cell that is within 15 digits today moves off. `make check` is green; reviewers
      approve.

## Out of scope
- T-4044, T-4045, T-3754 and T-3209.

## Decision 0267 cells
Decision 0267 (the owner's cut-loss plan, 2026-11-04) gives this task 5 probes-numeric cells that no open task listed: 2 first wrong at the 11th significant digit or earlier, 3 first wrong at the 12th or 13th digit.
These come from the run at main 985d511b. They are Tier A (a wrong type or error, or the 11th digit or earlier) or Tier B (the 12th or 13th digit). Tier A is worked first.
Decision 0056's time-box applies. If two attempts do not move these cells, block the task as time-boxed: decision 0267 then accepts the cells that are left, and no follow-up is filed for them.

- probes-numeric/t4017-f-huge-df2-above-mean-1e7.case.json (1): Calc!A9
- probes-numeric/t4017-f-huge-df2-above-mean-1e8.case.json (1): Calc!A13
- probes-numeric/t4017-f-huge-df2-above-mean-1e9.case.json (1): Calc!A11
- probes-numeric/t4017-f-huge-df2-above-mean-3e7.case.json (2): Calc!A11, A13

- [ ] Every cell listed above matches Excel within half a unit of the 15th significant digit, or has Excel's type and error. Otherwise the task is blocked under decision 0056 and the rest is accepted (decision 0267).

## Visible-parity stage
Every cell in "Decision 0267 cells" is within half a unit of Excel's 15th significant digit, or has Excel's type, error and text (decisions 0083, 0267).
