---
id: T-5017
title: "Typed entry: a doubled sign is a formula; a `**` exponent and a trailing-colon clock are numbers"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A typed entry reads three shapes as Excel does:
- a doubled sign makes it a formula: `+-122` is the formula `=+-122` (-122), and likewise `-+584.17` and `++78.2218`;
- `**` as the exponent marker: `39**9` is 39E9;
- a clock with no minutes: `10:` is 10:00 (0.416666…).

## Context
- Found by T-3464's typed-input batch (docs/excel-semantics/coercion.md "Typed entries: a doubled sign is a
  formula; a `**` exponent and `10:` are numbers (T-3464)"). The engine keeps all three shapes as text. For `+-122` the engine's
  A is `"+-122"`, its A+0 is `#VALUE!` and its ISTEXT is TRUE. For `39**9` and `10:` only A+0 or ISTEXT is off: the
  engine's `0+"10:"` already reads 10:00.
- The doubled-sign entries are formulas in Excel. The fill records only formula cells, and these four are the only
  column-A cells in the sidecars. The engine keeps them as text: `Model::formula_without_prefix` (model.rs) strips one
  sign and, through `typed_sign_text_is_number`, refuses a formula when the rest (`-122`) is a number. Excel seems to
  test the whole entry instead: one led by a sign is a formula unless all of it reads as a typed number (`  -3` and
  `-  92` stay constants). Fix it there, not in `parse_typed_number`.
- Probe first with the `excel-probe` skill. Put ISFORMULA and FORMULATEXT beside each typed cell. Cover `+-`, `-+`,
  `++` and `--` (not drawn by this batch) before a plain number, currency, a percent, E-notation and a mixed
  fraction, and `0+"+-1"` and VALUE of the same texts. Record the result in coercion.md.
- syntax.md "`**` in a typed number is `E`" covers typed formulas only: lexer/number.rs. A typed value (`39**9`,
  `10:`) goes through `parse_typed_number` (crates/rustsheet-engine/src/formatter/parse_formatted.rs), called from
  `set_user_input` (typed_input.rs, user_model/common.rs).
- Owner of these visibly-off cells (decision 0245):
  - gen-m2/7d9ef4824d1c5286 Sheet1!C172, Sheet1!A196, Sheet1!B196, Sheet1!C196
  - gen-m2/361fd843151f1560 Sheet1!B27, Sheet1!C27, Sheet1!A79, Sheet1!B79, Sheet1!C79, Sheet1!A87, Sheet1!B87,
    Sheet1!C87, Sheet1!A129, Sheet1!B129, Sheet1!C129

## Acceptance criteria
- [ ] A probe case filled on excel-graph covers the doubled-sign shapes above, including `--`, with ISFORMULA and
      FORMULATEXT of each typed cell.
- [ ] The cells above pass, and no other case regresses (`make eval-baseline`).
- [ ] coercion.md's entry says what the engine now does.
- [ ] `make check` green; reviewers approve.

## Out of scope
- TAB and CHAR(160) padding (T-5016).
