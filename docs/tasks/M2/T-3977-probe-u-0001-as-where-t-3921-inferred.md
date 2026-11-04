---
id: T-3977
title: "Probe U+0001 as * where T-3921 inferred it: COUNTIF <>, COUNTIFS, SUMIFS, D-functions, XLOOKUP mode 2, escaped ~*"
milestone: M2
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for the paths where T-3921 (#3028) applies the U+0001-as-`*` rule without a probe, so that
lookup.md "U+0001 is a wildcard" states for each one whether the rule holds.

## Context
- T-3921 probed probes/t3921-match-u0001-wildcard (MATCH, VLOOKUP, HLOOKUP, XMATCH modes 0 and 2, COUNTIF, SEARCH). Its
  rule: every U+0001 is `*` when the pattern starts with U+0001 and holds no `*`, and always in SEARCH.
- `WildcardPattern::new` (crates/rustsheet-engine/src/functions/criteria.rs) is shared with paths the probe never ran,
  which the T-3921 excel-fidelity review flagged as MINOR:
  - COUNTIF's `<>` branch (`"<>"&CHAR(1)&"?"`) and the `=` prefix;
  - COUNTIFS, SUMIFS and AVERAGEIFS;
  - the D-functions (DCOUNT, DSUM), whose criteria are database-flavoured;
  - XLOOKUP's match mode 2 (`wildcard_text_matcher`).
- The rule's "holds no `*`" is read after the `~` escapes, so `CHAR(1)&"~*"` still turns U+0001 into `*`. Probe it and
  `CHAR(1)&"~?"`.

## Acceptance criteria
- [ ] One probe case covering each path above, with the same D1:D6 data as t3921, filled from excel-graph.
- [ ] lookup.md records the result for each path. A path where the engine differs gets its owner task, or the fix
      in this PR if it is small.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
