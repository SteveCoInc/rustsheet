---
id: T-3905
title: "sigma_is_final gives the final sigma only at the end of text or before Excel's 30 ASCII separators (probes/t3820-lower-sigma-next-char)"
milestone: M2
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
LOWER, PROPER and TEXTBEFORE/TEXTAFTER's case-insensitive mode lower a capital `Σ` to the final `ς` exactly where
Excel does: at the end of the text or before one of 30 ASCII characters, and to `σ` before anything else.

## Context
- Evidence: docs/excel-semantics/text.md, the T-1747 final-sigma rule as rewritten by T-3820
  (probes/t3820-lower-sigma-next-char, excel-graph 2026-11-03). The 30 characters that give `ς`: U+0009, U+000A,
  U+000D, space and ``! " # $ % & ' + , - . / : ; < = > ? @ [ \ ] _ { | }``. Every other BMP code point, a
  supplementary letter and an emoji give `σ` (Calc!E1:F256 is the whole-BMP sweep for LOWER and PROPER).
- Code: `sigma_is_final` (crates/rustsheet-engine/src/functions/text/string_format.rs) tests
  `char::is_alphanumeric`, U+0300-U+036F, U+2019 and U+200B today; it also drives `fold_for_before_after`
  (functions/text/casefold.rs) and `proper_case`. Replace the test with the fixed list (one definition).
- Failing cells: probes/t3820-lower-sigma-next-char Calc!A1:C50, E1:F256 (the 216 listed in eval/baseline/probes.txt).
- Check probes/case-mapping-expanding and probes/regexreplace-final-sigma keep passing (REGEXREPLACE's `\L` uses the
  same rule).

## Acceptance criteria
- [ ] Every cell of probes/t3820-lower-sigma-next-char passes and leaves eval/baseline/probes.txt; 0 regressions.
- [ ] A unit test covers one character of each class (end, listed separator, `(`, U+00A0, a mark, U+2018, an emoji).
- [ ] text.md's entry says "Implemented in" with this task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- UPPER and LOWER's mapping table (T-3583); PROPER's word starts (T-3657).
