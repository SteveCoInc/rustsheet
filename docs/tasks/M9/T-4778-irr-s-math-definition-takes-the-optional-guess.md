---
id: T-4778
title: "IRR's math definition takes the optional guess: 25 counted IRR cells with a guess have no math value (argument count)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`rustsheet-accuracy math` gives IRR cells that pass a guess (`=IRR({42,1,1,-1},0.5)`) the same math value as the call
without it, as RATE's definition already does. Then the 25 visibly-off IRR cells below get a math value, and decision
0188 R1(c) sorts them by it. Today they count only because they have no math value (decision 0325).

## Context
- Gap triage 2026-11-07 (`make eval` at b9ae5216f, excel-graph). `eval_math.candidates` gives every cell below the
  skip reason `argument count`. `tools/src/rustsheet_tools/accuracy/defs/cashflow.py` registers IRR with the one
  parameter `values`, so a call with a second argument does not fit (eval_math module doc: "the call has a different
  number of arguments than the definition").
- The pattern to copy is RATE in `defs/annuity.py`, which gives its guess as
  `call_params=(Param("guess", "real", _AMOUNT, source=RATE_SOURCE, optional=True),)`, with the comment "The guess only
  picks among several roots, and a point here has one". `_rate` drops it (`real_args(args)[:5]`). IRR's `flow_rate`
  already raises NoValueError at several sign changes, so a guess never chooses a root here. The multi-root cells are
  T-4779's.
- t2565-irr-guess-sweeps P!C157 passes its guess as `B157`, a constant cell, which `parse_arg` reads. If a guess is a
  formula cell instead, the cell stays skipped (`unparsed`). That is T-4777's rule, not this task's.
- Owners of these cells' fix work, which keep them: T-2131 and T-2928 (irr-fallback-roots, t2565), T-2928 and T-3117
  (t2927), T-3548 (t3408, t3478), T-3347, T-4213 and T-4338 (t3411 fresh), T-4038 and T-4213 (t3411 residual-scale).

Cells this task owns for their math value:
- probes/irr-fallback-roots.case.json (7): P!A12, P!A13, P!A14, P!A15, P!A22, P!A23, P!A24
- probes/t2565-irr-guess-sweeps.case.json (1): P!C157
- probes-numeric/t2927-irr-fallback-sweep.case.json (4): P!B1, P!B11, P!B21, P!B51
- probes-numeric/t3408-irr-r4-fresh.case.json (3): Calc!A2, Calc!A13, Calc!A31
- probes-numeric/t3411-irr-r4-fresh-2.case.json (3): Calc!A64, Calc!A119, Calc!A125
- probes-numeric/t3411-irr-r4-fresh.case.json (1): Calc!A146
- probes-numeric/t3411-irr-residual-scale.case.json (4): Calc!A22, Calc!A23, Calc!A31, Calc!A32
- probes-numeric/t3478-irr-r4-wide.case.json (2): Calc!A79, Calc!A292

## Acceptance criteria
- [ ] IRR's definition has an optional `guess` call parameter (cited like RATE's), and `_irr` ignores it. A unit test
      shows `IRR({42,1,1,-1},0.5)` and `IRR({42,1,1,-1})` give the same math value.
- [ ] `cd tools && uv run rustsheet-accuracy math --suite probes` and `--suite probes-numeric` regenerate the eight
      sidecars, and each cell above has a math value or a NoValueError skip (`no single value`) listed in the Outcome.
      `rustsheet-accuracy math --check` passes, and `rustsheet-accuracy verify IRR` shows no change.
- [ ] After `make eval`, the Outcome lists how many of the 25 cells are Phase 1b evidence (`r1c_left_out`) and how many
      still count, with Excel-minus-math for those. `eval/baseline/` gains no new line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- XIRR's guess (no counted XIRR cell lacks a math value for it today).
- IRR or RATE with several sign changes (T-4779), and any engine change to IRR.
