---
id: T-4605
title: "arrays.md: two stale unprobed notes (INDEX of a LAMBDA out of range, IFERROR over blank FILTER/INDEX/VSTACK elements) now answered by T-1946 and T-2999"
milestone: M7
lane: functions-logic
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Two arrays.md sentences no longer call rules unprobed when committed Excel cells already answer them. Each one
points to the section that holds the rule and its cells.

## Context
- Gap triage 2026-11-06 (`make eval` on agent/tasks/refill-sonnet-1206, cut from origin/main). No unowned visibly-off
  cells were left, so this triage checked the semantics notes against the done tasks they cite.
1. arrays.md "An uncalled LAMBDA is one value", the "Ours" paragraph: "INDEX of a LAMBDA with an index of 2 is `#REF!`
   there, as for a scalar, which is unprobed (T-1946)". T-1946 is done. probes/map-later-array-error-blank A16:A23
   (`INDEX(LAMBDA(x,x),1)(5)`, index 2, -1, 0 and their ERROR.TYPE, plus `INDEX(..,1,2)`; excel-graph, 23 cells, all
   pass) answers it. arrays.md already states the rule in "MAP over single values": "INDEX of a LAMBDA: `INDEX(LAMBDA(x,x),1)(5)`
   is 5, an index of 2 is `#REF!` ... Evidence: probes/map-later-array-error-blank (... T-1946)".
2. arrays.md "GROUPBY and PIVOTBY": "Whether IFERROR reads blanks of other arrays (FILTER, INDEX lifted over blank
   cells, VSTACK) as 0, and what N and arithmetic do with a blank element, is unprobed (T-2999)". T-2999 is done.
   arrays.md "IFERROR, IFNA, N and arithmetic over blank array elements (T-2999)" holds the answer
   (probes/blank-elements-iferror-n-arith, 11 cells, excel-graph, all pass; ours already matched).
- Code (unchanged by this task): `index_select` (lookup_and_reference/index.rs), `if_error_like`
  (crates/rustsheet-engine/src/functions/logical/mod.rs).

## Acceptance criteria
- [ ] Sentence 1 cites probes/map-later-array-error-blank A17 (or points to "MAP over single values"),
      and the "unprobed (T-1946)" wording is gone.
- [ ] Sentence 2 points to "IFERROR, IFNA, N and arithmetic over blank array elements (T-2999)" and no longer says
      unprobed. Neither section restates the other's rule (one definition per concept).
- [ ] Docs only: nothing changes under crates/, eval/ or tools/. `make check` green; reviewers approve.

## Out of scope
- The other unprobed notes in arrays.md, which have open tasks (T-2463, T-2609, T-3087).
