---
id: T-4917
title: "Round pivot date-time items to the second on refresh"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Pivot items of date-time source values are the values rounded to the whole second, as Excel's refreshed items are.

## Context
Found by T-4132's first gen-pivot batch (excel-graph, 2026-11-07; seed 0, rustsheet-gen --milestone M16). Each case's description names its sheets' covering-array levels.

A date-formatted source value with a time part off a whole second becomes an item rounded to the second: 366.99999999 shows as 367 (gen-pivot/2251b0f5e5a256e1 P05!N14), 60.999988426 as 60.99998842592593 (= 86399/86400 past day 60; gen-pivot/ca5c8fa99c13d747 P08!M8), 45382.99999999 as 45383 (gen-pivot/36996978427905b2 P02!L25). The engine keeps the source value. The whole-second unit is shown by the refreshed cache's `xsd:dateTime` items (`1900-02-28T23:59:59` in the gen-pivot/ca5c8fa99c13d747 P08 cache field `When`); the cells alone only bound it below about 2 ms. Whether values without a date format are rounded too is not probed (the gen-pivot pools put times only in the date field).

The rule and its evidence: docs/excel-semantics/pivots.md "What the gen-pivot batch found (T-4132)". Code: crates/rustsheet-engine/src/pivot/refresh.rs (date items). The engine pivot code is another stream's: this task is that stream's to implement.

Split: one gen-pivot sheet can show several causes, so these spans can overlap those of the other T-4132 owner tasks (T-4807, T-4914, T-4915, T-4916, T-4918, T-4919, T-4920, T-4921, T-4922, T-4924, T-4925, T-4926, T-4927); each owns its own cause there.

Owner of these visibly-off cells (decision 0245), 22 values and texts on 9 sheets (a sheet's span is the bounding box of the cells this cause explains; other causes of the same sheet have their own tasks):
- gen-pivot/2251b0f5e5a256e1 P05!N14:N22
- gen-pivot/36996978427905b2 P02!L25:L28
- gen-pivot/36996978427905b2 P06!N10:N17
- gen-pivot/462f10551d480f9d P05!M14:M37
- gen-pivot/ca5c8fa99c13d747 P08!M8
- gen-pivot/f9e7edbb5b74011a P03!M7:U7
- gen-pivot/f9e7edbb5b74011a P06!N29
- gen-pivot/f9e7edbb5b74011a P08!L9:L12
- gen-pivot/f9e7edbb5b74011a P12!L16

## Acceptance criteria
- [ ] The cells above pass (`rustsheet-eval run --suite gen-pivot`), and no other case regresses (`make eval-baseline`).
- [ ] The pivots.md entry says the engine now matches, with the implementing code path.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
