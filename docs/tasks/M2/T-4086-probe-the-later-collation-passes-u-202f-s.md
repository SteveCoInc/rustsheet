---
id: T-4086
title: "Probe the later collation passes: U+202F's accent weight, a mark after hyphen and a weightless character, width before small kana; GROUPBY/PIVOTBY grouping"
milestone: M2
lane: research
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel-graph probe cases settle the questions the excel-fidelity review of #3132 (T-4072) left open, and one the
T-4060 docs marked UNVERIFIED. The questions:
- which later pass separates U+202F from the space;
- where a mark after a hyphen or apostrophe followed by a weightless character weighs;
- whether GROUPBY and PIVOTBY apply `=`'s prefix rule.

## Context
- The review findings are on PR #3132. coercion.md "Format characters, U+200B and the C1 controls (T-3662)", the
  T-4060 bullets, marks the GROUPBY and PIVOTBY order UNVERIFIED.
- Method: the `excel-probe` skill. probes/t4086-collation-later-passes (M2) covers the passes;
  probes/t4086-groupby-weightless (M7, the subject's milestone) covers GROUPBY and PIVOTBY.

## Acceptance criteria
- [ ] Both cases are committed with their excel-graph sidecars.
- [ ] coercion.md records each finding with cells. Every cell the engine fails is owned by an engine task filed in
      the same PR (T-4087, T-4088).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The engine changes (T-4087, T-4088).
