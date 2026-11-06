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

## Rule frozen (before the hold-out fill)
Main set (probes/t4465-extent-near-integer-main, excel-graph, 63 rows), extent read from SEQUENCE rows (B) and
RANDARRAY (D): shared 63/63 and 63/63, eps 51/51, f32 48/48, floor 41/41, trunc 38/38. Frozen: SEQUENCE (rows and
columns) and RANDARRAY read `near_integer_whole`; TAKE, DROP, EXPAND, WRAPROWS and WRAPCOLS truncate with no snap
(F, G, H, I, J); MAKEARRAY is `#VALUE!` for any fractional extent (E). Hold-out: predictions-holdout.tsv (seed 4465),
split fixed by the generator before any fill; margin: the shared rule must match every hold-out row (a structural
rule, so no rate tolerance).

## Outcome
Probed and implemented. SEQUENCE (rows, columns) and RANDARRAY read an extent by the shared `near_integer_whole`
(sequence.rs's `EXTENT_EPSILON` is removed; RANDARRAY's two copied extent blocks became one `randarray_extent`, and the duplication lock drops 2876 -> 2836). TAKE, DROP, EXPAND, WRAPROWS and
WRAPCOLS truncate with no snap and MAKEARRAY is `#VALUE!` for any fraction, as ours already did. arrays.md: new entry
"Array-constructor extents just below an integer (T-4465)"; the SEQUENCE paragraph's UNVERIFIED sentence is dropped.
- R4 guards (a structural rule chosen from ranked candidates, decision 0188 R4 kind (c); no `FITTED` tag): split fixed
  by the generator before any fill (main 63 rows; hold-out seed 4465, 36 rows), predictions committed before the main
  fill (38629e6be froze the rule before the hold-out fill). Cells matched, all columns (`docs/research/t4465/score.py`):
  | set | frozen rule | shared everywhere | eps | f32 | floor | trunc |
  |---|---|---|---|---|---|---|
  | main | 539/539 | 449 | 464 | 435 | 464 | 469 |
  | hold-out | 318/318 | 273 | 276 | 260 | 268 | 268 |
  | fresh sweep (seed 44651, generated after the freeze) | 316/316 | 290 | 290 | 254 | 266 | 281 |
  Margin stated at the freeze: every hold-out row. Gate A envelope: not applicable (the result is a shape, not a
  number). The WRAPROWS/WRAPCOLS shape predictor was corrected after the hold-out scored (a wrap of 3 or more gives the
  3-cell vector unpadded, main I29:J29); it does not touch the extent rule.
- Eval delta: probes +3 cases (main 602, hold-out 354, fresh 352 cells), all pass; 0 regressions in every suite.
