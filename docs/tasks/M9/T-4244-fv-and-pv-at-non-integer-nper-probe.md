---
id: T-4244
title: "FV and PV at non-integer nper: probe cancelling calls for the twice-rounded form (after T-4146)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4146]
---

## Goal
Evidence on whether FV's and PV's twice-rounded operators (financial.md "FV and PV round each operation twice",
T-4146) also hold at non-integer nper, and the T-4146 record made precise.

## Context
- T-4146 (PR #3226) switched `compute_future_value` and `fn_pv` (annuity.rs) to Excel's twice-rounded `+`, `*`
  and `/` for every nper. Its two probes (docs/research/t4146/gen_probe.py) hold only cancelling calls with integer
  nper 1..400; the repo's two non-integer-nper FV/PV oracle cells (`PV(-1+2^-52,20.5,...)`,
  `PV(-0.9999999999,31.5,...)`) do not cancel, so the form is unevidenced there.
- Excel-fidelity review of #3226 (MINOR): financial.md's status line says "non-integer nper keep their forms", which
  is wrong (only rate = 0 does). The T-4146 Outcome should show the hold-out as bit-exact / within 4 ulp / within 15
  digits for the chosen and the correctly rounded candidate (score.py gives the first two: twice 240/240 both;
  correctly rounded 0 both; plain PV within 4 ulp 21/87 train, 8/33 test), say the fresh sweep is 120/120 inside the
  envelope (no FV/PV Gate A table exists, so it is a probes case), and say both sweeps cover cancelling integer-nper
  calls only.
- Design review of #3226 (MINOR): `annuity_power`'s doc comment wraps "matches all" onto its own line.

## Acceptance criteria
- [ ] A probe of at least 60 cancelling FV/PV calls at non-integer nper (gen_probe.py with a non-integer nper draw),
      committed before its excel-graph fill, scored with docs/research/t4146/score.py (extend `exact` past integer
      nper or drop that candidate there).
- [ ] financial.md's status line and T-4146's Outcome corrected as above; the doc comment rewrapped.
- [ ] If the probe disagrees, the form at non-integer nper follows the evidence; `make check` green; reviewers
      approve.

## Out of scope
- rate = 0.
