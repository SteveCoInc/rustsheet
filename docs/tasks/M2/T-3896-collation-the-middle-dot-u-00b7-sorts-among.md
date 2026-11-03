---
id: T-3896
title: "Collation: the middle dot U+00B7 sorts among the punctuation, not after the letters (T-3465 sweep)"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`<`, `=`, criteria and XMATCH place the middle dot `·` (U+00B7) where Excel does, before the letters.

## Context
- probes/t3465-collation-holdout row 110 (excel-graph 2026-11-03): `"·¯÷¯¹" < "Æ"` in Excel; the engine
  says greater because it sends `·` to a weight after every letter (`cmp_text_excel("·","a")` is Greater,
  while `¯`, `÷` and `¹` sort before `a`). No probe has ranked `·`: the Latin-1 probes of T-1590/T-1595
  (coercion.md "Collation equality in lookups and criteria; unprobed characters (T-1590)") list the other
  Latin-1 symbols but not U+00B7.
- Code: `element` in crates/rustsheet-engine/src/functions/util/collation.rs and collation_tables.rs.
- Owned cells: probes/t3465-collation-holdout C110, D110

## Acceptance criteria
- [x] Probe (excel-probe skill) `·` against the Latin-1 symbol ladder of the T-1595 probe (its rank, equality
      with `""` and `"."`, and whether it is skipped like the hyphen) before changing the engine.
- [x] The engine places `·` as probed; C110 and D110 pass; coercion.md records the rank.
- [x] `make check` green; reviewers approve.

## Out of scope
- Other unprobed symbol blocks (T-3559).

## Outcome
Probed `·` (probes/t3896-collation-middle-dot, 272 cells): rank right after `µ`, before `…`; not skipped, not equal to `""`. Added `'·'` to `PUNCTUATION` in collation.rs; recorded in coercion.md. Holdout C110/D110 now pass.
