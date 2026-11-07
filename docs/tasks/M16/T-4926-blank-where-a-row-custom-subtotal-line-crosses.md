---
id: T-4926
title: "Blank where a row custom subtotal line crosses a column custom subtotal"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Where a row field's custom subtotal line crosses a column field's custom subtotal column, the cell is blank, as Excel leaves it.

## Context
Found by T-4132's first gen-pivot batch (excel-graph, 2026-11-07; seed 0, rustsheet-gen --milestone M16). Each case's description names its sheets' covering-array levels.

gen-pivot/2251b0f5e5a256e1 P10 (tabular, custom Count and Product on Product rows, custom StdDev and StdDevp on Rep columns): `Silver Count` meets `Ann StdDev` blank in Excel and 1 in the engine (P9:U16); gen-pivot/ec473260bb2065b3 P06 (`B Count` x `Store StdDev`: Excel blank, engine 2), gen-pivot/ec473260bb2065b3 P09 and gen-pivot/2251b0f5e5a256e1 P04 are the same. Where a custom line meets a default subtotal column, or an item column, Excel computes it.

The rule and its evidence: docs/excel-semantics/pivots.md "What the gen-pivot batch found (T-4132)". Code: crates/rustsheet-engine/src/pivot/result.rs, crates/rustsheet-engine/src/pivot/layout/render.rs. The engine pivot code is another stream's: this task is that stream's to implement.

Split: one gen-pivot sheet can show several causes, so these spans can overlap those of the other T-4132 owner tasks (T-4807, T-4914, T-4915, T-4916, T-4917, T-4918, T-4919, T-4920, T-4921, T-4922, T-4924, T-4925, T-4927); each owns its own cause there.

Owner of these visibly-off cells (decision 0245), 114 values and texts on 4 sheets (a sheet's span is the bounding box of the cells this cause explains; other causes of the same sheet have their own tasks):
- gen-pivot/2251b0f5e5a256e1 P04!Q13:W66
- gen-pivot/2251b0f5e5a256e1 P10!P9:U16
- gen-pivot/ec473260bb2065b3 P06!Q9:V39
- gen-pivot/ec473260bb2065b3 P09!N12:AW23

## Acceptance criteria
- [ ] The cells above pass (`rustsheet-eval run --suite gen-pivot`), and no other case regresses (`make eval-baseline`).
- [ ] The pivots.md entry says the engine now matches, with the implementing code path.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
