---
id: T-4208
title: "HYPERLINK with a blank friendly_name: probe what value it returns, so LOGEST of it is #REF! as in gen-m9 fa25422d6f18c157 A75 (ours a computed blank, #VALUE!)"
milestone: M6
lane: functions-lookup
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
HYPERLINK whose friendly_name is a blank cell returns the value Excel returns, as the cell's value and as an
argument to other functions. Excel shows that this value is not the computed blank the engine passes on: in
gen-m9/fa25422d6f18c157 Calc!A75 the inner `LOGEST(HYPERLINK("#Data!A1",Data!D1),Data!B10:D12,OR(Data!A12))` is
`#REF!` in Excel, and `#VALUE!` in ours.

## Context
- Gap triage 2026-11-04c (`make eval` on origin/main 9a01e9b25). This is the one cell of the 343 unowned
  FORECAST.ETS-family cells that excel-graph can decide: Excel `#REF!`, not the family's `#N/A` (T-4207 takes the
  others). Formula: `=FORECAST.ETS.CONFINT(LOGEST(HYPERLINK("#Data!A1",Data!D1),Data!B10:D12,OR(Data!A12)),
  BESSELI(PERCENTILE.EXC(Data!A11,1),0),Data!E9:E10,COLUMN(Data!B1))`. Data!D1 is blank, B10:D12 is 3x3
  (B11 is the text "TRUE", D11 "ABC"), A12 is -0.0001, A11 is 1, E10 is "abc".
- Inference from existing evidence. On excel-graph, an error in the family's target_date comes first
  (statistics.md "FORECAST.ETS family on excel-graph: argument errors, text and lifting decide before the #N/A
  (T-2255)"), so `#REF!` is the LOGEST target's error. LINEST/LOGEST give `#REF!` for unequal rows (statistics.md
  "LINEST and LOGEST report error values first (T-2455)"), but a computed blank known_y is `#VALUE!` before that
  (`LINEST(IFERROR(B10,1),1/0)` with B10 blank, probes/t2455-transpose-single-value A68). So Excel's
  `HYPERLINK("#Data!A1",<blank>)` reaches LOGEST as something other than a computed blank, perhaps 0 or a blank
  reference. The probe has to establish which.
- Ours (scratch evaluation on 9a01e9b25): `HYPERLINK("#Sheet1!A1",D1)` with D1 blank shows Number 0 as a cell value,
  and `TYPE` of it is 1. Yet `LOGEST(HYPERLINK("#Sheet1!A1",D1),B10:D12,FALSE)` is `#VALUE!` while `LOGEST(D1,...)` and
  `LOGEST(0,...)` are `#REF!`. `fn_hyperlink` (crates/rustsheet-engine/src/functions/lookup_and_reference/mod.rs)
  returns friendly_name as it reads it (`EmptyCell`), and LINEST's reader
  (functions/statistical/linest.rs, `read_regression_inputs`) treats that as a computed blank.
- No probe covers HYPERLINK of a blank friendly_name. lookup.md has HYPERLINK entries only for an error
  link_location and its length. T-2865 lists `friendly_name: error` only.
- HYPERLINK is M6 (catalog prior 0.3). The visibly-off cell is in gen-m9.

## Acceptance criteria
- [ ] A probes case (excel-probe skill, excel-graph, canary-verified) with a blank friendly_name cell holds:
      `HYPERLINK("#A1",B1)` as a cell value; `TYPE`, `ISBLANK`, `ISNUMBER`, `ISTEXT`, `LEN` and `&"x"` of it;
      `LINEST`/`LOGEST(HYPERLINK("#A1",B1),{1;2})` and `(..., <3x3 range>)`; `IFERROR(B1,1)` controls on the same
      readers; `COUNT`, `COUNTA` and `SUM` of it; and HYPERLINK of an omitted friendly_name `HYPERLINK("#A1",)`.
      Wrap each error in `ERROR.TYPE`.
- [ ] lookup.md gains a "HYPERLINK: a blank friendly_name" entry citing the new cells and gen-m9/fa25422d6f18c157
      Calc!A75.
- [ ] The engine matches every new cell, and gen-m9/fa25422d6f18c157 Calc!A75's known-failure line disappears from
      eval/baseline/gen-m9.txt with no new ones (`make eval-baseline`, 0 regressions). If the fix is in a shared
      reader rather than in `fn_hyperlink`, it changes that one helper (directive 3).
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083/0114: the owned cell is visibly off (a wrong error: `#VALUE!` against Excel's `#REF!`), so this task is
in the campaign at stage `semantics`. The target is the cell's error matching Excel's.

## Out of scope
- The rest of the FORECAST.ETS family's `#N/A` cells (T-4207; decision 0266 rule 2).
- HYPERLINK's link attachment and the link_location rules (T-2277, T-2358).
