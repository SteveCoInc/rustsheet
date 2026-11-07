---
id: T-5017
title: "Typed entry reads a doubled sign, a `**` exponent and a trailing-colon clock as numbers"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A typed entry reads three shapes as numbers, as Excel does:
- a doubled sign: `+-122` is -122, `-+584.17` is -584.17, `++78.2218` is 78.2218;
- `**` as the exponent marker: `39**9` is 39E9;
- a clock with no minutes: `10:` is 10:00 (0.416666…).

## Context
- Found by T-3464's typed-input batch (docs/excel-semantics/coercion.md "Typed entries: a doubled sign, a `**`
  exponent and `10:` are numbers (T-3464)"). The engine keeps all three shapes as text. For `+-122` the engine's
  A is `"+-122"`, its A+0 is `#VALUE!` and its ISTEXT is TRUE. For `39**9` and `10:` only A+0 or ISTEXT is off: the
  engine's `0+"10:"` already reads 10:00.
- `--` before a number was not drawn by this batch (seed 0), so probe it, along with `+-`, `-+`, `++` and `--` before
  currency, percent, E-notation and a mixed fraction. Also probe `0+"+-1"` and VALUE of these shapes: coercion may
  differ from typed entry. Use the `excel-probe` skill and record the result in coercion.md.
- syntax.md "`**` in a typed number is `E`" covers typed formulas only: lexer/number.rs. A typed value goes through
  `parse_typed_number` (crates/rustsheet-engine/src/formatter/parse_formatted.rs), called from
  `set_user_input` (typed_input.rs, user_model/common.rs).
- Owner of these visibly-off cells (decision 0245):
  - gen-m2/7d9ef4824d1c5286 Sheet1!C172, Sheet1!A196, Sheet1!B196, Sheet1!C196
  - gen-m2/361fd843151f1560 Sheet1!B27, Sheet1!C27, Sheet1!A79, Sheet1!B79, Sheet1!C79, Sheet1!A87, Sheet1!B87,
    Sheet1!C87, Sheet1!A129, Sheet1!B129, Sheet1!C129

## Acceptance criteria
- [ ] A probe case filled on excel-graph covers the doubled-sign shapes above, including `--`.
- [ ] The cells above pass, and no other case regresses (`make eval-baseline`).
- [ ] coercion.md's entry says what the engine now does, and the number format the typed entry gets
      (from the oracle's `ENTERED` log).
- [ ] `make check` green; reviewers approve.

## Out of scope
- TAB and CHAR(160) padding (T-5016).
