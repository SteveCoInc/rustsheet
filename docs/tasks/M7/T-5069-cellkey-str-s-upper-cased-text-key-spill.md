---
id: T-5069
title: "CellKey::Str's upper-cased text key (spill_functions.rs) follows the shared collation key"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Every text-equality key of the spill functions is the shared collation key: `cell_key` (functions/spill_functions.rs) no longer keys text by `to_uppercase()`, or is removed if nothing needs it.

## Context
T-4847's fidelity review (PR #4062): T-4847's Context names `CellKey::Str(s.to_uppercase())` in spill_functions.rs as a third text-equality key beside `collation_key` (`CellKey::Collated`, UNIQUE) and the lookups' `cmp_text_pairwise`; its doc comment still names a removed `array_nodes_equal`. Upper-casing neither equates ß with ss nor ignores hyphens as Excel's collation does (docs/excel-semantics/lookup.md, T-4847 entry; probes/t4847-xmatch-binary-text-collation). Find its callers first; where two rules disagree and no probe settles it, probe (excel-probe skill). One definition per concept (CLAUDE.md directive 3).

## Acceptance criteria
- [ ] `cell_key` keys text by `collation_key` (or is gone); a unit or eval case shows a ß/ss or hyphen pair keyed as Excel equates it, with oracle evidence for the caller.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
