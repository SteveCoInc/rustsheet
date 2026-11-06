---
id: T-3408
title: "IRR's digits on conventional flows where the secant fails: match Excel's stop (21 cells of the T-3347 cases)"
milestone: M9
lane: functions-financial
status: in-progress
priority: P0
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
IRR returns Excel's value to 15 significant digits on the conventional cash-flow series where the secant model fails
(near r = 0 and near -10%): today T-3347's last stage returns the exact root (within 1.5e-11), which is 1e-13 to
2.2e-11 relative from Excel's double on 21 of the 162 cells of the T-3347 cases.

## Context
- Corrected 2026-11-06 (claim): the Goal above assumed T-3347's Newton stage `irr_conventional` landed. It did not:
  draft #2529 closed unmerged, and T-3347 merged (#3580) blocked, time-boxed (decision 0056), with only its
  `#DIV/0!` flat-step rule (decision 0316). There is no `irr_conventional`/`irr_newton` stage; the engine runs the
  secant, then its fallbacks (crates/rustsheet-engine/src/functions/financial/financial_util.rs). T-3347 no longer
  gates this task (`depends_on` cleared).
- State on origin/main 73e09221b (`rustsheet-eval run --suite probes --suite probes-numeric --filter irr`):
  - probes/t3347-irr-conventional-num A10-A13, A16, A17, A19, A37-A40 (11): all `#NUM!` for Excel's root.
  - probes-numeric/t3347-irr-conventional-sweep: 23 cells `#NUM!` for Excel's root; A33 and A82 numeric_close.
  - The `#NUM!` cells are T-3347's blocked cells, accepted under decision 0267 item 4 (T-3347's "Blocked
    because"). They are out of this task's scope.
- In scope: the cells where the engine returns a root on digits other than Excel's (numeric_close, and
  t3411-irr-r4-fresh A53 numeric_mismatch): the sweep's A33 and A82, the hold-out's four below and the t3411 cells.
- Digit cells of T-3347's hold-out, right type, beyond 15 digits (gap triage 2026-11-01):
- probes-numeric/t3347-irr-conventional-holdout.case.json (4): P!A12, A20, A22, A41
- Related: T-2928 (blocked: the v = 0 fallback's digits, the same question for non-conventional flows).
- Any fit follows decision 0188 R4 (hold-out split, fresh sweep, `FITTED` tag).

## Acceptance criteria
- [ ] Every cell of the two T-3347 cases is within 15 significant digits of Excel's value, with the right type.
- [ ] A fresh hold-out sweep (decision 0188 R4.1) shows the same, and the Gate A floor still holds.
- [ ] Oracle cells for the shapes the stage takes but T-3347 did not probe: the mirrored sign (a positive first flow,
      then negative ones) and zero flows after the outlay; the stage is narrowed if Excel differs there.
- [ ] No gated cell regresses.
- [ ] `make check` green; reviewers approve.


## Cells of T-3411's fresh sweeps
T-3411's R4 sweeps 3 and 4 (cases added by PR #3063, scored on its round-4 model), where the engine returns the root on digits other than Excel's:
- probes-numeric/t3411-irr-r4-fresh-3.case.json (7): Calc!A53, A58, A62, A66, A73, A106, A134
- probes-numeric/t3411-irr-r4-fresh-4.case.json (2): Calc!A58, A75
- probes-numeric/t3411-irr-residual-scale.case.json (7): Calc!A10, A22, A23, A31, A32, A48, A49

## Cells left by T-3411's close (gap triage 2026-11-04b)
T-3411's R4 sweeps 1 and 2, which T-3411 owned until it closed and no open task named after it: the engine returns
the root on digits more than 15 significant digits from Excel's (`make eval` on origin/main a7b115e9e; e.g. fresh
A53 Excel -0.00016023194079306524, engine -0.00016023193850578377). Lifting the fallback's Newton cap moves none.
- probes-numeric/t3411-irr-r4-fresh.case.json (7): Calc!A53, A63, A67, A68, A74, A75, A146
- probes-numeric/t3411-irr-r4-fresh-2.case.json (7): Calc!A53, A60, A64, A75, A119, A125, A147

## Out of scope
- Bit-level parity (Phase 1b, decision 0188 R5).

## Decision 0267
The owner's cut-loss plan (decision 0267, 2026-11-04) makes this a Tier A task, worked first: its cells have the wrong type or error, or are first wrong at the 11th digit or earlier. Decision 0056's time-box applies. If two attempts do not move the cells, block the task as time-boxed, and decision 0267 accepts the cells that are left.
