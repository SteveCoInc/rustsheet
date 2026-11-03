---
id: T-3868
title: "GROUPBY: correct the #NUM! option-order citation and a same-height test comment from T-3837"
milestone: M7
lane: functions-lookup
status: done
priority: P3
size: S
tier: sonnet
depends_on: [T-3837]
---

## Goal
The evidence cited for GROUPBY's "text values lose to a later option's error" rule names the cell
that holds it, and the unit test's comment describes its cases correctly.

## Context
Excel-fidelity review of #2860 (T-3837), MINOR findings on de3e461:
- `crates/rustsheet-engine/src/functions/lookup_and_reference/groupby.rs` (near line 514),
  `docs/excel-semantics/arrays.md` (near line 2175) and the T-3837 task file cite
  `GROUPBY(Data!D1:D12,"abc",LAMBDA(v,v),1,#NUM!)` as gen-m7 d03de506903f5a35 A11835. That cell
  holds another formula (`#REF!`). The quoted formula is gen-m7 ec5094d09c82f94d A6223 (Excel
  `#NUM!`); 0a91d5918d798930 A4759 (`GROUPBY(Data!B1:E12,"abc",LAMBDA(v,v),#NUM!)`, `#NUM!`)
  also supports it. Check both in the cases' `expected` values before citing them.
- `crates/rustsheet-engine/src/test/test_fn_groupby.rs`, test
  `scalar_inputs_and_the_order_of_their_errors`: the comment says "at the same height too", but
  `GROUPBY(Z1,B1:B3,_xleta.SUM,Z1)` has 1 key row against 3 value rows; only the
  `GROUPBY(Z1,Z4,..,,,,Z1)` row is at the same height.

## Acceptance criteria
- [ ] The three citations name ec5094d09c82f94d A6223 (and may add 0a91d5918d798930 A4759).
- [ ] The test comment says which rows are at the same height.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any behavior change.

## Outcome
The three citations (groupby.rs, arrays.md, T-3837 task file) now name gen-m7 ec5094d09c82f94d A6223
(oracle `#NUM!`; 0a91d5918d798930 A4759, also `#NUM!`, supports it). The test comment now says which
case is at the same height. No behavior change; eval delta none.
