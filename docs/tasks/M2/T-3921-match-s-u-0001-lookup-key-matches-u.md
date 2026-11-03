---
id: T-3921
title: "MATCH's U+0001 lookup key matches U+0524 and empty text like a wildcard (probes/t3584-match-fold-weightless R1)"
milestone: M2
lane: engine
status: todo
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
- [ ] A probe settles where U+0001 acts as a wildcard; the engine matches it, and t3584 Calc!R1's MATCH digits
      pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- `=` and COUNTIF's weightless equality (T-3904).
