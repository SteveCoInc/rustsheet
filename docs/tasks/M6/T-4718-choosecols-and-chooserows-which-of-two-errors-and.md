---
id: T-4718
title: "CHOOSECOLS and CHOOSEROWS: which of two errors, and uncoercible text against a later error, wins in the index list (lookup.md UNVERIFIED): probe and match"
milestone: M6
lane: functions-lookup
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel cells decide which result CHOOSECOLS and CHOOSEROWS give when their index arguments hold more than one
problem, a case docs/excel-semantics/lookup.md leaves UNVERIFIED. The engine matches every probe cell, and lookup.md
states the observed order in place of the two UNVERIFIED sentences.

## Context
- Gap triage 2026-11-06 (opus worker run, `make eval` on origin/main 4def66f7 with calipers, corpus-bench and poi
  fetched). `scripts/exit_owners.py --all-off` reports 0 visibly-off cells with no open owner, so this lane's
  UNVERIFIED notes with no open task are the gap left. CHOOSECOLS and CHOOSEROWS are M6 in eval/catalog/functions.yaml
  (prior 0.2).
- The UNVERIFIED items, both in lookup.md:
  1. "The value-type lift" entry (around line 939): "which problem wins inside a CHOOSECOLS/CHOOSEROWS index list
     between uncoercible text and a later error, or between two errors".
  2. "CHOOSEROWS and CHOOSECOLS: an error argument beats an out-of-range index (T-2273)": "Between two errors, ours
     keeps the first in argument order (UNVERIFIED)".
- VERIFIED so far (T-2273, gen-m6 cells listed in that entry): an error in any argument, the array or a later index,
  beats an earlier 0 or out-of-range index's `#VALUE!`.
- What the engine does (`chosen_indices` in
  crates/rustsheet-engine/src/functions/lookup_and_reference/choosecols_chooserows.rs): it walks the index
  arguments in order and each argument's elements row-major, and returns at once on the first element that is an error
  or does not coerce to a number (`cast_number_ref`'s `?`); an out-of-range index only sets a flag. So
  `CHOOSECOLS(A1:C1,"x",NA())` is `#VALUE!` and `CHOOSECOLS(A1:C1,1/0,NA())` is `#DIV/0!` in ours. The array
  argument is evaluated before any index, so an error array also beats an index error in ours.

## Acceptance criteria
- [ ] A probe case `probes/t4718-choose-index-error-order.case.json` is committed with excel-graph oracle values. For
      both functions it holds at least: two different errors in two index arguments, in both orders
      (`CHOOSECOLS(A1:C2,1/0,NA())`, `CHOOSECOLS(A1:C2,NA(),1/0)`); two different errors inside one array index, as
      a row and as a 2x2 array (`{#N/A,#DIV/0!}`, `{#DIV/0!,#N/A}`, `{1,#N/A;#DIV/0!,2}` and `{1,#DIV/0!;#N/A,2}`, to
      tell row-major from column-major); uncoercible text before and after an error (`"x"` then `NA()`, `NA()` then
      `"x"`, and both inside one array); an out-of-range index, then text, then an error; and an error array argument
      against an error index (`CHOOSECOLS(NA(),1/0)`, `CHOOSECOLS(1/0,NA())`). Wrap results in `ERROR.TYPE` or read
      them directly; both work for plain errors.
- [ ] Every probe cell passes, or each cell that does not is listed in a new functions-lookup task (`campaign:
      semantics` with a `## Visible-parity stage` section) filed with `make new-task` that names the case and its
      cells, and those known failures are recorded with `make eval-baseline`.
- [ ] lookup.md replaces both UNVERIFIED sentences with the observed order, citing the probe cells, and the
      `chosen_indices` doc comment cites the entry.
- [ ] Any engine change has a unit test that pins a probed result.
- [ ] `make check-fast` green; no new known-failure line in eval/baseline/ other than those owned as above; reviewers
      approve.

## Out of scope
- Index lists of references to other sheets, lifted CHOOSECOLS over arrays of references, and the out-of-range rule
  itself (VERIFIED, T-2273).
- TAKE, DROP, EXPAND and the other reshapers.
