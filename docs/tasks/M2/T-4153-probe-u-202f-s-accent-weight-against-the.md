---
id: T-4153
title: "Probe U+202F's accent weight against the space with one combining mark (T-4087's UNVERIFIED NARROW_NBSP value)"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
An excel-graph probe pins where U+202F's accent weight sits among the space's accents with one combining mark, so
`NARROW_NBSP` (crates/rustsheet-engine/src/functions/util/collation.rs) is VERIFIED or corrected.

## Context
- T-4087 (#3142) gave U+202F the space's primary and the accent weight `NO_ACCENT + 1` (3), marked UNVERIFIED. The
  evidence fixes only the order: above the space, U+3000 and U+205F, and below CHAR(160) (probes/t4086-collation-
  later-passes Calc!B1, B46:B48; probes/t4039-collation-space-forms Calc!B2, B4, B16).
- The value matters against a space with a mark. The smallest `MARK_WEIGHT` is 3, so `" "&U+030D` (2 + 3 = 5) sorts
  above U+202F today, and so does every other `" "&<mark>` (excel-fidelity review of #3142).
- Probe with excel-probe (M2): U+202F against `" "&c` for the combining marks of the smallest weights (U+030D,
  U+0301, U+0300, U+0302, U+0323 and a few more), with `<`, `=` and `>`, and ranks against a ladder of them.

## Acceptance criteria
- [ ] A probe case `probes/t4153-narrow-nbsp-accent` with its excel-graph sidecar.
- [ ] coercion.md's T-4086 U+202F bullet states the weight VERIFIED, or an engine task owns the off cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing collation.rs (the engine task this may file).
