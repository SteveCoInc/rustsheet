---
id: T-4465
title: "Array-constructor extents just below an integer: is SEQUENCE's 2^-23 snap the shared near-integer rule, and do RANDARRAY, MAKEARRAY, TAKE, DROP, EXPAND and WRAPROWS snap too"
milestone: M7
lane: functions-math
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
The whole number each M7 array constructor reads from a rows, columns, count or wrap argument just below (and just
above) an integer matches Excel, evidenced on excel-graph, and the engine reads it through one definition per rule
Excel actually has (directive 3), instead of the four ad hoc reads it has today.

## Context
- Gap triage 2026-11-06 (`make eval` on origin/main 07a6b88a). No visibly-off cell is unowned; this is a decision 0031
  evidence gap in M7 functions. arrays.md (the SEQUENCE paragraph of "empty array arguments", T-2580): "An extent is
  floored after adding 2^-23 ... The epsilon is probed near 0, 1 and 1048577 only; for large extents a relative or
  float32 rounding is UNVERIFIED."
- The SEQUENCE evidence does not separate 2^-23 from the engine's shared rule. probes/empty-array-args-sequence
  A120:A134: `SEQUENCE(0.99999999,1)` is 1 and `SEQUENCE(0.999999,1)` `#CALC!` (the up-snap lies between 1E-8 and
  1E-6), `SEQUENCE(4,-1.19E-7)` `#CALC!` and `SEQUENCE(4,-1.2E-7)` `#VALUE!` (the down-snap lies between 1.19E-7 and
  1.2E-7). `near_integer_whole` (functions/util/near_integer.rs: 2^-22 + 2^-33 up, half of it, 2^-23 + 2^-34, down;
  DATE, TIME, XMATCH, XLOOKUP and SORTBY's sort_order, bracketed to 2^-14 relative and shown absolute up to 100000)
  fits every one of those cells, and so does sequence.rs's `EXTENT_EPSILON` (2^-23 both ways). They differ for an
  up-snap between 2^-23 and 2^-22 + 2^-33: `SEQUENCE(1-1.5*2^-23,1)` is 1 under the shared rule and `#CALC!` under
  2^-23. A float32 conversion of the extent (the other hypothesis arrays.md names) differs from both at large extents,
  e.g. `ROWS(SEQUENCE(100000-2^-20))`.
- The other constructors read their extents each its own way, all unprobed near an integer: RANDARRAY floors
  (math_and_trigonometry/random.rs), TAKE and DROP truncate (`as i64`, lookup_and_reference/drop_take.rs), EXPAND
  casts (`as usize`, expand.rs), WRAPROWS/WRAPCOLS truncate after a below-1 check (wrapcols_wraprows.rs), and MAKEARRAY
  refuses any fraction (logical/bycol_byrow.rs, so `MAKEARRAY(2-2^-30,1,..)` is `#VALUE!` in ours).
- RANDARRAY is volatile: read it through invariant wrappers (`ROWS`, `COLUMNS`, `ERROR.TYPE`), as
  probes/t3631-randarray-invariant does, and list those cells as the case file allows (T-1283).
- Predictions of both candidate rules (and of the engine) are committed before the fill: the rule is chosen on this
  probe's cells, so it owes the hold-out of decision 0188 R4 (a second seeded set of offsets, scored only after the
  rule is frozen).

## Acceptance criteria
- [x] A probe case (excel-probe skill, excel-graph, tag M7) steps the rows or count argument of SEQUENCE (rows and
      columns), RANDARRAY, MAKEARRAY, TAKE, DROP, EXPAND and WRAPROWS by 2^-24, 2^-23, 1.5·2^-23, 2^-22, 2^-22 + 2^-33,
      2^-22 + 2^-32 and 2^-21 below and above 1, 2, 1000 and 100000 (and below 0 for the ones that accept 0), read
      through `ROWS`/`COLUMNS`/`ERROR.TYPE`, with a hold-out set of offsets scored after the rule is frozen.
- [x] arrays.md states each constructor's rule (or that it shares `near_integer_whole`'s), citing cells, and drops
      the UNVERIFIED sentence; every probe cell passes, the hold-out included.
- [x] A constructor that shares an existing rule calls that helper; `EXTENT_EPSILON` remains only if the probe shows a
      rule of its own. No other eval cell regresses.
- [x] `make check` green; reviewers approve.

## Out of scope
- Extents past the grid (T-2608) and argument kinds other than numbers (T-2873, T-2874).

## Outcome
- Probe (excel-graph, 2026-11-06; generator docs/research/t4465/gen_probe.py): probes/t4465-extent-near-integer (760
  cells) and -randarray (95) step SEQUENCE's rows and columns, RANDARRAY, MAKEARRAY, TAKE, DROP, EXPAND, WRAPROWS and
  WRAPCOLS by the task's offsets (plus 2^-23 + 2^-34 and 2^-23 + 2^-33) around 0, 1, 2, 1000 and 100000 (and -1 for
  TAKE and DROP). Predictions of five candidate rules and of the engine were committed before the fill (commit "T-4465: probe cases, generator and committed predictions",
  docs/research/t4465/predictions.tsv).
- Rule chosen on the fixed cells and frozen in docs/research/t4465/frozen-rule.md (7d3d69a51) before the hold-out was
  filled. SEQUENCE and RANDARRAY read `near_integer_whole` (0 misses; a 2^-23 epsilon missed 16 to 21 per column, float32
  17 to 19). MAKEARRAY refuses fractions, and TAKE, DROP, EXPAND, WRAPROWS and WRAPCOLS truncate: the engine already
  matched all of these.
- Hold-out (decision 0188 R4): -holdout (240) and -holdout-randarray (30), offsets k*2^-30 seeded from
  random.Random(4465), filled after the freeze: 270/270 under the frozen rule, against 0 to 21 misses per column for
  the others. A fresh sweep of RANDARRAY's columns (-randarray-cols, 76 cells, written after the freeze): 76/76.
- Engine: sequence.rs drops `EXTENT_EPSILON` and calls `near_integer_whole`; random.rs's RANDARRAY rows and columns call
  it too (they floored before). arrays.md: new section "Array-constructor extents near an integer", and the SEQUENCE
  paragraph's UNVERIFIED sentence is dropped. Also recorded there: WRAPROWS/WRAPCOLS give the vector's length, not
  wrap_count, when wrap_count is longer. Probe note: Excel's unary minus binds before `^`, so `-2^-22` is +2^-22.
- Not a numeric fit in the R4 sense: an existing rule (`near_integer_whole`) was selected from ranked structural
  candidates, so no `FITTED` tag; the hold-out and the fresh sweep are recorded above.
- Eval delta: probes +5 cases, 1201 cells, all passing (eval/baseline/probes.txt); 0 regressions in any suite.
  probes/empty-array-args-sequence (117) and t3631-randarray-invariant (45) still pass.
