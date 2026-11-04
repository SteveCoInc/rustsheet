---
id: T-4233
title: "SPEC-0024: charge the text bytes a whole-array reader compares (lookups, MATCH, UNIQUE, SORT, the *IF(S)) over a bound array of long texts"
milestone: M7
lane: engine
status: todo
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
