---
id: T-4153
title: "Probe U+202F's accent weight against the space with one combining mark (T-4087's UNVERIFIED NARROW_NBSP value)"
milestone: M2
lane: research
status: done
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
- [x] A probe case `probes/t4153-narrow-nbsp-accent` with its excel-graph sidecar.
- [x] coercion.md's T-4086 U+202F bullet states the weight VERIFIED, or an engine task owns the off cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing collation.rs beyond `NARROW_NBSP`'s value. Scope note: the probe pins the value exactly, and the fix is
  that one constant, so this PR sets it instead of filing an engine task for a one-line change.

## Outcome
- probes/t4153-narrow-nbsp-accent (M2, excel-graph, 2026-11-04) sets U+202F against the space with each of 15
  combining marks, with stacked U+030D (sums 8, 11, 14), and with 87, 88, 89 and 173 U+030D (sums that wrap to 7,
  10, 13 and 9). U+202F equals the space with 88 U+030D. **Its accent weight is 10**: VERIFIED.
- `NARROW_NBSP` (functions/util/collation.rs) is now 10 (was `NO_ACCENT + 1` = 3, which put U+202F below
  `" "&U+030D`); the doc comment cites the cells. coercion.md's T-4086 U+202F bullet records the finding.
- Unit test `t4153_narrow_nbsp_accent_weight`.
- Eval delta: fixed 0, regressions 0; 1 new case (106 cells), all pass with 10 (with 3, the cells against one, two and 88 U+030D did not).
