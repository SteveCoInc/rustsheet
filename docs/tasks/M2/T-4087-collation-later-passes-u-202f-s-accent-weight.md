---
id: T-4087
title: "Collation later passes: U+202F's accent weight, a mark after a skipped and a weightless character joins the skipped one, width and superscripts before small kana (probes/t4086-collation-later-passes)"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The collation matches Excel on three later-pass details that probes/t4086-collation-later-passes (T-4086) settled.
Each comes from the T-4072 review.
- U+202F differs from the space at the accent pass, not the variant pass.
- A combining mark after a hyphen or apostrophe and a weightless character still joins the hyphen or apostrophe.
- A full-width form's width and a superscript are weighed before the small-kana pass. A kana's type
  (half-width, katakana, hiragana) is still weighed after it.

## Context
- Evidence: coercion.md "Format characters, U+200B and the C1 controls (T-3662)", the T-4086 bullets (excel-graph,
  2026-11-04).
- **U+202F.** `U+202F&"a"<" "&"á"` is FALSE: the accent pass already decides it. Likewise `U+202F&"a"<U+3000&"á"`
  and `U+202F&ぁ<" "&あ` are FALSE (Calc!B1, C1, F1). Today `NARROW_NBSP` is a variant weight
  (crates/rustsheet-engine/src/functions/util/collation.rs, T-4072). Fix: give U+202F the space's primary with an
  accent weight above `NO_ACCENT`; its value is unprobed. U+3000 < U+202F < CHAR(160) must still hold
  (probes/t4039-collation-space-forms).
- **Mark after a skipped and a weightless character.** `"a-"&U+00AD&U+0301 = "a-"&U+0301` and
  `"a'"&U+00AD&U+0301 = "a'"&U+0301` are TRUE (Calc!B10:B22). Since T-4072, `Element::Weightless` ends the skipped
  character's run in `skipped` (collation.rs) and clears `after_skipped` in `accent_units` (collation/accents.rs).
  Fix: let a weightless character end only a letter. In `skipped`, the marks after a weightless character still
  add to the skipped character; in `accent_units`, keep `after_skipped`.
- **Width before small kana.** `Ａ&ぁ > A&あ`, `¹&ぁ > 1&あ` and `U+3000&ぁ > " "&あ` are TRUE, while `ア&ぁ < あ&あ`
  is TRUE (Calc!B30:B38, F2). Today `variants` weighs width, superscript and kana type in one pass after
  `kana_weights`. Fix: split it into a width/superscript pass before `kana_weights` and a kana-type pass after it.
  probes/t3275-collation-width-kana and -pairs must keep every cell.
- Owned cells: probes/t4086-collation-later-passes (14): Calc!B1, C1, F1, F2, B10, B12, B17, B21, B22, B30, B31, B32, B33, B36

## Acceptance criteria
- [ ] Every probes/t4086-collation-later-passes cell passes and leaves eval/baseline/probes.txt. `make eval-check`
      shows 0 regressions; in particular, every t3275, t3154, t4039, t4060 and t3662 cell keeps passing.
- [ ] Unit tests for the three rules.
- [ ] coercion.md's T-4086 bullets say "Implemented in" with this task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- GROUPBY and PIVOTBY (T-4088).
