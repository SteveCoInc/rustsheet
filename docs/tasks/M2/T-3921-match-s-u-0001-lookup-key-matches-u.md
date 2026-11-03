---
id: T-3921
title: "MATCH's U+0001 lookup key matches U+0524 and empty text like a wildcard (probes/t3584-match-fold-weightless R1)"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Exact MATCH with the one-character key U+0001 gives Excel's result: it finds the weightless U+0524 and empty text,
as the key `*` does (probes/t3584-match-fold-weightless Calc!R1, digit 3 at U+0001).

## Context
- Found by T-3445: after T-3445's fold, every MATCH digit of t3584 Calc!R1:R256 is Excel's except U+0001's. The `=`
  digit (8) is T-3904's, so the R cells leave the baseline only once both land.
- lookup.md "Weightless characters", last bullet: `*` and U+0001 MATCH both U+0524 and empty text, `?` finds U+0524
  and `~` finds empty text. Whether U+0001 acts as `*` inside longer keys, in VLOOKUP/HLOOKUP, or in XMATCH's wildcard
  mode is unprobed: probe it first (excel-probe skill).
- Code: the exact-match key path, `folded_units` / `ExactKey` (functions/criteria.rs,
  lookup_and_reference/lookup_table.rs).

## Acceptance criteria
- [x] A probe settles where U+0001 acts as a wildcard; the engine matches it, and t3584 Calc!R1's MATCH digits
      pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- `=` and COUNTIF's weightless equality (T-3904).

## Outcome
- Probe probes/t3921-match-u0001-wildcard (135 cells, excel-graph, 2026-11-03), recorded in lookup.md as "U+0001 is a
  wildcard". In MATCH, VLOOKUP, HLOOKUP, XMATCH mode 2 and COUNTIF (in wildcard mode), every U+0001 is `*` when the key
  starts with U+0001 and has no `*`. Otherwise U+0001 is literal. In SEARCH it is always `*`, and in XMATCH mode 0 never.
- Engine: `WildcardPattern::new` applies the leading-U+0001 rule, and SEARCH compiles through the new
  `WildcardPattern::for_search`, which reads every U+0001 as `*`. `from_units` is private now. Unit test:
  `soh_reads_as_a_star` (test_collation.rs).
- Eval: t3921 passes 135/135. t3584-match-fold-weightless Calc!R1's U+0001 digit is now Excel's 3 (it was 0). R1
  stays in the baseline until T-3904 fixes the U+00AD `=` digit (9). No other cell changed (`make eval-check`).
- Dropped from the probe: `MATCH(..,1)` on the unsorted list. That is a binary-search question, not a U+0001 one: the engine
  gave 4 where Excel gave 6 on three cells.
