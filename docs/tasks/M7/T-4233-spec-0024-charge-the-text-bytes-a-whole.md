---
id: T-4233
title: "SPEC-0024: charge the text bytes a whole-array reader compares (lookups, MATCH, UNIQUE, SORT, the *IF(S)) over a bound array of long texts"
milestone: M7
lane: engine
status: in-progress
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
A LAMBDA that repeats a text-comparing reader over a LET-bound array of long texts stays within the formula work budget:
the bytes the reader compares are charged, not only the cells it copies (SPEC-0024 section 1, "text values").

## Context
T-2805 charges each read of a bound array by its cells (`evaluate_node_with_reference`, `variable_reference`), but
not by the bytes of its text elements. Those bytes are shared since T-2806, so the copy is cheap, and charging every read
by length would refuse `IF(SEQUENCE(1024,1024),REPT("a",32767))`, which computes today. A reader that compares texts
byte by byte does more work than the copy, though. For example, in
`LET(s,IF(SEQUENCE(1E5),REPT("a",32767)),MAP(SEQUENCE(900),LAMBDA(i,MATCH(REPT("a",32766)&"b",s,0))))`, each call compares
up to 32 KB per element, which is about 3 GB a call within the cell charge. Find the comparing readers (MATCH/XMATCH/XLOOKUP/LOOKUP and
the V/HLOOKUP searches, UNIQUE, SORT/SORTBY, the *IF(S) criteria matching, GROUPBY keys) and charge the bytes they
compare, at a shared comparison chokepoint if one exists (directive 3). See docs/excel-semantics/arrays.md "One formula
evaluation does at most 268,435,456 work units".

## Acceptance criteria
- [ ] The MATCH formula above, and one formula per other comparing reader, is `#VALUE!` (`TOO_MUCH_WORK`) under the
      default limit (engine tests that count units, not time).
- [ ] No baseline gains a line; the DEVIATION entry lists the text-byte charge.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cell charges for array reads (T-2805, done).

## Progress notes (2026-11-04, opus run; work in progress, not a handoff)
Data from the run that started this branch, for whoever resumes it (check it against the code):
- WIP on this branch: a thread-local byte counter in `functions/util/collation.rs` (`note_compared_bytes`,
  `take_compared_bytes`), noted in `cmp_text_in` (left + right bytes), `collation_key`, `fold_lookup` and the ASCII arms
  of `criteria::folded_units` and `WildcardPattern::matches`; `Model::spend_compared` (area_budget.rs) charges it after
  each node in `evaluate_node_in_context`, and `with_area_budget` sets an outer run's pending bytes aside. Builds; the
  two new tests in test/test_formula_work.rs (`TEXT_COMPARERS`) fail as committed.
- Measured deltas (units for 100 texts going from 8 to 4,008 bytes; 100,000 wanted): XMATCH, XLOOKUP, `s=t`, SORT, SORTBY,
  UNIQUE, wildcard MATCH pass; MATCH 0, VLOOKUP and HLOOKUP FALSE give 5,000 because `ExactKey::text_is`
  (lookup_and_reference/lookup_table.rs) compares ASCII bytes itself, outside the collation module: note there too.
  MATCH 1 and LOOKUP (binary search, log n compares) are right to pay less: test them with a smaller bound. COUNTIF,
  COUNTIFS and SUMIF give about 4,000 because a criterion over 255 characters is `#VALUE!` before any compare
  (`if_ifs.rs` TOO_LONG): put the long texts in the range and keep the criterion short. GROUPBY gives 3,000: not yet
  understood (its key is `unique_cell_key` -> `collation_key`, which notes).
- Open design point: `left + right` per compare is a bound, not the work; an early difference stops the passes, so a
  short criterion against 32 KB texts is overcharged 32 KB per element (could refuse MAP-repeated COUNTIFs that compute
  today), while charging `min` undercharges weightless runs (`"a"&REPT("-",32766)` vs `"b"` scans the hyphens). Counting
  the units the pass iterators actually consume would be exact; decide before finishing. The task's MATCH example
  (a 32,767-byte target against `s`) compares each element in `text_is` (same length, so a full byte walk).

