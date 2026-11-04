---
id: T-4039
title: "Weigh U+200B after the whitespace and the C1 controls before the digits in the collation (probes/t3662-collation-ignorables*)"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel's `<`, `=`, `>`, the text criteria and XMATCH order the zero-width space U+200B and the C1 controls
U+0080 to U+009F as Excel does. U+200B sorts after every whitespace character and before `!`. The C1 controls
sort after every symbol and before the digits, in code-point order. Neither is equal to any other character.

## Context
- Evidence (excel-graph, 2026-11-04, T-3662): coercion.md "Format characters, U+200B and the C1 controls (T-3662)".
  `"a"&UNICHAR(8203)&"b"<"ab"` and `"a"&UNICHAR(129)&"b"<"ab"` are TRUE, U+200B is above CHAR(160), CHAR(9),
  CHAR(10), CHAR(13), U+2002 and U+3000 and below `!`, and U+0081/U+009F are above `+ = > ¤ § ° ± × ÷ • → ★ © ¬`
  and below `¼` and `∞`. Among them: U+200B < U+0081 < U+0085 < U+009F < the weightless characters (which tie
  with `ab` when they sit between `a` and `b`).
- Ours: `element` (crates/rustsheet-engine/src/functions/util/collation.rs) sends them to `other(c)`, a primary
  weight after the letters. Room exists: the whitespace weights end at `PUNCTUATION_BASE`, and `BULLET` sits just
  below `DIGIT_BASE`. U+200B after U+3000 (which is `full_width(single(SPACE))` today) needs a weight between the
  last whitespace weight and the first punctuation weight. Unprobed C1 units (U+0080, U+0082 to U+0084, U+0086 to
  U+009E) keep code-point order with the probed ones (record it as UNVERIFIED there).
- Owned cells:
  - probes/t3662-collation-ignorables (25): Calc!R2, R10:R12, S2, S10:S12, U2, U10:U12, V2, V10:V12, Y2, Y10:Y12, AA2, AA10:AA12, AB2
  - probes/t3662-collation-ignorables-prefix (23): Calc!C10, D10, E10, F10, G10, H10, I10, J10, K10, L10, M10:M13, N10:N13, P10:P13, R10
  - probes/t3662-collation-ignorables-sweep (4): Calc!K13:K14, L13:L14
- The weightless characters of the same cases are T-3904's.

## Acceptance criteria
- [x] The 52 cells above pass and leave eval/baseline/probes.txt; `make eval-check` shows 0 regressions (the
      collation hold-out probes/t3465-collation-holdout and probes/text-collation-order keep every cell).
- [x] Unit tests: U+200B between U+3000 and `!`, U+0081 between `•` and `0`, U+0081 < U+0085 < U+009F.
- [x] coercion.md's T-3662 section says "Implemented in" with this task for U+200B and the C1 controls.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The weightless characters and the trailing rule (T-3904).
- Exact MATCH, VLOOKUP and HLOOKUP, which count every unit (T-3445; they already pass these cells).

## Outcome
- New probe probes/t4039-collation-space-forms (322 cells, excel-graph, 2026-11-04): U+205F equals the space; then
  U+3000 < U+202F < CHAR(160) < CHAR(9) < U+2000 = U+2002 < U+2001 = U+2003 < U+2004 ... U+200A < U+200B < `!`; the
  C1 controls after `¬` and `•`, before `0`, by code point (U+0080, U+0081, U+0090, U+009E, U+009F).
- `element` (collation.rs) weighs U+205F as `SPACE`, U+202F at new `NARROW_NBSP` (between `SPACE` and `NBSP`),
  U+2000 to U+200B at new `SPACE_FORMS` (after CHAR(13), before the punctuation) and the C1 controls at new
  `C1_BASE` (after `BULLET`, a const assert keeps them below the digits). UNVERIFIED and recorded: U+2000 to
  U+200A against CHAR(10) to CHAR(13), and the unprobed C1 units.
- Landed in T-3904's PR: the -ignorables Calc!AB ranks need both (see T-3904's Outcome).
- Unit test `t4039_space_forms_zero_width_space_and_c1_controls`; the new case passes all 322 cells.
- Eval delta: see T-3904 (one PR); the 52 owned cells pass.
