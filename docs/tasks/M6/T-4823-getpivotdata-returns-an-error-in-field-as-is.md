---
id: T-4823
title: "GETPIVOTDATA returns an error in field as is, and a referenced error as data_field is #REF!"
milestone: M6
lane: functions-lookup
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
GETPIVOTDATA gives Excel's error when an argument is an error. Two rules decide 65 gen-m6 cells (T-4006, excel-graph,
2026-11-07; each case's Calc!A1 is a valid control query Excel answers with 6.75, so the pivot was live):
1. An error in `field`, typed or read from a cell, is the result as is (`"Units",Pivot!$H$3,#NAME?,2` is `#NAME?`;
   `Data!G1,Pivot!$H$3,Data!G9,"C"` with Data!G9 `#DIV/0!` is `#DIV/0!`). Ours is `#REF!`.
2. A *referenced* error as `data_field` is `#REF!` (`Data!G9,Pivot!$H$3,"Product",Data!G1` is `#REF!`), while a
   typed error literal is the result (`#NULL!,Pivot!$H$3,2,"C"` is `#NULL!`, which already passes). Ours returns
   the referenced error.

## Context
- Code: `fn_getpivotdata` and `getpivotdata_on` in
  crates/rustsheet-engine/src/functions/lookup_and_reference/getpivotdata.rs. Today an error `data_field` returns
  at once, and a field that is not text is a miss (`#REF!`).
- The evidence, in the same cases, that fixes the order (docs/excel-semantics/web.md "GETPIVOTDATA error
  arguments (T-4006)"):
  - An error in `item` (typed or referenced) is `#REF!`, like any item miss (passing today), e.g.
    gen-m6/37eb2ebaf9386ade Calc!A298 `"Units",Pivot!$H$3,"Product",#DIV/0!`.
  - A field error beats an item error: gen-m6/2963992505456e99 Calc!A223 `#REF!,#NAME?` is `#REF!`, and A226
    `Data!G9,Data!G9` is `#DIV/0!`.
  - A field error beats a referenced data_field error: gen-m6/ccc550562ee5fd06 Calc!A16
    `Data!G9,Pivot!$H$3,Data!G9,"A"` is `#DIV/0!`.
  - A typed data_field error beats an item error: gen-m6/37eb2ebaf9386ade Calc!A88 `#NULL!,...,"Product",#VALUE!`
    is `#NULL!`.
  - Not separated: a typed data_field error against a field error (gen-m6/ccc550562ee5fd06 Calc!A13 has `#NULL!`
    in both). Keep the typed data_field error first and say so in a comment; a probe that separates them is welcome
    but not required.
- Failing cells, rule 1 (an error in `field`):
  - gen-m6/16b67941f34e0346 Calc!A94, A97, A160, A163, A226, A229, A292, A295, A358, A361, A412, A415, A478,
    A481, A544, A547
  - gen-m6/ccc550562ee5fd06 Calc!A79, A82, A145, A148
  - gen-m6/37eb2ebaf9386ade Calc!A274, A277
  - gen-m6/2963992505456e99 Calc!A175, A178, A181, A184, A187, A190, A193, A196, A199, A202, A205, A208, A211,
    A214, A217, A220, A226, A229, A232, A235, A238
- Failing cells, rule 2 (a referenced error as `data_field`):
  - gen-m6/16b67941f34e0346 Calc!A37, A565, A571, A577, A583, A589, A595
  - gen-m6/ccc550562ee5fd06 Calc!A4, A10, A22, A28
  - gen-m6/37eb2ebaf9386ade Calc!A43, A49, A55, A61, A67, A73, A79, A85, A91, A97, A103

## Acceptance criteria
- [ ] The 65 cells above pass, and the item-error cells and the typed data_field error cells still pass
      (`make eval-show CASE=gen-m6/<stem>.case.json`); `make eval-baseline` records it with 0 regressions.
- [ ] A unit or spec test covers each rule and the field-before-item order.
- [ ] The doc comment of `fn_getpivotdata` cites the rules and their evidence.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Every cell this task owns has Excel's error (decision 0083): these are wrong-error cells, so the visible stage is the
whole task.

## Out of scope
- Errors in `pivot_table` (typed literals were not generated: a `range` parameter is classified only as a reference).
- Separating a typed data_field error from a field error (see Context).

## Outcome
`fn_getpivotdata` now returns a non-referenced `data_field` error first (a typed error; computed errors such as
`NA()` unprobed, kept as the result), then a `pivot_table` error, then an error in any `field` as is, and a
*referenced* error as `data_field` falls to `#REF!`. Unit test `error_arguments_follow_the_t4006_order` plus the
updated `unknown_items_fields_repeats_and_odd_pairs_are_ref`. Eval: gen-m6 baseline -65 failing cells, 0 regressions.
