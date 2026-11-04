---
id: T-4072
title: "U+202F weighs as the space; a combining mark after a weightless character is a lone mark (probes/t4060-collation-open-choices)"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The engine matches Excel on two collation details that probes/t4060-collation-open-choices (T-4060) settled:
- U+202F has the space's first-pass weight, differing from it only at a later pass, above U+3000.
- A combining mark right after a weightless character does not attach to the letter before it. It weighs as a
  lone mark does.

## Context
- Evidence: coercion.md "Format characters, U+200B and the C1 controls (T-3662)", the T-4060 bullets
  (excel-graph, 2026-11-04).
- **U+202F.** `U+202F&"a"<" b"` is TRUE, as for U+3000 and U+205F; CHAR(160) and U+2000 to U+200B are FALSE
  (Calc!B1:B15). The engine gives U+202F its own primary, `NARROW_NBSP` (crates/rustsheet-engine/src/functions/
  util/collation.rs, T-4039). Fix: `SPACE` with a variant weight above `FULL_WIDTH`, so U+3000 < U+202F < CHAR(160)
  still holds (probes/t4039-collation-space-forms B4, B16, B2).
- **Marks.** Excel's `"a"&U+00AD&U+0301` (likewise with U+0524) is between `"a"` and `"á"` (Calc!B50:B57), and
  `U+00AD&U+0301` equals a lone U+0301 (B62). The engine drops the weightless character in `elements` before the
  accent pass, so the mark joins `a` and equals `á`. Fix: the weightless character ends the letter its marks
  would join, so the mark after it gets a letter unit of its own (`accent_units`' lone-mark rule,
  collation/accents.rs), with no primary, accent or position of its own when no mark follows (`"a"&U+00AD&"b"` stays
  equal to `"ab"`).
- Owned cells: probes/t4060-collation-open-choices Calc!B2, K2, B50, B51, B55, B56, B58, B59, B63.

## Acceptance criteria
- [ ] The 9 cells pass and leave eval/baseline/probes.txt. `make eval-check` shows 0 regressions: the t3662, t4039,
      t3584, t3154 accent and t3465 hold-out cases keep every cell.
- [ ] Unit tests for both rules.
- [ ] coercion.md's T-4060 bullets say "Implemented in" with this task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- GROUPBY and PIVOTBY's text order (the total `cmp_text`; whether they apply the prefix rule is unprobed).
