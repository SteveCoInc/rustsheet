---
id: T-4453
title: "Near-integer counts in REPT, FIND/SEARCH start_num, CHAR/UNICHAR, SUBSTITUTE instance_num and FIXED/DOLLAR decimals: probe whether LEFT's 2^-22 snap applies, and match"
milestone: M5
lane: functions-text
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Every whole-number argument of the M5 text functions that still reads with a plain floor or
truncation in the engine is read the way Excel reads it just below an integer: either with LEFT's
near-integer snap (a value within 2^-22 + 2^-33 below an integer counts as that integer), with
DATE's asymmetric variant (half the tolerance for a negative value), or exactly truncated, as MID
is. The choice per argument rests on committed excel-graph probe cells, not on analogy.

## Context
- docs/excel-semantics/text.md "LEFT, RIGHT and REPLACE round counts up near an integer" (T-0219,
  probes/text-count-argument-bounds): LEFT, RIGHT, LEFTB and REPLACE snap counts within
  2^-22 + 2^-33 below an integer; MID and MIDB truncate exactly. The entry ends: "whether other
  functions (REPT, FIND/SEARCH start_num, CHAR) do is UNVERIFIED". INDEX, VLOOKUP/HLOOKUP, DATE,
  TIME, XMATCH/XLOOKUP modes and SORTBY's sort_order snap too (`near_integer_count` /
  `near_integer_whole` in crates/rustsheet-engine/src/functions/util/near_integer.rs), so the rule
  is not one global conversion: some arguments snap and some do not (MID; the BIT* number in
  gen-m9/7c7e5bff12d93365 Calc!A145, `BITLSHIFT(0.9999999999,-5)` is `#NUM!`, so 1E-10 below 1 does
  not count as 1 there).
- Existing evidence only covers fractions far from an integer, where floor and snap agree:
  probes/dollar-fixed-unichar-edges Sheet1!A9, A23, A31, D68, D71 (`DOLLAR(1234.5,1.9)` "$1,234.5",
  `FIXED(1234.5,1.9)` "1,234.5", `UNICHAR(65.9)` "A", `UNICHAR(128.9)` U+0080, `FIXED(1,127.9)` 127
  places), probes/t3302-text-funcs-arrays Sheet1!E17 (`CHAR(1.9)` U+0001). No committed cell probes
  any of these arguments within 2^-22 of an integer.
- Engine reads today (all plain): REPT count `floor` (functions/text/common.rs), FIND/SEARCH
  start_num `floor` (functions/text/position.rs), CHAR `floor` and UNICHAR `trunc`
  (functions/text/char_code.rs), SUBSTITUTE instance_num `floor` (functions/text/util.rs), FIXED and
  DOLLAR decimals `trunc` (functions/text/string_format.rs).
- Sibling questions in other lanes: T-4447 (SORT sort_order/sort_index, functions-lookup), the
  date-lane and engineering-lane near-integer tasks filed alongside this one. Reuse
  `near_integer_count` / `near_integer_whole`; do not add a second tolerance constant.
- Opus: no Excel evidence exists yet, and each argument may follow a different one of three
  candidate models (snap, asymmetric snap, exact truncation), so the probe must be designed to
  separate them (both sides of 1 - 2^-22 - 2^-33, a negative just above -1 or 0 where the argument
  allows it, and a large base such as 1000 or 32767 to confirm the bound is absolute).

## Acceptance criteria
- [ ] A committed probe case (excel-probe skill, excel-graph sidecar) brackets the snap bound for
      each of REPT number_times, FIND and SEARCH start_num, CHAR, UNICHAR, SUBSTITUTE
      instance_num, FIXED decimals and DOLLAR decimals: at least the values n - 2^-22 - 2^-33,
      n - 2^-22 - 9*2^-36 and n - 1E-15 for two bases n (one small, one >= 1000 where the argument
      range allows it), plus a negative-side cell for FIXED/DOLLAR decimals (e.g. -1 + 2^-23).
- [ ] docs/excel-semantics/text.md's "LEFT, RIGHT and REPLACE round counts up near an integer"
      entry names, per argument above, which model Excel follows, citing the probe cells, and drops
      the UNVERIFIED sentence for the arguments probed.
- [ ] The engine reads each probed argument with the model its cells show, through the existing
      helpers in functions/util/near_integer.rs (no new tolerance constant); every probe cell
      passes bit-exact.
- [ ] Unit tests pin one snapping and one non-snapping value per changed argument.
- [ ] No new known-failure line in eval/baseline/ (`make eval-baseline` refuses none); `make check`
      green; reviewers approve.

## Out of scope
- TEXTBEFORE/TEXTAFTER instance_num, TEXTSPLIT and REGEX* occurrence arguments (M7): file a
  follow-up if the probe shows the snap is common to the text functions.
- MID, MIDB, LEFT, RIGHT, REPLACE (already probed, T-0219).
- Non-text lanes (dates, engineering, lookup): their own tasks.
