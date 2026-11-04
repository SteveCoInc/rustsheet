---
id: T-4086
title: "Probe the later collation passes: U+202F's accent weight, a mark after hyphen and a weightless character, width before small kana; GROUPBY/PIVOTBY grouping"
milestone: M2
lane: research
status: done
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
- [x] Both cases are committed with their excel-graph sidecars.
- [x] coercion.md records each finding with cells. Every cell the engine fails is owned by an engine task filed in
      the same PR (T-4087, T-4088).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The engine changes (T-4087, T-4088).

## Outcome
- probes/t4086-collation-later-passes (M2, 53 cells) and probes/t4086-groupby-weightless (M7, 26 cells),
  excel-graph, 2026-11-04. Findings are in coercion.md, T-3662 section, "T-4086 settled":
  - U+202F differs from the space at the accent pass, not the variant pass (T-4072 chose the variant).
  - A weightless character ends a letter but not a hyphen or apostrophe: a mark after `-`&U+00AD still joins the
    hyphen.
  - New quirk: width and superscripts are weighed before small kana (`"Ａ"&ぁ > "A"&あ`), kana type after it.
  - GROUPBY and PIVOTBY group by `=`, prefix rule included, in pairwise order.
- New engine tasks: T-4087 (M2, 14 cells) and T-4088 (M7, 13 cells).
- Eval delta: fixed 0, regressions 0; 2 new cases (79 cells), 52 pass, 27 known failures, all owned.
