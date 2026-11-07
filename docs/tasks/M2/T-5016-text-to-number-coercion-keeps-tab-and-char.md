---
id: T-5016
title: "Text-to-number coercion keeps TAB and CHAR(160) padding: `0+CHAR(9)&\"-3\"` is #VALUE!"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Text-to-number coercion (`+0`, and every reader of `Model::cast_number`) refuses number text padded with a TAB or
CHAR(160), as Excel does: `0+(CHAR(9)&"-3")` and `0+(CHAR(160)&"0")` are `#VALUE!`. Only the space is padding.

## Context
- Found by T-3464's typed-input batch (docs/excel-semantics/coercion.md "Number text padded with a TAB or
  CHAR(160) is not a number (T-3464)"). In Excel every entry the batch padded with a TAB or CHAR(160) stays text,
  and `=A<n>+0` is `#VALUE!` for all 22 of them. The engine keeps them as text too (column A passes), but it reads 5
  of them as numbers. Those 5 are a signed or unsigned plain number with nothing else:
  `\t-3`, `CHAR(160)0`, `\t\t-701.80`, `-92598\t\t`, `-2.88\t`.
- Cause: `Model::cast_number_in` (crates/rustsheet-engine/src/cast.rs) calls `parse_excel_number(s.trim())`.
  `str::trim` strips every Unicode White_Space character, which includes TAB and U+00A0. The formatted path,
  `parse_formatted_number`, already trims only `' '` (probes/t3334-criteria-operand-forms H47, H51, H55, H59).
  The other forms (`\t$55.50`, `401.2%\t`, `\t2,428,397`) are refused already.
- Before you change the trim, check which other characters `trim()` strips that some case relies on (CHAR(10),
  CHAR(13), U+3000, ...). Run `make eval`. If a case passes only because of `trim()`, keep that character and cite
  the case.
- Owner of these visibly-off cells (decision 0245):
  - gen-m2/7d9ef4824d1c5286 Sheet1!B47, Sheet1!B211
  - gen-m2/361fd843151f1560 Sheet1!B77, Sheet1!B219, Sheet1!B221

## Acceptance criteria
- [ ] The cells above pass, and no other case regresses (`make eval-baseline`).
- [ ] A unit test pins `0+` of TAB- and CHAR(160)-padded plain number text as `#VALUE!` and space padding as a number.
- [ ] coercion.md's entry says what the engine now does ("Ours").
- [ ] `make check` green; reviewers approve.

## Out of scope
- Typed entry (`set_user_input`), which already keeps these as text.
