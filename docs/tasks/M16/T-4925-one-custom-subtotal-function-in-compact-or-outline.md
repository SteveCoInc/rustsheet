---
id: T-4925
title: "One custom subtotal function in compact or outline form sits on the item's own line"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
In compact and outline form, a row field with exactly one custom subtotal function shows that subtotal on the item's own line, as a default subtotal shows, with no `<item> <Function>` line after the children.

## Context
Found by T-4132's first gen-pivot batch (excel-graph, 2026-11-07; seed 0, rustsheet-gen --milestone M16). Each case's description names its sheets' covering-array levels.

Excel puts the one custom subtotal at the top: gen-pivot/640780d7b6f4ad53 P05 (outline, custom Sum on Channel) shows `Web shown` with its values on L8 and no `Web Sum` line, which the engine adds after the children (L11); gen-pivot/36996978427905b2 P01 (outline, custom Var, Values first on rows), gen-pivot/f9e7edbb5b74011a P09 and gen-pivot/ec473260bb2065b3 P05 (compact, custom Var and Min) are the same. With two or more custom functions every gen-pivot pivot has one line per function after the children (e.g. gen-pivot/2251b0f5e5a256e1 P04), as probes/t4168-pivot-layout-options F2 shows for eleven; in tabular form one function has its own line too (gen-pivot/2251b0f5e5a256e1 P10). The engine's reading for plain outline form was UNVERIFIED in pivots.md "Layout lines" (T-4699).

The rule and its evidence: docs/excel-semantics/pivots.md "What the gen-pivot batch found (T-4132)". Code: crates/rustsheet-engine/src/pivot/layout/lines.rs (`AxisPlan::lines`). The engine pivot code is another stream's: this task is that stream's to implement.

Split: one gen-pivot sheet can show several causes, so these spans can overlap those of the other T-4132 owner tasks (T-4807, T-4914, T-4915, T-4916, T-4917, T-4918, T-4919, T-4920, T-4921, T-4922, T-4924, T-4926, T-4927); each owns its own cause there.

Owner of these visibly-off cells (decision 0245), 752 values and texts on 4 sheets (a sheet's span is the bounding box of the cells this cause explains; other causes of the same sheet have their own tasks):
- gen-pivot/36996978427905b2 P01!L7:O29
- gen-pivot/640780d7b6f4ad53 P05!L8:V13
- gen-pivot/ec473260bb2065b3 P05!L8:P249
- gen-pivot/f9e7edbb5b74011a P09!L7:M23
- excel-graph fill, 2026-11-13, probe request val11 (#4128). Owns: probes/t4925-pivot-one-custom-subtotal (every visibly-off cell of each). Run `make eval-show CASE=<case>` for the cells, and score them against the request's frozen predictions where it has them.

## Acceptance criteria
- [ ] The cells above pass (`rustsheet-eval run --suite gen-pivot`), and no other case regresses (`make eval-baseline`).
- [ ] The pivots.md entry says the engine now matches, with the implementing code path.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
