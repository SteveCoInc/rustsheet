---
id: T-5099
title: "Read a written-empty wrapper's pick once: the call reuses the probe's pick, so a random draw or an opaque callee hands an omission on too (T-4912 follow-up)"
milestone: M5
lane: functions-math
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A written-empty branch that an IF, CHOOSE, IFERROR or IFNA picks is an omission for every reader, whatever the
wrapper's condition calls: `MROUND(IF(RAND()<0.5,,1),1)` is `#N/A` or 1, never 0, and so are the forms whose
condition calls a LET-bound or defined-name LAMBDA or reads a defined name.

## Context
- T-4912 (`Model::is_empty_argument`, functions/util/empty_args.rs) tells a picked written-empty branch by running
  the wrapper's pick step before the call evaluates the argument, so the condition is evaluated twice. To keep the two
  reads from disagreeing it skips a wrapper whose condition draws a random value (`RANDOM_DRAWS`) or holds a call
  whose body the walk cannot see (a name call, a defined name). Those forms read the branch as 0, where Excel's one
  evaluation hands the omission on (docs/excel-semantics/numbers.md "Which wrappers hand a written-empty branch on
  as an omission (T-5067)", Ours; the known-gap test `a_wrapper_whose_pick_cannot_be_repeated_is_not_probed`).
- Design to start from: the probe's pick is kept for the call to reuse (the pick step already separates the pick
  from the picked value, `pick_step`/`picked_value`), or the picked omission travels with the value. Either removes
  the second evaluation and the exclusion, and the `MAX_OMISSION_PROBES` DEVIATION with it.
- Probe with the oracle as well (excel-probe skill), since no cell covers them: `LET(e,,MROUND((e),1))` (ours
  `#N/A`), `LET(e,,MROUND(IF(TRUE,e),1))` (ours 0) and `MROUND(IFERROR(Z1,),1)` with Z1 an error cell (ours 0).

## Acceptance criteria
- [ ] The wrapper's condition, index or IFERROR value is evaluated once per argument read; the random-draw and
      opaque-callee exclusion is gone, and the known-gap test asserts `#N/A`.
- [ ] The three unprobed forms above are probed and matched.
- [ ] t4901, t5067 and t4912 keep passing.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
