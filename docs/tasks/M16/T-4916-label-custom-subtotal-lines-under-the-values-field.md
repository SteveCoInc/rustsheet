---
id: T-4916
title: "Label custom subtotal lines under the Values field on rows with the data field's source name"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Custom subtotal lines of a row field with the Values field inside it read `<item> <Function> of <source field>` per data field, as Excel labels them.

## Context
Found by T-4132's first gen-pivot batch (excel-graph, 2026-11-07; seed 0, rustsheet-gen --milestone M16). Each case's description names its sheets' covering-array levels.

With custom subtotal functions on an outer row field and the Values field innermost on rows, Excel labels each line with the function and the data field's source field: `Bronze Sum of Score`, `Bronze Sum of Units`, `Bronze Sum of Sales` (gen-pivot/2251b0f5e5a256e1 P04!L13:L15, outline, data fields Sum of Score, Min of Units, StdDevp of Sales); `Store Count of Score` (gen-pivot/f9e7edbb5b74011a P02!L18, tabular). The engine writes `Bronze Sum` and `Store Count`. Note the source field, not the data field's name (`Min of Units` gives `Sum of Units` under the custom Sum).

The rule and its evidence: docs/excel-semantics/pivots.md "What the gen-pivot batch found (T-4132)". Code: crates/rustsheet-engine/src/pivot/layout/render.rs (subtotal labels). The engine pivot code is another stream's: this task is that stream's to implement.

Split: one gen-pivot sheet can show several causes, so these spans can overlap those of the other T-4132 owner tasks (T-4807, T-4914, T-4915, T-4917, T-4918, T-4919, T-4920, T-4921, T-4922, T-4924, T-4925, T-4926, T-4927); each owns its own cause there.

Owner of these visibly-off cells (decision 0245), 106 values and texts on 4 sheets (a sheet's span is the bounding box of the cells this cause explains; other causes of the same sheet have their own tasks):
- gen-pivot/2251b0f5e5a256e1 P04!L13:L66
- gen-pivot/36996978427905b2 P04!L9:L30
- gen-pivot/a6d74990f3ae755e P09!L13:L102
- gen-pivot/f9e7edbb5b74011a P02!L8:M19

## Acceptance criteria
- [ ] The cells above pass (`rustsheet-eval run --suite gen-pivot`), and no other case regresses (`make eval-baseline`).
- [ ] The pivots.md entry says the engine now matches, with the implementing code path.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
