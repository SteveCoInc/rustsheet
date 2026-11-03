---
id: T-3887
title: "VSTACK/HSTACK: probe a variable bound to a blank cell; name T-3765's helpers in arrays.md"
milestone: M7
lane: functions-lookup
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence shows what VSTACK and HSTACK do with a LET variable or LAMBDA parameter bound to a one-cell **blank**
reference, and the engine matches it; docs/excel-semantics/arrays.md names T-3765's helpers.

## Context
- Follow-up of T-3765 (PR #2891), from its excel-fidelity and design reviews (MINOR findings).
- Since T-3765, `fn_vstack` sends a variable bound to a one-cell reference through `eval_to_array_keep_blank`
  (`crates/rustsheet-engine/src/functions/lookup_and_reference/hstack_vstack.rs`), so `LET(z,Z1,VSTACK(z,1))` with a
  blank Z1 may now stack a blank element where it used to stack 0; HSTACK's blank path did not change, so the two may
  differ. No probe covers a bare variable bound to a blank cell (t3053 A21 is a LET wrapper; t2413 is direct refs only).
- Probe (excel-probe skill) cells such as `=ISBLANK(INDEX(LET(z,Z1,VSTACK(z,1)),1))`,
  `=ISBLANK(INDEX(MAP(Z1,LAMBDA(z,VSTACK(z,1))),1))`, the HSTACK forms and the value forms (`=INDEX(...,1)&"x"`).
- arrays.md's "Ours:" line of the VSTACK/HSTACK one-cell error reference entry names only
  `one_cell_error_ref_as_array`; add `Model::is_one_cell_variable` / `stack_error_ref_as_array` (T-3765).
- Design note: `stack_error_ref_as_array` (spill_functions.rs) repeats the 1x1 error wrap of
  `wrap_one_cell_error_ref` with an extra variable condition; fold it into the shared helper if this task touches it.

## Acceptance criteria
- [ ] A probe case with excel-graph oracle values covers blank-cell variables in VSTACK and HSTACK (LET and MAP).
- [ ] The engine matches every probe cell (or each failing cell is owned by a task).
- [ ] arrays.md's "Ours:" line names the T-3765 helpers.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other stacking functions (TOCOL, TOROW, CHOOSECOLS).

## Outcome
Added probe `probes/t3887-stack-blank-variable` (16 cells, excel-graph oracle): VSTACK/HSTACK of a LET variable and a MAP
parameter bound to a blank cell. The engine already matched all 16 cells, so no engine change; arrays.md records the
findings and names T-3765's helpers. The shared-helper fold-in was not needed (no engine change). Eval delta: +16 passing
probe cells, none failing.
